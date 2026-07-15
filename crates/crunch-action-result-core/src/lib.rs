#![cfg_attr(not(kani), feature(register_tool))]
#![register_tool(tigerstyle)]
//! Pure bounded identity and admission planning for shared Mantle action results.
//!
//! This crate has no filesystem, network, clock, environment, CAS, or executor
//! access. Shells supply verified facts; the core canonicalizes immutable
//! records and decides whether strong reuse is admissible.

use std::collections::BTreeMap;
use std::collections::BTreeSet;

use serde::Deserialize;
use serde::Serialize;

// r[impl build_correctness.shared_action_result_records]
pub const ACTION_RESULT_SCHEMA: &str = "mantle-action-result-v1";
pub const ACTION_RESULT_INDEX_SCHEMA: &str = "mantle-action-result-index-v1";
pub const ACTION_RESULT_POLICY_SCHEMA: &str = "mantle-action-result-policy-v1";
pub const ACTION_REF_PREFIX: &str = "mantle-action://blake3/";
pub const ACTION_RESULT_REF_PREFIX: &str = "mantle-action-result://blake3/";
pub const ACTION_RESULT_INDEX_REF_PREFIX: &str = "mantle-action-result-index://blake3/";
pub const OBJECT_REF_PREFIX: &str = "mantle-object://blake3/";
pub const PATH_INFO_REF_PREFIX: &str = "mantle-pathinfo://blake3/";
pub const ACTION_RECEIPT_REF_PREFIX: &str = "mantle-action-receipt://blake3/";
pub const REFERENCE_SCAN_REF_PREFIX: &str = "mantle-reference-scan://blake3/";
pub const SANDBOX_POLICY_REF_PREFIX: &str = "mantle-sandbox-policy://blake3/";
pub const NETWORK_POLICY_REF_PREFIX: &str = "mantle-network-policy://blake3/";
pub const PRODUCER_POLICY_REF_PREFIX: &str = "mantle-producer-policy://blake3/";
pub const PUBLICATION_POLICY_REF_PREFIX: &str = "mantle-publication-policy://blake3/";
pub const SIGNATURE_REF_PREFIX: &str = "mantle-signature://blake3/";
pub const CONFLICTING_ACTION_RESULTS: &str = "conflicting-action-results";
pub const STRONG_CLAIM: &str = "strong";
const DEFAULT_CONFIGURED_SOURCE_CLASS: &str = "configured-source";
pub const MAX_ACTION_RESULT_OUTPUTS: usize = 64;
pub const MAX_ACTION_RESULT_METADATA_REFS: usize = 256;
pub const MAX_ACTION_RESULT_SIGNATURE_REFS: usize = 64;
pub const MAX_ACTION_RESULT_NON_CLAIMS: usize = 64;
pub const MAX_ACTION_RESULT_CANDIDATES: usize = 256;
pub const MAX_ACTION_RESULT_RECORD_BYTES: usize = 1_048_576;
pub const MAX_ACTION_RESULT_INDEX_BYTES: usize = 262_144;
pub const MAX_ACTION_RESULT_PATH_BYTES: usize = 4_096;
pub const MAX_ACTION_RESULT_ID_BYTES: usize = 512;
pub const MAX_ACTION_RESULT_DIAGNOSTICS: usize = 1_024;

