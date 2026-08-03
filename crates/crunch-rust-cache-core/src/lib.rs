#![cfg_attr(not(kani), feature(register_tool))]
#![register_tool(tigerstyle)]
//! Pure bounded identity and admission logic for local Rust unit results.
//!
//! This crate does not read files, inspect the environment, spawn processes,
//! or access castore. Shell code supplies normalized and verified facts.

use std::collections::BTreeMap;
use std::collections::BTreeSet;

use serde::Deserialize;
use serde::Serialize;

pub mod shared;

pub const RUST_ACTION_SCHEMA: &str = "mantle-rust-unit-action-v1";
pub const RUST_RESULT_SCHEMA: &str = "mantle-rust-unit-result-v1";
pub const RUST_RESULT_INDEX_SCHEMA: &str = "mantle-rust-unit-result-index-v1";
pub const RUST_ACTION_REF_PREFIX: &str = "mantle-rust-action://blake3/";
pub const RUST_RESULT_REF_PREFIX: &str = "mantle-rust-result://blake3/";
pub const RUST_RESULT_INDEX_REF_PREFIX: &str = "mantle-rust-result-index://blake3/";
pub const LOCAL_CACHE_POLICY_SCHEMA: &str = "mantle-rust-local-cache-policy-v1";
pub const LOCAL_RESULT_CONFLICT: &str = "local-rust-result-conflict";
pub const MAX_FEATURES: usize = 256;
pub const MAX_ARGUMENTS: usize = 4_096;
pub const MAX_ENVIRONMENT_ENTRIES: usize = 512;
pub const MAX_ARTIFACT_IDENTITIES: usize = 4_096;
pub const MAX_BUILD_FACTS: usize = 2_048;
pub const MAX_NATIVE_LINK_FACTS: usize = 2_048;
pub const MAX_RESULT_ARTIFACTS: usize = 4_096;
pub const MAX_RESULT_CANDIDATES: usize = 256;
pub const MAX_RECORD_BYTES: usize = 4_194_304;
pub const MAX_STRING_BYTES: usize = 16_384;
pub const MAX_RELATIVE_PATH_BYTES: usize = 4_096;
pub const MAX_TREE_BYTES: u64 = 8_589_934_592;
pub const MAX_TREE_DEPTH: u32 = 128;
pub const MAX_TREE_ENTRIES: u32 = 262_144;
pub const BLAKE3_HEX_CHARS: usize = 64;

const ACTION_DOMAIN: &[u8] = b"mantle.rust-unit.action.v1";
const RESULT_DOMAIN: &[u8] = b"mantle.rust-unit.result.v1";
const INDEX_DOMAIN: &[u8] = b"mantle.rust-unit.index.v1";
const ARTIFACT_SET_DOMAIN: &[u8] = b"mantle.rust-unit.artifact-set.v1";
const POLICY_DOMAIN: &[u8] = b"mantle.rust-unit.local-policy.v1";
const DOMAIN_SEPARATOR: u8 = 0;
const FILE_MODE_EXECUTABLE: u32 = 0o755;
const FILE_MODE_REGULAR: u32 = 0o644;

#[derive(Clone, Copy)]
struct ValidationCode<'a>(&'a str);

#[derive(Clone, Copy)]
struct TypedRefRule<'a> {
    prefix: &'a str,
    code: ValidationCode<'a>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(deny_unknown_fields)]
pub struct RustArtifactIdentity {
    pub role: String,
    pub name: String,
    pub digest_blake3: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(deny_unknown_fields)]
pub struct RustBuildFact {
    pub name: String,
    pub value_digest_blake3: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(deny_unknown_fields)]
