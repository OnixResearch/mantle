#![allow(dead_code)]

use std::collections::BTreeMap;
use std::collections::BTreeSet;

use serde::Deserialize;
use serde::Serialize;

const GRAPH_SCHEMA: &str = "foreign-derivation-graph-v1";
const PACKAGE_INDEX_SCHEMA: &str = "foreign-package-index-v1";
const IMPORT_RECEIPT_SCHEMA: &str = "foreign-derivation-import-receipt-v1";
const MANTLE_ADAPTER_SCHEMA: &str = "mantle-foreign-derivation-adapter-plan-v1";
const DIGEST_ALGORITHM: &str = "blake3";
const RECOMPUTE_BLAKE3_MODE: &str = "recompute-blake3-v1";
const SOURCE_REF_FIELD: &str = "source-ref";
const OUT_OUTPUT_NAME: &str = "out";
const HELLO_PACKAGE_NAME: &str = "hello";
const HELLO_SYSTEM: &str = "x86_64-linux";
const DEFAULT_TARGET_PREFIX: &str = "/mantle/store";
const GUIX_SOURCE_PREFIX: &str = "/gnu/store";
const NIX_SOURCE_PREFIX: &str = "/nix/store";
const UNKNOWN_FOREIGN_PREFIX: &str = "/foreign/store";
const GUIX_HELLO_NODE_ID: &str = "guix:hello";
const NIX_HELLO_NODE_ID: &str = "nix:hello";
const CHMOD_SETUID_CAPABILITY: &str = "chmod-setuid";
const TRUSTED_CACHE_SCOPE: &str = "trusted-binary-cache";
const OUTPUT_HASH_HEX_CHARS: usize = 32;
const MAX_GRAPH_NODES: usize = 64;
const MAX_GRAPH_EDGES: usize = 256;
const MAX_PACKAGE_INDEX_ENTRIES: usize = 128;
const MAX_FIELD_BYTES: usize = 4096;
const MAX_SOURCE_PAYLOADS: usize = 64;
const MAX_MIRROR_CANDIDATES: usize = 16;
const MAX_SANDBOX_CAPABILITIES: usize = 16;
const MAX_CACHE_HINTS: usize = 16;
const MAX_UNSUPPORTED_FEATURES: usize = 32;
const EMPTY_OUTPUT_COUNT: usize = 0;
const EMPTY_COMMAND_INVOCATION_COUNT: usize = 0;
const REQUIRED_HELLO_ROOT_COUNT: usize = 1;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct ForeignDerivationGraph {
    pub(crate) schema: String,
    pub(crate) producer: ProducerSummary,
    pub(crate) source_store_prefixes: Vec<String>,
    pub(crate) target_store_prefix: Option<String>,
    pub(crate) root_derivation_ids: Vec<String>,
    pub(crate) nodes: Vec<ForeignDerivationNode>,
    pub(crate) source_payloads: Vec<SourcePayload>,
    pub(crate) unsupported_features: Vec<UnsupportedFeature>,
    pub(crate) frontend_metadata: Vec<FrontendMetadata>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct ProducerSummary {
    pub(crate) kind: String,
    pub(crate) identity: String,
    pub(crate) revision: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct ForeignDerivationNode {
    pub(crate) node_id: String,
    pub(crate) original_derivation: String,
    pub(crate) name: String,
    pub(crate) system: String,
    pub(crate) builder: String,
    pub(crate) args: Vec<String>,
    pub(crate) env: BTreeMap<String, String>,
    pub(crate) outputs: BTreeMap<String, OutputDeclaration>,
    pub(crate) input_derivations: Vec<InputDerivationEdge>,
    pub(crate) source_refs: Vec<SourceRef>,
    pub(crate) fixed_output: Option<FixedOutputMetadata>,
    pub(crate) builtin: String,
    pub(crate) declared_references: Vec<String>,
    pub(crate) sandbox_capabilities: Vec<String>,
    pub(crate) unsupported_features: Vec<UnsupportedFeature>,
    pub(crate) cache_hints: Vec<CacheHint>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct OutputDeclaration {
    pub(crate) path: String,
    pub(crate) hash: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct InputDerivationEdge {
    pub(crate) node_id: String,
    pub(crate) output_name: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct SourceRef {
    pub(crate) payload_id: String,
    pub(crate) field: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct FixedOutputMetadata {
    pub(crate) algorithm: String,
    pub(crate) digest: String,
    pub(crate) recursive: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct SourcePayload {
    pub(crate) payload_id: String,
    pub(crate) kind: String,
    pub(crate) content_ref: String,
    pub(crate) embedded_text: Option<String>,
    pub(crate) mirrors: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct UnsupportedFeature {
    pub(crate) class: String,
    pub(crate) mandatory: bool,
    pub(crate) message: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct FrontendMetadata {
    pub(crate) class: String,
    pub(crate) lowered: bool,
    pub(crate) message: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct CacheHint {
    pub(crate) cache_url: String,
    pub(crate) trust_scope: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct PackageIndex {
    pub(crate) schema: String,
    pub(crate) entries: Vec<PackageIndexEntry>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct PackageIndexEntry {
    pub(crate) name: String,
    pub(crate) system: String,
    pub(crate) root_derivation_id: String,
    pub(crate) aliases: Vec<String>,
    pub(crate) provenance_ref: String,
    pub(crate) metadata_digest: String,
    pub(crate) unsupported_metadata_classes: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct TranslationPolicy {
    pub(crate) source_prefixes: Vec<String>,
    pub(crate) target_prefix: String,
    pub(crate) rewrite_builder: bool,
    pub(crate) rewrite_args: bool,
    pub(crate) rewrite_env: bool,
    pub(crate) rewrite_sources: bool,
    pub(crate) rewrite_declared_references: bool,
    pub(crate) allow_embedded_source_payload_rewrite: bool,
    pub(crate) builtin_mappings: BTreeMap<String, String>,
    pub(crate) output_path_recompute_mode: String,
    pub(crate) trusted_cache_scopes: BTreeSet<String>,
    pub(crate) allowed_sandbox_capabilities: BTreeSet<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct TranslatedGraph {
    pub(crate) schema: String,
    pub(crate) target_store_prefix: String,
    pub(crate) root_derivation_ids: Vec<String>,
    pub(crate) nodes: Vec<ForeignDerivationNode>,
    pub(crate) source_payloads: Vec<SourcePayload>,
    pub(crate) diagnostics: Vec<ImportDiagnostic>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct ImportDiagnostic {
    pub(crate) class: String,
    pub(crate) node_id: Option<String>,
    pub(crate) message: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct ImportReceipt {
    pub(crate) schema: String,
    pub(crate) producer_identity: String,
    pub(crate) raw_graph_digest: String,
    pub(crate) translation_policy_digest: String,
    pub(crate) translated_graph_digest: String,
    pub(crate) package_index_digest: Option<String>,
    pub(crate) fetch_cache_policy_digest: String,
    pub(crate) sandbox_policy_digest: String,
    pub(crate) diagnostics: Vec<ImportDiagnostic>,
    pub(crate) non_claims: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct MantleForeignPlan {
    pub(crate) schema: String,
    pub(crate) roots: Vec<MantleForeignRoot>,
    pub(crate) source_payloads: Vec<SourcePayload>,
    pub(crate) sandbox_audit: Vec<SandboxAuditEvent>,
    pub(crate) forbidden_process_invocations: Vec<String>,
    pub(crate) non_claims: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct MantleForeignRoot {
    pub(crate) package_name: String,
    pub(crate) node_id: String,
    pub(crate) output_paths: BTreeMap<String, String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct SandboxAuditEvent {
    pub(crate) node_id: String,
    pub(crate) capability: String,
    pub(crate) classification: String,
}

pub(crate) fn translate_foreign_graph(
    graph: &ForeignDerivationGraph,
    package_index: Option<&PackageIndex>,
    policy: &TranslationPolicy,
) -> Result<(TranslatedGraph, ImportReceipt), ImportDiagnostic> {
    validate_graph(graph)?;
    validate_policy(policy)?;
    if let Some(index) = package_index {
        validate_package_index(index, graph)?;
    }
    let mut translated_nodes = Vec::with_capacity(graph.nodes.len());
    let mut diagnostics = Vec::new();
    for node in sorted_nodes(&graph.nodes) {
        translated_nodes.push(translate_node(node, policy, &mut diagnostics)?);
    }
    let translated_payloads = translate_source_payloads(graph, policy)?;
    let translated_graph = TranslatedGraph {
        schema: GRAPH_SCHEMA.to_string(),
        target_store_prefix: policy.target_prefix.clone(),
        root_derivation_ids: sorted_strings(graph.root_derivation_ids.clone()),
        nodes: translated_nodes,
        source_payloads: translated_payloads,
        diagnostics,
    };
    let receipt = import_receipt(graph, package_index, policy, &translated_graph)?;
    Ok((translated_graph, receipt))
}

pub(crate) fn plan_mantle_foreign_import(
    translated_graph: &TranslatedGraph,
    package_index: &PackageIndex,
    package_name: &str,
    system: &str,
) -> Result<MantleForeignPlan, ImportDiagnostic> {
    let entry = lookup_package(package_index, package_name, system)?;
    let root = translated_graph
        .nodes
        .iter()
        .find(|node| node.node_id == entry.root_derivation_id)
        .ok_or_else(|| diagnostic("missing-adapter-root", None, "package root is absent from translated graph"))?;
    let sandbox_audit = translated_graph
        .nodes
        .iter()
        .flat_map(|node| {
            node.sandbox_capabilities.iter().map(|capability| SandboxAuditEvent {
                node_id: node.node_id.clone(),
                capability: capability.clone(),
                classification: "declared-foreign-sandbox-capability".to_string(),
            })
        })
        .collect::<Vec<_>>();
    debug_assert_eq!(EMPTY_COMMAND_INVOCATION_COUNT, 0);
    Ok(MantleForeignPlan {
        schema: MANTLE_ADAPTER_SCHEMA.to_string(),
        roots: vec![MantleForeignRoot {
            package_name: entry.name.clone(),
            node_id: root.node_id.clone(),
            output_paths: root.outputs.iter().map(|(name, output)| (name.clone(), output.path.clone())).collect(),
        }],
        source_payloads: translated_graph.source_payloads.clone(),
        sandbox_audit,
        forbidden_process_invocations: Vec::new(),
        non_claims: foreign_import_non_claims(),
    })
}

pub(crate) fn lookup_package<'a>(
    index: &'a PackageIndex,
    package_name: &str,
    system: &str,
) -> Result<&'a PackageIndexEntry, ImportDiagnostic> {
    validate_package_index_shape(index)?;
    let matches = index
        .entries
        .iter()
        .filter(|entry| entry.system == system)
        .filter(|entry| entry.name == package_name || entry.aliases.iter().any(|alias| alias == package_name))
        .collect::<Vec<_>>();
    match matches.as_slice() {
        [entry] => Ok(entry),
        [] => Err(diagnostic("missing-package-index-entry", None, "package index does not contain requested package")),
        _ => Err(diagnostic(
            "ambiguous-package-index-entry",
            None,
            "package index contains multiple matching packages",
        )),
    }
}

pub(crate) fn admit_translated_graph(
    graph: &ForeignDerivationGraph,
    package_index: Option<&PackageIndex>,
    policy: &TranslationPolicy,
    receipt: &ImportReceipt,
) -> Result<(), ImportDiagnostic> {
    let (_, expected) = translate_foreign_graph(graph, package_index, policy)?;
    if expected.raw_graph_digest != receipt.raw_graph_digest {
        return Err(diagnostic(
            "stale-raw-graph-digest",
            None,
            "receipt raw graph digest does not match current graph",
        ));
    }
    if expected.translation_policy_digest != receipt.translation_policy_digest {
        return Err(diagnostic(
            "stale-translation-policy-digest",
            None,
            "receipt translation policy digest does not match current policy",
        ));
    }
    if expected.translated_graph_digest != receipt.translated_graph_digest {
        return Err(diagnostic(
            "stale-translated-graph-digest",
            None,
            "receipt translated graph digest does not match current translation",
        ));
    }
    Ok(())
}

pub(crate) fn guix_like_hello_fixture() -> (ForeignDerivationGraph, PackageIndex) {
    hello_fixture("guix", GUIX_SOURCE_PREFIX, GUIX_HELLO_NODE_ID, "guix-time-machine:hello")
}

pub(crate) fn nix_like_hello_fixture() -> (ForeignDerivationGraph, PackageIndex) {
    hello_fixture("nix", NIX_SOURCE_PREFIX, NIX_HELLO_NODE_ID, "nix-derivation-json:hello")
}

fn hello_fixture(
    producer_kind: &str,
    source_prefix: &str,
    node_id: &str,
    producer_identity: &str,
) -> (ForeignDerivationGraph, PackageIndex) {
    let payload_id = format!("{producer_kind}-hello-source");
    let store_source = format!("{source_prefix}/00000000000000000000000000000000-hello-source");
    let output_path = format!("{source_prefix}/11111111111111111111111111111111-hello");
    let mut outputs = BTreeMap::new();
    outputs.insert(OUT_OUTPUT_NAME.to_string(), OutputDeclaration {
        path: output_path.clone(),
        hash: None,
    });
    let mut env = BTreeMap::new();
    env.insert("src".to_string(), store_source.clone());
    env.insert("out".to_string(), output_path.clone());
    let graph = ForeignDerivationGraph {
        schema: GRAPH_SCHEMA.to_string(),
        producer: ProducerSummary {
            kind: producer_kind.to_string(),
            identity: producer_identity.to_string(),
            revision: "fixture-revision".to_string(),
        },
        source_store_prefixes: vec![source_prefix.to_string()],
        target_store_prefix: None,
        root_derivation_ids: vec![node_id.to_string()],
        nodes: vec![ForeignDerivationNode {
            node_id: node_id.to_string(),
            original_derivation: format!("{source_prefix}/22222222222222222222222222222222-hello.drv"),
            name: HELLO_PACKAGE_NAME.to_string(),
            system: HELLO_SYSTEM.to_string(),
            builder: format!("{source_prefix}/33333333333333333333333333333333-bash/bin/bash"),
            args: vec!["-c".to_string(), format!("cp {store_source} $out")],
            env,
            outputs,
            input_derivations: Vec::new(),
            source_refs: vec![SourceRef {
                payload_id: payload_id.clone(),
                field: SOURCE_REF_FIELD.to_string(),
            }],
            fixed_output: Some(FixedOutputMetadata {
                algorithm: DIGEST_ALGORITHM.to_string(),
                digest: blake3_hex("hello-source"),
                recursive: true,
            }),
            builtin: "fixed-output-fetch".to_string(),
            declared_references: vec![store_source.clone()],
            sandbox_capabilities: Vec::new(),
            unsupported_features: Vec::new(),
            cache_hints: Vec::new(),
        }],
        source_payloads: vec![SourcePayload {
            payload_id,
            kind: "fixed-output-source".to_string(),
            content_ref: store_source,
            embedded_text: None,
            mirrors: vec!["https://mirror.example.invalid/hello.tar.gz".to_string()],
        }],
        unsupported_features: Vec::new(),
        frontend_metadata: Vec::new(),
    };
    let index = PackageIndex {
        schema: PACKAGE_INDEX_SCHEMA.to_string(),
        entries: vec![PackageIndexEntry {
            name: HELLO_PACKAGE_NAME.to_string(),
            system: HELLO_SYSTEM.to_string(),
            root_derivation_id: node_id.to_string(),
            aliases: vec![format!("{producer_kind}-hello")],
            provenance_ref: producer_identity.to_string(),
            metadata_digest: blake3_hex(producer_identity),
            unsupported_metadata_classes: Vec::new(),
        }],
    };
    (graph, index)
}

fn translate_node(
    node: &ForeignDerivationNode,
    policy: &TranslationPolicy,
    diagnostics: &mut Vec<ImportDiagnostic>,
) -> Result<ForeignDerivationNode, ImportDiagnostic> {
    validate_node_policy(node, policy)?;
    let mut translated = node.clone();
    translated.builtin = policy.builtin_mappings.get(&node.builtin).cloned().ok_or_else(|| {
        diagnostic("unsupported-builtin", Some(&node.node_id), "builtin operation is not mapped by policy")
    })?;
    if policy.rewrite_builder {
        translated.builder = rewrite_value(&node.builder, policy, Some(&node.node_id), "builder")?;
    }
    if policy.rewrite_args {
        translated.args = node
            .args
            .iter()
            .map(|arg| rewrite_value(arg, policy, Some(&node.node_id), "arg"))
            .collect::<Result<Vec<_>, _>>()?;
    }
    if policy.rewrite_env {
        translated.env = node
            .env
            .iter()
            .map(|(key, value)| {
                rewrite_value(value, policy, Some(&node.node_id), key).map(|value| (key.clone(), value))
            })
            .collect::<Result<BTreeMap<_, _>, _>>()?;
    }
    if policy.rewrite_declared_references {
        translated.declared_references = node
            .declared_references
            .iter()
            .map(|reference| rewrite_value(reference, policy, Some(&node.node_id), "declared-reference"))
            .collect::<Result<Vec<_>, _>>()?;
    }
    translated.outputs = recompute_outputs(node, policy)?;
    translated.input_derivations.sort();
    translated.source_refs.sort();
    translated.declared_references.sort();
    translated.sandbox_capabilities.sort();
    diagnostics.extend(node.sandbox_capabilities.iter().map(|capability| ImportDiagnostic {
        class: "sandbox-capability-declared".to_string(),
        node_id: Some(node.node_id.clone()),
        message: format!("capability {capability} is explicitly policy-allowed for this derivation"),
    }));
    Ok(translated)
}

fn validate_node_policy(node: &ForeignDerivationNode, policy: &TranslationPolicy) -> Result<(), ImportDiagnostic> {
    if node.unsupported_features.iter().any(|feature| feature.mandatory) {
        return Err(diagnostic(
            "unsupported-mandatory-feature",
            Some(&node.node_id),
            "node contains an unsupported mandatory feature",
        ));
    }
    if node.cache_hints.len() > MAX_CACHE_HINTS {
        return Err(diagnostic("cache-hint-limit-exceeded", Some(&node.node_id), "node declares too many cache hints"));
    }
    for hint in &node.cache_hints {
        if !policy.trusted_cache_scopes.contains(&hint.trust_scope) {
            return Err(diagnostic(
                "untrusted-cache-hint",
                Some(&node.node_id),
                "cache hint trust scope is not accepted by policy",
            ));
        }
    }
    if node.sandbox_capabilities.len() > MAX_SANDBOX_CAPABILITIES {
        return Err(diagnostic(
            "sandbox-capability-limit-exceeded",
            Some(&node.node_id),
            "node declares too many sandbox capabilities",
        ));
    }
    for capability in &node.sandbox_capabilities {
        if !policy.allowed_sandbox_capabilities.contains(capability) {
            return Err(diagnostic(
                "undeclared-sandbox-capability",
                Some(&node.node_id),
                "sandbox capability is not accepted by policy",
            ));
        }
    }
    Ok(())
}

fn translate_source_payloads(
    graph: &ForeignDerivationGraph,
    policy: &TranslationPolicy,
) -> Result<Vec<SourcePayload>, ImportDiagnostic> {
    let mut payloads = sorted_payloads(&graph.source_payloads);
    for payload in &mut payloads {
        if payload.mirrors.len() > MAX_MIRROR_CANDIDATES {
            return Err(diagnostic(
                "mirror-limit-exceeded",
                None,
                "source payload declares too many mirror candidates",
            ));
        }
        if policy.rewrite_sources {
            payload.content_ref = rewrite_value(&payload.content_ref, policy, None, "source-payload")?;
        }
        if let Some(text) = payload.embedded_text.as_ref() {
            if !policy.allow_embedded_source_payload_rewrite && contains_declared_source_prefix(text, policy) {
                return Err(diagnostic(
                    "undeclared-embedded-source-rewrite",
                    None,
                    "source payload contains embedded foreign store path without rewrite permission",
                ));
            }
            payload.embedded_text = Some(rewrite_value(text, policy, None, "embedded-source-payload")?);
        }
    }
    Ok(payloads)
}

fn recompute_outputs(
    node: &ForeignDerivationNode,
    policy: &TranslationPolicy,
) -> Result<BTreeMap<String, OutputDeclaration>, ImportDiagnostic> {
    if policy.output_path_recompute_mode != RECOMPUTE_BLAKE3_MODE {
        return Err(diagnostic(
            "unsupported-output-recompute-mode",
            Some(&node.node_id),
            "output path recomputation mode is not supported",
        ));
    }
    if node.outputs.len() == EMPTY_OUTPUT_COUNT {
        return Err(diagnostic("missing-output-declaration", Some(&node.node_id), "node has no output declarations"));
    }
    let mut outputs = BTreeMap::new();
    for (output_name, output) in &node.outputs {
        let digest_input = format!("{}:{}:{}:{}", node.node_id, node.name, output_name, policy.target_prefix);
        let digest = blake3_hex(&digest_input);
        let short_digest = &digest[..OUTPUT_HASH_HEX_CHARS];
        outputs.insert(output_name.clone(), OutputDeclaration {
            path: format!("{}/{short_digest}-{}", policy.target_prefix, node.name),
            hash: output.hash.clone(),
        });
    }
    Ok(outputs)
}

fn validate_graph(graph: &ForeignDerivationGraph) -> Result<(), ImportDiagnostic> {
    if graph.schema != GRAPH_SCHEMA {
        return Err(diagnostic("unsupported-graph-schema", None, "foreign graph schema is unsupported"));
    }
    if graph.nodes.is_empty() || graph.nodes.len() > MAX_GRAPH_NODES {
        return Err(diagnostic("graph-node-count-out-of-range", None, "graph node count is outside supported limits"));
    }
    if graph.source_payloads.len() > MAX_SOURCE_PAYLOADS {
        return Err(diagnostic("source-payload-limit-exceeded", None, "graph has too many source payloads"));
    }
    if graph.unsupported_features.len() > MAX_UNSUPPORTED_FEATURES {
        return Err(diagnostic(
            "unsupported-feature-limit-exceeded",
            None,
            "graph has too many unsupported feature records",
        ));
    }
    if graph.unsupported_features.iter().any(|feature| feature.mandatory) {
        return Err(diagnostic(
            "unsupported-mandatory-feature",
            None,
            "graph contains an unsupported mandatory feature",
        ));
    }
    if graph.frontend_metadata.iter().any(|metadata| !metadata.lowered) {
        return Err(diagnostic(
            "unsupported-frontend-composition-metadata",
            None,
            "frontend metadata was not lowered into graph or package-index data",
        ));
    }
    validate_unique_ids(graph)?;
    validate_roots(graph)?;
    validate_source_refs(graph)?;
    validate_field_limits(graph)?;
    Ok(())
}

fn validate_unique_ids(graph: &ForeignDerivationGraph) -> Result<(), ImportDiagnostic> {
    let mut node_ids = BTreeSet::new();
    for node in &graph.nodes {
        if !node_ids.insert(&node.node_id) {
            return Err(diagnostic("duplicate-node-id", Some(&node.node_id), "duplicate node id"));
        }
        let mut output_names = BTreeSet::new();
        for output_name in node.outputs.keys() {
            if !output_names.insert(output_name) {
                return Err(diagnostic("duplicate-output-name", Some(&node.node_id), "duplicate output name"));
            }
        }
    }
    let mut payload_ids = BTreeSet::new();
    for payload in &graph.source_payloads {
        if !payload_ids.insert(&payload.payload_id) {
            return Err(diagnostic("duplicate-source-payload-id", None, "duplicate source payload id"));
        }
    }
    Ok(())
}

fn validate_roots(graph: &ForeignDerivationGraph) -> Result<(), ImportDiagnostic> {
    let node_ids = graph.nodes.iter().map(|node| node.node_id.as_str()).collect::<BTreeSet<_>>();
    if graph.root_derivation_ids.is_empty() {
        return Err(diagnostic("missing-root-derivation", None, "foreign graph has no root derivation identities"));
    }
    for root in &graph.root_derivation_ids {
        if !node_ids.contains(root.as_str()) {
            return Err(diagnostic(
                "dangling-root-derivation",
                None,
                "root derivation identity is not present as a node",
            ));
        }
    }
    for node in &graph.nodes {
        if node.input_derivations.len() > MAX_GRAPH_EDGES {
            return Err(diagnostic(
                "edge-limit-exceeded",
                Some(&node.node_id),
                "node has too many input derivation edges",
            ));
        }
        for edge in &node.input_derivations {
            if !node_ids.contains(edge.node_id.as_str()) {
                return Err(diagnostic(
                    "dangling-input-derivation",
                    Some(&node.node_id),
                    "input derivation edge points at missing node",
                ));
            }
        }
    }
    Ok(())
}

fn validate_source_refs(graph: &ForeignDerivationGraph) -> Result<(), ImportDiagnostic> {
    let payload_ids = graph.source_payloads.iter().map(|payload| payload.payload_id.as_str()).collect::<BTreeSet<_>>();
    for node in &graph.nodes {
        for source_ref in &node.source_refs {
            if !payload_ids.contains(source_ref.payload_id.as_str()) {
                return Err(diagnostic(
                    "missing-source-payload-ref",
                    Some(&node.node_id),
                    "node references missing source payload",
                ));
            }
        }
    }
    Ok(())
}

fn validate_field_limits(graph: &ForeignDerivationGraph) -> Result<(), ImportDiagnostic> {
    require_field_limit(&graph.producer.identity, None, "producer.identity")?;
    for prefix in &graph.source_store_prefixes {
        require_field_limit(prefix, None, "source_store_prefix")?;
    }
    for node in &graph.nodes {
        require_field_limit(&node.node_id, Some(&node.node_id), "node_id")?;
        require_field_limit(&node.original_derivation, Some(&node.node_id), "original_derivation")?;
        require_field_limit(&node.name, Some(&node.node_id), "name")?;
        require_field_limit(&node.builder, Some(&node.node_id), "builder")?;
        for arg in &node.args {
            require_field_limit(arg, Some(&node.node_id), "arg")?;
        }
        for value in node.env.values() {
            require_field_limit(value, Some(&node.node_id), "env")?;
        }
    }
    Ok(())
}

fn require_field_limit(value: &str, node_id: Option<&str>, field: &str) -> Result<(), ImportDiagnostic> {
    if value.len() > MAX_FIELD_BYTES {
        return Err(diagnostic("field-limit-exceeded", node_id, &format!("field {field} exceeds byte limit")));
    }
    Ok(())
}

fn validate_policy(policy: &TranslationPolicy) -> Result<(), ImportDiagnostic> {
    if policy.source_prefixes.is_empty() {
        return Err(diagnostic("missing-source-prefix-policy", None, "translation policy has no source prefixes"));
    }
    if policy.target_prefix.is_empty() {
        return Err(diagnostic("missing-target-prefix-policy", None, "translation policy has no target prefix"));
    }
    Ok(())
}

fn validate_package_index(index: &PackageIndex, graph: &ForeignDerivationGraph) -> Result<(), ImportDiagnostic> {
    validate_package_index_shape(index)?;
    let node_ids = graph.nodes.iter().map(|node| node.node_id.as_str()).collect::<BTreeSet<_>>();
    for entry in &index.entries {
        if !node_ids.contains(entry.root_derivation_id.as_str()) {
            return Err(diagnostic("package-index-dangling-root", None, "package index root is not in graph"));
        }
    }
    Ok(())
}

fn validate_package_index_shape(index: &PackageIndex) -> Result<(), ImportDiagnostic> {
    if index.schema != PACKAGE_INDEX_SCHEMA {
        return Err(diagnostic("unsupported-package-index-schema", None, "package index schema is unsupported"));
    }
    if index.entries.is_empty() || index.entries.len() > MAX_PACKAGE_INDEX_ENTRIES {
        return Err(diagnostic(
            "package-index-count-out-of-range",
            None,
            "package index entry count is outside limits",
        ));
    }
    let mut keys = BTreeSet::new();
    for entry in &index.entries {
        if !entry.unsupported_metadata_classes.is_empty() {
            return Err(diagnostic(
                "unsupported-package-index-metadata",
                None,
                "package index entry contains unsupported frontend composition metadata",
            ));
        }
        let key = (&entry.name, &entry.system);
        if !keys.insert(key) {
            return Err(diagnostic(
                "duplicate-package-index-entry",
                None,
                "package index contains duplicate name/system",
            ));
        }
    }
    Ok(())
}

fn import_receipt(
    graph: &ForeignDerivationGraph,
    package_index: Option<&PackageIndex>,
    policy: &TranslationPolicy,
    translated_graph: &TranslatedGraph,
) -> Result<ImportReceipt, ImportDiagnostic> {
    let raw_graph_digest = canonical_digest(graph)?;
    let translation_policy_digest = canonical_digest(policy)?;
    let translated_graph_digest = canonical_digest(translated_graph)?;
    let package_index_digest = package_index.map(canonical_digest).transpose()?;
    Ok(ImportReceipt {
        schema: IMPORT_RECEIPT_SCHEMA.to_string(),
        producer_identity: graph.producer.identity.clone(),
        raw_graph_digest,
        translation_policy_digest,
        translated_graph_digest,
        package_index_digest,
        fetch_cache_policy_digest: blake3_hex(&format!("cache:{:?}", policy.trusted_cache_scopes)),
        sandbox_policy_digest: blake3_hex(&format!("sandbox:{:?}", policy.allowed_sandbox_capabilities)),
        diagnostics: translated_graph.diagnostics.clone(),
        non_claims: foreign_import_non_claims(),
    })
}

fn rewrite_value(
    value: &str,
    policy: &TranslationPolicy,
    node_id: Option<&str>,
    field: &str,
) -> Result<String, ImportDiagnostic> {
    let mut rewritten = value.to_string();
    for prefix in &policy.source_prefixes {
        rewritten = rewritten.replace(prefix, &policy.target_prefix);
    }
    if contains_known_foreign_store_path(&rewritten, policy) {
        return Err(diagnostic(
            "undeclared-foreign-reference",
            node_id,
            &format!("field {field} contains a foreign store path outside declared rewrite policy"),
        ));
    }
    Ok(rewritten)
}

fn contains_known_foreign_store_path(value: &str, policy: &TranslationPolicy) -> bool {
    [GUIX_SOURCE_PREFIX, NIX_SOURCE_PREFIX, UNKNOWN_FOREIGN_PREFIX]
        .iter()
        .any(|prefix| value.contains(prefix) && !policy.source_prefixes.iter().any(|declared| declared == prefix))
}

fn contains_declared_source_prefix(value: &str, policy: &TranslationPolicy) -> bool {
    policy.source_prefixes.iter().any(|prefix| value.contains(prefix))
}

fn sorted_nodes(nodes: &[ForeignDerivationNode]) -> Vec<&ForeignDerivationNode> {
    let mut sorted = nodes.iter().collect::<Vec<_>>();
    sorted.sort_by(|left, right| left.node_id.cmp(&right.node_id));
    sorted
}

fn sorted_payloads(payloads: &[SourcePayload]) -> Vec<SourcePayload> {
    let mut sorted = payloads.to_vec();
    sorted.sort_by(|left, right| left.payload_id.cmp(&right.payload_id));
    sorted
}

fn sorted_strings(mut values: Vec<String>) -> Vec<String> {
    values.sort();
    values.dedup();
    values
}

fn canonical_digest<T: Serialize>(value: &T) -> Result<String, ImportDiagnostic> {
    serde_json::to_vec(value)
        .map(|bytes| blake3::hash(&bytes).to_hex().to_string())
        .map_err(|err| diagnostic("canonical-serialization-failed", None, &format!("serialize canonical value: {err}")))
}

fn blake3_hex(value: &str) -> String {
    blake3::hash(value.as_bytes()).to_hex().to_string()
}

fn diagnostic(class: &str, node_id: Option<&str>, message: &str) -> ImportDiagnostic {
    ImportDiagnostic {
        class: class.to_string(),
        node_id: node_id.map(ToOwned::to_owned),
        message: message.to_string(),
    }
}

fn foreign_import_non_claims() -> Vec<String> {
    vec![
        "not-build-success".to_string(),
        "not-package-correctness".to_string(),
        "not-bootstrap-parity".to_string(),
        "not-output-trust".to_string(),
        "not-reproducibility".to_string(),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    const INVALID_BUILTIN: &str = "unsupported:magic";
    const OVERLAY_METADATA_CLASS: &str = "nix-overlay-order";
    const OVERSIZED_FIELD_BYTES: usize = MAX_FIELD_BYTES + 1;

    #[test]
    fn translates_guix_and_nix_hello_fixtures_deterministically() {
        let policy = fixture_policy(&[GUIX_SOURCE_PREFIX, NIX_SOURCE_PREFIX]);
        let (guix_graph, guix_index) = guix_like_hello_fixture();
        let (nix_graph, nix_index) = nix_like_hello_fixture();
        let mut reordered_guix = guix_graph.clone();
        reordered_guix.nodes.reverse();
        reordered_guix.source_payloads.reverse();

        let (translated_guix, guix_receipt) = translate_foreign_graph(&guix_graph, Some(&guix_index), &policy).unwrap();
        let (reordered_translated_guix, reordered_receipt) =
            translate_foreign_graph(&reordered_guix, Some(&guix_index), &policy).unwrap();
        let (translated_nix, nix_receipt) = translate_foreign_graph(&nix_graph, Some(&nix_index), &policy).unwrap();

        assert_eq!(translated_guix.nodes.len(), REQUIRED_HELLO_ROOT_COUNT);
        assert_eq!(translated_nix.nodes.len(), REQUIRED_HELLO_ROOT_COUNT);
        assert_eq!(guix_receipt.raw_graph_digest, reordered_receipt.raw_graph_digest);
        assert_eq!(guix_receipt.translated_graph_digest, reordered_receipt.translated_graph_digest);
        assert!(translated_guix.nodes[0].builder.starts_with(DEFAULT_TARGET_PREFIX));
        assert!(translated_nix.nodes[0].builder.starts_with(DEFAULT_TARGET_PREFIX));
        assert!(reordered_translated_guix.nodes[0].outputs[OUT_OUTPUT_NAME].path.starts_with(DEFAULT_TARGET_PREFIX));
        assert_eq!(guix_receipt.schema, IMPORT_RECEIPT_SCHEMA);
        assert_eq!(nix_receipt.schema, IMPORT_RECEIPT_SCHEMA);
    }

    #[test]
    fn mantle_adapter_consumes_translated_hello_without_foreign_frontend_invocations() {
        let policy = fixture_policy(&[GUIX_SOURCE_PREFIX]);
        let (graph, index) = guix_like_hello_fixture();
        let (translated, _receipt) = translate_foreign_graph(&graph, Some(&index), &policy).unwrap();

        let plan = plan_mantle_foreign_import(&translated, &index, HELLO_PACKAGE_NAME, HELLO_SYSTEM).unwrap();

        assert_eq!(plan.schema, MANTLE_ADAPTER_SCHEMA);
        assert_eq!(plan.roots.len(), REQUIRED_HELLO_ROOT_COUNT);
        assert_eq!(plan.roots[0].node_id, GUIX_HELLO_NODE_ID);
        assert!(plan.roots[0].output_paths[OUT_OUTPUT_NAME].starts_with(DEFAULT_TARGET_PREFIX));
        assert!(plan.forbidden_process_invocations.is_empty());
        assert!(!plan.non_claims.is_empty());
        assert!(!plan.non_claims.iter().any(|claim| claim == "build-success"));
    }

    #[test]
    fn translation_rejects_unsupported_builtins_references_stale_receipts_indexes_and_features() {
        let policy = fixture_policy(&[GUIX_SOURCE_PREFIX]);
        let (graph, index) = guix_like_hello_fixture();
        let (_translated, receipt) = translate_foreign_graph(&graph, Some(&index), &policy).unwrap();

        let mut unsupported_builtin = graph.clone();
        unsupported_builtin.nodes[0].builtin = INVALID_BUILTIN.to_string();
        assert_error_class(translate_foreign_graph(&unsupported_builtin, Some(&index), &policy), "unsupported-builtin");

        let mut undeclared_ref = graph.clone();
        undeclared_ref.nodes[0].args.push(format!("cat {UNKNOWN_FOREIGN_PREFIX}/bad-input"));
        assert_error_class(
            translate_foreign_graph(&undeclared_ref, Some(&index), &policy),
            "undeclared-foreign-reference",
        );

        let mut stale_graph = graph.clone();
        stale_graph.nodes[0].name = "hello-renamed".to_string();
        assert_error_class(
            admit_translated_graph(&stale_graph, Some(&index), &policy, &receipt),
            "stale-raw-graph-digest",
        );

        let mut malformed_index = index.clone();
        malformed_index.entries[0].unsupported_metadata_classes.push(OVERLAY_METADATA_CLASS.to_string());
        assert_error_class(
            translate_foreign_graph(&graph, Some(&malformed_index), &policy),
            "unsupported-package-index-metadata",
        );

        let mut oversized = graph.clone();
        oversized.nodes[0].builder = "x".repeat(OVERSIZED_FIELD_BYTES);
        assert_error_class(translate_foreign_graph(&oversized, Some(&index), &policy), "field-limit-exceeded");

        let mut mandatory_feature = graph.clone();
        mandatory_feature.nodes[0].unsupported_features.push(UnsupportedFeature {
            class: "frontend-only-feature".to_string(),
            mandatory: true,
            message: "cannot lower feature".to_string(),
        });
        assert_error_class(
            translate_foreign_graph(&mandatory_feature, Some(&index), &policy),
            "unsupported-mandatory-feature",
        );

        let mut frontend_metadata = graph.clone();
        frontend_metadata.frontend_metadata.push(FrontendMetadata {
            class: OVERLAY_METADATA_CLASS.to_string(),
            lowered: false,
            message: "overlay order is not graph data".to_string(),
        });
        assert_error_class(
            translate_foreign_graph(&frontend_metadata, Some(&index), &policy),
            "unsupported-frontend-composition-metadata",
        );
    }

    #[test]
    fn cache_and_sandbox_policy_fail_closed_without_trust_or_capability_allowance() {
        let policy = fixture_policy(&[GUIX_SOURCE_PREFIX]);
        let (mut graph, index) = guix_like_hello_fixture();
        graph.nodes[0].cache_hints.push(CacheHint {
            cache_url: "https://cache.example.invalid".to_string(),
            trust_scope: TRUSTED_CACHE_SCOPE.to_string(),
        });
        assert_error_class(translate_foreign_graph(&graph, Some(&index), &policy), "untrusted-cache-hint");

        let mut trusted_policy = policy.clone();
        trusted_policy.trusted_cache_scopes.insert(TRUSTED_CACHE_SCOPE.to_string());
        let (translated_with_cache, _receipt) = translate_foreign_graph(&graph, Some(&index), &trusted_policy).unwrap();
        assert_eq!(translated_with_cache.nodes[0].cache_hints.len(), REQUIRED_HELLO_ROOT_COUNT);

        let (mut sandbox_graph, sandbox_index) = guix_like_hello_fixture();
        sandbox_graph.nodes[0].sandbox_capabilities.push(CHMOD_SETUID_CAPABILITY.to_string());
        assert_error_class(
            translate_foreign_graph(&sandbox_graph, Some(&sandbox_index), &policy),
            "undeclared-sandbox-capability",
        );

        let mut sandbox_policy = policy;
        sandbox_policy.allowed_sandbox_capabilities.insert(CHMOD_SETUID_CAPABILITY.to_string());
        let (translated_with_sandbox, _receipt) =
            translate_foreign_graph(&sandbox_graph, Some(&sandbox_index), &sandbox_policy).unwrap();
        let plan =
            plan_mantle_foreign_import(&translated_with_sandbox, &sandbox_index, HELLO_PACKAGE_NAME, HELLO_SYSTEM)
                .unwrap();
        assert_eq!(plan.sandbox_audit.len(), REQUIRED_HELLO_ROOT_COUNT);
        assert_eq!(plan.sandbox_audit[0].capability, CHMOD_SETUID_CAPABILITY);
    }

    #[test]
    fn embedded_source_payload_rewrite_requires_explicit_permission() {
        let policy = fixture_policy(&[GUIX_SOURCE_PREFIX]);
        let (mut graph, index) = guix_like_hello_fixture();
        graph.source_payloads[0].embedded_text = Some(format!("source mentions {GUIX_SOURCE_PREFIX}/embedded"));

        assert_error_class(
            translate_foreign_graph(&graph, Some(&index), &policy),
            "undeclared-embedded-source-rewrite",
        );

        let mut permissive_policy = policy;
        permissive_policy.allow_embedded_source_payload_rewrite = true;
        let (translated, _receipt) = translate_foreign_graph(&graph, Some(&index), &permissive_policy).unwrap();
        let embedded = translated.source_payloads[0].embedded_text.as_deref().unwrap();
        assert!(embedded.contains(DEFAULT_TARGET_PREFIX));
        assert!(!embedded.contains(GUIX_SOURCE_PREFIX));
    }

    fn fixture_policy(source_prefixes: &[&str]) -> TranslationPolicy {
        let mut builtin_mappings = BTreeMap::new();
        builtin_mappings.insert("fixed-output-fetch".to_string(), "mantle.fetch".to_string());
        TranslationPolicy {
            source_prefixes: source_prefixes.iter().map(|prefix| (*prefix).to_string()).collect(),
            target_prefix: DEFAULT_TARGET_PREFIX.to_string(),
            rewrite_builder: true,
            rewrite_args: true,
            rewrite_env: true,
            rewrite_sources: true,
            rewrite_declared_references: true,
            allow_embedded_source_payload_rewrite: false,
            builtin_mappings,
            output_path_recompute_mode: RECOMPUTE_BLAKE3_MODE.to_string(),
            trusted_cache_scopes: BTreeSet::new(),
            allowed_sandbox_capabilities: BTreeSet::new(),
        }
    }

    fn assert_error_class<T>(result: Result<T, ImportDiagnostic>, expected_class: &str) {
        let err = match result {
            Ok(_) => panic!("expected {expected_class} diagnostic"),
            Err(err) => err,
        };
        assert_eq!(err.class, expected_class);
        assert!(!err.message.is_empty());
    }
}
