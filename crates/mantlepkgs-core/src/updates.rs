// r[impl mantlepkgs_updates.typed_policy]
// r[impl mantlepkgs_updates.source_observations]
// r[impl mantlepkgs_updates.candidate_selection]
// r[impl mantlepkgs_updates.preimage_bound_mutation]
// r[impl mantlepkgs_updates.advisory_evidence]
// r[impl mantlepkgs_updates.validation_evidence]
// r[impl mantlepkgs_updates.functional_core]
// r[impl mantlepkgs_updates.claim_boundary]
// r[verify mantlepkgs_updates.typed_policy]
// r[verify mantlepkgs_updates.source_observations]
// r[verify mantlepkgs_updates.candidate_selection]
// r[verify mantlepkgs_updates.preimage_bound_mutation]
// r[verify mantlepkgs_updates.advisory_evidence]
// r[verify mantlepkgs_updates.validation_evidence]
// r[verify mantlepkgs_updates.functional_core]
// r[verify mantlepkgs_updates.claim_boundary]

use alloc::collections::BTreeMap;
use alloc::collections::BTreeSet;
use alloc::format;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec;
use alloc::vec::Vec;
use core::cmp::Ordering;

use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;

use crate::CoreFailure;
use crate::Diagnostic;
use crate::ImpactPackageKey;

pub const UPDATE_POLICY_SCHEMA: &str = "mantlepkgs-update-policy-v1";
pub const UPDATE_SOURCE_OBSERVATION_SCHEMA: &str = "mantlepkgs-source-observation-v1";
pub const UPDATE_ADVISORY_OBSERVATION_SCHEMA: &str = "mantlepkgs-advisory-observation-v1";
pub const UPDATE_VALIDATION_EVIDENCE_SCHEMA: &str = "mantlepkgs-update-validation-evidence-v1";
pub const UPDATE_MUTATION_DOCUMENT_SCHEMA: &str = "mantlepkgs-update-mutation-document-v1";
pub const UPDATE_PLAN_SCHEMA: &str = "mantlepkgs-update-plan-v1";
pub const UPDATE_EXECUTION_RECEIPT_SCHEMA: &str = "mantlepkgs-update-execution-receipt-v1";
pub const UPDATE_IDENTITY_DOMAIN: &str = "mantle.mantlepkgs.update-plan.v1";
pub const UPDATE_MIGRATION_SOURCE_SCHEMA: &str = "mantlepkgs-update-policy-v0";
pub const SOURCE_RESPONSE_GIT_TAGS_SCHEMA: &str = "mantlepkgs-git-tags-response-v1";
pub const SOURCE_RESPONSE_RELEASE_INDEX_SCHEMA: &str = "mantlepkgs-release-index-response-v1";
pub const SOURCE_RESPONSE_DIRECTORY_INDEX_SCHEMA: &str = "mantlepkgs-directory-index-response-v1";
pub const ADVISORY_RESPONSE_OSV_SCHEMA: &str = "mantlepkgs-osv-response-v1";
pub const ADVISORY_RESPONSE_REPOLOGY_SCHEMA: &str = "mantlepkgs-repology-response-v1";

const POLICY_IDENTITY_DOMAIN: &[u8] = b"mantle.mantlepkgs.update-policy.v1";
const SOURCE_OBSERVATION_IDENTITY_DOMAIN: &[u8] = b"mantle.mantlepkgs.source-observation.v1";
const ADVISORY_OBSERVATION_IDENTITY_DOMAIN: &[u8] = b"mantle.mantlepkgs.advisory-observation.v1";
const PLAN_IDENTITY_DOMAIN: &[u8] = b"mantle.mantlepkgs.update-plan.v1";
const EXECUTION_RECEIPT_IDENTITY_DOMAIN: &[u8] = b"mantle.mantlepkgs.update-execution-receipt.v1";
const DOMAIN_SEPARATOR: u8 = 0;
const BLAKE3_HEX_CHARS: usize = 64;
const VERSION_COMPONENT_COUNT_MIN: u8 = 1;
const VERSION_COMPONENT_COUNT_MAX: u8 = 4;
const VERSION_MINOR_INDEX: usize = 1;
const VERSION_PATCH_INDEX: usize = 2;
const DEVELOPMENT_ODD_DIVISOR: u64 = 2;
const HARD_MAX_RESPONSE_BYTES: u64 = 67_108_864;
const HARD_MAX_DOCUMENT_BYTES: u64 = 16_777_216;
const HARD_MAX_PLAN_BYTES: u64 = 67_108_864;
const HARD_MAX_CANDIDATES: u32 = 65_536;
const HARD_MAX_FINDINGS: u32 = 65_536;
const HARD_MAX_EFFECTS: u32 = 4_096;
const HARD_MAX_DIAGNOSTICS: u32 = 4_096;
const HARD_MAX_ARTIFACTS: u32 = 65_536;
const HARD_MAX_REDIRECTS: u32 = 16;
const HARD_MAX_RETRIES: u32 = 16;
const HARD_MAX_ELAPSED_MILLIS: u64 = 3_600_000;
const UPDATE_NON_CLAIM_COUNT: usize = 6;
const MUTATION_POINTER_COUNT: usize = 3;
const VALIDATION_LINK_COUNT: usize = 4;
const ADVISORY_SERVICE_COUNT: usize = 2;
const ADVISORY_SERVICE_COUNT_MAX: u32 = 2;
const EXPECTED_MUTATION_DOCUMENT_COUNT: usize = 1;
const DUPLICATE_WINDOW_SIZE: usize = 2;

