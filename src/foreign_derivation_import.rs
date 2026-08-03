#![allow(dead_code)]

// r[impl foreign_derivation_import.prefix_aware_aterm]
// r[verify foreign_derivation_import.prefix_aware_aterm]

use std::collections::BTreeMap;
use std::collections::BTreeSet;

use data_encoding::HEXLOWER;
use serde::Deserialize;
use serde::Serialize;

const GRAPH_SCHEMA: &str = "foreign-derivation-graph-v1";
const PACKAGE_INDEX_SCHEMA: &str = "foreign-package-index-v1";
const IMPORT_RECEIPT_SCHEMA: &str = "foreign-derivation-import-receipt-v1";
const MANTLE_ADAPTER_SCHEMA: &str = "mantle-foreign-derivation-adapter-plan-v1";
const DIGEST_ALGORITHM: &str = "blake3";
const SHA256_ALGORITHM: &str = "sha256";
const NIX_STORE_PATH_SHA256_ALGORITHM: &str = "nix-store-path-sha256";
pub(crate) const RECOMPUTE_BLAKE3_MODE: &str = "recompute-blake3-v1";
pub(crate) const PRESERVE_CACHE_PATHS_MODE: &str = "preserve-cache-paths-v1";
const SOURCE_REF_FIELD: &str = "source-ref";
const OUT_OUTPUT_NAME: &str = "out";
const HELLO_PACKAGE_NAME: &str = "hello";
const HELLO_SYSTEM: &str = "x86_64-linux";
const DEFAULT_TARGET_PREFIX: &str = "/mantle/store";
const GUIX_SOURCE_PREFIX: &str = "/gnu/store";
pub(crate) const NIX_SOURCE_PREFIX: &str = "/nix/store";
pub(crate) const FIXED_OUTPUT_SEED_KIND: &str = "nix-fixed-output-seed";
const NIX_STORE_PREFIX_WITH_SLASH: &str = "/nix/store/";
const NIX_DERIVATION_SUFFIX: &str = ".drv";
const KIBIBYTE_BYTES: usize = 1_024;
const MEBIBYTE_BYTES: usize = KIBIBYTE_BYTES * KIBIBYTE_BYTES;
const MAX_ATERM_DERIVATION_MEBIBYTES: usize = 16;
const MAX_ATERM_BUNDLE_MEBIBYTES: usize = 256;
pub(crate) const MAX_ATERM_DERIVATION_BYTES: usize = MAX_ATERM_DERIVATION_MEBIBYTES * MEBIBYTE_BYTES;
pub(crate) const MAX_ATERM_BUNDLE_BYTES: usize = MAX_ATERM_BUNDLE_MEBIBYTES * MEBIBYTE_BYTES;
pub(crate) const MAX_ATERM_BUNDLE_DERIVATIONS: usize = MAX_GRAPH_NODES;
const MAX_ATERM_COLLECTION_ITEMS: usize = 256;
const MAX_ATERM_STORE_REFERENCES_PER_FIELD: usize = 256;
const UNKNOWN_FOREIGN_PREFIX: &str = "/foreign/store";
const GUIX_HELLO_NODE_ID: &str = "guix:hello";
const NIX_HELLO_NODE_ID: &str = "nix:hello";
const NIXPKGS_PRODUCER_KIND: &str = "nixpkgs";
const NIX_DERIVATION_BUILTIN: &str = "nix.derivation";
const FIXED_OUTPUT_FETCH_BUILTIN: &str = "fixed-output-fetch";
const OUTPUT_HASH_MODE_ENV: &str = "outputHashMode";
const RECURSIVE_OUTPUT_HASH_MODE: &str = "recursive";
const FLAT_OUTPUT_HASH_MODE: &str = "flat";
const CHMOD_SETUID_CAPABILITY: &str = "chmod-setuid";
const TRUSTED_CACHE_SCOPE: &str = "trusted-binary-cache";
const CACHE_NIXOS_ORG_URL: &str = "https://cache.nixos.org";
const NIX_COMPATIBLE_HASH_DOMAIN: &str = "nix-compatible";
const MANTLE_RECEIPT_HASH_DOMAIN: &str = "mantle-receipt";
const DERIVATION_STORE_PATH_HASH_KIND: &str = "derivation-store-path";
const RAW_GRAPH_HASH_KIND: &str = "raw-graph";
const TRANSLATION_POLICY_HASH_KIND: &str = "translation-policy";
const TRANSLATED_GRAPH_HASH_KIND: &str = "translated-graph";
const SUBSTITUTION_POLICY_CLASSIFICATION: &str = "cache-hint-policy-data-store-admission-required";
const OUTPUT_HASH_HEX_CHARS: usize = 32;
const SOURCE_PAYLOAD_HASH_HEX_CHARS: usize = 32;
const SHA256_HEX_CHARS: usize = 64;
const NIX_STORE_BASENAME_HASH_CHARS: usize = 32;
const NIX_BASE32_ALPHABET: &[u8] = b"0123456789abcdfghijklmnpqrsvwxyz";
const MAX_GRAPH_NODES: usize = 2048;
const MAX_GRAPH_EDGES: usize = 256;
const MAX_PACKAGE_INDEX_ENTRIES: usize = 128;
const MAX_FIELD_BYTES: usize = 32768;
const MAX_SOURCE_PAYLOADS: usize = 512;
const MAX_MIRROR_CANDIDATES: usize = 16;
const MAX_SANDBOX_CAPABILITIES: usize = 16;
const MAX_CACHE_HINTS: usize = 16;
const MAX_UNSUPPORTED_FEATURES: usize = 32;
const MAX_HASH_DOMAIN_RECORDS: usize = 4096;
const EMPTY_OUTPUT_COUNT: usize = 0;
const EMPTY_COMMAND_INVOCATION_COUNT: usize = 0;
const REQUIRED_HELLO_ROOT_COUNT: usize = 1;
const RECEIPT_HASH_DOMAIN_ADDITIONS: usize = 3;

fn empty_hash_domain_records() -> Vec<HashDomainRecord> {
    Vec::new()
}

fn empty_substitution_audit_events() -> Vec<SubstitutionAuditEvent> {
    Vec::new()
}

fn empty_strings() -> Vec<String> {
    Vec::new()
}

fn empty_string_map() -> BTreeMap<String, String> {
    BTreeMap::new()
}

fn empty_nix_input_map() -> BTreeMap<String, NixDerivationJsonInput> {
    BTreeMap::new()
}

fn absent_string() -> Option<String> {
    None
}