pub struct RustSemanticArgument {
    pub value: String,
    pub contains_absolute_path: bool,
    pub absolute_paths_classified: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct RustUnitActionInput {
    pub unit_id: String,
    pub package_id: String,
    pub crate_name: String,
    pub target_kind: String,
    pub execution_kind: String,
    pub host_triple: String,
    pub target_triple: String,
    pub profile: String,
    pub mode: String,
    pub features: Vec<String>,
    pub source_digest_blake3: String,
    pub compiler_digest_blake3: String,
    pub compiler_version_digest_blake3: String,
    pub toolchain_closure_digest_blake3: String,
    pub execution_platform_digest_blake3: String,
    pub semantic_arguments: Vec<RustSemanticArgument>,
    pub admitted_environment: BTreeMap<String, String>,
    pub dependency_artifacts: Vec<RustArtifactIdentity>,
    pub host_artifacts: Vec<RustArtifactIdentity>,
    pub build_script_facts: Vec<RustBuildFact>,
    pub native_link_facts: Vec<RustBuildFact>,
    pub compiler_policy_digest_blake3: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct RustUnitAction {
    pub schema: String,
    pub action_ref: String,
    #[serde(flatten)]
    pub input: RustUnitActionInput,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum CastoreNodeKind {
    Directory,
    File,
    Symlink,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct CastoreNodeIdentity {
    pub kind: CastoreNodeKind,
    pub digest_blake3: String,
    pub size_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "kebab-case")]
pub enum RustArtifactKind {
    File,
    Symlink,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(deny_unknown_fields)]
pub struct RustResultArtifact {
    pub relative_path: String,
    pub kind: RustArtifactKind,
    pub mode: u32,
    pub size_bytes: u64,
    pub digest_blake3: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct RustUnitResultInput {
    pub action_ref: String,
    pub root_node: CastoreNodeIdentity,
    pub artifacts: Vec<RustResultArtifact>,
    pub producer_receipt_ref: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct RustUnitResult {
    pub schema: String,
    pub result_ref: String,
    #[serde(flatten)]
    pub input: RustUnitResultInput,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct RustResultIndex {
    pub schema: String,
    pub index_ref: String,
    pub action_ref: String,
    pub result_refs: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct LocalCachePolicy {
    pub schema: String,
    pub policy_id: String,
    pub reads_enabled: bool,
    pub writes_enabled: bool,
    pub execute_after_rejection: bool,
    pub max_candidates: u32,
    pub max_tree_entries: u32,
    pub max_tree_depth: u32,
    pub max_tree_bytes: u64,
}

impl Default for LocalCachePolicy {
    fn default() -> Self {
        Self {
            schema: LOCAL_CACHE_POLICY_SCHEMA.to_string(),
            policy_id: "mantle-rust-local-cache-default-v1".to_string(),
            reads_enabled: false,
            writes_enabled: false,
            execute_after_rejection: true,
            max_candidates: MAX_RESULT_CANDIDATES as u32,
            max_tree_entries: MAX_TREE_ENTRIES,
            max_tree_depth: MAX_TREE_DEPTH,
            max_tree_bytes: MAX_TREE_BYTES,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct LocalCandidateFacts {
    pub result: RustUnitResult,
    pub content_complete: bool,
    pub artifact_manifest_verified: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct LocalCandidateDecision {
    pub result_ref: String,
    pub admitted: bool,
    pub reason_codes: Vec<String>,
    pub artifact_set_digest_blake3: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct LocalReusePlan {
    pub selected_result_ref: Option<String>,
    pub conflict_class: Option<String>,
    pub decisions: Vec<LocalCandidateDecision>,
}

pub fn canonical_rust_action(input: RustUnitActionInput) -> Result<RustUnitAction, String> {
    let input = normalize_action_input(input);
    validate_action_input(&input)?;
    let action_ref = digest_ref(RUST_ACTION_REF_PREFIX, ACTION_DOMAIN, &input)?;
    let action = RustUnitAction {
        schema: RUST_ACTION_SCHEMA.to_string(),
        action_ref,
        input,
    };
    validate_rust_action(&action)?;
    assert_eq!(action.schema, RUST_ACTION_SCHEMA);
    assert!(action.action_ref.starts_with(RUST_ACTION_REF_PREFIX));
    Ok(action)
}

pub fn validate_rust_action(action: &RustUnitAction) -> Result<(), String> {
    if action.schema != RUST_ACTION_SCHEMA {
        return Err("rust-action-schema-unsupported".to_string());
    }
    validate_action_input(&action.input)?;
    let expected = digest_ref(RUST_ACTION_REF_PREFIX, ACTION_DOMAIN, &action.input)?;
    if action.action_ref != expected {
        return Err("rust-action-ref-mismatch".to_string());
    }
    ensure_record_bound(action, "rust-action-record-too-large")?;
    assert_eq!(action.action_ref.len(), RUST_ACTION_REF_PREFIX.len().saturating_add(BLAKE3_HEX_CHARS));
    assert_eq!(action.schema, RUST_ACTION_SCHEMA);
    Ok(())
}

pub fn canonical_rust_result(input: RustUnitResultInput) -> Result<RustUnitResult, String> {
    let input = normalize_result_input(input);
    validate_result_input(&input)?;
    let result_ref = digest_ref(RUST_RESULT_REF_PREFIX, RESULT_DOMAIN, &input)?;
    let result = RustUnitResult {
        schema: RUST_RESULT_SCHEMA.to_string(),
        result_ref,
        input,
    };
    validate_rust_result(&result)?;
    assert_eq!(result.schema, RUST_RESULT_SCHEMA);
    assert!(result.result_ref.starts_with(RUST_RESULT_REF_PREFIX));
    Ok(result)
}

pub fn validate_rust_result(result: &RustUnitResult) -> Result<(), String> {
    if result.schema != RUST_RESULT_SCHEMA {
        return Err("rust-result-schema-unsupported".to_string());
    }
    validate_result_input(&result.input)?;
    let expected = digest_ref(RUST_RESULT_REF_PREFIX, RESULT_DOMAIN, &result.input)?;
    if result.result_ref != expected {
        return Err("rust-result-ref-mismatch".to_string());
    }
    ensure_record_bound(result, "rust-result-record-too-large")?;
    assert_eq!(result.result_ref.len(), RUST_RESULT_REF_PREFIX.len().saturating_add(BLAKE3_HEX_CHARS));
    assert_eq!(result.schema, RUST_RESULT_SCHEMA);
    Ok(())
}

pub fn canonical_result_index(action_ref: String, result_refs: Vec<String>) -> Result<RustResultIndex, String> {
    validate_typed_ref(&action_ref, TypedRefRule {
        prefix: RUST_ACTION_REF_PREFIX,
        code: ValidationCode("rust-index-action-ref-invalid"),
    })?;
    let result_refs = sorted_unique(result_refs);
    validate_result_refs(&result_refs)?;
    let hashable = RustIndexHashable {
        schema: RUST_RESULT_INDEX_SCHEMA,
        action_ref: &action_ref,
        result_refs: &result_refs,
    };
    let index_ref = digest_ref(RUST_RESULT_INDEX_REF_PREFIX, INDEX_DOMAIN, &hashable)?;
    let index = RustResultIndex {
        schema: RUST_RESULT_INDEX_SCHEMA.to_string(),
        index_ref,
        action_ref,
        result_refs,
    };
    validate_result_index(&index)?;
    assert_eq!(index.schema, RUST_RESULT_INDEX_SCHEMA);
    assert!(index.index_ref.starts_with(RUST_RESULT_INDEX_REF_PREFIX));
    Ok(index)
}

pub fn validate_result_index(index: &RustResultIndex) -> Result<(), String> {
    if index.schema != RUST_RESULT_INDEX_SCHEMA {
        return Err("rust-result-index-schema-unsupported".to_string());
    }
    validate_typed_ref(&index.action_ref, TypedRefRule {
        prefix: RUST_ACTION_REF_PREFIX,
        code: ValidationCode("rust-index-action-ref-invalid"),
    })?;
    validate_result_refs(&index.result_refs)?;
    if index.result_refs != sorted_unique(index.result_refs.clone()) {
        return Err("rust-result-index-not-canonical".to_string());
    }
    let hashable = RustIndexHashable {
        schema: RUST_RESULT_INDEX_SCHEMA,
        action_ref: &index.action_ref,
        result_refs: &index.result_refs,
    };
    let expected = digest_ref(RUST_RESULT_INDEX_REF_PREFIX, INDEX_DOMAIN, &hashable)?;
    if index.index_ref != expected {
        return Err("rust-result-index-ref-mismatch".to_string());
    }
    ensure_record_bound(index, "rust-result-index-too-large")?;
    assert_eq!(index.schema, RUST_RESULT_INDEX_SCHEMA);
    assert!(index.result_refs.len() <= MAX_RESULT_CANDIDATES);
    Ok(())
}

pub fn local_cache_policy_digest(policy: &LocalCachePolicy) -> Result<String, String> {
    validate_local_cache_policy(policy)?;
    let bytes = serde_json::to_vec(policy).map_err(|error| format!("rust-cache-policy-json:{error}"))?;
    let digest = domain_digest(POLICY_DOMAIN, &bytes);
    assert_eq!(digest.len(), BLAKE3_HEX_CHARS);
    assert!(digest.bytes().all(|byte| byte.is_ascii_hexdigit()));
    Ok(digest)
}

pub fn validate_local_cache_policy(policy: &LocalCachePolicy) -> Result<(), String> {
    if policy.schema != LOCAL_CACHE_POLICY_SCHEMA {
        return Err("rust-cache-policy-schema-unsupported".to_string());
    }
    validate_identifier(&policy.policy_id, ValidationCode("rust-cache-policy-id-invalid"))?;
    if policy.max_candidates == 0 || policy.max_candidates > MAX_RESULT_CANDIDATES as u32 {
        return Err("rust-cache-policy-candidate-limit-invalid".to_string());
    }
    if policy.max_tree_entries == 0 || policy.max_tree_entries > MAX_TREE_ENTRIES {
        return Err("rust-cache-policy-tree-entry-limit-invalid".to_string());
    }
    if policy.max_tree_depth == 0 || policy.max_tree_depth > MAX_TREE_DEPTH {
        return Err("rust-cache-policy-tree-depth-limit-invalid".to_string());
    }
    if policy.max_tree_bytes == 0 || policy.max_tree_bytes > MAX_TREE_BYTES {
        return Err("rust-cache-policy-tree-byte-limit-invalid".to_string());
    }
    assert!(policy.max_candidates <= MAX_RESULT_CANDIDATES as u32);
    assert!(policy.max_tree_bytes <= MAX_TREE_BYTES);
    Ok(())
}

pub fn plan_local_reuse(
    action_ref: &str,
    policy: &LocalCachePolicy,
    candidates: Vec<LocalCandidateFacts>,
) -> Result<LocalReusePlan, String> {
    validate_typed_ref(action_ref, TypedRefRule {
        prefix: RUST_ACTION_REF_PREFIX,
        code: ValidationCode("rust-local-action-ref-invalid"),
    })?;
    validate_local_cache_policy(policy)?;
    let max_candidates =
        usize::try_from(policy.max_candidates).map_err(|_| "rust-local-candidate-limit-invalid".to_string())?;
    if candidates.len() > max_candidates {
        return Err("rust-local-candidate-limit-exceeded".to_string());
    }
    let unique = unique_candidates(candidates);
    let mut decisions = Vec::with_capacity(unique.len());
    let mut admitted = BTreeMap::<String, Vec<String>>::new();
    for candidate in unique.into_values() {
        let decision = evaluate_local_candidate(action_ref, policy, candidate);
        if let Some(digest) = decision.artifact_set_digest_blake3.as_ref().filter(|_| decision.admitted) {
            admitted.entry(digest.clone()).or_default().push(decision.result_ref.clone());
        }
        decisions.push(decision);
    }
    assert!(decisions.len() <= max_candidates);
    assert!(admitted.len() <= decisions.len());
    finish_local_reuse_plan(decisions, admitted)
}

fn normalize_action_input(mut input: RustUnitActionInput) -> RustUnitActionInput {
    input.features = sorted_unique(input.features);
    input.dependency_artifacts = sorted_unique(input.dependency_artifacts);
    input.host_artifacts = sorted_unique(input.host_artifacts);
    input.build_script_facts = sorted_unique(input.build_script_facts);
    input.native_link_facts = sorted_unique(input.native_link_facts);
    input
}

fn normalize_result_input(mut input: RustUnitResultInput) -> RustUnitResultInput {
    input.artifacts = sorted_unique(input.artifacts);
    input
}

fn validate_action_input(input: &RustUnitActionInput) -> Result<(), String> {
    for (value, code) in [
        (&input.unit_id, "rust-action-unit-id-invalid"),
        (&input.package_id, "rust-action-package-id-invalid"),
        (&input.crate_name, "rust-action-crate-name-invalid"),
        (&input.target_kind, "rust-action-target-kind-invalid"),
        (&input.execution_kind, "rust-action-execution-kind-invalid"),
        (&input.host_triple, "rust-action-host-triple-invalid"),
        (&input.target_triple, "rust-action-target-triple-invalid"),
        (&input.profile, "rust-action-profile-invalid"),
        (&input.mode, "rust-action-mode-invalid"),
    ] {
        validate_identifier(value, ValidationCode(code))?;
    }
    validate_action_digests(input)?;
    validate_action_collections(input)?;
    assert!(input.features.len() <= MAX_FEATURES);
    assert!(input.semantic_arguments.len() <= MAX_ARGUMENTS);
    Ok(())
}

fn validate_action_digests(input: &RustUnitActionInput) -> Result<(), String> {
    for (value, code) in [
        (&input.source_digest_blake3, "rust-action-source-digest-invalid"),
        (&input.compiler_digest_blake3, "rust-action-compiler-digest-invalid"),
        (&input.compiler_version_digest_blake3, "rust-action-compiler-version-digest-invalid"),
        (&input.toolchain_closure_digest_blake3, "rust-action-toolchain-closure-digest-invalid"),
        (&input.execution_platform_digest_blake3, "rust-action-platform-digest-invalid"),
        (&input.compiler_policy_digest_blake3, "rust-action-policy-digest-invalid"),
    ] {
        validate_blake3(value, ValidationCode(code))?;
    }
    assert_eq!(input.source_digest_blake3.len(), BLAKE3_HEX_CHARS);
    assert_eq!(input.compiler_digest_blake3.len(), BLAKE3_HEX_CHARS);
    Ok(())
}

fn validate_action_collections(input: &RustUnitActionInput) -> Result<(), String> {
    validate_sorted_strings(&input.features, MAX_FEATURES, "rust-action-features-invalid")?;
    if input.semantic_arguments.len() > MAX_ARGUMENTS {
        return Err("rust-action-argument-limit-exceeded".to_string());
    }
    for argument in &input.semantic_arguments {
        validate_semantic_argument(argument)?;
    }
    validate_environment(&input.admitted_environment)?;
    validate_artifact_identities(&input.dependency_artifacts, "rust-action-dependency-artifacts-invalid")?;
    validate_artifact_identities(&input.host_artifacts, "rust-action-host-artifacts-invalid")?;
    validate_build_facts(&input.build_script_facts, MAX_BUILD_FACTS, "rust-action-build-facts-invalid")?;
    validate_build_facts(&input.native_link_facts, MAX_NATIVE_LINK_FACTS, "rust-action-native-link-facts-invalid")?;
    assert!(input.admitted_environment.len() <= MAX_ENVIRONMENT_ENTRIES);
    assert!(input.dependency_artifacts.len() <= MAX_ARTIFACT_IDENTITIES);
    Ok(())
}

fn validate_semantic_argument(argument: &RustSemanticArgument) -> Result<(), String> {
    validate_string(&argument.value, ValidationCode("rust-action-argument-invalid"))?;
    if argument.contains_absolute_path && !argument.absolute_paths_classified {
        return Err("rust-action-unclassified-absolute-path".to_string());
    }
    if !argument.contains_absolute_path && argument.absolute_paths_classified {
        return Err("rust-action-spurious-path-classification".to_string());
    }
    assert!(!argument.value.is_empty());
    assert_eq!(argument.contains_absolute_path, argument.absolute_paths_classified);
    Ok(())
}

fn validate_environment(environment: &BTreeMap<String, String>) -> Result<(), String> {
    if environment.len() > MAX_ENVIRONMENT_ENTRIES {
        return Err("rust-action-environment-limit-exceeded".to_string());
    }
    for (name, value) in environment {
        validate_identifier(name, ValidationCode("rust-action-environment-name-invalid"))?;
        validate_string(value, ValidationCode("rust-action-environment-value-invalid"))?;
    }
    assert!(environment.len() <= MAX_ENVIRONMENT_ENTRIES);
    assert!(environment.keys().all(|name| !name.is_empty()));
    Ok(())
}

fn validate_artifact_identities(artifacts: &[RustArtifactIdentity], code: &str) -> Result<(), String> {
    if artifacts.len() > MAX_ARTIFACT_IDENTITIES || artifacts != sorted_unique(artifacts.to_vec()) {
        return Err(code.to_string());
    }
    for artifact in artifacts {
        validate_identifier(&artifact.role, ValidationCode(code))?;
        validate_identifier(&artifact.name, ValidationCode(code))?;
        validate_blake3(&artifact.digest_blake3, ValidationCode(code))?;
    }
    assert!(artifacts.len() <= MAX_ARTIFACT_IDENTITIES);
    assert!(artifacts.windows(2).all(|pair| pair[0] < pair[1]));
    Ok(())
}

fn validate_build_facts(facts: &[RustBuildFact], limit: usize, code: &str) -> Result<(), String> {
    if facts.len() > limit || facts != sorted_unique(facts.to_vec()) {
        return Err(code.to_string());
    }
    for fact in facts {
        validate_identifier(&fact.name, ValidationCode(code))?;
        validate_blake3(&fact.value_digest_blake3, ValidationCode(code))?;
    }
    assert!(facts.len() <= limit);
    assert!(facts.windows(2).all(|pair| pair[0] < pair[1]));
    Ok(())
}

fn validate_result_input(input: &RustUnitResultInput) -> Result<(), String> {
    validate_typed_ref(&input.action_ref, TypedRefRule {
        prefix: RUST_ACTION_REF_PREFIX,
        code: ValidationCode("rust-result-action-ref-invalid"),
    })?;
    validate_castore_node(&input.root_node)?;
    if input.artifacts.is_empty() || input.artifacts.len() > MAX_RESULT_ARTIFACTS {
        return Err("rust-result-artifact-count-invalid".to_string());
    }
    if input.artifacts != sorted_unique(input.artifacts.clone()) {
        return Err("rust-result-artifacts-not-canonical".to_string());
    }
    let mut total_bytes = 0_u64;
    for artifact in &input.artifacts {
        validate_result_artifact(artifact)?;
        total_bytes = total_bytes
            .checked_add(artifact.size_bytes)
            .ok_or_else(|| "rust-result-artifact-bytes-overflow".to_string())?;
    }
    if total_bytes > MAX_TREE_BYTES {
        return Err("rust-result-artifact-bytes-limit-exceeded".to_string());
    }
    validate_typed_ref(&input.producer_receipt_ref, TypedRefRule {
        prefix: "mantle-rust-receipt://blake3/",
        code: ValidationCode("rust-result-receipt-ref-invalid"),
    })?;
    assert!(!input.artifacts.is_empty());
    assert!(total_bytes <= MAX_TREE_BYTES);
    Ok(())
}

fn validate_castore_node(node: &CastoreNodeIdentity) -> Result<(), String> {
    validate_blake3(&node.digest_blake3, ValidationCode("rust-result-root-digest-invalid"))?;
    if node.size_bytes > MAX_TREE_BYTES {
        return Err("rust-result-root-size-limit-exceeded".to_string());
    }
    assert_eq!(node.digest_blake3.len(), BLAKE3_HEX_CHARS);
    assert!(node.size_bytes <= MAX_TREE_BYTES);
    Ok(())
}

fn validate_result_artifact(artifact: &RustResultArtifact) -> Result<(), String> {
    validate_relative_path(&artifact.relative_path)?;
    validate_blake3(&artifact.digest_blake3, ValidationCode("rust-result-artifact-digest-invalid"))?;
    if artifact.size_bytes > MAX_TREE_BYTES {
        return Err("rust-result-artifact-size-limit-exceeded".to_string());
    }
    match artifact.kind {
        RustArtifactKind::File if !matches!(artifact.mode, FILE_MODE_REGULAR | FILE_MODE_EXECUTABLE) => {
            return Err("rust-result-artifact-mode-invalid".to_string());
        }
        RustArtifactKind::Symlink if artifact.mode != 0 => {
            return Err("rust-result-symlink-mode-invalid".to_string());
        }
        _ => {}
    }
    assert!(!artifact.relative_path.is_empty());
    assert!(artifact.size_bytes <= MAX_TREE_BYTES);
    Ok(())
}

fn validate_relative_path(path: &str) -> Result<(), String> {
    if path.is_empty() {
        return Err("rust-result-artifact-path-invalid".to_string());
    }
    if path.len() > MAX_RELATIVE_PATH_BYTES {
        return Err("rust-result-artifact-path-invalid".to_string());
    }
    if path.starts_with('/') {
        return Err("rust-result-artifact-path-invalid".to_string());
    }
    if path.contains('\\') {
        return Err("rust-result-artifact-path-invalid".to_string());
    }
    if path.split('/').any(|component| component.is_empty() || matches!(component, "." | "..")) {
        return Err("rust-result-artifact-path-invalid".to_string());
    }
    assert!(!path.starts_with('/'));
    assert!(!path.contains(".."));
    Ok(())
}

fn validate_result_refs(result_refs: &[String]) -> Result<(), String> {
    if result_refs.is_empty() || result_refs.len() > MAX_RESULT_CANDIDATES {
        return Err("rust-result-index-count-invalid".to_string());
    }
    for result_ref in result_refs {
        validate_typed_ref(result_ref, TypedRefRule {
            prefix: RUST_RESULT_REF_PREFIX,
            code: ValidationCode("rust-result-index-ref-invalid"),
        })?;
    }
    assert!(!result_refs.is_empty());
    assert!(result_refs.len() <= MAX_RESULT_CANDIDATES);
    Ok(())
}

fn unique_candidates(candidates: Vec<LocalCandidateFacts>) -> BTreeMap<String, LocalCandidateFacts> {
    let mut unique = BTreeMap::new();
    for candidate in candidates {
        unique.entry(candidate.result.result_ref.clone()).or_insert(candidate);
    }
    assert!(unique.len() <= MAX_RESULT_CANDIDATES);
    assert!(unique.keys().all(|result_ref| !result_ref.is_empty()));
    unique
}

fn evaluate_local_candidate(
    action_ref: &str,
    policy: &LocalCachePolicy,
    candidate: LocalCandidateFacts,
) -> LocalCandidateDecision {
    let result_ref = candidate.result.result_ref.clone();
    let mut reason_codes = Vec::new();
    push_error(&mut reason_codes, validate_rust_result(&candidate.result));
    if !policy.reads_enabled {
        reason_codes.push("rust-local-cache-read-disabled".to_string());
    }
    if candidate.result.input.action_ref != action_ref {
        reason_codes.push("rust-local-cache-action-mismatch".to_string());
    }
    if !candidate.content_complete {
        reason_codes.push("rust-local-cache-content-incomplete".to_string());
    }
    if !candidate.artifact_manifest_verified {
        reason_codes.push("rust-local-cache-artifact-verification-failed".to_string());
    }
    reason_codes.sort();
    reason_codes.dedup();
    let artifact_set_digest_blake3 = artifact_set_digest(&candidate.result).ok();
    let is_admitted = reason_codes.is_empty();
    assert_eq!(is_admitted, reason_codes.is_empty());
    assert!(reason_codes.windows(2).all(|pair| pair[0] < pair[1]));
    LocalCandidateDecision {
        result_ref,
        admitted: is_admitted,
        reason_codes,
        artifact_set_digest_blake3,
    }
}

fn finish_local_reuse_plan(
    mut decisions: Vec<LocalCandidateDecision>,
    admitted: BTreeMap<String, Vec<String>>,
) -> Result<LocalReusePlan, String> {
    decisions.sort_by(|left, right| left.result_ref.cmp(&right.result_ref));
    let is_conflict = admitted.len() > 1;
    let selected_result_ref = if is_conflict {
        None
    } else {
        admitted.values().next().and_then(|refs| refs.iter().min().cloned())
    };
    assert!(decisions.len() <= MAX_RESULT_CANDIDATES);
    assert!(admitted.len() <= decisions.len());
    Ok(LocalReusePlan {
        selected_result_ref,
        conflict_class: is_conflict.then(|| LOCAL_RESULT_CONFLICT.to_string()),
        decisions,
    })
}

fn artifact_set_digest(result: &RustUnitResult) -> Result<String, String> {
    let bytes =
        serde_json::to_vec(&result.input.artifacts).map_err(|error| format!("rust-artifact-set-json:{error}"))?;
    let digest = domain_digest(ARTIFACT_SET_DOMAIN, &bytes);
    assert_eq!(digest.len(), BLAKE3_HEX_CHARS);
    assert!(!result.input.artifacts.is_empty());
    Ok(digest)
}

fn validate_sorted_strings(values: &[String], limit: usize, code: &str) -> Result<(), String> {
    if values.len() > limit || values != sorted_unique(values.to_vec()) {
        return Err(code.to_string());
    }
    for value in values {
        validate_string(value, ValidationCode(code))?;
    }
    assert!(values.len() <= limit);
    assert!(values.windows(2).all(|pair| pair[0] < pair[1]));
    Ok(())
}

fn validate_identifier(value: &str, code: ValidationCode<'_>) -> Result<(), String> {
    validate_string(value, code)?;
    if value.bytes().any(|byte| byte.is_ascii_control()) {
        return Err(code.0.to_string());
    }
    assert!(!value.is_empty());
    assert!(value.len() <= MAX_STRING_BYTES);
    Ok(())
}

fn validate_string(value: &str, code: ValidationCode<'_>) -> Result<(), String> {
    if value.is_empty() || value.len() > MAX_STRING_BYTES {
        return Err(code.0.to_string());
    }
    assert!(!value.is_empty());
    assert!(value.len() <= MAX_STRING_BYTES);
    Ok(())
}

fn validate_blake3(value: &str, code: ValidationCode<'_>) -> Result<(), String> {
    let is_valid = value.len() == BLAKE3_HEX_CHARS
        && value.bytes().all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase());
    if !is_valid {
        return Err(code.0.to_string());
    }
    assert_eq!(value.len(), BLAKE3_HEX_CHARS);
    assert!(value.bytes().all(|byte| !byte.is_ascii_uppercase()));
    Ok(())
}

fn validate_typed_ref(value: &str, rule: TypedRefRule<'_>) -> Result<(), String> {
    let Some(digest) = value.strip_prefix(rule.prefix) else {
        return Err(rule.code.0.to_string());
    };
    validate_blake3(digest, rule.code)?;
    assert!(value.starts_with(rule.prefix));
    assert_eq!(digest.len(), BLAKE3_HEX_CHARS);
    Ok(())
}

fn ensure_record_bound(value: &impl Serialize, code: &str) -> Result<(), String> {
    let bytes = serde_json::to_vec(value).map_err(|error| format!("rust-cache-json:{error}"))?;
    if bytes.len() > MAX_RECORD_BYTES {
        return Err(code.to_string());
    }
    assert!(!bytes.is_empty());
    assert!(bytes.len() <= MAX_RECORD_BYTES);
    Ok(())
}

fn digest_ref(prefix: &str, domain: &[u8], value: &impl Serialize) -> Result<String, String> {
    let bytes = serde_json::to_vec(value).map_err(|error| format!("rust-cache-canonical-json:{error}"))?;
    let digest = domain_digest(domain, &bytes);
    let reference = format!("{prefix}{digest}");
    assert!(reference.starts_with(prefix));
    assert_eq!(digest.len(), BLAKE3_HEX_CHARS);
    Ok(reference)
}

fn domain_digest(domain: &[u8], bytes: &[u8]) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(domain);
    hasher.update(&[DOMAIN_SEPARATOR]);
    hasher.update(bytes);
    let digest = hasher.finalize().to_hex().to_string();
    assert_eq!(digest.len(), BLAKE3_HEX_CHARS);
    assert!(digest.bytes().all(|byte| !byte.is_ascii_uppercase()));
    digest
}

fn sorted_unique<T: Ord>(values: Vec<T>) -> Vec<T> {
    let set = values.into_iter().collect::<BTreeSet<_>>();
    let values = set.into_iter().collect::<Vec<_>>();
    assert!(values.windows(2).all(|pair| pair[0] < pair[1]));
    assert!(values.len() <= values.capacity());
    values
}

fn push_error(errors: &mut Vec<String>, result: Result<(), String>) {
    if let Err(error) = result {
        errors.push(error);
    }
    assert!(errors.len() <= MAX_RESULT_CANDIDATES);
    assert!(errors.iter().all(|error| !error.is_empty()));
}

#[derive(Serialize)]
struct RustIndexHashable<'a> {
    schema: &'a str,
    action_ref: &'a str,
    result_refs: &'a [String],
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    const DIGEST_A: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    const DIGEST_B: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
    const TEST_ARTIFACT_BYTES: u64 = 10;
    const DECLARED_ACTION_INPUT_CLASSES: usize = 22;
    const HEX_PAIR_BYTES: usize = 2;

    fn argument(value: &str) -> RustSemanticArgument {
        RustSemanticArgument {
            value: value.to_string(),
            contains_absolute_path: false,
            absolute_paths_classified: false,
        }
    }

    fn artifact(role: &str, name: &str, digest: &str) -> RustArtifactIdentity {
        RustArtifactIdentity {
            role: role.to_string(),
            name: name.to_string(),
            digest_blake3: digest.to_string(),
        }
    }

    fn action_input() -> RustUnitActionInput {
        RustUnitActionInput {
            unit_id: "unit-1".to_string(),
            package_id: "package-1".to_string(),
            crate_name: "crate_name".to_string(),
            target_kind: "lib".to_string(),
            execution_kind: "target".to_string(),
            host_triple: "x86_64-unknown-linux-gnu".to_string(),
            target_triple: "x86_64-unknown-linux-musl".to_string(),
            profile: "release".to_string(),
            mode: "build".to_string(),
            features: vec!["std".to_string(), "serde".to_string()],
            source_digest_blake3: DIGEST_A.to_string(),
            compiler_digest_blake3: DIGEST_A.to_string(),
            compiler_version_digest_blake3: DIGEST_A.to_string(),
            toolchain_closure_digest_blake3: DIGEST_A.to_string(),
            execution_platform_digest_blake3: DIGEST_A.to_string(),
            semantic_arguments: vec![argument("--crate-type=lib")],
            admitted_environment: BTreeMap::from([("CARGO_PKG_NAME".to_string(), "package-1".to_string())]),
            dependency_artifacts: vec![artifact("dependency", "serde", DIGEST_A)],
            host_artifacts: vec![artifact("proc-macro", "serde_derive", DIGEST_A)],
            build_script_facts: vec![RustBuildFact {
                name: "out-dir".to_string(),
                value_digest_blake3: DIGEST_A.to_string(),
            }],
            native_link_facts: vec![RustBuildFact {
                name: "link-lib".to_string(),
                value_digest_blake3: DIGEST_A.to_string(),
            }],
            compiler_policy_digest_blake3: DIGEST_A.to_string(),
        }
    }

    fn result_input(action_ref: &str, digest: &str) -> RustUnitResultInput {
        RustUnitResultInput {
            action_ref: action_ref.to_string(),
            root_node: CastoreNodeIdentity {
                kind: CastoreNodeKind::Directory,
                digest_blake3: digest.to_string(),
                size_bytes: TEST_ARTIFACT_BYTES,
            },
            artifacts: vec![RustResultArtifact {
                relative_path: "libcrate.rlib".to_string(),
                kind: RustArtifactKind::File,
                mode: FILE_MODE_REGULAR,
                size_bytes: TEST_ARTIFACT_BYTES,
                digest_blake3: digest.to_string(),
            }],
            producer_receipt_ref: format!("mantle-rust-receipt://blake3/{DIGEST_A}"),
        }
    }

    #[test]
    fn action_identity_is_canonical_and_excludes_output_root() {
        let mut reordered = action_input();
        reordered.features.reverse();
        let first = canonical_rust_action(action_input()).unwrap();
        let second = canonical_rust_action(reordered).unwrap();

        assert_eq!(first, second);
        assert_eq!(first.action_ref.len(), RUST_ACTION_REF_PREFIX.len() + BLAKE3_HEX_CHARS);
        assert!(!serde_json::to_string(&first).unwrap().contains("execution-output-root"));
    }

    #[test]
    fn every_declared_action_input_invalidates_identity() {
        let baseline = canonical_rust_action(action_input()).unwrap().action_ref;
        let mut changed = Vec::new();
        macro_rules! changed {
            ($field:ident, $value:expr) => {{
                let mut input = action_input();
                input.$field = $value;
                changed.push(canonical_rust_action(input).unwrap().action_ref);
            }};
        }
        changed!(unit_id, "unit-2".to_string());
        changed!(package_id, "package-2".to_string());
        changed!(crate_name, "other_crate".to_string());
        changed!(target_kind, "bin".to_string());
        changed!(execution_kind, "host".to_string());
        changed!(host_triple, "aarch64-unknown-linux-gnu".to_string());
        changed!(target_triple, "aarch64-unknown-linux-musl".to_string());
        changed!(profile, "debug".to_string());
        changed!(mode, "test".to_string());
        changed!(features, vec!["std".to_string()]);
        changed!(source_digest_blake3, DIGEST_B.to_string());
        changed!(compiler_digest_blake3, DIGEST_B.to_string());
        changed!(compiler_version_digest_blake3, DIGEST_B.to_string());
        changed!(toolchain_closure_digest_blake3, DIGEST_B.to_string());
        changed!(execution_platform_digest_blake3, DIGEST_B.to_string());
        changed!(semantic_arguments, vec![argument("--crate-type=bin")]);
        changed!(admitted_environment, BTreeMap::from([("CARGO_PKG_NAME".to_string(), "package-2".to_string())]));
        changed!(dependency_artifacts, vec![artifact("dependency", "serde", DIGEST_B)]);
        changed!(host_artifacts, vec![artifact("proc-macro", "serde_derive", DIGEST_B)]);
        changed!(build_script_facts, vec![RustBuildFact {
            name: "out-dir".to_string(),
            value_digest_blake3: DIGEST_B.to_string()
        }]);
        changed!(native_link_facts, vec![RustBuildFact {
            name: "link-lib".to_string(),
            value_digest_blake3: DIGEST_B.to_string()
        }]);
        changed!(compiler_policy_digest_blake3, DIGEST_B.to_string());

        assert_eq!(changed.len(), DECLARED_ACTION_INPUT_CLASSES);
        assert!(changed.iter().all(|candidate| candidate != &baseline));
    }

    #[test]
    fn action_rejects_unclassified_absolute_path_and_bad_digest() {
        let mut absolute = action_input();
        absolute.semantic_arguments = vec![RustSemanticArgument {
            value: "--sysroot=/secret/path".to_string(),
            contains_absolute_path: true,
            absolute_paths_classified: false,
        }];
        let mut bad_digest = action_input();
        bad_digest.compiler_digest_blake3 = "AA".repeat(BLAKE3_HEX_CHARS / HEX_PAIR_BYTES);

        assert_eq!(canonical_rust_action(absolute).unwrap_err(), "rust-action-unclassified-absolute-path");
        assert_eq!(canonical_rust_action(bad_digest).unwrap_err(), "rust-action-compiler-digest-invalid");
    }

    #[test]
    fn result_and_index_are_canonical() {
        let action = canonical_rust_action(action_input()).unwrap();
        let result = canonical_rust_result(result_input(&action.action_ref, DIGEST_A)).unwrap();
        let index = canonical_result_index(action.action_ref.clone(), vec![result.result_ref.clone()]).unwrap();

        assert_eq!(index.action_ref, action.action_ref);
        assert_eq!(index.result_refs, vec![result.result_ref]);
        assert!(validate_result_index(&index).is_ok());
    }

    #[test]
    fn result_rejects_path_escape_and_tampered_reference() {
        let action = canonical_rust_action(action_input()).unwrap();
        let mut escape = result_input(&action.action_ref, DIGEST_A);
        escape.artifacts[0].relative_path = "../escape".to_string();
        let mut result = canonical_rust_result(result_input(&action.action_ref, DIGEST_A)).unwrap();
        result.result_ref = format!("{RUST_RESULT_REF_PREFIX}{DIGEST_B}");

        assert_eq!(canonical_rust_result(escape).unwrap_err(), "rust-result-artifact-path-invalid");
        assert_eq!(validate_rust_result(&result).unwrap_err(), "rust-result-ref-mismatch");
    }

    #[test]
    fn local_reuse_admits_one_complete_verified_result() {
        let action = canonical_rust_action(action_input()).unwrap();
        let result = canonical_rust_result(result_input(&action.action_ref, DIGEST_A)).unwrap();
        let policy = LocalCachePolicy {
            reads_enabled: true,
            ..LocalCachePolicy::default()
        };
        let plan = plan_local_reuse(&action.action_ref, &policy, vec![LocalCandidateFacts {
            result: result.clone(),
            content_complete: true,
            artifact_manifest_verified: true,
        }])
        .unwrap();

        assert_eq!(plan.selected_result_ref, Some(result.result_ref));
        assert!(plan.conflict_class.is_none());
        assert!(plan.decisions[0].admitted);
    }

    #[test]
    fn local_reuse_rejects_incomplete_and_mismatched_candidates() {
        let action = canonical_rust_action(action_input()).unwrap();
        let other = canonical_rust_action({
            let mut input = action_input();
            input.source_digest_blake3 = DIGEST_B.to_string();
            input
        })
        .unwrap();
        let result = canonical_rust_result(result_input(&other.action_ref, DIGEST_A)).unwrap();
        let policy = LocalCachePolicy {
            reads_enabled: true,
            ..LocalCachePolicy::default()
        };
        let plan = plan_local_reuse(&action.action_ref, &policy, vec![LocalCandidateFacts {
            result,
            content_complete: false,
            artifact_manifest_verified: false,
        }])
        .unwrap();

        assert!(plan.selected_result_ref.is_none());
        assert_eq!(plan.decisions[0].reason_codes, vec![
            "rust-local-cache-action-mismatch".to_string(),
            "rust-local-cache-artifact-verification-failed".to_string(),
            "rust-local-cache-content-incomplete".to_string(),
        ]);
    }

    #[test]
    fn local_reuse_reports_conflicting_artifact_sets() {
        let action = canonical_rust_action(action_input()).unwrap();
        let first = canonical_rust_result(result_input(&action.action_ref, DIGEST_A)).unwrap();
        let second = canonical_rust_result(result_input(&action.action_ref, DIGEST_B)).unwrap();
        let policy = LocalCachePolicy {
            reads_enabled: true,
            ..LocalCachePolicy::default()
        };
        let plan = plan_local_reuse(&action.action_ref, &policy, vec![
            LocalCandidateFacts {
                result: first,
                content_complete: true,
                artifact_manifest_verified: true,
            },
            LocalCandidateFacts {
                result: second,
                content_complete: true,
                artifact_manifest_verified: true,
            },
        ])
        .unwrap();

        assert!(plan.selected_result_ref.is_none());
        assert_eq!(plan.conflict_class.as_deref(), Some(LOCAL_RESULT_CONFLICT));
        assert!(plan.decisions.iter().all(|decision| decision.admitted));
    }

    #[test]
    fn policy_rejects_invalid_bounds() {
        let policy = LocalCachePolicy {
            max_candidates: 0,
            ..LocalCachePolicy::default()
        };
        let oversized = LocalCachePolicy {
            max_tree_bytes: MAX_TREE_BYTES.checked_add(1).unwrap(),
            ..LocalCachePolicy::default()
        };

        assert_eq!(validate_local_cache_policy(&policy).unwrap_err(), "rust-cache-policy-candidate-limit-invalid");
        assert_eq!(validate_local_cache_policy(&oversized).unwrap_err(), "rust-cache-policy-tree-byte-limit-invalid");
    }
}