pub const BLAKE3_HEX_CHARS: usize = 64;
const RECORD_DOMAIN: &[u8] = b"mantle.action-result.record.v1";
const INDEX_DOMAIN: &[u8] = b"mantle.action-result.index.v1";
const OUTPUT_SET_DOMAIN: &[u8] = b"mantle.action-result.output-set.v1";
const POLICY_DOMAIN: &[u8] = b"mantle.action-result.policy.v1";
const DOMAIN_SEPARATOR: u8 = 0;
const REQUIRED_NON_CLAIM_INDEX_NOT_TRUST: &str = "index-presence-is-not-output-trust";
const REQUIRED_NON_CLAIM_CA_MAPPING_NOT_TRUST: &str = "ca-mapping-presence-is-not-output-trust";
const REQUIRED_NON_CLAIM_EXECUTOR_CORRECTNESS: &str = "executor-correctness";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(deny_unknown_fields)]
pub struct ActionResultOutput {
    pub name: String,
    pub object_ref: String,
    pub store_path: String,
    pub path_info_ref: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ActionResultRecordInput {
    pub action_ref: String,
    pub outputs: Vec<ActionResultOutput>,
    pub action_receipt_ref: String,
    pub reference_scan_refs: Vec<String>,
    pub sandbox_policy_ref: String,
    pub network_policy_ref: String,
    pub producer_identity: String,
    pub producer_policy_ref: String,
    pub signature_refs: Vec<String>,
    pub publication_policy_ref: String,
    pub non_claims: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ActionResultRecord {
    pub schema: String,
    pub result_ref: String,
    pub action_ref: String,
    pub outputs: Vec<ActionResultOutput>,
    pub action_receipt_ref: String,
    pub reference_scan_refs: Vec<String>,
    pub sandbox_policy_ref: String,
    pub network_policy_ref: String,
    pub producer_identity: String,
    pub producer_policy_ref: String,
    pub signature_refs: Vec<String>,
    pub publication_policy_ref: String,
    pub non_claims: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(deny_unknown_fields)]
pub struct DetachedRecordSignature {
    pub key_name: String,
    pub signature: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct SignedActionResultRecord {
    pub record: ActionResultRecord,
    pub record_signatures: Vec<DetachedRecordSignature>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ActionResultIndex {
    pub schema: String,
    pub index_ref: String,
    pub action_ref: String,
    pub result_refs: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ActionResultTrustPolicy {
    pub schema: String,
    pub policy_id: String,
    pub trusted_producers: Vec<String>,
    pub trusted_record_signers: Vec<String>,
    pub allowed_source_classes: Vec<String>,
    pub allowed_sandbox_policy_refs: Vec<String>,
    pub allowed_network_policy_refs: Vec<String>,
    pub required_non_claims: Vec<String>,
    pub require_record_signature: bool,
    pub require_path_info_signature: bool,
}

impl Default for ActionResultTrustPolicy {
    fn default() -> Self {
        Self {
            schema: ACTION_RESULT_POLICY_SCHEMA.to_string(),
            policy_id: "mantle-action-result-default-v1".to_string(),
            trusted_producers: Vec::new(),
            trusted_record_signers: Vec::new(),
            allowed_source_classes: vec![DEFAULT_CONFIGURED_SOURCE_CLASS.to_string()],
            allowed_sandbox_policy_refs: Vec::new(),
            allowed_network_policy_refs: Vec::new(),
            required_non_claims: required_non_claims(),
            require_record_signature: true,
            require_path_info_signature: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct CandidateAdmissionFacts {
    pub source_id: String,
    pub source_class: String,
    pub verified_record_signers: Vec<String>,
    pub action_receipt_linked: bool,
    pub object_refs_complete: bool,
    pub path_info_refs_linked: bool,
    pub path_info_signatures_verified: bool,
    pub producer_policy_admitted: bool,
    pub publication_policy_admitted: bool,
    pub sandbox_policy_admitted: bool,
    pub network_policy_admitted: bool,
    pub reference_scans_admitted: bool,
    pub claim_strength: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct DiscoveredActionResultCandidate {
    pub signed_record: SignedActionResultRecord,
    pub facts: CandidateAdmissionFacts,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct StrongReuseRequest {
    pub action_ref: String,
    pub output_names: Vec<String>,
    pub policy: ActionResultTrustPolicy,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct CandidateDecision {
    pub result_ref: String,
    pub source_id: String,
    pub source_class: String,
    pub admitted: bool,
    pub diagnostics: Vec<String>,
    pub trust_basis: Vec<String>,
    pub output_set_digest_blake3: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct StrongReusePlan {
    pub selected_result_ref: Option<String>,
    pub conflict_class: Option<String>,
    pub admitted_output_set_digests_blake3: Vec<String>,
    pub candidate_decisions: Vec<CandidateDecision>,
    pub non_claims: Vec<String>,
}

pub fn canonical_action_result(input: ActionResultRecordInput) -> Result<ActionResultRecord, String> {
    let normalized = normalize_record_input(input);
    assert!(is_strictly_sorted(&normalized.outputs), "normalized outputs must be sorted and unique");
    assert!(is_strictly_sorted(&normalized.non_claims), "normalized non-claims must be sorted and unique");
    validate_record_input(&normalized)?;
    let result_ref = digest_ref(ACTION_RESULT_REF_PREFIX, RECORD_DOMAIN, &normalized)?;
    let record = ActionResultRecord {
        schema: ACTION_RESULT_SCHEMA.to_string(),
        result_ref,
        action_ref: normalized.action_ref,
        outputs: normalized.outputs,
        action_receipt_ref: normalized.action_receipt_ref,
        reference_scan_refs: normalized.reference_scan_refs,
        sandbox_policy_ref: normalized.sandbox_policy_ref,
        network_policy_ref: normalized.network_policy_ref,
        producer_identity: normalized.producer_identity,
        producer_policy_ref: normalized.producer_policy_ref,
        signature_refs: normalized.signature_refs,
        publication_policy_ref: normalized.publication_policy_ref,
        non_claims: normalized.non_claims,
    };
    validate_action_result(&record)?;
    Ok(record)
}

pub fn validate_action_result(record: &ActionResultRecord) -> Result<(), String> {
    if record.schema != ACTION_RESULT_SCHEMA {
        return Err("action-result-schema-unsupported".to_string());
    }
    let input = record_input(record);
    validate_record_input(&input)?;
    let expected_ref = digest_ref(ACTION_RESULT_REF_PREFIX, RECORD_DOMAIN, &input)?;
    if record.result_ref != expected_ref {
        return Err("action-result-ref-mismatch".to_string());
    }
    let bytes = canonical_record_bytes(record)?;
    if bytes.len() > MAX_ACTION_RESULT_RECORD_BYTES {
        return Err("action-result-record-too-large".to_string());
    }
    Ok(())
}

pub fn canonical_action_result_index(
    action_ref: String,
    result_refs: Vec<String>,
) -> Result<ActionResultIndex, String> {
    validate_typed_ref(TypedRefValidation {
        field: "action-ref",
        value: &action_ref,
        prefix: ACTION_REF_PREFIX,
    })?;
    let result_refs = sorted_unique_strings(result_refs);
    validate_index_result_refs(&result_refs)?;
    assert!(result_refs.len() <= MAX_ACTION_RESULT_CANDIDATES, "canonical result refs must remain bounded");
    assert!(is_strictly_sorted(&result_refs), "canonical result refs must be sorted and unique");
    let hashable = IndexHashable {
        schema: ACTION_RESULT_INDEX_SCHEMA,
        action_ref: &action_ref,
        result_refs: &result_refs,
    };
    let index_ref = digest_ref(ACTION_RESULT_INDEX_REF_PREFIX, INDEX_DOMAIN, &hashable)?;
    let index = ActionResultIndex {
        schema: ACTION_RESULT_INDEX_SCHEMA.to_string(),
        index_ref,
        action_ref,
        result_refs,
    };
    validate_action_result_index(&index)?;
    Ok(index)
}

pub fn validate_action_result_index(index: &ActionResultIndex) -> Result<(), String> {
    if index.schema != ACTION_RESULT_INDEX_SCHEMA {
        return Err("action-result-index-schema-unsupported".to_string());
    }
    assert_eq!(index.schema, ACTION_RESULT_INDEX_SCHEMA, "validated index schema must match");
    validate_typed_ref(TypedRefValidation {
        field: "action-ref",
        value: &index.action_ref,
        prefix: ACTION_REF_PREFIX,
    })?;
    validate_index_result_refs(&index.result_refs)?;
    if index.result_refs != sorted_unique_strings(index.result_refs.clone()) {
        return Err("action-result-index-not-canonical".to_string());
    }
    assert!(is_strictly_sorted(&index.result_refs), "validated result refs must be sorted and unique");
    let hashable = IndexHashable {
        schema: ACTION_RESULT_INDEX_SCHEMA,
        action_ref: &index.action_ref,
        result_refs: &index.result_refs,
    };
    let expected_ref = digest_ref(ACTION_RESULT_INDEX_REF_PREFIX, INDEX_DOMAIN, &hashable)?;
    if index.index_ref != expected_ref {
        return Err("action-result-index-ref-mismatch".to_string());
    }
    let bytes = canonical_index_bytes(index)?;
    if bytes.len() > MAX_ACTION_RESULT_INDEX_BYTES {
        return Err("action-result-index-too-large".to_string());
    }
    Ok(())
}

pub fn canonical_record_bytes(record: &ActionResultRecord) -> Result<Vec<u8>, String> {
    serde_json::to_vec(record).map_err(|error| format!("action-result-record-json:{error}"))
}

pub fn canonical_signed_record_bytes(record: &SignedActionResultRecord) -> Result<Vec<u8>, String> {
    let normalized = SignedActionResultRecord {
        record: record.record.clone(),
        record_signatures: sorted_unique_structs(record.record_signatures.clone()),
    };
    serde_json::to_vec(&normalized).map_err(|error| format!("action-result-signed-record-json:{error}"))
}

pub fn canonical_index_bytes(index: &ActionResultIndex) -> Result<Vec<u8>, String> {
    serde_json::to_vec(index).map_err(|error| format!("action-result-index-json:{error}"))
}

pub fn policy_digest_blake3(policy: &ActionResultTrustPolicy) -> Result<String, String> {
    validate_policy(policy)?;
    let normalized = normalize_policy(policy.clone());
    let bytes = serde_json::to_vec(&normalized).map_err(|error| format!("action-result-policy-json:{error}"))?;
    Ok(domain_digest(POLICY_DOMAIN, &bytes))
}

pub fn plan_strong_reuse(
    request: StrongReuseRequest,
    candidates: Vec<DiscoveredActionResultCandidate>,
) -> Result<StrongReusePlan, String> {
    validate_request(&request, candidates.len())?;
    let mut unique = BTreeMap::new();
    for candidate in candidates {
        let result_ref = candidate.signed_record.record.result_ref.clone();
        unique.entry(result_ref).or_insert(candidate);
    }
    let mut decisions = Vec::with_capacity(unique.len());
    let mut admitted_by_output = BTreeMap::<String, Vec<String>>::new();
    for candidate in unique.into_values() {
        let decision = evaluate_candidate(&request, &candidate);
        if let Some(digest) = admitted_output_digest(&decision)? {
            admitted_by_output.entry(digest).or_default().push(decision.result_ref.clone());
        }
        decisions.push(decision);
    }
    finish_plan(decisions, admitted_by_output)
}

fn admitted_output_digest(decision: &CandidateDecision) -> Result<Option<String>, String> {
    if !decision.admitted {
        return Ok(None);
    }
    let Some(digest) = decision.output_set_digest_blake3.clone() else {
        return Err("action-result-admitted-candidate-digest-missing".to_string());
    };
    Ok(Some(digest))
}

fn finish_plan(
    mut decisions: Vec<CandidateDecision>,
    admitted_by_output: BTreeMap<String, Vec<String>>,
) -> Result<StrongReusePlan, String> {
    decisions.sort_by(|left, right| left.result_ref.cmp(&right.result_ref).then(left.source_id.cmp(&right.source_id)));
    if decisions.len() > MAX_ACTION_RESULT_DIAGNOSTICS {
        return Err("action-result-diagnostic-limit-exceeded".to_string());
    }
    assert!(decisions.len() <= MAX_ACTION_RESULT_DIAGNOSTICS, "validated decision count must remain bounded");
    assert!(
        admitted_by_output.len() <= decisions.len(),
        "admitted output groups cannot exceed candidate decisions"
    );
    let admitted_output_set_digests_blake3 = admitted_by_output.keys().cloned().collect::<Vec<_>>();
    let has_conflict = admitted_by_output.len() > 1;
    let selected_result_ref = if has_conflict {
        None
    } else {
        admitted_by_output.values().next().and_then(|refs| refs.iter().min().cloned())
    };
    Ok(StrongReusePlan {
        selected_result_ref,
        conflict_class: has_conflict.then(|| CONFLICTING_ACTION_RESULTS.to_string()),
        admitted_output_set_digests_blake3,
        candidate_decisions: decisions,
        non_claims: required_non_claims(),
    })
}

fn evaluate_candidate(request: &StrongReuseRequest, candidate: &DiscoveredActionResultCandidate) -> CandidateDecision {
    let record = &candidate.signed_record.record;
    let facts = &candidate.facts;
    let mut diagnostics = Vec::new();
    push_error(&mut diagnostics, validate_action_result(record));
    validate_candidate_shape(request, record, facts, &mut diagnostics);
    validate_candidate_trust(request, &candidate.signed_record, facts, &mut diagnostics);
    validate_candidate_admission_facts(facts, &mut diagnostics);
    let output_set_digest_blake3 = output_set_digest(record).ok();
    CandidateDecision {
        result_ref: record.result_ref.clone(),
        source_id: facts.source_id.clone(),
        source_class: facts.source_class.clone(),
        admitted: diagnostics.is_empty(),
        diagnostics: sorted_unique_strings(diagnostics),
        trust_basis: sorted_unique_strings(facts.verified_record_signers.clone()),
        output_set_digest_blake3,
    }
}

fn validate_candidate_shape(
    request: &StrongReuseRequest,
    record: &ActionResultRecord,
    facts: &CandidateAdmissionFacts,
    diagnostics: &mut Vec<String>,
) {
    if record.action_ref != request.action_ref {
        diagnostics.push("stale-action-ref".to_string());
    }
    let record_names = record.outputs.iter().map(|output| output.name.clone()).collect::<Vec<_>>();
    if sorted_unique_strings(record_names) != sorted_unique_strings(request.output_names.clone()) {
        diagnostics.push("action-result-output-declaration-mismatch".to_string());
    }
    if !request.policy.allowed_source_classes.contains(&facts.source_class) {
        diagnostics.push("action-result-source-class-disallowed".to_string());
    }
    for required in &request.policy.required_non_claims {
        if !record.non_claims.contains(required) {
            diagnostics.push(format!("action-result-required-non-claim-missing:{required}"));
        }
    }
}

fn validate_candidate_trust(
    request: &StrongReuseRequest,
    signed: &SignedActionResultRecord,
    facts: &CandidateAdmissionFacts,
    diagnostics: &mut Vec<String>,
) {
    assert_eq!(
        request.policy.schema, ACTION_RESULT_POLICY_SCHEMA,
        "candidate trust requires a validated policy schema"
    );
    assert!(
        !request.policy.allowed_source_classes.is_empty(),
        "candidate trust requires an allowed source class"
    );
    let record = &signed.record;
    if !request.policy.trusted_producers.is_empty()
        && !request.policy.trusted_producers.contains(&record.producer_identity)
    {
        diagnostics.push("action-result-producer-untrusted".to_string());
    }
    if request.policy.require_record_signature && signed.record_signatures.is_empty() {
        diagnostics.push("action-result-record-signature-missing".to_string());
    }
    let has_trusted_record_signer = facts
        .verified_record_signers
        .iter()
        .any(|signer| request.policy.trusted_record_signers.contains(signer));
    if request.policy.require_record_signature && !has_trusted_record_signer {
        diagnostics.push("action-result-record-signature-untrusted".to_string());
    }
    if !request.policy.allowed_sandbox_policy_refs.is_empty()
        && !request.policy.allowed_sandbox_policy_refs.contains(&record.sandbox_policy_ref)
    {
        diagnostics.push("action-result-sandbox-policy-untrusted".to_string());
    }
    if !request.policy.allowed_network_policy_refs.is_empty()
        && !request.policy.allowed_network_policy_refs.contains(&record.network_policy_ref)
    {
        diagnostics.push("action-result-network-policy-untrusted".to_string());
    }
}

fn validate_candidate_admission_facts(facts: &CandidateAdmissionFacts, diagnostics: &mut Vec<String>) {
    push_missing_fact(diagnostics, facts.action_receipt_linked, "action-result-receipt-linkage-invalid");
    push_missing_fact(diagnostics, facts.object_refs_complete, "action-result-object-incomplete");
    push_missing_fact(diagnostics, facts.path_info_refs_linked, "action-result-pathinfo-linkage-invalid");
    push_missing_fact(diagnostics, facts.path_info_signatures_verified, "action-result-pathinfo-signature-untrusted");
    push_missing_fact(diagnostics, facts.producer_policy_admitted, "action-result-producer-policy-rejected");
    push_missing_fact(diagnostics, facts.publication_policy_admitted, "action-result-publication-policy-rejected");
    push_missing_fact(diagnostics, facts.sandbox_policy_admitted, "action-result-sandbox-policy-rejected");
    push_missing_fact(diagnostics, facts.network_policy_admitted, "action-result-network-policy-rejected");
    push_missing_fact(diagnostics, facts.reference_scans_admitted, "action-result-reference-scan-rejected");
    if facts.claim_strength != STRONG_CLAIM {
        diagnostics.push("action-result-claim-strength-insufficient".to_string());
    }
}

fn output_set_digest(record: &ActionResultRecord) -> Result<String, String> {
    let outputs = record
        .outputs
        .iter()
        .map(|output| OutputSetMember {
            name: &output.name,
            object_ref: &output.object_ref,
        })
        .collect::<Vec<_>>();
    let bytes = serde_json::to_vec(&outputs).map_err(|error| format!("action-result-output-set-json:{error}"))?;
    Ok(domain_digest(OUTPUT_SET_DOMAIN, &bytes))
}

fn validate_record_input(input: &ActionResultRecordInput) -> Result<(), String> {
    validate_typed_ref(TypedRefValidation {
        field: "action-ref",
        value: &input.action_ref,
        prefix: ACTION_REF_PREFIX,
    })?;
    if input.outputs.is_empty() || input.outputs.len() > MAX_ACTION_RESULT_OUTPUTS {
        return Err("action-result-output-count-invalid".to_string());
    }
    assert!(!input.outputs.is_empty(), "validated action result must declare an output");
    assert!(
        input.outputs.len() <= MAX_ACTION_RESULT_OUTPUTS,
        "validated action result outputs must remain bounded"
    );
    if input.outputs != sorted_unique_structs(input.outputs.clone()) {
        return Err("action-result-outputs-not-canonical".to_string());
    }
    for output in &input.outputs {
        validate_output(output)?;
    }
    validate_typed_ref(TypedRefValidation {
        field: "action-receipt-ref",
        value: &input.action_receipt_ref,
        prefix: ACTION_RECEIPT_REF_PREFIX,
    })?;
    validate_bounded_refs(
        "reference-scan-ref",
        &input.reference_scan_refs,
        REFERENCE_SCAN_REF_PREFIX,
        MAX_ACTION_RESULT_METADATA_REFS,
    )?;
    validate_typed_ref(TypedRefValidation {
        field: "sandbox-policy-ref",
        value: &input.sandbox_policy_ref,
        prefix: SANDBOX_POLICY_REF_PREFIX,
    })?;
    validate_typed_ref(TypedRefValidation {
        field: "network-policy-ref",
        value: &input.network_policy_ref,
        prefix: NETWORK_POLICY_REF_PREFIX,
    })?;
    validate_identifier(IdentifierValidation {
        field: "producer-identity",
        value: &input.producer_identity,
    })?;
    validate_typed_ref(TypedRefValidation {
        field: "producer-policy-ref",
        value: &input.producer_policy_ref,
        prefix: PRODUCER_POLICY_REF_PREFIX,
    })?;
    validate_bounded_refs(
        "signature-ref",
        &input.signature_refs,
        SIGNATURE_REF_PREFIX,
        MAX_ACTION_RESULT_SIGNATURE_REFS,
    )?;
    validate_typed_ref(TypedRefValidation {
        field: "publication-policy-ref",
        value: &input.publication_policy_ref,
        prefix: PUBLICATION_POLICY_REF_PREFIX,
    })?;
    validate_non_claims(&input.non_claims)
}

fn validate_output(output: &ActionResultOutput) -> Result<(), String> {
    validate_identifier(IdentifierValidation {
        field: "output-name",
        value: &output.name,
    })?;
    validate_typed_ref(TypedRefValidation {
        field: "object-ref",
        value: &output.object_ref,
        prefix: OBJECT_REF_PREFIX,
    })?;
    validate_typed_ref(TypedRefValidation {
        field: "path-info-ref",
        value: &output.path_info_ref,
        prefix: PATH_INFO_REF_PREFIX,
    })?;
    if !output.store_path.starts_with('/') || output.store_path.len() > MAX_ACTION_RESULT_PATH_BYTES {
        return Err("action-result-store-path-invalid".to_string());
    }
    if output.store_path.split('/').any(|segment| segment == "..") {
        return Err("action-result-store-path-traversal".to_string());
    }
    assert!(output.store_path.starts_with('/'), "validated store path must remain absolute");
    assert!(output.store_path.len() <= MAX_ACTION_RESULT_PATH_BYTES, "validated store path must remain bounded");
    Ok(())
}

fn validate_non_claims(non_claims: &[String]) -> Result<(), String> {
    if non_claims.is_empty() || non_claims.len() > MAX_ACTION_RESULT_NON_CLAIMS {
        return Err("action-result-non-claim-count-invalid".to_string());
    }
    if non_claims != sorted_unique_strings(non_claims.to_vec()) {
        return Err("action-result-non-claims-not-canonical".to_string());
    }
    for required in required_non_claims() {
        if !non_claims.contains(&required) {
            return Err(format!("action-result-required-non-claim-missing:{required}"));
        }
    }
    Ok(())
}

fn validate_policy(policy: &ActionResultTrustPolicy) -> Result<(), String> {
    if policy.schema != ACTION_RESULT_POLICY_SCHEMA {
        return Err("action-result-policy-schema-unsupported".to_string());
    }
    validate_identifier(IdentifierValidation {
        field: "policy-id",
        value: &policy.policy_id,
    })?;
    if policy.allowed_source_classes.is_empty() {
        return Err("action-result-policy-source-classes-empty".to_string());
    }
    if policy.require_record_signature && policy.trusted_record_signers.is_empty() {
        return Err("action-result-policy-trusted-signers-empty".to_string());
    }
    if policy.require_path_info_signature && policy.trusted_record_signers.is_empty() {
        return Err("action-result-policy-pathinfo-trust-empty".to_string());
    }
    Ok(())
}

fn validate_request(request: &StrongReuseRequest, candidate_count: usize) -> Result<(), String> {
    validate_typed_ref(TypedRefValidation {
        field: "action-ref",
        value: &request.action_ref,
        prefix: ACTION_REF_PREFIX,
    })?;
    if request.output_names.is_empty() || request.output_names.len() > MAX_ACTION_RESULT_OUTPUTS {
        return Err("action-result-request-output-count-invalid".to_string());
    }
    if candidate_count > MAX_ACTION_RESULT_CANDIDATES {
        return Err("action-result-candidate-count-exceeded".to_string());
    }
    validate_policy(&request.policy)
}

fn validate_index_result_refs(refs: &[String]) -> Result<(), String> {
    if refs.len() > MAX_ACTION_RESULT_CANDIDATES {
        return Err("result-ref-count-invalid".to_string());
    }
    if refs != sorted_unique_strings(refs.to_vec()) {
        return Err("result-ref-list-not-canonical".to_string());
    }
    for value in refs {
        validate_typed_ref(TypedRefValidation {
            field: "result-ref",
            value,
            prefix: ACTION_RESULT_REF_PREFIX,
        })?;
    }
    Ok(())
}

fn validate_bounded_refs(field: &str, refs: &[String], prefix: &str, count_max: usize) -> Result<(), String> {
    if refs.is_empty() || refs.len() > count_max {
        return Err(format!("{field}-count-invalid"));
    }
    if refs != sorted_unique_strings(refs.to_vec()) {
        return Err(format!("{field}-list-not-canonical"));
    }
    for value in refs {
        validate_typed_ref(TypedRefValidation { field, value, prefix })?;
    }
    Ok(())
}

struct TypedRefValidation<'a> {
    field: &'a str,
    value: &'a str,
    prefix: &'a str,
}

fn validate_typed_ref(input: TypedRefValidation<'_>) -> Result<(), String> {
    let Some(digest) = input.value.strip_prefix(input.prefix) else {
        return Err(format!("{}-prefix-invalid", input.field));
    };
    if digest.len() != BLAKE3_HEX_CHARS {
        return Err(format!("{}-digest-length-invalid", input.field));
    }
    if !digest.chars().all(|character| character.is_ascii_digit() || ('a'..='f').contains(&character)) {
        return Err(format!("{}-digest-encoding-invalid", input.field));
    }
    Ok(())
}

struct IdentifierValidation<'a> {
    field: &'a str,
    value: &'a str,
}

fn validate_identifier(input: IdentifierValidation<'_>) -> Result<(), String> {
    if input.value.is_empty() || input.value.len() > MAX_ACTION_RESULT_ID_BYTES {
        return Err(format!("{}-invalid", input.field));
    }
    if input.value.chars().any(char::is_control) {
        return Err(format!("{}-control-character", input.field));
    }
    Ok(())
}

fn normalize_record_input(mut input: ActionResultRecordInput) -> ActionResultRecordInput {
    input.outputs = sorted_unique_structs(input.outputs);
    input.reference_scan_refs = sorted_unique_strings(input.reference_scan_refs);
    input.signature_refs = sorted_unique_strings(input.signature_refs);
    input.non_claims = sorted_unique_strings(input.non_claims);
    input
}

fn normalize_policy(mut policy: ActionResultTrustPolicy) -> ActionResultTrustPolicy {
    policy.trusted_producers = sorted_unique_strings(policy.trusted_producers);
    policy.trusted_record_signers = sorted_unique_strings(policy.trusted_record_signers);
    policy.allowed_source_classes = sorted_unique_strings(policy.allowed_source_classes);
    policy.allowed_sandbox_policy_refs = sorted_unique_strings(policy.allowed_sandbox_policy_refs);
    policy.allowed_network_policy_refs = sorted_unique_strings(policy.allowed_network_policy_refs);
    policy.required_non_claims = sorted_unique_strings(policy.required_non_claims);
    policy
}

fn record_input(record: &ActionResultRecord) -> ActionResultRecordInput {
    ActionResultRecordInput {
        action_ref: record.action_ref.clone(),
        outputs: record.outputs.clone(),
        action_receipt_ref: record.action_receipt_ref.clone(),
        reference_scan_refs: record.reference_scan_refs.clone(),
        sandbox_policy_ref: record.sandbox_policy_ref.clone(),
        network_policy_ref: record.network_policy_ref.clone(),
        producer_identity: record.producer_identity.clone(),
        producer_policy_ref: record.producer_policy_ref.clone(),
        signature_refs: record.signature_refs.clone(),
        publication_policy_ref: record.publication_policy_ref.clone(),
        non_claims: record.non_claims.clone(),
    }
}

fn required_non_claims() -> Vec<String> {
    vec![
        REQUIRED_NON_CLAIM_CA_MAPPING_NOT_TRUST.to_string(),
        REQUIRED_NON_CLAIM_EXECUTOR_CORRECTNESS.to_string(),
        REQUIRED_NON_CLAIM_INDEX_NOT_TRUST.to_string(),
    ]
}

fn digest_ref<T: Serialize>(prefix: &str, domain: &[u8], value: &T) -> Result<String, String> {
    let bytes = serde_json::to_vec(value).map_err(|error| format!("action-result-canonical-json:{error}"))?;
    Ok(format!("{prefix}{}", domain_digest(domain, &bytes)))
}

fn domain_digest(domain: &[u8], bytes: &[u8]) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(domain);
    hasher.update(&[DOMAIN_SEPARATOR]);
    hasher.update(bytes);
    hasher.finalize().to_hex().to_string()
}

fn sorted_unique_strings(values: Vec<String>) -> Vec<String> {
    values.into_iter().collect::<BTreeSet<_>>().into_iter().collect()
}

fn sorted_unique_structs<T: Ord>(values: Vec<T>) -> Vec<T> {
    values.into_iter().collect::<BTreeSet<_>>().into_iter().collect()
}

fn is_strictly_sorted<T: Ord>(values: &[T]) -> bool {
    values.iter().zip(values.iter().skip(1)).all(|(left, right)| left < right)
}

fn push_error(diagnostics: &mut Vec<String>, result: Result<(), String>) {
    if let Err(error) = result {
        diagnostics.push(error);
    }
}

fn push_missing_fact(diagnostics: &mut Vec<String>, admitted: bool, diagnostic: &str) {
    if !admitted {
        diagnostics.push(diagnostic.to_string());
    }
}

#[derive(Serialize)]
struct IndexHashable<'a> {
    schema: &'static str,
    action_ref: &'a str,
    result_refs: &'a [String],
}

#[derive(Serialize)]
struct OutputSetMember<'a> {
    name: &'a str,
    object_ref: &'a str,
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    const PRODUCER: &str = "builder-key-1";
    const SOURCE: &str = "local-action-results";
    const SOURCE_CLASS_LOCAL: &str = "local";
    const SOURCE_CLASS_HTTP: &str = "http";
    const SIGNER: &str = "builder-key-1";

    fn typed_ref(prefix: &str, seed: &str) -> String {
        format!("{prefix}{}", blake3::hash(seed.as_bytes()).to_hex())
    }

    fn record_input(seed: &str) -> ActionResultRecordInput {
        ActionResultRecordInput {
            action_ref: typed_ref(ACTION_REF_PREFIX, "action"),
            outputs: vec![ActionResultOutput {
                name: "out".to_string(),
                object_ref: typed_ref(OBJECT_REF_PREFIX, seed),
                store_path: format!("/mantle/store/{seed}-demo"),
                path_info_ref: typed_ref(PATH_INFO_REF_PREFIX, seed),
            }],
            action_receipt_ref: typed_ref(ACTION_RECEIPT_REF_PREFIX, seed),
            reference_scan_refs: vec![typed_ref(REFERENCE_SCAN_REF_PREFIX, seed)],
            sandbox_policy_ref: typed_ref(SANDBOX_POLICY_REF_PREFIX, "sandbox"),
            network_policy_ref: typed_ref(NETWORK_POLICY_REF_PREFIX, "network"),
            producer_identity: PRODUCER.to_string(),
            producer_policy_ref: typed_ref(PRODUCER_POLICY_REF_PREFIX, "producer"),
            signature_refs: vec![typed_ref(SIGNATURE_REF_PREFIX, seed)],
            publication_policy_ref: typed_ref(PUBLICATION_POLICY_REF_PREFIX, "publication"),
            non_claims: required_non_claims(),
        }
    }

    fn signed_record(seed: &str) -> SignedActionResultRecord {
        SignedActionResultRecord {
            record: canonical_action_result(record_input(seed)).unwrap(),
            record_signatures: vec![DetachedRecordSignature {
                key_name: SIGNER.to_string(),
                signature: "detached-signature-bytes".to_string(),
            }],
        }
    }

    fn policy() -> ActionResultTrustPolicy {
        ActionResultTrustPolicy {
            trusted_producers: vec![PRODUCER.to_string()],
            trusted_record_signers: vec![SIGNER.to_string()],
            allowed_source_classes: vec![SOURCE_CLASS_LOCAL.to_string(), SOURCE_CLASS_HTTP.to_string()],
            allowed_sandbox_policy_refs: vec![typed_ref(SANDBOX_POLICY_REF_PREFIX, "sandbox")],
            allowed_network_policy_refs: vec![typed_ref(NETWORK_POLICY_REF_PREFIX, "network")],
            ..ActionResultTrustPolicy::default()
        }
    }

    fn admitted_facts() -> CandidateAdmissionFacts {
        CandidateAdmissionFacts {
            source_id: SOURCE.to_string(),
            source_class: SOURCE_CLASS_LOCAL.to_string(),
            verified_record_signers: vec![SIGNER.to_string()],
            action_receipt_linked: true,
            object_refs_complete: true,
            path_info_refs_linked: true,
            path_info_signatures_verified: true,
            producer_policy_admitted: true,
            publication_policy_admitted: true,
            sandbox_policy_admitted: true,
            network_policy_admitted: true,
            reference_scans_admitted: true,
            claim_strength: STRONG_CLAIM.to_string(),
        }
    }

    fn request() -> StrongReuseRequest {
        StrongReuseRequest {
            action_ref: typed_ref(ACTION_REF_PREFIX, "action"),
            output_names: vec!["out".to_string()],
            policy: policy(),
        }
    }

    // r[verify build_correctness.shared_action_result_records]
    #[test]
    fn permutation_equivalent_records_and_indexes_have_identical_refs() {
        let mut left_input = record_input("same");
        left_input.reference_scan_refs.push(typed_ref(REFERENCE_SCAN_REF_PREFIX, "second"));
        left_input.signature_refs.push(typed_ref(SIGNATURE_REF_PREFIX, "second"));
        let mut right_input = left_input.clone();
        right_input.reference_scan_refs.reverse();
        right_input.signature_refs.reverse();
        right_input.non_claims.reverse();

        let left = canonical_action_result(left_input).unwrap();
        let right = canonical_action_result(right_input).unwrap();
        let left_index = canonical_action_result_index(left.action_ref.clone(), vec![left.result_ref.clone()]).unwrap();
        let right_index = canonical_action_result_index(right.action_ref.clone(), vec![
            right.result_ref.clone(),
            right.result_ref.clone(),
        ])
        .unwrap();

        assert_eq!(left.result_ref, right.result_ref);
        assert_eq!(left_index.index_ref, right_index.index_ref);
        assert_eq!(left_index.result_refs.len(), 1);
        assert!(left.result_ref.starts_with(ACTION_RESULT_REF_PREFIX));
    }

    #[test]
    fn identical_candidates_deduplicate_and_one_matching_signed_result_is_admitted() {
        let candidate = DiscoveredActionResultCandidate {
            signed_record: signed_record("same"),
            facts: admitted_facts(),
        };
        let plan = plan_strong_reuse(request(), vec![candidate.clone(), candidate]).unwrap();

        assert_eq!(plan.candidate_decisions.len(), 1);
        assert_eq!(plan.selected_result_ref.as_deref(), Some(plan.candidate_decisions[0].result_ref.as_str()));
        assert!(plan.candidate_decisions[0].admitted);
        assert!(plan.conflict_class.is_none());
    }

    // r[verify build_correctness.shared_action_result_admission]
    #[test]
    fn conflicting_admitted_output_sets_fail_strong_reuse_without_source_order_selection() {
        let left = DiscoveredActionResultCandidate {
            signed_record: signed_record("left"),
            facts: admitted_facts(),
        };
        let mut right_facts = admitted_facts();
        right_facts.source_id = "remote-action-results".to_string();
        right_facts.source_class = SOURCE_CLASS_HTTP.to_string();
        let right = DiscoveredActionResultCandidate {
            signed_record: signed_record("right"),
            facts: right_facts,
        };
        let forward = plan_strong_reuse(request(), vec![left.clone(), right.clone()]).unwrap();
        let reverse = plan_strong_reuse(request(), vec![right, left]).unwrap();

        assert_eq!(forward.conflict_class.as_deref(), Some(CONFLICTING_ACTION_RESULTS));
        assert_eq!(forward.selected_result_ref, None);
        assert_eq!(forward, reverse);
        assert_eq!(forward.admitted_output_set_digests_blake3.len(), 2);
    }

    #[test]
    fn stale_drift_unsigned_policy_and_missing_admission_facts_fail_closed() {
        let mut candidate = DiscoveredActionResultCandidate {
            signed_record: signed_record("bad"),
            facts: admitted_facts(),
        };
        candidate.signed_record.record.action_ref = typed_ref(ACTION_REF_PREFIX, "stale");
        candidate.signed_record.record.outputs[0].object_ref = typed_ref(OBJECT_REF_PREFIX, "drift");
        candidate.signed_record.record_signatures.clear();
        candidate.facts.verified_record_signers.clear();
        candidate.facts.object_refs_complete = false;
        candidate.facts.path_info_signatures_verified = false;
        candidate.facts.producer_policy_admitted = false;
        candidate.facts.publication_policy_admitted = false;
        candidate.facts.reference_scans_admitted = false;

        let plan = plan_strong_reuse(request(), vec![candidate]).unwrap();
        let diagnostics = &plan.candidate_decisions[0].diagnostics;

        assert!(plan.selected_result_ref.is_none());
        assert!(diagnostics.contains(&"action-result-ref-mismatch".to_string()));
        assert!(diagnostics.contains(&"stale-action-ref".to_string()));
        assert!(diagnostics.contains(&"action-result-record-signature-missing".to_string()));
        assert!(diagnostics.contains(&"action-result-object-incomplete".to_string()));
        assert!(diagnostics.contains(&"action-result-pathinfo-signature-untrusted".to_string()));
        assert!(diagnostics.contains(&"action-result-producer-policy-rejected".to_string()));
        assert!(diagnostics.contains(&"action-result-publication-policy-rejected".to_string()));
        assert!(diagnostics.contains(&"action-result-reference-scan-rejected".to_string()));
    }

    #[test]
    fn malformed_refs_partial_records_and_oversized_indexes_are_rejected() {
        let mut malformed = record_input("malformed");
        malformed.action_receipt_ref = "not-a-ref".to_string();
        let partial = ActionResultRecordInput {
            outputs: Vec::new(),
            ..record_input("partial")
        };
        let oversized_refs = (0..=MAX_ACTION_RESULT_CANDIDATES)
            .map(|index| typed_ref(ACTION_RESULT_REF_PREFIX, &format!("result-{index}")))
            .collect::<Vec<_>>();

        assert_eq!(canonical_action_result(malformed).unwrap_err(), "action-receipt-ref-prefix-invalid");
        assert_eq!(canonical_action_result(partial).unwrap_err(), "action-result-output-count-invalid");
        assert_eq!(
            canonical_action_result_index(typed_ref(ACTION_REF_PREFIX, "action"), oversized_refs).unwrap_err(),
            "result-ref-count-invalid"
        );
    }

    #[test]
    fn ca_mapping_and_index_presence_never_replace_verified_admission_facts() {
        let mut candidate = DiscoveredActionResultCandidate {
            signed_record: signed_record("hint-only"),
            facts: admitted_facts(),
        };
        candidate.facts.object_refs_complete = false;
        candidate.facts.path_info_refs_linked = false;
        candidate.facts.path_info_signatures_verified = false;
        candidate.facts.action_receipt_linked = false;
        let plan = plan_strong_reuse(request(), vec![candidate]).unwrap();
        let decision = &plan.candidate_decisions[0];

        assert!(!decision.admitted);
        assert!(plan.selected_result_ref.is_none());
        assert!(decision.diagnostics.contains(&"action-result-receipt-linkage-invalid".to_string()));
        assert!(decision.diagnostics.contains(&"action-result-pathinfo-linkage-invalid".to_string()));
        assert!(plan.non_claims.contains(&REQUIRED_NON_CLAIM_CA_MAPPING_NOT_TRUST.to_string()));
        assert!(plan.non_claims.contains(&REQUIRED_NON_CLAIM_INDEX_NOT_TRUST.to_string()));
    }

    #[test]
    fn admitted_output_digest_is_present_or_fails_closed_without_panicking() {
        let candidate = DiscoveredActionResultCandidate {
            signed_record: signed_record("digest-invariant"),
            facts: admitted_facts(),
        };
        let plan = plan_strong_reuse(request(), vec![candidate]).unwrap();
        let admitted = plan.candidate_decisions[0].clone();
        let expected_digest = admitted.output_set_digest_blake3.clone();

        assert_eq!(admitted_output_digest(&admitted), Ok(expected_digest));

        let mut missing = admitted;
        missing.output_set_digest_blake3 = None;
        assert_eq!(
            admitted_output_digest(&missing),
            Err("action-result-admitted-candidate-digest-missing".to_string())
        );

        missing.admitted = false;
        assert_eq!(admitted_output_digest(&missing), Ok(None));
    }
}
