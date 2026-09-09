// r[impl mantlepkgs_versions.typed_index]
// r[impl mantlepkgs_versions.observation_status]
// r[impl mantlepkgs_versions.deterministic_resolution]
// r[impl mantlepkgs_versions.resolution_receipt]
// r[impl mantlepkgs_versions.producer_recheck]
// r[impl mantlepkgs_versions.revision_grouping]
// r[impl mantlepkgs_versions.functional_core]
// r[impl mantlepkgs_versions.claim_boundary]
// r[verify mantlepkgs_versions.typed_index]
// r[verify mantlepkgs_versions.observation_status]
// r[verify mantlepkgs_versions.deterministic_resolution]
// r[verify mantlepkgs_versions.resolution_receipt]
// r[verify mantlepkgs_versions.producer_recheck]
// r[verify mantlepkgs_versions.revision_grouping]
// r[verify mantlepkgs_versions.functional_core]
// r[verify mantlepkgs_versions.claim_boundary]

use alloc::collections::BTreeMap;
use alloc::collections::BTreeSet;
use alloc::format;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec;
use alloc::vec::Vec;

use serde::Deserialize;
use serde::Serialize;

use crate::CoreFailure;
use crate::Diagnostic;
use crate::MantlepkgsManifest;
use crate::PackageSelector;
use crate::SUPPORTED_SYSTEM_X86_64_LINUX;
use crate::normalize_manifest;

pub const VERSION_COHORT_SCHEMA: &str = "mantlepkgs-version-cohort-v1";
pub const VERSION_OBSERVATION_SET_SCHEMA: &str = "mantlepkgs-version-observation-set-v1";
pub const VERSION_OBSERVATION_SCHEMA: &str = "mantlepkgs-version-observation-v1";
pub const VERSION_INDEX_SCHEMA: &str = "mantlepkgs-version-index-v1";
pub const VERSION_SELECTION_POLICY_SCHEMA: &str = "mantlepkgs-version-selection-policy-v1";
pub const VERSION_REQUEST_SET_SCHEMA: &str = "mantlepkgs-version-request-set-v1";
pub const VERSION_RESOLUTION_SET_SCHEMA: &str = "mantlepkgs-version-resolution-set-v1";
pub const VERSION_RESOLUTION_RECEIPT_SCHEMA: &str = "mantlepkgs-version-resolution-v1";
pub const VERSION_PRODUCTION_PLAN_SCHEMA: &str = "mantlepkgs-version-production-plan-v1";
pub const VERSION_RECHECK_SET_SCHEMA: &str = "mantlepkgs-version-recheck-set-v1";
pub const VERSION_RECHECK_SCHEMA: &str = "mantlepkgs-version-recheck-v1";
pub const VERSION_SELECTION_METHOD_NEWEST: &str = "newest-published-revision-for-reported-version";

const COHORT_IDENTITY_DOMAIN: &[u8] = b"mantle.mantlepkgs.version-cohort.v1";
const OBSERVATION_IDENTITY_DOMAIN: &[u8] = b"mantle.mantlepkgs.version-observation.v1";
const OBSERVATION_SET_IDENTITY_DOMAIN: &[u8] = b"mantle.mantlepkgs.version-observation-set.v1";
const INDEX_IDENTITY_DOMAIN: &[u8] = b"mantle.mantlepkgs.version-index.v1";
const POLICY_IDENTITY_DOMAIN: &[u8] = b"mantle.mantlepkgs.version-selection-policy.v1";
const REQUEST_IDENTITY_DOMAIN: &[u8] = b"mantle.mantlepkgs.version-request.v1";
const REQUEST_SET_IDENTITY_DOMAIN: &[u8] = b"mantle.mantlepkgs.version-request-set.v1";
const RECEIPT_IDENTITY_DOMAIN: &[u8] = b"mantle.mantlepkgs.version-resolution-receipt.v1";
const RESOLUTION_SET_IDENTITY_DOMAIN: &[u8] = b"mantle.mantlepkgs.version-resolution-set.v1";
const GROUP_IDENTITY_DOMAIN: &[u8] = b"mantle.mantlepkgs.version-revision-group.v1";
const PRODUCTION_PLAN_IDENTITY_DOMAIN: &[u8] = b"mantle.mantlepkgs.version-production-plan.v1";
const RECHECK_IDENTITY_DOMAIN: &[u8] = b"mantle.mantlepkgs.version-recheck.v1";
const RECHECK_SET_IDENTITY_DOMAIN: &[u8] = b"mantle.mantlepkgs.version-recheck-set.v1";
const DOMAIN_SEPARATOR: u8 = 0;
const BLAKE3_HEX_CHARS: usize = 64;
const GIT_REVISION_HEX_CHARS: usize = 40;
const NIX_SHA256_PREFIX: &str = "sha256-";
const NIX_SHA256_BASE64_CHARS: usize = 44;
const HARD_MAX_REVISIONS: u32 = 4_096;
const HARD_MAX_ATTRIBUTES: u32 = 4_096;
const HARD_MAX_OBSERVATIONS: u32 = 65_536;
const HARD_MAX_REQUESTS: u32 = 16_384;
const HARD_MAX_GROUPS: u32 = 4_096;
const HARD_MAX_DIAGNOSTICS: u32 = 4_096;
const HARD_MAX_ALIASES: u32 = 256;
const HARD_MAX_TEXT_BYTES: u32 = 4_096;
const VERSION_NON_CLAIM_COUNT: usize = 6;
const OBSERVATION_STATUS_COUNT: usize = 3;
const MINIMUM_POSITIVE_LIMIT: u32 = 1;