fn empty_versioned_inputs() -> NixDerivationJsonVersionedInputs {
    NixDerivationJsonVersionedInputs {
        drvs: BTreeMap::new(),
        srcs: Vec::new(),
    }
}

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
    #[serde(default = "empty_hash_domain_records", skip_serializing_if = "Vec::is_empty")]
    pub(crate) hash_domains: Vec<HashDomainRecord>,
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) expected_content_blake3: Option<String>,
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

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct HashDomainRecord {
    pub(crate) domain: String,
    pub(crate) kind: String,
    pub(crate) algorithm: String,
    pub(crate) value: String,
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
    #[serde(default = "empty_hash_domain_records", skip_serializing_if = "Vec::is_empty")]
    pub(crate) hash_domains: Vec<HashDomainRecord>,
    pub(crate) diagnostics: Vec<ImportDiagnostic>,
    pub(crate) non_claims: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct MantleForeignPlan {
    pub(crate) schema: String,
    pub(crate) roots: Vec<MantleForeignRoot>,
    pub(crate) source_payloads: Vec<SourcePayload>,
    pub(crate) sandbox_audit: Vec<SandboxAuditEvent>,
    #[serde(default = "empty_substitution_audit_events", skip_serializing_if = "Vec::is_empty")]
    pub(crate) substitution_audit: Vec<SubstitutionAuditEvent>,
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

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct SubstitutionAuditEvent {
    pub(crate) node_id: String,
    pub(crate) cache_url: String,
    pub(crate) trust_scope: String,
    pub(crate) classification: String,
    pub(crate) store_admission_required: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct NixDerivationJsonNode {
    pub(crate) name: String,
    pub(crate) system: String,
    pub(crate) builder: String,
    #[serde(default = "empty_strings")]
    pub(crate) args: Vec<String>,
    #[serde(default = "empty_string_map")]
    pub(crate) env: BTreeMap<String, String>,
    pub(crate) outputs: BTreeMap<String, NixDerivationJsonOutput>,
    #[serde(rename = "inputDrvs", default = "empty_nix_input_map")]
    pub(crate) input_drvs: BTreeMap<String, NixDerivationJsonInput>,
    #[serde(rename = "inputSrcs", default = "empty_strings")]
    pub(crate) input_srcs: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct NixDerivationJsonOutput {
    #[serde(default = "absent_string")]
    pub(crate) path: Option<String>,
    #[serde(default = "absent_string")]
    pub(crate) hash: Option<String>,
    #[serde(rename = "hashAlgo", default = "absent_string")]
    pub(crate) hash_algo: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct NixDerivationJsonInput {
    #[serde(default = "empty_strings")]
    pub(crate) outputs: Vec<String>,
}

pub(crate) type NixDerivationJsonClosure = BTreeMap<String, NixDerivationJsonNode>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct AtermDerivationInput {
    pub(crate) logical_path: String,
    pub(crate) bytes: Vec<u8>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub(crate) enum NixDerivationJsonExport {
    Versioned(NixDerivationJsonVersionedExport),
    Legacy(NixDerivationJsonClosure),
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
pub(crate) struct NixDerivationJsonVersionedExport {
    pub(crate) derivations: BTreeMap<String, NixDerivationJsonVersionedNode>,
    pub(crate) version: u32,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
pub(crate) struct NixDerivationJsonVersionedNode {
    pub(crate) name: String,
    pub(crate) system: String,
    pub(crate) builder: String,
    #[serde(default = "empty_strings")]
    pub(crate) args: Vec<String>,
    #[serde(default = "empty_string_map")]
    pub(crate) env: BTreeMap<String, String>,
    pub(crate) outputs: BTreeMap<String, NixDerivationJsonOutput>,
    #[serde(default = "empty_versioned_inputs")]
    pub(crate) inputs: NixDerivationJsonVersionedInputs,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Eq)]
pub(crate) struct NixDerivationJsonVersionedInputs {
    #[serde(default = "empty_nix_input_map")]
    pub(crate) drvs: BTreeMap<String, NixDerivationJsonInput>,
    #[serde(default = "empty_strings")]
    pub(crate) srcs: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct NixProducerConfig {
    pub(crate) package_name: String,
    pub(crate) system: String,
    pub(crate) root_derivation: String,
    pub(crate) producer_identity: String,
    pub(crate) producer_revision: String,
    pub(crate) cache_hints: Vec<CacheHint>,
    pub(crate) unsupported_metadata_classes: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct AtermProducerConfig {
    pub(crate) source_prefix: String,
    pub(crate) producer_kind: String,
    pub(crate) package_name: String,
    pub(crate) system: String,
    pub(crate) root_derivation: String,
    pub(crate) producer_identity: String,
    pub(crate) producer_revision: String,
    pub(crate) cache_hints: Vec<CacheHint>,
    pub(crate) unsupported_metadata_classes: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct NixProducerArtifacts {
    pub(crate) graph: ForeignDerivationGraph,
    pub(crate) package_index: PackageIndex,
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
    debug_assert_eq!(translated_graph.nodes.len(), graph.nodes.len());
    debug_assert_eq!(receipt.producer_identity, graph.producer.identity);
    Ok((translated_graph, receipt))
}

pub(crate) fn plan_mantle_foreign_import(
    translated_graph: &TranslatedGraph,
    package_index: &PackageIndex,
    package_name: &str,
    system: impl AsRef<str>,
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
    let substitution_audit = translated_graph
        .nodes
        .iter()
        .flat_map(|node| {
            node.cache_hints.iter().map(|hint| SubstitutionAuditEvent {
                node_id: node.node_id.clone(),
                cache_url: hint.cache_url.clone(),
                trust_scope: hint.trust_scope.clone(),
                classification: SUBSTITUTION_POLICY_CLASSIFICATION.to_string(),
                store_admission_required: true,
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
        substitution_audit,
        forbidden_process_invocations: Vec::new(),
        non_claims: foreign_import_non_claims(),
    })
}

pub(crate) fn lookup_package<'a>(
    index: &'a PackageIndex,
    package_name: &str,
    system: impl AsRef<str>,
) -> Result<&'a PackageIndexEntry, ImportDiagnostic> {
    validate_package_index_shape(index)?;
    let system = system.as_ref();
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
    if expected.hash_domains != receipt.hash_domains {
        return Err(diagnostic(
            "stale-hash-domain-summary",
            None,
            "receipt hash-domain summary does not match current graph and policy",
        ));
    }
    debug_assert_eq!(expected.raw_graph_digest, receipt.raw_graph_digest);
    debug_assert_eq!(expected.translated_graph_digest, receipt.translated_graph_digest);
    Ok(())
}

pub(crate) fn normalize_nix_derivation_json_export(
    export: NixDerivationJsonExport,
) -> Result<NixDerivationJsonClosure, ImportDiagnostic> {
    match export {
        NixDerivationJsonExport::Legacy(closure) => Ok(closure),
        NixDerivationJsonExport::Versioned(export) => normalize_versioned_nix_derivations(export),
    }
}

pub(crate) fn parse_prefix_aware_aterm_bundle(
    source_prefix: &str,
    inputs: &[AtermDerivationInput],
) -> Result<NixDerivationJsonClosure, ImportDiagnostic> {
    validate_foreign_store_prefix(source_prefix)?;
    if inputs.is_empty() || inputs.len() > MAX_ATERM_BUNDLE_DERIVATIONS {
        return Err(diagnostic(
            "foreign-aterm-derivation-count-out-of-range",
            None,
            "foreign ATerm derivation bundle size is outside supported limits",
        ));
    }

    let total_bytes = inputs.iter().try_fold(0usize, |total, input| {
        total.checked_add(input.bytes.len()).ok_or_else(|| {
            diagnostic("foreign-aterm-bundle-bytes-out-of-range", None, "foreign ATerm bundle byte count overflowed")
        })
    })?;
    if total_bytes > MAX_ATERM_BUNDLE_BYTES {
        return Err(diagnostic(
            "foreign-aterm-bundle-bytes-out-of-range",
            None,
            "foreign ATerm bundle exceeds the total byte limit",
        ));
    }
    let input_count = inputs.len();
    let mut closure = BTreeMap::new();
    for input in inputs {
        validate_foreign_derivation_path(&input.logical_path, source_prefix)?;
        if closure.contains_key(&input.logical_path) {
            return Err(diagnostic(
                "duplicate-foreign-derivation-path",
                None,
                &format!("foreign ATerm bundle repeats logical path: {}", input.logical_path),
            ));
        }
        let node = parse_prefix_aware_aterm_derivation(input, source_prefix)?;
        let replaced = closure.insert(input.logical_path.clone(), node);
        debug_assert!(replaced.is_none(), "duplicate logical paths are rejected before insertion");
        debug_assert!(closure.len() <= input_count, "parsed closure cannot exceed declared inputs");
    }
    validate_foreign_aterm_closure_inputs(&closure)?;
    debug_assert_eq!(closure.len(), input_count);
    debug_assert!(total_bytes <= MAX_ATERM_BUNDLE_BYTES);
    debug_assert!(!closure.is_empty());
    Ok(closure)
}

fn parse_prefix_aware_aterm_derivation(
    input: &AtermDerivationInput,
    source_prefix: &str,
) -> Result<NixDerivationJsonNode, ImportDiagnostic> {
    if input.bytes.is_empty() || input.bytes.len() > MAX_ATERM_DERIVATION_BYTES {
        return Err(diagnostic(
            "foreign-aterm-bytes-out-of-range",
            None,
            &format!("foreign ATerm bytes are outside supported limits: {}", input.logical_path),
        ));
    }
    let text = std::str::from_utf8(&input.bytes).map_err(|_| {
        diagnostic("non-utf8-foreign-aterm", None, &format!("foreign ATerm input is not UTF-8: {}", input.logical_path))
    })?;
    reject_mixed_foreign_store_prefix(text, source_prefix, &input.logical_path)?;
    let parser_text = restore_store_prefix(text, source_prefix, NIX_SOURCE_PREFIX);
    let derivation = nix_compat::derivation::Derivation::from_aterm_bytes(parser_text.as_bytes()).map_err(|_| {
        diagnostic(
            "malformed-foreign-aterm",
            None,
            &format!("foreign ATerm input is malformed: {}", input.logical_path),
        )
    })?;
    validate_foreign_aterm_limits(&derivation, &input.logical_path)?;
    let node = normalize_prefix_aware_aterm_node(&input.logical_path, derivation, source_prefix)?;
    debug_assert!(input.logical_path.starts_with(source_prefix));
    debug_assert!(node.input_drvs.len() <= MAX_ATERM_COLLECTION_ITEMS);
    Ok(node)
}

fn validate_foreign_aterm_limits(
    derivation: &nix_compat::derivation::Derivation,
    logical_path: &str,
) -> Result<(), ImportDiagnostic> {
    let collections = [
        derivation.outputs.len(),
        derivation.input_derivations.len(),
        derivation.input_sources.len(),
        derivation.arguments.len(),
        derivation.environment.len(),
    ];
    if collections.iter().any(|count| *count > MAX_ATERM_COLLECTION_ITEMS) {
        return Err(diagnostic(
            "foreign-aterm-collection-limit-exceeded",
            None,
            &format!("foreign ATerm collection exceeds limit: {logical_path}"),
        ));
    }
    let input_edge_count = derivation.input_derivations.values().try_fold(0usize, |count, outputs| {
        count.checked_add(outputs.len()).ok_or_else(|| {
            diagnostic(
                "foreign-aterm-collection-limit-exceeded",
                None,
                &format!("foreign ATerm input edge count overflowed: {logical_path}"),
            )
        })
    })?;
    if input_edge_count > MAX_GRAPH_EDGES {
        return Err(diagnostic(
            "foreign-aterm-collection-limit-exceeded",
            None,
            &format!("foreign ATerm input edge count exceeds limit: {logical_path}"),
        ));
    }
    for outputs in derivation.input_derivations.values() {
        for output_name in outputs {
            require_field_limit(output_name, None, "foreign-aterm-input-output-name")?;
        }
    }
    require_field_limit(&derivation.builder, None, "foreign-aterm-builder")?;
    require_field_limit(&derivation.system, None, "foreign-aterm-system")?;
    for argument in &derivation.arguments {
        require_field_limit(argument, None, "foreign-aterm-argument")?;
    }
    for (key, value) in &derivation.environment {
        require_field_limit(key, None, "foreign-aterm-environment-key")?;
        if value.len() > MAX_FIELD_BYTES {
            return Err(diagnostic(
                "field-limit-exceeded",
                None,
                &format!("field foreign-aterm-environment-value exceeds byte limit: {logical_path}"),
            ));
        }
    }
    debug_assert!(collections.iter().all(|count| *count <= MAX_ATERM_COLLECTION_ITEMS));
    debug_assert!(input_edge_count <= MAX_GRAPH_EDGES);
    debug_assert!(derivation.builder.len() <= MAX_FIELD_BYTES);
    Ok(())
}

fn normalize_prefix_aware_aterm_node(
    drv_path: &str,
    derivation: nix_compat::derivation::Derivation,
    source_prefix: &str,
) -> Result<NixDerivationJsonNode, ImportDiagnostic> {
    let raw_env = foreign_aterm_environment_to_strings(&derivation)?;
    let env = raw_env
        .into_iter()
        .map(|(key, value)| {
            restore_and_validate_foreign_text(&value, source_prefix, "environment").map(|value| (key, value))
        })
        .collect::<Result<BTreeMap<_, _>, _>>()?;
    let name = env.get("name").cloned().unwrap_or_else(|| nix_derivation_name_from_path(drv_path));
    let outputs = normalize_foreign_aterm_outputs(derivation.outputs, &env, source_prefix)?;
    let input_drvs = derivation
        .input_derivations
        .into_iter()
        .map(|(path, outputs)| {
            (path.to_absolute_path_with_prefix(source_prefix), NixDerivationJsonInput {
                outputs: outputs.into_iter().collect(),
            })
        })
        .collect();
    let input_srcs = derivation
        .input_sources
        .into_iter()
        .map(|path| path.to_absolute_path_with_prefix(source_prefix))
        .collect();
    let system = restore_and_validate_foreign_text(&derivation.system, source_prefix, "system")?;
    let builder = restore_and_validate_foreign_text(&derivation.builder, source_prefix, "builder")?;
    let args = derivation
        .arguments
        .into_iter()
        .map(|argument| restore_and_validate_foreign_text(&argument, source_prefix, "argument"))
        .collect::<Result<Vec<_>, _>>()?;
    let node = NixDerivationJsonNode {
        name,
        system,
        builder,
        args,
        env,
        outputs,
        input_drvs,
        input_srcs,
    };
    debug_assert!(!node.outputs.is_empty());
    debug_assert!(node.input_drvs.len() <= MAX_ATERM_COLLECTION_ITEMS);
    Ok(node)
}

fn normalize_foreign_aterm_outputs(
    outputs: BTreeMap<String, nix_compat::derivation::Output>,
    env: &BTreeMap<String, String>,
    source_prefix: &str,
) -> Result<BTreeMap<String, NixDerivationJsonOutput>, ImportDiagnostic> {
    let output_count = outputs.len();
    let mut normalized = BTreeMap::new();
    for (name, output) in outputs {
        require_field_limit(&name, None, "foreign-aterm-output-name")?;
        let (hash, hash_algo) = nix_aterm_output_hash_fields(output.ca_hash.as_ref());
        let path = output
            .path
            .map(|path| path.to_absolute_path_with_prefix(source_prefix))
            .or_else(|| env.get(&name).cloned())
            .ok_or_else(|| diagnostic("missing-foreign-output-path", None, "foreign ATerm output is missing a path"))?;
        validate_foreign_store_path(&path, source_prefix)?;
        let replaced = normalized.insert(name, NixDerivationJsonOutput {
            path: Some(path),
            hash,
            hash_algo,
        });
        debug_assert!(replaced.is_none(), "ATerm parser rejects duplicate output names");
        debug_assert!(normalized.len() <= output_count);
    }
    if normalized.is_empty() {
        return Err(diagnostic("missing-output-declaration", None, "foreign ATerm derivation has no outputs"));
    }
    debug_assert_eq!(normalized.len(), output_count);
    debug_assert!(normalized.len() <= MAX_ATERM_COLLECTION_ITEMS);
    Ok(normalized)
}

fn foreign_aterm_environment_to_strings(
    derivation: &nix_compat::derivation::Derivation,
) -> Result<BTreeMap<String, String>, ImportDiagnostic> {
    let mut env = BTreeMap::new();
    for (key, value) in &derivation.environment {
        let value = std::str::from_utf8(value.as_ref()).map_err(|_| {
            diagnostic("non-utf8-foreign-aterm-field", None, "foreign ATerm environment contains non-UTF-8 bytes")
        })?;
        let replaced = env.insert(key.clone(), value.to_string());
        debug_assert!(replaced.is_none(), "ATerm parser rejects duplicate environment keys");
        debug_assert!(env.len() <= derivation.environment.len());
    }
    Ok(env)
}

fn restore_and_validate_foreign_text(
    value: &str,
    source_prefix: &str,
    field: &str,
) -> Result<String, ImportDiagnostic> {
    let restored = restore_store_prefix(value, NIX_SOURCE_PREFIX, source_prefix);
    validate_embedded_foreign_store_paths(&restored, source_prefix, field)?;
    debug_assert!(source_prefix == NIX_SOURCE_PREFIX || !restored.contains(NIX_STORE_PREFIX_WITH_SLASH));
    debug_assert!(source_prefix == GUIX_SOURCE_PREFIX || !restored.contains("/gnu/store/"));
    Ok(restored)
}

fn validate_embedded_foreign_store_paths(
    value: &str,
    source_prefix: &str,
    field: &str,
) -> Result<(), ImportDiagnostic> {
    let needle = format!("{source_prefix}/");
    let mut search_offset = 0usize;
    let mut reference_count = 0usize;
    while let Some(relative_start) = value[search_offset..].find(&needle) {
        reference_count = reference_count.checked_add(1).ok_or_else(|| {
            diagnostic(
                "foreign-aterm-store-reference-limit-exceeded",
                None,
                &format!("foreign ATerm store reference count overflowed in {field}"),
            )
        })?;
        if reference_count > MAX_ATERM_STORE_REFERENCES_PER_FIELD {
            return Err(diagnostic(
                "foreign-aterm-store-reference-limit-exceeded",
                None,
                &format!("foreign ATerm store reference count exceeds limit in {field}"),
            ));
        }
        let path_start = search_offset.saturating_add(relative_start);
        let component_start = path_start.saturating_add(needle.len());
        let component_tail = &value[component_start..];
        let component_bytes = component_tail.as_bytes();
        let component_end = component_bytes
            .iter()
            .position(|byte| is_foreign_store_path_delimiter(*byte))
            .unwrap_or(component_bytes.len());
        let path_end = component_start.saturating_add(component_end);
        validate_foreign_store_path(&value[path_start..path_end], source_prefix)?;
        debug_assert!(path_end > path_start, "store prefix must make each scan advance");
        search_offset = path_end;
    }
    debug_assert!(reference_count <= MAX_ATERM_STORE_REFERENCES_PER_FIELD);
    debug_assert!(search_offset <= value.len());
    Ok(())
}

fn is_foreign_store_path_delimiter(byte: u8) -> bool {
    byte == b'/'
        || byte == b':'
        || byte == b'"'
        || byte == b'\''
        || byte == b')'
        || byte == b']'
        || byte == b'}'
        || byte == b','
        || byte == b';'
        || byte.is_ascii_whitespace()
}

fn validate_foreign_aterm_closure_inputs(closure: &NixDerivationJsonClosure) -> Result<(), ImportDiagnostic> {
    for (drv_path, node) in closure {
        for input_drv in node.input_drvs.keys() {
            if !closure.contains_key(input_drv) {
                return Err(diagnostic(
                    "missing-foreign-input-derivation",
                    None,
                    &format!("foreign ATerm input is absent from bundle: {input_drv}; referenced by {drv_path}"),
                ));
            }
        }
    }
    debug_assert!(closure.values().all(|node| node.input_drvs.keys().all(|path| closure.contains_key(path))));
    debug_assert!(!closure.is_empty());
    Ok(())
}

fn validate_foreign_store_prefix(source_prefix: &str) -> Result<(), ImportDiagnostic> {
    if source_prefix != NIX_SOURCE_PREFIX && source_prefix != GUIX_SOURCE_PREFIX {
        return Err(diagnostic(
            "unsupported-foreign-store-prefix",
            None,
            "foreign ATerm store prefix must be /nix/store or /gnu/store",
        ));
    }
    Ok(())
}

fn validate_foreign_derivation_path(path: &str, source_prefix: &str) -> Result<(), ImportDiagnostic> {
    validate_foreign_store_path(path, source_prefix)?;
    if !path.ends_with(NIX_DERIVATION_SUFFIX) {
        return Err(diagnostic(
            "invalid-foreign-derivation-path",
            None,
            &format!("foreign derivation path must end in .drv: {path}"),
        ));
    }
    Ok(())
}

fn validate_foreign_store_path(path: &str, source_prefix: &str) -> Result<(), ImportDiagnostic> {
    require_field_limit(path, None, "foreign-store-path")?;
    nix_compat::store_path::StorePath::<String>::from_absolute_path_with_prefix(path.as_bytes(), source_prefix)
        .map_err(|_| {
            diagnostic(
                "invalid-foreign-store-path",
                None,
                &format!("foreign store path is malformed or outside {source_prefix}: {path}"),
            )
        })?;
    Ok(())
}

fn reject_mixed_foreign_store_prefix(
    text: &str,
    source_prefix: &str,
    logical_path: &str,
) -> Result<(), ImportDiagnostic> {
    let other_prefix = if source_prefix == NIX_SOURCE_PREFIX {
        GUIX_SOURCE_PREFIX
    } else {
        NIX_SOURCE_PREFIX
    };
    let other_prefix_with_slash = format!("{other_prefix}/");
    if text.contains(&other_prefix_with_slash) {
        return Err(diagnostic(
            "mixed-foreign-store-prefix",
            None,
            &format!("foreign ATerm input contains {other_prefix} outside declared {source_prefix}: {logical_path}"),
        ));
    }
    Ok(())
}

fn restore_store_prefix(value: &str, from_prefix: &str, to_prefix: &str) -> String {
    if from_prefix == to_prefix {
        return value.to_string();
    }
    let from_prefix_with_slash = format!("{from_prefix}/");
    let to_prefix_with_slash = format!("{to_prefix}/");
    value.replace(&from_prefix_with_slash, &to_prefix_with_slash)
}

pub(crate) fn normalize_nix_aterm_derivation_closure(
    derivations: BTreeMap<String, nix_compat::derivation::Derivation>,
) -> Result<NixDerivationJsonClosure, ImportDiagnostic> {
    if derivations.is_empty() || derivations.len() > MAX_GRAPH_NODES {
        return Err(diagnostic(
            "nix-derivation-count-out-of-range",
            None,
            "Nix derivation closure size is outside supported limits",
        ));
    }
    let derivation_count = derivations.len();
    let mut closure = BTreeMap::new();
    for (drv_key, derivation) in derivations {
        let drv_path = normalize_nix_store_key(&drv_key)?;
        let normalized_node = normalize_nix_aterm_derivation_node(&drv_path, derivation)?;
        debug_assert!(closure.len() < derivation_count);
        if closure.insert(drv_path, normalized_node).is_some() {
            return Err(diagnostic(
                "duplicate-nix-derivation-path",
                None,
                "Nix ATerm derivation closure contains duplicate derivation paths",
            ));
        }
    }
    debug_assert_eq!(closure.len(), derivation_count);
    debug_assert!(closure.len() <= MAX_GRAPH_NODES);
    Ok(closure)
}

pub(crate) fn select_nix_derivation_json_closure(
    closure: &NixDerivationJsonClosure,
    root_derivation: &str,
) -> Result<NixDerivationJsonClosure, ImportDiagnostic> {
    validate_nix_derivation_path(root_derivation)?;
    if closure.is_empty() || closure.len() > MAX_GRAPH_NODES {
        return Err(diagnostic(
            "nix-derivation-count-out-of-range",
            None,
            "Nix derivation closure size is outside supported limits",
        ));
    }
    if !closure.contains_key(root_derivation) {
        return Err(diagnostic(
            "missing-nix-root-derivation",
            None,
            "selected root derivation is absent from the concrete Nix closure",
        ));
    }
    select_reachable_nix_derivations(closure, root_derivation)
}

pub(crate) fn lower_nix_derivation_json_closure(
    closure: &NixDerivationJsonClosure,
    config: &NixProducerConfig,
) -> Result<NixProducerArtifacts, ImportDiagnostic> {
    lower_prefix_aware_aterm_closure(closure, &AtermProducerConfig {
        source_prefix: NIX_SOURCE_PREFIX.to_string(),
        producer_kind: NIXPKGS_PRODUCER_KIND.to_string(),
        package_name: config.package_name.clone(),
        system: config.system.clone(),
        root_derivation: config.root_derivation.clone(),
        producer_identity: config.producer_identity.clone(),
        producer_revision: config.producer_revision.clone(),
        cache_hints: config.cache_hints.clone(),
        unsupported_metadata_classes: config.unsupported_metadata_classes.clone(),
    })
}

pub(crate) fn lower_prefix_aware_aterm_closure(
    closure: &NixDerivationJsonClosure,
    config: &AtermProducerConfig,
) -> Result<NixProducerArtifacts, ImportDiagnostic> {
    validate_aterm_producer_inputs(closure, config)?;
    let path_to_node_id = foreign_node_id_map(closure, &config.source_prefix)?;
    let root_node_id = path_to_node_id.get(&config.root_derivation).cloned().ok_or_else(|| {
        diagnostic(
            "missing-foreign-root-derivation",
            None,
            "selected root derivation is absent from the concrete foreign closure",
        )
    })?;
    let (nodes, source_payloads) = lower_aterm_nodes(closure, config, &path_to_node_id)?;
    let artifacts = build_aterm_producer_artifacts(closure, config, root_node_id, nodes, source_payloads);
    debug_assert_eq!(artifacts.graph.nodes.len(), closure.len());
    debug_assert_eq!(artifacts.package_index.entries.len(), REQUIRED_HELLO_ROOT_COUNT);
    Ok(artifacts)
}

fn lower_aterm_nodes(
    closure: &NixDerivationJsonClosure,
    config: &AtermProducerConfig,
    path_to_node_id: &BTreeMap<String, String>,
) -> Result<(Vec<ForeignDerivationNode>, BTreeMap<String, SourcePayload>), ImportDiagnostic> {
    let mut source_payloads = BTreeMap::new();
    let mut nodes = Vec::with_capacity(closure.len());
    let lowering_context = AtermLoweringContext {
        path_to_node_id,
        closure,
        config,
    };
    for (drv_path, derivation) in closure {
        nodes.push(lower_aterm_derivation_node(
            AtermNodeLoweringRequest {
                drv_path,
                derivation,
                is_root: drv_path == &config.root_derivation,
            },
            &lowering_context,
            &mut source_payloads,
        )?);
    }
    debug_assert_eq!(nodes.len(), closure.len());
    debug_assert!(source_payloads.len() <= MAX_SOURCE_PAYLOADS);
    Ok((nodes, source_payloads))
}

fn build_aterm_producer_artifacts(
    closure: &NixDerivationJsonClosure,
    config: &AtermProducerConfig,
    root_node_id: String,
    nodes: Vec<ForeignDerivationNode>,
    source_payloads: BTreeMap<String, SourcePayload>,
) -> NixProducerArtifacts {
    let hash_domains = if config.source_prefix == NIX_SOURCE_PREFIX {
        nix_hash_domain_records(closure)
    } else {
        Vec::new()
    };
    let graph = ForeignDerivationGraph {
        schema: GRAPH_SCHEMA.to_string(),
        producer: ProducerSummary {
            kind: config.producer_kind.clone(),
            identity: config.producer_identity.clone(),
            revision: config.producer_revision.clone(),
        },
        source_store_prefixes: vec![config.source_prefix.clone()],
        target_store_prefix: None,
        root_derivation_ids: vec![root_node_id.clone()],
        nodes,
        source_payloads: source_payloads.into_values().collect(),
        unsupported_features: Vec::new(),
        frontend_metadata: Vec::new(),
        hash_domains,
    };
    let package_index = PackageIndex {
        schema: PACKAGE_INDEX_SCHEMA.to_string(),
        entries: vec![PackageIndexEntry {
            name: config.package_name.clone(),
            system: config.system.clone(),
            root_derivation_id: root_node_id,
            aliases: Vec::new(),
            provenance_ref: config.producer_identity.clone(),
            metadata_digest: aterm_package_metadata_digest(config),
            unsupported_metadata_classes: sorted_strings(config.unsupported_metadata_classes.clone()),
        }],
    };
    debug_assert_eq!(graph.nodes.len(), closure.len());
    debug_assert_eq!(package_index.entries.len(), REQUIRED_HELLO_ROOT_COUNT);
    NixProducerArtifacts { graph, package_index }
}

pub(crate) fn guix_like_hello_fixture() -> (ForeignDerivationGraph, PackageIndex) {
    hello_fixture(HelloFixtureSpec {
        producer_kind: "guix",
        source_prefix: GUIX_SOURCE_PREFIX,
        node_id: GUIX_HELLO_NODE_ID,
        producer_identity: "guix-time-machine:hello",
    })
}

pub(crate) fn nix_like_hello_fixture() -> (ForeignDerivationGraph, PackageIndex) {
    hello_fixture(HelloFixtureSpec {
        producer_kind: "nix",
        source_prefix: NIX_SOURCE_PREFIX,
        node_id: NIX_HELLO_NODE_ID,
        producer_identity: "nix-derivation-json:hello",
    })
}

struct HelloFixtureSpec<'a> {
    producer_kind: &'a str,
    source_prefix: &'a str,
    node_id: &'a str,
    producer_identity: &'a str,
}

fn hello_fixture(spec: HelloFixtureSpec<'_>) -> (ForeignDerivationGraph, PackageIndex) {
    let graph = hello_fixture_graph(&spec);
    let index = hello_fixture_index(&spec);
    debug_assert_eq!(graph.root_derivation_ids.len(), REQUIRED_HELLO_ROOT_COUNT);
    debug_assert_eq!(index.entries.len(), REQUIRED_HELLO_ROOT_COUNT);
    (graph, index)
}

fn hello_fixture_graph(spec: &HelloFixtureSpec<'_>) -> ForeignDerivationGraph {
    let payload_id = format!("{}-hello-source", spec.producer_kind);
    let store_source = format!("{}/00000000000000000000000000000000-hello-source", spec.source_prefix);
    let output_path = format!("{}/11111111111111111111111111111111-hello", spec.source_prefix);
    let fixed_output_digest = "0".repeat(SHA256_HEX_CHARS);
    let outputs = BTreeMap::from([(OUT_OUTPUT_NAME.to_string(), OutputDeclaration {
        path: output_path.clone(),
        hash: Some(fixed_output_digest.clone()),
    })]);
    let env = BTreeMap::from([
        ("src".to_string(), store_source.clone()),
        ("out".to_string(), output_path.clone()),
    ]);
    let graph = ForeignDerivationGraph {
        schema: GRAPH_SCHEMA.to_string(),
        producer: ProducerSummary {
            kind: spec.producer_kind.to_string(),
            identity: spec.producer_identity.to_string(),
            revision: "fixture-revision".to_string(),
        },
        source_store_prefixes: vec![spec.source_prefix.to_string()],
        target_store_prefix: None,
        root_derivation_ids: vec![spec.node_id.to_string()],
        nodes: vec![ForeignDerivationNode {
            node_id: spec.node_id.to_string(),
            original_derivation: format!("{}/22222222222222222222222222222222-hello.drv", spec.source_prefix),
            name: HELLO_PACKAGE_NAME.to_string(),
            system: HELLO_SYSTEM.to_string(),
            builder: format!("{}/33333333333333333333333333333333-bash/bin/bash", spec.source_prefix),
            args: vec!["-c".to_string(), format!("cp {store_source} $out")],
            env,
            outputs,
            input_derivations: Vec::new(),
            source_refs: vec![SourceRef {
                payload_id: payload_id.clone(),
                field: SOURCE_REF_FIELD.to_string(),
            }],
            fixed_output: Some(FixedOutputMetadata {
                algorithm: SHA256_ALGORITHM.to_string(),
                digest: fixed_output_digest,
                recursive: true,
            }),
            builtin: FIXED_OUTPUT_FETCH_BUILTIN.to_string(),
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
            expected_content_blake3: None,
        }],
        unsupported_features: Vec::new(),
        frontend_metadata: Vec::new(),
        hash_domains: Vec::new(),
    };
    debug_assert_eq!(graph.nodes.len(), REQUIRED_HELLO_ROOT_COUNT);
    debug_assert_eq!(graph.source_payloads.len(), REQUIRED_HELLO_ROOT_COUNT);
    graph
}

fn hello_fixture_index(spec: &HelloFixtureSpec<'_>) -> PackageIndex {
    PackageIndex {
        schema: PACKAGE_INDEX_SCHEMA.to_string(),
        entries: vec![PackageIndexEntry {
            name: HELLO_PACKAGE_NAME.to_string(),
            system: HELLO_SYSTEM.to_string(),
            root_derivation_id: spec.node_id.to_string(),
            aliases: vec![format!("{}-hello", spec.producer_kind)],
            provenance_ref: spec.producer_identity.to_string(),
            metadata_digest: blake3_hex(spec.producer_identity),
            unsupported_metadata_classes: Vec::new(),
        }],
    }
}

fn normalize_versioned_nix_derivations(
    export: NixDerivationJsonVersionedExport,
) -> Result<NixDerivationJsonClosure, ImportDiagnostic> {
    if export.derivations.is_empty() || export.derivations.len() > MAX_GRAPH_NODES {
        return Err(diagnostic(
            "nix-derivation-count-out-of-range",
            None,
            "Nix derivation export size is outside supported limits",
        ));
    }
    let derivation_count = export.derivations.len();
    let mut closure = BTreeMap::new();
    for (drv_key, node) in export.derivations {
        let drv_path = normalize_nix_store_key(&drv_key)?;
        let env = normalize_nix_env(node.env);
        let outputs = normalize_nix_outputs(node.outputs, &env)?;
        let normalized_node = NixDerivationJsonNode {
            name: node.name,
            system: node.system,
            builder: normalize_nix_path_field(&node.builder)?,
            args: node.args,
            env,
            outputs,
            input_drvs: normalize_versioned_input_drvs(node.inputs.drvs)?,
            input_srcs: normalize_nix_store_paths(node.inputs.srcs)?,
        };
        if closure.len() >= derivation_count {
            return Err(diagnostic("nix-derivation-count-out-of-range", None, "versioned Nix closure exceeded input"));
        }
        if closure.insert(drv_path, normalized_node).is_some() {
            return Err(diagnostic(
                "duplicate-nix-derivation-path",
                None,
                "versioned Nix derivation export contains duplicate derivation paths",
            ));
        }
    }
    debug_assert_eq!(closure.len(), derivation_count);
    debug_assert!(closure.len() <= MAX_GRAPH_NODES);
    Ok(closure)
}

fn normalize_versioned_input_drvs(
    input_drvs: BTreeMap<String, NixDerivationJsonInput>,
) -> Result<BTreeMap<String, NixDerivationJsonInput>, ImportDiagnostic> {
    let input_count = input_drvs.len();
    let mut normalized = BTreeMap::new();
    for (input_drv, input) in input_drvs {
        let drv_path = normalize_nix_store_key(&input_drv)?;
        if normalized.len() >= input_count {
            return Err(diagnostic("nix-derivation-count-out-of-range", None, "normalized Nix inputs exceeded input"));
        }
        normalized.insert(drv_path, input);
    }
    Ok(normalized)
}

fn normalize_nix_outputs(
    outputs: BTreeMap<String, NixDerivationJsonOutput>,
    env: &BTreeMap<String, String>,
) -> Result<BTreeMap<String, NixDerivationJsonOutput>, ImportDiagnostic> {
    let output_count = outputs.len();
    let mut normalized = BTreeMap::new();
    for (output_name, mut output) in outputs {
        let path = output
            .path
            .as_deref()
            .or_else(|| env.get(&output_name).map(String::as_str))
            .or_else(|| {
                if output_name == OUT_OUTPUT_NAME {
                    env.get(OUT_OUTPUT_NAME).map(String::as_str)
                } else {
                    None
                }
            })
            .ok_or_else(|| diagnostic("missing-nix-output-path", None, "Nix output is missing a path"))?;
        output.path = Some(normalize_nix_path_field(path)?);
        if normalized.len() >= output_count {
            return Err(diagnostic("nix-output-count-out-of-range", None, "normalized Nix outputs exceeded input"));
        }
        normalized.insert(output_name, output);
    }
    debug_assert_eq!(normalized.len(), output_count);
    debug_assert!(normalized.len() <= MAX_GRAPH_NODES);
    Ok(normalized)
}

fn normalize_nix_store_paths(paths: Vec<String>) -> Result<Vec<String>, ImportDiagnostic> {
    paths.into_iter().map(|path| normalize_nix_path_field(&path)).collect()
}

fn normalize_nix_env(env: BTreeMap<String, String>) -> BTreeMap<String, String> {
    env.into_iter().map(|(key, value)| (key, normalize_nix_embedded_store_paths(&value))).collect()
}

fn select_reachable_nix_derivations(
    closure: &NixDerivationJsonClosure,
    root_derivation: &str,
) -> Result<NixDerivationJsonClosure, ImportDiagnostic> {
    let mut selected = BTreeMap::new();
    let mut seen = BTreeSet::new();
    let mut pending = vec![root_derivation.to_string()];
    while let Some(drv_path) = pending.pop() {
        if !seen.insert(drv_path.clone()) {
            continue;
        }
        let derivation = closure.get(&drv_path).ok_or_else(|| {
            diagnostic(
                "missing-nix-input-derivation",
                None,
                &format!("reachable Nix input derivation is absent from bundle: {drv_path}"),
            )
        })?;
        for input_drv in derivation.input_drvs.keys() {
            if pending.len() >= MAX_GRAPH_NODES {
                return Err(diagnostic(
                    "nix-derivation-count-out-of-range",
                    None,
                    "reachable Nix closure exceeds limit",
                ));
            }
            pending.push(input_drv.clone());
        }
        if selected.len() >= closure.len() {
            return Err(diagnostic("nix-derivation-count-out-of-range", None, "reachable Nix closure exceeds input"));
        }
        selected.insert(drv_path, derivation.clone());
    }
    debug_assert!(selected.contains_key(root_derivation));
    debug_assert!(selected.len() <= closure.len());
    Ok(selected)
}

fn normalize_nix_aterm_derivation_node(
    drv_path: &str,
    derivation: nix_compat::derivation::Derivation,
) -> Result<NixDerivationJsonNode, ImportDiagnostic> {
    let raw_env = nix_aterm_environment_to_strings(&derivation)?;
    let env = normalize_nix_env(raw_env);
    let name = env.get("name").cloned().unwrap_or_else(|| nix_derivation_name_from_path(drv_path));
    let outputs = normalize_nix_outputs(nix_aterm_outputs_to_json(derivation.outputs), &env)?;
    Ok(NixDerivationJsonNode {
        name,
        system: derivation.system,
        builder: normalize_nix_path_field(&derivation.builder)?,
        args: derivation.arguments,
        env,
        outputs,
        input_drvs: nix_aterm_input_derivations_to_json(derivation.input_derivations),
        input_srcs: nix_aterm_input_sources_to_json(derivation.input_sources),
    })
}

fn nix_aterm_environment_to_strings(
    derivation: &nix_compat::derivation::Derivation,
) -> Result<BTreeMap<String, String>, ImportDiagnostic> {
    let environment_count = derivation.environment.len();
    let mut env = BTreeMap::new();
    for (key, value) in &derivation.environment {
        let value = std::str::from_utf8(value.as_ref()).map_err(|_| {
            diagnostic("non-utf8-nix-derivation-env", None, "Nix derivation environment contains non-UTF-8 bytes")
        })?;
        if env.len() >= environment_count {
            return Err(diagnostic(
                "nix-environment-count-out-of-range",
                None,
                "normalized Nix environment exceeded input",
            ));
        }
        env.insert(key.clone(), value.to_string());
    }
    Ok(env)
}

fn nix_aterm_outputs_to_json(
    outputs: BTreeMap<String, nix_compat::derivation::Output>,
) -> BTreeMap<String, NixDerivationJsonOutput> {
    outputs
        .into_iter()
        .map(|(name, output)| {
            let (hash, hash_algo) = nix_aterm_output_hash_fields(output.ca_hash.as_ref());
            let path = output.path.map(|path| path.to_absolute_path());
            (name, NixDerivationJsonOutput { path, hash, hash_algo })
        })
        .collect()
}

fn nix_aterm_output_hash_fields(hash: Option<&nix_compat::nixhash::CAHash>) -> (Option<String>, Option<String>) {
    let Some(hash) = hash else {
        return (None, None);
    };
    match hash {
        nix_compat::nixhash::CAHash::Flat(digest) => {
            (Some(HEXLOWER.encode(digest.digest_as_bytes())), Some(digest.algo().to_string()))
        }
        nix_compat::nixhash::CAHash::Nar(digest) => {
            (Some(HEXLOWER.encode(digest.digest_as_bytes())), Some(format!("r:{}", digest.algo())))
        }
        nix_compat::nixhash::CAHash::Text(digest) => (Some(HEXLOWER.encode(digest.as_ref())), Some("text".to_string())),
    }
}

fn nix_aterm_input_derivations_to_json(
    input_drvs: BTreeMap<nix_compat::store_path::StorePath<String>, BTreeSet<String>>,
) -> BTreeMap<String, NixDerivationJsonInput> {
    input_drvs
        .into_iter()
        .map(|(drv_path, outputs)| {
            (drv_path.to_absolute_path(), NixDerivationJsonInput {
                outputs: outputs.into_iter().collect(),
            })
        })
        .collect()
}

fn nix_aterm_input_sources_to_json(input_srcs: BTreeSet<nix_compat::store_path::StorePath<String>>) -> Vec<String> {
    input_srcs.into_iter().map(|source| source.to_absolute_path()).collect()
}

fn nix_derivation_name_from_path(drv_path: &str) -> String {
    let basename = drv_path.rsplit('/').next().unwrap_or(drv_path);
    let component = basename.strip_suffix(NIX_DERIVATION_SUFFIX).unwrap_or(basename);
    if component.len() > NIX_STORE_BASENAME_HASH_CHARS
        && component.as_bytes().get(NIX_STORE_BASENAME_HASH_CHARS) == Some(&b'-')
        && let Some(name) = component.get(NIX_STORE_BASENAME_HASH_CHARS.saturating_add(1)..)
    {
        return name.to_string();
    }
    component.to_string()
}

fn normalize_nix_embedded_store_paths(value: &str) -> String {
    value
        .split(':')
        .map(|segment| {
            if looks_like_store_basename(segment) {
                format!("{NIX_STORE_PREFIX_WITH_SLASH}{segment}")
            } else {
                segment.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join(":")
}

fn normalize_nix_path_field(path: &str) -> Result<String, ImportDiagnostic> {
    if path.starts_with(NIX_STORE_PREFIX_WITH_SLASH) {
        validate_nix_store_path(path)?;
        return Ok(path.to_string());
    }
    if looks_like_store_basename(path) {
        let normalized = format!("{NIX_STORE_PREFIX_WITH_SLASH}{path}");
        validate_nix_store_path(&normalized)?;
        return Ok(normalized);
    }
    Ok(path.to_string())
}

fn normalize_nix_store_key(key: &str) -> Result<String, ImportDiagnostic> {
    if key.starts_with(NIX_STORE_PREFIX_WITH_SLASH) {
        validate_nix_derivation_path(key)?;
        return Ok(key.to_string());
    }
    let normalized = format!("{NIX_STORE_PREFIX_WITH_SLASH}{key}");
    validate_nix_derivation_path(&normalized)?;
    Ok(normalized)
}

fn looks_like_store_basename(value: &str) -> bool {
    let Some((digest, name)) = value.split_once('-') else {
        return false;
    };
    digest.len() == NIX_STORE_BASENAME_HASH_CHARS
        && digest.bytes().all(|byte| NIX_BASE32_ALPHABET.contains(&byte))
        && !name.is_empty()
        && !name.contains('/')
}

fn validate_aterm_producer_inputs(
    closure: &NixDerivationJsonClosure,
    config: &AtermProducerConfig,
) -> Result<(), ImportDiagnostic> {
    validate_foreign_store_prefix(&config.source_prefix)?;
    if closure.is_empty() || closure.len() > MAX_GRAPH_NODES {
        return Err(diagnostic(
            "foreign-derivation-count-out-of-range",
            None,
            "foreign derivation closure size is outside supported limits",
        ));
    }
    require_field_limit(&config.package_name, None, "foreign-package-name")?;
    require_field_limit(&config.system, None, "foreign-system")?;
    require_field_limit(&config.root_derivation, None, "foreign-root-derivation")?;
    require_field_limit(&config.producer_kind, None, "foreign-producer-kind")?;
    require_field_limit(&config.producer_identity, None, "foreign-producer-identity")?;
    require_field_limit(&config.producer_revision, None, "foreign-producer-revision")?;
    if config.producer_kind.is_empty() {
        return Err(diagnostic("missing-foreign-producer-kind", None, "foreign producer kind must not be empty"));
    }
    validate_foreign_derivation_path(&config.root_derivation, &config.source_prefix)?;
    validate_aterm_cache_hints(config)?;
    for drv_path in closure.keys() {
        validate_foreign_derivation_path(drv_path, &config.source_prefix)?;
    }
    if !closure.contains_key(&config.root_derivation) {
        return Err(diagnostic(
            "missing-foreign-root-derivation",
            None,
            "selected root derivation is absent from the concrete foreign closure",
        ));
    }
    debug_assert!(!closure.is_empty());
    debug_assert!(closure.len() <= MAX_GRAPH_NODES);
    Ok(())
}

fn validate_aterm_cache_hints(config: &AtermProducerConfig) -> Result<(), ImportDiagnostic> {
    if config.cache_hints.len() > MAX_CACHE_HINTS {
        return Err(diagnostic(
            "cache-hint-limit-exceeded",
            None,
            "foreign producer config declares too many cache hints",
        ));
    }
    for hint in &config.cache_hints {
        require_field_limit(&hint.cache_url, None, "foreign-cache-url")?;
        require_field_limit(&hint.trust_scope, None, "foreign-cache-trust-scope")?;
        if hint.trust_scope.is_empty() {
            return Err(diagnostic("missing-cache-trust-scope", None, "cache hint is missing a trust scope"));
        }
    }
    debug_assert!(config.cache_hints.len() <= MAX_CACHE_HINTS);
    debug_assert!(config.cache_hints.iter().all(|hint| !hint.trust_scope.is_empty()));
    Ok(())
}

fn foreign_node_id_map(
    closure: &NixDerivationJsonClosure,
    source_prefix: &str,
) -> Result<BTreeMap<String, String>, ImportDiagnostic> {
    let mut path_to_node_id = BTreeMap::new();
    let mut node_ids = BTreeSet::new();
    for drv_path in closure.keys() {
        let node_id = foreign_node_id(drv_path, source_prefix)?;
        if !node_ids.insert(node_id.clone()) {
            return Err(diagnostic(
                "duplicate-foreign-node-id",
                None,
                "foreign derivation paths produce duplicate node IDs",
            ));
        }
        if path_to_node_id.len() >= closure.len() {
            return Err(diagnostic("foreign-derivation-count-out-of-range", None, "foreign node IDs exceeded closure"));
        }
        path_to_node_id.insert(drv_path.clone(), node_id);
    }
    debug_assert_eq!(path_to_node_id.len(), closure.len());
    debug_assert_eq!(node_ids.len(), closure.len());
    Ok(path_to_node_id)
}

struct AtermLoweringContext<'a> {
    path_to_node_id: &'a BTreeMap<String, String>,
    closure: &'a NixDerivationJsonClosure,
    config: &'a AtermProducerConfig,
}

struct AtermNodeLoweringRequest<'a> {
    drv_path: &'a str,
    derivation: &'a NixDerivationJsonNode,
    is_root: bool,
}

fn lower_aterm_derivation_node(
    request: AtermNodeLoweringRequest<'_>,
    context: &AtermLoweringContext<'_>,
    source_payloads: &mut BTreeMap<String, SourcePayload>,
) -> Result<ForeignDerivationNode, ImportDiagnostic> {
    let node_id = context.path_to_node_id.get(request.drv_path).cloned().ok_or_else(|| {
        diagnostic("missing-foreign-node-id", None, "foreign derivation path was not assigned a node ID")
    })?;
    let derivation = request.derivation;
    let outputs = lower_aterm_outputs(&derivation.outputs, &context.config.source_prefix)?;
    let fixed_output = nix_fixed_output_metadata(&derivation.outputs, &derivation.env)?;
    let input_derivations = lower_nix_input_derivations(&derivation.input_drvs, context.path_to_node_id)?;
    let source_refs = lower_aterm_source_refs(&derivation.input_srcs, source_payloads, context.config)?;
    let declared_references = nix_declared_references(&derivation.input_drvs, &derivation.input_srcs, context.closure)?;
    let cache_hints = if request.is_root {
        context.config.cache_hints.to_vec()
    } else {
        Vec::new()
    };
    let lowered = ForeignDerivationNode {
        node_id,
        original_derivation: request.drv_path.to_string(),
        name: derivation.name.clone(),
        system: derivation.system.clone(),
        builder: derivation.builder.clone(),
        args: derivation.args.clone(),
        env: derivation.env.clone(),
        outputs,
        input_derivations,
        source_refs,
        fixed_output: fixed_output.clone(),
        builtin: foreign_aterm_builtin(&context.config.source_prefix, fixed_output.is_some()),
        declared_references,
        sandbox_capabilities: Vec::new(),
        unsupported_features: Vec::new(),
        cache_hints,
    };
    debug_assert_eq!(lowered.original_derivation, request.drv_path);
    debug_assert_eq!(lowered.outputs.len(), derivation.outputs.len());
    Ok(lowered)
}

fn lower_aterm_outputs(
    outputs: &BTreeMap<String, NixDerivationJsonOutput>,
    source_prefix: &str,
) -> Result<BTreeMap<String, OutputDeclaration>, ImportDiagnostic> {
    if outputs.is_empty() {
        return Err(diagnostic("missing-output-declaration", None, "foreign derivation has no outputs"));
    }
    let output_count = outputs.len();
    let mut lowered = BTreeMap::new();
    for (name, output) in outputs {
        let path = nix_output_path(output)?;
        validate_foreign_store_path(path, source_prefix)?;
        if lowered.len() >= output_count {
            return Err(diagnostic(
                "foreign-output-count-out-of-range",
                None,
                "lowered foreign outputs exceeded input",
            ));
        }
        lowered.insert(name.clone(), OutputDeclaration {
            path: path.to_string(),
            hash: output.hash.clone(),
        });
    }
    debug_assert_eq!(lowered.len(), output_count);
    debug_assert!(!lowered.is_empty());
    Ok(lowered)
}

fn nix_output_path(output: &NixDerivationJsonOutput) -> Result<&str, ImportDiagnostic> {
    output
        .path
        .as_deref()
        .ok_or_else(|| diagnostic("missing-nix-output-path", None, "Nix output is missing a path"))
}

fn lower_nix_input_derivations(
    input_drvs: &BTreeMap<String, NixDerivationJsonInput>,
    path_to_node_id: &BTreeMap<String, String>,
) -> Result<Vec<InputDerivationEdge>, ImportDiagnostic> {
    let mut edges = Vec::with_capacity(MAX_GRAPH_EDGES);
    for (input_drv, input) in input_drvs {
        let input_node_id = path_to_node_id.get(input_drv).ok_or_else(|| {
            diagnostic("dangling-nix-input-derivation", None, "Nix input derivation is absent from closure")
        })?;
        if input.outputs.is_empty() {
            return Err(diagnostic("missing-nix-input-output", None, "Nix input derivation has no output names"));
        }
        for output_name in sorted_strings(input.outputs.clone()) {
            if edges.len() >= MAX_GRAPH_EDGES {
                return Err(diagnostic("edge-limit-exceeded", None, "Nix input derivation edge count exceeds limit"));
            }
            edges.push(InputDerivationEdge {
                node_id: input_node_id.clone(),
                output_name,
            });
        }
    }
    edges.sort();
    debug_assert!(edges.len() <= MAX_GRAPH_EDGES);
    debug_assert!(edges.iter().all(|edge| path_to_node_id.values().any(|node_id| node_id == &edge.node_id)));
    Ok(edges)
}

fn lower_aterm_source_refs(
    input_srcs: &[String],
    source_payloads: &mut BTreeMap<String, SourcePayload>,
    config: &AtermProducerConfig,
) -> Result<Vec<SourceRef>, ImportDiagnostic> {
    let mut refs = Vec::with_capacity(input_srcs.len());
    for input_src in sorted_strings(input_srcs.to_vec()) {
        validate_foreign_store_path(&input_src, &config.source_prefix)?;
        let payload_id = foreign_source_payload_id(&input_src, &config.source_prefix);
        source_payloads.entry(payload_id.clone()).or_insert_with(|| SourcePayload {
            payload_id: payload_id.clone(),
            kind: format!("{}-input-source", foreign_store_namespace(&config.source_prefix)),
            content_ref: input_src.clone(),
            embedded_text: None,
            mirrors: Vec::new(),
            expected_content_blake3: None,
        });
        debug_assert!(refs.len() < input_srcs.len());
        refs.push(SourceRef {
            payload_id,
            field: SOURCE_REF_FIELD.to_string(),
        });
    }
    refs.sort();
    debug_assert_eq!(refs.len(), input_srcs.len());
    debug_assert!(refs.len() <= MAX_SOURCE_PAYLOADS);
    Ok(refs)
}

fn nix_fixed_output_metadata(
    outputs: &BTreeMap<String, NixDerivationJsonOutput>,
    env: &BTreeMap<String, String>,
) -> Result<Option<FixedOutputMetadata>, ImportDiagnostic> {
    let output_hash_mode = env.get(OUTPUT_HASH_MODE_ENV).map(String::as_str).filter(|mode| !mode.is_empty());
    let mut metadata: Option<FixedOutputMetadata> = None;
    for output in outputs.values() {
        let Some(digest) = output.hash.as_ref() else {
            continue;
        };
        let hash_algo = output.hash_algo.as_deref().unwrap_or(SHA256_ALGORITHM);
        let (algorithm, recursive) = parse_nix_fixed_output_hash(hash_algo, output_hash_mode)?;
        let candidate = FixedOutputMetadata {
            algorithm,
            digest: digest.clone(),
            recursive,
        };
        if let Some(existing) = metadata.as_ref() {
            if existing != &candidate {
                return Err(diagnostic(
                    "conflicting-fixed-output-hashes",
                    None,
                    "Nix derivation outputs declare conflicting fixed-output hashes",
                ));
            }
        } else {
            metadata = Some(candidate);
        }
    }
    debug_assert!(metadata.is_none() || !outputs.is_empty());
    debug_assert!(metadata.as_ref().is_none_or(|value| !value.digest.is_empty()));
    Ok(metadata)
}

fn parse_nix_fixed_output_hash(
    hash_algo: &str,
    output_hash_mode: Option<&str>,
) -> Result<(String, bool), ImportDiagnostic> {
    let (algorithm, algorithm_marks_recursive) = parse_nix_hash_algorithm(hash_algo);
    let mode_marks_recursive = match output_hash_mode {
        None => None,
        Some(RECURSIVE_OUTPUT_HASH_MODE) => Some(true),
        Some(FLAT_OUTPUT_HASH_MODE) => Some(false),
        Some(_) => {
            return Err(diagnostic(
                "unsupported-fixed-output-hash-mode",
                None,
                "Nix derivation declares an unsupported fixed-output hash mode",
            ));
        }
    };
    if algorithm_marks_recursive && mode_marks_recursive == Some(false) {
        return Err(diagnostic(
            "conflicting-fixed-output-hash-mode",
            None,
            "Nix derivation hash algorithm and outputHashMode declare conflicting fixed-output modes",
        ));
    }
    Ok((algorithm, mode_marks_recursive.unwrap_or(algorithm_marks_recursive)))
}

fn parse_nix_hash_algorithm(hash_algo: &str) -> (String, bool) {
    if let Some(stripped) = hash_algo.strip_prefix("r:") {
        return (stripped.to_string(), true);
    }
    (hash_algo.to_string(), false)
}

fn nix_declared_references(
    input_drvs: &BTreeMap<String, NixDerivationJsonInput>,
    input_srcs: &[String],
    closure: &NixDerivationJsonClosure,
) -> Result<Vec<String>, ImportDiagnostic> {
    let mut references = input_srcs.to_vec();
    for (input_drv, input) in input_drvs {
        let input_derivation = closure.get(input_drv).ok_or_else(|| {
            diagnostic("dangling-nix-input-derivation", None, "Nix input derivation is absent from closure")
        })?;
        for output_name in &input.outputs {
            let output = input_derivation.outputs.get(output_name).ok_or_else(|| {
                diagnostic("dangling-nix-input-output", None, "Nix input output is absent from closure")
            })?;
            references.push(nix_output_path(output)?.to_string());
        }
    }
    Ok(sorted_strings(references))
}

fn foreign_node_id(drv_path: &str, source_prefix: &str) -> Result<String, ImportDiagnostic> {
    let component = drv_path.rsplit('/').next().filter(|component| !component.is_empty()).ok_or_else(|| {
        diagnostic("invalid-foreign-derivation-path", None, "foreign derivation path has no basename")
    })?;
    let node_id = format!("{}:{component}", foreign_store_namespace(source_prefix));
    debug_assert!(node_id.contains(':'));
    debug_assert!(node_id.ends_with(component));
    Ok(node_id)
}

fn foreign_source_payload_id(input_src: &str, source_prefix: &str) -> String {
    let digest = blake3_hex(input_src);
    let short_digest = &digest[..SOURCE_PAYLOAD_HASH_HEX_CHARS];
    format!("{}-source:{short_digest}", foreign_store_namespace(source_prefix))
}

fn foreign_aterm_builtin(source_prefix: &str, fixed_output: bool) -> String {
    if fixed_output {
        return FIXED_OUTPUT_FETCH_BUILTIN.to_string();
    }
    format!("{}.derivation", foreign_store_namespace(source_prefix))
}

fn foreign_store_namespace(source_prefix: &str) -> &'static str {
    if source_prefix == NIX_SOURCE_PREFIX {
        "nix"
    } else {
        "guix"
    }
}

fn validate_nix_derivation_path(path: &str) -> Result<(), ImportDiagnostic> {
    validate_nix_store_path(path)?;
    if !path.ends_with(NIX_DERIVATION_SUFFIX) {
        return Err(diagnostic("invalid-nix-derivation-path", None, "Nix derivation path must end in .drv"));
    }
    Ok(())
}

fn validate_nix_store_path(path: &str) -> Result<(), ImportDiagnostic> {
    require_field_limit(path, None, "nix-store-path")?;
    if !path.starts_with(NIX_STORE_PREFIX_WITH_SLASH) {
        return Err(diagnostic("invalid-nix-store-path", None, "Nix path must be under /nix/store"));
    }
    Ok(())
}

fn nix_hash_domain_records(closure: &NixDerivationJsonClosure) -> Vec<HashDomainRecord> {
    let mut records = closure
        .keys()
        .map(|drv_path| HashDomainRecord {
            domain: NIX_COMPATIBLE_HASH_DOMAIN.to_string(),
            kind: DERIVATION_STORE_PATH_HASH_KIND.to_string(),
            algorithm: NIX_STORE_PATH_SHA256_ALGORITHM.to_string(),
            value: drv_path.clone(),
        })
        .collect::<Vec<_>>();
    records.sort();
    records.dedup();
    records
}

fn aterm_package_metadata_digest(config: &AtermProducerConfig) -> String {
    blake3_hex(&format!(
        "{}:{}:{}:{}:{}:{:?}",
        config.producer_kind,
        config.package_name,
        config.system,
        config.root_derivation,
        config.producer_identity,
        sorted_strings(config.unsupported_metadata_classes.clone())
    ))
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
    debug_assert_eq!(translated.node_id, node.node_id);
    debug_assert_eq!(translated.outputs.len(), node.outputs.len());
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
    debug_assert!(node.cache_hints.len() <= MAX_CACHE_HINTS);
    debug_assert!(node.sandbox_capabilities.len() <= MAX_SANDBOX_CAPABILITIES);
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
    debug_assert_eq!(payloads.len(), graph.source_payloads.len());
    debug_assert!(payloads.len() <= MAX_SOURCE_PAYLOADS);
    Ok(payloads)
}

fn recompute_outputs(
    node: &ForeignDerivationNode,
    policy: &TranslationPolicy,
) -> Result<BTreeMap<String, OutputDeclaration>, ImportDiagnostic> {
    if node.outputs.len() == EMPTY_OUTPUT_COUNT {
        return Err(diagnostic("missing-output-declaration", Some(&node.node_id), "node has no output declarations"));
    }
    if policy.output_path_recompute_mode == PRESERVE_CACHE_PATHS_MODE {
        return Ok(node.outputs.clone());
    }
    if policy.output_path_recompute_mode != RECOMPUTE_BLAKE3_MODE {
        return Err(diagnostic(
            "unsupported-output-recompute-mode",
            Some(&node.node_id),
            "output path recomputation mode is not supported",
        ));
    }
    let output_count = node.outputs.len();
    let mut outputs = BTreeMap::new();
    for (output_name, output) in &node.outputs {
        let digest_input = format!("{}:{}:{}:{}", node.node_id, node.name, output_name, policy.target_prefix);
        let digest = blake3_hex(&digest_input);
        let short_digest = &digest[..OUTPUT_HASH_HEX_CHARS];
        if outputs.len() >= output_count {
            return Err(diagnostic(
                "output-count-out-of-range",
                Some(&node.node_id),
                "recomputed outputs exceeded input",
            ));
        }
        outputs.insert(output_name.clone(), OutputDeclaration {
            path: format!("{}/{short_digest}-{}", policy.target_prefix, node.name),
            hash: output.hash.clone(),
        });
    }
    debug_assert_eq!(outputs.len(), output_count);
    debug_assert!(!outputs.is_empty());
    Ok(outputs)
}

pub(crate) fn validate_graph(graph: &ForeignDerivationGraph) -> Result<(), ImportDiagnostic> {
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
    validate_hash_domains(graph)?;
    validate_unique_ids(graph)?;
    validate_roots(graph)?;
    validate_source_refs(graph)?;
    validate_field_limits(graph)?;
    debug_assert!(!graph.nodes.is_empty());
    debug_assert!(graph.nodes.len() <= MAX_GRAPH_NODES);
    Ok(())
}

fn validate_hash_domains(graph: &ForeignDerivationGraph) -> Result<(), ImportDiagnostic> {
    if graph.hash_domains.len() > MAX_HASH_DOMAIN_RECORDS {
        return Err(diagnostic(
            "hash-domain-limit-exceeded",
            None,
            "foreign graph declares too many hash-domain records",
        ));
    }
    let mut has_nix_identity = false;
    for record in &graph.hash_domains {
        require_field_limit(&record.domain, None, "hash-domain")?;
        require_field_limit(&record.kind, None, "hash-kind")?;
        require_field_limit(&record.algorithm, None, "hash-algorithm")?;
        require_field_limit(&record.value, None, "hash-value")?;
        match record.domain.as_str() {
            NIX_COMPATIBLE_HASH_DOMAIN => {
                has_nix_identity = true;
                if record.algorithm == DIGEST_ALGORITHM {
                    return Err(diagnostic(
                        "hash-domain-mismatch",
                        None,
                        "BLAKE3 receipt digest cannot be used as a Nix-compatible identity",
                    ));
                }
            }
            MANTLE_RECEIPT_HASH_DOMAIN => {
                if record.algorithm != DIGEST_ALGORITHM {
                    return Err(diagnostic("hash-domain-mismatch", None, "Mantle receipt identity must use BLAKE3"));
                }
            }
            _ => {
                return Err(diagnostic("unknown-hash-domain", None, "hash-domain record declares an unknown domain"));
            }
        }
    }
    if graph.producer.kind == NIXPKGS_PRODUCER_KIND && !has_nix_identity {
        return Err(diagnostic(
            "missing-nix-compatible-identity",
            None,
            "nixpkgs producer graph is missing Nix-compatible derivation identity records",
        ));
    }
    debug_assert!(graph.hash_domains.len() <= MAX_HASH_DOMAIN_RECORDS);
    if graph.producer.kind == NIXPKGS_PRODUCER_KIND {
        debug_assert!(has_nix_identity);
    }
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
    debug_assert_eq!(node_ids.len(), graph.nodes.len());
    debug_assert_eq!(payload_ids.len(), graph.source_payloads.len());
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
    debug_assert!(!graph.root_derivation_ids.is_empty());
    debug_assert!(graph.root_derivation_ids.iter().all(|root| node_ids.contains(root.as_str())));
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
    if policy.output_path_recompute_mode != RECOMPUTE_BLAKE3_MODE
        && policy.output_path_recompute_mode != PRESERVE_CACHE_PATHS_MODE
    {
        return Err(diagnostic(
            "unsupported-output-recompute-mode",
            None,
            "output path recomputation mode is not supported",
        ));
    }
    if policy.output_path_recompute_mode == PRESERVE_CACHE_PATHS_MODE
        && (policy.source_prefixes.len() != 1 || policy.source_prefixes[0] != policy.target_prefix)
    {
        return Err(diagnostic(
            "preserved-cache-prefix-mismatch",
            None,
            "preserved cache paths require one source prefix equal to the target prefix",
        ));
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
    debug_assert_eq!(keys.len(), index.entries.len());
    debug_assert!(index.entries.len() <= MAX_PACKAGE_INDEX_ENTRIES);
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
    let hash_domains = receipt_hash_domains(graph, ReceiptDigests {
        raw_graph: &raw_graph_digest,
        translation_policy: &translation_policy_digest,
        translated_graph: &translated_graph_digest,
    });
    debug_assert!(!translated_graph_digest.is_empty());
    let receipt = ImportReceipt {
        schema: IMPORT_RECEIPT_SCHEMA.to_string(),
        producer_identity: graph.producer.identity.clone(),
        raw_graph_digest,
        translation_policy_digest,
        translated_graph_digest,
        package_index_digest,
        fetch_cache_policy_digest: blake3_hex(&format!("cache:{:?}", policy.trusted_cache_scopes)),
        sandbox_policy_digest: blake3_hex(&format!("sandbox:{:?}", policy.allowed_sandbox_capabilities)),
        hash_domains,
        diagnostics: translated_graph.diagnostics.clone(),
        non_claims: foreign_import_non_claims(),
    };
    debug_assert_eq!(receipt.producer_identity, graph.producer.identity);
    Ok(receipt)
}

struct ReceiptDigests<'a> {
    raw_graph: &'a str,
    translation_policy: &'a str,
    translated_graph: &'a str,
}

fn receipt_hash_domains(graph: &ForeignDerivationGraph, digests: ReceiptDigests<'_>) -> Vec<HashDomainRecord> {
    if graph.hash_domains.is_empty() {
        return Vec::new();
    }
    let mut records = graph.hash_domains.clone();
    records.push(HashDomainRecord {
        domain: MANTLE_RECEIPT_HASH_DOMAIN.to_string(),
        kind: RAW_GRAPH_HASH_KIND.to_string(),
        algorithm: DIGEST_ALGORITHM.to_string(),
        value: digests.raw_graph.to_string(),
    });
    records.push(HashDomainRecord {
        domain: MANTLE_RECEIPT_HASH_DOMAIN.to_string(),
        kind: TRANSLATION_POLICY_HASH_KIND.to_string(),
        algorithm: DIGEST_ALGORITHM.to_string(),
        value: digests.translation_policy.to_string(),
    });
    records.push(HashDomainRecord {
        domain: MANTLE_RECEIPT_HASH_DOMAIN.to_string(),
        kind: TRANSLATED_GRAPH_HASH_KIND.to_string(),
        algorithm: DIGEST_ALGORITHM.to_string(),
        value: digests.translated_graph.to_string(),
    });
    records.sort();
    records.dedup();
    debug_assert!(records.len() >= graph.hash_domains.len());
    debug_assert!(records.len() <= MAX_HASH_DOMAIN_RECORDS.saturating_add(RECEIPT_HASH_DOMAIN_ADDITIONS));
    records
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

pub(crate) fn foreign_import_non_claims() -> Vec<String> {
    vec![
        "not-build-success".to_string(),
        "not-package-correctness".to_string(),
        "not-bootstrap-parity".to_string(),
        "not-output-trust".to_string(),
        "not-reproducibility".to_string(),
        "not-foreign-frontend-availability".to_string(),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    const INVALID_BUILTIN: &str = "unsupported:magic";
    const OVERLAY_METADATA_CLASS: &str = "nix-overlay-order";
    const OVERSIZED_FIELD_BYTES: usize = MAX_FIELD_BYTES + 1;
    const NIXPKGS_HELLO_DRV: &str = "/nix/store/22222222222222222222222222222222-hello.drv";
    const NIXPKGS_SOURCE_DRV: &str = "/nix/store/44444444444444444444444444444444-hello-source.drv";
    const NIXPKGS_DERIVATION_COUNT: usize = 2;
    const NIXPKGS_HELLO_OUT: &str = "/nix/store/11111111111111111111111111111111-hello";
    const NIXPKGS_SOURCE_OUT: &str = "/nix/store/00000000000000000000000000000000-hello-source";

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
    fn nixpkgs_derivation_json_lowering_preserves_identities_and_substitution_policy() {
        let closure = nixpkgs_hello_closure();
        let config = nixpkgs_producer_config();
        let artifacts = lower_nix_derivation_json_closure(&closure, &config).unwrap();
        let policy = nixpkgs_fixture_policy();

        let (translated, receipt) =
            translate_foreign_graph(&artifacts.graph, Some(&artifacts.package_index), &policy).unwrap();
        let plan = plan_mantle_foreign_import(&translated, &artifacts.package_index, HELLO_PACKAGE_NAME, HELLO_SYSTEM)
            .unwrap();

        assert_eq!(artifacts.graph.producer.kind, NIXPKGS_PRODUCER_KIND);
        assert_eq!(artifacts.graph.root_derivation_ids.len(), REQUIRED_HELLO_ROOT_COUNT);
        assert!(artifacts.graph.hash_domains.iter().any(|record| record.domain == NIX_COMPATIBLE_HASH_DOMAIN));
        assert!(receipt.hash_domains.iter().any(|record| record.domain == MANTLE_RECEIPT_HASH_DOMAIN));
        assert!(receipt.raw_graph_digest.len() > OUTPUT_HASH_HEX_CHARS);
        assert_eq!(plan.roots[0].package_name, HELLO_PACKAGE_NAME);
        assert_eq!(plan.substitution_audit.len(), REQUIRED_HELLO_ROOT_COUNT);
        assert_eq!(plan.substitution_audit[0].cache_url, CACHE_NIXOS_ORG_URL);
        assert!(plan.substitution_audit[0].store_admission_required);
        assert!(plan.forbidden_process_invocations.is_empty());
    }

    #[test]
    fn nix_aterm_derivation_closure_lowering_preserves_graph_facts() {
        let closure = normalize_nix_aterm_derivation_closure(nixpkgs_hello_aterm_closure()).unwrap();
        let source = closure.get(NIXPKGS_SOURCE_DRV).expect("source drv should be present");
        let hello = closure.get(NIXPKGS_HELLO_DRV).expect("hello drv should be present");
        let artifacts = lower_nix_derivation_json_closure(&closure, &nixpkgs_producer_config()).unwrap();
        let policy = nixpkgs_fixture_policy();

        let (translated, receipt) =
            translate_foreign_graph(&artifacts.graph, Some(&artifacts.package_index), &policy).unwrap();

        assert_eq!(closure.len(), NIXPKGS_DERIVATION_COUNT);
        assert_eq!(hello.input_drvs[NIXPKGS_SOURCE_DRV].outputs, vec![OUT_OUTPUT_NAME.to_string()]);
        assert_eq!(
            source.outputs[OUT_OUTPUT_NAME].hash.as_deref(),
            Some("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")
        );
        assert_eq!(source.outputs[OUT_OUTPUT_NAME].hash_algo.as_deref(), Some("r:sha256"));
        assert!(artifacts.graph.hash_domains.iter().any(|record| record.value == NIXPKGS_HELLO_DRV));
        assert_eq!(translated.nodes.len(), NIXPKGS_DERIVATION_COUNT);
        assert!(receipt.hash_domains.iter().any(|record| record.domain == MANTLE_RECEIPT_HASH_DOMAIN));
    }

    #[test]
    fn fixed_output_hash_mode_preserves_modern_and_legacy_recursive_facts() {
        let modern = parse_nix_fixed_output_hash(SHA256_ALGORITHM, Some(RECURSIVE_OUTPUT_HASH_MODE)).unwrap();
        let legacy = parse_nix_fixed_output_hash("r:sha256", None).unwrap();
        let flat = parse_nix_fixed_output_hash(SHA256_ALGORITHM, Some(FLAT_OUTPUT_HASH_MODE)).unwrap();

        assert_eq!(modern, (SHA256_ALGORITHM.to_string(), true));
        assert_eq!(legacy, modern);
        assert_eq!(flat, (SHA256_ALGORITHM.to_string(), false));
    }

    #[test]
    fn fixed_output_hash_mode_rejects_unknown_and_conflicting_facts() {
        assert_error_class(
            parse_nix_fixed_output_hash(SHA256_ALGORITHM, Some("unknown")),
            "unsupported-fixed-output-hash-mode",
        );
        assert_error_class(
            parse_nix_fixed_output_hash("r:sha256", Some(FLAT_OUTPUT_HASH_MODE)),
            "conflicting-fixed-output-hash-mode",
        );
    }

    #[test]
    fn prefix_aware_aterm_bundle_parses_nix_and_guix_paths() {
        let nix_inputs = nixpkgs_hello_aterm_inputs(NIX_SOURCE_PREFIX);
        let guix_inputs = nixpkgs_hello_aterm_inputs(GUIX_SOURCE_PREFIX);

        let nix = parse_prefix_aware_aterm_bundle(NIX_SOURCE_PREFIX, &nix_inputs).unwrap();
        let guix = parse_prefix_aware_aterm_bundle(GUIX_SOURCE_PREFIX, &guix_inputs).unwrap();

        assert_eq!(nix.len(), NIXPKGS_DERIVATION_COUNT);
        assert_eq!(guix.len(), NIXPKGS_DERIVATION_COUNT);
        assert!(nix.keys().all(|path| path.starts_with(NIX_SOURCE_PREFIX)));
        assert!(guix.keys().all(|path| path.starts_with(GUIX_SOURCE_PREFIX)));
        assert!(guix.values().all(|node| {
            node.outputs
                .values()
                .all(|output| output.path.as_deref().is_some_and(|path| path.starts_with(GUIX_SOURCE_PREFIX)))
        }));
        assert!(guix.values().all(|node| node.input_drvs.keys().all(|path| path.starts_with(GUIX_SOURCE_PREFIX))));
    }

    #[test]
    fn prefix_aware_aterm_lowering_emits_guix_graph_facts() {
        let inputs = nixpkgs_hello_aterm_inputs(GUIX_SOURCE_PREFIX);
        let closure = parse_prefix_aware_aterm_bundle(GUIX_SOURCE_PREFIX, &inputs).unwrap();
        let root_derivation = restore_store_prefix(NIXPKGS_HELLO_DRV, NIX_SOURCE_PREFIX, GUIX_SOURCE_PREFIX);
        let artifacts = lower_prefix_aware_aterm_closure(&closure, &AtermProducerConfig {
            source_prefix: GUIX_SOURCE_PREFIX.to_string(),
            producer_kind: "guix".to_string(),
            package_name: HELLO_PACKAGE_NAME.to_string(),
            system: HELLO_SYSTEM.to_string(),
            root_derivation,
            producer_identity: "guix:hello-fixture".to_string(),
            producer_revision: "fixture-revision".to_string(),
            cache_hints: Vec::new(),
            unsupported_metadata_classes: Vec::new(),
        })
        .unwrap();

        assert_eq!(artifacts.graph.producer.kind, "guix");
        assert_eq!(artifacts.graph.source_store_prefixes, vec![GUIX_SOURCE_PREFIX.to_string()]);
        assert_eq!(artifacts.graph.nodes.len(), NIXPKGS_DERIVATION_COUNT);
        assert!(artifacts.graph.nodes.iter().all(|node| node.node_id.starts_with("guix:")));
        assert!(artifacts.graph.nodes.iter().all(|node| node.original_derivation.starts_with(GUIX_SOURCE_PREFIX)));
        assert!(artifacts.graph.nodes.iter().any(|node| node.builtin == "guix.derivation"));
        assert!(artifacts.graph.nodes.iter().any(|node| {
            node.fixed_output
                .as_ref()
                .is_some_and(|fixed| fixed.algorithm == SHA256_ALGORITHM && fixed.recursive)
        }));
    }

    #[test]
    fn prefix_aware_aterm_bundle_rejects_malformed_and_mixed_inputs() {
        let malformed = vec![AtermDerivationInput {
            logical_path: NIXPKGS_SOURCE_DRV.to_string(),
            bytes: b"not-an-aterm-derivation".to_vec(),
        }];
        assert_error_class(parse_prefix_aware_aterm_bundle(NIX_SOURCE_PREFIX, &malformed), "malformed-foreign-aterm");

        let mut mixed = nixpkgs_hello_aterm_inputs(GUIX_SOURCE_PREFIX);
        mixed[0].bytes = include_bytes!("../tests/fixtures/foreign-import/nixpkgs-hello-source.drv").to_vec();
        assert_error_class(parse_prefix_aware_aterm_bundle(GUIX_SOURCE_PREFIX, &mixed), "mixed-foreign-store-prefix");

        let invalid_path = vec![AtermDerivationInput {
            logical_path: format!("{NIX_SOURCE_PREFIX}/not-a-store-object.drv"),
            bytes: include_bytes!("../tests/fixtures/foreign-import/nixpkgs-hello-source.drv").to_vec(),
        }];
        assert_error_class(
            parse_prefix_aware_aterm_bundle(NIX_SOURCE_PREFIX, &invalid_path),
            "invalid-foreign-store-path",
        );
        assert_error_class(
            parse_prefix_aware_aterm_bundle("/unsupported/store", &invalid_path),
            "unsupported-foreign-store-prefix",
        );

        let malformed_embedded_text =
            std::str::from_utf8(include_bytes!("../tests/fixtures/foreign-import/nixpkgs-hello-source.drv"))
                .expect("source fixture must be UTF-8")
                .replace("55555555555555555555555555555555-builtin-fetchurl", "not-a-store-object");
        let malformed_embedded = vec![AtermDerivationInput {
            logical_path: NIXPKGS_SOURCE_DRV.to_string(),
            bytes: malformed_embedded_text.into_bytes(),
        }];
        assert_error_class(
            parse_prefix_aware_aterm_bundle(NIX_SOURCE_PREFIX, &malformed_embedded),
            "invalid-foreign-store-path",
        );
    }

    #[test]
    fn prefix_aware_aterm_bundle_rejects_missing_duplicate_non_utf8_and_limits() {
        let mut missing = nixpkgs_hello_aterm_inputs(NIX_SOURCE_PREFIX);
        missing.remove(0);
        assert_error_class(
            parse_prefix_aware_aterm_bundle(NIX_SOURCE_PREFIX, &missing),
            "missing-foreign-input-derivation",
        );

        let source = nixpkgs_hello_aterm_inputs(NIX_SOURCE_PREFIX).remove(0);
        let duplicate = vec![source.clone(), source];
        assert_error_class(
            parse_prefix_aware_aterm_bundle(NIX_SOURCE_PREFIX, &duplicate),
            "duplicate-foreign-derivation-path",
        );

        let non_utf8 = vec![AtermDerivationInput {
            logical_path: NIXPKGS_SOURCE_DRV.to_string(),
            bytes: vec![u8::MAX],
        }];
        assert_error_class(parse_prefix_aware_aterm_bundle(NIX_SOURCE_PREFIX, &non_utf8), "non-utf8-foreign-aterm");

        let oversized = vec![AtermDerivationInput {
            logical_path: NIXPKGS_SOURCE_DRV.to_string(),
            bytes: vec![b'x'; MAX_ATERM_DERIVATION_BYTES + 1],
        }];
        assert_error_class(
            parse_prefix_aware_aterm_bundle(NIX_SOURCE_PREFIX, &oversized),
            "foreign-aterm-bytes-out-of-range",
        );
    }

    #[test]
    fn nix_closure_selection_filters_unreachable_and_requires_inputs() {
        let mut closure = nixpkgs_hello_closure();
        let unrelated_drv = "/nix/store/66666666666666666666666666666666-unrelated.drv";
        closure.insert(unrelated_drv.to_string(), nixpkgs_source_derivation());

        let selected = select_nix_derivation_json_closure(&closure, NIXPKGS_HELLO_DRV).unwrap();

        assert_eq!(selected.len(), NIXPKGS_DERIVATION_COUNT);
        assert!(selected.contains_key(NIXPKGS_HELLO_DRV));
        assert!(selected.contains_key(NIXPKGS_SOURCE_DRV));
        assert!(!selected.contains_key(unrelated_drv));

        let mut missing = nixpkgs_hello_closure();
        missing.remove(NIXPKGS_SOURCE_DRV);
        assert_error_class(
            select_nix_derivation_json_closure(&missing, NIXPKGS_HELLO_DRV),
            "missing-nix-input-derivation",
        );
    }

    #[test]
    fn nixpkgs_hash_domain_and_frontend_metadata_fail_closed() {
        let closure = nixpkgs_hello_closure();
        let mut artifacts = lower_nix_derivation_json_closure(&closure, &nixpkgs_producer_config()).unwrap();
        let policy = nixpkgs_fixture_policy();

        artifacts.graph.hash_domains[0].algorithm = DIGEST_ALGORITHM.to_string();
        assert_error_class(
            translate_foreign_graph(&artifacts.graph, Some(&artifacts.package_index), &policy),
            "hash-domain-mismatch",
        );

        let mut unsupported = lower_nix_derivation_json_closure(&closure, &nixpkgs_producer_config()).unwrap();
        unsupported.package_index.entries[0]
            .unsupported_metadata_classes
            .push(OVERLAY_METADATA_CLASS.to_string());
        assert_error_class(
            translate_foreign_graph(&unsupported.graph, Some(&unsupported.package_index), &policy),
            "unsupported-package-index-metadata",
        );
    }

    #[test]
    fn versioned_nix_derivation_export_normalizes_relative_paths_and_fod_env_outputs() {
        let export = NixDerivationJsonExport::Versioned(versioned_nix_export_with_fod_env_path(true));

        let closure = normalize_nix_derivation_json_export(export).unwrap();
        let source = closure.get(NIXPKGS_SOURCE_DRV).expect("source drv should normalize to absolute path");

        assert_eq!(closure.len(), REQUIRED_HELLO_ROOT_COUNT);
        assert_eq!(source.outputs[OUT_OUTPUT_NAME].path.as_deref(), Some(NIXPKGS_SOURCE_OUT));
        assert_eq!(source.input_srcs.len(), 0);
        assert!(source.builder.starts_with("builtin:"));
    }

    #[test]
    fn nix_environment_normalization_distinguishes_store_basenames_from_sri_hashes() {
        const SRI_HASH: &str = "sha256-XyvbrWKXB6p9hcYj+ZSqih0t7FWnPeUgW6wL9gWKL3w=";
        const STORE_BASENAME: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa-source";
        assert_eq!(normalize_nix_embedded_store_paths(SRI_HASH), SRI_HASH);
        assert_eq!(
            normalize_nix_embedded_store_paths(STORE_BASENAME),
            format!("{NIX_STORE_PREFIX_WITH_SLASH}{STORE_BASENAME}")
        );
    }

    #[test]
    fn versioned_nix_derivation_export_rejects_missing_output_path_without_env_fallback() {
        let export = NixDerivationJsonExport::Versioned(versioned_nix_export_with_fod_env_path(false));

        assert_error_class(normalize_nix_derivation_json_export(export), "missing-nix-output-path");
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

    fn nixpkgs_hello_closure() -> NixDerivationJsonClosure {
        let mut closure = BTreeMap::new();
        closure.insert(NIXPKGS_SOURCE_DRV.to_string(), nixpkgs_source_derivation());
        closure.insert(NIXPKGS_HELLO_DRV.to_string(), nixpkgs_hello_derivation());
        closure
    }

    fn nixpkgs_hello_aterm_closure() -> BTreeMap<String, nix_compat::derivation::Derivation> {
        let mut closure = BTreeMap::new();
        let source = nix_compat::derivation::Derivation::from_aterm_bytes(include_bytes!(
            "../tests/fixtures/foreign-import/nixpkgs-hello-source.drv"
        ))
        .expect("source .drv should parse");
        let hello = nix_compat::derivation::Derivation::from_aterm_bytes(include_bytes!(
            "../tests/fixtures/foreign-import/nixpkgs-hello-root.drv"
        ))
        .expect("hello .drv should parse");
        closure.insert(NIXPKGS_SOURCE_DRV.to_string(), source);
        closure.insert(NIXPKGS_HELLO_DRV.to_string(), hello);
        closure
    }

    fn nixpkgs_hello_aterm_inputs(source_prefix: &str) -> Vec<AtermDerivationInput> {
        let source_bytes = include_bytes!("../tests/fixtures/foreign-import/nixpkgs-hello-source.drv");
        let hello_bytes = include_bytes!("../tests/fixtures/foreign-import/nixpkgs-hello-root.drv");
        let source_path = restore_store_prefix(NIXPKGS_SOURCE_DRV, NIX_SOURCE_PREFIX, source_prefix);
        let hello_path = restore_store_prefix(NIXPKGS_HELLO_DRV, NIX_SOURCE_PREFIX, source_prefix);
        vec![
            AtermDerivationInput {
                logical_path: source_path,
                bytes: restore_store_prefix(
                    std::str::from_utf8(source_bytes).expect("source fixture must be UTF-8"),
                    NIX_SOURCE_PREFIX,
                    source_prefix,
                )
                .into_bytes(),
            },
            AtermDerivationInput {
                logical_path: hello_path,
                bytes: restore_store_prefix(
                    std::str::from_utf8(hello_bytes).expect("root fixture must be UTF-8"),
                    NIX_SOURCE_PREFIX,
                    source_prefix,
                )
                .into_bytes(),
            },
        ]
    }

    fn versioned_nix_export_with_fod_env_path(include_env_out: bool) -> NixDerivationJsonVersionedExport {
        let mut env = BTreeMap::from([
            ("builder".to_string(), "builtin:fetchurl".to_string()),
            ("name".to_string(), "hello-source".to_string()),
            ("system".to_string(), "builtin".to_string()),
        ]);
        if include_env_out {
            env.insert(OUT_OUTPUT_NAME.to_string(), NIXPKGS_SOURCE_OUT.to_string());
        }
        let source_key = NIXPKGS_SOURCE_DRV
            .strip_prefix(NIX_STORE_PREFIX_WITH_SLASH)
            .expect("fixture drv should have Nix store prefix")
            .to_string();
        let mut derivations = BTreeMap::new();
        derivations.insert(source_key, NixDerivationJsonVersionedNode {
            name: "hello-source".to_string(),
            system: "builtin".to_string(),
            builder: "builtin:fetchurl".to_string(),
            args: Vec::new(),
            env,
            outputs: BTreeMap::from([(OUT_OUTPUT_NAME.to_string(), NixDerivationJsonOutput {
                path: None,
                hash: Some("sha256-aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa=".to_string()),
                hash_algo: None,
            })]),
            inputs: NixDerivationJsonVersionedInputs::default(),
        });
        NixDerivationJsonVersionedExport {
            derivations,
            version: 4,
        }
    }

    fn nixpkgs_source_derivation() -> NixDerivationJsonNode {
        let mut outputs = BTreeMap::new();
        outputs.insert(OUT_OUTPUT_NAME.to_string(), NixDerivationJsonOutput {
            path: Some(NIXPKGS_SOURCE_OUT.to_string()),
            hash: Some("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_string()),
            hash_algo: Some("r:sha256".to_string()),
        });
        NixDerivationJsonNode {
            name: "hello-source".to_string(),
            system: HELLO_SYSTEM.to_string(),
            builder: "/nix/store/55555555555555555555555555555555-builtin-fetchurl".to_string(),
            args: Vec::new(),
            env: BTreeMap::new(),
            outputs,
            input_drvs: BTreeMap::new(),
            input_srcs: Vec::new(),
        }
    }

    fn nixpkgs_hello_derivation() -> NixDerivationJsonNode {
        let mut outputs = BTreeMap::new();
        outputs.insert(OUT_OUTPUT_NAME.to_string(), NixDerivationJsonOutput {
            path: Some(NIXPKGS_HELLO_OUT.to_string()),
            hash: None,
            hash_algo: None,
        });
        let mut input_drvs = BTreeMap::new();
        input_drvs.insert(NIXPKGS_SOURCE_DRV.to_string(), NixDerivationJsonInput {
            outputs: vec![OUT_OUTPUT_NAME.to_string()],
        });
        NixDerivationJsonNode {
            name: HELLO_PACKAGE_NAME.to_string(),
            system: HELLO_SYSTEM.to_string(),
            builder: "/nix/store/33333333333333333333333333333333-bash/bin/bash".to_string(),
            args: vec!["-c".to_string(), "cp $src $out".to_string()],
            env: BTreeMap::from([
                ("out".to_string(), NIXPKGS_HELLO_OUT.to_string()),
                ("src".to_string(), NIXPKGS_SOURCE_OUT.to_string()),
            ]),
            outputs,
            input_drvs,
            input_srcs: Vec::new(),
        }
    }

    fn nixpkgs_producer_config() -> NixProducerConfig {
        NixProducerConfig {
            package_name: HELLO_PACKAGE_NAME.to_string(),
            system: HELLO_SYSTEM.to_string(),
            root_derivation: NIXPKGS_HELLO_DRV.to_string(),
            producer_identity: "nixpkgs:hello-fixture".to_string(),
            producer_revision: "fixture-revision".to_string(),
            cache_hints: vec![CacheHint {
                cache_url: CACHE_NIXOS_ORG_URL.to_string(),
                trust_scope: TRUSTED_CACHE_SCOPE.to_string(),
            }],
            unsupported_metadata_classes: Vec::new(),
        }
    }

    fn nixpkgs_fixture_policy() -> TranslationPolicy {
        let mut policy = fixture_policy(&[NIX_SOURCE_PREFIX]);
        policy
            .builtin_mappings
            .insert(NIX_DERIVATION_BUILTIN.to_string(), "mantle.foreign.nix.derivation".to_string());
        policy.trusted_cache_scopes.insert(TRUSTED_CACHE_SCOPE.to_string());
        policy
    }

    fn fixture_policy(source_prefixes: &[&str]) -> TranslationPolicy {
        let mut builtin_mappings = BTreeMap::new();
        builtin_mappings.insert(FIXED_OUTPUT_FETCH_BUILTIN.to_string(), "mantle.fetch".to_string());
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
