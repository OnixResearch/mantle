use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

use futures::StreamExt;
use serde::Deserialize;
use serde::Serialize;
use snix_store::path_info::PathInfo;
use snix_store::pathinfoservice::PathInfoService;
use snix_store::pathinfoservice::RedbPathInfoService;
use snix_store::pathinfoservice::RedbPathInfoServiceConfig;

use crate::errors::RunError;

pub const RECEIPT_BUNDLE_FORMAT: &str = "mantle-build-receipt-bundle-v1";
pub const RECEIPT_BUNDLE_VERSION: u32 = 1;
pub const RECEIPT_BUNDLE_NON_CLAIM: &str =
    "receipt bundle evidence proves only matched receipt, attestation, graph, and trust-basis facts";
pub const MAX_RECEIPT_RECORDS: usize = 65_536;
pub const MAX_RECEIPT_FIELD_BYTES: usize = 4_096;
pub const MAX_RECEIPT_METADATA_ENTRIES: usize = 128;
pub const MAX_RECEIPT_METADATA_BYTES: usize = 16_384;

const RECORD_SPEC_SEPARATOR: char = ':';
const SEMANTIC_EDGE_SEPARATOR: &str = "->";
const TRUST_WINDOW_START_UNIX_S: u64 = 0;
const RECEIPT_STATE_DIR: &str = "receipt-bundles";
const RECEIPT_RECORDS_DIR: &str = "records";
const SOURCE_STATE_DIR: &str = "source-bundles";
const SOURCE_RECORDS_DIR: &str = "records";
const PATHINFO_DB_FILE: &str = "pathinfo.redb";
const SEMANTIC_GRAPH_FILE: &str = "semantic-graph.json";
const TEMP_FILE_EXTENSION: &str = "tmp";
const BLAKE3_DIGEST_PREFIX: &str = "blake3:";
const SHA256_DIGEST_PREFIX: &str = "sha256:";
const DIGEST_HEX_CHARS: usize = 64;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ReceiptRecordKind {
    SourceRef,
    ActionRef,
    SandboxReport,
    NetworkPolicy,
    ReferenceScan,
    OutputRef,
    ArtifactAttestation,
    ClosureAttestation,
    SemanticGraphEdge,
    TrustBasis,
    Signature,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ClaimStrengthRequest {
    Diagnostic,
    StrongActionCorrectness,
    ReleaseFacing,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReceiptRecord {
    pub kind: ReceiptRecordKind,
    pub identity: String,
    pub digest: String,
    pub metadata: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrustSnapshot {
    pub policy_hash: String,
    pub public_key_digests: Vec<String>,
    pub revocation_ref: Option<String>,
    pub valid_after_unix_s: u64,
    pub valid_before_unix_s: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReceiptBundle {
    pub format: String,
    pub version: u32,
    pub store_prefix: String,
    pub claim_strength: ClaimStrengthRequest,
    pub trust_snapshot: TrustSnapshot,
    pub records: Vec<ReceiptRecord>,
    pub bundle_blake3: String,
    pub non_claim: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ReceiptRecordSummary {
    pub kind: ReceiptRecordKind,
    pub identity: String,
    pub digest: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ReceiptBundleReport {
    pub format: &'static str,
    pub store_prefix: String,
    pub claim_strength: ClaimStrengthRequest,
    pub evidence_complete: bool,
    pub missing_evidence: Vec<ReceiptRecordKind>,
    pub output_identities: Vec<String>,
    pub records: Vec<ReceiptRecordSummary>,
    pub record_count: u32,
    pub bundle_blake3: String,
    pub non_claim: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ReceiptOutputMatch {
    pub identity: String,
    pub record_digest: String,
    pub local_digest: String,
    pub nar_sha256: String,
    pub nar_size_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ReceiptVerifyReport {
    pub format: &'static str,
    pub store_prefix: String,
    pub policy_hash: String,
    pub expected_policy_hash: String,
    pub policy_hash_matched: bool,
    pub valid_at_unix_s: u64,
    pub trust_window_valid: bool,
    pub public_key_digests: Vec<String>,
    pub revocation_ref: Option<String>,
    pub bundle_blake3: String,
    pub evidence_complete: bool,
    pub missing_evidence: Vec<ReceiptRecordKind>,
    pub output_matches: Vec<ReceiptOutputMatch>,
    pub non_claim: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ReceiptImportReport {
    pub imported: bool,
    pub idempotent: bool,
    pub bundle_imported: bool,
    pub graph_imported: bool,
    pub graph_nodes_imported: u32,
    pub graph_edges_imported: u32,
    pub bundle_blake3: String,
    pub non_claim: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrustVerificationContext {
    pub expected_policy_hash: String,
    pub valid_at_unix_s: u64,
    pub expected_revocation_ref: Option<String>,
    pub revoked_public_key_digests: BTreeSet<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReceiptRecordSpec {
    pub kind: ReceiptRecordKind,
    pub identity: String,
    pub digest: String,
}

impl std::str::FromStr for ReceiptRecordKind {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "source-ref" => Ok(Self::SourceRef),
            "action-ref" => Ok(Self::ActionRef),
            "sandbox-report" => Ok(Self::SandboxReport),
            "network-policy" => Ok(Self::NetworkPolicy),
            "reference-scan" => Ok(Self::ReferenceScan),
            "output-ref" => Ok(Self::OutputRef),
            "artifact-attestation" => Ok(Self::ArtifactAttestation),
            "closure-attestation" => Ok(Self::ClosureAttestation),
            "semantic-graph-edge" => Ok(Self::SemanticGraphEdge),
            "trust-basis" => Ok(Self::TrustBasis),
            "signature" => Ok(Self::Signature),
            other => Err(format!("unsupported receipt record kind '{other}'")),
        }
    }
}

pub fn parse_receipt_record_spec(raw: &str) -> Result<ReceiptRecordSpec, RunError> {
    let parts = raw.splitn(3, RECORD_SPEC_SEPARATOR).collect::<Vec<_>>();
    if parts.len() != 3 {
        return Err(RunError::Internal(format!("receipt record spec must be kind:identity:digest, got '{raw}'")));
    }
    let kind = parts[0].parse::<ReceiptRecordKind>().map_err(RunError::Internal)?;
    validate_field("identity", parts[1])?;
    validate_digest(parts[2])?;
    Ok(ReceiptRecordSpec {
        kind,
        identity: parts[1].to_string(),
        digest: parts[2].to_string(),
    })
}

pub fn build_receipt_bundle(
    specs: &[ReceiptRecordSpec],
    store_prefix: &str,
    policy_hash: &str,
    claim_strength: ClaimStrengthRequest,
) -> Result<ReceiptBundle, RunError> {
    let records = specs.iter().map(record_from_spec).collect::<Vec<_>>();
    build_receipt_bundle_from_records(records, store_prefix, policy_hash, claim_strength)
}

pub fn build_receipt_bundle_from_records(
    records: Vec<ReceiptRecord>,
    store_prefix: &str,
    policy_hash: &str,
    claim_strength: ClaimStrengthRequest,
) -> Result<ReceiptBundle, RunError> {
    if records.is_empty() {
        return Err(RunError::Internal("receipt bundle requires at least one evidence record".to_string()));
    }
    if records.len() > MAX_RECEIPT_RECORDS {
        return Err(RunError::Internal(format!("receipt record count exceeds {MAX_RECEIPT_RECORDS}")));
    }
    validate_field("policy_hash", policy_hash)?;
    if !store_prefix.starts_with('/') {
        return Err(RunError::Internal(format!("store prefix must be absolute: {store_prefix}")));
    }
    let records = normalize_receipt_records(records)?;
    let mut bundle = ReceiptBundle {
        format: RECEIPT_BUNDLE_FORMAT.to_string(),
        version: RECEIPT_BUNDLE_VERSION,
        store_prefix: store_prefix.to_string(),
        claim_strength,
        trust_snapshot: TrustSnapshot {
            policy_hash: policy_hash.to_string(),
            public_key_digests: Vec::new(),
            revocation_ref: None,
            valid_after_unix_s: 0,
            valid_before_unix_s: u64::MAX,
        },
        records,
        bundle_blake3: String::new(),
        non_claim: RECEIPT_BUNDLE_NON_CLAIM.to_string(),
    };
    bundle.bundle_blake3 = digest_bundle_without_digest(&bundle)?;
    Ok(bundle)
}

pub fn validate_receipt_bundle(bundle: &ReceiptBundle) -> Result<ReceiptBundleReport, RunError> {
    if bundle.format != RECEIPT_BUNDLE_FORMAT {
        return Err(RunError::Internal(format!("unsupported receipt bundle format {}", bundle.format)));
    }
    if bundle.version != RECEIPT_BUNDLE_VERSION {
        return Err(RunError::Internal(format!("unsupported receipt bundle version {}", bundle.version)));
    }
    if !bundle.store_prefix.starts_with('/') {
        return Err(RunError::Internal(format!(
            "receipt bundle store prefix is not absolute: {}",
            bundle.store_prefix
        )));
    }
    validate_trust_snapshot_shape(&bundle.trust_snapshot)?;
    reject_duplicate_records(&bundle.records)?;
    let expected = digest_bundle_without_digest(bundle)?;
    if expected != bundle.bundle_blake3 {
        return Err(RunError::Internal("receipt bundle digest mismatch".to_string()));
    }
    let missing_evidence = classify_missing_evidence(bundle);
    Ok(ReceiptBundleReport {
        format: RECEIPT_BUNDLE_FORMAT,
        store_prefix: bundle.store_prefix.clone(),
        claim_strength: bundle.claim_strength.clone(),
        evidence_complete: missing_evidence.is_empty(),
        missing_evidence,
        output_identities: output_identities(bundle),
        records: bundle.records.iter().map(record_summary).collect(),
        record_count: checked_u32(bundle.records.len(), "receipt record count")?,
        bundle_blake3: bundle.bundle_blake3.clone(),
        non_claim: RECEIPT_BUNDLE_NON_CLAIM,
    })
}

pub fn write_receipt_bundle(path: &Path, bundle: &ReceiptBundle) -> Result<(), RunError> {
    validate_receipt_bundle(bundle)?;
    let rendered = serde_json::to_string_pretty(bundle)
        .map_err(|err| RunError::Internal(format!("serializing receipt bundle: {err}")))?;
    fs::write(path, format!("{rendered}\n"))
        .map_err(|err| RunError::Internal(format!("writing receipt bundle {}: {err}", path.display())))
}

pub fn read_receipt_bundle(path: &Path) -> Result<ReceiptBundle, RunError> {
    let bytes = fs::read(path)
        .map_err(|err| RunError::Internal(format!("reading receipt bundle {}: {err}", path.display())))?;
    let bundle = serde_json::from_slice::<ReceiptBundle>(&bytes)
        .map_err(|err| RunError::Internal(format!("parsing receipt bundle {}: {err}", path.display())))?;
    validate_receipt_bundle(&bundle)?;
    Ok(bundle)
}

pub fn verify_receipt_bundle_against_state(
    bundle: &ReceiptBundle,
    state_dir: &Path,
    store_prefix: &str,
    expected_policy_hash: &str,
    outputs: &[String],
) -> Result<ReceiptVerifyReport, RunError> {
    let context =
        TrustVerificationContext::new(expected_policy_hash.to_string(), TRUST_WINDOW_START_UNIX_S, None, Vec::new());
    verify_receipt_bundle_against_state_with_context(bundle, state_dir, store_prefix, &context, outputs)
}

pub fn verify_receipt_bundle_against_state_with_context(
    bundle: &ReceiptBundle,
    state_dir: &Path,
    store_prefix: &str,
    context: &TrustVerificationContext,
    outputs: &[String],
) -> Result<ReceiptVerifyReport, RunError> {
    let bundle_report = validate_receipt_bundle(bundle)?;
    if outputs.is_empty() {
        return Err(RunError::Internal("receipt output verification requires at least one --output".to_string()));
    }
    let trust = validate_trust_snapshot(bundle, store_prefix, context)?;
    let local_output_records = local_output_records_for_outputs(state_dir, store_prefix, outputs)?;
    let output_matches = verify_output_records(bundle, &local_output_records, outputs)?;
    Ok(ReceiptVerifyReport {
        format: RECEIPT_BUNDLE_FORMAT,
        store_prefix: bundle.store_prefix.clone(),
        policy_hash: bundle.trust_snapshot.policy_hash.clone(),
        expected_policy_hash: context.expected_policy_hash.clone(),
        policy_hash_matched: true,
        valid_at_unix_s: context.valid_at_unix_s,
        trust_window_valid: true,
        public_key_digests: trust.public_key_digests,
        revocation_ref: trust.revocation_ref,
        bundle_blake3: bundle.bundle_blake3.clone(),
        evidence_complete: bundle_report.evidence_complete,
        missing_evidence: bundle_report.missing_evidence,
        output_matches,
        non_claim: RECEIPT_BUNDLE_NON_CLAIM,
    })
}

pub fn import_verified_receipt_bundle(
    bundle: &ReceiptBundle,
    state_dir: &Path,
    store_prefix: &str,
    context: &TrustVerificationContext,
    outputs: &[String],
) -> Result<ReceiptImportReport, RunError> {
    verify_receipt_bundle_against_state_with_context(bundle, state_dir, store_prefix, context, outputs)?;
    import_receipt_bundle(bundle, state_dir)
}

pub fn import_receipt_bundle(bundle: &ReceiptBundle, state_dir: &Path) -> Result<ReceiptImportReport, RunError> {
    validate_receipt_bundle(bundle)?;
    let records_dir = state_dir.join(RECEIPT_STATE_DIR).join(RECEIPT_RECORDS_DIR);
    fs::create_dir_all(&records_dir)
        .map_err(|err| RunError::Internal(format!("creating receipt records dir {}: {err}", records_dir.display())))?;
    let target = records_dir.join(format!("{}.json", bundle.bundle_blake3));
    let bundle_imported = plan_receipt_bundle_import(bundle, &target)?;
    let graph_plan = plan_semantic_graph_import(bundle, state_dir)?;
    if bundle_imported {
        let tmp = target.with_extension(TEMP_FILE_EXTENSION);
        write_receipt_bundle(&tmp, bundle)?;
        fs::rename(&tmp, &target)
            .map_err(|err| RunError::Internal(format!("committing receipt bundle {}: {err}", target.display())))?;
    }
    if graph_plan.changed {
        graph_plan
            .merged
            .save(&state_dir.join(SEMANTIC_GRAPH_FILE))
            .map_err(|err| RunError::Internal(err.to_string()))?;
    }
    let imported = bundle_imported || graph_plan.changed;
    Ok(ReceiptImportReport {
        imported,
        idempotent: !imported,
        bundle_imported,
        graph_imported: graph_plan.changed,
        graph_nodes_imported: checked_u32(graph_plan.nodes_imported, "imported semantic graph node count")?,
        graph_edges_imported: checked_u32(graph_plan.edges_imported, "imported semantic graph edge count")?,
        bundle_blake3: bundle.bundle_blake3.clone(),
        non_claim: RECEIPT_BUNDLE_NON_CLAIM,
    })
}

impl TrustVerificationContext {
    pub fn new(
        expected_policy_hash: String,
        valid_at_unix_s: u64,
        expected_revocation_ref: Option<String>,
        revoked_public_key_digests: Vec<String>,
    ) -> Self {
        Self {
            expected_policy_hash,
            valid_at_unix_s,
            expected_revocation_ref,
            revoked_public_key_digests: revoked_public_key_digests.into_iter().collect(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct TrustSnapshotValidation {
    public_key_digests: Vec<String>,
    revocation_ref: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SemanticGraphImportPlan {
    merged: crate::semantic_graph::SemanticGraph,
    changed: bool,
    nodes_imported: usize,
    edges_imported: usize,
}

fn plan_receipt_bundle_import(bundle: &ReceiptBundle, target: &Path) -> Result<bool, RunError> {
    if !target.exists() {
        return Ok(true);
    }
    let existing = read_receipt_bundle(target)?;
    if &existing != bundle {
        return Err(RunError::Internal(format!(
            "conflicting receipt bundle already exists for {}",
            bundle.bundle_blake3
        )));
    }
    Ok(false)
}

fn validate_trust_snapshot(
    bundle: &ReceiptBundle,
    store_prefix: &str,
    context: &TrustVerificationContext,
) -> Result<TrustSnapshotValidation, RunError> {
    validate_expected_policy_and_prefix(bundle, store_prefix, &context.expected_policy_hash)?;
    validate_trust_snapshot_shape(&bundle.trust_snapshot)?;
    validate_trust_window(bundle, context.valid_at_unix_s)?;
    let public_key_digests = validate_public_key_digests(&bundle.trust_snapshot.public_key_digests)?;
    validate_revocation_ref(
        bundle.trust_snapshot.revocation_ref.as_deref(),
        context.expected_revocation_ref.as_deref(),
    )?;
    reject_revoked_public_keys(&public_key_digests, &context.revoked_public_key_digests)?;
    Ok(TrustSnapshotValidation {
        public_key_digests,
        revocation_ref: bundle.trust_snapshot.revocation_ref.clone(),
    })
}

fn validate_trust_snapshot_shape(snapshot: &TrustSnapshot) -> Result<(), RunError> {
    validate_field("policy_hash", &snapshot.policy_hash)?;
    validate_public_key_digests(&snapshot.public_key_digests)?;
    if let Some(value) = &snapshot.revocation_ref {
        validate_field("revocation_ref", value)?;
    }
    if snapshot.valid_after_unix_s >= snapshot.valid_before_unix_s {
        return Err(RunError::Internal("receipt bundle trust window is empty".to_string()));
    }
    Ok(())
}

fn validate_trust_window(bundle: &ReceiptBundle, valid_at_unix_s: u64) -> Result<(), RunError> {
    if bundle.trust_snapshot.valid_after_unix_s >= bundle.trust_snapshot.valid_before_unix_s {
        return Err(RunError::Internal("receipt bundle trust window is empty".to_string()));
    }
    if valid_at_unix_s < bundle.trust_snapshot.valid_after_unix_s {
        return Err(RunError::Internal(format!(
            "receipt bundle trust snapshot is not valid until {}",
            bundle.trust_snapshot.valid_after_unix_s
        )));
    }
    if valid_at_unix_s >= bundle.trust_snapshot.valid_before_unix_s {
        return Err(RunError::Internal(format!(
            "receipt bundle trust snapshot expired at {}",
            bundle.trust_snapshot.valid_before_unix_s
        )));
    }
    Ok(())
}

fn validate_public_key_digests(digests: &[String]) -> Result<Vec<String>, RunError> {
    let mut sorted = BTreeSet::new();
    for digest in digests {
        validate_digest(digest)?;
        sorted.insert(digest.clone());
    }
    Ok(sorted.into_iter().collect())
}

fn validate_revocation_ref(bundle_ref: Option<&str>, expected_ref: Option<&str>) -> Result<(), RunError> {
    if let Some(value) = bundle_ref {
        validate_field("revocation_ref", value)?;
    }
    let Some(expected) = expected_ref else {
        return Ok(());
    };
    validate_field("expected_revocation_ref", expected)?;
    if bundle_ref != Some(expected) {
        return Err(RunError::Internal(format!(
            "receipt bundle revocation ref mismatch: bundle={} expected={}",
            bundle_ref.unwrap_or("<none>"),
            expected
        )));
    }
    Ok(())
}

fn reject_revoked_public_keys(
    public_key_digests: &[String],
    revoked_public_key_digests: &BTreeSet<String>,
) -> Result<(), RunError> {
    for digest in revoked_public_key_digests {
        validate_digest(digest)?;
        if public_key_digests.binary_search(digest).is_ok() {
            return Err(RunError::Internal(format!("receipt bundle signer key is revoked: {digest}")));
        }
    }
    Ok(())
}

fn plan_semantic_graph_import(bundle: &ReceiptBundle, state_dir: &Path) -> Result<SemanticGraphImportPlan, RunError> {
    let imported = semantic_graph_from_receipt_bundle(bundle)?;
    let path = state_dir.join(SEMANTIC_GRAPH_FILE);
    let existing = if path.is_file() {
        crate::semantic_graph::SemanticGraph::load(&path).map_err(|err| RunError::Internal(err.to_string()))?
    } else {
        crate::semantic_graph::SemanticGraph::empty()
    };
    merge_semantic_graph_import(existing, imported)
}

fn semantic_graph_from_receipt_bundle(
    bundle: &ReceiptBundle,
) -> Result<crate::semantic_graph::SemanticGraph, RunError> {
    let mut nodes = BTreeMap::<String, crate::semantic_graph::SemanticNode>::new();
    let mut edges = Vec::new();
    for record in &bundle.records {
        if record.kind == ReceiptRecordKind::SemanticGraphEdge {
            edges.push(semantic_edge_from_receipt_record(record)?);
            continue;
        }
        if let Some(kind) = semantic_node_kind_for_receipt_record(&record.kind) {
            insert_receipt_graph_node(&mut nodes, record, kind)?;
        }
    }
    Ok(crate::semantic_graph::SemanticGraph {
        schema: crate::semantic_graph::SEMANTIC_GRAPH_SCHEMA.to_string(),
        nodes: nodes.into_values().collect(),
        edges,
        aliases: Vec::new(),
    })
}

fn insert_receipt_graph_node(
    nodes: &mut BTreeMap<String, crate::semantic_graph::SemanticNode>,
    record: &ReceiptRecord,
    kind: crate::semantic_graph::SemanticNodeKind,
) -> Result<(), RunError> {
    let node = crate::semantic_graph::SemanticNode {
        id: record.identity.clone(),
        kind,
        digest: Some(record.digest.clone()),
        metadata: graph_node_metadata(record),
    };
    if let Some(existing) = nodes.get(&node.id) {
        if existing.kind != node.kind || existing.digest != node.digest || existing.metadata != node.metadata {
            return Err(RunError::Internal(format!("conflicting receipt graph node {}", node.id)));
        }
        return Ok(());
    }
    nodes.insert(node.id.clone(), node);
    Ok(())
}

fn graph_node_metadata(record: &ReceiptRecord) -> BTreeMap<String, String> {
    record
        .metadata
        .iter()
        .filter(|(key, _)| key.as_str() != "semantic_node_kind")
        .map(|(key, value)| (key.clone(), value.clone()))
        .collect()
}

fn semantic_node_kind_for_receipt_record(kind: &ReceiptRecordKind) -> Option<crate::semantic_graph::SemanticNodeKind> {
    match kind {
        ReceiptRecordKind::SourceRef => Some(crate::semantic_graph::SemanticNodeKind::SourceTree),
        ReceiptRecordKind::ActionRef => Some(crate::semantic_graph::SemanticNodeKind::Recipe),
        ReceiptRecordKind::SandboxReport => Some(crate::semantic_graph::SemanticNodeKind::SandboxProfile),
        ReceiptRecordKind::OutputRef => Some(crate::semantic_graph::SemanticNodeKind::StoreOutput),
        ReceiptRecordKind::TrustBasis => Some(crate::semantic_graph::SemanticNodeKind::Provider),
        _ => None,
    }
}

fn semantic_edge_from_receipt_record(record: &ReceiptRecord) -> Result<crate::semantic_graph::SemanticEdge, RunError> {
    let (kind, from, to) = parse_semantic_edge_identity(&record.identity)?;
    let edge = crate::semantic_graph::SemanticEdge {
        from,
        to,
        kind,
        metadata: record.metadata.clone(),
    };
    let digest = blake3_digest_for_json(&edge)?;
    if digest != record.digest {
        return Err(RunError::Internal(format!("semantic graph edge digest mismatch for {}", record.identity)));
    }
    Ok(edge)
}

fn parse_semantic_edge_identity(
    identity: &str,
) -> Result<(crate::semantic_graph::SemanticEdgeKind, String, String), RunError> {
    let Some((kind_raw, endpoints)) = identity.split_once(RECORD_SPEC_SEPARATOR) else {
        return Err(RunError::Internal(format!("invalid semantic graph edge identity {identity}")));
    };
    let Some((from, to)) = endpoints.split_once(SEMANTIC_EDGE_SEPARATOR) else {
        return Err(RunError::Internal(format!("invalid semantic graph edge identity {identity}")));
    };
    validate_field("semantic graph edge from", from)?;
    validate_field("semantic graph edge to", to)?;
    Ok((semantic_edge_kind_from_str(kind_raw)?, from.to_string(), to.to_string()))
}

fn semantic_edge_kind_from_str(raw: &str) -> Result<crate::semantic_graph::SemanticEdgeKind, RunError> {
    match raw {
        "depends-on" => Ok(crate::semantic_graph::SemanticEdgeKind::DependsOn),
        "produced-by" => Ok(crate::semantic_graph::SemanticEdgeKind::ProducedBy),
        "uses-source" => Ok(crate::semantic_graph::SemanticEdgeKind::UsesSource),
        "uses-sandbox" => Ok(crate::semantic_graph::SemanticEdgeKind::UsesSandbox),
        "uses-provider" => Ok(crate::semantic_graph::SemanticEdgeKind::UsesProvider),
        "supports-proof" => Ok(crate::semantic_graph::SemanticEdgeKind::SupportsProof),
        "requests-witness" => Ok(crate::semantic_graph::SemanticEdgeKind::RequestsWitness),
        "releases-as" => Ok(crate::semantic_graph::SemanticEdgeKind::ReleasesAs),
        "alias-of" => Ok(crate::semantic_graph::SemanticEdgeKind::AliasOf),
        other => Err(RunError::Internal(format!("unsupported semantic graph edge kind {other}"))),
    }
}

fn merge_semantic_graph_import(
    mut existing: crate::semantic_graph::SemanticGraph,
    imported: crate::semantic_graph::SemanticGraph,
) -> Result<SemanticGraphImportPlan, RunError> {
    let mut nodes_imported = 0usize;
    let mut edges_imported = 0usize;
    for node in imported.nodes {
        if merge_semantic_node(&mut existing, node)? {
            nodes_imported = nodes_imported.saturating_add(1);
        }
    }
    for edge in imported.edges {
        if merge_semantic_edge(&mut existing, edge)? {
            edges_imported = edges_imported.saturating_add(1);
        }
    }
    sort_semantic_graph(&mut existing);
    existing.validate().map_err(|err| RunError::Internal(err.to_string()))?;
    let changed = nodes_imported > 0 || edges_imported > 0;
    Ok(SemanticGraphImportPlan {
        merged: existing,
        changed,
        nodes_imported,
        edges_imported,
    })
}

fn merge_semantic_node(
    graph: &mut crate::semantic_graph::SemanticGraph,
    imported: crate::semantic_graph::SemanticNode,
) -> Result<bool, RunError> {
    let Some(existing) = graph.nodes.iter_mut().find(|node| node.id == imported.id) else {
        graph.nodes.push(imported);
        return Ok(true);
    };
    if existing.kind != imported.kind {
        return Err(RunError::Internal(format!("semantic graph node conflict for {}", imported.id)));
    }
    if let (Some(current), Some(incoming)) = (&existing.digest, &imported.digest)
        && current != incoming
    {
        return Err(RunError::Internal(format!("semantic graph node digest conflict for {}", imported.id)));
    }
    let mut changed = false;
    if existing.digest.is_none() && imported.digest.is_some() {
        existing.digest = imported.digest;
        changed = true;
    }
    for (key, value) in imported.metadata {
        if let Some(current) = existing.metadata.get(&key) {
            if current != &value {
                return Err(RunError::Internal(format!(
                    "semantic graph node metadata conflict for {} key {}",
                    imported.id, key
                )));
            }
            continue;
        }
        existing.metadata.insert(key, value);
        changed = true;
    }
    Ok(changed)
}

fn merge_semantic_edge(
    graph: &mut crate::semantic_graph::SemanticGraph,
    imported: crate::semantic_graph::SemanticEdge,
) -> Result<bool, RunError> {
    let key_matches = |edge: &crate::semantic_graph::SemanticEdge| {
        edge.from == imported.from && edge.to == imported.to && edge.kind == imported.kind
    };
    if let Some(existing) = graph.edges.iter().find(|edge| key_matches(edge)) {
        if existing.metadata != imported.metadata {
            return Err(RunError::Internal(format!(
                "semantic graph edge metadata conflict for {}:{}->{}",
                imported.kind, imported.from, imported.to
            )));
        }
        return Ok(false);
    }
    graph.edges.push(imported);
    Ok(true)
}

fn sort_semantic_graph(graph: &mut crate::semantic_graph::SemanticGraph) {
    graph.nodes.sort_by(|left, right| left.id.cmp(&right.id));
    graph.edges.sort_by(|left, right| {
        (left.kind.clone(), left.from.clone(), left.to.clone()).cmp(&(
            right.kind.clone(),
            right.from.clone(),
            right.to.clone(),
        ))
    });
    graph.aliases.sort_by(|left, right| {
        (left.alias.clone(), left.target.clone()).cmp(&(right.alias.clone(), right.target.clone()))
    });
}

fn record_from_spec(spec: &ReceiptRecordSpec) -> ReceiptRecord {
    ReceiptRecord {
        kind: spec.kind.clone(),
        identity: spec.identity.clone(),
        digest: spec.digest.clone(),
        metadata: BTreeMap::new(),
    }
}

fn normalize_receipt_records(mut records: Vec<ReceiptRecord>) -> Result<Vec<ReceiptRecord>, RunError> {
    records.sort_by(|left, right| record_total_key(left).cmp(&record_total_key(right)));
    let mut normalized = Vec::with_capacity(records.len());
    let mut exact_seen = BTreeSet::new();
    for record in records {
        validate_record(&record)?;
        let total_key = record_total_key(&record);
        if exact_seen.insert(total_key) {
            normalized.push(record);
        }
    }
    reject_duplicate_records(&normalized)?;
    normalized.sort_by(|left, right| record_key(left).cmp(&record_key(right)));
    Ok(normalized)
}

fn validate_record(record: &ReceiptRecord) -> Result<(), RunError> {
    validate_field("identity", &record.identity)?;
    validate_digest(&record.digest)?;
    validate_metadata(&record.metadata)
}

fn classify_missing_evidence(bundle: &ReceiptBundle) -> Vec<ReceiptRecordKind> {
    if bundle.claim_strength == ClaimStrengthRequest::Diagnostic {
        return Vec::new();
    }
    let present = bundle.records.iter().map(|record| record.kind.clone()).collect::<BTreeSet<_>>();
    let required = [
        ReceiptRecordKind::SourceRef,
        ReceiptRecordKind::ActionRef,
        ReceiptRecordKind::SandboxReport,
        ReceiptRecordKind::NetworkPolicy,
        ReceiptRecordKind::ReferenceScan,
        ReceiptRecordKind::OutputRef,
        ReceiptRecordKind::ArtifactAttestation,
        ReceiptRecordKind::TrustBasis,
        ReceiptRecordKind::Signature,
    ];
    required.iter().filter(|kind| !present.contains(*kind)).cloned().collect::<Vec<_>>()
}

fn reject_duplicate_records(records: &[ReceiptRecord]) -> Result<(), RunError> {
    let mut seen = BTreeMap::<String, String>::new();
    for record in records {
        validate_record(record)?;
        let key = record_key(record);
        let value = record_value_key(record);
        if let Some(existing) = seen.insert(key.clone(), value.clone())
            && existing != value
        {
            return Err(RunError::Internal(format!("duplicate receipt record {key}")));
        }
    }
    Ok(())
}

fn record_key(record: &ReceiptRecord) -> String {
    format!("{:?}:{}", record.kind, record.identity)
}

fn record_total_key(record: &ReceiptRecord) -> String {
    format!("{}:{}", record_key(record), record_value_key(record))
}

fn record_value_key(record: &ReceiptRecord) -> String {
    format!("{}:{:?}", record.digest, record.metadata)
}

fn digest_bundle_without_digest(bundle: &ReceiptBundle) -> Result<String, RunError> {
    let mut clone = bundle.clone();
    clone.bundle_blake3.clear();
    let encoded = serde_json::to_vec(&clone)
        .map_err(|err| RunError::Internal(format!("serializing receipt bundle for digest: {err}")))?;
    Ok(blake3::hash(&encoded).to_hex().to_string())
}

fn validate_field(label: &str, value: &str) -> Result<(), RunError> {
    if value.is_empty() || value.len() > MAX_RECEIPT_FIELD_BYTES || value.contains('\0') || value.contains("..") {
        return Err(RunError::Internal(format!("invalid receipt {label}: '{value}'")));
    }
    Ok(())
}

fn validate_metadata(metadata: &BTreeMap<String, String>) -> Result<(), RunError> {
    if metadata.len() > MAX_RECEIPT_METADATA_ENTRIES {
        return Err(RunError::Internal(format!("receipt metadata entry count exceeds {MAX_RECEIPT_METADATA_ENTRIES}")));
    }
    let mut total_bytes = 0usize;
    for (key, value) in metadata {
        validate_field("metadata key", key)?;
        validate_field("metadata value", value)?;
        total_bytes = total_bytes.saturating_add(key.len()).saturating_add(value.len());
    }
    if total_bytes > MAX_RECEIPT_METADATA_BYTES {
        return Err(RunError::Internal(format!("receipt metadata exceeds {MAX_RECEIPT_METADATA_BYTES} bytes")));
    }
    Ok(())
}

fn validate_digest(value: &str) -> Result<(), RunError> {
    validate_field("digest", value)?;
    let digest_like = value.starts_with(BLAKE3_DIGEST_PREFIX)
        || value.starts_with(SHA256_DIGEST_PREFIX)
        || value.len() == DIGEST_HEX_CHARS;
    if !digest_like {
        return Err(RunError::Internal(format!("receipt digest must be blake3:/sha256:/hex, got '{value}'")));
    }
    Ok(())
}

fn blake3_digest_for_bytes(bytes: &[u8]) -> String {
    format!("{BLAKE3_DIGEST_PREFIX}{}", blake3::hash(bytes).to_hex())
}

fn blake3_digest_for_json(value: &impl Serialize) -> Result<String, RunError> {
    let bytes = serde_json::to_vec(value)
        .map_err(|err| RunError::Internal(format!("serializing receipt evidence for digest: {err}")))?;
    Ok(blake3_digest_for_bytes(&bytes))
}

fn record_summary(record: &ReceiptRecord) -> ReceiptRecordSummary {
    ReceiptRecordSummary {
        kind: record.kind.clone(),
        identity: record.identity.clone(),
        digest: record.digest.clone(),
    }
}

fn output_identities(bundle: &ReceiptBundle) -> Vec<String> {
    bundle
        .records
        .iter()
        .filter(|record| record.kind == ReceiptRecordKind::OutputRef)
        .map(|record| record.identity.clone())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn checked_u32(count: usize, label: &str) -> Result<u32, RunError> {
    u32::try_from(count).map_err(|_| RunError::Internal(format!("{label} does not fit in u32: {count}")))
}

fn validate_expected_policy_and_prefix(
    bundle: &ReceiptBundle,
    store_prefix: &str,
    expected_policy_hash: &str,
) -> Result<(), RunError> {
    validate_field("expected_policy_hash", expected_policy_hash)?;
    if bundle.store_prefix != store_prefix {
        return Err(RunError::Internal(format!(
            "receipt bundle store prefix mismatch: bundle={} local={}",
            bundle.store_prefix, store_prefix
        )));
    }
    if bundle.trust_snapshot.policy_hash != expected_policy_hash {
        return Err(RunError::Internal(format!(
            "receipt bundle policy hash mismatch: bundle={} expected={}",
            bundle.trust_snapshot.policy_hash, expected_policy_hash
        )));
    }
    Ok(())
}

fn verify_output_records(
    bundle: &ReceiptBundle,
    local_records: &[ReceiptRecord],
    outputs: &[String],
) -> Result<Vec<ReceiptOutputMatch>, RunError> {
    if local_records.is_empty() {
        return Err(RunError::Internal("missing local PathInfo for receipt outputs".to_string()));
    }
    let bundle_outputs = bundle_output_records(bundle);
    if bundle_outputs.is_empty() {
        return Err(RunError::Internal("receipt bundle is missing output-ref evidence".to_string()));
    }
    let mut matches = Vec::with_capacity(local_records.len());
    for local in local_records {
        if !selector_matches_record(outputs, &local.identity, local.metadata.get("pathinfo_store_path")) {
            continue;
        }
        let Some(bundle_record) = bundle_outputs.get(&local.identity) else {
            return Err(RunError::Internal(format!("receipt bundle is missing output-ref for {}", local.identity)));
        };
        if bundle_record.digest != local.digest {
            return Err(RunError::Internal(format!("receipt output-ref digest mismatch for {}", local.identity)));
        }
        matches.push(output_match_from_record(bundle_record, local)?);
    }
    if matches.is_empty() {
        return Err(RunError::Internal("missing local PathInfo for receipt outputs".to_string()));
    }
    Ok(matches)
}

fn bundle_output_records(bundle: &ReceiptBundle) -> BTreeMap<String, &ReceiptRecord> {
    bundle
        .records
        .iter()
        .filter(|record| record.kind == ReceiptRecordKind::OutputRef)
        .map(|record| (record.identity.clone(), record))
        .collect()
}

fn selector_matches_record(outputs: &[String], logical_path: &str, store_path: Option<&String>) -> bool {
    outputs.iter().any(|output| output == logical_path || store_path.is_some_and(|path| output == path))
}

fn output_match_from_record(
    bundle_record: &ReceiptRecord,
    local_record: &ReceiptRecord,
) -> Result<ReceiptOutputMatch, RunError> {
    let nar_sha256 = required_metadata(local_record, "nar_sha256")?;
    let nar_size_bytes = parse_u64_metadata(local_record, "nar_size_bytes")?;
    Ok(ReceiptOutputMatch {
        identity: local_record.identity.clone(),
        record_digest: bundle_record.digest.clone(),
        local_digest: local_record.digest.clone(),
        nar_sha256,
        nar_size_bytes,
    })
}

fn required_metadata(record: &ReceiptRecord, key: &str) -> Result<String, RunError> {
    record
        .metadata
        .get(key)
        .cloned()
        .ok_or_else(|| RunError::Internal(format!("receipt local output metadata missing {key}")))
}

fn parse_u64_metadata(record: &ReceiptRecord, key: &str) -> Result<u64, RunError> {
    let value = required_metadata(record, key)?;
    value
        .parse::<u64>()
        .map_err(|err| RunError::Internal(format!("receipt local output metadata {key} is not u64: {err}")))
}

fn local_output_records_for_outputs(
    state_dir: &Path,
    store_prefix: &str,
    outputs: &[String],
) -> Result<Vec<ReceiptRecord>, RunError> {
    let records = collect_pathinfo_records_for_outputs(state_dir, store_prefix, outputs)?;
    Ok(records.into_iter().filter(|record| record.kind == ReceiptRecordKind::OutputRef).collect())
}

fn gather_receipt_records_from_state(
    state_dir: &Path,
    store_prefix: &str,
    outputs: &[String],
) -> Result<Vec<ReceiptRecord>, RunError> {
    if outputs.is_empty() {
        return Ok(Vec::new());
    }
    let mut records = collect_pathinfo_records_for_outputs(state_dir, store_prefix, outputs)?;
    records.extend(collect_semantic_graph_records_for_outputs(state_dir, store_prefix, outputs)?);
    normalize_receipt_records(records)
}

fn collect_pathinfo_records_for_outputs(
    state_dir: &Path,
    store_prefix: &str,
    outputs: &[String],
) -> Result<Vec<ReceiptRecord>, RunError> {
    let db_path = state_dir.join(PATHINFO_DB_FILE);
    if !db_path.is_file() {
        return Ok(Vec::new());
    }
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|err| RunError::Internal(format!("creating PathInfo reader runtime: {err}")))?;
    runtime.block_on(collect_pathinfo_records_for_outputs_async(
        db_path,
        state_dir.to_path_buf(),
        store_prefix.to_string(),
        outputs.to_vec(),
    ))
}

async fn collect_pathinfo_records_for_outputs_async(
    db_path: PathBuf,
    state_dir: PathBuf,
    store_prefix: String,
    outputs: Vec<String>,
) -> Result<Vec<ReceiptRecord>, RunError> {
    let service = RedbPathInfoService::new("receipt-bundle-pathinfo".to_string(), RedbPathInfoServiceConfig {
        path: Some(db_path.clone()),
        cache_size: None,
        read_only: true,
    })
    .await
    .map_err(|err| RunError::Internal(format!("opening PathInfo database {}: {err}", db_path.display())))?;
    let mut records = Vec::new();
    let mut stream = service.list();
    for _ in 0..MAX_RECEIPT_RECORDS {
        let Some(result) = stream.next().await else { break };
        let path_info = result.map_err(|err| RunError::Internal(format!("listing PathInfo database: {err}")))?;
        if !pathinfo_matches_outputs(&path_info, &store_prefix, &outputs) {
            continue;
        }
        records.push(output_record_from_pathinfo(&path_info, &store_prefix)?);
        records.extend(attestation_records_for_pathinfo(&state_dir, &store_prefix, &path_info)?);
    }
    Ok(records)
}

fn pathinfo_matches_outputs(path_info: &PathInfo, store_prefix: &str, outputs: &[String]) -> bool {
    let store_path = path_info.store_path.to_string();
    let logical_path = path_info.store_path.to_absolute_path_with_prefix(store_prefix);
    outputs.iter().any(|output| output == &store_path || output == &logical_path)
}

fn output_record_from_pathinfo(path_info: &PathInfo, store_prefix: &str) -> Result<ReceiptRecord, RunError> {
    let logical_path = path_info.store_path.to_absolute_path_with_prefix(store_prefix);
    let nar_sha256 = data_encoding::HEXLOWER.encode(&path_info.nar_sha256);
    let mut metadata = BTreeMap::new();
    metadata.insert("evidence_source".to_string(), PATHINFO_DB_FILE.to_string());
    metadata.insert("pathinfo_store_path".to_string(), path_info.store_path.to_string());
    metadata.insert("nar_sha256".to_string(), nar_sha256.clone());
    metadata.insert("nar_size_bytes".to_string(), path_info.nar_size.to_string());
    metadata.insert("reference_count".to_string(), path_info.references.len().to_string());
    metadata.insert("signature_count".to_string(), path_info.signatures.len().to_string());
    if let Some(deriver) = &path_info.deriver {
        metadata.insert("deriver".to_string(), deriver.to_string());
    }
    let digest = blake3_digest_for_json(&pathinfo_digest_payload(path_info, store_prefix))?;
    Ok(ReceiptRecord {
        kind: ReceiptRecordKind::OutputRef,
        identity: logical_path,
        digest,
        metadata,
    })
}

fn pathinfo_digest_payload(path_info: &PathInfo, store_prefix: &str) -> BTreeMap<String, String> {
    let mut payload = BTreeMap::new();
    payload.insert("logical_path".to_string(), path_info.store_path.to_absolute_path_with_prefix(store_prefix));
    payload.insert("store_path".to_string(), path_info.store_path.to_string());
    payload.insert("nar_sha256".to_string(), data_encoding::HEXLOWER.encode(&path_info.nar_sha256));
    payload.insert("nar_size_bytes".to_string(), path_info.nar_size.to_string());
    payload.insert("references".to_string(), joined_sorted(path_info.references.iter().map(ToString::to_string)));
    payload.insert("signatures".to_string(), joined_sorted(path_info.signatures.iter().map(ToString::to_string)));
    if let Some(deriver) = &path_info.deriver {
        payload.insert("deriver".to_string(), deriver.to_string());
    }
    if let Some(ca) = &path_info.ca {
        payload.insert("ca".to_string(), format!("{ca:?}"));
    }
    payload
}

fn attestation_records_for_pathinfo(
    state_dir: &Path,
    store_prefix: &str,
    path_info: &PathInfo,
) -> Result<Vec<ReceiptRecord>, RunError> {
    let logical_path = path_info.store_path.to_absolute_path_with_prefix(store_prefix);
    let artifact = crunch_store::artifact_attestation_file_path(state_dir, store_prefix, &path_info.store_path);
    let closure = crunch_store::closure_attestation_file_path(
        state_dir,
        store_prefix,
        std::slice::from_ref(&path_info.store_path),
        crunch_attestation::ClosureSemantics::Runtime,
    );
    let mut records = Vec::new();
    if artifact.is_file() {
        records.push(sidecar_record(ReceiptRecordKind::ArtifactAttestation, &logical_path, &artifact)?);
    }
    if closure.is_file() {
        records.push(sidecar_record(ReceiptRecordKind::ClosureAttestation, &logical_path, &closure)?);
    }
    Ok(records)
}

fn sidecar_record(kind: ReceiptRecordKind, identity: &str, path: &Path) -> Result<ReceiptRecord, RunError> {
    let bytes = fs::read(path)
        .map_err(|err| RunError::Internal(format!("reading evidence sidecar {}: {err}", path.display())))?;
    let mut metadata = BTreeMap::new();
    if let Some(file_name) = path.file_name().and_then(|name| name.to_str()) {
        metadata.insert("sidecar_file".to_string(), file_name.to_string());
    }
    metadata.insert("sidecar_bytes".to_string(), bytes.len().to_string());
    Ok(ReceiptRecord {
        kind,
        identity: identity.to_string(),
        digest: blake3_digest_for_bytes(&bytes),
        metadata,
    })
}

fn collect_semantic_graph_records_for_outputs(
    state_dir: &Path,
    store_prefix: &str,
    outputs: &[String],
) -> Result<Vec<ReceiptRecord>, RunError> {
    let path = state_dir.join(SEMANTIC_GRAPH_FILE);
    if !path.is_file() {
        return Ok(Vec::new());
    }
    let graph = crate::semantic_graph::SemanticGraph::load(&path).map_err(|err| RunError::Internal(err.to_string()))?;
    let source_records = read_source_records_by_identity(state_dir)?;
    let mut records = Vec::new();
    for query in graph_queries_for_outputs(store_prefix, outputs) {
        let Ok(result) = graph.graph_for_root(&query) else {
            continue;
        };
        records.extend(records_from_graph_nodes(&result.nodes, &source_records)?);
        records.extend(records_from_graph_edges(&result.edges)?);
    }
    normalize_receipt_records(records)
}

fn graph_queries_for_outputs(store_prefix: &str, outputs: &[String]) -> Vec<String> {
    let mut queries = BTreeSet::new();
    for output in outputs {
        queries.insert(output.clone());
        if let Some(stripped) = output.strip_prefix(&format!("{store_prefix}/")) {
            queries.insert(stripped.to_string());
        }
    }
    queries.into_iter().collect()
}

fn records_from_graph_nodes(
    nodes: &[&crate::semantic_graph::SemanticNode],
    source_records: &BTreeMap<String, crate::source_bundle::SourceRecord>,
) -> Result<Vec<ReceiptRecord>, RunError> {
    let mut records = Vec::new();
    for node in nodes {
        if let Some(record) = source_ref_from_source_state(node, source_records)? {
            records.push(record);
            continue;
        }
        if let Some(kind) = receipt_kind_for_graph_node(&node.kind)
            && let Some(digest) = &node.digest
        {
            records.push(graph_node_record(kind, node, digest)?);
        }
    }
    Ok(records)
}

fn source_ref_from_source_state(
    node: &crate::semantic_graph::SemanticNode,
    source_records: &BTreeMap<String, crate::source_bundle::SourceRecord>,
) -> Result<Option<ReceiptRecord>, RunError> {
    if node.kind != crate::semantic_graph::SemanticNodeKind::SourceTree {
        return Ok(None);
    }
    let Some(source) = source_records.get(&node.id) else {
        return Ok(None);
    };
    let digest = format!("{BLAKE3_DIGEST_PREFIX}{}", source.content_blake3);
    let mut metadata = BTreeMap::new();
    metadata.insert("source_kind".to_string(), format!("{:?}", source.kind));
    metadata.insert("source_payload_bytes".to_string(), source.payload_bytes.to_string());
    metadata.insert("source_file_count".to_string(), source.files.len().to_string());
    Ok(Some(ReceiptRecord {
        kind: ReceiptRecordKind::SourceRef,
        identity: source.identity.clone(),
        digest,
        metadata,
    }))
}

fn receipt_kind_for_graph_node(kind: &crate::semantic_graph::SemanticNodeKind) -> Option<ReceiptRecordKind> {
    match kind {
        crate::semantic_graph::SemanticNodeKind::SourceTree => Some(ReceiptRecordKind::SourceRef),
        crate::semantic_graph::SemanticNodeKind::Recipe => Some(ReceiptRecordKind::ActionRef),
        crate::semantic_graph::SemanticNodeKind::SandboxProfile => Some(ReceiptRecordKind::SandboxReport),
        crate::semantic_graph::SemanticNodeKind::Provider => Some(ReceiptRecordKind::TrustBasis),
        _ => None,
    }
}

fn graph_node_record(
    kind: ReceiptRecordKind,
    node: &crate::semantic_graph::SemanticNode,
    digest: &str,
) -> Result<ReceiptRecord, RunError> {
    let mut metadata = node.metadata.clone();
    metadata.insert("semantic_node_kind".to_string(), node.kind.to_string());
    Ok(ReceiptRecord {
        kind,
        identity: node.id.clone(),
        digest: digest.to_string(),
        metadata,
    })
}

fn records_from_graph_edges(edges: &[&crate::semantic_graph::SemanticEdge]) -> Result<Vec<ReceiptRecord>, RunError> {
    let mut records = Vec::with_capacity(edges.len());
    for edge in edges {
        records.push(ReceiptRecord {
            kind: ReceiptRecordKind::SemanticGraphEdge,
            identity: semantic_edge_identity(edge),
            digest: blake3_digest_for_json(edge)?,
            metadata: edge.metadata.clone(),
        });
    }
    Ok(records)
}

fn semantic_edge_identity(edge: &crate::semantic_graph::SemanticEdge) -> String {
    format!("{}:{}->{}", edge.kind, edge.from, edge.to)
}

fn read_source_records_by_identity(
    state_dir: &Path,
) -> Result<BTreeMap<String, crate::source_bundle::SourceRecord>, RunError> {
    let records_dir = state_dir.join(SOURCE_STATE_DIR).join(SOURCE_RECORDS_DIR);
    if !records_dir.is_dir() {
        return Ok(BTreeMap::new());
    }
    let mut records = BTreeMap::new();
    for entry in sorted_json_files(&records_dir)? {
        let bytes = fs::read(&entry)
            .map_err(|err| RunError::Internal(format!("reading source record {}: {err}", entry.display())))?;
        let record = serde_json::from_slice::<crate::source_bundle::SourceRecord>(&bytes)
            .map_err(|err| RunError::Internal(format!("parsing source record {}: {err}", entry.display())))?;
        records.insert(record.identity.clone(), record);
    }
    Ok(records)
}

fn sorted_json_files(dir: &Path) -> Result<Vec<PathBuf>, RunError> {
    let mut paths = Vec::new();
    for entry in fs::read_dir(dir).map_err(|err| RunError::Internal(format!("reading {}: {err}", dir.display())))? {
        let path = entry.map_err(|err| RunError::Internal(format!("reading {} entry: {err}", dir.display())))?.path();
        if path.extension().and_then(|ext| ext.to_str()) == Some("json") {
            paths.push(path);
        }
    }
    paths.sort();
    Ok(paths)
}

fn joined_sorted(values: impl Iterator<Item = String>) -> String {
    values.collect::<BTreeSet<_>>().into_iter().collect::<Vec<_>>().join(",")
}

pub fn render_json(value: &impl Serialize) -> Result<String, RunError> {
    serde_json::to_string_pretty(value)
        .map_err(|err| RunError::Internal(format!("serializing receipt bundle report: {err}")))
}

pub fn cmd_receipt(
    action: crate::ReceiptAction,
    state_dir: &Path,
    store_prefix: &str,
    json_output: bool,
    now_unix_s: u64,
) -> Result<(), RunError> {
    match action {
        crate::ReceiptAction::Bundle { action } => {
            cmd_receipt_bundle(action, state_dir, store_prefix, json_output, now_unix_s)
        }
    }
}

fn cmd_receipt_bundle(
    action: crate::ReceiptBundleAction,
    state_dir: &Path,
    store_prefix: &str,
    json_output: bool,
    now_unix_s: u64,
) -> Result<(), RunError> {
    match action {
        crate::ReceiptBundleAction::Export {
            records,
            outputs,
            to,
            policy_hash,
            strong,
        } => {
            let bundle = build_from_cli_records(&records, &outputs, state_dir, store_prefix, &policy_hash, strong)?;
            write_receipt_bundle(&to, &bundle)?;
            print_report(&validate_receipt_bundle(&bundle)?, json_output)
        }
        crate::ReceiptBundleAction::List { from } => {
            let bundle = read_receipt_bundle(&from)?;
            print_report(&validate_receipt_bundle(&bundle)?, json_output)
        }
        crate::ReceiptBundleAction::Verify {
            from,
            outputs,
            policy_hash,
            valid_at_unix_s,
            revocation_ref,
            revoked_public_key_digests,
        } => {
            let bundle = read_receipt_bundle(&from)?;
            if outputs.is_empty() {
                return print_report(&validate_receipt_bundle(&bundle)?, json_output);
            }
            let Some(policy_hash) = policy_hash else {
                return Err(RunError::Internal("receipt bundle verify --output requires --policy-hash".to_string()));
            };
            let context = TrustVerificationContext::new(
                policy_hash,
                valid_at_unix_s.unwrap_or(now_unix_s),
                revocation_ref,
                revoked_public_key_digests,
            );
            let report =
                verify_receipt_bundle_against_state_with_context(&bundle, state_dir, store_prefix, &context, &outputs)?;
            print_verify(&report, json_output)
        }
        crate::ReceiptBundleAction::Import {
            from,
            outputs,
            policy_hash,
            valid_at_unix_s,
            revocation_ref,
            revoked_public_key_digests,
        } => {
            let bundle = read_receipt_bundle(&from)?;
            if outputs.is_empty() {
                return Err(RunError::Internal("receipt bundle import requires --output".to_string()));
            }
            let Some(policy_hash) = policy_hash else {
                return Err(RunError::Internal("receipt bundle import requires --policy-hash".to_string()));
            };
            let context = TrustVerificationContext::new(
                policy_hash,
                valid_at_unix_s.unwrap_or(now_unix_s),
                revocation_ref,
                revoked_public_key_digests,
            );
            let report = import_verified_receipt_bundle(&bundle, state_dir, store_prefix, &context, &outputs)?;
            print_import(&report, json_output)
        }
    }
}

fn build_from_cli_records(
    records: &[String],
    outputs: &[String],
    state_dir: &Path,
    store_prefix: &str,
    policy_hash: &str,
    strong: bool,
) -> Result<ReceiptBundle, RunError> {
    let specs = records.iter().map(|record| parse_receipt_record_spec(record)).collect::<Result<Vec<_>, _>>()?;
    let mut gathered = specs.iter().map(record_from_spec).collect::<Vec<_>>();
    gathered.extend(gather_receipt_records_from_state(state_dir, store_prefix, outputs)?);
    let claim = if strong {
        ClaimStrengthRequest::StrongActionCorrectness
    } else {
        ClaimStrengthRequest::Diagnostic
    };
    build_receipt_bundle_from_records(gathered, store_prefix, policy_hash, claim)
}

fn print_report(report: &ReceiptBundleReport, json_output: bool) -> Result<(), RunError> {
    if json_output {
        println!("{}", render_json(report)?);
        return Ok(());
    }
    println!(
        "format={} store_prefix={} records={} complete={} bundle_blake3={}",
        report.format, report.store_prefix, report.record_count, report.evidence_complete, report.bundle_blake3
    );
    if !report.output_identities.is_empty() {
        println!("outputs={}", report.output_identities.join(","));
    }
    if !report.missing_evidence.is_empty() {
        println!("missing_evidence={:?}", report.missing_evidence);
    }
    eprintln!("non_claim={}", report.non_claim);
    Ok(())
}

fn print_verify(report: &ReceiptVerifyReport, json_output: bool) -> Result<(), RunError> {
    if json_output {
        println!("{}", render_json(report)?);
        return Ok(());
    }
    println!(
        "RECEIPT_VERIFY bundle_blake3={} store_prefix={} outputs={} policy_hash_matched={} trust_window_valid={}",
        report.bundle_blake3,
        report.store_prefix,
        report.output_matches.len(),
        report.policy_hash_matched,
        report.trust_window_valid
    );
    for output in &report.output_matches {
        println!(
            "OUTPUT_MATCH {} nar_size={} nar_sha256={} digest={}",
            output.identity, output.nar_size_bytes, output.nar_sha256, output.record_digest
        );
    }
    if !report.missing_evidence.is_empty() {
        println!("missing_evidence={:?}", report.missing_evidence);
    }
    eprintln!("non_claim={}", report.non_claim);
    Ok(())
}

fn print_import(report: &ReceiptImportReport, json_output: bool) -> Result<(), RunError> {
    if json_output {
        println!("{}", render_json(report)?);
        return Ok(());
    }
    println!(
        "RECEIPT_IMPORT bundle_blake3={} imported={} idempotent={} bundle_imported={} graph_imported={} graph_nodes={} graph_edges={}",
        report.bundle_blake3,
        report.imported,
        report.idempotent,
        report.bundle_imported,
        report.graph_imported,
        report.graph_nodes_imported,
        report.graph_edges_imported
    );
    eprintln!("non_claim={}", report.non_claim);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn complete_specs() -> Vec<ReceiptRecordSpec> {
        [
            ReceiptRecordKind::SourceRef,
            ReceiptRecordKind::ActionRef,
            ReceiptRecordKind::SandboxReport,
            ReceiptRecordKind::NetworkPolicy,
            ReceiptRecordKind::ReferenceScan,
            ReceiptRecordKind::OutputRef,
            ReceiptRecordKind::ArtifactAttestation,
            ReceiptRecordKind::TrustBasis,
            ReceiptRecordKind::Signature,
        ]
        .iter()
        .enumerate()
        .map(|(index, kind)| ReceiptRecordSpec {
            kind: kind.clone(),
            identity: format!("record-{index}"),
            digest: format!("blake3:{:064x}", index + 1),
        })
        .collect()
    }

    #[test]
    fn complete_bundle_supports_strong_claim_classification() {
        let bundle = build_receipt_bundle(
            &complete_specs(),
            "/mantle/store",
            "policy-v1",
            ClaimStrengthRequest::StrongActionCorrectness,
        )
        .unwrap();
        let report = validate_receipt_bundle(&bundle).unwrap();
        assert!(report.evidence_complete);
        assert!(report.missing_evidence.is_empty());
    }

    #[test]
    fn partial_bundle_is_diagnostic_for_strong_claim() {
        let specs = vec![ReceiptRecordSpec {
            kind: ReceiptRecordKind::OutputRef,
            identity: "out".to_string(),
            digest: "blake3:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_string(),
        }];
        let bundle =
            build_receipt_bundle(&specs, "/mantle/store", "policy-v1", ClaimStrengthRequest::StrongActionCorrectness)
                .unwrap();
        let report = validate_receipt_bundle(&bundle).unwrap();
        assert!(!report.evidence_complete);
        assert!(report.missing_evidence.contains(&ReceiptRecordKind::SourceRef));
    }

    #[test]
    fn duplicate_receipt_records_fail_closed() {
        let specs = vec![
            ReceiptRecordSpec {
                kind: ReceiptRecordKind::OutputRef,
                identity: "out".to_string(),
                digest: "blake3:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_string(),
            },
            ReceiptRecordSpec {
                kind: ReceiptRecordKind::OutputRef,
                identity: "out".to_string(),
                digest: "blake3:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb".to_string(),
            },
        ];
        let err =
            build_receipt_bundle(&specs, "/mantle/store", "policy-v1", ClaimStrengthRequest::Diagnostic).unwrap_err();
        assert!(err.to_string().contains("duplicate receipt record"));
    }

    #[test]
    fn receipt_import_is_idempotent_and_conflict_checked() {
        let temp = tempfile::tempdir().unwrap();
        let bundle =
            build_receipt_bundle(&complete_specs(), "/mantle/store", "policy-v1", ClaimStrengthRequest::Diagnostic)
                .unwrap();
        let first = import_receipt_bundle(&bundle, temp.path()).unwrap();
        assert!(first.imported);
        let second = import_receipt_bundle(&bundle, temp.path()).unwrap();
        assert!(second.idempotent);
    }

    #[test]
    fn malformed_record_spec_is_rejected() {
        let err = parse_receipt_record_spec("output-ref:only-two-parts").unwrap_err();
        assert!(err.to_string().contains("kind:identity:digest"));
    }

    #[test]
    fn pathinfo_and_attestation_state_records_are_gathered_for_output() {
        let temp = tempfile::tempdir().unwrap();
        let state_dir = temp.path().join("state");
        let path_info = test_path_info("demo", TEST_STORE_DIGEST_BYTE, TEST_NAR_HASH_BYTE);
        write_pathinfo_db(&state_dir, path_info.clone());
        write_artifact_sidecar(&state_dir, "/mantle/store", &path_info.store_path, b"artifact-attestation");

        let output = path_info.store_path.to_absolute_path_with_prefix("/mantle/store");
        let records =
            gather_receipt_records_from_state(&state_dir, "/mantle/store", std::slice::from_ref(&output)).unwrap();

        assert!(records.iter().any(|record| record.kind == ReceiptRecordKind::OutputRef));
        assert!(records.iter().any(|record| record.kind == ReceiptRecordKind::ArtifactAttestation));
        let output_record = records.iter().find(|record| record.kind == ReceiptRecordKind::OutputRef).unwrap();
        assert_eq!(output_record.identity, output);
        assert!(output_record.digest.starts_with(BLAKE3_DIGEST_PREFIX));
        assert_eq!(output_record.metadata.get("nar_size_bytes").map(String::as_str), Some("17"));
    }

    #[test]
    fn graph_and_source_state_records_are_gathered_without_placeholders() {
        let temp = tempfile::tempdir().unwrap();
        let state_dir = temp.path().join("state");
        let output = "/mantle/store/11111111111111111111111111111111-demo".to_string();
        write_source_record(&state_dir, "src:demo", &"b".repeat(DIGEST_HEX_CHARS));
        write_semantic_graph(&state_dir, &output);

        let records =
            gather_receipt_records_from_state(&state_dir, "/mantle/store", std::slice::from_ref(&output)).unwrap();

        assert!(records.iter().any(|record| record.kind == ReceiptRecordKind::SourceRef));
        assert!(records.iter().any(|record| record.kind == ReceiptRecordKind::ActionRef));
        assert!(records.iter().any(|record| record.kind == ReceiptRecordKind::SandboxReport));
        assert!(records.iter().any(|record| record.kind == ReceiptRecordKind::SemanticGraphEdge));
        let source = records.iter().find(|record| record.kind == ReceiptRecordKind::SourceRef).unwrap();
        assert_eq!(source.identity, "src:demo");
        assert_eq!(source.digest, format!("{BLAKE3_DIGEST_PREFIX}{}", "b".repeat(DIGEST_HEX_CHARS)));
    }

    #[test]
    fn matching_local_output_verifies_against_policy_and_pathinfo() {
        let temp = tempfile::tempdir().unwrap();
        let state_dir = temp.path().join("state");
        let path_info = test_path_info("verify-demo", TEST_STORE_DIGEST_BYTE, TEST_NAR_HASH_BYTE);
        write_pathinfo_db(&state_dir, path_info.clone());
        let output = path_info.store_path.to_absolute_path_with_prefix("/mantle/store");
        let bundle = bundle_for_state_output(&state_dir, &output, "policy-v1");

        let report = verify_receipt_bundle_against_state(
            &bundle,
            &state_dir,
            "/mantle/store",
            "policy-v1",
            std::slice::from_ref(&output),
        )
        .unwrap();

        assert!(report.policy_hash_matched);
        assert_eq!(report.output_matches.len(), 1);
        assert_eq!(report.output_matches[0].identity, output);
        assert_eq!(report.output_matches[0].nar_size_bytes, TEST_NAR_SIZE_BYTES);
    }

    #[test]
    fn wrong_store_prefix_rejects_output_verification() {
        let temp = tempfile::tempdir().unwrap();
        let state_dir = temp.path().join("state");
        let path_info = test_path_info("prefix-demo", TEST_STORE_DIGEST_BYTE, TEST_NAR_HASH_BYTE);
        write_pathinfo_db(&state_dir, path_info.clone());
        let output = path_info.store_path.to_absolute_path_with_prefix("/mantle/store");
        let bundle = bundle_for_state_output(&state_dir, &output, "policy-v1");

        let err = verify_receipt_bundle_against_state(
            &bundle,
            &state_dir,
            "/other/store",
            "policy-v1",
            std::slice::from_ref(&output),
        )
        .unwrap_err();

        assert!(err.to_string().contains("store prefix mismatch"));
    }

    #[test]
    fn stale_local_output_digest_rejects_output_verification() {
        let temp = tempfile::tempdir().unwrap();
        let bundle_state = temp.path().join("bundle-state");
        let verify_state = temp.path().join("verify-state");
        let original = test_path_info("stale-demo", TEST_STORE_DIGEST_BYTE, TEST_NAR_HASH_BYTE);
        let stale = test_path_info("stale-demo", TEST_STORE_DIGEST_BYTE, TEST_STALE_NAR_HASH_BYTE);
        write_pathinfo_db(&bundle_state, original.clone());
        write_pathinfo_db(&verify_state, stale.clone());
        let output = original.store_path.to_absolute_path_with_prefix("/mantle/store");
        let bundle = bundle_for_state_output(&bundle_state, &output, "policy-v1");

        let err = verify_receipt_bundle_against_state(
            &bundle,
            &verify_state,
            "/mantle/store",
            "policy-v1",
            std::slice::from_ref(&output),
        )
        .unwrap_err();

        assert!(err.to_string().contains("output-ref digest mismatch"));
    }

    #[test]
    fn missing_output_ref_rejects_output_verification() {
        let temp = tempfile::tempdir().unwrap();
        let state_dir = temp.path().join("state");
        let path_info = test_path_info("missing-ref-demo", TEST_STORE_DIGEST_BYTE, TEST_NAR_HASH_BYTE);
        write_pathinfo_db(&state_dir, path_info.clone());
        let output = path_info.store_path.to_absolute_path_with_prefix("/mantle/store");
        let bundle = build_receipt_bundle(
            &[ReceiptRecordSpec {
                kind: ReceiptRecordKind::ActionRef,
                identity: "recipe:demo".to_string(),
                digest: format!("{BLAKE3_DIGEST_PREFIX}{}", "d".repeat(DIGEST_HEX_CHARS)),
            }],
            "/mantle/store",
            "policy-v1",
            ClaimStrengthRequest::Diagnostic,
        )
        .unwrap();

        let err = verify_receipt_bundle_against_state(
            &bundle,
            &state_dir,
            "/mantle/store",
            "policy-v1",
            std::slice::from_ref(&output),
        )
        .unwrap_err();

        assert!(err.to_string().contains("missing output-ref evidence"));
    }

    #[test]
    fn policy_hash_mismatch_rejects_output_verification() {
        let temp = tempfile::tempdir().unwrap();
        let state_dir = temp.path().join("state");
        let path_info = test_path_info("policy-demo", TEST_STORE_DIGEST_BYTE, TEST_NAR_HASH_BYTE);
        write_pathinfo_db(&state_dir, path_info.clone());
        let output = path_info.store_path.to_absolute_path_with_prefix("/mantle/store");
        let bundle = bundle_for_state_output(&state_dir, &output, "policy-v1");

        let err = verify_receipt_bundle_against_state(
            &bundle,
            &state_dir,
            "/mantle/store",
            "policy-v2",
            std::slice::from_ref(&output),
        )
        .unwrap_err();

        assert!(err.to_string().contains("policy hash mismatch"));
    }

    #[test]
    fn expired_trust_snapshot_rejects_output_verification() {
        let temp = tempfile::tempdir().unwrap();
        let state_dir = temp.path().join("state");
        let path_info = test_path_info("expired-demo", TEST_STORE_DIGEST_BYTE, TEST_NAR_HASH_BYTE);
        write_pathinfo_db(&state_dir, path_info.clone());
        let output = path_info.store_path.to_absolute_path_with_prefix("/mantle/store");
        let mut bundle = bundle_for_state_output(&state_dir, &output, "policy-v1");
        bundle.trust_snapshot.valid_before_unix_s = TEST_EXPIRED_BEFORE_UNIX_S;
        refresh_bundle_digest(&mut bundle);
        let context = verification_context("policy-v1", TEST_VALID_AT_UNIX_S);

        let err = verify_receipt_bundle_against_state_with_context(
            &bundle,
            &state_dir,
            "/mantle/store",
            &context,
            std::slice::from_ref(&output),
        )
        .unwrap_err();

        assert!(err.to_string().contains("expired"));
    }

    #[test]
    fn revoked_public_key_rejects_output_verification() {
        let temp = tempfile::tempdir().unwrap();
        let state_dir = temp.path().join("state");
        let path_info = test_path_info("revoked-demo", TEST_STORE_DIGEST_BYTE, TEST_NAR_HASH_BYTE);
        write_pathinfo_db(&state_dir, path_info.clone());
        let output = path_info.store_path.to_absolute_path_with_prefix("/mantle/store");
        let revoked = format!("{BLAKE3_DIGEST_PREFIX}{}", "e".repeat(DIGEST_HEX_CHARS));
        let mut bundle = bundle_for_state_output(&state_dir, &output, "policy-v1");
        bundle.trust_snapshot.public_key_digests = vec![revoked.clone()];
        bundle.trust_snapshot.revocation_ref = Some(TEST_REVOCATION_REF.to_string());
        refresh_bundle_digest(&mut bundle);
        let context = TrustVerificationContext::new(
            "policy-v1".to_string(),
            TEST_VALID_AT_UNIX_S,
            Some(TEST_REVOCATION_REF.to_string()),
            vec![revoked],
        );

        let err = verify_receipt_bundle_against_state_with_context(
            &bundle,
            &state_dir,
            "/mantle/store",
            &context,
            std::slice::from_ref(&output),
        )
        .unwrap_err();

        assert!(err.to_string().contains("revoked"));
    }

    #[test]
    fn verified_import_persists_graph_evidence_for_why() {
        let temp = tempfile::tempdir().unwrap();
        let bundle_state = temp.path().join("bundle-state");
        let import_state = temp.path().join("import-state");
        let path_info = test_path_info("graph-import-demo", TEST_STORE_DIGEST_BYTE, TEST_NAR_HASH_BYTE);
        write_pathinfo_db(&bundle_state, path_info.clone());
        write_pathinfo_db(&import_state, path_info.clone());
        let output = path_info.store_path.to_absolute_path_with_prefix("/mantle/store");
        write_source_record(&bundle_state, "src:demo", &"b".repeat(DIGEST_HEX_CHARS));
        write_semantic_graph(&bundle_state, &output);
        let bundle = bundle_for_state_output(&bundle_state, &output, "policy-v1");
        let context = verification_context("policy-v1", TEST_VALID_AT_UNIX_S);

        let report = import_verified_receipt_bundle(
            &bundle,
            &import_state,
            "/mantle/store",
            &context,
            std::slice::from_ref(&output),
        )
        .unwrap();

        assert!(report.imported);
        assert!(report.graph_imported);
        assert!(report.graph_nodes_imported > 0);
        assert!(report.graph_edges_imported > 0);
        let graph = crate::semantic_graph::SemanticGraph::load(&import_state.join(SEMANTIC_GRAPH_FILE)).unwrap();
        let why = graph.why(&output).unwrap();
        assert_eq!(why.producing_recipe.unwrap().id, "recipe:demo");
        assert_eq!(why.sources[0].id, "src:demo");
    }

    #[test]
    fn conflicting_graph_import_fails_without_persisting_bundle() {
        let temp = tempfile::tempdir().unwrap();
        let bundle_state = temp.path().join("bundle-state");
        let import_state = temp.path().join("import-state");
        let path_info = test_path_info("graph-conflict-demo", TEST_STORE_DIGEST_BYTE, TEST_NAR_HASH_BYTE);
        write_pathinfo_db(&bundle_state, path_info.clone());
        write_pathinfo_db(&import_state, path_info.clone());
        let output = path_info.store_path.to_absolute_path_with_prefix("/mantle/store");
        write_source_record(&bundle_state, "src:demo", &"b".repeat(DIGEST_HEX_CHARS));
        write_semantic_graph(&bundle_state, &output);
        write_conflicting_semantic_node(&import_state, "recipe:demo");
        let bundle = bundle_for_state_output(&bundle_state, &output, "policy-v1");
        let context = verification_context("policy-v1", TEST_VALID_AT_UNIX_S);

        let err = import_verified_receipt_bundle(
            &bundle,
            &import_state,
            "/mantle/store",
            &context,
            std::slice::from_ref(&output),
        )
        .unwrap_err();

        assert!(err.to_string().contains("semantic graph node conflict"));
        assert!(!receipt_bundle_state_file(&import_state, &bundle).exists());
    }

    #[test]
    fn missing_output_state_does_not_fabricate_records() {
        let temp = tempfile::tempdir().unwrap();
        let records = gather_receipt_records_from_state(temp.path(), "/mantle/store", &[
            "/mantle/store/22222222222222222222222222222222-missing".to_string(),
        ])
        .unwrap();
        let err = build_receipt_bundle_from_records(
            records,
            "/mantle/store",
            "policy-v1",
            ClaimStrengthRequest::StrongActionCorrectness,
        )
        .unwrap_err();
        assert!(err.to_string().contains("at least one evidence record"));
    }

    #[test]
    fn metadata_path_traversal_is_rejected() {
        let mut metadata = BTreeMap::new();
        metadata.insert("sidecar_file".to_string(), "../artifact.json".to_string());
        let record = ReceiptRecord {
            kind: ReceiptRecordKind::ArtifactAttestation,
            identity: "out".to_string(),
            digest: format!("{BLAKE3_DIGEST_PREFIX}{}", "a".repeat(DIGEST_HEX_CHARS)),
            metadata,
        };
        let err = build_receipt_bundle_from_records(
            vec![record],
            "/mantle/store",
            "policy-v1",
            ClaimStrengthRequest::Diagnostic,
        )
        .unwrap_err();
        assert!(err.to_string().contains("invalid receipt metadata value"));
    }

    const TEST_STORE_DIGEST_BYTE: u8 = 7;
    const TEST_NAR_HASH_BYTE: u8 = 0x11;
    const TEST_STALE_NAR_HASH_BYTE: u8 = 0x22;
    const TEST_NAR_SIZE_BYTES: u64 = 17;
    const TEST_VALID_AT_UNIX_S: u64 = 100;
    const TEST_EXPIRED_BEFORE_UNIX_S: u64 = 99;
    const TEST_REVOCATION_REF: &str = "revocations:v1";

    fn bundle_for_state_output(state_dir: &Path, output: &str, policy_hash: &str) -> ReceiptBundle {
        let records = gather_receipt_records_from_state(state_dir, "/mantle/store", &[output.to_string()]).unwrap();
        build_receipt_bundle_from_records(records, "/mantle/store", policy_hash, ClaimStrengthRequest::Diagnostic)
            .unwrap()
    }

    fn verification_context(policy_hash: &str, valid_at_unix_s: u64) -> TrustVerificationContext {
        TrustVerificationContext::new(policy_hash.to_string(), valid_at_unix_s, None, Vec::new())
    }

    fn refresh_bundle_digest(bundle: &mut ReceiptBundle) {
        bundle.bundle_blake3 = digest_bundle_without_digest(bundle).unwrap();
    }

    fn receipt_bundle_state_file(state_dir: &Path, bundle: &ReceiptBundle) -> PathBuf {
        state_dir
            .join(RECEIPT_STATE_DIR)
            .join(RECEIPT_RECORDS_DIR)
            .join(format!("{}.json", bundle.bundle_blake3))
    }

    fn test_store_path(name: &str, digest_byte: u8) -> nix_compat::store_path::StorePath<String> {
        nix_compat::store_path::StorePath::from_name_and_digest_fixed(name, [digest_byte; 20]).unwrap()
    }

    fn test_path_info(name: &str, digest_byte: u8, nar_hash_byte: u8) -> PathInfo {
        PathInfo {
            store_path: test_store_path(name, digest_byte),
            node: snix_castore::Node::Symlink {
                target: snix_castore::SymlinkTarget::try_from("target").unwrap(),
            },
            references: Vec::new(),
            nar_size: TEST_NAR_SIZE_BYTES,
            nar_sha256: [nar_hash_byte; 32],
            signatures: Vec::new(),
            deriver: None,
            ca: None,
        }
    }

    fn write_pathinfo_db(state_dir: &Path, path_info: PathInfo) {
        let db_path = state_dir.join(PATHINFO_DB_FILE);
        let runtime = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
        runtime.block_on(async move {
            let service = RedbPathInfoService::new("receipt-test".to_string(), RedbPathInfoServiceConfig {
                path: Some(db_path),
                cache_size: None,
                read_only: false,
            })
            .await
            .unwrap();
            service.put(path_info).await.unwrap();
        });
    }

    fn write_artifact_sidecar(
        state_dir: &Path,
        store_prefix: &str,
        store_path: &nix_compat::store_path::StorePath<String>,
        bytes: &[u8],
    ) {
        let path = crunch_store::artifact_attestation_file_path(state_dir, store_prefix, store_path);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, bytes).unwrap();
    }

    fn write_source_record(state_dir: &Path, identity: &str, content_blake3: &str) {
        let record = crate::source_bundle::SourceRecord {
            kind: crate::source_bundle::SourceRecordKind::LocalPath,
            identity: identity.to_string(),
            store_prefix: Some("/mantle/store".to_string()),
            adapter: None,
            metadata: BTreeMap::new(),
            payload_bytes: 0,
            content_blake3: content_blake3.to_string(),
            files: Vec::new(),
        };
        let records_dir = state_dir.join(SOURCE_STATE_DIR).join(SOURCE_RECORDS_DIR);
        fs::create_dir_all(&records_dir).unwrap();
        let rendered = serde_json::to_string_pretty(&record).unwrap();
        fs::write(records_dir.join(format!("{content_blake3}.json")), rendered).unwrap();
    }

    fn write_semantic_graph(state_dir: &Path, output: &str) {
        let node = |id: &str, kind: crate::semantic_graph::SemanticNodeKind| crate::semantic_graph::SemanticNode {
            id: id.to_string(),
            kind,
            digest: Some(format!("{BLAKE3_DIGEST_PREFIX}{}", "c".repeat(DIGEST_HEX_CHARS))),
            metadata: BTreeMap::new(),
        };
        let graph = crate::semantic_graph::SemanticGraph {
            schema: crate::semantic_graph::SEMANTIC_GRAPH_SCHEMA.to_string(),
            nodes: vec![
                node(output, crate::semantic_graph::SemanticNodeKind::StoreOutput),
                node("recipe:demo", crate::semantic_graph::SemanticNodeKind::Recipe),
                node("src:demo", crate::semantic_graph::SemanticNodeKind::SourceTree),
                node("sandbox:demo", crate::semantic_graph::SemanticNodeKind::SandboxProfile),
            ],
            edges: vec![
                semantic_edge(output, "recipe:demo", crate::semantic_graph::SemanticEdgeKind::ProducedBy),
                semantic_edge(output, "src:demo", crate::semantic_graph::SemanticEdgeKind::UsesSource),
                semantic_edge(output, "sandbox:demo", crate::semantic_graph::SemanticEdgeKind::UsesSandbox),
            ],
            aliases: Vec::new(),
        };
        graph.save(&state_dir.join(SEMANTIC_GRAPH_FILE)).unwrap();
    }

    fn write_conflicting_semantic_node(state_dir: &Path, node_id: &str) {
        let graph = crate::semantic_graph::SemanticGraph {
            schema: crate::semantic_graph::SEMANTIC_GRAPH_SCHEMA.to_string(),
            nodes: vec![crate::semantic_graph::SemanticNode {
                id: node_id.to_string(),
                kind: crate::semantic_graph::SemanticNodeKind::Provider,
                digest: Some(format!("{BLAKE3_DIGEST_PREFIX}{}", "f".repeat(DIGEST_HEX_CHARS))),
                metadata: BTreeMap::new(),
            }],
            edges: Vec::new(),
            aliases: Vec::new(),
        };
        graph.save(&state_dir.join(SEMANTIC_GRAPH_FILE)).unwrap();
    }

    fn semantic_edge(
        from: &str,
        to: &str,
        kind: crate::semantic_graph::SemanticEdgeKind,
    ) -> crate::semantic_graph::SemanticEdge {
        crate::semantic_graph::SemanticEdge {
            from: from.to_string(),
            to: to.to_string(),
            kind,
            metadata: BTreeMap::new(),
        }
    }
}