const VERSION_NON_CLAIMS: [&str; VERSION_NON_CLAIM_COUNT] = [
    "reported versions do not prove package identity",
    "resolution does not prove package correctness",
    "resolution does not prove package compatibility",
    "channel publication does not prove binary-cache retention",
    "Nix evaluation does not prove evaluator parity",
    "version-resolution evidence does not prove release eligibility",
];

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum VersionObservationMethod {
    PackageVersionAttribute,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum VersionObservationStatus {
    Success,
    Unavailable,
    Failed,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum VersionResolutionStatus {
    Resolved,
    Blocked,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct VersionResolutionLimits {
    pub max_revisions: u32,
    pub max_attributes: u32,
    pub max_observations: u32,
    pub max_requests: u32,
    pub max_groups: u32,
    pub max_diagnostics: u32,
    pub max_aliases_per_request: u32,
    pub max_text_bytes: u32,
}

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(deny_unknown_fields)]
pub struct VersionCohortRevision {
    pub published_order: u32,
    pub reference: String,
    pub revision: String,
    pub nar_hash: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct VersionCohort {
    pub schema: String,
    pub cohort_identity_blake3: String,
    pub system: String,
    pub generator_identity: String,
    pub index_relative_path: String,
    pub observation_method: VersionObservationMethod,
    pub revisions: Vec<VersionCohortRevision>,
    pub attributes: Vec<String>,
    pub limits: VersionResolutionLimits,
}

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(deny_unknown_fields)]
pub struct VersionObservation {
    pub schema: String,
    pub observation_identity_blake3: String,
    pub cohort_identity_blake3: String,
    pub system: String,
    pub source_reference: String,
    pub revision: String,
    pub published_order: u32,
    pub nar_hash: String,
    pub attribute: String,
    pub method: VersionObservationMethod,
    pub status: VersionObservationStatus,
    pub reported_version: Option<String>,
    pub response_identity_blake3: Option<String>,
    pub reason_codes: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct VersionObservationSet {
    pub schema: String,
    pub observation_set_identity_blake3: String,
    pub cohort_identity_blake3: String,
    pub observations: Vec<VersionObservation>,
}

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(deny_unknown_fields)]
pub struct VersionIndexEntry {
    pub system: String,
    pub attribute: String,
    pub reported_version: String,
    pub source_reference: String,
    pub revision: String,
    pub published_order: u32,
    pub nar_hash: String,
    pub observation_identity_blake3: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct VersionIndex {
    pub schema: String,
    pub index_identity_blake3: String,
    pub cohort_identity_blake3: String,
    pub observation_set_identity_blake3: String,
    pub generator_identity: String,
    pub system: String,
    pub method: VersionObservationMethod,
    pub entries: Vec<VersionIndexEntry>,
    pub successful_observations: u32,
    pub unavailable_observations: u32,
    pub failed_observations: u32,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct VersionSelectionPolicy {
    pub schema: String,
    pub policy_identity_blake3: String,
    pub method: String,
    pub system: String,
    pub accepted_cohort_identity_blake3: String,
    pub accepted_observation_set_identity_blake3: String,
    pub accepted_index_identity_blake3: String,
    pub limits: VersionResolutionLimits,
}

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(deny_unknown_fields)]
pub struct VersionRequest {
    pub request_identity_blake3: String,
    pub system: String,
    pub attribute: String,
    pub reported_version: String,
    pub public_selector: String,
    pub aliases: Vec<String>,
    pub unversioned_default: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct VersionRequestSet {
    pub schema: String,
    pub request_set_identity_blake3: String,
    pub policy_identity_blake3: String,
    pub requests: Vec<VersionRequest>,
}

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ResolvedVersionSource {
    pub source_reference: String,
    pub revision: String,
    pub published_order: u32,
    pub nar_hash: String,
    pub observation_identity_blake3: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct VersionResolutionReceipt {
    pub schema: String,
    pub receipt_identity_blake3: String,
    pub request_identity_blake3: String,
    pub policy_identity_blake3: String,
    pub index_identity_blake3: String,
    pub observation_set_identity_blake3: String,
    pub cohort_identity_blake3: String,
    pub system: String,
    pub attribute: String,
    pub reported_version: String,
    pub public_selector: String,
    pub aliases: Vec<String>,
    pub unversioned_default: bool,
    pub method: String,
    pub status: VersionResolutionStatus,
    pub selected: Option<ResolvedVersionSource>,
    pub blocker_codes: Vec<String>,
    pub non_claims: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct VersionResolutionSet {
    pub schema: String,
    pub resolution_set_identity_blake3: String,
    pub request_set_identity_blake3: String,
    pub policy_identity_blake3: String,
    pub index_identity_blake3: String,
    pub receipts: Vec<VersionResolutionReceipt>,
}

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(deny_unknown_fields)]
pub struct VersionGroupSelector {
    pub request_identity_blake3: String,
    pub receipt_identity_blake3: String,
    pub attribute: String,
    pub reported_version: String,
    pub public_selector: String,
    pub aliases: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct VersionRevisionGroup {
    pub group_identity_blake3: String,
    pub system: String,
    pub source_reference: String,
    pub revision: String,
    pub published_order: u32,
    pub nar_hash: String,
    pub selectors: Vec<VersionGroupSelector>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct VersionProductionPlan {
    pub schema: String,
    pub plan_identity_blake3: String,
    pub resolution_set_identity_blake3: String,
    pub groups: Vec<VersionRevisionGroup>,
    pub blocked_receipt_identity_blake3: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(deny_unknown_fields)]
pub struct VersionRecheckEntry {
    pub receipt_identity_blake3: String,
    pub attribute: String,
    pub expected_version: String,
    pub observed_version: Option<String>,
    pub status: VersionObservationStatus,
    pub reason_codes: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct VersionGroupRecheck {
    pub schema: String,
    pub recheck_identity_blake3: String,
    pub group_identity_blake3: String,
    pub source_reference: String,
    pub revision: String,
    pub nar_hash: String,
    pub source_tree_blake3: Option<String>,
    pub entries: Vec<VersionRecheckEntry>,
    pub reason_codes: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct VersionRecheckSet {
    pub schema: String,
    pub recheck_set_identity_blake3: String,
    pub plan_identity_blake3: String,
    pub rechecks: Vec<VersionGroupRecheck>,
}

pub fn seal_version_cohort(cohort: &VersionCohort) -> Result<VersionCohort, CoreFailure> {
    let mut normalized = cohort.clone();
    normalized.attributes.sort();
    normalized.revisions.sort();
    validate_cohort(&normalized)?;
    let supplied_identity = normalized.cohort_identity_blake3.clone();
    normalized.cohort_identity_blake3.clear();
    let identity = canonical_digest(COHORT_IDENTITY_DOMAIN, &normalized, "cohort-serialization-failed")?;
    reject_stale_identity(&supplied_identity, &identity, "cohort_identity_blake3", "stale-cohort-identity")?;
    normalized.cohort_identity_blake3 = identity;
    debug_assert!(!normalized.attributes.is_empty());
    debug_assert!(!normalized.revisions.is_empty());
    Ok(normalized)
}

pub fn seal_version_observation(
    cohort: &VersionCohort,
    observation: &VersionObservation,
) -> Result<VersionObservation, CoreFailure> {
    let cohort = seal_version_cohort(cohort)?;
    let mut normalized = observation.clone();
    normalized.reason_codes.sort();
    normalized.reason_codes.dedup();
    validate_observation(&cohort, &normalized)?;
    let supplied_identity = normalized.observation_identity_blake3.clone();
    normalized.observation_identity_blake3.clear();
    let identity =
        canonical_digest(OBSERVATION_IDENTITY_DOMAIN, &normalized, "version-observation-serialization-failed")?;
    reject_stale_identity(
        &supplied_identity,
        &identity,
        "observation_identity_blake3",
        "stale-version-observation-identity",
    )?;
    normalized.observation_identity_blake3 = identity;
    debug_assert_eq!(normalized.cohort_identity_blake3, cohort.cohort_identity_blake3);
    debug_assert!(!normalized.observation_identity_blake3.is_empty());
    Ok(normalized)
}

pub fn build_version_observation_set(
    cohort: &VersionCohort,
    observations: &[VersionObservation],
) -> Result<VersionObservationSet, CoreFailure> {
    let cohort = seal_version_cohort(cohort)?;
    let expected_count = bounded_product(
        cohort.revisions.len(),
        cohort.attributes.len(),
        cohort.limits.max_observations,
        "observation-count-overflow",
    )?;
    if observations.len() != expected_count {
        return Err(failure(
            "incomplete-version-observation-set",
            "observations",
            "the observation set must contain one record for every revision and attribute pair",
        ));
    }
    let mut sealed = observations
        .iter()
        .map(|item| seal_version_observation(&cohort, item))
        .collect::<Result<Vec<_>, _>>()?;
    sealed.sort();
    reject_duplicate_observation_keys(&sealed)?;
    let mut observation_set = VersionObservationSet {
        schema: VERSION_OBSERVATION_SET_SCHEMA.into(),
        observation_set_identity_blake3: String::new(),
        cohort_identity_blake3: cohort.cohort_identity_blake3,
        observations: sealed,
    };
    observation_set.observation_set_identity_blake3 = canonical_digest(
        OBSERVATION_SET_IDENTITY_DOMAIN,
        &observation_set,
        "version-observation-set-serialization-failed",
    )?;
    debug_assert_eq!(observation_set.observations.len(), expected_count);
    debug_assert!(!observation_set.observation_set_identity_blake3.is_empty());
    Ok(observation_set)
}

pub fn build_version_index(
    cohort: &VersionCohort,
    observation_set: &VersionObservationSet,
) -> Result<VersionIndex, CoreFailure> {
    let cohort = seal_version_cohort(cohort)?;
    let expected_set = build_version_observation_set(&cohort, &observation_set.observations)?;
    require_equal(
        &observation_set.schema,
        VERSION_OBSERVATION_SET_SCHEMA,
        "observation_set.schema",
        "unsupported-version-observation-set-schema",
    )?;
    require_equal(
        &observation_set.observation_set_identity_blake3,
        &expected_set.observation_set_identity_blake3,
        "observation_set.observation_set_identity_blake3",
        "stale-version-observation-set-identity",
    )?;
    let (entries, status_counts) = compact_index_entries(&expected_set.observations);
    let mut index = VersionIndex {
        schema: VERSION_INDEX_SCHEMA.into(),
        index_identity_blake3: String::new(),
        cohort_identity_blake3: cohort.cohort_identity_blake3,
        observation_set_identity_blake3: expected_set.observation_set_identity_blake3,
        generator_identity: cohort.generator_identity,
        system: cohort.system,
        method: cohort.observation_method,
        entries,
        successful_observations: status_counts[0],
        unavailable_observations: status_counts[1],
        failed_observations: status_counts[2],
    };
    index.index_identity_blake3 =
        canonical_digest(INDEX_IDENTITY_DOMAIN, &index, "version-index-serialization-failed")?;
    validate_version_index(&index, &cohort.limits)?;
    debug_assert_eq!(status_counts.len(), OBSERVATION_STATUS_COUNT);
    debug_assert!(!index.index_identity_blake3.is_empty());
    Ok(index)
}

pub fn seal_version_selection_policy(policy: &VersionSelectionPolicy) -> Result<VersionSelectionPolicy, CoreFailure> {
    validate_limits(&policy.limits)?;
    require_equal(
        &policy.schema,
        VERSION_SELECTION_POLICY_SCHEMA,
        "policy.schema",
        "unsupported-version-selection-policy-schema",
    )?;
    require_equal(
        &policy.method,
        VERSION_SELECTION_METHOD_NEWEST,
        "policy.method",
        "unsupported-version-selection-method",
    )?;
    require_equal(&policy.system, SUPPORTED_SYSTEM_X86_64_LINUX, "policy.system", "unsupported-version-policy-system")?;
    require_blake3(&policy.accepted_cohort_identity_blake3, "policy.accepted_cohort_identity_blake3")?;
    require_blake3(
        &policy.accepted_observation_set_identity_blake3,
        "policy.accepted_observation_set_identity_blake3",
    )?;
    require_blake3(&policy.accepted_index_identity_blake3, "policy.accepted_index_identity_blake3")?;
    let supplied_identity = policy.policy_identity_blake3.clone();
    let mut sealed = policy.clone();
    sealed.policy_identity_blake3.clear();
    let identity = canonical_digest(POLICY_IDENTITY_DOMAIN, &sealed, "version-policy-serialization-failed")?;
    reject_stale_identity(&supplied_identity, &identity, "policy_identity_blake3", "stale-version-policy-identity")?;
    sealed.policy_identity_blake3 = identity;
    debug_assert_eq!(sealed.method, VERSION_SELECTION_METHOD_NEWEST);
    debug_assert!(!sealed.policy_identity_blake3.is_empty());
    Ok(sealed)
}

pub fn seal_version_request_set(
    policy: &VersionSelectionPolicy,
    request_set: &VersionRequestSet,
) -> Result<VersionRequestSet, CoreFailure> {
    let policy = seal_version_selection_policy(policy)?;
    require_equal(
        &request_set.schema,
        VERSION_REQUEST_SET_SCHEMA,
        "request_set.schema",
        "unsupported-version-request-set-schema",
    )?;
    require_equal(
        &request_set.policy_identity_blake3,
        &policy.policy_identity_blake3,
        "request_set.policy_identity_blake3",
        "version-request-policy-mismatch",
    )?;
    require_bounded_count(
        request_set.requests.len(),
        policy.limits.max_requests,
        HARD_MAX_REQUESTS,
        "requests",
        "version-request-limit-exceeded",
    )?;
    if request_set.requests.is_empty() {
        return Err(failure("missing-version-request", "requests", "at least one version request is required"));
    }
    let mut requests =
        request_set.requests.iter().map(|item| seal_request(&policy, item)).collect::<Result<Vec<_>, _>>()?;
    requests.sort();
    validate_request_uniqueness(&requests)?;
    let supplied_identity = request_set.request_set_identity_blake3.clone();
    let mut sealed = VersionRequestSet {
        schema: VERSION_REQUEST_SET_SCHEMA.into(),
        request_set_identity_blake3: String::new(),
        policy_identity_blake3: policy.policy_identity_blake3,
        requests,
    };
    let identity = canonical_digest(REQUEST_SET_IDENTITY_DOMAIN, &sealed, "version-request-set-serialization-failed")?;
    reject_stale_identity(
        &supplied_identity,
        &identity,
        "request_set_identity_blake3",
        "stale-version-request-set-identity",
    )?;
    sealed.request_set_identity_blake3 = identity;
    debug_assert!(!sealed.requests.is_empty());
    debug_assert!(!sealed.request_set_identity_blake3.is_empty());
    Ok(sealed)
}

pub fn resolve_version_requests(
    index: &VersionIndex,
    policy: &VersionSelectionPolicy,
    request_set: &VersionRequestSet,
) -> Result<VersionResolutionSet, CoreFailure> {
    let policy = seal_version_selection_policy(policy)?;
    validate_resolution_inputs(index, &policy)?;
    let request_set = seal_version_request_set(&policy, request_set)?;
    let entries = index
        .entries
        .iter()
        .map(|entry| ((entry.system.as_str(), entry.attribute.as_str(), entry.reported_version.as_str()), entry))
        .collect::<BTreeMap<_, _>>();
    let mut receipts = request_set
        .requests
        .iter()
        .map(|request| build_resolution_receipt(index, &policy, request, &entries))
        .collect::<Result<Vec<_>, _>>()?;
    receipts.sort_by(|left, right| left.request_identity_blake3.cmp(&right.request_identity_blake3));
    let mut set = VersionResolutionSet {
        schema: VERSION_RESOLUTION_SET_SCHEMA.into(),
        resolution_set_identity_blake3: String::new(),
        request_set_identity_blake3: request_set.request_set_identity_blake3,
        policy_identity_blake3: policy.policy_identity_blake3,
        index_identity_blake3: index.index_identity_blake3.clone(),
        receipts,
    };
    set.resolution_set_identity_blake3 =
        canonical_digest(RESOLUTION_SET_IDENTITY_DOMAIN, &set, "version-resolution-set-serialization-failed")?;
    debug_assert_eq!(set.receipts.len(), request_set.requests.len());
    debug_assert!(!set.resolution_set_identity_blake3.is_empty());
    Ok(set)
}

pub fn validate_version_resolution_set(
    index: &VersionIndex,
    policy: &VersionSelectionPolicy,
    request_set: &VersionRequestSet,
    resolution_set: &VersionResolutionSet,
) -> Result<(), CoreFailure> {
    let expected = resolve_version_requests(index, policy, request_set)?;
    if expected != *resolution_set {
        return Err(failure(
            "version-resolution-receipt-mismatch",
            "resolution_set",
            "the resolution set differs from deterministic replay",
        ));
    }
    debug_assert_eq!(expected.resolution_set_identity_blake3, resolution_set.resolution_set_identity_blake3);
    debug_assert_eq!(expected.receipts.len(), resolution_set.receipts.len());
    Ok(())
}

fn seal_resolution_set_artifact(
    resolution_set: &VersionResolutionSet,
    limits: &VersionResolutionLimits,
) -> Result<VersionResolutionSet, CoreFailure> {
    require_equal(
        &resolution_set.schema,
        VERSION_RESOLUTION_SET_SCHEMA,
        "resolution_set.schema",
        "unsupported-version-resolution-set-schema",
    )?;
    require_blake3(&resolution_set.request_set_identity_blake3, "resolution_set.request_set_identity_blake3")?;
    require_blake3(&resolution_set.policy_identity_blake3, "resolution_set.policy_identity_blake3")?;
    require_blake3(&resolution_set.index_identity_blake3, "resolution_set.index_identity_blake3")?;
    require_bounded_count(
        resolution_set.receipts.len(),
        limits.max_requests,
        HARD_MAX_REQUESTS,
        "resolution_set.receipts",
        "version-resolution-receipt-limit-exceeded",
    )?;
    let mut receipts = resolution_set.receipts.clone();
    receipts.sort_by(|left, right| left.request_identity_blake3.cmp(&right.request_identity_blake3));
    let mut request_ids = BTreeSet::new();
    let mut receipt_ids = BTreeSet::new();
    for receipt in &receipts {
        validate_resolution_receipt(receipt)?;
        if !request_ids.insert(receipt.request_identity_blake3.clone()) {
            return Err(failure(
                "duplicate-version-resolution-request",
                "resolution_set.receipts",
                "a request has two receipts",
            ));
        }
        if !receipt_ids.insert(receipt.receipt_identity_blake3.clone()) {
            return Err(failure(
                "duplicate-version-resolution-receipt",
                "resolution_set.receipts",
                "a receipt identity repeats",
            ));
        }
    }
    let supplied_identity = resolution_set.resolution_set_identity_blake3.clone();
    let mut sealed = VersionResolutionSet {
        schema: VERSION_RESOLUTION_SET_SCHEMA.into(),
        resolution_set_identity_blake3: String::new(),
        request_set_identity_blake3: resolution_set.request_set_identity_blake3.clone(),
        policy_identity_blake3: resolution_set.policy_identity_blake3.clone(),
        index_identity_blake3: resolution_set.index_identity_blake3.clone(),
        receipts,
    };
    let identity =
        canonical_digest(RESOLUTION_SET_IDENTITY_DOMAIN, &sealed, "version-resolution-set-serialization-failed")?;
    reject_stale_identity(
        &supplied_identity,
        &identity,
        "resolution_set.resolution_set_identity_blake3",
        "stale-version-resolution-set-identity",
    )?;
    sealed.resolution_set_identity_blake3 = identity;
    debug_assert_eq!(request_ids.len(), sealed.receipts.len());
    debug_assert_eq!(receipt_ids.len(), sealed.receipts.len());
    Ok(sealed)
}

pub fn group_version_resolutions(
    resolution_set: &VersionResolutionSet,
    limits: &VersionResolutionLimits,
) -> Result<VersionProductionPlan, CoreFailure> {
    validate_limits(limits)?;
    let resolution_set = seal_resolution_set_artifact(resolution_set, limits)?;
    let mut groups = BTreeMap::<(String, String, String, String, u32), Vec<VersionGroupSelector>>::new();
    let mut blocked = Vec::new();
    let mut public_keys = BTreeSet::new();
    for receipt in &resolution_set.receipts {
        validate_resolution_receipt(receipt)?;
        if receipt.status == VersionResolutionStatus::Blocked {
            blocked.push(receipt.receipt_identity_blake3.clone());
            continue;
        }
        let selected = receipt.selected.as_ref().ok_or_else(|| {
            failure("resolved-receipt-missing-source", "receipt.selected", "a resolved receipt must contain a source")
        })?;
        if !public_keys.insert((receipt.system.clone(), receipt.public_selector.clone())) {
            return Err(failure(
                "ambiguous-versioned-public-selector",
                "receipts",
                "two resolved requests use the same public selector",
            ));
        }
        let key = (
            receipt.system.clone(),
            selected.source_reference.clone(),
            selected.revision.clone(),
            selected.nar_hash.clone(),
            selected.published_order,
        );
        groups.entry(key).or_default().push(VersionGroupSelector {
            request_identity_blake3: receipt.request_identity_blake3.clone(),
            receipt_identity_blake3: receipt.receipt_identity_blake3.clone(),
            attribute: receipt.attribute.clone(),
            reported_version: receipt.reported_version.clone(),
            public_selector: receipt.public_selector.clone(),
            aliases: receipt.aliases.clone(),
        });
    }
    require_bounded_count(groups.len(), limits.max_groups, HARD_MAX_GROUPS, "groups", "version-group-limit-exceeded")?;
    let mut sealed_groups = groups
        .into_iter()
        .map(|(key, selectors)| seal_revision_group(key, selectors))
        .collect::<Result<Vec<_>, _>>()?;
    sealed_groups.sort_by(|left, right| left.group_identity_blake3.cmp(&right.group_identity_blake3));
    blocked.sort();
    reject_plan_identity_duplicates(&sealed_groups, &blocked)?;
    let mut plan = VersionProductionPlan {
        schema: VERSION_PRODUCTION_PLAN_SCHEMA.into(),
        plan_identity_blake3: String::new(),
        resolution_set_identity_blake3: resolution_set.resolution_set_identity_blake3.clone(),
        groups: sealed_groups,
        blocked_receipt_identity_blake3: blocked,
    };
    plan.plan_identity_blake3 =
        canonical_digest(PRODUCTION_PLAN_IDENTITY_DOMAIN, &plan, "version-production-plan-serialization-failed")?;
    debug_assert!(plan.groups.len() <= resolution_set.receipts.len());
    debug_assert!(!plan.plan_identity_blake3.is_empty());
    Ok(plan)
}

pub fn seal_version_production_plan(plan: &VersionProductionPlan) -> Result<VersionProductionPlan, CoreFailure> {
    require_equal(
        &plan.schema,
        VERSION_PRODUCTION_PLAN_SCHEMA,
        "plan.schema",
        "unsupported-version-production-plan-schema",
    )?;
    require_blake3(&plan.resolution_set_identity_blake3, "plan.resolution_set_identity_blake3")?;
    require_bounded_count(
        plan.groups.len(),
        HARD_MAX_GROUPS,
        HARD_MAX_GROUPS,
        "plan.groups",
        "version-group-limit-exceeded",
    )?;
    require_bounded_count(
        plan.blocked_receipt_identity_blake3.len(),
        HARD_MAX_REQUESTS,
        HARD_MAX_REQUESTS,
        "plan.blocked_receipt_identity_blake3",
        "version-blocked-receipt-limit-exceeded",
    )?;
    let mut groups = plan.groups.iter().map(seal_existing_revision_group).collect::<Result<Vec<_>, _>>()?;
    groups.sort_by(|left, right| left.group_identity_blake3.cmp(&right.group_identity_blake3));
    let mut blocked = plan.blocked_receipt_identity_blake3.clone();
    blocked.sort();
    reject_plan_identity_duplicates(&groups, &blocked)?;
    let supplied_identity = plan.plan_identity_blake3.clone();
    let mut sealed = VersionProductionPlan {
        schema: VERSION_PRODUCTION_PLAN_SCHEMA.into(),
        plan_identity_blake3: String::new(),
        resolution_set_identity_blake3: plan.resolution_set_identity_blake3.clone(),
        groups,
        blocked_receipt_identity_blake3: blocked,
    };
    let identity =
        canonical_digest(PRODUCTION_PLAN_IDENTITY_DOMAIN, &sealed, "version-production-plan-serialization-failed")?;
    reject_stale_identity(
        &supplied_identity,
        &identity,
        "plan.plan_identity_blake3",
        "stale-version-production-plan-identity",
    )?;
    sealed.plan_identity_blake3 = identity;
    debug_assert!(!sealed.plan_identity_blake3.is_empty());
    debug_assert!(sealed.groups.windows(2).all(|pair| pair[0].group_identity_blake3 < pair[1].group_identity_blake3));
    Ok(sealed)
}

pub fn seal_version_recheck_set(
    plan: &VersionProductionPlan,
    recheck_set: &VersionRecheckSet,
) -> Result<VersionRecheckSet, CoreFailure> {
    let plan = seal_version_production_plan(plan)?;
    require_equal(
        &recheck_set.schema,
        VERSION_RECHECK_SET_SCHEMA,
        "recheck_set.schema",
        "unsupported-version-recheck-set-schema",
    )?;
    require_equal(
        &recheck_set.plan_identity_blake3,
        &plan.plan_identity_blake3,
        "recheck_set.plan_identity_blake3",
        "version-recheck-plan-mismatch",
    )?;
    if recheck_set.rechecks.len() != plan.groups.len() {
        return Err(failure(
            "incomplete-version-recheck-set",
            "rechecks",
            "the recheck set must contain one record for each revision group",
        ));
    }
    let groups = plan
        .groups
        .iter()
        .map(|group| (group.group_identity_blake3.as_str(), group))
        .collect::<BTreeMap<_, _>>();
    let mut rechecks = recheck_set
        .rechecks
        .iter()
        .map(|item| {
            let group = groups.get(item.group_identity_blake3.as_str()).ok_or_else(|| {
                failure("unknown-version-recheck-group", "recheck.group_identity_blake3", "the recheck group is absent")
            })?;
            seal_group_recheck(group, item)
        })
        .collect::<Result<Vec<_>, _>>()?;
    rechecks.sort_by(|left, right| left.group_identity_blake3.cmp(&right.group_identity_blake3));
    let supplied_identity = recheck_set.recheck_set_identity_blake3.clone();
    let mut sealed = VersionRecheckSet {
        schema: VERSION_RECHECK_SET_SCHEMA.into(),
        recheck_set_identity_blake3: String::new(),
        plan_identity_blake3: plan.plan_identity_blake3.clone(),
        rechecks,
    };
    let identity = canonical_digest(RECHECK_SET_IDENTITY_DOMAIN, &sealed, "version-recheck-set-serialization-failed")?;
    reject_stale_identity(
        &supplied_identity,
        &identity,
        "recheck_set_identity_blake3",
        "stale-version-recheck-set-identity",
    )?;
    sealed.recheck_set_identity_blake3 = identity;
    debug_assert_eq!(sealed.rechecks.len(), plan.groups.len());
    debug_assert!(!sealed.recheck_set_identity_blake3.is_empty());
    Ok(sealed)
}

pub fn build_version_group_manifests(
    template: &MantlepkgsManifest,
    plan: &VersionProductionPlan,
    recheck_set: &VersionRecheckSet,
) -> Result<Vec<(String, MantlepkgsManifest)>, CoreFailure> {
    let template = normalize_manifest(template)?;
    let plan = seal_version_production_plan(plan)?;
    let recheck_set = seal_version_recheck_set(&plan, recheck_set)?;
    let rechecks = recheck_set
        .rechecks
        .iter()
        .map(|item| (item.group_identity_blake3.as_str(), item))
        .collect::<BTreeMap<_, _>>();
    let mut manifests = Vec::with_capacity(plan.groups.len());
    for group in &plan.groups {
        let recheck = rechecks
            .get(group.group_identity_blake3.as_str())
            .ok_or_else(|| failure("missing-version-recheck", "rechecks", "the production group has no recheck"))?;
        require_successful_recheck(group, recheck)?;
        let source_tree_blake3 = recheck.source_tree_blake3.as_ref().ok_or_else(|| {
            failure(
                "missing-version-source-tree-identity",
                "recheck.source_tree_blake3",
                "a successful recheck needs a source identity",
            )
        })?;
        let mut manifest = template.clone();
        manifest.source.reference = group.source_reference.clone();
        manifest.source.revision = group.revision.clone();
        manifest.source.lock_digest_blake3 = source_tree_blake3.clone();
        manifest.systems = vec![group.system.clone()];
        manifest.selectors = group
            .selectors
            .iter()
            .map(|selector| PackageSelector {
                name: selector.public_selector.clone(),
                attribute: selector.attribute.clone(),
                system: group.system.clone(),
                aliases: selector.aliases.clone(),
            })
            .collect();
        manifest.output.generation_directory = format!("mantlepkgs/version-resolved/{}", group.group_identity_blake3);
        let manifest = normalize_manifest(&manifest)?;
        manifests.push((group.group_identity_blake3.clone(), manifest));
    }
    manifests.sort_by(|left, right| left.0.cmp(&right.0));
    debug_assert_eq!(manifests.len(), plan.groups.len());
    debug_assert!(manifests.iter().all(|(_, manifest)| !manifest.selectors.is_empty()));
    Ok(manifests)
}

fn validate_cohort(cohort: &VersionCohort) -> Result<(), CoreFailure> {
    let mut diagnostics = Vec::new();
    collect_equal(
        &cohort.schema,
        VERSION_COHORT_SCHEMA,
        "schema",
        "unsupported-version-cohort-schema",
        &mut diagnostics,
    );
    collect_equal(
        &cohort.system,
        SUPPORTED_SYSTEM_X86_64_LINUX,
        "system",
        "unsupported-version-cohort-system",
        &mut diagnostics,
    );
    collect_text(&cohort.generator_identity, "generator_identity", cohort.limits.max_text_bytes, &mut diagnostics);
    collect_safe_relative_path(
        &cohort.index_relative_path,
        "index_relative_path",
        cohort.limits.max_text_bytes,
        &mut diagnostics,
    );
    collect_limits(&cohort.limits, &mut diagnostics);
    collect_cohort_revisions(cohort, &mut diagnostics);
    collect_cohort_attributes(cohort, &mut diagnostics);
    if diagnostics.is_empty() {
        debug_assert!(!cohort.revisions.is_empty());
        debug_assert!(!cohort.attributes.is_empty());
        Ok(())
    } else {
        Err(CoreFailure::from_diagnostics(diagnostics))
    }
}

fn collect_cohort_revisions(cohort: &VersionCohort, diagnostics: &mut Vec<Diagnostic>) {
    if cohort.revisions.is_empty() {
        diagnostics.push(Diagnostic::new(
            "missing-version-revision",
            "revisions",
            "at least one exact revision is required",
        ));
    }
    collect_bounded_count(
        cohort.revisions.len(),
        cohort.limits.max_revisions,
        HARD_MAX_REVISIONS,
        "revisions",
        "version-revision-limit-exceeded",
        diagnostics,
    );
    let mut orders = BTreeSet::new();
    let mut revisions = BTreeSet::new();
    for (index, revision) in cohort.revisions.iter().enumerate() {
        let path = format!("revisions[{index}]");
        if !orders.insert(revision.published_order) {
            diagnostics.push(Diagnostic::new(
                "duplicate-published-order",
                &path,
                "published revision order must be unique",
            ));
        }
        if !revisions.insert(revision.revision.as_str()) {
            diagnostics.push(Diagnostic::new("duplicate-version-revision", &path, "the cohort repeats a revision"));
        }
        collect_exact_revision(&revision.revision, &format!("{path}.revision"), diagnostics);
        collect_nar_hash(&revision.nar_hash, &format!("{path}.nar_hash"), diagnostics);
        let expected_reference = format!("github:NixOS/nixpkgs/{}", revision.revision);
        if revision.reference != expected_reference {
            diagnostics.push(Diagnostic::new(
                "floating-version-cohort-reference",
                &format!("{path}.reference"),
                "the cohort reference must name the exact Nixpkgs revision",
            ));
        }
    }
    debug_assert!(orders.len() <= cohort.revisions.len());
    debug_assert!(revisions.len() <= cohort.revisions.len());
}

fn collect_cohort_attributes(cohort: &VersionCohort, diagnostics: &mut Vec<Diagnostic>) {
    if cohort.attributes.is_empty() {
        diagnostics.push(Diagnostic::new(
            "missing-version-attribute",
            "attributes",
            "at least one package attribute is required",
        ));
    }
    collect_bounded_count(
        cohort.attributes.len(),
        cohort.limits.max_attributes,
        HARD_MAX_ATTRIBUTES,
        "attributes",
        "version-attribute-limit-exceeded",
        diagnostics,
    );
    let mut attributes = BTreeSet::new();
    for (index, attribute) in cohort.attributes.iter().enumerate() {
        let path = format!("attributes[{index}]");
        collect_attribute(attribute, &path, diagnostics);
        if !attributes.insert(attribute.as_str()) {
            diagnostics.push(Diagnostic::new("duplicate-version-attribute", &path, "the cohort repeats an attribute"));
        }
    }
    debug_assert!(attributes.len() <= cohort.attributes.len());
}

fn validate_observation(cohort: &VersionCohort, observation: &VersionObservation) -> Result<(), CoreFailure> {
    let mut diagnostics = Vec::new();
    collect_equal(
        &observation.schema,
        VERSION_OBSERVATION_SCHEMA,
        "schema",
        "unsupported-version-observation-schema",
        &mut diagnostics,
    );
    collect_equal(
        &observation.cohort_identity_blake3,
        &cohort.cohort_identity_blake3,
        "cohort_identity_blake3",
        "version-observation-cohort-mismatch",
        &mut diagnostics,
    );
    collect_equal(
        &observation.system,
        &cohort.system,
        "system",
        "version-observation-system-mismatch",
        &mut diagnostics,
    );
    if observation.method != cohort.observation_method {
        diagnostics.push(Diagnostic::new(
            "version-observation-method-mismatch",
            "method",
            "the observation method differs from the cohort method",
        ));
    }
    collect_observation_source(cohort, observation, &mut diagnostics);
    collect_observation_attribute(cohort, observation, &mut diagnostics);
    collect_observation_status(observation, cohort.limits.max_text_bytes, &mut diagnostics);
    if diagnostics.is_empty() {
        debug_assert_eq!(observation.system, cohort.system);
        debug_assert_eq!(observation.method, cohort.observation_method);
        Ok(())
    } else {
        Err(CoreFailure::from_diagnostics(diagnostics))
    }
}

fn collect_observation_source(
    cohort: &VersionCohort,
    observation: &VersionObservation,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let matches = cohort.revisions.iter().filter(|item| item.revision == observation.revision).collect::<Vec<_>>();
    if matches.len() != 1 {
        diagnostics.push(Diagnostic::new(
            "unknown-version-observation-revision",
            "revision",
            "the observation revision is absent from the cohort",
        ));
        return;
    }
    let expected = matches[0];
    collect_equal(
        &observation.source_reference,
        &expected.reference,
        "source_reference",
        "version-observation-reference-mismatch",
        diagnostics,
    );
    collect_equal(
        &observation.nar_hash,
        &expected.nar_hash,
        "nar_hash",
        "version-observation-nar-hash-mismatch",
        diagnostics,
    );
    if observation.published_order != expected.published_order {
        diagnostics.push(Diagnostic::new(
            "version-observation-order-mismatch",
            "published_order",
            "the observation order differs from the cohort",
        ));
    }
    debug_assert_eq!(matches.len(), 1);
    debug_assert_eq!(expected.revision, observation.revision);
}

fn collect_observation_attribute(
    cohort: &VersionCohort,
    observation: &VersionObservation,
    diagnostics: &mut Vec<Diagnostic>,
) {
    collect_attribute(&observation.attribute, "attribute", diagnostics);
    if !cohort.attributes.contains(&observation.attribute) {
        diagnostics.push(Diagnostic::new(
            "unknown-version-observation-attribute",
            "attribute",
            "the observation attribute is absent from the cohort",
        ));
    }
    debug_assert!(!cohort.attributes.is_empty());
    debug_assert!(cohort.attributes.len() <= HARD_MAX_ATTRIBUTES as usize || !diagnostics.is_empty());
}

fn collect_observation_status(
    observation: &VersionObservation,
    max_text_bytes: u32,
    diagnostics: &mut Vec<Diagnostic>,
) {
    match observation.status {
        VersionObservationStatus::Success => {
            let Some(version) = observation.reported_version.as_ref() else {
                diagnostics.push(Diagnostic::new(
                    "missing-reported-version",
                    "reported_version",
                    "a successful observation must contain the package version attribute",
                ));
                return;
            };
            collect_text(version, "reported_version", max_text_bytes, diagnostics);
            if observation.response_identity_blake3.as_ref().is_none_or(|digest| !is_blake3(digest)) {
                diagnostics.push(Diagnostic::new(
                    "missing-version-response-identity",
                    "response_identity_blake3",
                    "a successful observation must bind its response bytes",
                ));
            }
            if !observation.reason_codes.is_empty() {
                diagnostics.push(Diagnostic::new(
                    "successful-version-observation-has-reasons",
                    "reason_codes",
                    "a successful observation must not contain failure reasons",
                ));
            }
        }
        VersionObservationStatus::Unavailable | VersionObservationStatus::Failed => {
            if observation.reported_version.is_some() {
                diagnostics.push(Diagnostic::new(
                    "failed-version-observation-has-version",
                    "reported_version",
                    "an unsuccessful observation must not report a package version",
                ));
            }
            if observation.reason_codes.is_empty() {
                diagnostics.push(Diagnostic::new(
                    "missing-version-observation-reason",
                    "reason_codes",
                    "an unsuccessful observation must contain a stable reason",
                ));
            }
        }
    }
    collect_reason_codes(&observation.reason_codes, max_text_bytes, diagnostics);
    debug_assert!(observation.reason_codes.len() <= HARD_MAX_DIAGNOSTICS as usize || !diagnostics.is_empty());
    debug_assert!(
        matches!(observation.status, VersionObservationStatus::Success) || observation.reported_version.is_none()
    );
}

fn reject_duplicate_observation_keys(observations: &[VersionObservation]) -> Result<(), CoreFailure> {
    let mut keys = BTreeSet::new();
    for observation in observations {
        let key = (observation.revision.as_str(), observation.attribute.as_str());
        if !keys.insert(key) {
            return Err(failure(
                "duplicate-version-observation",
                "observations",
                "the observation set repeats a revision and attribute pair",
            ));
        }
    }
    debug_assert_eq!(keys.len(), observations.len());
    debug_assert!(observations.windows(2).all(|pair| pair[0] <= pair[1]));
    Ok(())
}

fn compact_index_entries(
    observations: &[VersionObservation],
) -> (Vec<VersionIndexEntry>, [u32; OBSERVATION_STATUS_COUNT]) {
    let mut entries = BTreeMap::<(String, String, String), VersionIndexEntry>::new();
    let mut counts = [0_u32; OBSERVATION_STATUS_COUNT];
    for observation in observations {
        let status_index = match observation.status {
            VersionObservationStatus::Success => 0,
            VersionObservationStatus::Unavailable => 1,
            VersionObservationStatus::Failed => 2,
        };
        counts[status_index] = counts[status_index].saturating_add(1);
        if observation.status != VersionObservationStatus::Success {
            continue;
        }
        let version = observation.reported_version.as_ref().expect("validated success has a version");
        let entry = VersionIndexEntry {
            system: observation.system.clone(),
            attribute: observation.attribute.clone(),
            reported_version: version.clone(),
            source_reference: observation.source_reference.clone(),
            revision: observation.revision.clone(),
            published_order: observation.published_order,
            nar_hash: observation.nar_hash.clone(),
            observation_identity_blake3: observation.observation_identity_blake3.clone(),
        };
        let key = (entry.system.clone(), entry.attribute.clone(), entry.reported_version.clone());
        let replace = entries.get(&key).is_none_or(|current| entry.published_order > current.published_order);
        if replace {
            entries.insert(key, entry);
        }
    }
    let result = entries.into_values().collect::<Vec<_>>();
    debug_assert!(result.len() <= observations.len());
    debug_assert_eq!(counts.iter().copied().sum::<u32>() as usize, observations.len());
    (result, counts)
}

fn validate_version_index(index: &VersionIndex, limits: &VersionResolutionLimits) -> Result<(), CoreFailure> {
    require_equal(&index.schema, VERSION_INDEX_SCHEMA, "index.schema", "unsupported-version-index-schema")?;
    require_blake3(&index.index_identity_blake3, "index.index_identity_blake3")?;
    require_blake3(&index.cohort_identity_blake3, "index.cohort_identity_blake3")?;
    require_blake3(&index.observation_set_identity_blake3, "index.observation_set_identity_blake3")?;
    require_equal(&index.system, SUPPORTED_SYSTEM_X86_64_LINUX, "index.system", "unsupported-version-index-system")?;
    require_bounded_count(
        index.entries.len(),
        limits.max_observations,
        HARD_MAX_OBSERVATIONS,
        "index.entries",
        "version-index-entry-limit-exceeded",
    )?;
    let mut keys = BTreeSet::new();
    for entry in &index.entries {
        let key = (&entry.system, &entry.attribute, &entry.reported_version);
        if !keys.insert(key) {
            return Err(failure(
                "duplicate-version-index-key",
                "index.entries",
                "the compact index repeats a lookup key",
            ));
        }
        require_equal(&entry.system, &index.system, "index.entries.system", "version-index-entry-system-mismatch")?;
        require_attribute(&entry.attribute, "index.entries.attribute")?;
        require_text(&entry.reported_version, "index.entries.reported_version", limits.max_text_bytes)?;
        require_exact_revision(&entry.revision, "index.entries.revision")?;
        require_nar_hash(&entry.nar_hash, "index.entries.nar_hash")?;
        require_blake3(&entry.observation_identity_blake3, "index.entries.observation_identity_blake3")?;
    }
    let mut normalized = index.clone();
    let supplied_identity = normalized.index_identity_blake3.clone();
    normalized.index_identity_blake3.clear();
    normalized.entries.sort();
    let expected = canonical_digest(INDEX_IDENTITY_DOMAIN, &normalized, "version-index-serialization-failed")?;
    require_equal(&supplied_identity, &expected, "index.index_identity_blake3", "stale-version-index-identity")?;
    debug_assert_eq!(keys.len(), index.entries.len());
    debug_assert!(!index.index_identity_blake3.is_empty());
    Ok(())
}

fn seal_request(policy: &VersionSelectionPolicy, request: &VersionRequest) -> Result<VersionRequest, CoreFailure> {
    require_equal(&request.system, &policy.system, "request.system", "version-request-system-mismatch")?;
    require_attribute(&request.attribute, "request.attribute")?;
    require_text(&request.reported_version, "request.reported_version", policy.limits.max_text_bytes)?;
    require_selector(&request.public_selector, "request.public_selector", policy.limits.max_text_bytes)?;
    if !request.public_selector.contains(&request.reported_version) {
        return Err(failure(
            "unversioned-public-selector",
            "request.public_selector",
            "the public selector must include the requested version",
        ));
    }
    require_bounded_count(
        request.aliases.len(),
        policy.limits.max_aliases_per_request,
        HARD_MAX_ALIASES,
        "request.aliases",
        "version-request-alias-limit-exceeded",
    )?;
    let mut sealed = request.clone();
    sealed.aliases.sort();
    sealed.aliases.dedup();
    for alias in &sealed.aliases {
        require_selector(alias, "request.aliases", policy.limits.max_text_bytes)?;
    }
    if sealed.unversioned_default && sealed.aliases.is_empty() {
        return Err(failure(
            "default-version-missing-alias",
            "request.aliases",
            "an unversioned default must declare its public alias",
        ));
    }
    if !sealed.unversioned_default && sealed.aliases.iter().any(|alias| !alias.contains(&sealed.reported_version)) {
        return Err(failure(
            "unversioned-alias-without-default",
            "request.aliases",
            "an unversioned alias requires explicit default policy",
        ));
    }
    let supplied_identity = sealed.request_identity_blake3.clone();
    sealed.request_identity_blake3.clear();
    let identity = canonical_digest(REQUEST_IDENTITY_DOMAIN, &sealed, "version-request-serialization-failed")?;
    reject_stale_identity(&supplied_identity, &identity, "request_identity_blake3", "stale-version-request-identity")?;
    sealed.request_identity_blake3 = identity;
    debug_assert!(!sealed.request_identity_blake3.is_empty());
    debug_assert!(sealed.public_selector.contains(&sealed.reported_version));
    Ok(sealed)
}

fn validate_request_uniqueness(requests: &[VersionRequest]) -> Result<(), CoreFailure> {
    let mut keys = BTreeSet::new();
    let mut public_names = BTreeSet::new();
    for request in requests {
        let key = (request.system.as_str(), request.attribute.as_str(), request.reported_version.as_str());
        if !keys.insert(key) {
            return Err(failure(
                "duplicate-version-request",
                "requests",
                "the request set repeats an exact version request",
            ));
        }
        let selector_key = (request.system.as_str(), request.public_selector.as_str());
        if !public_names.insert(selector_key) {
            return Err(failure(
                "ambiguous-version-public-name",
                "requests",
                "two requests use the same selector or alias",
            ));
        }
        for alias in &request.aliases {
            let alias_key = (request.system.as_str(), alias.as_str());
            if !public_names.insert(alias_key) {
                return Err(failure(
                    "ambiguous-version-public-name",
                    "requests",
                    "two requests use the same selector or alias",
                ));
            }
        }
    }
    debug_assert_eq!(keys.len(), requests.len());
    debug_assert!(public_names.len() >= requests.len());
    Ok(())
}

fn validate_resolution_inputs(index: &VersionIndex, policy: &VersionSelectionPolicy) -> Result<(), CoreFailure> {
    validate_version_index(index, &policy.limits)?;
    require_equal(&index.system, &policy.system, "index.system", "version-index-policy-system-mismatch")?;
    require_equal(
        &index.cohort_identity_blake3,
        &policy.accepted_cohort_identity_blake3,
        "index.cohort_identity_blake3",
        "stale-version-cohort",
    )?;
    require_equal(
        &index.observation_set_identity_blake3,
        &policy.accepted_observation_set_identity_blake3,
        "index.observation_set_identity_blake3",
        "stale-version-observation-set",
    )?;
    require_equal(
        &index.index_identity_blake3,
        &policy.accepted_index_identity_blake3,
        "index.index_identity_blake3",
        "stale-version-index",
    )?;
    debug_assert_eq!(index.system, policy.system);
    debug_assert_eq!(index.index_identity_blake3, policy.accepted_index_identity_blake3);
    Ok(())
}

fn build_resolution_receipt(
    index: &VersionIndex,
    policy: &VersionSelectionPolicy,
    request: &VersionRequest,
    entries: &BTreeMap<(&str, &str, &str), &VersionIndexEntry>,
) -> Result<VersionResolutionReceipt, CoreFailure> {
    let key = (request.system.as_str(), request.attribute.as_str(), request.reported_version.as_str());
    let selected = entries.get(&key).map(|entry| ResolvedVersionSource {
        source_reference: entry.source_reference.clone(),
        revision: entry.revision.clone(),
        published_order: entry.published_order,
        nar_hash: entry.nar_hash.clone(),
        observation_identity_blake3: entry.observation_identity_blake3.clone(),
    });
    let status = if selected.is_some() {
        VersionResolutionStatus::Resolved
    } else {
        VersionResolutionStatus::Blocked
    };
    let blocker_codes = if selected.is_some() {
        Vec::new()
    } else {
        vec!["version-not-observed".into()]
    };
    let mut receipt = VersionResolutionReceipt {
        schema: VERSION_RESOLUTION_RECEIPT_SCHEMA.into(),
        receipt_identity_blake3: String::new(),
        request_identity_blake3: request.request_identity_blake3.clone(),
        policy_identity_blake3: policy.policy_identity_blake3.clone(),
        index_identity_blake3: index.index_identity_blake3.clone(),
        observation_set_identity_blake3: index.observation_set_identity_blake3.clone(),
        cohort_identity_blake3: index.cohort_identity_blake3.clone(),
        system: request.system.clone(),
        attribute: request.attribute.clone(),
        reported_version: request.reported_version.clone(),
        public_selector: request.public_selector.clone(),
        aliases: request.aliases.clone(),
        unversioned_default: request.unversioned_default,
        method: policy.method.clone(),
        status,
        selected,
        blocker_codes,
        non_claims: VERSION_NON_CLAIMS.iter().map(|item| (*item).into()).collect(),
    };
    receipt.receipt_identity_blake3 =
        canonical_digest(RECEIPT_IDENTITY_DOMAIN, &receipt, "version-resolution-receipt-serialization-failed")?;
    validate_resolution_receipt(&receipt)?;
    debug_assert_eq!(receipt.status == VersionResolutionStatus::Resolved, receipt.selected.is_some());
    debug_assert!(!receipt.receipt_identity_blake3.is_empty());
    Ok(receipt)
}

fn validate_resolution_receipt(receipt: &VersionResolutionReceipt) -> Result<(), CoreFailure> {
    require_equal(
        &receipt.schema,
        VERSION_RESOLUTION_RECEIPT_SCHEMA,
        "receipt.schema",
        "unsupported-version-resolution-receipt-schema",
    )?;
    require_blake3(&receipt.request_identity_blake3, "receipt.request_identity_blake3")?;
    require_blake3(&receipt.policy_identity_blake3, "receipt.policy_identity_blake3")?;
    require_blake3(&receipt.index_identity_blake3, "receipt.index_identity_blake3")?;
    require_blake3(&receipt.observation_set_identity_blake3, "receipt.observation_set_identity_blake3")?;
    require_blake3(&receipt.cohort_identity_blake3, "receipt.cohort_identity_blake3")?;
    require_equal(
        &receipt.system,
        SUPPORTED_SYSTEM_X86_64_LINUX,
        "receipt.system",
        "unsupported-version-receipt-system",
    )?;
    require_attribute(&receipt.attribute, "receipt.attribute")?;
    require_text(&receipt.reported_version, "receipt.reported_version", HARD_MAX_TEXT_BYTES)?;
    require_selector(&receipt.public_selector, "receipt.public_selector", HARD_MAX_TEXT_BYTES)?;
    require_equal(
        &receipt.method,
        VERSION_SELECTION_METHOD_NEWEST,
        "receipt.method",
        "unsupported-version-selection-method",
    )?;
    validate_receipt_public_names(receipt)?;
    validate_reason_code_list(&receipt.blocker_codes, "receipt.blocker_codes")?;
    if let Some(selected) = &receipt.selected {
        validate_resolved_source(selected)?;
    }
    if receipt.non_claims != VERSION_NON_CLAIMS.iter().map(|item| (*item).to_string()).collect::<Vec<_>>() {
        return Err(failure(
            "version-receipt-non-claim-mismatch",
            "receipt.non_claims",
            "the receipt non-claims changed",
        ));
    }
    match receipt.status {
        VersionResolutionStatus::Resolved if receipt.selected.is_none() || !receipt.blocker_codes.is_empty() => {
            return Err(failure(
                "malformed-resolved-version-receipt",
                "receipt",
                "a resolved receipt needs one source and no blocker",
            ));
        }
        VersionResolutionStatus::Blocked if receipt.selected.is_some() || receipt.blocker_codes.is_empty() => {
            return Err(failure(
                "malformed-blocked-version-receipt",
                "receipt",
                "a blocked receipt needs blockers and no source",
            ));
        }
        _ => {}
    }
    let supplied_identity = receipt.receipt_identity_blake3.clone();
    let mut normalized = receipt.clone();
    normalized.receipt_identity_blake3.clear();
    let expected =
        canonical_digest(RECEIPT_IDENTITY_DOMAIN, &normalized, "version-resolution-receipt-serialization-failed")?;
    require_equal(&supplied_identity, &expected, "receipt.receipt_identity_blake3", "stale-version-receipt-identity")?;
    debug_assert_eq!(receipt.status == VersionResolutionStatus::Resolved, receipt.selected.is_some());
    debug_assert!(!receipt.non_claims.is_empty());
    Ok(())
}

fn validate_receipt_public_names(receipt: &VersionResolutionReceipt) -> Result<(), CoreFailure> {
    if !receipt.public_selector.contains(&receipt.reported_version) {
        return Err(failure(
            "unversioned-public-selector",
            "receipt.public_selector",
            "the selector omits its version",
        ));
    }
    require_bounded_count(
        receipt.aliases.len(),
        HARD_MAX_ALIASES,
        HARD_MAX_ALIASES,
        "receipt.aliases",
        "version-receipt-alias-limit-exceeded",
    )?;
    let mut names = BTreeSet::new();
    names.insert(receipt.public_selector.as_str());
    for alias in &receipt.aliases {
        require_selector(alias, "receipt.aliases", HARD_MAX_TEXT_BYTES)?;
        if !names.insert(alias.as_str()) {
            return Err(failure("ambiguous-version-public-name", "receipt.aliases", "a selector or alias repeats"));
        }
        if !receipt.unversioned_default && !alias.contains(&receipt.reported_version) {
            return Err(failure("unversioned-alias-without-default", "receipt.aliases", "an alias omits its version"));
        }
    }
    if receipt.unversioned_default && receipt.aliases.is_empty() {
        return Err(failure(
            "default-version-missing-alias",
            "receipt.aliases",
            "the default has no unversioned alias",
        ));
    }
    debug_assert!(names.contains(receipt.public_selector.as_str()));
    debug_assert_eq!(names.len(), receipt.aliases.len() + 1);
    Ok(())
}

fn validate_resolved_source(selected: &ResolvedVersionSource) -> Result<(), CoreFailure> {
    require_exact_revision(&selected.revision, "receipt.selected.revision")?;
    require_nar_hash(&selected.nar_hash, "receipt.selected.nar_hash")?;
    require_blake3(&selected.observation_identity_blake3, "receipt.selected.observation_identity_blake3")?;
    let expected_reference = format!("github:NixOS/nixpkgs/{}", selected.revision);
    require_equal(
        &selected.source_reference,
        &expected_reference,
        "receipt.selected.source_reference",
        "version-receipt-reference-mismatch",
    )?;
    debug_assert_eq!(selected.source_reference, expected_reference);
    debug_assert!(!selected.observation_identity_blake3.is_empty());
    Ok(())
}

fn validate_reason_code_list(reason_codes: &[String], path: &str) -> Result<(), CoreFailure> {
    let mut diagnostics = Vec::new();
    collect_reason_codes(reason_codes, HARD_MAX_TEXT_BYTES, &mut diagnostics);
    if diagnostics.is_empty() {
        debug_assert!(reason_codes.len() <= HARD_MAX_DIAGNOSTICS as usize);
        debug_assert!(reason_codes.iter().all(|code| !code.is_empty()));
        Ok(())
    } else {
        for diagnostic in &mut diagnostics {
            diagnostic.path = format!("{path}.{}", diagnostic.path);
        }
        Err(CoreFailure::from_diagnostics(diagnostics))
    }
}

fn seal_revision_group(
    key: (String, String, String, String, u32),
    mut selectors: Vec<VersionGroupSelector>,
) -> Result<VersionRevisionGroup, CoreFailure> {
    selectors.sort();
    if selectors.is_empty() {
        return Err(failure("empty-version-revision-group", "group.selectors", "a revision group needs a selector"));
    }
    let mut group = VersionRevisionGroup {
        group_identity_blake3: String::new(),
        system: key.0,
        source_reference: key.1,
        revision: key.2,
        nar_hash: key.3,
        published_order: key.4,
        selectors,
    };
    validate_revision_group_content(&group)?;
    group.group_identity_blake3 =
        canonical_digest(GROUP_IDENTITY_DOMAIN, &group, "version-group-serialization-failed")?;
    debug_assert!(!group.selectors.is_empty());
    debug_assert!(!group.group_identity_blake3.is_empty());
    Ok(group)
}

fn seal_existing_revision_group(group: &VersionRevisionGroup) -> Result<VersionRevisionGroup, CoreFailure> {
    let sealed = seal_revision_group(
        (
            group.system.clone(),
            group.source_reference.clone(),
            group.revision.clone(),
            group.nar_hash.clone(),
            group.published_order,
        ),
        group.selectors.clone(),
    )?;
    require_equal(
        &group.group_identity_blake3,
        &sealed.group_identity_blake3,
        "group.group_identity_blake3",
        "stale-version-group-identity",
    )?;
    debug_assert_eq!(sealed.group_identity_blake3, group.group_identity_blake3);
    debug_assert!(!sealed.selectors.is_empty());
    Ok(sealed)
}

fn validate_revision_group_content(group: &VersionRevisionGroup) -> Result<(), CoreFailure> {
    require_equal(&group.system, SUPPORTED_SYSTEM_X86_64_LINUX, "group.system", "unsupported-version-group-system")?;
    require_exact_revision(&group.revision, "group.revision")?;
    require_nar_hash(&group.nar_hash, "group.nar_hash")?;
    let expected_reference = format!("github:NixOS/nixpkgs/{}", group.revision);
    require_equal(
        &group.source_reference,
        &expected_reference,
        "group.source_reference",
        "version-group-reference-mismatch",
    )?;
    require_bounded_count(
        group.selectors.len(),
        HARD_MAX_REQUESTS,
        HARD_MAX_REQUESTS,
        "group.selectors",
        "version-group-selector-limit-exceeded",
    )?;
    let mut receipt_ids = BTreeSet::new();
    let mut request_ids = BTreeSet::new();
    let mut public_names = BTreeSet::new();
    for selector in &group.selectors {
        validate_group_selector(selector, &mut receipt_ids, &mut request_ids, &mut public_names)?;
    }
    debug_assert_eq!(receipt_ids.len(), group.selectors.len());
    debug_assert_eq!(request_ids.len(), group.selectors.len());
    Ok(())
}

fn validate_group_selector<'a>(
    selector: &'a VersionGroupSelector,
    receipt_ids: &mut BTreeSet<&'a str>,
    request_ids: &mut BTreeSet<&'a str>,
    public_names: &mut BTreeSet<&'a str>,
) -> Result<(), CoreFailure> {
    require_blake3(&selector.request_identity_blake3, "group.selectors.request_identity_blake3")?;
    require_blake3(&selector.receipt_identity_blake3, "group.selectors.receipt_identity_blake3")?;
    require_attribute(&selector.attribute, "group.selectors.attribute")?;
    require_text(&selector.reported_version, "group.selectors.reported_version", HARD_MAX_TEXT_BYTES)?;
    require_selector(&selector.public_selector, "group.selectors.public_selector", HARD_MAX_TEXT_BYTES)?;
    if !selector.public_selector.contains(&selector.reported_version) {
        return Err(failure(
            "unversioned-public-selector",
            "group.selectors.public_selector",
            "the selector omits its version",
        ));
    }
    require_bounded_count(
        selector.aliases.len(),
        HARD_MAX_ALIASES,
        HARD_MAX_ALIASES,
        "group.selectors.aliases",
        "version-group-alias-limit-exceeded",
    )?;
    require_unique_group_selector_names(selector, public_names)?;
    if !receipt_ids.insert(selector.receipt_identity_blake3.as_str()) {
        return Err(failure("duplicate-version-group-receipt", "group.selectors", "a receipt repeats in one group"));
    }
    if !request_ids.insert(selector.request_identity_blake3.as_str()) {
        return Err(failure("duplicate-version-group-request", "group.selectors", "a request repeats in one group"));
    }
    debug_assert!(receipt_ids.contains(selector.receipt_identity_blake3.as_str()));
    debug_assert!(request_ids.contains(selector.request_identity_blake3.as_str()));
    Ok(())
}

fn require_unique_group_selector_names<'a>(
    selector: &'a VersionGroupSelector,
    public_names: &mut BTreeSet<&'a str>,
) -> Result<(), CoreFailure> {
    if !public_names.insert(selector.public_selector.as_str()) {
        return Err(failure("ambiguous-version-public-name", "group.selectors", "a selector or alias repeats"));
    }
    for alias in &selector.aliases {
        require_selector(alias, "group.selectors.aliases", HARD_MAX_TEXT_BYTES)?;
        if !public_names.insert(alias.as_str()) {
            return Err(failure("ambiguous-version-public-name", "group.selectors", "a selector or alias repeats"));
        }
    }
    debug_assert!(public_names.contains(selector.public_selector.as_str()));
    debug_assert!(public_names.len() > selector.aliases.len());
    Ok(())
}

fn reject_plan_identity_duplicates(groups: &[VersionRevisionGroup], blocked: &[String]) -> Result<(), CoreFailure> {
    if groups.is_empty() && blocked.is_empty() {
        return Err(failure("empty-version-production-plan", "plan", "the production plan has no receipt"));
    }
    let mut group_ids = BTreeSet::new();
    let mut receipt_ids = BTreeSet::new();
    let mut public_names = BTreeSet::new();
    let mut selector_count = 0_u32;
    for group in groups {
        if !group_ids.insert(group.group_identity_blake3.as_str()) {
            return Err(failure("duplicate-version-group", "plan.groups", "a revision group repeats"));
        }
        selector_count = selector_count
            .checked_add(u32::try_from(group.selectors.len()).map_err(|_| {
                failure("version-plan-selector-limit-exceeded", "plan.groups", "the selector count overflowed")
            })?)
            .ok_or_else(|| {
                failure("version-plan-selector-limit-exceeded", "plan.groups", "the selector count overflowed")
            })?;
        for selector in &group.selectors {
            if !receipt_ids.insert(selector.receipt_identity_blake3.as_str()) {
                return Err(failure(
                    "duplicate-version-plan-receipt",
                    "plan.groups",
                    "a receipt repeats across groups",
                ));
            }
            let selector_key = (group.system.as_str(), selector.public_selector.as_str());
            if !public_names.insert(selector_key) {
                return Err(failure(
                    "ambiguous-version-public-name",
                    "plan.groups",
                    "a public name repeats across groups",
                ));
            }
            for alias in &selector.aliases {
                if !public_names.insert((group.system.as_str(), alias.as_str())) {
                    return Err(failure(
                        "ambiguous-version-public-name",
                        "plan.groups",
                        "a public name repeats across groups",
                    ));
                }
            }
        }
    }
    if selector_count > HARD_MAX_REQUESTS {
        return Err(failure(
            "version-plan-selector-limit-exceeded",
            "plan.groups",
            "the selector count exceeds the hard limit",
        ));
    }
    for identity in blocked {
        require_blake3(identity, "plan.blocked_receipt_identity_blake3")?;
        if !receipt_ids.insert(identity.as_str()) {
            return Err(failure("duplicate-version-plan-receipt", "plan", "a receipt is both grouped and blocked"));
        }
    }
    debug_assert_eq!(group_ids.len(), groups.len());
    debug_assert_eq!(receipt_ids.len(), usize::try_from(selector_count).unwrap_or(usize::MAX) + blocked.len());
    Ok(())
}

fn seal_group_recheck(
    group: &VersionRevisionGroup,
    recheck: &VersionGroupRecheck,
) -> Result<VersionGroupRecheck, CoreFailure> {
    require_equal(&recheck.schema, VERSION_RECHECK_SCHEMA, "recheck.schema", "unsupported-version-recheck-schema")?;
    require_equal(
        &recheck.group_identity_blake3,
        &group.group_identity_blake3,
        "recheck.group_identity_blake3",
        "version-recheck-group-mismatch",
    )?;
    require_equal(
        &recheck.source_reference,
        &group.source_reference,
        "recheck.source_reference",
        "version-recheck-reference-mismatch",
    )?;
    require_equal(&recheck.revision, &group.revision, "recheck.revision", "version-recheck-revision-mismatch")?;
    require_equal(&recheck.nar_hash, &group.nar_hash, "recheck.nar_hash", "version-recheck-nar-hash-mismatch")?;
    if recheck.entries.len() != group.selectors.len() {
        return Err(failure(
            "incomplete-version-group-recheck",
            "recheck.entries",
            "each group selector needs a recheck",
        ));
    }
    if let Some(digest) = &recheck.source_tree_blake3 {
        require_blake3(digest, "recheck.source_tree_blake3")?;
    }
    let selectors = group
        .selectors
        .iter()
        .map(|selector| (selector.receipt_identity_blake3.as_str(), selector))
        .collect::<BTreeMap<_, _>>();
    let mut entries = recheck.entries.clone();
    entries.sort();
    let mut seen_receipts = BTreeSet::new();
    for entry in &mut entries {
        entry.reason_codes.sort();
        entry.reason_codes.dedup();
        if !seen_receipts.insert(entry.receipt_identity_blake3.clone()) {
            return Err(failure(
                "duplicate-version-recheck-receipt",
                "recheck.entries",
                "the recheck repeats one receipt and omits another",
            ));
        }
        let selector = selectors.get(entry.receipt_identity_blake3.as_str()).ok_or_else(|| {
            failure(
                "unknown-version-recheck-receipt",
                "recheck.entries",
                "the recheck receipt is absent from the group",
            )
        })?;
        require_equal(
            &entry.attribute,
            &selector.attribute,
            "recheck.entries.attribute",
            "version-recheck-attribute-mismatch",
        )?;
        require_equal(
            &entry.expected_version,
            &selector.reported_version,
            "recheck.entries.expected_version",
            "version-recheck-expected-version-mismatch",
        )?;
        validate_recheck_entry(entry)?;
    }
    validate_reason_code_list(&recheck.reason_codes, "recheck.reason_codes")?;
    let supplied_identity = recheck.recheck_identity_blake3.clone();
    let mut sealed = VersionGroupRecheck {
        schema: VERSION_RECHECK_SCHEMA.into(),
        recheck_identity_blake3: String::new(),
        group_identity_blake3: group.group_identity_blake3.clone(),
        source_reference: group.source_reference.clone(),
        revision: group.revision.clone(),
        nar_hash: group.nar_hash.clone(),
        source_tree_blake3: recheck.source_tree_blake3.clone(),
        entries,
        reason_codes: {
            let mut reasons = recheck.reason_codes.clone();
            reasons.sort();
            reasons.dedup();
            reasons
        },
    };
    let identity = canonical_digest(RECHECK_IDENTITY_DOMAIN, &sealed, "version-recheck-serialization-failed")?;
    reject_stale_identity(&supplied_identity, &identity, "recheck_identity_blake3", "stale-version-recheck-identity")?;
    sealed.recheck_identity_blake3 = identity;
    debug_assert_eq!(sealed.entries.len(), group.selectors.len());
    debug_assert_eq!(seen_receipts.len(), group.selectors.len());
    debug_assert!(!sealed.recheck_identity_blake3.is_empty());
    Ok(sealed)
}

fn validate_recheck_entry(entry: &VersionRecheckEntry) -> Result<(), CoreFailure> {
    require_blake3(&entry.receipt_identity_blake3, "recheck.entries.receipt_identity_blake3")?;
    require_attribute(&entry.attribute, "recheck.entries.attribute")?;
    require_text(&entry.expected_version, "recheck.entries.expected_version", HARD_MAX_TEXT_BYTES)?;
    if let Some(observed) = &entry.observed_version {
        require_text(observed, "recheck.entries.observed_version", HARD_MAX_TEXT_BYTES)?;
    }
    validate_reason_code_list(&entry.reason_codes, "recheck.entries.reason_codes")?;
    match entry.status {
        VersionObservationStatus::Success => {
            let observed = entry.observed_version.as_ref().ok_or_else(|| {
                failure(
                    "missing-rechecked-version",
                    "recheck.entries.observed_version",
                    "a successful recheck needs a version",
                )
            })?;
            require_equal(
                observed,
                &entry.expected_version,
                "recheck.entries.observed_version",
                "rechecked-version-mismatch",
            )?;
            if !entry.reason_codes.is_empty() {
                return Err(failure(
                    "successful-recheck-has-reasons",
                    "recheck.entries.reason_codes",
                    "a successful recheck has no failure reason",
                ));
            }
        }
        VersionObservationStatus::Unavailable | VersionObservationStatus::Failed => {
            if entry.reason_codes.is_empty() {
                return Err(failure(
                    "missing-recheck-reason",
                    "recheck.entries.reason_codes",
                    "an unsuccessful recheck needs a reason",
                ));
            }
        }
    }
    debug_assert!(entry.status != VersionObservationStatus::Success || entry.observed_version.is_some());
    debug_assert!(entry.status == VersionObservationStatus::Success || !entry.reason_codes.is_empty());
    Ok(())
}

fn require_successful_recheck(group: &VersionRevisionGroup, recheck: &VersionGroupRecheck) -> Result<(), CoreFailure> {
    if recheck.source_tree_blake3.is_none() {
        return Err(failure(
            "version-source-recheck-failed",
            "recheck.source_tree_blake3",
            "the selected source did not produce a source-tree identity",
        ));
    }
    if !recheck.reason_codes.is_empty() {
        return Err(failure(
            "version-source-recheck-failed",
            "recheck.reason_codes",
            "the selected source recheck failed",
        ));
    }
    if recheck.entries.iter().any(|entry| entry.status != VersionObservationStatus::Success) {
        return Err(failure(
            "version-attribute-recheck-failed",
            "recheck.entries",
            "a selected package version recheck failed",
        ));
    }
    if recheck.entries.len() != group.selectors.len() {
        return Err(failure("incomplete-version-group-recheck", "recheck.entries", "the group recheck is incomplete"));
    }
    debug_assert!(recheck.source_tree_blake3.is_some());
    debug_assert!(recheck.entries.iter().all(|entry| entry.status == VersionObservationStatus::Success));
    Ok(())
}

fn validate_limits(limits: &VersionResolutionLimits) -> Result<(), CoreFailure> {
    let mut diagnostics = Vec::new();
    collect_limits(limits, &mut diagnostics);
    if diagnostics.is_empty() {
        debug_assert!(limits.max_revisions >= MINIMUM_POSITIVE_LIMIT);
        debug_assert!(limits.max_text_bytes <= HARD_MAX_TEXT_BYTES);
        Ok(())
    } else {
        Err(CoreFailure::from_diagnostics(diagnostics))
    }
}

fn collect_limits(limits: &VersionResolutionLimits, diagnostics: &mut Vec<Diagnostic>) {
    let values = [
        (limits.max_revisions, HARD_MAX_REVISIONS, "limits.max_revisions"),
        (limits.max_attributes, HARD_MAX_ATTRIBUTES, "limits.max_attributes"),
        (limits.max_observations, HARD_MAX_OBSERVATIONS, "limits.max_observations"),
        (limits.max_requests, HARD_MAX_REQUESTS, "limits.max_requests"),
        (limits.max_groups, HARD_MAX_GROUPS, "limits.max_groups"),
        (limits.max_diagnostics, HARD_MAX_DIAGNOSTICS, "limits.max_diagnostics"),
        (limits.max_aliases_per_request, HARD_MAX_ALIASES, "limits.max_aliases_per_request"),
        (limits.max_text_bytes, HARD_MAX_TEXT_BYTES, "limits.max_text_bytes"),
    ];
    for (value, hard_maximum, path) in values {
        if value < MINIMUM_POSITIVE_LIMIT || value > hard_maximum {
            diagnostics.push(Diagnostic::new(
                "invalid-version-resolution-limit",
                path,
                "the named limit must be positive and no larger than its hard maximum",
            ));
        }
    }
    debug_assert!(values.len() > 1);
    debug_assert!(diagnostics.len() <= values.len());
}

fn collect_reason_codes(reason_codes: &[String], max_text_bytes: u32, diagnostics: &mut Vec<Diagnostic>) {
    if exceeds_u32(reason_codes.len(), HARD_MAX_DIAGNOSTICS) {
        diagnostics.push(Diagnostic::new(
            "version-diagnostic-limit-exceeded",
            "reason_codes",
            "the reason-code count exceeds the hard limit",
        ));
    }
    let mut seen = BTreeSet::new();
    for (index, code) in reason_codes.iter().enumerate() {
        let path = format!("reason_codes[{index}]");
        collect_stable_code(code, &path, max_text_bytes, diagnostics);
        if !seen.insert(code.as_str()) {
            diagnostics.push(Diagnostic::new(
                "duplicate-version-reason",
                &path,
                "the observation repeats a reason code",
            ));
        }
    }
    debug_assert!(seen.len() <= reason_codes.len());
}

fn collect_text(value: &str, path: &str, max_bytes: u32, diagnostics: &mut Vec<Diagnostic>) {
    let exceeds = usize::try_from(max_bytes).map_or(true, |maximum| value.len() > maximum);
    if value.is_empty() || exceeds || value.chars().any(char::is_control) {
        diagnostics.push(Diagnostic::new(
            "invalid-version-text",
            path,
            "the text must be nonempty, bounded, and free of control characters",
        ));
    }
}

fn collect_attribute(value: &str, path: &str, diagnostics: &mut Vec<Diagnostic>) {
    let valid = !value.is_empty()
        && value.len() <= HARD_MAX_TEXT_BYTES as usize
        && value.split('.').all(|segment| {
            !segment.is_empty()
                && segment.chars().all(|character| character.is_ascii_alphanumeric() || character == '_')
        });
    if !valid {
        diagnostics.push(Diagnostic::new(
            "unsafe-version-attribute",
            path,
            "the attribute must contain bounded ASCII identifier segments",
        ));
    }
}

fn collect_safe_relative_path(value: &str, path: &str, max_bytes: u32, diagnostics: &mut Vec<Diagnostic>) {
    collect_text(value, path, max_bytes, diagnostics);
    let unsafe_path = value.starts_with('/')
        || value.ends_with('/')
        || value.contains("//")
        || value.contains('\\')
        || value.split('/').any(|component| component == "." || component == "..");
    if unsafe_path {
        diagnostics.push(Diagnostic::new(
            "unsafe-version-index-path",
            path,
            "the index path must be a safe relative path",
        ));
    }
    debug_assert!(value.is_empty() || !value.starts_with('/') || !diagnostics.is_empty());
    debug_assert!(value.is_empty() || !value.contains("..") || !diagnostics.is_empty());
}

fn collect_stable_code(value: &str, path: &str, max_bytes: u32, diagnostics: &mut Vec<Diagnostic>) {
    collect_text(value, path, max_bytes, diagnostics);
    if !value
        .chars()
        .all(|character| character.is_ascii_lowercase() || character.is_ascii_digit() || character == '-')
    {
        diagnostics.push(Diagnostic::new(
            "invalid-version-reason-code",
            path,
            "the reason code must use lowercase ASCII letters, digits, and hyphens",
        ));
    }
}

fn collect_exact_revision(value: &str, path: &str, diagnostics: &mut Vec<Diagnostic>) {
    if !is_lower_hex(value, GIT_REVISION_HEX_CHARS) {
        diagnostics.push(Diagnostic::new(
            "floating-version-revision",
            path,
            "the Nixpkgs revision must be an exact lowercase Git identity",
        ));
    }
}

fn collect_nar_hash(value: &str, path: &str, diagnostics: &mut Vec<Diagnostic>) {
    let payload = value.strip_prefix(NIX_SHA256_PREFIX);
    let valid = payload.is_some_and(|digest| {
        digest.len() == NIX_SHA256_BASE64_CHARS
            && digest
                .chars()
                .all(|character| character.is_ascii_alphanumeric() || matches!(character, '+' | '/' | '='))
    });
    if !valid {
        diagnostics.push(Diagnostic::new(
            "invalid-nix-nar-hash",
            path,
            "the Nix source hash must be tagged SHA-256 SRI text",
        ));
    }
}

fn collect_equal(actual: &str, expected: &str, path: &str, code: &str, diagnostics: &mut Vec<Diagnostic>) {
    if actual != expected {
        diagnostics.push(Diagnostic::new(code, path, "the value differs from the required binding"));
    }
}

fn collect_bounded_count(
    count: usize,
    named_limit: u32,
    hard_limit: u32,
    path: &str,
    code: &str,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if exceeds_u32(count, named_limit) || exceeds_u32(count, hard_limit) {
        diagnostics.push(Diagnostic::new(code, path, "the item count exceeds a named or hard limit"));
    }
}

fn require_equal(actual: &str, expected: &str, path: &str, code: &str) -> Result<(), CoreFailure> {
    if actual == expected {
        Ok(())
    } else {
        Err(failure(code, path, "the value differs from the required binding"))
    }
}

fn require_text(value: &str, path: &str, max_bytes: u32) -> Result<(), CoreFailure> {
    let mut diagnostics = Vec::new();
    collect_text(value, path, max_bytes, &mut diagnostics);
    if diagnostics.is_empty() {
        Ok(())
    } else {
        Err(CoreFailure::from_diagnostics(diagnostics))
    }
}

fn require_attribute(value: &str, path: &str) -> Result<(), CoreFailure> {
    let mut diagnostics = Vec::new();
    collect_attribute(value, path, &mut diagnostics);
    if diagnostics.is_empty() {
        Ok(())
    } else {
        Err(CoreFailure::from_diagnostics(diagnostics))
    }
}

fn require_selector(value: &str, path: &str, max_bytes: u32) -> Result<(), CoreFailure> {
    require_text(value, path, max_bytes)?;
    let invalid = value.starts_with('.')
        || value.ends_with('.')
        || value.contains("..")
        || value.contains('/')
        || value.contains('\\');
    if invalid {
        Err(failure("unsafe-version-public-selector", path, "the public selector is not safe"))
    } else {
        Ok(())
    }
}

fn require_blake3(value: &str, path: &str) -> Result<(), CoreFailure> {
    if is_blake3(value) {
        Ok(())
    } else {
        Err(failure("invalid-blake3-identity", path, "the identity must be lowercase BLAKE3 hexadecimal text"))
    }
}

fn require_exact_revision(value: &str, path: &str) -> Result<(), CoreFailure> {
    let mut diagnostics = Vec::new();
    collect_exact_revision(value, path, &mut diagnostics);
    if diagnostics.is_empty() {
        Ok(())
    } else {
        Err(CoreFailure::from_diagnostics(diagnostics))
    }
}

fn require_nar_hash(value: &str, path: &str) -> Result<(), CoreFailure> {
    let mut diagnostics = Vec::new();
    collect_nar_hash(value, path, &mut diagnostics);
    if diagnostics.is_empty() {
        Ok(())
    } else {
        Err(CoreFailure::from_diagnostics(diagnostics))
    }
}

fn require_bounded_count(
    count: usize,
    named_limit: u32,
    hard_limit: u32,
    path: &str,
    code: &str,
) -> Result<(), CoreFailure> {
    if exceeds_u32(count, named_limit) || exceeds_u32(count, hard_limit) {
        Err(failure(code, path, "the item count exceeds a named or hard limit"))
    } else {
        Ok(())
    }
}

fn reject_stale_identity(supplied: &str, expected: &str, path: &str, code: &str) -> Result<(), CoreFailure> {
    if supplied.is_empty() || supplied == expected {
        Ok(())
    } else {
        Err(failure(code, path, "the supplied identity differs from canonical bytes"))
    }
}

fn canonical_digest<T: Serialize>(domain: &[u8], value: &T, code: &str) -> Result<String, CoreFailure> {
    let bytes = serde_json::to_vec(value)
        .map_err(|_| failure(code, "identity", "the canonical identity input did not serialize"))?;
    let mut hasher = blake3::Hasher::new();
    hasher.update(domain);
    hasher.update(&[DOMAIN_SEPARATOR]);
    hasher.update(&bytes);
    Ok(hasher.finalize().to_hex().as_str().into())
}

fn bounded_product(left: usize, right: usize, named_limit: u32, code: &str) -> Result<usize, CoreFailure> {
    let product = left
        .checked_mul(right)
        .ok_or_else(|| failure(code, "observations", "the observation count overflowed"))?;
    require_bounded_count(product, named_limit, HARD_MAX_OBSERVATIONS, "observations", code)?;
    debug_assert!(product >= left || right == 0);
    debug_assert!(product >= right || left == 0);
    Ok(product)
}

fn is_blake3(value: &str) -> bool {
    is_lower_hex(value, BLAKE3_HEX_CHARS)
}

fn is_lower_hex(value: &str, length: usize) -> bool {
    value.len() == length && value.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn exceeds_u32(count: usize, limit: u32) -> bool {
    u32::try_from(count).map_or(true, |value| value > limit)
}

fn failure(code: &str, path: &str, message: &str) -> CoreFailure {
    CoreFailure::from_diagnostic(Diagnostic::new(code, path, message))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ConversionPolicy;
    use crate::ManifestArtifact;
    use crate::ManifestLimits;
    use crate::NixpkgsSourceLock;
    use crate::OutputLayout;
    use crate::RECOMPUTE_CONVERSION_MODE;
    use crate::SOURCE_BUNDLE_POLICY_MODE;
    use crate::SourcePolicy;

    const REVISION_OLD: &str = "1111111111111111111111111111111111111111";
    const REVISION_NEW: &str = "2222222222222222222222222222222222222222";
    const NAR_HASH_OLD: &str = "sha256-AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=";
    const NAR_HASH_NEW: &str = "sha256-BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB=";
    const RESPONSE_OLD: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    const RESPONSE_NEW: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
    const SOURCE_TREE: &str = "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc";

    fn limits() -> VersionResolutionLimits {
        VersionResolutionLimits {
            max_revisions: 8,
            max_attributes: 8,
            max_observations: 64,
            max_requests: 16,
            max_groups: 8,
            max_diagnostics: 64,
            max_aliases_per_request: 8,
            max_text_bytes: 256,
        }
    }

    fn cohort() -> VersionCohort {
        seal_version_cohort(&VersionCohort {
            schema: VERSION_COHORT_SCHEMA.into(),
            cohort_identity_blake3: String::new(),
            system: SUPPORTED_SYSTEM_X86_64_LINUX.into(),
            generator_identity: "mantle-version-index-producer-v1".into(),
            index_relative_path: "indexes/x86_64-linux.json".into(),
            observation_method: VersionObservationMethod::PackageVersionAttribute,
            revisions: vec![
                VersionCohortRevision {
                    published_order: 1,
                    reference: format!("github:NixOS/nixpkgs/{REVISION_OLD}"),
                    revision: REVISION_OLD.into(),
                    nar_hash: NAR_HASH_OLD.into(),
                },
                VersionCohortRevision {
                    published_order: 2,
                    reference: format!("github:NixOS/nixpkgs/{REVISION_NEW}"),
                    revision: REVISION_NEW.into(),
                    nar_hash: NAR_HASH_NEW.into(),
                },
            ],
            attributes: vec!["hello".into()],
            limits: limits(),
        })
        .unwrap()
    }

    fn observation(
        cohort: &VersionCohort,
        revision: &VersionCohortRevision,
        version: Option<&str>,
        status: VersionObservationStatus,
    ) -> VersionObservation {
        VersionObservation {
            schema: VERSION_OBSERVATION_SCHEMA.into(),
            observation_identity_blake3: String::new(),
            cohort_identity_blake3: cohort.cohort_identity_blake3.clone(),
            system: cohort.system.clone(),
            source_reference: revision.reference.clone(),
            revision: revision.revision.clone(),
            published_order: revision.published_order,
            nar_hash: revision.nar_hash.clone(),
            attribute: "hello".into(),
            method: VersionObservationMethod::PackageVersionAttribute,
            status,
            reported_version: version.map(Into::into),
            response_identity_blake3: version.map(|value| {
                if value == "2.12.1" {
                    RESPONSE_OLD.into()
                } else {
                    RESPONSE_NEW.into()
                }
            }),
            reason_codes: if status == VersionObservationStatus::Success {
                Vec::new()
            } else {
                vec!["version-unavailable".into()]
            },
        }
    }

    fn index() -> (VersionCohort, VersionObservationSet, VersionIndex) {
        let cohort = cohort();
        let observations = vec![
            observation(&cohort, &cohort.revisions[0], Some("2.12.1"), VersionObservationStatus::Success),
            observation(&cohort, &cohort.revisions[1], Some("2.12.1"), VersionObservationStatus::Success),
        ];
        let observation_set = build_version_observation_set(&cohort, &observations).unwrap();
        let index = build_version_index(&cohort, &observation_set).unwrap();
        (cohort, observation_set, index)
    }

    fn policy(
        cohort: &VersionCohort,
        observation_set: &VersionObservationSet,
        index: &VersionIndex,
    ) -> VersionSelectionPolicy {
        seal_version_selection_policy(&VersionSelectionPolicy {
            schema: VERSION_SELECTION_POLICY_SCHEMA.into(),
            policy_identity_blake3: String::new(),
            method: VERSION_SELECTION_METHOD_NEWEST.into(),
            system: SUPPORTED_SYSTEM_X86_64_LINUX.into(),
            accepted_cohort_identity_blake3: cohort.cohort_identity_blake3.clone(),
            accepted_observation_set_identity_blake3: observation_set.observation_set_identity_blake3.clone(),
            accepted_index_identity_blake3: index.index_identity_blake3.clone(),
            limits: limits(),
        })
        .unwrap()
    }

    fn requests(policy: &VersionSelectionPolicy, version: &str) -> VersionRequestSet {
        seal_version_request_set(policy, &VersionRequestSet {
            schema: VERSION_REQUEST_SET_SCHEMA.into(),
            request_set_identity_blake3: String::new(),
            policy_identity_blake3: policy.policy_identity_blake3.clone(),
            requests: vec![VersionRequest {
                request_identity_blake3: String::new(),
                system: SUPPORTED_SYSTEM_X86_64_LINUX.into(),
                attribute: "hello".into(),
                reported_version: version.into(),
                public_selector: format!("hello@{version}"),
                aliases: vec![format!("hello-{version}")],
                unversioned_default: false,
            }],
        })
        .unwrap()
    }

    fn template_manifest() -> MantlepkgsManifest {
        MantlepkgsManifest {
            schema: crate::MANIFEST_SCHEMA.into(),
            source: NixpkgsSourceLock {
                reference: format!("github:NixOS/nixpkgs/{REVISION_OLD}"),
                revision: REVISION_OLD.into(),
                lock_digest_blake3: SOURCE_TREE.into(),
            },
            systems: vec![SUPPORTED_SYSTEM_X86_64_LINUX.into()],
            selectors: vec![PackageSelector {
                name: "template".into(),
                attribute: "hello".into(),
                system: SUPPORTED_SYSTEM_X86_64_LINUX.into(),
                aliases: Vec::new(),
            }],
            conversion_policy: ConversionPolicy {
                mode: RECOMPUTE_CONVERSION_MODE.into(),
                target_store_prefix: "/mantle/store".into(),
                translation_policy: ManifestArtifact {
                    path: "policy/translation.json".into(),
                    digest_blake3: RESPONSE_OLD.into(),
                },
                execution_profile: ManifestArtifact {
                    path: "policy/execution-profile.json".into(),
                    digest_blake3: RESPONSE_NEW.into(),
                },
            },
            source_policy: SourcePolicy {
                mode: SOURCE_BUNDLE_POLICY_MODE.into(),
                optional_transports: Vec::new(),
            },
            output: OutputLayout {
                generation_directory: "mantlepkgs/template".into(),
            },
            limits: ManifestLimits {
                max_selectors: 16,
                max_graph_nodes: 128,
                max_graph_bytes: 1_024,
                max_source_requirements: 128,
                max_artifact_bytes: 1_024,
            },
        }
    }

    #[test]
    fn newest_observation_wins_and_input_order_does_not_change_identity() {
        let (cohort, first_set, first_index) = index();
        assert_eq!(first_index.entries.len(), 1);
        assert_eq!(first_index.entries[0].revision, REVISION_NEW);

        let mut observations = first_set.observations.clone();
        observations.reverse();
        for observation in &mut observations {
            observation.observation_identity_blake3.clear();
        }
        let second_set = build_version_observation_set(&cohort, &observations).unwrap();
        let second_index = build_version_index(&cohort, &second_set).unwrap();
        assert_eq!(first_set.observation_set_identity_blake3, second_set.observation_set_identity_blake3);
        assert_eq!(first_index.index_identity_blake3, second_index.index_identity_blake3);
    }

    #[test]
    fn invalid_cohort_rejects_floating_wrong_system_duplicates_and_limits() {
        let mut invalid = cohort();
        invalid.cohort_identity_blake3.clear();
        invalid.system = "aarch64-linux".into();
        invalid.revisions[0].reference = "github:NixOS/nixpkgs/nixos-unstable".into();
        invalid.revisions[1].published_order = invalid.revisions[0].published_order;
        invalid.attributes.push("hello".into());
        invalid.index_relative_path = "../index.json".into();
        invalid.limits.max_revisions = 0;
        let error = seal_version_cohort(&invalid).unwrap_err();
        let codes = error.diagnostics.iter().map(|item| item.code.as_str()).collect::<BTreeSet<_>>();
        assert!(codes.contains("unsupported-version-cohort-system"));
        assert!(codes.contains("floating-version-cohort-reference"));
        assert!(codes.contains("duplicate-published-order"));
        assert!(codes.contains("duplicate-version-attribute"));
        assert!(codes.contains("unsafe-version-index-path"));
        assert!(codes.contains("invalid-version-resolution-limit"));
    }

    #[test]
    fn unavailable_observation_stays_explicit_and_never_parses_a_name() {
        let cohort = cohort();
        let observations = vec![
            observation(&cohort, &cohort.revisions[0], Some("2.12.1"), VersionObservationStatus::Success),
            observation(&cohort, &cohort.revisions[1], None, VersionObservationStatus::Unavailable),
        ];
        let set = build_version_observation_set(&cohort, &observations).unwrap();
        let index = build_version_index(&cohort, &set).unwrap();
        assert_eq!(index.entries.len(), 1);
        assert_eq!(index.unavailable_observations, 1);
        assert_eq!(index.entries[0].revision, REVISION_OLD);
        assert_eq!(index.entries[0].reported_version, "2.12.1");
    }

    #[test]
    fn malformed_success_and_wrong_nar_hash_fail_closed() {
        let cohort = cohort();
        let mut malformed = observation(&cohort, &cohort.revisions[0], None, VersionObservationStatus::Success);
        malformed.nar_hash = NAR_HASH_NEW.into();
        let error = seal_version_observation(&cohort, &malformed).unwrap_err();
        let codes = error.diagnostics.iter().map(|item| item.code.as_str()).collect::<BTreeSet<_>>();
        assert!(codes.contains("version-observation-nar-hash-mismatch"));
        assert!(codes.contains("missing-reported-version"));
    }

    #[test]
    fn resolution_receipt_binds_newest_exact_source_and_replays() {
        let (cohort, observation_set, index) = index();
        let policy = policy(&cohort, &observation_set, &index);
        let requests = requests(&policy, "2.12.1");
        let resolved = resolve_version_requests(&index, &policy, &requests).unwrap();
        assert_eq!(resolved.receipts.len(), 1);
        assert_eq!(resolved.receipts[0].status, VersionResolutionStatus::Resolved);
        assert_eq!(resolved.receipts[0].selected.as_ref().unwrap().revision, REVISION_NEW);
        validate_version_resolution_set(&index, &policy, &requests, &resolved).unwrap();
    }

    #[test]
    fn unknown_version_is_blocked_without_nearby_selection() {
        let (cohort, observation_set, index) = index();
        let policy = policy(&cohort, &observation_set, &index);
        let requests = requests(&policy, "2.12.2");
        let resolved = resolve_version_requests(&index, &policy, &requests).unwrap();
        let receipt = &resolved.receipts[0];
        assert_eq!(receipt.status, VersionResolutionStatus::Blocked);
        assert_eq!(receipt.blocker_codes, vec!["version-not-observed"]);
        assert!(receipt.selected.is_none());
    }

    #[test]
    fn stale_index_and_mutated_receipt_fail_replay() {
        let (cohort, observation_set, index) = index();
        let mut stale_policy = policy(&cohort, &observation_set, &index);
        stale_policy.policy_identity_blake3.clear();
        stale_policy.accepted_index_identity_blake3 = SOURCE_TREE.into();
        let stale_policy = seal_version_selection_policy(&stale_policy).unwrap();
        let stale_requests = requests(&stale_policy, "2.12.1");
        let error = resolve_version_requests(&index, &stale_policy, &stale_requests).unwrap_err();
        assert_eq!(error.diagnostics[0].code, "stale-version-index");

        let valid_policy = policy(&cohort, &observation_set, &index);
        let valid_requests = requests(&valid_policy, "2.12.1");
        let mut resolved = resolve_version_requests(&index, &valid_policy, &valid_requests).unwrap();
        resolved.receipts[0].reported_version = "2.12.2".into();
        let error = validate_version_resolution_set(&index, &valid_policy, &valid_requests, &resolved).unwrap_err();
        assert_eq!(error.diagnostics[0].code, "version-resolution-receipt-mismatch");
    }

    #[test]
    fn request_names_reject_selector_alias_collisions() {
        let (cohort, observation_set, index) = index();
        let policy = policy(&cohort, &observation_set, &index);
        let mut request_set = requests(&policy, "2.12.1");
        request_set.request_set_identity_blake3.clear();
        request_set.requests.push(VersionRequest {
            request_identity_blake3: String::new(),
            system: SUPPORTED_SYSTEM_X86_64_LINUX.into(),
            attribute: "hello".into(),
            reported_version: "hello".into(),
            public_selector: "other@hello".into(),
            aliases: vec!["hello@2.12.1".into()],
            unversioned_default: false,
        });
        let error = seal_version_request_set(&policy, &request_set).unwrap_err();
        assert_eq!(error.diagnostics[0].code, "ambiguous-version-public-name");
        assert_eq!(error.diagnostics[0].path, "requests");
    }

    #[test]
    fn stale_resolution_set_and_production_plan_fail_closed() {
        let (cohort, observation_set, index) = index();
        let policy = policy(&cohort, &observation_set, &index);
        let request_set = requests(&policy, "2.12.1");
        let resolved = resolve_version_requests(&index, &policy, &request_set).unwrap();
        let mut dropped = resolved.clone();
        dropped.receipts.clear();
        let error = group_version_resolutions(&dropped, &limits()).unwrap_err();
        assert_eq!(error.diagnostics[0].code, "stale-version-resolution-set-identity");

        let mut plan = group_version_resolutions(&resolved, &limits()).unwrap();
        plan.groups[0].selectors[0].attribute = "changed".into();
        let error = seal_version_production_plan(&plan).unwrap_err();
        assert_eq!(error.diagnostics[0].code, "stale-version-group-identity");
    }

    #[test]
    fn duplicate_recheck_receipt_cannot_hide_an_unchecked_selector() {
        let (cohort, observation_set, index) = index();
        let policy = policy(&cohort, &observation_set, &index);
        let request_set = requests(&policy, "2.12.1");
        let resolved = resolve_version_requests(&index, &policy, &request_set).unwrap();
        let mut plan = group_version_resolutions(&resolved, &limits()).unwrap();
        let original = plan.groups[0].selectors[0].clone();
        let mut second = original.clone();
        second.request_identity_blake3 = RESPONSE_OLD.into();
        second.receipt_identity_blake3 = RESPONSE_NEW.into();
        second.public_selector = "other@2.12.1".into();
        second.aliases = vec!["other-2.12.1".into()];
        plan.groups[0] = seal_revision_group(
            (
                plan.groups[0].system.clone(),
                plan.groups[0].source_reference.clone(),
                plan.groups[0].revision.clone(),
                plan.groups[0].nar_hash.clone(),
                plan.groups[0].published_order,
            ),
            vec![original.clone(), second],
        )
        .unwrap();
        plan.plan_identity_blake3.clear();
        let plan = seal_version_production_plan(&plan).unwrap();
        let duplicate_entry = VersionRecheckEntry {
            receipt_identity_blake3: original.receipt_identity_blake3,
            attribute: original.attribute,
            expected_version: original.reported_version.clone(),
            observed_version: Some(original.reported_version),
            status: VersionObservationStatus::Success,
            reason_codes: Vec::new(),
        };
        let recheck = VersionGroupRecheck {
            schema: VERSION_RECHECK_SCHEMA.into(),
            recheck_identity_blake3: String::new(),
            group_identity_blake3: plan.groups[0].group_identity_blake3.clone(),
            source_reference: plan.groups[0].source_reference.clone(),
            revision: plan.groups[0].revision.clone(),
            nar_hash: plan.groups[0].nar_hash.clone(),
            source_tree_blake3: Some(SOURCE_TREE.into()),
            entries: vec![duplicate_entry.clone(), duplicate_entry],
            reason_codes: Vec::new(),
        };
        let error = seal_version_recheck_set(&plan, &VersionRecheckSet {
            schema: VERSION_RECHECK_SET_SCHEMA.into(),
            recheck_set_identity_blake3: String::new(),
            plan_identity_blake3: plan.plan_identity_blake3.clone(),
            rechecks: vec![recheck],
        })
        .unwrap_err();
        assert_eq!(error.diagnostics[0].code, "duplicate-version-recheck-receipt");
        assert_eq!(error.diagnostics[0].path, "recheck.entries");
    }

    #[test]
    fn two_versions_group_by_exact_revision_and_keep_selectors_distinct() {
        let cohort = cohort();
        let observations = vec![
            observation(&cohort, &cohort.revisions[0], Some("2.12.1"), VersionObservationStatus::Success),
            observation(&cohort, &cohort.revisions[1], Some("2.12.2"), VersionObservationStatus::Success),
        ];
        let observation_set = build_version_observation_set(&cohort, &observations).unwrap();
        let index = build_version_index(&cohort, &observation_set).unwrap();
        let policy = policy(&cohort, &observation_set, &index);
        let mut request_set = requests(&policy, "2.12.1");
        request_set.request_set_identity_blake3.clear();
        request_set.requests.push(VersionRequest {
            request_identity_blake3: String::new(),
            system: SUPPORTED_SYSTEM_X86_64_LINUX.into(),
            attribute: "hello".into(),
            reported_version: "2.12.2".into(),
            public_selector: "hello@2.12.2".into(),
            aliases: vec!["hello-2.12.2".into()],
            unversioned_default: false,
        });
        let request_set = seal_version_request_set(&policy, &request_set).unwrap();
        let resolved = resolve_version_requests(&index, &policy, &request_set).unwrap();
        let plan = group_version_resolutions(&resolved, &limits()).unwrap();
        assert_eq!(plan.groups.len(), 2);
        assert!(plan.blocked_receipt_identity_blake3.is_empty());
        assert_ne!(plan.groups[0].revision, plan.groups[1].revision);
    }

    #[test]
    fn successful_rechecks_emit_existing_manifests() {
        let (cohort, observation_set, index) = index();
        let policy = policy(&cohort, &observation_set, &index);
        let requests = requests(&policy, "2.12.1");
        let resolved = resolve_version_requests(&index, &policy, &requests).unwrap();
        let plan = group_version_resolutions(&resolved, &limits()).unwrap();
        let group = &plan.groups[0];
        let recheck_set = VersionRecheckSet {
            schema: VERSION_RECHECK_SET_SCHEMA.into(),
            recheck_set_identity_blake3: String::new(),
            plan_identity_blake3: plan.plan_identity_blake3.clone(),
            rechecks: vec![VersionGroupRecheck {
                schema: VERSION_RECHECK_SCHEMA.into(),
                recheck_identity_blake3: String::new(),
                group_identity_blake3: group.group_identity_blake3.clone(),
                source_reference: group.source_reference.clone(),
                revision: group.revision.clone(),
                nar_hash: group.nar_hash.clone(),
                source_tree_blake3: Some(SOURCE_TREE.into()),
                entries: vec![VersionRecheckEntry {
                    receipt_identity_blake3: group.selectors[0].receipt_identity_blake3.clone(),
                    attribute: "hello".into(),
                    expected_version: "2.12.1".into(),
                    observed_version: Some("2.12.1".into()),
                    status: VersionObservationStatus::Success,
                    reason_codes: Vec::new(),
                }],
                reason_codes: Vec::new(),
            }],
        };
        let manifests = build_version_group_manifests(&template_manifest(), &plan, &recheck_set).unwrap();
        assert_eq!(manifests.len(), 1);
        assert_eq!(manifests[0].1.source.revision, REVISION_NEW);
        assert_eq!(manifests[0].1.source.lock_digest_blake3, SOURCE_TREE);
        assert_eq!(manifests[0].1.selectors[0].name, "hello@2.12.1");
    }

    #[test]
    fn failed_recheck_never_emits_a_partial_manifest() {
        let (cohort, observation_set, index) = index();
        let policy = policy(&cohort, &observation_set, &index);
        let requests = requests(&policy, "2.12.1");
        let resolved = resolve_version_requests(&index, &policy, &requests).unwrap();
        let plan = group_version_resolutions(&resolved, &limits()).unwrap();
        let group = &plan.groups[0];
        let recheck_set = VersionRecheckSet {
            schema: VERSION_RECHECK_SET_SCHEMA.into(),
            recheck_set_identity_blake3: String::new(),
            plan_identity_blake3: plan.plan_identity_blake3.clone(),
            rechecks: vec![VersionGroupRecheck {
                schema: VERSION_RECHECK_SCHEMA.into(),
                recheck_identity_blake3: String::new(),
                group_identity_blake3: group.group_identity_blake3.clone(),
                source_reference: group.source_reference.clone(),
                revision: group.revision.clone(),
                nar_hash: group.nar_hash.clone(),
                source_tree_blake3: None,
                entries: vec![VersionRecheckEntry {
                    receipt_identity_blake3: group.selectors[0].receipt_identity_blake3.clone(),
                    attribute: "hello".into(),
                    expected_version: "2.12.1".into(),
                    observed_version: None,
                    status: VersionObservationStatus::Failed,
                    reason_codes: vec!["nix-evaluation-failed".into()],
                }],
                reason_codes: vec!["source-materialization-failed".into()],
            }],
        };
        let error = build_version_group_manifests(&template_manifest(), &plan, &recheck_set).unwrap_err();
        assert_eq!(error.diagnostics[0].code, "version-source-recheck-failed");
        assert_eq!(error.diagnostics[0].path, "recheck.source_tree_blake3");
    }
}