const NON_CLAIMS: [&str; UPDATE_NON_CLAIM_COUNT] = [
    "source observations do not prove source trust",
    "advisory observations do not prove the absence of unknown vulnerabilities",
    "candidate selection does not prove package correctness",
    "validation links do not prove reproducibility",
    "ready-for-review does not prove deployment safety",
    "update evidence does not prove release eligibility",
];

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum UpdateSourceKind {
    GitTags,
    ReleaseIndex,
    DirectoryIndex,
    EcosystemRegistry,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ObservationStatus {
    Success,
    Unavailable,
    Failed,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum AdvisoryService {
    Osv,
    Repology,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum AdvisoryRequirementMode {
    Disabled,
    Optional,
    Required,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum EvidenceStatus {
    Success,
    Missing,
    Unavailable,
    Failed,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum UpdateProposalStatus {
    ReadyForReview,
    Blocked,
    Unavailable,
    Failed,
    NoCandidate,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum UpdateExecutionDisposition {
    Applied,
    Denied,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct UpdatePolicyMigration {
    pub source_schema: Option<String>,
    pub source_policy_identity_blake3: Option<String>,
    pub reviewed: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct VersionRules {
    pub required_prefix: String,
    pub component_count: u8,
    pub max_component: u64,
    pub minimum_version: Option<String>,
    pub maximum_version: Option<String>,
    pub ignored_versions: Vec<String>,
    pub allow_prerelease: bool,
    pub odd_minor_is_development: bool,
    pub high_patch_development_from: Option<u64>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PatchPolicy {
    pub allow_major: bool,
    pub allow_minor: bool,
    pub allow_patch: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AdvisoryRequirement {
    pub mode: AdvisoryRequirementMode,
    pub service_identity: String,
    pub query: String,
    pub package_coordinate: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AdvisoryPolicy {
    pub osv: AdvisoryRequirement,
    pub repology: AdvisoryRequirement,
    pub block_on_findings: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct UpdateValidationPolicy {
    pub require_catalog: bool,
    pub require_build_observations: bool,
    pub require_validation_roots: bool,
    pub require_impact_report: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct UpdateLimits {
    pub max_response_bytes: u64,
    pub max_document_bytes: u64,
    pub max_plan_bytes: u64,
    pub max_candidates: u32,
    pub max_findings: u32,
    pub max_effects: u32,
    pub max_diagnostics: u32,
    pub max_artifacts: u32,
    pub max_redirects: u32,
    pub max_retries: u32,
    pub max_elapsed_millis: u64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct UpdateMutationTarget {
    pub relative_path: String,
    pub version_pointer: String,
    pub source_ref_pointer: String,
    pub source_identity_pointer: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct UpdatePolicy {
    pub schema: String,
    pub policy_identity_blake3: String,
    pub migration: UpdatePolicyMigration,
    pub package: ImpactPackageKey,
    pub system: String,
    pub source_kind: UpdateSourceKind,
    pub adapter_identity: String,
    pub source_authority: String,
    pub source_query: String,
    pub response_schema: String,
    pub current_version: String,
    pub current_source_ref: String,
    pub current_source_identity_blake3: String,
    pub version_rules: VersionRules,
    pub patch_policy: PatchPolicy,
    pub advisory_policy: AdvisoryPolicy,
    pub validation_policy: UpdateValidationPolicy,
    pub mutation: UpdateMutationTarget,
    pub allow_ambient_credentials: bool,
    pub allow_ambient_proxy: bool,
    pub limits: UpdateLimits,
}

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SourceCandidate {
    pub version: String,
    pub source_ref: String,
    pub source_identity_blake3: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CollectionFacts {
    pub response_bytes: u64,
    pub redirect_count: u32,
    pub retry_count: u32,
    pub elapsed_millis: u64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SourceObservation {
    pub schema: String,
    pub observation_identity_blake3: String,
    pub policy_identity_blake3: String,
    pub adapter_identity: String,
    pub source_kind: UpdateSourceKind,
    pub query: String,
    pub source_authority: String,
    pub response_schema: String,
    pub response_identity_blake3: Option<String>,
    pub status: ObservationStatus,
    pub candidates: Vec<SourceCandidate>,
    pub reason_codes: Vec<String>,
    pub collection: CollectionFacts,
}

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AdvisoryFinding {
    pub finding_id: String,
    pub finding_identity_blake3: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AdvisoryObservation {
    pub schema: String,
    pub observation_identity_blake3: String,
    pub policy_identity_blake3: String,
    pub service: AdvisoryService,
    pub service_identity: String,
    pub query: String,
    pub package_coordinate: String,
    pub version: String,
    pub response_schema: String,
    pub response_identity_blake3: Option<String>,
    pub status: ObservationStatus,
    pub findings: Vec<AdvisoryFinding>,
    pub reason_codes: Vec<String>,
    pub collection: CollectionFacts,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LinkedEvidence {
    pub status: EvidenceStatus,
    pub artifact_identity_blake3: Vec<String>,
    pub reason_codes: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct UpdateValidationEvidence {
    pub schema: String,
    pub candidate_version: String,
    pub candidate_source_identity_blake3: String,
    pub catalog: LinkedEvidence,
    pub build_observations: LinkedEvidence,
    pub validation_roots: LinkedEvidence,
    pub impact_report: LinkedEvidence,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MutationDocument {
    pub schema: String,
    pub relative_path: String,
    pub input_bytes: u64,
    pub input_digest_blake3: String,
    pub document: Value,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct StructuredEdit {
    pub field_pointer: String,
    pub old_value: Value,
    pub new_value: Value,
    pub reason_code: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DocumentMutationEffect {
    pub relative_path: String,
    pub input_digest_blake3: String,
    pub output_digest_blake3: String,
    pub edits: Vec<StructuredEdit>,
    pub output_document: Value,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct NormalizedVersion {
    pub canonical: String,
    pub components: Vec<u64>,
    pub prerelease: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CandidateDecision {
    pub candidate: SourceCandidate,
    pub normalized_version: Option<String>,
    pub eligible: bool,
    pub reason_codes: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CandidateSelection {
    pub selected: Option<SourceCandidate>,
    pub decisions: Vec<CandidateDecision>,
    pub reason_codes: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct UpdatePlan {
    pub schema: String,
    pub plan_identity_blake3: String,
    pub policy_identity_blake3: String,
    pub source_observation_identity_blake3: String,
    pub advisory_observation_identity_blake3: Vec<String>,
    pub package: ImpactPackageKey,
    pub selection: CandidateSelection,
    pub effects: Vec<DocumentMutationEffect>,
    pub validation_evidence: UpdateValidationEvidence,
    pub proposal_status: UpdateProposalStatus,
    pub reason_codes: Vec<String>,
    pub non_claims: Vec<String>,
}

pub struct UpdatePlanInput<'a> {
    pub policy: &'a UpdatePolicy,
    pub source_observation: &'a SourceObservation,
    pub advisory_observations: &'a [AdvisoryObservation],
    pub validation_evidence: &'a UpdateValidationEvidence,
    pub mutation_documents: &'a [MutationDocument],
}

struct TextField<'a> {
    value: &'a str,
    path: &'a str,
}

struct TextExpectation<'a> {
    actual: &'a str,
    expected: &'a str,
    path: &'a str,
}

struct IdentityExpectation<'a> {
    path: &'a str,
    expected: &'a str,
    actual: &'a str,
}

struct DiagnosticInput<'a> {
    code: &'a str,
    path: &'a str,
    message: &'a str,
}

struct MutationPointerInput<'a> {
    name: &'a str,
    pointer: &'a str,
}

struct U64LimitInput<'a> {
    value: u64,
    maximum: u64,
    path: &'a str,
}

struct U32LimitInput<'a> {
    value: u32,
    maximum: u32,
    path: &'a str,
}

struct CountInput<'a> {
    count: usize,
    maximum: u32,
    path: &'a str,
    code: &'a str,
}

struct ObservedU64LimitInput<'a> {
    value: u64,
    maximum: u64,
    path: &'a str,
    code: &'a str,
    message: &'a str,
}

struct ObservedU32LimitInput<'a> {
    value: u32,
    maximum: u32,
    path: &'a str,
    code: &'a str,
    message: &'a str,
}

struct StatusPayloadInput<'a, T> {
    status: ObservationStatus,
    response_identity: Option<&'a str>,
    is_payload_empty: bool,
    reason_codes: &'a [T],
    path: &'a str,
}

struct CandidateDecisionInput<'a> {
    policy: &'a UpdatePolicy,
    candidate: &'a SourceCandidate,
    current: &'a NormalizedVersion,
    minimum: Option<&'a NormalizedVersion>,
    maximum: Option<&'a NormalizedVersion>,
    ignored: &'a BTreeSet<String>,
    ambiguity: &'a BTreeSet<String>,
    duplicates: &'a BTreeSet<SourceCandidate>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AppliedOutput {
    pub relative_path: String,
    pub output_digest_blake3: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct UpdateExecutionReceipt {
    pub schema: String,
    pub receipt_identity_blake3: String,
    pub plan_identity_blake3: String,
    pub disposition: UpdateExecutionDisposition,
    pub published_root: Option<String>,
    pub applied_outputs: Vec<AppliedOutput>,
    pub reason_codes: Vec<String>,
    pub non_claims: Vec<String>,
}

pub fn seal_update_policy(policy: &UpdatePolicy) -> Result<UpdatePolicy, CoreFailure> {
    let mut sealed = policy.clone();
    sealed.version_rules.ignored_versions.sort();
    sealed.version_rules.ignored_versions.dedup();
    validate_policy(&sealed)?;
    sealed.policy_identity_blake3 = update_policy_identity_blake3(&sealed)?;
    validate_policy(&sealed)?;
    Ok(sealed)
}

pub fn update_policy_identity_blake3(policy: &UpdatePolicy) -> Result<String, CoreFailure> {
    let mut preimage = policy.clone();
    preimage.policy_identity_blake3.clear();
    canonical_identity(POLICY_IDENTITY_DOMAIN, &preimage, "policy")
}

pub fn seal_source_observation(
    policy: &UpdatePolicy,
    observation: &SourceObservation,
) -> Result<SourceObservation, CoreFailure> {
    let sealed_policy = seal_update_policy(policy)?;
    let mut sealed = observation.clone();
    sealed.candidates.sort();
    normalize_reason_codes(&mut sealed.reason_codes);
    validate_source_observation(&sealed_policy, &sealed)?;
    sealed.observation_identity_blake3 = source_observation_identity_blake3(&sealed)?;
    validate_source_observation(&sealed_policy, &sealed)?;
    Ok(sealed)
}

pub fn source_observation_identity_blake3(observation: &SourceObservation) -> Result<String, CoreFailure> {
    let mut preimage = observation.clone();
    preimage.observation_identity_blake3.clear();
    canonical_identity(SOURCE_OBSERVATION_IDENTITY_DOMAIN, &preimage, "source-observation")
}

pub fn seal_advisory_observation(
    policy: &UpdatePolicy,
    observation: &AdvisoryObservation,
) -> Result<AdvisoryObservation, CoreFailure> {
    let sealed_policy = seal_update_policy(policy)?;
    let mut sealed = observation.clone();
    sealed.findings.sort();
    if sealed.findings.windows(DUPLICATE_WINDOW_SIZE).any(|window| window.first() == window.last()) {
        return Err(single_failure(DiagnosticInput {
            code: "update-duplicate-advisory-finding",
            path: "advisory_observation.findings",
            message: "advisory findings must be unique",
        }));
    }
    normalize_reason_codes(&mut sealed.reason_codes);
    validate_advisory_observation(&sealed_policy, &sealed)?;
    sealed.observation_identity_blake3 = advisory_observation_identity_blake3(&sealed)?;
    validate_advisory_observation(&sealed_policy, &sealed)?;
    Ok(sealed)
}

pub fn advisory_observation_identity_blake3(observation: &AdvisoryObservation) -> Result<String, CoreFailure> {
    let mut preimage = observation.clone();
    preimage.observation_identity_blake3.clear();
    canonical_identity(ADVISORY_OBSERVATION_IDENTITY_DOMAIN, &preimage, "advisory-observation")
}

pub fn build_update_plan(input: UpdatePlanInput<'_>) -> Result<UpdatePlan, CoreFailure> {
    let policy = seal_update_policy(input.policy)?;
    require_identity_equal(IdentityExpectation {
        path: "policy.policy_identity_blake3",
        expected: &policy.policy_identity_blake3,
        actual: &input.policy.policy_identity_blake3,
    })?;
    let source = seal_source_observation(&policy, input.source_observation)?;
    require_identity_equal(IdentityExpectation {
        path: "source_observation.observation_identity_blake3",
        expected: &source.observation_identity_blake3,
        actual: &input.source_observation.observation_identity_blake3,
    })?;
    let advisories = normalize_advisory_observations(&policy, input.advisory_observations)?;
    validate_validation_evidence(&policy, input.validation_evidence)?;
    let selection = select_candidate(&policy, &source)?;
    let effects = plan_mutation_effects(&policy, &selection, input.mutation_documents)?;
    let (proposal_status, reason_codes) =
        derive_proposal_status(&policy, &source, &selection, &advisories, input.validation_evidence)?;
    let advisory_ids = advisories
        .iter()
        .map(|observation| observation.observation_identity_blake3.clone())
        .collect::<Vec<_>>();
    let mut plan = UpdatePlan {
        schema: UPDATE_PLAN_SCHEMA.into(),
        plan_identity_blake3: String::new(),
        policy_identity_blake3: policy.policy_identity_blake3.clone(),
        source_observation_identity_blake3: source.observation_identity_blake3,
        advisory_observation_identity_blake3: advisory_ids,
        package: policy.package.clone(),
        selection,
        effects,
        validation_evidence: input.validation_evidence.clone(),
        proposal_status,
        reason_codes,
        non_claims: NON_CLAIMS.iter().map(|claim| (*claim).into()).collect(),
    };
    plan.plan_identity_blake3 = update_plan_identity_blake3(&plan)?;
    enforce_plan_byte_limit(&policy, &plan)?;
    debug_assert!(is_blake3(&plan.plan_identity_blake3));
    debug_assert!(u32::try_from(plan.effects.len()).is_ok_and(|count| count <= policy.limits.max_effects));
    Ok(plan)
}

pub fn update_plan_identity_blake3(plan: &UpdatePlan) -> Result<String, CoreFailure> {
    let mut preimage = plan.clone();
    preimage.plan_identity_blake3.clear();
    canonical_identity(PLAN_IDENTITY_DOMAIN, &preimage, "update-plan")
}

pub fn canonical_mutation_document_bytes(document: &Value) -> Result<Vec<u8>, CoreFailure> {
    let mut bytes = serde_json::to_vec_pretty(document).map_err(|error| {
        single_failure(DiagnosticInput {
            code: "update-json-serialization-failed",
            path: "mutation.document",
            message: &format!("serializing mutation document: {error}"),
        })
    })?;
    bytes.push(b'\n');
    Ok(bytes)
}

pub fn record_update_execution(
    plan: &UpdatePlan,
    disposition: UpdateExecutionDisposition,
    published_root: Option<String>,
    mut applied_outputs: Vec<AppliedOutput>,
    mut reason_codes: Vec<String>,
) -> Result<UpdateExecutionReceipt, CoreFailure> {
    validate_plan_identity(plan)?;
    applied_outputs.sort_by(|left, right| left.relative_path.cmp(&right.relative_path));
    normalize_reason_codes(&mut reason_codes);
    validate_execution_result(plan, disposition, &published_root, &applied_outputs, &reason_codes)?;
    let mut receipt = UpdateExecutionReceipt {
        schema: UPDATE_EXECUTION_RECEIPT_SCHEMA.into(),
        receipt_identity_blake3: String::new(),
        plan_identity_blake3: plan.plan_identity_blake3.clone(),
        disposition,
        published_root,
        applied_outputs,
        reason_codes,
        non_claims: NON_CLAIMS.iter().map(|claim| (*claim).into()).collect(),
    };
    receipt.receipt_identity_blake3 = execution_receipt_identity_blake3(&receipt)?;
    Ok(receipt)
}

pub fn execution_receipt_identity_blake3(receipt: &UpdateExecutionReceipt) -> Result<String, CoreFailure> {
    let mut preimage = receipt.clone();
    preimage.receipt_identity_blake3.clear();
    canonical_identity(EXECUTION_RECEIPT_IDENTITY_DOMAIN, &preimage, "execution-receipt")
}

fn validate_policy(policy: &UpdatePolicy) -> Result<(), CoreFailure> {
    let mut diagnostics = Vec::new();
    validate_policy_identity(policy, &mut diagnostics);
    validate_policy_source(policy, &mut diagnostics);
    validate_policy_rules(policy, &mut diagnostics);
    validate_ambient_authority(policy, &mut diagnostics);
    validate_limits(&policy.limits, &mut diagnostics);
    finish_policy_validation(policy, diagnostics)
}

fn validate_policy_identity(policy: &UpdatePolicy, diagnostics: &mut Vec<Diagnostic>) {
    let diagnostics_before = diagnostics.len();
    require_exact_text(
        TextExpectation {
            actual: &policy.schema,
            expected: UPDATE_POLICY_SCHEMA,
            path: "policy.schema",
        },
        diagnostics,
    );
    validate_optional_identity(
        TextField {
            value: &policy.policy_identity_blake3,
            path: "policy.policy_identity_blake3",
        },
        diagnostics,
    );
    validate_migration(&policy.migration, diagnostics);
    validate_package_key(&policy.package, diagnostics);
    require_nonempty(
        TextField {
            value: &policy.system,
            path: "policy.system",
        },
        diagnostics,
    );
    debug_assert!(diagnostics.len() >= diagnostics_before);
    debug_assert!(diagnostics.iter().skip(diagnostics_before).all(|item| !item.code.is_empty()));
}

fn validate_policy_source(policy: &UpdatePolicy, diagnostics: &mut Vec<Diagnostic>) {
    let diagnostics_before = diagnostics.len();
    require_nonempty(
        TextField {
            value: &policy.adapter_identity,
            path: "policy.adapter_identity",
        },
        diagnostics,
    );
    require_https_origin(
        TextField {
            value: &policy.source_authority,
            path: "policy.source_authority",
        },
        diagnostics,
    );
    require_relative_query(
        TextField {
            value: &policy.source_query,
            path: "policy.source_query",
        },
        diagnostics,
    );
    require_exact_text(
        TextExpectation {
            actual: &policy.response_schema,
            expected: response_schema_for_source(policy.source_kind),
            path: "policy.response_schema",
        },
        diagnostics,
    );
    require_nonempty(
        TextField {
            value: &policy.current_version,
            path: "policy.current_version",
        },
        diagnostics,
    );
    require_nonempty(
        TextField {
            value: &policy.current_source_ref,
            path: "policy.current_source_ref",
        },
        diagnostics,
    );
    require_blake3(
        TextField {
            value: &policy.current_source_identity_blake3,
            path: "policy.current_source_identity_blake3",
        },
        diagnostics,
    );
    debug_assert!(diagnostics.len() >= diagnostics_before);
    debug_assert!(diagnostics.iter().skip(diagnostics_before).all(|item| !item.code.is_empty()));
}

fn validate_policy_rules(policy: &UpdatePolicy, diagnostics: &mut Vec<Diagnostic>) {
    validate_version_rules(&policy.version_rules, diagnostics);
    validate_patch_policy(&policy.patch_policy, diagnostics);
    validate_advisory_policy(&policy.advisory_policy, diagnostics);
    validate_mutation_target(&policy.mutation, diagnostics);
}

fn validate_ambient_authority(policy: &UpdatePolicy, diagnostics: &mut Vec<Diagnostic>) {
    if policy.allow_ambient_credentials {
        diagnostics.push(diagnostic(DiagnosticInput {
            code: "update-ambient-credentials-forbidden",
            path: "policy.allow_ambient_credentials",
            message: "v1 update policy does not allow ambient credentials",
        }));
    }
    if policy.allow_ambient_proxy {
        diagnostics.push(diagnostic(DiagnosticInput {
            code: "update-ambient-proxy-forbidden",
            path: "policy.allow_ambient_proxy",
            message: "v1 update policy does not allow an ambient proxy",
        }));
    }
}

fn finish_policy_validation(policy: &UpdatePolicy, diagnostics: Vec<Diagnostic>) -> Result<(), CoreFailure> {
    if diagnostics.is_empty() {
        validate_policy_versions(policy)?;
        return Ok(());
    }
    Err(CoreFailure::from_diagnostics(diagnostics))
}

fn validate_migration(migration: &UpdatePolicyMigration, diagnostics: &mut Vec<Diagnostic>) {
    let is_fresh =
        migration.source_schema.is_none() && migration.source_policy_identity_blake3.is_none() && !migration.reviewed;
    if is_fresh {
        return;
    }
    let is_reviewed_migration = migration.source_schema.as_deref() == Some(UPDATE_MIGRATION_SOURCE_SCHEMA)
        && migration.source_policy_identity_blake3.as_deref().is_some_and(is_blake3)
        && migration.reviewed;
    if !is_reviewed_migration {
        diagnostics.push(diagnostic(DiagnosticInput {
            code: "update-policy-migration-invalid",
            path: "policy.migration",
            message: "migration facts must be absent or bind one reviewed v0 policy identity",
        }));
    }
}

fn validate_package_key(key: &ImpactPackageKey, diagnostics: &mut Vec<Diagnostic>) {
    require_nonempty(
        TextField {
            value: &key.public_selector,
            path: "policy.package.public_selector",
        },
        diagnostics,
    );
}

fn validate_version_rules(rules: &VersionRules, diagnostics: &mut Vec<Diagnostic>) {
    let diagnostics_before = diagnostics.len();
    if rules.component_count < VERSION_COMPONENT_COUNT_MIN || rules.component_count > VERSION_COMPONENT_COUNT_MAX {
        diagnostics.push(diagnostic(DiagnosticInput {
            code: "update-version-component-count-invalid",
            path: "policy.version_rules.component_count",
            message: "version component count is outside the supported range",
        }));
    }
    if rules.max_component == 0 {
        diagnostics.push(diagnostic(DiagnosticInput {
            code: "update-version-component-limit-invalid",
            path: "policy.version_rules.max_component",
            message: "version component limit must be positive",
        }));
    }
    for (index, ignored) in rules.ignored_versions.iter().enumerate() {
        require_nonempty(
            TextField {
                value: ignored,
                path: &format!("policy.version_rules.ignored_versions[{index}]"),
            },
            diagnostics,
        );
    }
    debug_assert!(diagnostics.len() >= diagnostics_before);
    debug_assert!(diagnostics.iter().skip(diagnostics_before).all(|item| !item.code.is_empty()));
}

fn validate_patch_policy(policy: &PatchPolicy, diagnostics: &mut Vec<Diagnostic>) {
    if !policy.allow_major && !policy.allow_minor && !policy.allow_patch {
        diagnostics.push(diagnostic(DiagnosticInput {
            code: "update-patch-policy-empty",
            path: "policy.patch_policy",
            message: "at least one update class must be allowed",
        }));
    }
}

fn validate_advisory_policy(policy: &AdvisoryPolicy, diagnostics: &mut Vec<Diagnostic>) {
    validate_advisory_requirement(&policy.osv, "policy.advisory_policy.osv", diagnostics);
    validate_advisory_requirement(&policy.repology, "policy.advisory_policy.repology", diagnostics);
}

fn validate_advisory_requirement(requirement: &AdvisoryRequirement, path: &str, diagnostics: &mut Vec<Diagnostic>) {
    let diagnostics_before = diagnostics.len();
    if requirement.mode == AdvisoryRequirementMode::Disabled {
        if !requirement.service_identity.is_empty()
            || !requirement.query.is_empty()
            || !requirement.package_coordinate.is_empty()
        {
            diagnostics.push(diagnostic(DiagnosticInput {
                code: "update-disabled-advisory-has-query",
                path,
                message: "a disabled advisory service must not contain query facts",
            }));
        }
        debug_assert!(diagnostics.len() >= diagnostics_before);
        debug_assert!(diagnostics.iter().skip(diagnostics_before).all(|item| !item.code.is_empty()));
        return;
    }
    require_nonempty(
        TextField {
            value: &requirement.service_identity,
            path: &format!("{path}.service_identity"),
        },
        diagnostics,
    );
    require_credential_free_https_url(
        TextField {
            value: &requirement.query,
            path: &format!("{path}.query"),
        },
        diagnostics,
    );
    require_nonempty(
        TextField {
            value: &requirement.package_coordinate,
            path: &format!("{path}.package_coordinate"),
        },
        diagnostics,
    );
    debug_assert!(diagnostics.len() >= diagnostics_before);
    debug_assert!(diagnostics.iter().skip(diagnostics_before).all(|item| !item.code.is_empty()));
}

fn validate_mutation_target(target: &UpdateMutationTarget, diagnostics: &mut Vec<Diagnostic>) {
    validate_mutation_path(&target.relative_path, diagnostics);
    let pointers = [
        ("version_pointer", target.version_pointer.as_str()),
        ("source_ref_pointer", target.source_ref_pointer.as_str()),
        ("source_identity_pointer", target.source_identity_pointer.as_str()),
    ];
    validate_mutation_pointers(pointers, diagnostics);
}

fn validate_mutation_path(relative_path: &str, diagnostics: &mut Vec<Diagnostic>) {
    if !is_safe_relative_path(relative_path) {
        diagnostics.push(diagnostic(DiagnosticInput {
            code: "update-unsafe-mutation-path",
            path: "policy.mutation.relative_path",
            message: "mutation path must be a safe relative path",
        }));
    }
}

fn validate_mutation_pointers(pointers: [(&str, &str); MUTATION_POINTER_COUNT], diagnostics: &mut Vec<Diagnostic>) {
    let mut unique = BTreeSet::new();
    for (name, pointer) in pointers {
        validate_mutation_pointer(MutationPointerInput { name, pointer }, diagnostics);
        if !unique.insert(pointer) {
            diagnostics.push(diagnostic(DiagnosticInput {
                code: "update-duplicate-field-pointer",
                path: "policy.mutation",
                message: "mutation field pointers must be unique",
            }));
        }
    }
}

fn validate_mutation_pointer(input: MutationPointerInput<'_>, diagnostics: &mut Vec<Diagnostic>) {
    if !is_safe_json_pointer(input.pointer) {
        diagnostics.push(diagnostic(DiagnosticInput {
            code: "update-unsafe-field-pointer",
            path: &format!("policy.mutation.{}", input.name),
            message: "mutation field must be one non-root JSON pointer",
        }));
    }
}

fn validate_limits(limits: &UpdateLimits, diagnostics: &mut Vec<Diagnostic>) {
    validate_byte_limits(limits, diagnostics);
    validate_count_limits(limits, diagnostics);
    validate_request_limits(limits, diagnostics);
}

fn validate_byte_limits(limits: &UpdateLimits, diagnostics: &mut Vec<Diagnostic>) {
    let diagnostics_before = diagnostics.len();
    validate_u64_limit(
        U64LimitInput {
            value: limits.max_response_bytes,
            maximum: HARD_MAX_RESPONSE_BYTES,
            path: "policy.limits.max_response_bytes",
        },
        diagnostics,
    );
    validate_u64_limit(
        U64LimitInput {
            value: limits.max_document_bytes,
            maximum: HARD_MAX_DOCUMENT_BYTES,
            path: "policy.limits.max_document_bytes",
        },
        diagnostics,
    );
    validate_u64_limit(
        U64LimitInput {
            value: limits.max_plan_bytes,
            maximum: HARD_MAX_PLAN_BYTES,
            path: "policy.limits.max_plan_bytes",
        },
        diagnostics,
    );
    debug_assert!(diagnostics.len() >= diagnostics_before);
    debug_assert!(diagnostics.iter().skip(diagnostics_before).all(|item| !item.code.is_empty()));
}

fn validate_count_limits(limits: &UpdateLimits, diagnostics: &mut Vec<Diagnostic>) {
    let diagnostics_before = diagnostics.len();
    validate_u32_limit(
        U32LimitInput {
            value: limits.max_candidates,
            maximum: HARD_MAX_CANDIDATES,
            path: "policy.limits.max_candidates",
        },
        diagnostics,
    );
    validate_u32_limit(
        U32LimitInput {
            value: limits.max_findings,
            maximum: HARD_MAX_FINDINGS,
            path: "policy.limits.max_findings",
        },
        diagnostics,
    );
    validate_u32_limit(
        U32LimitInput {
            value: limits.max_effects,
            maximum: HARD_MAX_EFFECTS,
            path: "policy.limits.max_effects",
        },
        diagnostics,
    );
    validate_u32_limit(
        U32LimitInput {
            value: limits.max_diagnostics,
            maximum: HARD_MAX_DIAGNOSTICS,
            path: "policy.limits.max_diagnostics",
        },
        diagnostics,
    );
    validate_u32_limit(
        U32LimitInput {
            value: limits.max_artifacts,
            maximum: HARD_MAX_ARTIFACTS,
            path: "policy.limits.max_artifacts",
        },
        diagnostics,
    );
    debug_assert!(diagnostics.len() >= diagnostics_before);
    debug_assert!(diagnostics.iter().skip(diagnostics_before).all(|item| !item.code.is_empty()));
}

fn validate_request_limits(limits: &UpdateLimits, diagnostics: &mut Vec<Diagnostic>) {
    let diagnostics_before = diagnostics.len();
    validate_bounded_zero_u32(
        U32LimitInput {
            value: limits.max_redirects,
            maximum: HARD_MAX_REDIRECTS,
            path: "policy.limits.max_redirects",
        },
        diagnostics,
    );
    validate_bounded_zero_u32(
        U32LimitInput {
            value: limits.max_retries,
            maximum: HARD_MAX_RETRIES,
            path: "policy.limits.max_retries",
        },
        diagnostics,
    );
    validate_u64_limit(
        U64LimitInput {
            value: limits.max_elapsed_millis,
            maximum: HARD_MAX_ELAPSED_MILLIS,
            path: "policy.limits.max_elapsed_millis",
        },
        diagnostics,
    );
    debug_assert!(diagnostics.len() >= diagnostics_before);
    debug_assert!(diagnostics.iter().skip(diagnostics_before).all(|item| !item.code.is_empty()));
}

fn validate_policy_versions(policy: &UpdatePolicy) -> Result<(), CoreFailure> {
    let current = normalize_version(&policy.version_rules, &policy.current_version)?;
    debug_assert!(!current.canonical.is_empty());
    debug_assert_eq!(current.components.len(), usize::from(policy.version_rules.component_count));
    if let Some(minimum) = &policy.version_rules.minimum_version {
        let minimum = normalize_version(&policy.version_rules, minimum)?;
        if compare_versions(&minimum, &current) == Ordering::Greater {
            return Err(single_failure(DiagnosticInput {
                code: "update-current-version-below-minimum",
                path: "policy.current_version",
                message: "current version is below the configured minimum",
            }));
        }
    }
    if let Some(maximum) = &policy.version_rules.maximum_version {
        let maximum = normalize_version(&policy.version_rules, maximum)?;
        if compare_versions(&current, &maximum) == Ordering::Greater {
            return Err(single_failure(DiagnosticInput {
                code: "update-current-version-above-maximum",
                path: "policy.current_version",
                message: "current version is above the configured maximum",
            }));
        }
    }
    for ignored in &policy.version_rules.ignored_versions {
        let _ = normalize_version(&policy.version_rules, ignored)?;
    }
    Ok(())
}

fn validate_source_observation(policy: &UpdatePolicy, observation: &SourceObservation) -> Result<(), CoreFailure> {
    let mut diagnostics = Vec::new();
    validate_source_observation_identity(policy, observation, &mut diagnostics);
    validate_source_observation_query(policy, observation, &mut diagnostics);
    validate_source_observation_payload(policy, observation, &mut diagnostics);
    finish_diagnostics(diagnostics)
}

fn validate_source_observation_identity(
    policy: &UpdatePolicy,
    observation: &SourceObservation,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let diagnostics_before = diagnostics.len();
    require_exact_text(
        TextExpectation {
            actual: &observation.schema,
            expected: UPDATE_SOURCE_OBSERVATION_SCHEMA,
            path: "source_observation.schema",
        },
        diagnostics,
    );
    validate_optional_identity(
        TextField {
            value: &observation.observation_identity_blake3,
            path: "source_observation.observation_identity_blake3",
        },
        diagnostics,
    );
    require_exact_text(
        TextExpectation {
            actual: &observation.policy_identity_blake3,
            expected: &policy.policy_identity_blake3,
            path: "source_observation.policy_identity_blake3",
        },
        diagnostics,
    );
    debug_assert!(diagnostics.len() >= diagnostics_before);
    debug_assert!(diagnostics.iter().skip(diagnostics_before).all(|item| !item.code.is_empty()));
}

fn validate_source_observation_query(
    policy: &UpdatePolicy,
    observation: &SourceObservation,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let diagnostics_before = diagnostics.len();
    require_exact_text(
        TextExpectation {
            actual: &observation.adapter_identity,
            expected: &policy.adapter_identity,
            path: "source_observation.adapter_identity",
        },
        diagnostics,
    );
    if observation.source_kind != policy.source_kind {
        diagnostics.push(diagnostic(DiagnosticInput {
            code: "update-source-kind-mismatch",
            path: "source_observation.source_kind",
            message: "source observation kind differs from policy",
        }));
    }
    require_exact_text(
        TextExpectation {
            actual: &observation.query,
            expected: &policy.source_query,
            path: "source_observation.query",
        },
        diagnostics,
    );
    require_exact_text(
        TextExpectation {
            actual: &observation.source_authority,
            expected: &policy.source_authority,
            path: "source_observation.source_authority",
        },
        diagnostics,
    );
    require_exact_text(
        TextExpectation {
            actual: &observation.response_schema,
            expected: &policy.response_schema,
            path: "source_observation.response_schema",
        },
        diagnostics,
    );
    debug_assert!(diagnostics.len() >= diagnostics_before);
    debug_assert!(diagnostics.iter().skip(diagnostics_before).all(|item| !item.code.is_empty()));
}

fn validate_source_observation_payload(
    policy: &UpdatePolicy,
    observation: &SourceObservation,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let diagnostics_before = diagnostics.len();
    validate_collection(&policy.limits, &observation.collection, "source_observation.collection", diagnostics);
    validate_count(
        CountInput {
            count: observation.candidates.len(),
            maximum: policy.limits.max_candidates,
            path: "source_observation.candidates",
            code: "update-candidate-limit-exceeded",
        },
        diagnostics,
    );
    validate_count(
        CountInput {
            count: observation.reason_codes.len(),
            maximum: policy.limits.max_diagnostics,
            path: "source_observation.reason_codes",
            code: "update-diagnostic-limit-exceeded",
        },
        diagnostics,
    );
    validate_status_payload(
        StatusPayloadInput {
            status: observation.status,
            response_identity: observation.response_identity_blake3.as_deref(),
            is_payload_empty: observation.candidates.is_empty(),
            reason_codes: &observation.reason_codes,
            path: "source_observation",
        },
        diagnostics,
    );
    for (index, candidate) in observation.candidates.iter().enumerate() {
        validate_source_candidate(candidate, index, diagnostics);
    }
    debug_assert!(diagnostics.len() >= diagnostics_before);
    debug_assert!(diagnostics.iter().skip(diagnostics_before).all(|item| !item.code.is_empty()));
}

fn validate_source_candidate(candidate: &SourceCandidate, index: usize, diagnostics: &mut Vec<Diagnostic>) {
    let diagnostics_before = diagnostics.len();
    require_nonempty(
        TextField {
            value: &candidate.version,
            path: &format!("source_observation.candidates[{index}].version"),
        },
        diagnostics,
    );
    require_nonempty(
        TextField {
            value: &candidate.source_ref,
            path: &format!("source_observation.candidates[{index}].source_ref"),
        },
        diagnostics,
    );
    require_blake3(
        TextField {
            value: &candidate.source_identity_blake3,
            path: &format!("source_observation.candidates[{index}].source_identity_blake3"),
        },
        diagnostics,
    );
    debug_assert!(diagnostics.len() >= diagnostics_before);
    debug_assert!(diagnostics.iter().skip(diagnostics_before).all(|item| !item.code.is_empty()));
}

fn validate_advisory_observation(policy: &UpdatePolicy, observation: &AdvisoryObservation) -> Result<(), CoreFailure> {
    let requirement = advisory_requirement(&policy.advisory_policy, observation.service);
    let mut diagnostics = Vec::new();
    validate_advisory_observation_identity(policy, observation, &mut diagnostics);
    validate_advisory_observation_query(requirement, observation, &mut diagnostics);
    validate_advisory_observation_payload(policy, observation, &mut diagnostics);
    finish_diagnostics(diagnostics)
}

fn validate_advisory_observation_identity(
    policy: &UpdatePolicy,
    observation: &AdvisoryObservation,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let diagnostics_before = diagnostics.len();
    require_exact_text(
        TextExpectation {
            actual: &observation.schema,
            expected: UPDATE_ADVISORY_OBSERVATION_SCHEMA,
            path: "advisory_observation.schema",
        },
        diagnostics,
    );
    validate_optional_identity(
        TextField {
            value: &observation.observation_identity_blake3,
            path: "advisory_observation.observation_identity_blake3",
        },
        diagnostics,
    );
    require_exact_text(
        TextExpectation {
            actual: &observation.policy_identity_blake3,
            expected: &policy.policy_identity_blake3,
            path: "advisory_observation.policy_identity_blake3",
        },
        diagnostics,
    );
    debug_assert!(diagnostics.len() >= diagnostics_before);
    debug_assert!(diagnostics.iter().skip(diagnostics_before).all(|item| !item.code.is_empty()));
}

fn validate_advisory_observation_query(
    requirement: &AdvisoryRequirement,
    observation: &AdvisoryObservation,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let diagnostics_before = diagnostics.len();
    if requirement.mode == AdvisoryRequirementMode::Disabled {
        diagnostics.push(diagnostic(DiagnosticInput {
            code: "update-disabled-advisory-observation",
            path: "advisory_observation.service",
            message: "policy disables this advisory service",
        }));
    }
    require_exact_text(
        TextExpectation {
            actual: &observation.service_identity,
            expected: &requirement.service_identity,
            path: "advisory_observation.service_identity",
        },
        diagnostics,
    );
    require_exact_text(
        TextExpectation {
            actual: &observation.query,
            expected: &requirement.query,
            path: "advisory_observation.query",
        },
        diagnostics,
    );
    require_exact_text(
        TextExpectation {
            actual: &observation.package_coordinate,
            expected: &requirement.package_coordinate,
            path: "advisory_observation.package_coordinate",
        },
        diagnostics,
    );
    require_exact_text(
        TextExpectation {
            actual: &observation.response_schema,
            expected: advisory_response_schema(observation.service),
            path: "advisory_observation.response_schema",
        },
        diagnostics,
    );
    debug_assert!(diagnostics.len() >= diagnostics_before);
    debug_assert!(diagnostics.iter().skip(diagnostics_before).all(|item| !item.code.is_empty()));
}

fn validate_advisory_observation_payload(
    policy: &UpdatePolicy,
    observation: &AdvisoryObservation,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let diagnostics_before = diagnostics.len();
    validate_collection(&policy.limits, &observation.collection, "advisory_observation.collection", diagnostics);
    validate_count(
        CountInput {
            count: observation.findings.len(),
            maximum: policy.limits.max_findings,
            path: "advisory_observation.findings",
            code: "update-finding-limit-exceeded",
        },
        diagnostics,
    );
    validate_count(
        CountInput {
            count: observation.reason_codes.len(),
            maximum: policy.limits.max_diagnostics,
            path: "advisory_observation.reason_codes",
            code: "update-diagnostic-limit-exceeded",
        },
        diagnostics,
    );
    validate_status_payload(
        StatusPayloadInput {
            status: observation.status,
            response_identity: observation.response_identity_blake3.as_deref(),
            is_payload_empty: observation.findings.is_empty(),
            reason_codes: &observation.reason_codes,
            path: "advisory_observation",
        },
        diagnostics,
    );
    for (index, finding) in observation.findings.iter().enumerate() {
        validate_advisory_finding(finding, index, diagnostics);
    }
    debug_assert!(diagnostics.len() >= diagnostics_before);
    debug_assert!(diagnostics.iter().skip(diagnostics_before).all(|item| !item.code.is_empty()));
}

fn validate_advisory_finding(finding: &AdvisoryFinding, index: usize, diagnostics: &mut Vec<Diagnostic>) {
    require_nonempty(
        TextField {
            value: &finding.finding_id,
            path: &format!("advisory_observation.findings[{index}].finding_id"),
        },
        diagnostics,
    );
    require_blake3(
        TextField {
            value: &finding.finding_identity_blake3,
            path: &format!("advisory_observation.findings[{index}].finding_identity_blake3"),
        },
        diagnostics,
    );
}

fn finish_diagnostics(diagnostics: Vec<Diagnostic>) -> Result<(), CoreFailure> {
    if diagnostics.is_empty() {
        return Ok(());
    }
    Err(CoreFailure::from_diagnostics(diagnostics))
}

fn validate_collection(
    limits: &UpdateLimits,
    collection: &CollectionFacts,
    path: &str,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let diagnostics_before = diagnostics.len();
    validate_observed_u64_limit(
        ObservedU64LimitInput {
            value: collection.response_bytes,
            maximum: limits.max_response_bytes,
            path: &format!("{path}.response_bytes"),
            code: "update-response-byte-limit-exceeded",
            message: "response byte count exceeds policy",
        },
        diagnostics,
    );
    validate_observed_u32_limit(
        ObservedU32LimitInput {
            value: collection.redirect_count,
            maximum: limits.max_redirects,
            path: &format!("{path}.redirect_count"),
            code: "update-redirect-limit-exceeded",
            message: "redirect count exceeds policy",
        },
        diagnostics,
    );
    validate_observed_u32_limit(
        ObservedU32LimitInput {
            value: collection.retry_count,
            maximum: limits.max_retries,
            path: &format!("{path}.retry_count"),
            code: "update-retry-limit-exceeded",
            message: "retry count exceeds policy",
        },
        diagnostics,
    );
    validate_observed_u64_limit(
        ObservedU64LimitInput {
            value: collection.elapsed_millis,
            maximum: limits.max_elapsed_millis,
            path: &format!("{path}.elapsed_millis"),
            code: "update-elapsed-limit-exceeded",
            message: "elapsed time exceeds policy",
        },
        diagnostics,
    );
    debug_assert!(diagnostics.len() >= diagnostics_before);
    debug_assert!(diagnostics.iter().skip(diagnostics_before).all(|item| !item.code.is_empty()));
}

fn validate_observed_u64_limit(input: ObservedU64LimitInput<'_>, diagnostics: &mut Vec<Diagnostic>) {
    if input.value > input.maximum {
        diagnostics.push(diagnostic(DiagnosticInput {
            code: input.code,
            path: input.path,
            message: input.message,
        }));
    }
}

fn validate_observed_u32_limit(input: ObservedU32LimitInput<'_>, diagnostics: &mut Vec<Diagnostic>) {
    if input.value > input.maximum {
        diagnostics.push(diagnostic(DiagnosticInput {
            code: input.code,
            path: input.path,
            message: input.message,
        }));
    }
}

fn validate_status_payload<T>(input: StatusPayloadInput<'_, T>, diagnostics: &mut Vec<Diagnostic>) {
    match input.status {
        ObservationStatus::Success => validate_success_payload(&input, diagnostics),
        ObservationStatus::Unavailable | ObservationStatus::Failed => {
            validate_nonsuccess_payload(&input, diagnostics);
        }
    }
}

fn validate_success_payload<T>(input: &StatusPayloadInput<'_, T>, diagnostics: &mut Vec<Diagnostic>) {
    if input.response_identity.is_none_or(|identity| !is_blake3(identity)) {
        diagnostics.push(diagnostic(DiagnosticInput {
            code: "update-success-response-identity-missing",
            path: &format!("{}.response_identity_blake3", input.path),
            message: "successful observation requires one response identity",
        }));
    }
    if !input.reason_codes.is_empty() {
        diagnostics.push(diagnostic(DiagnosticInput {
            code: "update-success-has-failure-reasons",
            path: &format!("{}.reason_codes", input.path),
            message: "successful observation must not contain failure reasons",
        }));
    }
}

fn validate_nonsuccess_payload<T>(input: &StatusPayloadInput<'_, T>, diagnostics: &mut Vec<Diagnostic>) {
    let diagnostics_before = diagnostics.len();
    if !input.is_payload_empty {
        diagnostics.push(diagnostic(DiagnosticInput {
            code: "update-failed-observation-has-success-payload",
            path: input.path,
            message: "unavailable or failed observation must not contain success payload",
        }));
    }
    if input.reason_codes.is_empty() {
        diagnostics.push(diagnostic(DiagnosticInput {
            code: "update-failed-observation-reason-missing",
            path: &format!("{}.reason_codes", input.path),
            message: "unavailable or failed observation requires one reason",
        }));
    }
    if input.response_identity.is_some_and(|identity| !is_blake3(identity)) {
        diagnostics.push(diagnostic(DiagnosticInput {
            code: "update-response-identity-invalid",
            path: &format!("{}.response_identity_blake3", input.path),
            message: "response identity must be a lowercase BLAKE3 digest",
        }));
    }
    debug_assert!(diagnostics.len() >= diagnostics_before);
    debug_assert!(diagnostics.iter().skip(diagnostics_before).all(|item| !item.code.is_empty()));
}

fn normalize_advisory_observations(
    policy: &UpdatePolicy,
    observations: &[AdvisoryObservation],
) -> Result<Vec<AdvisoryObservation>, CoreFailure> {
    let observation_count = u32::try_from(observations.len()).map_err(|_| {
        single_failure(DiagnosticInput {
            code: "update-advisory-observation-count-overflow",
            path: "advisory_observations",
            message: "advisory observation count exceeds u32",
        })
    })?;
    if observation_count > ADVISORY_SERVICE_COUNT_MAX {
        return Err(single_failure(DiagnosticInput {
            code: "update-advisory-observation-limit-exceeded",
            path: "advisory_observations",
            message: "advisory observations exceed the supported service count",
        }));
    }
    let mut osv = None;
    let mut repology = None;
    for observation in observations {
        let sealed = seal_advisory_observation(policy, observation)?;
        require_identity_equal(IdentityExpectation {
            path: "advisory_observation.observation_identity_blake3",
            expected: &sealed.observation_identity_blake3,
            actual: &observation.observation_identity_blake3,
        })?;
        match sealed.service {
            AdvisoryService::Osv => insert_unique_advisory(&mut osv, sealed)?,
            AdvisoryService::Repology => insert_unique_advisory(&mut repology, sealed)?,
        }
    }
    let mut normalized = Vec::with_capacity(ADVISORY_SERVICE_COUNT);
    normalized.extend(osv);
    normalized.extend(repology);
    debug_assert!(normalized.len() <= observations.len());
    debug_assert!(normalized.iter().all(|item| is_blake3(&item.observation_identity_blake3)));
    Ok(normalized)
}

fn insert_unique_advisory(
    slot: &mut Option<AdvisoryObservation>,
    observation: AdvisoryObservation,
) -> Result<(), CoreFailure> {
    if let Some(existing) = slot {
        let code = if existing == &observation {
            "update-duplicate-advisory-observation"
        } else {
            "update-conflicting-advisory-observation"
        };
        return Err(single_failure(DiagnosticInput {
            code,
            path: "advisory_observations",
            message: "one advisory service was supplied more than once",
        }));
    }
    *slot = Some(observation);
    debug_assert!(slot.is_some());
    debug_assert!(slot.as_ref().is_some_and(|item| is_blake3(&item.observation_identity_blake3)));
    Ok(())
}

fn select_candidate(policy: &UpdatePolicy, observation: &SourceObservation) -> Result<CandidateSelection, CoreFailure> {
    if observation.status != ObservationStatus::Success {
        return Ok(CandidateSelection {
            selected: None,
            decisions: Vec::new(),
            reason_codes: vec![match observation.status {
                ObservationStatus::Unavailable => "source-observation-unavailable".into(),
                ObservationStatus::Failed => "source-observation-failed".into(),
                ObservationStatus::Success => unreachable!(),
            }],
        });
    }
    let current = normalize_version(&policy.version_rules, &policy.current_version)?;
    let minimum = optional_normalized_version(&policy.version_rules, policy.version_rules.minimum_version.as_deref())?;
    let maximum = optional_normalized_version(&policy.version_rules, policy.version_rules.maximum_version.as_deref())?;
    let ignored = normalized_ignored_versions(&policy.version_rules)?;
    let ambiguity = ambiguous_versions(&policy.version_rules, &observation.candidates);
    let duplicates = duplicate_candidates(&observation.candidates);
    let mut decisions = observation
        .candidates
        .iter()
        .map(|candidate| {
            candidate_decision(CandidateDecisionInput {
                policy,
                candidate,
                current: &current,
                minimum: minimum.as_ref(),
                maximum: maximum.as_ref(),
                ignored: &ignored,
                ambiguity: &ambiguity,
                duplicates: &duplicates,
            })
        })
        .collect::<Vec<_>>();
    decisions.sort_by(|left, right| left.candidate.cmp(&right.candidate));
    let mut eligible = decisions
        .iter()
        .filter(|decision| decision.eligible)
        .filter_map(|decision| {
            let normalized = normalize_version(&policy.version_rules, &decision.candidate.version).ok()?;
            Some((normalized, decision.candidate.clone()))
        })
        .collect::<Vec<_>>();
    eligible.sort_by(|left, right| {
        compare_versions(&left.0, &right.0)
            .then_with(|| left.1.source_identity_blake3.cmp(&right.1.source_identity_blake3))
    });
    let selected = eligible.last().map(|(_, candidate)| candidate.clone());
    let reason_codes = if selected.is_some() {
        Vec::new()
    } else {
        vec!["no-eligible-candidate".into()]
    };
    debug_assert_eq!(reason_codes.is_empty(), selected.is_some());
    debug_assert!(
        selected
            .as_ref()
            .is_none_or(|candidate| { decisions.iter().any(|item| item.eligible && item.candidate == *candidate) })
    );
    Ok(CandidateSelection {
        selected,
        decisions,
        reason_codes,
    })
}

fn candidate_decision(input: CandidateDecisionInput<'_>) -> CandidateDecision {
    let policy = input.policy;
    let candidate = input.candidate;
    let current = input.current;
    let normalized = normalize_version(&policy.version_rules, &candidate.version);
    let mut reasons = Vec::new();
    let normalized = match normalized {
        Ok(normalized) => normalized,
        Err(_) => {
            reasons.push("candidate-version-malformed".into());
            return CandidateDecision {
                candidate: candidate.clone(),
                normalized_version: None,
                eligible: false,
                reason_codes: reasons,
            };
        }
    };
    if input.ambiguity.contains(&normalized.canonical) {
        reasons.push("candidate-version-ambiguous".into());
    }
    if input.duplicates.contains(candidate) {
        reasons.push("candidate-duplicate".into());
    }
    if compare_versions(&normalized, current) != Ordering::Greater {
        reasons.push("candidate-not-newer".into());
    }
    if input.ignored.contains(&normalized.canonical) {
        reasons.push("candidate-version-ignored".into());
    }
    if normalized.prerelease.is_some() && !policy.version_rules.allow_prerelease {
        reasons.push("candidate-prerelease-disallowed".into());
    }
    if input.minimum.is_some_and(|minimum| compare_versions(&normalized, minimum) == Ordering::Less) {
        reasons.push("candidate-below-minimum".into());
    }
    if input.maximum.is_some_and(|maximum| compare_versions(&normalized, maximum) == Ordering::Greater) {
        reasons.push("candidate-above-maximum".into());
    }
    append_development_reasons(&policy.version_rules, &normalized, &mut reasons);
    append_patch_reasons(&policy.patch_policy, current, &normalized, &mut reasons);
    normalize_reason_codes(&mut reasons);
    let is_eligible = reasons.is_empty();
    debug_assert_eq!(is_eligible, reasons.is_empty());
    debug_assert!(!normalized.canonical.is_empty());
    CandidateDecision {
        candidate: candidate.clone(),
        normalized_version: Some(normalized.canonical),
        eligible: is_eligible,
        reason_codes: reasons,
    }
}

fn append_development_reasons(rules: &VersionRules, version: &NormalizedVersion, reasons: &mut Vec<String>) {
    if rules.odd_minor_is_development
        && version
            .components
            .get(VERSION_MINOR_INDEX)
            .is_some_and(|minor| minor % DEVELOPMENT_ODD_DIVISOR == 1)
    {
        reasons.push("candidate-development-minor".into());
    }
    if rules
        .high_patch_development_from
        .is_some_and(|threshold| version.components.get(VERSION_PATCH_INDEX).is_some_and(|patch| *patch >= threshold))
    {
        reasons.push("candidate-development-patch".into());
    }
}

fn append_patch_reasons(
    policy: &PatchPolicy,
    current: &NormalizedVersion,
    candidate: &NormalizedVersion,
    reasons: &mut Vec<String>,
) {
    let is_major_changed = current.components.first() != candidate.components.first();
    let is_minor_changed = current.components.get(VERSION_MINOR_INDEX) != candidate.components.get(VERSION_MINOR_INDEX);
    if is_major_changed && !policy.allow_major {
        reasons.push("candidate-major-update-disallowed".into());
        return;
    }
    if !is_major_changed && is_minor_changed && !policy.allow_minor {
        reasons.push("candidate-minor-update-disallowed".into());
        return;
    }
    if !is_major_changed && !is_minor_changed && !policy.allow_patch {
        reasons.push("candidate-patch-update-disallowed".into());
    }
}

fn duplicate_candidates(candidates: &[SourceCandidate]) -> BTreeSet<SourceCandidate> {
    let mut seen = BTreeSet::new();
    let mut duplicates = BTreeSet::new();
    for candidate in candidates {
        if !seen.insert(candidate) {
            duplicates.insert(candidate.clone());
        }
    }
    duplicates
}

fn ambiguous_versions(rules: &VersionRules, candidates: &[SourceCandidate]) -> BTreeSet<String> {
    let mut keyed = BTreeMap::<String, BTreeSet<(String, String)>>::new();
    for candidate in candidates {
        if let Ok(normalized) = normalize_version(rules, &candidate.version) {
            keyed
                .entry(normalized.canonical)
                .or_default()
                .insert((candidate.source_ref.clone(), candidate.source_identity_blake3.clone()));
        }
    }
    keyed.into_iter().filter_map(|(version, sources)| (sources.len() > 1).then_some(version)).collect()
}

fn optional_normalized_version(
    rules: &VersionRules,
    value: Option<&str>,
) -> Result<Option<NormalizedVersion>, CoreFailure> {
    value.map(|version| normalize_version(rules, version)).transpose()
}

fn normalized_ignored_versions(rules: &VersionRules) -> Result<BTreeSet<String>, CoreFailure> {
    rules
        .ignored_versions
        .iter()
        .map(|version| normalize_version(rules, version).map(|normalized| normalized.canonical))
        .collect()
}

fn normalize_version(rules: &VersionRules, raw: &str) -> Result<NormalizedVersion, CoreFailure> {
    let unprefixed = raw.strip_prefix(&rules.required_prefix).ok_or_else(|| {
        single_failure(DiagnosticInput {
            code: "update-version-prefix-mismatch",
            path: "version",
            message: "version does not have the required prefix",
        })
    })?;
    let (release, prerelease) = match unprefixed.split_once('-') {
        Some((release, prerelease)) => (release, Some(prerelease)),
        None => (unprefixed, None),
    };
    if prerelease.is_some_and(|value| !valid_prerelease(value)) {
        return Err(single_failure(DiagnosticInput {
            code: "update-version-prerelease-invalid",
            path: "version",
            message: "prerelease contains unsupported characters",
        }));
    }
    let parts = release.split('.').collect::<Vec<_>>();
    let expected_count = usize::from(rules.component_count);
    if parts.len() != expected_count {
        return Err(single_failure(DiagnosticInput {
            code: "update-version-component-count-mismatch",
            path: "version",
            message: "version component count differs from policy",
        }));
    }
    let mut components = Vec::with_capacity(parts.len());
    for part in parts {
        if part.is_empty() || (part.len() > 1 && part.starts_with('0')) {
            return Err(single_failure(DiagnosticInput {
                code: "update-version-component-invalid",
                path: "version",
                message: "version component is empty or has an ambiguous leading zero",
            }));
        }
        let component = part.parse::<u64>().map_err(|_| {
            single_failure(DiagnosticInput {
                code: "update-version-component-invalid",
                path: "version",
                message: "version component is not an unsigned integer",
            })
        })?;
        if component > rules.max_component {
            return Err(single_failure(DiagnosticInput {
                code: "update-version-component-limit-exceeded",
                path: "version",
                message: "version component exceeds policy",
            }));
        }
        components.push(component);
    }
    let mut canonical = components.iter().map(u64::to_string).collect::<Vec<_>>().join(".");
    if let Some(prerelease) = prerelease {
        canonical.push('-');
        canonical.push_str(prerelease);
    }
    debug_assert_eq!(components.len(), expected_count);
    debug_assert!(components.iter().all(|component| *component <= rules.max_component));
    Ok(NormalizedVersion {
        canonical,
        components,
        prerelease: prerelease.map(String::from),
    })
}

fn valid_prerelease(value: &str) -> bool {
    !value.is_empty() && value.bytes().all(|byte| byte.is_ascii_alphanumeric() || byte == b'.' || byte == b'-')
}

fn compare_versions(left: &NormalizedVersion, right: &NormalizedVersion) -> Ordering {
    left.components.cmp(&right.components).then_with(|| match (&left.prerelease, &right.prerelease) {
        (None, None) => Ordering::Equal,
        (None, Some(_)) => Ordering::Greater,
        (Some(_), None) => Ordering::Less,
        (Some(left), Some(right)) => left.cmp(right),
    })
}

fn plan_mutation_effects(
    policy: &UpdatePolicy,
    selection: &CandidateSelection,
    documents: &[MutationDocument],
) -> Result<Vec<DocumentMutationEffect>, CoreFailure> {
    let Some(candidate) = &selection.selected else {
        return Ok(Vec::new());
    };
    if documents.len() != EXPECTED_MUTATION_DOCUMENT_COUNT {
        return Err(single_failure(DiagnosticInput {
            code: "update-mutation-document-count-invalid",
            path: "mutation_documents",
            message: "the v1 policy requires exactly one mutation document",
        }));
    }
    let document = documents.first().ok_or_else(|| {
        single_failure(DiagnosticInput {
            code: "update-mutation-document-missing",
            path: "mutation_documents",
            message: "policy mutation target has no input document",
        })
    })?;
    validate_mutation_document(policy, document)?;
    if document.relative_path != policy.mutation.relative_path {
        return Err(single_failure(DiagnosticInput {
            code: "update-mutation-document-target-mismatch",
            path: "mutation_documents",
            message: "mutation document path differs from the policy target",
        }));
    }
    let effect = plan_document_effect(policy, document, candidate)?;
    debug_assert_eq!(effect.relative_path, policy.mutation.relative_path);
    debug_assert_eq!(effect.edits.len(), MUTATION_POINTER_COUNT);
    Ok(vec![effect])
}

fn validate_mutation_document(policy: &UpdatePolicy, document: &MutationDocument) -> Result<(), CoreFailure> {
    if document.schema != UPDATE_MUTATION_DOCUMENT_SCHEMA {
        return Err(single_failure(DiagnosticInput {
            code: "update-mutation-document-schema-mismatch",
            path: "mutation_document.schema",
            message: "mutation document schema is unsupported",
        }));
    }
    if !is_safe_relative_path(&document.relative_path) {
        return Err(single_failure(DiagnosticInput {
            code: "update-unsafe-mutation-path",
            path: "mutation_document.relative_path",
            message: "mutation path must be a safe relative path",
        }));
    }
    require_blake3_result(TextField {
        value: &document.input_digest_blake3,
        path: "mutation_document.input_digest_blake3",
    })?;
    if document.input_bytes > policy.limits.max_document_bytes {
        return Err(single_failure(DiagnosticInput {
            code: "update-document-byte-limit-exceeded",
            path: "mutation_document.input_bytes",
            message: "mutation document exceeds policy",
        }));
    }
    debug_assert_eq!(document.schema, UPDATE_MUTATION_DOCUMENT_SCHEMA);
    debug_assert!(is_safe_relative_path(&document.relative_path));
    Ok(())
}

fn plan_document_effect(
    policy: &UpdatePolicy,
    document: &MutationDocument,
    candidate: &SourceCandidate,
) -> Result<DocumentMutationEffect, CoreFailure> {
    let mut output = document.document.clone();
    let edits = [
        (
            policy.mutation.version_pointer.as_str(),
            Value::String(policy.current_version.clone()),
            Value::String(candidate.version.clone()),
            "update-version-field",
        ),
        (
            policy.mutation.source_ref_pointer.as_str(),
            Value::String(policy.current_source_ref.clone()),
            Value::String(candidate.source_ref.clone()),
            "update-source-ref-field",
        ),
        (
            policy.mutation.source_identity_pointer.as_str(),
            Value::String(policy.current_source_identity_blake3.clone()),
            Value::String(candidate.source_identity_blake3.clone()),
            "update-source-identity-field",
        ),
    ];
    let mut planned = Vec::with_capacity(MUTATION_POINTER_COUNT);
    for (pointer, expected_old, new_value, reason_code) in edits {
        let field = output.pointer_mut(pointer).ok_or_else(|| {
            single_failure(DiagnosticInput {
                code: "update-mutation-field-missing",
                path: pointer,
                message: "mutation field does not exist in the document",
            })
        })?;
        if *field != expected_old {
            return Err(single_failure(DiagnosticInput {
                code: "update-mutation-old-value-mismatch",
                path: pointer,
                message: "mutation field differs from the policy preimage",
            }));
        }
        let old_value = field.clone();
        *field = new_value.clone();
        planned.push(StructuredEdit {
            field_pointer: pointer.into(),
            old_value,
            new_value,
            reason_code: reason_code.into(),
        });
    }
    let output_digest_blake3 = mutation_output_digest(policy, &output)?;
    debug_assert_eq!(planned.len(), MUTATION_POINTER_COUNT);
    debug_assert!(is_blake3(&output_digest_blake3));
    Ok(DocumentMutationEffect {
        relative_path: document.relative_path.clone(),
        input_digest_blake3: document.input_digest_blake3.clone(),
        output_digest_blake3,
        edits: planned,
        output_document: output,
    })
}

fn mutation_output_digest(policy: &UpdatePolicy, output: &Value) -> Result<String, CoreFailure> {
    let output_bytes = canonical_mutation_document_bytes(output)?;
    let output_len = u64::try_from(output_bytes.len()).map_err(|_| {
        single_failure(DiagnosticInput {
            code: "update-document-byte-count-overflow",
            path: "mutation_document.output",
            message: "output byte count exceeds u64",
        })
    })?;
    if output_len > policy.limits.max_document_bytes {
        return Err(single_failure(DiagnosticInput {
            code: "update-document-byte-limit-exceeded",
            path: "mutation_document.output",
            message: "mutation output exceeds policy",
        }));
    }
    let digest = blake3::hash(&output_bytes).to_hex().to_string();
    debug_assert!(output_len <= policy.limits.max_document_bytes);
    debug_assert!(is_blake3(&digest));
    Ok(digest)
}

fn validate_validation_evidence(policy: &UpdatePolicy, evidence: &UpdateValidationEvidence) -> Result<(), CoreFailure> {
    let mut diagnostics = Vec::new();
    validate_validation_evidence_identity(evidence, &mut diagnostics);
    let links = [
        ("validation_evidence.catalog", &evidence.catalog),
        ("validation_evidence.build_observations", &evidence.build_observations),
        ("validation_evidence.validation_roots", &evidence.validation_roots),
        ("validation_evidence.impact_report", &evidence.impact_report),
    ];
    debug_assert_eq!(links.len(), VALIDATION_LINK_COUNT);
    for (path, linked) in links {
        validate_linked_evidence(&policy.limits, linked, path, &mut diagnostics);
    }
    finish_diagnostics(diagnostics)
}

fn validate_validation_evidence_identity(evidence: &UpdateValidationEvidence, diagnostics: &mut Vec<Diagnostic>) {
    let diagnostics_before = diagnostics.len();
    require_exact_text(
        TextExpectation {
            actual: &evidence.schema,
            expected: UPDATE_VALIDATION_EVIDENCE_SCHEMA,
            path: "validation_evidence.schema",
        },
        diagnostics,
    );
    require_nonempty(
        TextField {
            value: &evidence.candidate_version,
            path: "validation_evidence.candidate_version",
        },
        diagnostics,
    );
    require_blake3(
        TextField {
            value: &evidence.candidate_source_identity_blake3,
            path: "validation_evidence.candidate_source_identity_blake3",
        },
        diagnostics,
    );
    debug_assert!(diagnostics.len() >= diagnostics_before);
    debug_assert!(diagnostics.iter().skip(diagnostics_before).all(|item| !item.code.is_empty()));
}

fn validate_linked_evidence(
    limits: &UpdateLimits,
    evidence: &LinkedEvidence,
    path: &str,
    diagnostics: &mut Vec<Diagnostic>,
) {
    validate_linked_evidence_limits(limits, evidence, path, diagnostics);
    validate_linked_evidence_identities(evidence, path, diagnostics);
    validate_linked_evidence_shape(evidence, path, diagnostics);
}

fn validate_linked_evidence_limits(
    limits: &UpdateLimits,
    evidence: &LinkedEvidence,
    path: &str,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let diagnostics_before = diagnostics.len();
    validate_count(
        CountInput {
            count: evidence.artifact_identity_blake3.len(),
            maximum: limits.max_artifacts,
            path: &format!("{path}.artifact_identity_blake3"),
            code: "update-artifact-limit-exceeded",
        },
        diagnostics,
    );
    validate_count(
        CountInput {
            count: evidence.reason_codes.len(),
            maximum: limits.max_diagnostics,
            path: &format!("{path}.reason_codes"),
            code: "update-diagnostic-limit-exceeded",
        },
        diagnostics,
    );
    debug_assert!(diagnostics.len() >= diagnostics_before);
    debug_assert!(diagnostics.iter().skip(diagnostics_before).all(|item| !item.code.is_empty()));
}

fn validate_linked_evidence_identities(evidence: &LinkedEvidence, path: &str, diagnostics: &mut Vec<Diagnostic>) {
    for (index, identity) in evidence.artifact_identity_blake3.iter().enumerate() {
        require_blake3(
            TextField {
                value: identity,
                path: &format!("{path}.artifact_identity_blake3[{index}]"),
            },
            diagnostics,
        );
    }
}

fn validate_linked_evidence_shape(evidence: &LinkedEvidence, path: &str, diagnostics: &mut Vec<Diagnostic>) {
    match evidence.status {
        EvidenceStatus::Success => validate_success_evidence_shape(evidence, path, diagnostics),
        EvidenceStatus::Missing | EvidenceStatus::Unavailable | EvidenceStatus::Failed => {
            validate_nonsuccess_evidence_shape(evidence, path, diagnostics);
        }
    }
}

fn validate_success_evidence_shape(evidence: &LinkedEvidence, path: &str, diagnostics: &mut Vec<Diagnostic>) {
    if evidence.artifact_identity_blake3.is_empty() {
        diagnostics.push(diagnostic(DiagnosticInput {
            code: "update-success-evidence-artifact-missing",
            path,
            message: "successful linked evidence requires one artifact identity",
        }));
    }
    if !evidence.reason_codes.is_empty() {
        diagnostics.push(diagnostic(DiagnosticInput {
            code: "update-success-evidence-has-reasons",
            path,
            message: "successful linked evidence must not have failure reasons",
        }));
    }
}

fn validate_nonsuccess_evidence_shape(evidence: &LinkedEvidence, path: &str, diagnostics: &mut Vec<Diagnostic>) {
    if !evidence.artifact_identity_blake3.is_empty() {
        diagnostics.push(diagnostic(DiagnosticInput {
            code: "update-nonsuccess-evidence-has-artifacts",
            path,
            message: "non-success linked evidence must not claim accepted artifacts",
        }));
    }
    if evidence.reason_codes.is_empty() {
        diagnostics.push(diagnostic(DiagnosticInput {
            code: "update-nonsuccess-evidence-reason-missing",
            path,
            message: "non-success linked evidence requires one reason",
        }));
    }
}

fn derive_proposal_status(
    policy: &UpdatePolicy,
    source: &SourceObservation,
    selection: &CandidateSelection,
    advisories: &[AdvisoryObservation],
    validation: &UpdateValidationEvidence,
) -> Result<(UpdateProposalStatus, Vec<String>), CoreFailure> {
    if source.status == ObservationStatus::Unavailable {
        return Ok((UpdateProposalStatus::Unavailable, vec!["source-observation-unavailable".into()]));
    }
    if source.status == ObservationStatus::Failed {
        return Ok((UpdateProposalStatus::Failed, vec!["source-observation-failed".into()]));
    }
    let Some(candidate) = &selection.selected else {
        return Ok((UpdateProposalStatus::NoCandidate, vec!["no-eligible-candidate".into()]));
    };
    if validation.candidate_version != candidate.version
        || validation.candidate_source_identity_blake3 != candidate.source_identity_blake3
    {
        return Err(single_failure(DiagnosticInput {
            code: "update-validation-candidate-mismatch",
            path: "validation_evidence",
            message: "validation evidence is stale for the selected candidate",
        }));
    }
    let mut unavailable = Vec::new();
    let mut failed = Vec::new();
    let mut blocked = Vec::new();
    classify_advisories(policy, advisories, &mut unavailable, &mut failed, &mut blocked);
    classify_validation(policy, validation, &mut unavailable, &mut failed, &mut blocked);
    if !failed.is_empty() {
        normalize_reason_codes(&mut failed);
        return Ok((UpdateProposalStatus::Failed, failed));
    }
    if !unavailable.is_empty() {
        normalize_reason_codes(&mut unavailable);
        return Ok((UpdateProposalStatus::Unavailable, unavailable));
    }
    if !blocked.is_empty() {
        normalize_reason_codes(&mut blocked);
        return Ok((UpdateProposalStatus::Blocked, blocked));
    }
    debug_assert!(failed.is_empty());
    debug_assert!(unavailable.is_empty());
    Ok((UpdateProposalStatus::ReadyForReview, Vec::new()))
}

fn classify_advisories(
    policy: &UpdatePolicy,
    advisories: &[AdvisoryObservation],
    unavailable: &mut Vec<String>,
    failed: &mut Vec<String>,
    blocked: &mut Vec<String>,
) {
    let unavailable_before = unavailable.len();
    let failed_before = failed.len();
    let keyed = advisories.iter().map(|observation| (observation.service, observation)).collect::<BTreeMap<_, _>>();
    for service in [AdvisoryService::Osv, AdvisoryService::Repology] {
        let requirement = advisory_requirement(&policy.advisory_policy, service);
        if requirement.mode == AdvisoryRequirementMode::Disabled {
            continue;
        }
        let label = advisory_service_label(service);
        let Some(observation) = keyed.get(&service) else {
            if requirement.mode == AdvisoryRequirementMode::Required {
                blocked.push(format!("{label}-observation-missing"));
            }
            continue;
        };
        match observation.status {
            ObservationStatus::Success => {
                if policy.advisory_policy.block_on_findings && !observation.findings.is_empty() {
                    blocked.push(format!("{label}-findings-require-review"));
                }
            }
            ObservationStatus::Unavailable => {
                if requirement.mode == AdvisoryRequirementMode::Required {
                    unavailable.push(format!("{label}-observation-unavailable"));
                }
            }
            ObservationStatus::Failed => {
                if requirement.mode == AdvisoryRequirementMode::Required {
                    failed.push(format!("{label}-observation-failed"));
                }
            }
        }
    }
    debug_assert!(unavailable.len() >= unavailable_before);
    debug_assert!(failed.len() >= failed_before);
}

fn classify_validation(
    policy: &UpdatePolicy,
    validation: &UpdateValidationEvidence,
    unavailable: &mut Vec<String>,
    failed: &mut Vec<String>,
    blocked: &mut Vec<String>,
) {
    let unavailable_before = unavailable.len();
    let failed_before = failed.len();
    let requirements = [
        (policy.validation_policy.require_catalog, "catalog", &validation.catalog),
        (
            policy.validation_policy.require_build_observations,
            "build-observations",
            &validation.build_observations,
        ),
        (policy.validation_policy.require_validation_roots, "validation-roots", &validation.validation_roots),
        (policy.validation_policy.require_impact_report, "impact-report", &validation.impact_report),
    ];
    for (required, label, evidence) in requirements {
        if !required {
            continue;
        }
        match evidence.status {
            EvidenceStatus::Success => {}
            EvidenceStatus::Missing => blocked.push(format!("{label}-missing")),
            EvidenceStatus::Unavailable => unavailable.push(format!("{label}-unavailable")),
            EvidenceStatus::Failed => failed.push(format!("{label}-failed")),
        }
    }
    debug_assert!(unavailable.len() >= unavailable_before);
    debug_assert!(failed.len() >= failed_before);
}

fn validate_plan_identity(plan: &UpdatePlan) -> Result<(), CoreFailure> {
    if plan.schema != UPDATE_PLAN_SCHEMA {
        return Err(single_failure(DiagnosticInput {
            code: "update-plan-schema-mismatch",
            path: "plan.schema",
            message: "update plan schema is unsupported",
        }));
    }
    let expected = update_plan_identity_blake3(plan)?;
    require_identity_equal(IdentityExpectation {
        path: "plan.plan_identity_blake3",
        expected: &expected,
        actual: &plan.plan_identity_blake3,
    })
}

fn validate_execution_result(
    plan: &UpdatePlan,
    disposition: UpdateExecutionDisposition,
    published_root: &Option<String>,
    outputs: &[AppliedOutput],
    reason_codes: &[String],
) -> Result<(), CoreFailure> {
    match disposition {
        UpdateExecutionDisposition::Applied => {
            if published_root.as_deref().is_none_or(str::is_empty) {
                return Err(single_failure(DiagnosticInput {
                    code: "update-applied-root-missing",
                    path: "execution.published_root",
                    message: "applied execution requires one published root",
                }));
            }
            if !reason_codes.is_empty() {
                return Err(single_failure(DiagnosticInput {
                    code: "update-applied-has-denial-reasons",
                    path: "execution.reason_codes",
                    message: "applied execution must not have denial reasons",
                }));
            }
            let expected = plan
                .effects
                .iter()
                .map(|effect| AppliedOutput {
                    relative_path: effect.relative_path.clone(),
                    output_digest_blake3: effect.output_digest_blake3.clone(),
                })
                .collect::<Vec<_>>();
            if outputs != expected {
                return Err(single_failure(DiagnosticInput {
                    code: "update-applied-output-mismatch",
                    path: "execution.applied_outputs",
                    message: "applied outputs differ from the accepted plan",
                }));
            }
        }
        UpdateExecutionDisposition::Denied => {
            if published_root.is_some() || !outputs.is_empty() || reason_codes.is_empty() {
                return Err(single_failure(DiagnosticInput {
                    code: "update-denial-shape-invalid",
                    path: "execution",
                    message: "denied execution requires reasons and no published outputs",
                }));
            }
        }
    }
    debug_assert_eq!(disposition == UpdateExecutionDisposition::Applied, published_root.is_some());
    debug_assert!(outputs.len() <= plan.effects.len());
    Ok(())
}

fn enforce_plan_byte_limit(policy: &UpdatePolicy, plan: &UpdatePlan) -> Result<(), CoreFailure> {
    let bytes = serde_json::to_vec(plan).map_err(|error| {
        single_failure(DiagnosticInput {
            code: "update-json-serialization-failed",
            path: "plan",
            message: &format!("serializing update plan: {error}"),
        })
    })?;
    let byte_count = u64::try_from(bytes.len()).map_err(|_| {
        single_failure(DiagnosticInput {
            code: "update-plan-byte-count-overflow",
            path: "plan",
            message: "serialized plan byte count exceeds u64",
        })
    })?;
    if byte_count > policy.limits.max_plan_bytes {
        return Err(single_failure(DiagnosticInput {
            code: "update-plan-byte-limit-exceeded",
            path: "plan",
            message: "complete update plan exceeds policy and cannot be truncated",
        }));
    }
    debug_assert!(byte_count <= policy.limits.max_plan_bytes);
    debug_assert!(policy.limits.max_plan_bytes <= HARD_MAX_PLAN_BYTES);
    Ok(())
}

fn advisory_requirement(policy: &AdvisoryPolicy, service: AdvisoryService) -> &AdvisoryRequirement {
    match service {
        AdvisoryService::Osv => &policy.osv,
        AdvisoryService::Repology => &policy.repology,
    }
}

fn advisory_response_schema(service: AdvisoryService) -> &'static str {
    match service {
        AdvisoryService::Osv => ADVISORY_RESPONSE_OSV_SCHEMA,
        AdvisoryService::Repology => ADVISORY_RESPONSE_REPOLOGY_SCHEMA,
    }
}

fn advisory_service_label(service: AdvisoryService) -> &'static str {
    match service {
        AdvisoryService::Osv => "osv",
        AdvisoryService::Repology => "repology",
    }
}

fn response_schema_for_source(kind: UpdateSourceKind) -> &'static str {
    match kind {
        UpdateSourceKind::GitTags => SOURCE_RESPONSE_GIT_TAGS_SCHEMA,
        UpdateSourceKind::ReleaseIndex | UpdateSourceKind::EcosystemRegistry => SOURCE_RESPONSE_RELEASE_INDEX_SCHEMA,
        UpdateSourceKind::DirectoryIndex => SOURCE_RESPONSE_DIRECTORY_INDEX_SCHEMA,
    }
}

fn require_identity_equal(input: IdentityExpectation<'_>) -> Result<(), CoreFailure> {
    if input.expected != input.actual {
        return Err(single_failure(DiagnosticInput {
            code: "update-identity-mismatch",
            path: input.path,
            message: "stored identity differs from canonical semantic content",
        }));
    }
    Ok(())
}

fn canonical_identity<T: Serialize>(domain: &[u8], value: &T, path: &str) -> Result<String, CoreFailure> {
    let encoded = serde_json::to_vec(value).map_err(|error| {
        single_failure(DiagnosticInput {
            code: "update-json-serialization-failed",
            path,
            message: &format!("serializing identity preimage: {error}"),
        })
    })?;
    let mut hasher = blake3::Hasher::new();
    hasher.update(domain);
    hasher.update(&[DOMAIN_SEPARATOR]);
    hasher.update(&encoded);
    Ok(hasher.finalize().to_hex().to_string())
}

fn normalize_reason_codes(reasons: &mut Vec<String>) {
    reasons.sort();
    reasons.dedup();
}

fn validate_optional_identity(input: TextField<'_>, diagnostics: &mut Vec<Diagnostic>) {
    if !input.value.is_empty() && !is_blake3(input.value) {
        diagnostics.push(diagnostic(DiagnosticInput {
            code: "update-identity-invalid",
            path: input.path,
            message: "identity must be empty before sealing or one lowercase BLAKE3 digest",
        }));
    }
}

fn require_blake3(input: TextField<'_>, diagnostics: &mut Vec<Diagnostic>) {
    if !is_blake3(input.value) {
        diagnostics.push(diagnostic(DiagnosticInput {
            code: "update-blake3-invalid",
            path: input.path,
            message: "value must be one lowercase BLAKE3 digest",
        }));
    }
}

fn require_blake3_result(input: TextField<'_>) -> Result<(), CoreFailure> {
    if !is_blake3(input.value) {
        return Err(single_failure(DiagnosticInput {
            code: "update-blake3-invalid",
            path: input.path,
            message: "value must be one lowercase BLAKE3 digest",
        }));
    }
    Ok(())
}

fn is_blake3(value: &str) -> bool {
    value.len() == BLAKE3_HEX_CHARS && value.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn require_nonempty(input: TextField<'_>, diagnostics: &mut Vec<Diagnostic>) {
    if input.value.trim().is_empty() {
        diagnostics.push(diagnostic(DiagnosticInput {
            code: "update-required-text-missing",
            path: input.path,
            message: "required text is empty",
        }));
    }
}

fn require_exact_text(input: TextExpectation<'_>, diagnostics: &mut Vec<Diagnostic>) {
    if input.actual != input.expected {
        diagnostics.push(diagnostic(DiagnosticInput {
            code: "update-text-mismatch",
            path: input.path,
            message: "value differs from the required contract",
        }));
    }
}

fn require_https_origin(input: TextField<'_>, diagnostics: &mut Vec<Diagnostic>) {
    let remainder = input.value.strip_prefix("https://").unwrap_or_default();
    let is_invalid =
        remainder.is_empty() || remainder.contains(['/', '?', '#', '@']) || remainder.chars().any(char::is_whitespace);
    if is_invalid {
        diagnostics.push(diagnostic(DiagnosticInput {
            code: "update-source-authority-invalid",
            path: input.path,
            message: "source authority must be one credential-free HTTPS origin",
        }));
    }
}

fn require_relative_query(input: TextField<'_>, diagnostics: &mut Vec<Diagnostic>) {
    let is_invalid = !input.value.starts_with('/')
        || input.value.starts_with("//")
        || input.value.contains(['\\', '#'])
        || input.value.chars().any(char::is_whitespace);
    if is_invalid {
        diagnostics.push(diagnostic(DiagnosticInput {
            code: "update-source-query-invalid",
            path: input.path,
            message: "source query must be one safe origin-relative path",
        }));
    }
}

fn require_credential_free_https_url(input: TextField<'_>, diagnostics: &mut Vec<Diagnostic>) {
    let remainder = input.value.strip_prefix("https://").unwrap_or_default();
    let authority = remainder.split(['/', '?', '#']).next().unwrap_or_default();
    let is_invalid = authority.is_empty() || authority.contains('@') || input.value.chars().any(char::is_whitespace);
    if is_invalid {
        diagnostics.push(diagnostic(DiagnosticInput {
            code: "update-advisory-query-invalid",
            path: input.path,
            message: "advisory query must be one credential-free HTTPS URL",
        }));
    }
}

fn validate_u64_limit(input: U64LimitInput<'_>, diagnostics: &mut Vec<Diagnostic>) {
    if input.value == 0 || input.value > input.maximum {
        diagnostics.push(diagnostic(DiagnosticInput {
            code: "update-limit-invalid",
            path: input.path,
            message: "limit must be positive and at most the hard bound",
        }));
    }
}

fn validate_u32_limit(input: U32LimitInput<'_>, diagnostics: &mut Vec<Diagnostic>) {
    if input.value == 0 || input.value > input.maximum {
        diagnostics.push(diagnostic(DiagnosticInput {
            code: "update-limit-invalid",
            path: input.path,
            message: "limit must be positive and at most the hard bound",
        }));
    }
}

fn validate_bounded_zero_u32(input: U32LimitInput<'_>, diagnostics: &mut Vec<Diagnostic>) {
    if input.value > input.maximum {
        diagnostics.push(diagnostic(DiagnosticInput {
            code: "update-limit-invalid",
            path: input.path,
            message: "limit exceeds the hard bound",
        }));
    }
}

fn validate_count(input: CountInput<'_>, diagnostics: &mut Vec<Diagnostic>) {
    match u32::try_from(input.count) {
        Ok(count) if count <= input.maximum => {}
        _ => diagnostics.push(diagnostic(DiagnosticInput {
            code: input.code,
            path: input.path,
            message: "collection count exceeds policy",
        })),
    }
}

fn is_safe_relative_path(path: &str) -> bool {
    if path.is_empty() {
        return false;
    }
    if path.starts_with('/') {
        return false;
    }
    if path.ends_with('/') {
        return false;
    }
    if path.contains('\\') {
        return false;
    }
    path.split('/').all(is_safe_relative_component)
}

fn is_safe_relative_component(component: &str) -> bool {
    !component.is_empty() && component != "." && component != ".."
}

fn is_safe_json_pointer(pointer: &str) -> bool {
    pointer.starts_with('/') && pointer.len() > 1 && !pointer.contains("//")
}

fn diagnostic(input: DiagnosticInput<'_>) -> Diagnostic {
    Diagnostic::new(input.code, input.path, input.message)
}

fn single_failure(input: DiagnosticInput<'_>) -> CoreFailure {
    CoreFailure::from_diagnostic(diagnostic(input))
}

#[cfg(test)]
mod tests {
    use super::*;

    const DIGEST_A: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    const DIGEST_B: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
    const DIGEST_C: &str = "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc";
    const RESPONSE_BYTES: u64 = 1_024;
    const MAX_COMPONENT: u64 = 9_999;
    const VERSION_COMPONENT_COUNT: u8 = 3;
    const EXPECTED_MUTATION_EDIT_COUNT: usize = 3;
    const DEVELOPMENT_PATCH_THRESHOLD: u64 = 5;
    const LIMIT_SMALL: u32 = 64;
    const LIMIT_BYTES: u64 = 1_048_576;
    const LIMIT_MILLIS: u64 = 30_000;

    fn policy() -> UpdatePolicy {
        let draft = UpdatePolicy {
            schema: UPDATE_POLICY_SCHEMA.into(),
            policy_identity_blake3: String::new(),
            migration: UpdatePolicyMigration {
                source_schema: None,
                source_policy_identity_blake3: None,
                reviewed: false,
            },
            package: ImpactPackageKey {
                public_selector: "hello".into(),
            },
            system: "x86_64-linux".into(),
            source_kind: UpdateSourceKind::GitTags,
            adapter_identity: "mantle-git-tags-v1".into(),
            source_authority: "https://api.example.invalid".into(),
            source_query: "/repos/example/hello/tags".into(),
            response_schema: SOURCE_RESPONSE_GIT_TAGS_SCHEMA.into(),
            current_version: "v1.2.3".into(),
            current_source_ref: "refs/tags/v1.2.3".into(),
            current_source_identity_blake3: DIGEST_A.into(),
            version_rules: VersionRules {
                required_prefix: "v".into(),
                component_count: VERSION_COMPONENT_COUNT,
                max_component: MAX_COMPONENT,
                minimum_version: Some("v1.0.0".into()),
                maximum_version: Some("v2.0.0".into()),
                ignored_versions: vec!["v1.2.4".into()],
                allow_prerelease: false,
                odd_minor_is_development: false,
                high_patch_development_from: None,
            },
            patch_policy: PatchPolicy {
                allow_major: false,
                allow_minor: true,
                allow_patch: true,
            },
            advisory_policy: AdvisoryPolicy {
                osv: AdvisoryRequirement {
                    mode: AdvisoryRequirementMode::Required,
                    service_identity: "osv-v1".into(),
                    query: "https://api.osv.dev/v1/query".into(),
                    package_coordinate: "pkg:generic/hello".into(),
                },
                repology: AdvisoryRequirement {
                    mode: AdvisoryRequirementMode::Required,
                    service_identity: "repology-v1".into(),
                    query: "https://repology.org/api/v1/project/hello".into(),
                    package_coordinate: "hello".into(),
                },
                block_on_findings: true,
            },
            validation_policy: UpdateValidationPolicy {
                require_catalog: true,
                require_build_observations: true,
                require_validation_roots: true,
                require_impact_report: true,
            },
            mutation: UpdateMutationTarget {
                relative_path: "locks/hello.json".into(),
                version_pointer: "/version".into(),
                source_ref_pointer: "/source/ref".into(),
                source_identity_pointer: "/source/identity_blake3".into(),
            },
            allow_ambient_credentials: false,
            allow_ambient_proxy: false,
            limits: UpdateLimits {
                max_response_bytes: LIMIT_BYTES,
                max_document_bytes: LIMIT_BYTES,
                max_plan_bytes: LIMIT_BYTES,
                max_candidates: LIMIT_SMALL,
                max_findings: LIMIT_SMALL,
                max_effects: LIMIT_SMALL,
                max_diagnostics: LIMIT_SMALL,
                max_artifacts: LIMIT_SMALL,
                max_redirects: 0,
                max_retries: 0,
                max_elapsed_millis: LIMIT_MILLIS,
            },
        };
        seal_update_policy(&draft).unwrap()
    }

    fn source_observation(policy: &UpdatePolicy) -> SourceObservation {
        let draft = SourceObservation {
            schema: UPDATE_SOURCE_OBSERVATION_SCHEMA.into(),
            observation_identity_blake3: String::new(),
            policy_identity_blake3: policy.policy_identity_blake3.clone(),
            adapter_identity: policy.adapter_identity.clone(),
            source_kind: policy.source_kind,
            query: policy.source_query.clone(),
            source_authority: policy.source_authority.clone(),
            response_schema: policy.response_schema.clone(),
            response_identity_blake3: Some(DIGEST_B.into()),
            status: ObservationStatus::Success,
            candidates: vec![
                SourceCandidate {
                    version: "v1.3.0".into(),
                    source_ref: "refs/tags/v1.3.0".into(),
                    source_identity_blake3: DIGEST_B.into(),
                },
                SourceCandidate {
                    version: "v1.2.5".into(),
                    source_ref: "refs/tags/v1.2.5".into(),
                    source_identity_blake3: DIGEST_C.into(),
                },
            ],
            reason_codes: Vec::new(),
            collection: CollectionFacts {
                response_bytes: RESPONSE_BYTES,
                redirect_count: 0,
                retry_count: 0,
                elapsed_millis: 1,
            },
        };
        seal_source_observation(policy, &draft).unwrap()
    }

    fn advisory(policy: &UpdatePolicy, service: AdvisoryService) -> AdvisoryObservation {
        let requirement = advisory_requirement(&policy.advisory_policy, service);
        let draft = AdvisoryObservation {
            schema: UPDATE_ADVISORY_OBSERVATION_SCHEMA.into(),
            observation_identity_blake3: String::new(),
            policy_identity_blake3: policy.policy_identity_blake3.clone(),
            service,
            service_identity: requirement.service_identity.clone(),
            query: requirement.query.clone(),
            package_coordinate: requirement.package_coordinate.clone(),
            version: "v1.3.0".into(),
            response_schema: advisory_response_schema(service).into(),
            response_identity_blake3: Some(DIGEST_C.into()),
            status: ObservationStatus::Success,
            findings: Vec::new(),
            reason_codes: Vec::new(),
            collection: CollectionFacts {
                response_bytes: RESPONSE_BYTES,
                redirect_count: 0,
                retry_count: 0,
                elapsed_millis: 1,
            },
        };
        seal_advisory_observation(policy, &draft).unwrap()
    }

    fn success_link() -> LinkedEvidence {
        LinkedEvidence {
            status: EvidenceStatus::Success,
            artifact_identity_blake3: vec![DIGEST_A.into()],
            reason_codes: Vec::new(),
        }
    }

    fn validation() -> UpdateValidationEvidence {
        UpdateValidationEvidence {
            schema: UPDATE_VALIDATION_EVIDENCE_SCHEMA.into(),
            candidate_version: "v1.3.0".into(),
            candidate_source_identity_blake3: DIGEST_B.into(),
            catalog: success_link(),
            build_observations: success_link(),
            validation_roots: success_link(),
            impact_report: success_link(),
        }
    }

    fn mutation_document() -> MutationDocument {
        let document = serde_json::json!({
            "version": "v1.2.3",
            "source": {
                "ref": "refs/tags/v1.2.3",
                "identity_blake3": DIGEST_A,
            }
        });
        MutationDocument {
            schema: UPDATE_MUTATION_DOCUMENT_SCHEMA.into(),
            relative_path: "locks/hello.json".into(),
            input_bytes: RESPONSE_BYTES,
            input_digest_blake3: DIGEST_A.into(),
            document,
        }
    }

    fn plan() -> UpdatePlan {
        let policy = policy();
        let source = source_observation(&policy);
        let advisories = vec![
            advisory(&policy, AdvisoryService::Osv),
            advisory(&policy, AdvisoryService::Repology),
        ];
        build_update_plan(UpdatePlanInput {
            policy: &policy,
            source_observation: &source,
            advisory_observations: &advisories,
            validation_evidence: &validation(),
            mutation_documents: &[mutation_document()],
        })
        .unwrap()
    }

    #[test]
    fn complete_evidence_selects_highest_candidate_and_plans_exact_effect() {
        let plan = plan();
        assert_eq!(plan.proposal_status, UpdateProposalStatus::ReadyForReview);
        assert_eq!(plan.selection.selected.as_ref().unwrap().version, "v1.3.0");
        assert_eq!(plan.effects.len(), 1);
        assert_eq!(plan.effects[0].edits.len(), EXPECTED_MUTATION_EDIT_COUNT);
        assert_eq!(plan.plan_identity_blake3.len(), BLAKE3_HEX_CHARS);
    }

    #[test]
    fn candidate_order_does_not_change_plan_identity() {
        let policy = policy();
        let first = source_observation(&policy);
        let mut reordered = first.clone();
        reordered.candidates.reverse();
        reordered.observation_identity_blake3.clear();
        let reordered = seal_source_observation(&policy, &reordered).unwrap();
        let advisories = vec![
            advisory(&policy, AdvisoryService::Repology),
            advisory(&policy, AdvisoryService::Osv),
        ];
        let first_plan = build_update_plan(UpdatePlanInput {
            policy: &policy,
            source_observation: &first,
            advisory_observations: &advisories,
            validation_evidence: &validation(),
            mutation_documents: &[mutation_document()],
        })
        .unwrap();
        let second_plan = build_update_plan(UpdatePlanInput {
            policy: &policy,
            source_observation: &reordered,
            advisory_observations: &advisories,
            validation_evidence: &validation(),
            mutation_documents: &[mutation_document()],
        })
        .unwrap();
        assert_eq!(first_plan.plan_identity_blake3, second_plan.plan_identity_blake3);
    }

    #[test]
    fn conflicting_duplicate_version_is_ambiguous() {
        let policy = policy();
        let mut source = source_observation(&policy);
        source.candidates.push(SourceCandidate {
            version: "v1.3.0".into(),
            source_ref: "refs/tags/other-v1.3.0".into(),
            source_identity_blake3: DIGEST_C.into(),
        });
        source.observation_identity_blake3.clear();
        let source = seal_source_observation(&policy, &source).unwrap();
        let selection = select_candidate(&policy, &source).unwrap();
        assert_eq!(selection.selected.as_ref().unwrap().version, "v1.2.5");
        assert!(
            selection
                .decisions
                .iter()
                .filter(|decision| decision.candidate.version == "v1.3.0")
                .all(|decision| decision.reason_codes.contains(&"candidate-version-ambiguous".into()))
        );
    }

    #[test]
    fn exact_duplicate_source_candidates_produce_no_candidate() {
        let policy = policy();
        let mut source = source_observation(&policy);
        source.observation_identity_blake3.clear();
        let duplicate = source.candidates.iter().find(|candidate| candidate.version == "v1.3.0").unwrap().clone();
        source.candidates = vec![duplicate.clone(), duplicate];
        let source = seal_source_observation(&policy, &source).unwrap();
        let advisories = vec![
            advisory(&policy, AdvisoryService::Osv),
            advisory(&policy, AdvisoryService::Repology),
        ];

        let plan = build_update_plan(UpdatePlanInput {
            policy: &policy,
            source_observation: &source,
            advisory_observations: &advisories,
            validation_evidence: &validation(),
            mutation_documents: &[mutation_document()],
        })
        .unwrap();

        assert!(plan.selection.selected.is_none());
        assert_eq!(plan.proposal_status, UpdateProposalStatus::NoCandidate);
        assert!(plan.selection.decisions.iter().all(|decision| decision.reason_codes == vec!["candidate-duplicate"]));
    }

    #[test]
    fn duplicate_advisory_findings_and_observations_are_rejected() {
        let policy = policy();
        let mut osv = advisory(&policy, AdvisoryService::Osv);
        osv.observation_identity_blake3.clear();
        let finding = AdvisoryFinding {
            finding_id: "OSV-DUPLICATE".into(),
            finding_identity_blake3: DIGEST_A.into(),
        };
        osv.findings = vec![finding.clone(), finding];
        let finding_error = seal_advisory_observation(&policy, &osv).unwrap_err();
        assert!(
            finding_error
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == "update-duplicate-advisory-finding")
        );

        let source = source_observation(&policy);
        let osv = advisory(&policy, AdvisoryService::Osv);
        let duplicate_error = build_update_plan(UpdatePlanInput {
            policy: &policy,
            source_observation: &source,
            advisory_observations: &[osv.clone(), osv],
            validation_evidence: &validation(),
            mutation_documents: &[mutation_document()],
        })
        .unwrap_err();
        assert!(
            duplicate_error
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == "update-duplicate-advisory-observation")
        );
    }

    #[test]
    fn malformed_ignored_prerelease_and_development_versions_are_rejected() {
        let mut policy = policy();
        policy.version_rules.odd_minor_is_development = true;
        policy.version_rules.high_patch_development_from = Some(DEVELOPMENT_PATCH_THRESHOLD);
        policy.policy_identity_blake3.clear();
        let policy = seal_update_policy(&policy).unwrap();
        let mut source = source_observation(&policy);
        source.candidates = vec![
            SourceCandidate {
                version: "v1.03.0".into(),
                source_ref: "bad".into(),
                source_identity_blake3: DIGEST_B.into(),
            },
            SourceCandidate {
                version: "v1.2.4".into(),
                source_ref: "ignored".into(),
                source_identity_blake3: DIGEST_B.into(),
            },
            SourceCandidate {
                version: "v1.4.0-rc1".into(),
                source_ref: "prerelease".into(),
                source_identity_blake3: DIGEST_B.into(),
            },
            SourceCandidate {
                version: "v1.3.0".into(),
                source_ref: "odd".into(),
                source_identity_blake3: DIGEST_B.into(),
            },
            SourceCandidate {
                version: "v1.2.5".into(),
                source_ref: "high-patch".into(),
                source_identity_blake3: DIGEST_B.into(),
            },
        ];
        source.observation_identity_blake3.clear();
        let source = seal_source_observation(&policy, &source).unwrap();
        let selection = select_candidate(&policy, &source).unwrap();
        assert!(selection.selected.is_none());
        assert!(selection.decisions.iter().all(|decision| !decision.eligible));
    }

    #[test]
    fn unavailable_source_is_not_an_empty_success() {
        let policy = policy();
        let mut source = source_observation(&policy);
        source.status = ObservationStatus::Unavailable;
        source.response_identity_blake3 = None;
        source.candidates.clear();
        source.reason_codes = vec!["source-timeout".into()];
        source.observation_identity_blake3.clear();
        let source = seal_source_observation(&policy, &source).unwrap();
        let selection = select_candidate(&policy, &source).unwrap();
        assert!(selection.selected.is_none());
        assert_eq!(selection.reason_codes, vec!["source-observation-unavailable"]);
    }

    #[test]
    fn unavailable_advisory_blocks_ready_status() {
        let policy = policy();
        let source = source_observation(&policy);
        let mut osv = advisory(&policy, AdvisoryService::Osv);
        osv.status = ObservationStatus::Unavailable;
        osv.response_identity_blake3 = None;
        osv.reason_codes = vec!["osv-timeout".into()];
        osv.observation_identity_blake3.clear();
        let osv = seal_advisory_observation(&policy, &osv).unwrap();
        let advisories = vec![osv, advisory(&policy, AdvisoryService::Repology)];
        let plan = build_update_plan(UpdatePlanInput {
            policy: &policy,
            source_observation: &source,
            advisory_observations: &advisories,
            validation_evidence: &validation(),
            mutation_documents: &[mutation_document()],
        })
        .unwrap();
        assert_eq!(plan.proposal_status, UpdateProposalStatus::Unavailable);
        assert_eq!(plan.reason_codes, vec!["osv-observation-unavailable"]);
    }

    #[test]
    fn advisory_findings_require_review() {
        let policy = policy();
        let source = source_observation(&policy);
        let mut osv = advisory(&policy, AdvisoryService::Osv);
        osv.findings.push(AdvisoryFinding {
            finding_id: "OSV-EXAMPLE".into(),
            finding_identity_blake3: DIGEST_A.into(),
        });
        osv.observation_identity_blake3.clear();
        let advisories = vec![
            seal_advisory_observation(&policy, &osv).unwrap(),
            advisory(&policy, AdvisoryService::Repology),
        ];
        let plan = build_update_plan(UpdatePlanInput {
            policy: &policy,
            source_observation: &source,
            advisory_observations: &advisories,
            validation_evidence: &validation(),
            mutation_documents: &[mutation_document()],
        })
        .unwrap();
        assert_eq!(plan.proposal_status, UpdateProposalStatus::Blocked);
        assert_eq!(plan.reason_codes, vec!["osv-findings-require-review"]);
    }

    #[test]
    fn missing_validation_never_becomes_success() {
        let policy = policy();
        let source = source_observation(&policy);
        let mut validation = validation();
        validation.validation_roots = LinkedEvidence {
            status: EvidenceStatus::Missing,
            artifact_identity_blake3: Vec::new(),
            reason_codes: vec!["validation-root-not-run".into()],
        };
        let advisories = vec![
            advisory(&policy, AdvisoryService::Osv),
            advisory(&policy, AdvisoryService::Repology),
        ];
        let plan = build_update_plan(UpdatePlanInput {
            policy: &policy,
            source_observation: &source,
            advisory_observations: &advisories,
            validation_evidence: &validation,
            mutation_documents: &[mutation_document()],
        })
        .unwrap();
        assert_eq!(plan.proposal_status, UpdateProposalStatus::Blocked);
        assert_eq!(plan.reason_codes, vec!["validation-roots-missing"]);
    }

    #[test]
    fn stale_policy_and_observation_identities_fail() {
        let policy = policy();
        let source = source_observation(&policy);
        let advisories = vec![
            advisory(&policy, AdvisoryService::Osv),
            advisory(&policy, AdvisoryService::Repology),
        ];
        let mut stale_policy = policy.clone();
        stale_policy.current_source_ref = "refs/tags/stale".into();
        let error = build_update_plan(UpdatePlanInput {
            policy: &stale_policy,
            source_observation: &source,
            advisory_observations: &advisories,
            validation_evidence: &validation(),
            mutation_documents: &[mutation_document()],
        })
        .unwrap_err();
        assert!(error.diagnostics.iter().any(|diagnostic| diagnostic.code == "update-identity-mismatch"));

        let mut stale_advisory = advisory(&policy, AdvisoryService::Osv);
        stale_advisory.findings.push(AdvisoryFinding {
            finding_id: "OSV-STALE".into(),
            finding_identity_blake3: DIGEST_A.into(),
        });
        let stale_advisories = vec![stale_advisory, advisory(&policy, AdvisoryService::Repology)];
        let error = build_update_plan(UpdatePlanInput {
            policy: &policy,
            source_observation: &source,
            advisory_observations: &stale_advisories,
            validation_evidence: &validation(),
            mutation_documents: &[mutation_document()],
        })
        .unwrap_err();
        assert!(error.diagnostics.iter().any(|diagnostic| diagnostic.code == "update-identity-mismatch"));

        let mut stale_source = source;
        stale_source.query = "stale".into();
        let error = seal_source_observation(&policy, &stale_source).unwrap_err();
        assert!(error.diagnostics.iter().any(|diagnostic| diagnostic.path == "source_observation.query"));
    }

    #[test]
    fn policy_rejects_cross_origin_and_insecure_queries() {
        let mut policy = policy();
        policy.policy_identity_blake3.clear();
        policy.source_authority = "https://api.example.invalid/path".into();
        policy.source_query = "https://other.example.invalid/tags".into();
        policy.advisory_policy.osv.query = "http://api.osv.dev/v1/query".into();

        let error = seal_update_policy(&policy).unwrap_err();
        let codes = error.diagnostics.iter().map(|diagnostic| diagnostic.code.as_str()).collect::<BTreeSet<_>>();

        assert!(codes.contains("update-source-authority-invalid"));
        assert!(codes.contains("update-source-query-invalid"));
        assert!(codes.contains("update-advisory-query-invalid"));
    }

    #[test]
    fn unsafe_path_duplicate_pointer_and_ambient_authority_fail() {
        let mut policy = policy();
        policy.policy_identity_blake3.clear();
        policy.mutation.relative_path = "../escape.json".into();
        policy.mutation.source_ref_pointer = policy.mutation.version_pointer.clone();
        policy.allow_ambient_credentials = true;
        policy.allow_ambient_proxy = true;
        let error = seal_update_policy(&policy).unwrap_err();
        let codes = error.diagnostics.iter().map(|diagnostic| diagnostic.code.as_str()).collect::<BTreeSet<_>>();
        assert!(codes.contains("update-unsafe-mutation-path"));
        assert!(codes.contains("update-duplicate-field-pointer"));
        assert!(codes.contains("update-ambient-credentials-forbidden"));
        assert!(codes.contains("update-ambient-proxy-forbidden"));
    }

    #[test]
    fn mutation_old_value_and_output_limits_fail_closed() {
        let policy = policy();
        let source = source_observation(&policy);
        let advisories = vec![
            advisory(&policy, AdvisoryService::Osv),
            advisory(&policy, AdvisoryService::Repology),
        ];
        let mut document = mutation_document();
        document.document["version"] = Value::String("v0.0.0".into());
        let error = build_update_plan(UpdatePlanInput {
            policy: &policy,
            source_observation: &source,
            advisory_observations: &advisories,
            validation_evidence: &validation(),
            mutation_documents: &[document],
        })
        .unwrap_err();
        assert!(error.diagnostics.iter().any(|diagnostic| diagnostic.code == "update-mutation-old-value-mismatch"));
    }

    #[test]
    fn multiple_mutation_documents_fail_closed() {
        let policy = policy();
        let source = source_observation(&policy);
        let advisories = vec![
            advisory(&policy, AdvisoryService::Osv),
            advisory(&policy, AdvisoryService::Repology),
        ];
        let document = mutation_document();

        let error = build_update_plan(UpdatePlanInput {
            policy: &policy,
            source_observation: &source,
            advisory_observations: &advisories,
            validation_evidence: &validation(),
            mutation_documents: &[document.clone(), document],
        })
        .unwrap_err();

        assert!(
            error
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == "update-mutation-document-count-invalid")
        );
    }

    #[test]
    fn candidate_and_artifact_limits_fail_before_selection() {
        let original_policy = policy();
        let mut draft = source_observation(&original_policy);
        let mut limited_policy = original_policy;
        limited_policy.policy_identity_blake3.clear();
        limited_policy.limits.max_candidates = 1;
        let limited_policy = seal_update_policy(&limited_policy).unwrap();
        draft.policy_identity_blake3 = limited_policy.policy_identity_blake3.clone();
        draft.observation_identity_blake3.clear();
        let error = seal_source_observation(&limited_policy, &draft).unwrap_err();
        assert!(error.diagnostics.iter().any(|diagnostic| diagnostic.code == "update-candidate-limit-exceeded"));
    }

    #[test]
    fn execution_receipts_bind_exact_outputs_and_denials() {
        let plan = plan();
        let effect = &plan.effects[0];
        let applied = record_update_execution(
            &plan,
            UpdateExecutionDisposition::Applied,
            Some("published-update".into()),
            vec![AppliedOutput {
                relative_path: effect.relative_path.clone(),
                output_digest_blake3: effect.output_digest_blake3.clone(),
            }],
            Vec::new(),
        )
        .unwrap();
        assert_eq!(applied.disposition, UpdateExecutionDisposition::Applied);
        let denied = record_update_execution(&plan, UpdateExecutionDisposition::Denied, None, Vec::new(), vec![
            "stale-preimage".into(),
        ])
        .unwrap();
        assert_eq!(denied.applied_outputs, Vec::new());
        assert_eq!(denied.reason_codes, vec!["stale-preimage"]);
    }

    #[test]
    fn execution_rejects_partial_or_wrong_output_claims() {
        let plan = plan();
        let error = record_update_execution(
            &plan,
            UpdateExecutionDisposition::Applied,
            Some("published-update".into()),
            Vec::new(),
            Vec::new(),
        )
        .unwrap_err();
        assert!(error.diagnostics.iter().any(|diagnostic| diagnostic.code == "update-applied-output-mismatch"));
    }

    #[test]
    fn semantic_mutation_changes_plan_identity() {
        let original = plan();
        let mut changed = original.clone();
        changed.reason_codes.push("new-review-reason".into());
        let changed_identity = update_plan_identity_blake3(&changed).unwrap();
        assert_ne!(original.plan_identity_blake3, changed_identity);
    }
}
