#![allow(
    tigerstyle::explicit_defaults,
    reason = "closed serde DTOs use explicit additive defaults to preserve the documented v1 wire rollout"
)]

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

pub const CAIRN_CONTENT_BOUND_REGISTRY_CONTRACT_REVISION: &str = "d953fe11ab620f3a42bdad1db51bc7672dd29824";
pub const VALENCE_CONTENT_BOUND_REQUIREMENT_CONTRACT_REVISION: &str = "6ab37aa34c9f81da812d6b42f2f06b7ac2d7e214";
pub const CAIRN_CONTENT_BOUND_REQUIREMENT_REGISTRY_SCHEMA_V1: &str = "cairn.content-bound-requirement-registry.v1";
pub const CAIRN_CONTENT_BOUND_REQUIREMENT_REGISTRY_RECEIPT_SCHEMA_V1: &str =
    "cairn.content-bound-requirement-registry-receipt.v1";
pub const CAIRN_CONTENT_BOUND_REQUIREMENT_REGISTRY_BOUNDARY: &str = "content-bound requirement registry validation proves canonical identity, current-root freshness, and declared linkage only; it does not prove revision authenticity, requirement satisfaction, source correctness, implementation correctness, or release eligibility";
pub const VALENCE_REQUIREMENT_REF_SCHEMA_V1: &str = "valence.requirement-ref.v1";
pub const CONTENT_BOUND_RELEASE_INPUT_SCHEMA_V1: &str = "mantle.content-bound-release-requirement-input.v1";
pub const CONTENT_BOUND_RELEASE_EVIDENCE_SCHEMA_V1: &str = "mantle.content-bound-release-requirement-evidence.v1";
pub const CONTENT_BOUND_EVIDENCE_MANIFEST_SCHEMA_V1: &str = "mantle.content-bound-evidence-manifest.v1";
pub const CONTENT_BOUND_EVIDENCE_ROW_SCHEMA_V1: &str = "mantle.content-bound-evidence-row.v1";
pub const CONTENT_BOUND_COVERAGE_ROW_SCHEMA_V1: &str = "mantle.content-bound-requirement-coverage-row.v1";
pub const CONTENT_BOUND_REQUIREMENT_MODE_OPTIONAL: &str = "optional";
pub const CONTENT_BOUND_REQUIREMENT_MODE_REQUIRED: &str = "required";
pub const CONTENT_BOUND_REQUIREMENT_DISPOSITION_ABSENT: &str = "absent";
pub const CONTENT_BOUND_REQUIREMENT_DISPOSITION_PRESENT: &str = "present";
pub const CONTENT_BOUND_REQUIREMENT_DISPOSITION_INVALID: &str = "invalid";
pub const CONTENT_BOUND_EVIDENCE_EXTERNAL_SCHEMA_V1: &str = "mantle.content-bound-release-evidence-file.v1";
pub const CONTENT_BOUND_EVIDENCE_CLAIM_SCOPE: &str = "content-bound requirement identity and evidence linkage only";
pub const CONTENT_BOUND_RELEASE_BOUNDARY: &str = "Mantle validates exact supplied Cairn registry, Valence requirement-reference, bundle evidence, release, and binary identities only; it does not prove registry freshness beyond supplied bytes, revision authenticity, requirement satisfaction, source correctness, test truth, producer authority, runtime safety, or release eligibility";
pub const LEGACY_COVERAGE_BOUNDARY: &str = "legacy release coverage remains recorded-only string linkage; it does not establish Cairn registry membership, Valence reference identity, current evidence bytes, requirement satisfaction, or release eligibility";

pub const CONTENT_BOUND_RELEASE_NON_CLAIMS: &[&str] = &[
    "identity and linkage evidence only",
    "supplied registry freshness was not independently established",
    "declared revision authenticity was not established",
    "not requirement satisfaction",
    "not source correctness",
    "not test truth",
    "not producer authority",
    "not runtime safety",
    "not release eligibility",
];
pub const CONTENT_BOUND_EVIDENCE_ROW_NON_CLAIMS: &[&str] = &[
    "exact file bytes and declared location only",
    "not source correctness",
    "not test truth",
    "not requirement satisfaction",
    "not release eligibility",
];

const CAIRN_REGISTRY_NON_CLAIMS: &[&str] = &[
    "declared revision authenticity was not established",
    "not requirement satisfaction",
    "not source correctness",
    "not implementation correctness",
    "not release eligibility",
];
const VALENCE_REQUIREMENT_NON_CLAIMS: &[&str] = &[
    "identity and linkage evidence only",
    "supplied registry freshness was not independently established",
    "declared revision authenticity was not established",
    "not requirement satisfaction",
    "not evidence truth",
    "not runtime authority",
    "not lifecycle readiness",
    "not release eligibility",
];
const CAIRN_SPECIFICATION_SET_DOMAIN: &str = "cairn.requirement-registry.specification-set.v1";
const CAIRN_REQUIREMENT_ROW_DOMAIN: &str = "cairn.requirement-registry.row.v1";
const CAIRN_REGISTRY_DOMAIN: &str = "cairn.requirement-registry.registry.v1";
const CAIRN_RECEIPT_DOMAIN: &str = "cairn.requirement-registry.receipt.v1";
const VALENCE_REQUIREMENT_REF_DOMAIN: &str = "valence.requirement-ref.v1";
const EVIDENCE_ROW_DOMAIN: &str = "mantle.content-bound-evidence-row.v1";
const EVIDENCE_MANIFEST_DOMAIN: &str = "mantle.content-bound-evidence-manifest.v1";
const COVERAGE_ROW_DOMAIN: &str = "mantle.content-bound-requirement-coverage-row.v1";
const RELEASE_LINK_DOMAIN: &str = "mantle.content-bound-release-link.v1";
const RELEASE_EVIDENCE_DOMAIN: &str = "mantle.content-bound-release-requirement-evidence.v1";
const DOMAIN_SEPARATOR: u8 = 0;
const GIT_SHA1_HEX_LENGTH: usize = 40;
const GIT_SHA256_HEX_LENGTH: usize = 64;
const JUJUTSU_REVISION_MIN_HEX_LENGTH: usize = 16;
const JUJUTSU_REVISION_MAX_HEX_LENGTH: usize = 64;
const MAX_OPAQUE_REVISION_BYTES: usize = 256;
const MAX_SYMBOL_BYTES: usize = 512;
const MAX_REGISTRY_COUNT: usize = 32;
const MAX_REQUIREMENT_REFERENCE_COUNT: usize = 4_096;
const MAX_EVIDENCE_ROW_COUNT: usize = 8_192;
const MAX_COVERAGE_ROW_COUNT: usize = 8_192;
const MAX_VALIDATION_ISSUE_COUNT: usize = 256;
const MAX_VALIDATION_ISSUE_COUNT_BEFORE_SENTINEL: usize = MAX_VALIDATION_ISSUE_COUNT.saturating_sub(1);

const _: () = {
    assert!(MAX_REGISTRY_COUNT > 0);
    assert!(MAX_REQUIREMENT_REFERENCE_COUNT > 0);
    assert!(MAX_EVIDENCE_ROW_COUNT > 0);
    assert!(MAX_COVERAGE_ROW_COUNT > 0);
    assert!(MAX_VALIDATION_ISSUE_COUNT > 1);
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ValenceVerificationRoleV1 {
    Property,
    RecordedOnly,
    Boundary,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RequirementDigestDomainV1 {
    Specification,
    RequirementBody,
    Policy,
    RequirementRow,
    RequirementRegistry,
    RequirementRegistryReceipt,
    RequirementReference,
    StackRevisionSet,
    StackTraceContext,
    EvidenceGraph,
    Release,
    Lifecycle,
    Runtime,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RequirementDigestV1 {
    pub domain: RequirementDigestDomainV1,
    pub blake3: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StackRevisionKindV1 {
    Git,
    Jujutsu,
    Opaque,
}

impl StackRevisionKindV1 {
    const fn as_str(&self) -> &'static str {
        match self {
            Self::Git => "git",
            Self::Jujutsu => "jujutsu",
            Self::Opaque => "opaque",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StackRevisionV1 {
    pub kind: StackRevisionKindV1,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CairnContentBoundRevisionV1 {
    pub kind: String,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CairnContentBoundSpecificationIdentityV1 {
    pub path: String,
    pub content_blake3: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CairnContentBoundSourceSpanV1 {
    pub start_byte: u64,
    pub end_byte: u64,
    pub start_line: u64,
    pub end_line: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CairnContentBoundRequirementRowV1 {
    pub requirement_id: String,
    pub specification_path: String,
    pub specification_blake3: String,
    pub source_span: CairnContentBoundSourceSpanV1,
    pub requirement_body_blake3: String,
    pub row_blake3: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CairnContentBoundRequirementRegistryReceiptV1 {
    pub schema_version: String,
    pub registry_blake3: String,
    pub repository_id: String,
    pub revision_kind: String,
    pub revision: String,
    pub specification_set_blake3: String,
    pub policy_blake3: String,
    pub row_count: usize,
    pub non_claims: Vec<String>,
    pub receipt_blake3: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CairnContentBoundRequirementRegistryV1 {
    pub schema_version: String,
    pub boundary: String,
    pub repository_id: String,
    pub revision: CairnContentBoundRevisionV1,
    pub specification_set_blake3: String,
    pub specifications: Vec<CairnContentBoundSpecificationIdentityV1>,
    pub policy_blake3: String,
    pub rows: Vec<CairnContentBoundRequirementRowV1>,
    pub non_claims: Vec<String>,
    pub registry_blake3: String,
    pub receipt: CairnContentBoundRequirementRegistryReceiptV1,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RequirementRefV1 {
    pub schema: String,
    pub repository_id: String,
    pub revision: StackRevisionV1,
    pub requirement_id: String,
    pub specification_path: String,
    pub specification_identity: RequirementDigestV1,
    pub requirement_body_identity: RequirementDigestV1,
    pub policy_identity: RequirementDigestV1,
    pub row_identity: RequirementDigestV1,
    pub registry_identity: RequirementDigestV1,
    pub registry_receipt_identity: RequirementDigestV1,
    pub verification_role: ValenceVerificationRoleV1,
    pub non_claims: Vec<String>,
    pub reference_identity: RequirementDigestV1,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContentBoundEvidenceRoleV1 {
    Source,
    Test,
    Proof,
    Receipt,
}

impl ContentBoundEvidenceRoleV1 {
    #[must_use]
    pub const fn external_role(self) -> &'static str {
        match self {
            Self::Source => "content-bound-source-evidence",
            Self::Test => "content-bound-test-evidence",
            Self::Proof => "content-bound-proof-evidence",
            Self::Receipt => "content-bound-receipt-evidence",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContentBoundDigestDomainV1 {
    EvidenceFile,
    EvidenceRow,
    EvidenceManifest,
    RequirementCoverage,
    Release,
    Binary,
    ReleaseEvidence,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContentBoundDigestV1 {
    pub domain: ContentBoundDigestDomainV1,
    pub blake3: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContentBoundEvidenceSpanV1 {
    pub start_byte: u64,
    pub end_byte: u64,
    pub start_line: u64,
    pub end_line: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContentBoundMeasuredEvidenceRowV1 {
    pub schema: String,
    pub repository_id: String,
    pub repository_relative_path: String,
    pub bundle_relative_path: String,
    pub role: ContentBoundEvidenceRoleV1,
    pub size_bytes: u64,
    pub content_identity: ContentBoundDigestV1,
    // Wire compatibility: absent optional fields remain readable during additive rollout.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub span: Option<ContentBoundEvidenceSpanV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub symbol: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub producer_receipt_row_identity: Option<ContentBoundDigestV1>,
    pub non_claims: Vec<String>,
    pub row_identity: ContentBoundDigestV1,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContentBoundEvidenceManifestV1 {
    pub schema: String,
    pub rows: Vec<ContentBoundMeasuredEvidenceRowV1>,
    pub non_claims: Vec<String>,
    pub manifest_identity: ContentBoundDigestV1,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContentBoundCoverageDraftV1 {
    pub requirement_reference_blake3: String,
    pub evidence_role: ContentBoundEvidenceRoleV1,
    pub evidence_repository_paths: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContentBoundCoverageRowV1 {
    pub schema: String,
    pub repository_id: String,
    pub requirement_id: String,
    pub requirement_reference_identity: RequirementDigestV1,
    pub registry_identity: RequirementDigestV1,
    pub evidence_role: ContentBoundEvidenceRoleV1,
    pub evidence_row_identities: Vec<ContentBoundDigestV1>,
    pub coverage_identity: ContentBoundDigestV1,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContentBoundProducerContractsV1 {
    pub cairn_revision: String,
    pub cairn_registry_schema: String,
    pub valence_revision: String,
    pub valence_requirement_ref_schema: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContentBoundReleaseInputV1 {
    pub schema: String,
    pub registries: Vec<CairnContentBoundRequirementRegistryV1>,
    pub requirement_refs: Vec<RequirementRefV1>,
    pub coverage: Vec<ContentBoundCoverageDraftV1>,
    pub non_claims: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContentBoundReleaseBuildInputV1 {
    pub input: ContentBoundReleaseInputV1,
    pub evidence_rows: Vec<ContentBoundMeasuredEvidenceRowV1>,
    pub release_id: String,
    pub binary_digests_blake3: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContentBoundReleaseEvidenceV1 {
    pub schema: String,
    pub producer_contracts: ContentBoundProducerContractsV1,
    pub registries: Vec<CairnContentBoundRequirementRegistryV1>,
    pub requirement_refs: Vec<RequirementRefV1>,
    pub evidence_manifest: ContentBoundEvidenceManifestV1,
    pub coverage: Vec<ContentBoundCoverageRowV1>,
    pub release_id: String,
    pub release_identity: ContentBoundDigestV1,
    pub binary_identities: Vec<ContentBoundDigestV1>,
    pub non_claims: Vec<String>,
    pub evidence_identity: ContentBoundDigestV1,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContentBoundExternalEvidenceLinkV1 {
    pub role: String,
    pub schema: String,
    pub bundle_relative_path: String,
    pub digest_blake3: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContentBoundRequirementIssueCodeV1 {
    Blake3Invalid,
    CanonicalSerializationFailed,
    ContractRevisionMismatch,
    CoverageDuplicate,
    CoverageEvidenceDuplicate,
    CoverageEvidenceMissing,
    CoverageMissing,
    CoverageRoleMismatch,
    DigestDomainSubstitution,
    EvidenceManifestIdentityStale,
    EvidencePathDuplicate,
    EvidenceReceiptLinkInvalid,
    EvidenceReceiptLinkMissing,
    EvidenceRowIdentityStale,
    EvidenceRowMissing,
    EvidenceSpanInvalid,
    EvidenceSymbolInvalid,
    ExternalEvidenceLinkMismatch,
    InputSchemaUnsupported,
    IssueLimitReached,
    NonClaimMissing,
    ReferenceIdentityStale,
    ReferenceSchemaUnsupported,
    RegistryBoundaryWeakened,
    RegistryDuplicate,
    RegistryHashInvalid,
    RegistryMissing,
    RegistryOrderInvalid,
    RegistryReceiptHashInvalid,
    RegistryReceiptLinkMismatch,
    RegistryReceiptSchemaUnsupported,
    RegistryRequirementDuplicate,
    RegistryRequirementMissing,
    RegistryRowHashInvalid,
    RegistrySchemaUnsupported,
    RegistrySpecificationDuplicate,
    RegistrySpecificationHashInvalid,
    RegistrySpecificationMissing,
    RegistrySpecificationSetHashInvalid,
    ReleaseBinaryLinkMismatch,
    ReleaseEvidenceIdentityStale,
    ReleaseIdMismatch,
    ReleaseInputEmpty,
    RepositoryIdInvalid,
    RepositoryMismatch,
    RequirementDuplicate,
    RequirementIdInvalid,
    RequirementMissing,
    RequirementReferenceDuplicate,
    RequirementStale,
    RevisionInvalid,
    RevisionKindUnsupported,
    RevisionMismatch,
    RowStale,
    SpecificationPathUnsafe,
    SpecificationStale,
    StalePolicy,
    StaleRegistry,
    StaleRegistryReceipt,
    VerificationRolePromotionDenied,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContentBoundRequirementIssueV1 {
    pub code: ContentBoundRequirementIssueCodeV1,
    pub field_path: String,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContentBoundRequirementVerificationV1 {
    pub mode: String,
    pub required: bool,
    pub valid: bool,
    pub disposition: String,
    // Wire compatibility: absence has an explicit bounded rollout disposition.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub evidence_identity: Option<ContentBoundDigestV1>,
    pub requirement_count: usize,
    pub evidence_row_count: usize,
    pub issues: Vec<ContentBoundRequirementIssueV1>,
    pub boundary: String,
}

#[derive(Serialize)]
struct CairnRequirementRowHashMaterial<'a> {
    requirement_id: &'a str,
    specification_path: &'a str,
    specification_blake3: &'a str,
    source_span: &'a CairnContentBoundSourceSpanV1,
    requirement_body_blake3: &'a str,
}

#[derive(Serialize)]
struct CairnRegistryHashMaterial<'a> {
    schema_version: &'a str,
    boundary: &'a str,
    repository_id: &'a str,
    revision: &'a CairnContentBoundRevisionV1,
    specification_set_blake3: &'a str,
    specifications: &'a [CairnContentBoundSpecificationIdentityV1],
    policy_blake3: &'a str,
    rows: &'a [CairnContentBoundRequirementRowV1],
    non_claims: &'a [String],
}

#[derive(Serialize)]
struct CairnReceiptHashMaterial<'a> {
    schema_version: &'a str,
    registry_blake3: &'a str,
    repository_id: &'a str,
    revision_kind: &'a str,
    revision: &'a str,
    specification_set_blake3: &'a str,
    policy_blake3: &'a str,
    row_count: usize,
    non_claims: &'a [String],
}

#[derive(Serialize)]
struct RequirementRefHashMaterial<'a> {
    schema: &'a str,
    repository_id: &'a str,
    revision: &'a StackRevisionV1,
    requirement_id: &'a str,
    specification_path: &'a str,
    specification_identity: &'a RequirementDigestV1,
    requirement_body_identity: &'a RequirementDigestV1,
    policy_identity: &'a RequirementDigestV1,
    row_identity: &'a RequirementDigestV1,
    registry_identity: &'a RequirementDigestV1,
    registry_receipt_identity: &'a RequirementDigestV1,
    verification_role: &'a ValenceVerificationRoleV1,
    non_claims: &'a [String],
}

#[derive(Serialize)]
struct EvidenceRowHashMaterial<'a> {
    schema: &'a str,
    repository_id: &'a str,
    repository_relative_path: &'a str,
    bundle_relative_path: &'a str,
    role: ContentBoundEvidenceRoleV1,
    size_bytes: u64,
    content_identity: &'a ContentBoundDigestV1,
    span: &'a Option<ContentBoundEvidenceSpanV1>,
    symbol: &'a Option<String>,
    producer_receipt_row_identity: &'a Option<ContentBoundDigestV1>,
    non_claims: &'a [String],
}

#[derive(Serialize)]
struct EvidenceManifestHashMaterial<'a> {
    schema: &'a str,
    rows: &'a [ContentBoundMeasuredEvidenceRowV1],
    non_claims: &'a [String],
}

#[derive(Serialize)]
struct CoverageRowHashMaterial<'a> {
    schema: &'a str,
    repository_id: &'a str,
    requirement_id: &'a str,
    requirement_reference_identity: &'a RequirementDigestV1,
    registry_identity: &'a RequirementDigestV1,
    evidence_role: ContentBoundEvidenceRoleV1,
    evidence_row_identities: &'a [ContentBoundDigestV1],
}

#[derive(Serialize)]
struct ReleaseLinkHashMaterial<'a> {
    release_id: &'a str,
    binary_identities: &'a [ContentBoundDigestV1],
}

#[derive(Serialize)]
struct ReleaseEvidenceHashMaterial<'a> {
    schema: &'a str,
    producer_contracts: &'a ContentBoundProducerContractsV1,
    registries: &'a [CairnContentBoundRequirementRegistryV1],
    requirement_refs: &'a [RequirementRefV1],
    evidence_manifest: &'a ContentBoundEvidenceManifestV1,
    coverage: &'a [ContentBoundCoverageRowV1],
    release_id: &'a str,
    release_identity: &'a ContentBoundDigestV1,
    binary_identities: &'a [ContentBoundDigestV1],
    non_claims: &'a [String],
}

#[must_use]
pub fn content_bound_requirement_mode_is_supported(mode: &str) -> bool {
    mode == CONTENT_BOUND_REQUIREMENT_MODE_OPTIONAL || mode == CONTENT_BOUND_REQUIREMENT_MODE_REQUIRED
}

pub fn build_content_bound_release_evidence(
    mut build: ContentBoundReleaseBuildInputV1,
) -> Result<ContentBoundReleaseEvidenceV1, Vec<ContentBoundRequirementIssueV1>> {
    let mut issues = Vec::new();
    debug_assert!(issues.capacity() >= issues.len());
    debug_assert!(issues.len() <= MAX_VALIDATION_ISSUE_COUNT);
    validate_release_input(&build.input, &mut issues);
    validate_evidence_rows(&build.evidence_rows, &mut issues);
    validate_release_id(&build.release_id, "release_id", &mut issues);
    validate_binary_digest_values(&build.binary_digests_blake3, "binary_digests_blake3", &mut issues);
    sort_issues(&mut issues);
    if !issues.is_empty() {
        return Err(issues);
    }

    canonicalize_release_build_input(&mut build);
    let evidence = assemble_content_bound_release_evidence(build)?;
    validate_built_content_bound_release(evidence)
}

fn canonicalize_release_build_input(build: &mut ContentBoundReleaseBuildInputV1) {
    build.input.registries.sort_by(|left, right| left.repository_id.cmp(&right.repository_id));
    build.input.requirement_refs.sort_by(requirement_ref_order);
    build.evidence_rows.sort_by(evidence_row_order);
}

fn assemble_content_bound_release_evidence(
    build: ContentBoundReleaseBuildInputV1,
) -> Result<ContentBoundReleaseEvidenceV1, Vec<ContentBoundRequirementIssueV1>> {
    debug_assert!(build.input.registries.capacity() >= build.input.registries.len());
    debug_assert!(build.input.requirement_refs.capacity() >= build.input.requirement_refs.len());
    let evidence_manifest = build_evidence_manifest(build.evidence_rows)?;
    let coverage = build_coverage_rows(&build.input, &evidence_manifest)?;
    let mut binary_identities = build
        .binary_digests_blake3
        .into_iter()
        .map(|blake3| ContentBoundDigestV1 {
            domain: ContentBoundDigestDomainV1::Binary,
            blake3,
        })
        .collect::<Vec<_>>();
    binary_identities.sort();
    let release_identity = ContentBoundDigestV1 {
        domain: ContentBoundDigestDomainV1::Release,
        blake3: domain_hash_json(
            RELEASE_LINK_DOMAIN,
            &ReleaseLinkHashMaterial {
                release_id: &build.release_id,
                binary_identities: &binary_identities,
            },
            "release_identity.blake3",
        )?,
    };
    let mut evidence = ContentBoundReleaseEvidenceV1 {
        schema: CONTENT_BOUND_RELEASE_EVIDENCE_SCHEMA_V1.to_string(),
        producer_contracts: ContentBoundProducerContractsV1 {
            cairn_revision: CAIRN_CONTENT_BOUND_REGISTRY_CONTRACT_REVISION.to_string(),
            cairn_registry_schema: CAIRN_CONTENT_BOUND_REQUIREMENT_REGISTRY_SCHEMA_V1.to_string(),
            valence_revision: VALENCE_CONTENT_BOUND_REQUIREMENT_CONTRACT_REVISION.to_string(),
            valence_requirement_ref_schema: VALENCE_REQUIREMENT_REF_SCHEMA_V1.to_string(),
        },
        registries: build.input.registries,
        requirement_refs: build.input.requirement_refs,
        evidence_manifest,
        coverage,
        release_id: build.release_id,
        release_identity,
        binary_identities,
        non_claims: required_non_claims(CONTENT_BOUND_RELEASE_NON_CLAIMS),
        evidence_identity: empty_content_digest(ContentBoundDigestDomainV1::ReleaseEvidence),
    };
    evidence.evidence_identity.blake3 = release_evidence_hash(&evidence)?;
    Ok(evidence)
}

fn validate_built_content_bound_release(
    evidence: ContentBoundReleaseEvidenceV1,
) -> Result<ContentBoundReleaseEvidenceV1, Vec<ContentBoundRequirementIssueV1>> {
    debug_assert!(evidence.binary_identities.capacity() >= evidence.binary_identities.len());
    debug_assert!(evidence.evidence_manifest.rows.capacity() >= evidence.evidence_manifest.rows.len());
    let binary_digests = evidence.binary_identities.iter().map(|identity| identity.blake3.clone()).collect::<Vec<_>>();
    let external_evidence = evidence
        .evidence_manifest
        .rows
        .iter()
        .map(|row| ContentBoundExternalEvidenceLinkV1 {
            role: row.role.external_role().to_string(),
            schema: CONTENT_BOUND_EVIDENCE_EXTERNAL_SCHEMA_V1.to_string(),
            bundle_relative_path: row.bundle_relative_path.clone(),
            digest_blake3: row.content_identity.blake3.clone(),
        })
        .collect::<Vec<_>>();
    let verification = validate_content_bound_release_evidence(
        &evidence,
        &evidence.release_id,
        &binary_digests,
        &external_evidence,
        CONTENT_BOUND_REQUIREMENT_MODE_REQUIRED,
    );
    if verification.valid {
        Ok(evidence)
    } else {
        Err(verification.issues)
    }
}

#[must_use]
pub fn validate_content_bound_release_evidence(
    evidence: &ContentBoundReleaseEvidenceV1,
    expected_release_id: &str,
    expected_binary_digests_blake3: &[String],
    external_evidence: &[ContentBoundExternalEvidenceLinkV1],
    mode: &str,
) -> ContentBoundRequirementVerificationV1 {
    // r[impl mantle.release_provenance.content_bound_requirement_coverage]
    let is_required = mode == CONTENT_BOUND_REQUIREMENT_MODE_REQUIRED;
    let mut issues = Vec::new();
    debug_assert!(issues.capacity() >= issues.len());
    debug_assert!(issues.len() <= MAX_VALIDATION_ISSUE_COUNT);
    if !content_bound_requirement_mode_is_supported(mode) {
        push_issue(
            &mut issues,
            ContentBoundRequirementIssueCodeV1::InputSchemaUnsupported,
            "mode",
            "content-bound requirement policy mode is unsupported",
        );
    }
    validate_release_evidence_shape(evidence, &mut issues);
    validate_release_input(
        &ContentBoundReleaseInputV1 {
            schema: CONTENT_BOUND_RELEASE_INPUT_SCHEMA_V1.to_string(),
            registries: evidence.registries.clone(),
            requirement_refs: evidence.requirement_refs.clone(),
            coverage: evidence
                .coverage
                .iter()
                .map(|row| ContentBoundCoverageDraftV1 {
                    requirement_reference_blake3: row.requirement_reference_identity.blake3.clone(),
                    evidence_role: row.evidence_role,
                    evidence_repository_paths: evidence_paths_for_coverage(row, &evidence.evidence_manifest.rows),
                })
                .collect(),
            non_claims: evidence.non_claims.clone(),
        },
        &mut issues,
    );
    validate_evidence_manifest(&evidence.evidence_manifest, &mut issues);
    validate_coverage_rows(
        &evidence.coverage,
        &evidence.requirement_refs,
        &evidence.registries,
        &evidence.evidence_manifest.rows,
        &mut issues,
    );
    validate_external_links(&evidence.evidence_manifest.rows, external_evidence, &mut issues);
    validate_release_links(evidence, expected_release_id, expected_binary_digests_blake3, &mut issues);
    if release_evidence_hash(evidence).is_ok_and(|expected| expected != evidence.evidence_identity.blake3) {
        push_issue(
            &mut issues,
            ContentBoundRequirementIssueCodeV1::ReleaseEvidenceIdentityStale,
            "evidence_identity.blake3",
            "content-bound release evidence identity does not match canonical fields",
        );
    }
    sort_issues(&mut issues);
    let is_valid = issues.is_empty();
    ContentBoundRequirementVerificationV1 {
        mode: mode.to_string(),
        required: is_required,
        valid: is_valid,
        disposition: if is_valid {
            CONTENT_BOUND_REQUIREMENT_DISPOSITION_PRESENT.to_string()
        } else {
            CONTENT_BOUND_REQUIREMENT_DISPOSITION_INVALID.to_string()
        },
        evidence_identity: Some(evidence.evidence_identity.clone()),
        requirement_count: evidence.requirement_refs.len(),
        evidence_row_count: evidence.evidence_manifest.rows.len(),
        issues,
        boundary: CONTENT_BOUND_RELEASE_BOUNDARY.to_string(),
    }
}

#[must_use]
pub fn evaluate_optional_content_bound_release_evidence(
    evidence: Option<&ContentBoundReleaseEvidenceV1>,
    expected_release_id: &str,
    expected_binary_digests_blake3: &[String],
    external_evidence: &[ContentBoundExternalEvidenceLinkV1],
    mode: &str,
) -> ContentBoundRequirementVerificationV1 {
    let is_required = mode == CONTENT_BOUND_REQUIREMENT_MODE_REQUIRED;
    let Some(evidence) = evidence else {
        let mut issues = Vec::new();
        debug_assert!(issues.capacity() >= issues.len());
        debug_assert!(issues.len() <= MAX_VALIDATION_ISSUE_COUNT);
        if !content_bound_requirement_mode_is_supported(mode) {
            push_issue(
                &mut issues,
                ContentBoundRequirementIssueCodeV1::InputSchemaUnsupported,
                "mode",
                "content-bound requirement policy mode is unsupported",
            );
        }
        if is_required {
            push_issue(
                &mut issues,
                ContentBoundRequirementIssueCodeV1::CoverageMissing,
                "content_bound_requirement_evidence",
                "strict content-bound requirement evidence is missing",
            );
        }
        sort_issues(&mut issues);
        return ContentBoundRequirementVerificationV1 {
            mode: mode.to_string(),
            required: is_required,
            valid: issues.is_empty(),
            disposition: CONTENT_BOUND_REQUIREMENT_DISPOSITION_ABSENT.to_string(),
            evidence_identity: None,
            requirement_count: 0,
            evidence_row_count: 0,
            issues,
            boundary: if is_required {
                CONTENT_BOUND_RELEASE_BOUNDARY.to_string()
            } else {
                LEGACY_COVERAGE_BOUNDARY.to_string()
            },
        };
    };
    validate_content_bound_release_evidence(
        evidence,
        expected_release_id,
        expected_binary_digests_blake3,
        external_evidence,
        mode,
    )
}

fn validate_release_input(input: &ContentBoundReleaseInputV1, issues: &mut Vec<ContentBoundRequirementIssueV1>) {
    debug_assert!(issues.capacity() >= issues.len());
    debug_assert!(issues.len() <= MAX_VALIDATION_ISSUE_COUNT);
    if input.schema != CONTENT_BOUND_RELEASE_INPUT_SCHEMA_V1 {
        push_issue(
            issues,
            ContentBoundRequirementIssueCodeV1::InputSchemaUnsupported,
            "input.schema",
            "content-bound release input schema is unsupported",
        );
    }
    validate_collection_count(input.registries.len(), MAX_REGISTRY_COUNT, "input.registries", issues);
    validate_collection_count(
        input.requirement_refs.len(),
        MAX_REQUIREMENT_REFERENCE_COUNT,
        "input.requirement_refs",
        issues,
    );
    validate_collection_count(input.coverage.len(), MAX_COVERAGE_ROW_COUNT, "input.coverage", issues);
    if input.registries.is_empty() || input.requirement_refs.is_empty() || input.coverage.is_empty() {
        push_issue(
            issues,
            ContentBoundRequirementIssueCodeV1::ReleaseInputEmpty,
            "input",
            "strict release input requires registries, requirement references, and coverage",
        );
    }
    validate_required_non_claims(&input.non_claims, CONTENT_BOUND_RELEASE_NON_CLAIMS, "input.non_claims", issues);
    let mut registry_index = BTreeMap::new();
    for (index, registry) in input.registries.iter().enumerate() {
        validate_cairn_registry(registry, &format!("input.registries[{index}]"), issues);
        if registry_index.insert(registry.repository_id.as_str(), registry).is_some() {
            push_issue(
                issues,
                ContentBoundRequirementIssueCodeV1::RegistryDuplicate,
                &format!("input.registries[{index}].repository_id"),
                "more than one registry was supplied for a repository",
            );
        }
    }
    let mut references = BTreeSet::new();
    for (index, reference) in input.requirement_refs.iter().enumerate() {
        let path = format!("input.requirement_refs[{index}]");
        if !references.insert((reference.repository_id.as_str(), reference.requirement_id.as_str())) {
            push_issue(
                issues,
                ContentBoundRequirementIssueCodeV1::RequirementReferenceDuplicate,
                &path,
                "content-bound requirement reference occurs more than once",
            );
        }
        match registry_index.get(reference.repository_id.as_str()) {
            Some(registry) => validate_requirement_ref(reference, registry, &path, issues),
            None => push_issue(
                issues,
                ContentBoundRequirementIssueCodeV1::RegistryMissing,
                &format!("{path}.repository_id"),
                "requirement reference has no supplied registry for its repository",
            ),
        }
    }
}

fn validate_cairn_registry(
    registry: &CairnContentBoundRequirementRegistryV1,
    prefix: &str,
    issues: &mut Vec<ContentBoundRequirementIssueV1>,
) {
    debug_assert!(issues.capacity() >= issues.len());
    debug_assert!(issues.len() <= MAX_VALIDATION_ISSUE_COUNT);
    if registry.schema_version != CAIRN_CONTENT_BOUND_REQUIREMENT_REGISTRY_SCHEMA_V1 {
        push_issue(
            issues,
            ContentBoundRequirementIssueCodeV1::RegistrySchemaUnsupported,
            &format!("{prefix}.schema_version"),
            "supplied Cairn registry schema is unsupported",
        );
    }
    if registry.boundary != CAIRN_CONTENT_BOUND_REQUIREMENT_REGISTRY_BOUNDARY {
        push_issue(
            issues,
            ContentBoundRequirementIssueCodeV1::RegistryBoundaryWeakened,
            &format!("{prefix}.boundary"),
            "supplied Cairn registry boundary is missing or changed",
        );
    }
    validate_repository_id(&registry.repository_id, &format!("{prefix}.repository_id"), issues);
    validate_cairn_revision(&registry.revision, &format!("{prefix}.revision"), issues);
    validate_required_non_claims(
        &registry.non_claims,
        CAIRN_REGISTRY_NON_CLAIMS,
        &format!("{prefix}.non_claims"),
        issues,
    );
    validate_blake3(&registry.specification_set_blake3, &format!("{prefix}.specification_set_blake3"), issues);
    validate_blake3(&registry.policy_blake3, &format!("{prefix}.policy_blake3"), issues);
    validate_blake3(&registry.registry_blake3, &format!("{prefix}.registry_blake3"), issues);
    validate_cairn_specifications(registry, prefix, issues);
    validate_cairn_rows(registry, prefix, issues);
    validate_cairn_registry_hash(registry, prefix, issues);
    validate_cairn_receipt(registry, prefix, issues);
}

fn validate_cairn_specifications(
    registry: &CairnContentBoundRequirementRegistryV1,
    prefix: &str,
    issues: &mut Vec<ContentBoundRequirementIssueV1>,
) {
    debug_assert!(issues.capacity() >= issues.len());
    debug_assert!(issues.len() <= MAX_VALIDATION_ISSUE_COUNT);
    let mut canonical = registry.specifications.clone();
    canonical.sort_by(|left, right| left.path.cmp(&right.path));
    if canonical != registry.specifications {
        push_issue(
            issues,
            ContentBoundRequirementIssueCodeV1::RegistryOrderInvalid,
            &format!("{prefix}.specifications"),
            "registry specifications are not in canonical path order",
        );
    }
    let mut seen = BTreeSet::new();
    for (index, specification) in registry.specifications.iter().enumerate() {
        let row = format!("{prefix}.specifications[{index}]");
        validate_specification_path(&specification.path, &format!("{row}.path"), issues);
        validate_blake3(&specification.content_blake3, &format!("{row}.content_blake3"), issues);
        if !seen.insert(specification.path.as_str()) {
            push_issue(
                issues,
                ContentBoundRequirementIssueCodeV1::RegistrySpecificationDuplicate,
                &format!("{row}.path"),
                "registry specification path occurs more than once",
            );
        }
    }
    if domain_hash_json(
        CAIRN_SPECIFICATION_SET_DOMAIN,
        &registry.specifications,
        &format!("{prefix}.specification_set_blake3"),
    )
    .is_ok_and(|expected| expected != registry.specification_set_blake3)
    {
        push_issue(
            issues,
            ContentBoundRequirementIssueCodeV1::RegistrySpecificationSetHashInvalid,
            &format!("{prefix}.specification_set_blake3"),
            "registry specification-set identity is not self-consistent",
        );
    }
}

fn validate_cairn_rows(
    registry: &CairnContentBoundRequirementRegistryV1,
    prefix: &str,
    issues: &mut Vec<ContentBoundRequirementIssueV1>,
) {
    debug_assert!(issues.capacity() >= issues.len());
    debug_assert!(issues.len() <= MAX_VALIDATION_ISSUE_COUNT);
    if registry.rows.is_empty() {
        push_issue(
            issues,
            ContentBoundRequirementIssueCodeV1::RegistryRequirementMissing,
            &format!("{prefix}.rows"),
            "registry has no requirement rows",
        );
    }
    let mut canonical = registry.rows.clone();
    canonical.sort_by(|left, right| {
        left.requirement_id
            .cmp(&right.requirement_id)
            .then(left.specification_path.cmp(&right.specification_path))
    });
    if canonical != registry.rows {
        push_issue(
            issues,
            ContentBoundRequirementIssueCodeV1::RegistryOrderInvalid,
            &format!("{prefix}.rows"),
            "registry requirement rows are not in canonical order",
        );
    }
    let specification_index = registry
        .specifications
        .iter()
        .map(|specification| (specification.path.as_str(), specification.content_blake3.as_str()))
        .collect::<BTreeMap<_, _>>();
    let mut seen = BTreeSet::new();
    for (index, row) in registry.rows.iter().enumerate() {
        let row_path = format!("{prefix}.rows[{index}]");
        validate_cairn_row(row, &row_path, &specification_index, &mut seen, issues);
    }
}

fn validate_cairn_row<'a>(
    row: &'a CairnContentBoundRequirementRowV1,
    row_path: &str,
    specification_index: &BTreeMap<&str, &str>,
    seen: &mut BTreeSet<&'a str>,
    issues: &mut Vec<ContentBoundRequirementIssueV1>,
) {
    debug_assert!(issues.capacity() >= issues.len());
    debug_assert!(issues.len() <= MAX_VALIDATION_ISSUE_COUNT);
    validate_requirement_id(&row.requirement_id, &format!("{row_path}.requirement_id"), issues);
    if !seen.insert(row.requirement_id.as_str()) {
        push_issue(
            issues,
            ContentBoundRequirementIssueCodeV1::RegistryRequirementDuplicate,
            &format!("{row_path}.requirement_id"),
            "registry requirement ID occurs more than once",
        );
    }
    validate_specification_path(&row.specification_path, &format!("{row_path}.specification_path"), issues);
    validate_blake3(&row.specification_blake3, &format!("{row_path}.specification_blake3"), issues);
    validate_blake3(&row.requirement_body_blake3, &format!("{row_path}.requirement_body_blake3"), issues);
    validate_blake3(&row.row_blake3, &format!("{row_path}.row_blake3"), issues);
    if row.source_span.start_byte >= row.source_span.end_byte || row.source_span.start_line > row.source_span.end_line {
        push_issue(
            issues,
            ContentBoundRequirementIssueCodeV1::EvidenceSpanInvalid,
            &format!("{row_path}.source_span"),
            "registry requirement source span is empty or unordered",
        );
    }
    match specification_index.get(row.specification_path.as_str()) {
        Some(expected) if *expected == row.specification_blake3 => {}
        Some(_) => push_issue(
            issues,
            ContentBoundRequirementIssueCodeV1::RegistrySpecificationHashInvalid,
            &format!("{row_path}.specification_blake3"),
            "registry row specification identity does not match its specification",
        ),
        None => push_issue(
            issues,
            ContentBoundRequirementIssueCodeV1::RegistrySpecificationMissing,
            &format!("{row_path}.specification_path"),
            "registry row references an absent specification",
        ),
    }
    let material = CairnRequirementRowHashMaterial {
        requirement_id: &row.requirement_id,
        specification_path: &row.specification_path,
        specification_blake3: &row.specification_blake3,
        source_span: &row.source_span,
        requirement_body_blake3: &row.requirement_body_blake3,
    };
    if domain_hash_json(CAIRN_REQUIREMENT_ROW_DOMAIN, &material, &format!("{row_path}.row_blake3"))
        .is_ok_and(|expected| expected != row.row_blake3)
    {
        push_issue(
            issues,
            ContentBoundRequirementIssueCodeV1::RegistryRowHashInvalid,
            &format!("{row_path}.row_blake3"),
            "registry row identity is not self-consistent",
        );
    }
}

fn validate_cairn_registry_hash(
    registry: &CairnContentBoundRequirementRegistryV1,
    prefix: &str,
    issues: &mut Vec<ContentBoundRequirementIssueV1>,
) {
    debug_assert!(issues.capacity() >= issues.len());
    debug_assert!(issues.len() <= MAX_VALIDATION_ISSUE_COUNT);
    let material = CairnRegistryHashMaterial {
        schema_version: &registry.schema_version,
        boundary: &registry.boundary,
        repository_id: &registry.repository_id,
        revision: &registry.revision,
        specification_set_blake3: &registry.specification_set_blake3,
        specifications: &registry.specifications,
        policy_blake3: &registry.policy_blake3,
        rows: &registry.rows,
        non_claims: &registry.non_claims,
    };
    if domain_hash_json(CAIRN_REGISTRY_DOMAIN, &material, &format!("{prefix}.registry_blake3"))
        .is_ok_and(|expected| expected != registry.registry_blake3)
    {
        push_issue(
            issues,
            ContentBoundRequirementIssueCodeV1::RegistryHashInvalid,
            &format!("{prefix}.registry_blake3"),
            "registry identity is not self-consistent",
        );
    }
}

fn validate_cairn_receipt(
    registry: &CairnContentBoundRequirementRegistryV1,
    prefix: &str,
    issues: &mut Vec<ContentBoundRequirementIssueV1>,
) {
    debug_assert!(issues.capacity() >= issues.len());
    debug_assert!(issues.len() <= MAX_VALIDATION_ISSUE_COUNT);
    let receipt = &registry.receipt;
    if receipt.schema_version != CAIRN_CONTENT_BOUND_REQUIREMENT_REGISTRY_RECEIPT_SCHEMA_V1 {
        push_issue(
            issues,
            ContentBoundRequirementIssueCodeV1::RegistryReceiptSchemaUnsupported,
            &format!("{prefix}.receipt.schema_version"),
            "supplied Cairn registry receipt schema is unsupported",
        );
    }
    validate_blake3(&receipt.receipt_blake3, &format!("{prefix}.receipt.receipt_blake3"), issues);
    validate_required_non_claims(
        &receipt.non_claims,
        CAIRN_REGISTRY_NON_CLAIMS,
        &format!("{prefix}.receipt.non_claims"),
        issues,
    );
    let is_linkage_consistent = receipt.registry_blake3 == registry.registry_blake3
        && receipt.repository_id == registry.repository_id
        && receipt.revision_kind == registry.revision.kind
        && receipt.revision == registry.revision.value
        && receipt.specification_set_blake3 == registry.specification_set_blake3
        && receipt.policy_blake3 == registry.policy_blake3
        && receipt.row_count == registry.rows.len()
        && receipt.non_claims == registry.non_claims;
    if !is_linkage_consistent {
        push_issue(
            issues,
            ContentBoundRequirementIssueCodeV1::RegistryReceiptLinkMismatch,
            &format!("{prefix}.receipt"),
            "registry receipt fields do not match the registry",
        );
    }
    let material = CairnReceiptHashMaterial {
        schema_version: &receipt.schema_version,
        registry_blake3: &receipt.registry_blake3,
        repository_id: &receipt.repository_id,
        revision_kind: &receipt.revision_kind,
        revision: &receipt.revision,
        specification_set_blake3: &receipt.specification_set_blake3,
        policy_blake3: &receipt.policy_blake3,
        row_count: receipt.row_count,
        non_claims: &receipt.non_claims,
    };
    if domain_hash_json(CAIRN_RECEIPT_DOMAIN, &material, &format!("{prefix}.receipt.receipt_blake3"))
        .is_ok_and(|expected| expected != receipt.receipt_blake3)
    {
        push_issue(
            issues,
            ContentBoundRequirementIssueCodeV1::RegistryReceiptHashInvalid,
            &format!("{prefix}.receipt.receipt_blake3"),
            "registry receipt identity is not self-consistent",
        );
    }
}

fn validate_requirement_ref(
    reference: &RequirementRefV1,
    registry: &CairnContentBoundRequirementRegistryV1,
    prefix: &str,
    issues: &mut Vec<ContentBoundRequirementIssueV1>,
) {
    debug_assert!(issues.capacity() >= issues.len());
    debug_assert!(issues.len() <= MAX_VALIDATION_ISSUE_COUNT);
    if reference.schema != VALENCE_REQUIREMENT_REF_SCHEMA_V1 {
        push_issue(
            issues,
            ContentBoundRequirementIssueCodeV1::ReferenceSchemaUnsupported,
            &format!("{prefix}.schema"),
            "Valence requirement reference schema is unsupported",
        );
    }
    validate_repository_id(&reference.repository_id, &format!("{prefix}.repository_id"), issues);
    validate_stack_revision(&reference.revision, &format!("{prefix}.revision"), issues);
    validate_requirement_id(&reference.requirement_id, &format!("{prefix}.requirement_id"), issues);
    validate_specification_path(&reference.specification_path, &format!("{prefix}.specification_path"), issues);
    for (digest, domain, field) in [
        (
            &reference.specification_identity,
            RequirementDigestDomainV1::Specification,
            "specification_identity",
        ),
        (
            &reference.requirement_body_identity,
            RequirementDigestDomainV1::RequirementBody,
            "requirement_body_identity",
        ),
        (&reference.policy_identity, RequirementDigestDomainV1::Policy, "policy_identity"),
        (&reference.row_identity, RequirementDigestDomainV1::RequirementRow, "row_identity"),
        (&reference.registry_identity, RequirementDigestDomainV1::RequirementRegistry, "registry_identity"),
        (
            &reference.registry_receipt_identity,
            RequirementDigestDomainV1::RequirementRegistryReceipt,
            "registry_receipt_identity",
        ),
        (&reference.reference_identity, RequirementDigestDomainV1::RequirementReference, "reference_identity"),
    ] {
        validate_requirement_digest(digest, domain, &format!("{prefix}.{field}"), issues);
    }
    if reference.verification_role != ValenceVerificationRoleV1::RecordedOnly {
        push_issue(
            issues,
            ContentBoundRequirementIssueCodeV1::VerificationRolePromotionDenied,
            &format!("{prefix}.verification_role"),
            "Valence content-bound requirement reference must remain recorded-only",
        );
    }
    validate_required_non_claims(
        &reference.non_claims,
        VALENCE_REQUIREMENT_NON_CLAIMS,
        &format!("{prefix}.non_claims"),
        issues,
    );
    if requirement_ref_hash(reference).is_ok_and(|expected| expected != reference.reference_identity.blake3) {
        push_issue(
            issues,
            ContentBoundRequirementIssueCodeV1::ReferenceIdentityStale,
            &format!("{prefix}.reference_identity.blake3"),
            "Valence requirement reference identity does not match canonical fields",
        );
    }
    compare_requirement_ref_to_registry(reference, registry, prefix, issues);
}

fn compare_requirement_ref_to_registry(
    reference: &RequirementRefV1,
    registry: &CairnContentBoundRequirementRegistryV1,
    prefix: &str,
    issues: &mut Vec<ContentBoundRequirementIssueV1>,
) {
    debug_assert!(issues.capacity() >= issues.len());
    debug_assert!(issues.len() <= MAX_VALIDATION_ISSUE_COUNT);
    if reference.repository_id != registry.repository_id {
        push_issue(
            issues,
            ContentBoundRequirementIssueCodeV1::RepositoryMismatch,
            &format!("{prefix}.repository_id"),
            "requirement reference repository does not match its registry",
        );
    }
    if reference.revision.kind.as_str() != registry.revision.kind || reference.revision.value != registry.revision.value
    {
        push_issue(
            issues,
            ContentBoundRequirementIssueCodeV1::RevisionMismatch,
            &format!("{prefix}.revision"),
            "requirement reference revision does not match its registry",
        );
    }
    compare_digest(
        &reference.policy_identity.blake3,
        &registry.policy_blake3,
        ContentBoundRequirementIssueCodeV1::StalePolicy,
        &format!("{prefix}.policy_identity.blake3"),
        issues,
    );
    compare_digest(
        &reference.registry_identity.blake3,
        &registry.registry_blake3,
        ContentBoundRequirementIssueCodeV1::StaleRegistry,
        &format!("{prefix}.registry_identity.blake3"),
        issues,
    );
    compare_digest(
        &reference.registry_receipt_identity.blake3,
        &registry.receipt.receipt_blake3,
        ContentBoundRequirementIssueCodeV1::StaleRegistryReceipt,
        &format!("{prefix}.registry_receipt_identity.blake3"),
        issues,
    );
    compare_requirement_ref_to_registry_row(reference, registry, prefix, issues);
}

fn compare_requirement_ref_to_registry_row(
    reference: &RequirementRefV1,
    registry: &CairnContentBoundRequirementRegistryV1,
    prefix: &str,
    issues: &mut Vec<ContentBoundRequirementIssueV1>,
) {
    debug_assert!(issues.capacity() >= issues.len());
    debug_assert!(issues.len() <= MAX_VALIDATION_ISSUE_COUNT);
    let matches = registry
        .rows
        .iter()
        .filter(|row| row.requirement_id == reference.requirement_id)
        .collect::<Vec<_>>();
    if matches.is_empty() {
        push_issue(
            issues,
            ContentBoundRequirementIssueCodeV1::RequirementMissing,
            &format!("{prefix}.requirement_id"),
            "requirement is absent from its supplied registry",
        );
        return;
    }
    if matches.len() > 1 {
        push_issue(
            issues,
            ContentBoundRequirementIssueCodeV1::RequirementDuplicate,
            &format!("{prefix}.requirement_id"),
            "requirement occurs more than once in its supplied registry",
        );
        return;
    }
    let row = matches[0];
    if reference.specification_path != row.specification_path {
        push_issue(
            issues,
            ContentBoundRequirementIssueCodeV1::SpecificationStale,
            &format!("{prefix}.specification_path"),
            "requirement reference specification path does not match its registry row",
        );
    }
    compare_digest(
        &reference.specification_identity.blake3,
        &row.specification_blake3,
        ContentBoundRequirementIssueCodeV1::SpecificationStale,
        &format!("{prefix}.specification_identity.blake3"),
        issues,
    );
    compare_digest(
        &reference.requirement_body_identity.blake3,
        &row.requirement_body_blake3,
        ContentBoundRequirementIssueCodeV1::RequirementStale,
        &format!("{prefix}.requirement_body_identity.blake3"),
        issues,
    );
    compare_digest(
        &reference.row_identity.blake3,
        &row.row_blake3,
        ContentBoundRequirementIssueCodeV1::RowStale,
        &format!("{prefix}.row_identity.blake3"),
        issues,
    );
}

fn validate_evidence_rows(
    rows: &[ContentBoundMeasuredEvidenceRowV1],
    issues: &mut Vec<ContentBoundRequirementIssueV1>,
) {
    debug_assert!(issues.capacity() >= issues.len());
    debug_assert!(issues.len() <= MAX_VALIDATION_ISSUE_COUNT);
    validate_collection_count(rows.len(), MAX_EVIDENCE_ROW_COUNT, "evidence_rows", issues);
    if rows.is_empty() {
        push_issue(
            issues,
            ContentBoundRequirementIssueCodeV1::EvidenceRowMissing,
            "evidence_rows",
            "strict release evidence requires at least one measured evidence row",
        );
    }
    let mut repository_paths = BTreeSet::new();
    let mut bundle_paths = BTreeSet::new();
    for (index, row) in rows.iter().enumerate() {
        let prefix = format!("evidence_rows[{index}]");
        validate_evidence_row(row, &prefix, issues);
        if !repository_paths.insert((row.repository_id.as_str(), row.repository_relative_path.as_str())) {
            push_issue(
                issues,
                ContentBoundRequirementIssueCodeV1::EvidencePathDuplicate,
                &format!("{prefix}.repository_relative_path"),
                "repository evidence path occurs more than once",
            );
        }
        if !bundle_paths.insert(row.bundle_relative_path.as_str()) {
            push_issue(
                issues,
                ContentBoundRequirementIssueCodeV1::EvidencePathDuplicate,
                &format!("{prefix}.bundle_relative_path"),
                "bundle evidence path occurs more than once",
            );
        }
    }
    validate_evidence_receipt_links(rows, issues);
}

fn validate_evidence_row(
    row: &ContentBoundMeasuredEvidenceRowV1,
    prefix: &str,
    issues: &mut Vec<ContentBoundRequirementIssueV1>,
) {
    debug_assert!(issues.capacity() >= issues.len());
    debug_assert!(issues.len() <= MAX_VALIDATION_ISSUE_COUNT);
    if row.schema != CONTENT_BOUND_EVIDENCE_ROW_SCHEMA_V1 {
        push_issue(
            issues,
            ContentBoundRequirementIssueCodeV1::InputSchemaUnsupported,
            &format!("{prefix}.schema"),
            "content-bound evidence row schema is unsupported",
        );
    }
    validate_repository_id(&row.repository_id, &format!("{prefix}.repository_id"), issues);
    validate_safe_relative_path(
        &row.repository_relative_path,
        &format!("{prefix}.repository_relative_path"),
        false,
        issues,
    );
    validate_safe_relative_path(&row.bundle_relative_path, &format!("{prefix}.bundle_relative_path"), true, issues);
    if row.size_bytes == 0 {
        push_issue(
            issues,
            ContentBoundRequirementIssueCodeV1::EvidenceSpanInvalid,
            &format!("{prefix}.size_bytes"),
            "evidence file size must be non-zero",
        );
    }
    validate_content_digest(
        &row.content_identity,
        ContentBoundDigestDomainV1::EvidenceFile,
        &format!("{prefix}.content_identity"),
        issues,
    );
    validate_content_digest(
        &row.row_identity,
        ContentBoundDigestDomainV1::EvidenceRow,
        &format!("{prefix}.row_identity"),
        issues,
    );
    validate_evidence_span(row, prefix, issues);
    validate_evidence_row_metadata(row, prefix, issues);
}

fn validate_evidence_row_metadata(
    row: &ContentBoundMeasuredEvidenceRowV1,
    prefix: &str,
    issues: &mut Vec<ContentBoundRequirementIssueV1>,
) {
    debug_assert!(issues.capacity() >= issues.len());
    debug_assert!(issues.len() <= MAX_VALIDATION_ISSUE_COUNT);
    if let Some(symbol) = &row.symbol {
        let is_valid = !symbol.is_empty()
            && symbol.len() <= MAX_SYMBOL_BYTES
            && symbol.trim() == symbol
            && !symbol.chars().any(char::is_control);
        if !is_valid {
            push_issue(
                issues,
                ContentBoundRequirementIssueCodeV1::EvidenceSymbolInvalid,
                &format!("{prefix}.symbol"),
                "evidence symbol must be bounded, trimmed, and free of control characters",
            );
        }
    }
    if let Some(receipt) = &row.producer_receipt_row_identity {
        validate_content_digest(
            receipt,
            ContentBoundDigestDomainV1::EvidenceRow,
            &format!("{prefix}.producer_receipt_row_identity"),
            issues,
        );
    }
    validate_required_non_claims(
        &row.non_claims,
        CONTENT_BOUND_EVIDENCE_ROW_NON_CLAIMS,
        &format!("{prefix}.non_claims"),
        issues,
    );
    if evidence_row_hash(row).is_ok_and(|expected| expected != row.row_identity.blake3) {
        push_issue(
            issues,
            ContentBoundRequirementIssueCodeV1::EvidenceRowIdentityStale,
            &format!("{prefix}.row_identity.blake3"),
            "content-bound evidence row identity does not match canonical fields",
        );
    }
}

fn validate_evidence_span(
    row: &ContentBoundMeasuredEvidenceRowV1,
    prefix: &str,
    issues: &mut Vec<ContentBoundRequirementIssueV1>,
) {
    let Some(span) = &row.span else {
        return;
    };
    let is_invalid =
        span.start_byte >= span.end_byte || span.end_byte > row.size_bytes || span.start_line > span.end_line;
    if is_invalid {
        push_issue(
            issues,
            ContentBoundRequirementIssueCodeV1::EvidenceSpanInvalid,
            &format!("{prefix}.span"),
            "evidence span is empty, unordered, or outside measured bytes",
        );
    }
}

fn validate_evidence_receipt_links(
    rows: &[ContentBoundMeasuredEvidenceRowV1],
    issues: &mut Vec<ContentBoundRequirementIssueV1>,
) {
    debug_assert!(issues.capacity() >= issues.len());
    debug_assert!(issues.len() <= MAX_VALIDATION_ISSUE_COUNT);
    let receipt_rows = rows
        .iter()
        .filter(|row| row.role == ContentBoundEvidenceRoleV1::Receipt)
        .map(|row| (row.row_identity.blake3.as_str(), row.repository_id.as_str()))
        .collect::<BTreeMap<_, _>>();
    for (index, row) in rows.iter().enumerate() {
        if row.role == ContentBoundEvidenceRoleV1::Receipt {
            if row.producer_receipt_row_identity.is_some() {
                push_issue(
                    issues,
                    ContentBoundRequirementIssueCodeV1::EvidenceReceiptLinkInvalid,
                    &format!("evidence_rows[{index}].producer_receipt_row_identity"),
                    "receipt evidence row cannot name itself as a parent receipt",
                );
            }
            continue;
        }
        let Some(receipt) = &row.producer_receipt_row_identity else {
            push_issue(
                issues,
                ContentBoundRequirementIssueCodeV1::EvidenceReceiptLinkMissing,
                &format!("evidence_rows[{index}].producer_receipt_row_identity"),
                "source, test, and proof evidence require a bundled producer receipt row",
            );
            continue;
        };
        match receipt_rows.get(receipt.blake3.as_str()) {
            Some(repository_id) if *repository_id == row.repository_id => {}
            Some(_) => push_issue(
                issues,
                ContentBoundRequirementIssueCodeV1::EvidenceReceiptLinkInvalid,
                &format!("evidence_rows[{index}].producer_receipt_row_identity.blake3"),
                "producer receipt identity names a receipt from a different repository",
            ),
            None => push_issue(
                issues,
                ContentBoundRequirementIssueCodeV1::EvidenceReceiptLinkInvalid,
                &format!("evidence_rows[{index}].producer_receipt_row_identity.blake3"),
                "producer receipt identity does not name a bundled receipt evidence row",
            ),
        }
    }
}

fn build_evidence_manifest(
    rows: Vec<ContentBoundMeasuredEvidenceRowV1>,
) -> Result<ContentBoundEvidenceManifestV1, Vec<ContentBoundRequirementIssueV1>> {
    debug_assert!(rows.capacity() >= rows.len());
    debug_assert!(!CONTENT_BOUND_RELEASE_NON_CLAIMS.is_empty());
    let non_claims = required_non_claims(CONTENT_BOUND_RELEASE_NON_CLAIMS);
    let identity = domain_hash_json(
        EVIDENCE_MANIFEST_DOMAIN,
        &EvidenceManifestHashMaterial {
            schema: CONTENT_BOUND_EVIDENCE_MANIFEST_SCHEMA_V1,
            rows: &rows,
            non_claims: &non_claims,
        },
        "evidence_manifest.manifest_identity.blake3",
    )?;
    Ok(ContentBoundEvidenceManifestV1 {
        schema: CONTENT_BOUND_EVIDENCE_MANIFEST_SCHEMA_V1.to_string(),
        rows,
        non_claims,
        manifest_identity: ContentBoundDigestV1 {
            domain: ContentBoundDigestDomainV1::EvidenceManifest,
            blake3: identity,
        },
    })
}

fn validate_evidence_manifest(
    manifest: &ContentBoundEvidenceManifestV1,
    issues: &mut Vec<ContentBoundRequirementIssueV1>,
) {
    debug_assert!(issues.capacity() >= issues.len());
    debug_assert!(issues.len() <= MAX_VALIDATION_ISSUE_COUNT);
    if manifest.schema != CONTENT_BOUND_EVIDENCE_MANIFEST_SCHEMA_V1 {
        push_issue(
            issues,
            ContentBoundRequirementIssueCodeV1::InputSchemaUnsupported,
            "evidence_manifest.schema",
            "content-bound evidence manifest schema is unsupported",
        );
    }
    validate_evidence_rows(&manifest.rows, issues);
    validate_required_non_claims(
        &manifest.non_claims,
        CONTENT_BOUND_RELEASE_NON_CLAIMS,
        "evidence_manifest.non_claims",
        issues,
    );
    validate_content_digest(
        &manifest.manifest_identity,
        ContentBoundDigestDomainV1::EvidenceManifest,
        "evidence_manifest.manifest_identity",
        issues,
    );
    if domain_hash_json(
        EVIDENCE_MANIFEST_DOMAIN,
        &EvidenceManifestHashMaterial {
            schema: &manifest.schema,
            rows: &manifest.rows,
            non_claims: &manifest.non_claims,
        },
        "evidence_manifest.manifest_identity.blake3",
    )
    .is_ok_and(|expected| expected != manifest.manifest_identity.blake3)
    {
        push_issue(
            issues,
            ContentBoundRequirementIssueCodeV1::EvidenceManifestIdentityStale,
            "evidence_manifest.manifest_identity.blake3",
            "content-bound evidence manifest identity does not match canonical rows",
        );
    }
}

struct CoverageBuildIndexes<'a> {
    references: BTreeMap<&'a str, &'a RequirementRefV1>,
    registries: BTreeMap<&'a str, &'a CairnContentBoundRequirementRegistryV1>,
    evidence_by_path: BTreeMap<(&'a str, &'a str), &'a ContentBoundMeasuredEvidenceRowV1>,
}

fn build_coverage_rows(
    input: &ContentBoundReleaseInputV1,
    manifest: &ContentBoundEvidenceManifestV1,
) -> Result<Vec<ContentBoundCoverageRowV1>, Vec<ContentBoundRequirementIssueV1>> {
    let indexes = CoverageBuildIndexes {
        references: input
            .requirement_refs
            .iter()
            .map(|reference| (reference.reference_identity.blake3.as_str(), reference))
            .collect(),
        registries: input.registries.iter().map(|registry| (registry.repository_id.as_str(), registry)).collect(),
        evidence_by_path: manifest
            .rows
            .iter()
            .map(|row| ((row.repository_id.as_str(), row.repository_relative_path.as_str()), row))
            .collect(),
    };
    let mut rows = Vec::with_capacity(input.coverage.len());
    let mut issues = Vec::new();
    debug_assert!(issues.capacity() >= issues.len());
    debug_assert!(issues.len() <= MAX_VALIDATION_ISSUE_COUNT);
    for (row_index, draft) in input.coverage.iter().enumerate() {
        if let Some(row) = build_coverage_row(draft, row_index, &indexes, &mut issues)? {
            rows.push(row);
        }
    }
    rows.sort_by(coverage_row_order);
    validate_coverage_rows(&rows, &input.requirement_refs, &input.registries, &manifest.rows, &mut issues);
    sort_issues(&mut issues);
    if issues.is_empty() { Ok(rows) } else { Err(issues) }
}

fn build_coverage_row(
    draft: &ContentBoundCoverageDraftV1,
    row_index: usize,
    indexes: &CoverageBuildIndexes<'_>,
    issues: &mut Vec<ContentBoundRequirementIssueV1>,
) -> Result<Option<ContentBoundCoverageRowV1>, Vec<ContentBoundRequirementIssueV1>> {
    debug_assert!(issues.capacity() >= issues.len());
    debug_assert!(issues.len() <= MAX_VALIDATION_ISSUE_COUNT);
    let Some(reference) = indexes.references.get(draft.requirement_reference_blake3.as_str()) else {
        push_issue(
            issues,
            ContentBoundRequirementIssueCodeV1::RequirementMissing,
            &format!("input.coverage[{row_index}].requirement_reference_blake3"),
            "coverage row names an absent Valence requirement reference",
        );
        return Ok(None);
    };
    let Some(registry) = indexes.registries.get(reference.repository_id.as_str()) else {
        push_issue(
            issues,
            ContentBoundRequirementIssueCodeV1::RegistryMissing,
            &format!("input.coverage[{row_index}]"),
            "coverage row requirement repository has no supplied registry",
        );
        return Ok(None);
    };
    let evidence_identities = collect_coverage_evidence_identities(draft, row_index, reference, indexes, issues);
    if evidence_identities.is_empty() {
        push_issue(
            issues,
            ContentBoundRequirementIssueCodeV1::CoverageEvidenceMissing,
            &format!("input.coverage[{row_index}].evidence_repository_paths"),
            "coverage row requires at least one evidence row",
        );
        return Ok(None);
    }
    let mut row = ContentBoundCoverageRowV1 {
        schema: CONTENT_BOUND_COVERAGE_ROW_SCHEMA_V1.to_string(),
        repository_id: reference.repository_id.clone(),
        requirement_id: reference.requirement_id.clone(),
        requirement_reference_identity: reference.reference_identity.clone(),
        registry_identity: RequirementDigestV1 {
            domain: RequirementDigestDomainV1::RequirementRegistry,
            blake3: registry.registry_blake3.clone(),
        },
        evidence_role: draft.evidence_role,
        evidence_row_identities: evidence_identities,
        coverage_identity: empty_content_digest(ContentBoundDigestDomainV1::RequirementCoverage),
    };
    row.coverage_identity.blake3 = coverage_row_hash(&row)?;
    Ok(Some(row))
}

fn collect_coverage_evidence_identities(
    draft: &ContentBoundCoverageDraftV1,
    row_index: usize,
    reference: &RequirementRefV1,
    indexes: &CoverageBuildIndexes<'_>,
    issues: &mut Vec<ContentBoundRequirementIssueV1>,
) -> Vec<ContentBoundDigestV1> {
    debug_assert!(issues.capacity() >= issues.len());
    debug_assert!(issues.len() <= MAX_VALIDATION_ISSUE_COUNT);
    let mut identities = Vec::with_capacity(draft.evidence_repository_paths.len());
    let mut path_seen = BTreeSet::new();
    for (path_index, path) in draft.evidence_repository_paths.iter().enumerate() {
        if !path_seen.insert(path.as_str()) {
            push_issue(
                issues,
                ContentBoundRequirementIssueCodeV1::CoverageEvidenceDuplicate,
                &format!("input.coverage[{row_index}].evidence_repository_paths[{path_index}]"),
                "coverage row names one evidence path more than once",
            );
            continue;
        }
        match indexes.evidence_by_path.get(&(reference.repository_id.as_str(), path.as_str())) {
            Some(row) if row.role == draft.evidence_role => identities.push(row.row_identity.clone()),
            Some(_) => push_issue(
                issues,
                ContentBoundRequirementIssueCodeV1::CoverageRoleMismatch,
                &format!("input.coverage[{row_index}].evidence_role"),
                "coverage evidence role does not match its manifest row",
            ),
            None => push_issue(
                issues,
                ContentBoundRequirementIssueCodeV1::CoverageEvidenceMissing,
                &format!("input.coverage[{row_index}].evidence_repository_paths[{path_index}]"),
                "coverage row names an absent measured evidence path",
            ),
        }
    }
    identities.sort();
    identities
}

struct CoverageValidationIndexes<'a> {
    references: BTreeMap<&'a str, &'a RequirementRefV1>,
    registries: BTreeMap<&'a str, &'a CairnContentBoundRequirementRegistryV1>,
    evidence: BTreeMap<&'a str, &'a ContentBoundMeasuredEvidenceRowV1>,
}

struct CoverageRowValidationState<'a, 'b> {
    indexes: &'a CoverageValidationIndexes<'b>,
    coverage_seen: &'a mut BTreeSet<(String, ContentBoundEvidenceRoleV1)>,
    required_roles: &'a mut BTreeSet<(String, ContentBoundEvidenceRoleV1)>,
    issues: &'a mut Vec<ContentBoundRequirementIssueV1>,
}

fn validate_coverage_rows(
    rows: &[ContentBoundCoverageRowV1],
    references: &[RequirementRefV1],
    registries: &[CairnContentBoundRequirementRegistryV1],
    evidence_rows: &[ContentBoundMeasuredEvidenceRowV1],
    issues: &mut Vec<ContentBoundRequirementIssueV1>,
) {
    debug_assert!(issues.capacity() >= issues.len());
    debug_assert!(issues.len() <= MAX_VALIDATION_ISSUE_COUNT);
    validate_collection_count(rows.len(), MAX_COVERAGE_ROW_COUNT, "coverage", issues);
    let indexes = CoverageValidationIndexes {
        references: references
            .iter()
            .map(|reference| (reference.reference_identity.blake3.as_str(), reference))
            .collect(),
        registries: registries.iter().map(|registry| (registry.repository_id.as_str(), registry)).collect(),
        evidence: evidence_rows.iter().map(|row| (row.row_identity.blake3.as_str(), row)).collect(),
    };
    let mut coverage_seen = BTreeSet::new();
    let mut required_roles = BTreeSet::new();
    for (row_index, row) in rows.iter().enumerate() {
        let mut state = CoverageRowValidationState {
            indexes: &indexes,
            coverage_seen: &mut coverage_seen,
            required_roles: &mut required_roles,
            issues,
        };
        validate_coverage_row(row, row_index, &mut state);
    }
    validate_required_coverage_roles(references, &required_roles, issues);
}

fn validate_coverage_row(
    row: &ContentBoundCoverageRowV1,
    row_index: usize,
    state: &mut CoverageRowValidationState<'_, '_>,
) {
    debug_assert!(state.issues.capacity() >= state.issues.len());
    debug_assert!(state.issues.len() <= MAX_VALIDATION_ISSUE_COUNT);
    let prefix = format!("coverage[{row_index}]");
    if row.schema != CONTENT_BOUND_COVERAGE_ROW_SCHEMA_V1 {
        push_issue(
            state.issues,
            ContentBoundRequirementIssueCodeV1::InputSchemaUnsupported,
            &format!("{prefix}.schema"),
            "content-bound coverage row schema is unsupported",
        );
    }
    let key = (row.requirement_reference_identity.blake3.clone(), row.evidence_role);
    if !state.coverage_seen.insert(key.clone()) {
        push_issue(
            state.issues,
            ContentBoundRequirementIssueCodeV1::CoverageDuplicate,
            &prefix,
            "requirement and evidence-role coverage occurs more than once",
        );
    }
    state.required_roles.insert(key);
    validate_coverage_row_links(row, &prefix, state.indexes, state.issues);
    validate_coverage_evidence_links(row, &prefix, &state.indexes.evidence, state.issues);
    if coverage_row_hash(row).is_ok_and(|expected| expected != row.coverage_identity.blake3) {
        push_issue(
            state.issues,
            ContentBoundRequirementIssueCodeV1::RowStale,
            &format!("{prefix}.coverage_identity.blake3"),
            "coverage row identity does not match canonical fields",
        );
    }
}

fn validate_coverage_row_links(
    row: &ContentBoundCoverageRowV1,
    prefix: &str,
    indexes: &CoverageValidationIndexes<'_>,
    issues: &mut Vec<ContentBoundRequirementIssueV1>,
) {
    debug_assert!(issues.capacity() >= issues.len());
    debug_assert!(issues.len() <= MAX_VALIDATION_ISSUE_COUNT);
    validate_requirement_digest(
        &row.requirement_reference_identity,
        RequirementDigestDomainV1::RequirementReference,
        &format!("{prefix}.requirement_reference_identity"),
        issues,
    );
    validate_requirement_digest(
        &row.registry_identity,
        RequirementDigestDomainV1::RequirementRegistry,
        &format!("{prefix}.registry_identity"),
        issues,
    );
    validate_content_digest(
        &row.coverage_identity,
        ContentBoundDigestDomainV1::RequirementCoverage,
        &format!("{prefix}.coverage_identity"),
        issues,
    );
    match indexes.references.get(row.requirement_reference_identity.blake3.as_str()) {
        Some(reference)
            if reference.repository_id == row.repository_id && reference.requirement_id == row.requirement_id => {}
        Some(_) => push_issue(
            issues,
            ContentBoundRequirementIssueCodeV1::RepositoryMismatch,
            prefix,
            "coverage row repository or requirement does not match its Valence reference",
        ),
        None => push_issue(
            issues,
            ContentBoundRequirementIssueCodeV1::RequirementMissing,
            &format!("{prefix}.requirement_reference_identity.blake3"),
            "coverage row names an absent Valence reference",
        ),
    }
    match indexes.registries.get(row.repository_id.as_str()) {
        Some(registry) if registry.registry_blake3 == row.registry_identity.blake3 => {}
        Some(_) => push_issue(
            issues,
            ContentBoundRequirementIssueCodeV1::StaleRegistry,
            &format!("{prefix}.registry_identity.blake3"),
            "coverage row registry identity is stale",
        ),
        None => push_issue(
            issues,
            ContentBoundRequirementIssueCodeV1::RegistryMissing,
            &format!("{prefix}.repository_id"),
            "coverage row repository has no registry",
        ),
    }
}

fn validate_coverage_evidence_links(
    row: &ContentBoundCoverageRowV1,
    prefix: &str,
    evidence_index: &BTreeMap<&str, &ContentBoundMeasuredEvidenceRowV1>,
    issues: &mut Vec<ContentBoundRequirementIssueV1>,
) {
    debug_assert!(issues.capacity() >= issues.len());
    debug_assert!(issues.len() <= MAX_VALIDATION_ISSUE_COUNT);
    let mut evidence_seen = BTreeSet::new();
    if row.evidence_row_identities.is_empty() {
        push_issue(
            issues,
            ContentBoundRequirementIssueCodeV1::CoverageEvidenceMissing,
            &format!("{prefix}.evidence_row_identities"),
            "coverage row has no evidence rows",
        );
    }
    for (evidence_row_index, identity) in row.evidence_row_identities.iter().enumerate() {
        let field_path = format!("{prefix}.evidence_row_identities[{evidence_row_index}]");
        validate_content_digest(identity, ContentBoundDigestDomainV1::EvidenceRow, &field_path, issues);
        if !evidence_seen.insert(identity.blake3.as_str()) {
            push_issue(
                issues,
                ContentBoundRequirementIssueCodeV1::CoverageEvidenceDuplicate,
                &field_path,
                "coverage row names an evidence row more than once",
            );
        }
        match evidence_index.get(identity.blake3.as_str()) {
            Some(evidence) if evidence.repository_id == row.repository_id && evidence.role == row.evidence_role => {}
            Some(_) => push_issue(
                issues,
                ContentBoundRequirementIssueCodeV1::CoverageRoleMismatch,
                &field_path,
                "coverage evidence repository or role does not match its row",
            ),
            None => push_issue(
                issues,
                ContentBoundRequirementIssueCodeV1::CoverageEvidenceMissing,
                &field_path,
                "coverage row names an absent evidence row",
            ),
        }
    }
}

fn validate_required_coverage_roles(
    references: &[RequirementRefV1],
    required_roles: &BTreeSet<(String, ContentBoundEvidenceRoleV1)>,
    issues: &mut Vec<ContentBoundRequirementIssueV1>,
) {
    for reference in references {
        for required_role in [ContentBoundEvidenceRoleV1::Source, ContentBoundEvidenceRoleV1::Test] {
            if !required_roles.contains(&(reference.reference_identity.blake3.clone(), required_role)) {
                push_issue(
                    issues,
                    ContentBoundRequirementIssueCodeV1::CoverageMissing,
                    "coverage",
                    "strict requirement reference requires source and test coverage rows",
                );
            }
        }
    }
}

fn validate_external_links(
    rows: &[ContentBoundMeasuredEvidenceRowV1],
    external: &[ContentBoundExternalEvidenceLinkV1],
    issues: &mut Vec<ContentBoundRequirementIssueV1>,
) {
    debug_assert!(rows.get(rows.len()).is_none());
    debug_assert!(external.get(external.len()).is_none());
    let linked_content_bound_count =
        external.iter().filter(|link| link.schema == CONTENT_BOUND_EVIDENCE_EXTERNAL_SCHEMA_V1).count();
    if linked_content_bound_count != rows.len() {
        push_issue(
            issues,
            ContentBoundRequirementIssueCodeV1::ExternalEvidenceLinkMismatch,
            "external_evidence",
            "release external evidence has missing or extra content-bound artifacts",
        );
    }
    for (index, row) in rows.iter().enumerate() {
        let matches = external
            .iter()
            .filter(|link| link.bundle_relative_path == row.bundle_relative_path)
            .collect::<Vec<_>>();
        let is_valid = matches.len() == 1
            && matches[0].role == row.role.external_role()
            && matches[0].schema == CONTENT_BOUND_EVIDENCE_EXTERNAL_SCHEMA_V1
            && matches[0].digest_blake3 == row.content_identity.blake3;
        if !is_valid {
            push_issue(
                issues,
                ContentBoundRequirementIssueCodeV1::ExternalEvidenceLinkMismatch,
                &format!("evidence_manifest.rows[{index}].bundle_relative_path"),
                "content-bound evidence row does not match exactly one release external-evidence artifact",
            );
        }
    }
}

fn validate_release_links(
    evidence: &ContentBoundReleaseEvidenceV1,
    expected_release_id: &str,
    expected_binary_digests_blake3: &[String],
    issues: &mut Vec<ContentBoundRequirementIssueV1>,
) {
    debug_assert!(issues.capacity() >= issues.len());
    debug_assert!(issues.len() <= MAX_VALIDATION_ISSUE_COUNT);
    if evidence.release_id != expected_release_id {
        push_issue(
            issues,
            ContentBoundRequirementIssueCodeV1::ReleaseIdMismatch,
            "release_id",
            "content-bound evidence release ID does not match the release manifest",
        );
    }
    let mut expected = expected_binary_digests_blake3.to_vec();
    expected.sort();
    let mut actual = evidence.binary_identities.iter().map(|identity| identity.blake3.clone()).collect::<Vec<_>>();
    actual.sort();
    for (index, identity) in evidence.binary_identities.iter().enumerate() {
        validate_content_digest(
            identity,
            ContentBoundDigestDomainV1::Binary,
            &format!("binary_identities[{index}]"),
            issues,
        );
    }
    if expected != actual {
        push_issue(
            issues,
            ContentBoundRequirementIssueCodeV1::ReleaseBinaryLinkMismatch,
            "binary_identities",
            "content-bound evidence binary identities do not match the release manifest",
        );
    }
    let expected_release_identity = domain_hash_json(
        RELEASE_LINK_DOMAIN,
        &ReleaseLinkHashMaterial {
            release_id: &evidence.release_id,
            binary_identities: &evidence.binary_identities,
        },
        "release_identity.blake3",
    );
    validate_content_digest(
        &evidence.release_identity,
        ContentBoundDigestDomainV1::Release,
        "release_identity",
        issues,
    );
    if expected_release_identity.is_ok_and(|identity| identity != evidence.release_identity.blake3) {
        push_issue(
            issues,
            ContentBoundRequirementIssueCodeV1::ReleaseBinaryLinkMismatch,
            "release_identity.blake3",
            "content-bound release link identity is stale",
        );
    }
}

fn validate_release_evidence_shape(
    evidence: &ContentBoundReleaseEvidenceV1,
    issues: &mut Vec<ContentBoundRequirementIssueV1>,
) {
    debug_assert!(issues.capacity() >= issues.len());
    debug_assert!(issues.len() <= MAX_VALIDATION_ISSUE_COUNT);
    if evidence.schema != CONTENT_BOUND_RELEASE_EVIDENCE_SCHEMA_V1 {
        push_issue(
            issues,
            ContentBoundRequirementIssueCodeV1::InputSchemaUnsupported,
            "schema",
            "content-bound release evidence schema is unsupported",
        );
    }
    if !producer_contracts_match(&evidence.producer_contracts) {
        push_issue(
            issues,
            ContentBoundRequirementIssueCodeV1::ContractRevisionMismatch,
            "producer_contracts",
            "content-bound producer contracts do not match reviewed Cairn and Valence revisions",
        );
    }
    validate_required_non_claims(&evidence.non_claims, CONTENT_BOUND_RELEASE_NON_CLAIMS, "non_claims", issues);
    validate_content_digest(
        &evidence.evidence_identity,
        ContentBoundDigestDomainV1::ReleaseEvidence,
        "evidence_identity",
        issues,
    );
}

fn producer_contracts_match(contracts: &ContentBoundProducerContractsV1) -> bool {
    if contracts.cairn_revision != CAIRN_CONTENT_BOUND_REGISTRY_CONTRACT_REVISION {
        return false;
    }
    if contracts.cairn_registry_schema != CAIRN_CONTENT_BOUND_REQUIREMENT_REGISTRY_SCHEMA_V1 {
        return false;
    }
    if contracts.valence_revision != VALENCE_CONTENT_BOUND_REQUIREMENT_CONTRACT_REVISION {
        return false;
    }
    contracts.valence_requirement_ref_schema == VALENCE_REQUIREMENT_REF_SCHEMA_V1
}

fn evidence_paths_for_coverage(
    coverage: &ContentBoundCoverageRowV1,
    rows: &[ContentBoundMeasuredEvidenceRowV1],
) -> Vec<String> {
    let identities = coverage
        .evidence_row_identities
        .iter()
        .map(|identity| identity.blake3.as_str())
        .collect::<BTreeSet<_>>();
    rows.iter()
        .filter(|row| identities.contains(row.row_identity.blake3.as_str()))
        .map(|row| row.repository_relative_path.clone())
        .collect()
}

fn evidence_row_hash(row: &ContentBoundMeasuredEvidenceRowV1) -> Result<String, Vec<ContentBoundRequirementIssueV1>> {
    domain_hash_json(
        EVIDENCE_ROW_DOMAIN,
        &EvidenceRowHashMaterial {
            schema: &row.schema,
            repository_id: &row.repository_id,
            repository_relative_path: &row.repository_relative_path,
            bundle_relative_path: &row.bundle_relative_path,
            role: row.role,
            size_bytes: row.size_bytes,
            content_identity: &row.content_identity,
            span: &row.span,
            symbol: &row.symbol,
            producer_receipt_row_identity: &row.producer_receipt_row_identity,
            non_claims: &row.non_claims,
        },
        "row_identity.blake3",
    )
}

pub fn seal_content_bound_evidence_row(
    mut row: ContentBoundMeasuredEvidenceRowV1,
) -> Result<ContentBoundMeasuredEvidenceRowV1, Vec<ContentBoundRequirementIssueV1>> {
    row.row_identity = empty_content_digest(ContentBoundDigestDomainV1::EvidenceRow);
    row.row_identity.blake3 = evidence_row_hash(&row)?;
    let mut issues = Vec::new();
    validate_evidence_row(&row, "evidence_row", &mut issues);
    sort_issues(&mut issues);
    if issues.is_empty() { Ok(row) } else { Err(issues) }
}

fn coverage_row_hash(row: &ContentBoundCoverageRowV1) -> Result<String, Vec<ContentBoundRequirementIssueV1>> {
    domain_hash_json(
        COVERAGE_ROW_DOMAIN,
        &CoverageRowHashMaterial {
            schema: &row.schema,
            repository_id: &row.repository_id,
            requirement_id: &row.requirement_id,
            requirement_reference_identity: &row.requirement_reference_identity,
            registry_identity: &row.registry_identity,
            evidence_role: row.evidence_role,
            evidence_row_identities: &row.evidence_row_identities,
        },
        "coverage_identity.blake3",
    )
}

fn release_evidence_hash(
    evidence: &ContentBoundReleaseEvidenceV1,
) -> Result<String, Vec<ContentBoundRequirementIssueV1>> {
    domain_hash_json(
        RELEASE_EVIDENCE_DOMAIN,
        &ReleaseEvidenceHashMaterial {
            schema: &evidence.schema,
            producer_contracts: &evidence.producer_contracts,
            registries: &evidence.registries,
            requirement_refs: &evidence.requirement_refs,
            evidence_manifest: &evidence.evidence_manifest,
            coverage: &evidence.coverage,
            release_id: &evidence.release_id,
            release_identity: &evidence.release_identity,
            binary_identities: &evidence.binary_identities,
            non_claims: &evidence.non_claims,
        },
        "evidence_identity.blake3",
    )
}

fn requirement_ref_hash(reference: &RequirementRefV1) -> Result<String, Vec<ContentBoundRequirementIssueV1>> {
    domain_hash_json(
        VALENCE_REQUIREMENT_REF_DOMAIN,
        &RequirementRefHashMaterial {
            schema: &reference.schema,
            repository_id: &reference.repository_id,
            revision: &reference.revision,
            requirement_id: &reference.requirement_id,
            specification_path: &reference.specification_path,
            specification_identity: &reference.specification_identity,
            requirement_body_identity: &reference.requirement_body_identity,
            policy_identity: &reference.policy_identity,
            row_identity: &reference.row_identity,
            registry_identity: &reference.registry_identity,
            registry_receipt_identity: &reference.registry_receipt_identity,
            verification_role: &reference.verification_role,
            non_claims: &reference.non_claims,
        },
        "reference_identity.blake3",
    )
}

fn requirement_ref_order(left: &RequirementRefV1, right: &RequirementRefV1) -> Ordering {
    left.repository_id
        .cmp(&right.repository_id)
        .then(left.requirement_id.cmp(&right.requirement_id))
        .then(left.reference_identity.blake3.cmp(&right.reference_identity.blake3))
}

fn evidence_row_order(left: &ContentBoundMeasuredEvidenceRowV1, right: &ContentBoundMeasuredEvidenceRowV1) -> Ordering {
    left.repository_id
        .cmp(&right.repository_id)
        .then(left.repository_relative_path.cmp(&right.repository_relative_path))
        .then(left.role.cmp(&right.role))
        .then(left.row_identity.blake3.cmp(&right.row_identity.blake3))
}

fn coverage_row_order(left: &ContentBoundCoverageRowV1, right: &ContentBoundCoverageRowV1) -> Ordering {
    left.repository_id
        .cmp(&right.repository_id)
        .then(left.requirement_id.cmp(&right.requirement_id))
        .then(left.evidence_role.cmp(&right.evidence_role))
        .then(left.coverage_identity.blake3.cmp(&right.coverage_identity.blake3))
}

fn validate_requirement_digest(
    digest: &RequirementDigestV1,
    expected: RequirementDigestDomainV1,
    field_path: &str,
    issues: &mut Vec<ContentBoundRequirementIssueV1>,
) {
    if digest.domain != expected {
        push_issue(
            issues,
            ContentBoundRequirementIssueCodeV1::DigestDomainSubstitution,
            &format!("{field_path}.domain"),
            "Valence digest role does not match the field semantic domain",
        );
    }
    if crate::CheckedRequirementDigest::new(digest.domain.clone(), &digest.blake3).is_err() {
        push_issue(
            issues,
            ContentBoundRequirementIssueCodeV1::Blake3Invalid,
            &format!("{field_path}.blake3"),
            "BLAKE3 identity must use 64 lowercase hexadecimal characters",
        );
    }
}

fn validate_content_digest(
    digest: &ContentBoundDigestV1,
    expected: ContentBoundDigestDomainV1,
    field_path: &str,
    issues: &mut Vec<ContentBoundRequirementIssueV1>,
) {
    if digest.domain != expected {
        push_issue(
            issues,
            ContentBoundRequirementIssueCodeV1::DigestDomainSubstitution,
            &format!("{field_path}.domain"),
            "Mantle digest role does not match the field semantic domain",
        );
    }
    if crate::CheckedContentBoundDigest::new(digest.domain, &digest.blake3).is_err() {
        push_issue(
            issues,
            ContentBoundRequirementIssueCodeV1::Blake3Invalid,
            &format!("{field_path}.blake3"),
            "BLAKE3 identity must use 64 lowercase hexadecimal characters",
        );
    }
}

// Internal validator: semantic parameter names keep all local call sites explicit.
#[allow(tigerstyle::ambiguous_params)]
fn validate_repository_id(repository_id: &str, field_path: &str, issues: &mut Vec<ContentBoundRequirementIssueV1>) {
    if crate::ContentBoundRepositoryId::new(repository_id).is_err() {
        push_issue(
            issues,
            ContentBoundRequirementIssueCodeV1::RepositoryIdInvalid,
            field_path,
            "repository identity is not a bounded portable identifier",
        );
    }
}

fn validate_cairn_revision(
    revision: &CairnContentBoundRevisionV1,
    field_path: &str,
    issues: &mut Vec<ContentBoundRequirementIssueV1>,
) {
    match revision.kind.as_str() {
        "git" => validate_git_revision(&revision.value, field_path, issues),
        "jujutsu" => validate_jujutsu_revision(&revision.value, field_path, issues),
        "opaque" => validate_opaque_revision(&revision.value, field_path, issues),
        _ => push_issue(
            issues,
            ContentBoundRequirementIssueCodeV1::RevisionKindUnsupported,
            &format!("{field_path}.kind"),
            "revision kind must be git, jujutsu, or opaque",
        ),
    }
}

fn validate_stack_revision(
    revision: &StackRevisionV1,
    field_path: &str,
    issues: &mut Vec<ContentBoundRequirementIssueV1>,
) {
    match revision.kind {
        StackRevisionKindV1::Git => validate_git_revision(&revision.value, field_path, issues),
        StackRevisionKindV1::Jujutsu => validate_jujutsu_revision(&revision.value, field_path, issues),
        StackRevisionKindV1::Opaque => validate_opaque_revision(&revision.value, field_path, issues),
    }
}

#[allow(tigerstyle::ambiguous_params)]
fn validate_git_revision(revision: &str, field_path: &str, issues: &mut Vec<ContentBoundRequirementIssueV1>) {
    let has_valid_length_bytes = matches!(revision.len(), GIT_SHA1_HEX_LENGTH | GIT_SHA256_HEX_LENGTH);
    if !has_valid_length_bytes || !is_lower_hex(revision) {
        push_issue(
            issues,
            ContentBoundRequirementIssueCodeV1::RevisionInvalid,
            &format!("{field_path}.value"),
            "Git revision must be a complete lowercase hexadecimal identity",
        );
    }
}

#[allow(tigerstyle::ambiguous_params)]
fn validate_jujutsu_revision(revision: &str, field_path: &str, issues: &mut Vec<ContentBoundRequirementIssueV1>) {
    let has_valid_length_bytes =
        (JUJUTSU_REVISION_MIN_HEX_LENGTH..=JUJUTSU_REVISION_MAX_HEX_LENGTH).contains(&revision.len());
    if !has_valid_length_bytes || !is_lower_hex(revision) {
        push_issue(
            issues,
            ContentBoundRequirementIssueCodeV1::RevisionInvalid,
            &format!("{field_path}.value"),
            "Jujutsu revision must be a bounded lowercase hexadecimal identity",
        );
    }
}

#[allow(tigerstyle::ambiguous_params)]
fn validate_opaque_revision(revision: &str, field_path: &str, issues: &mut Vec<ContentBoundRequirementIssueV1>) {
    let is_valid = !revision.is_empty()
        && revision.len() <= MAX_OPAQUE_REVISION_BYTES
        && revision.trim() == revision
        && !revision.chars().any(char::is_control);
    if !is_valid {
        push_issue(
            issues,
            ContentBoundRequirementIssueCodeV1::RevisionInvalid,
            &format!("{field_path}.value"),
            "opaque revision must be bounded, trimmed, and free of control characters",
        );
    }
}

#[allow(tigerstyle::ambiguous_params)]
fn validate_requirement_id(requirement_id: &str, field_path: &str, issues: &mut Vec<ContentBoundRequirementIssueV1>) {
    if crate::ContentBoundRequirementId::new(requirement_id).is_err() {
        push_issue(
            issues,
            ContentBoundRequirementIssueCodeV1::RequirementIdInvalid,
            field_path,
            "requirement ID is not a bounded portable identifier",
        );
    }
}

#[allow(tigerstyle::ambiguous_params)]
fn validate_specification_path(path: &str, field_path: &str, issues: &mut Vec<ContentBoundRequirementIssueV1>) {
    if crate::ContentBoundSpecificationPath::new(path).is_err() {
        push_issue(
            issues,
            ContentBoundRequirementIssueCodeV1::SpecificationPathUnsafe,
            field_path,
            "specification path is not a safe accepted-spec path",
        );
    }
}

#[allow(tigerstyle::ambiguous_params)]
fn validate_safe_relative_path(
    path: &str,
    field_path: &str,
    require_bundle_prefix: bool,
    issues: &mut Vec<ContentBoundRequirementIssueV1>,
) {
    let admitted = if require_bundle_prefix {
        crate::ContentBoundEvidencePath::new_bundle_path(path)
    } else {
        crate::ContentBoundEvidencePath::new_repository_path(path)
    };
    if admitted.is_err() {
        push_issue(
            issues,
            ContentBoundRequirementIssueCodeV1::SpecificationPathUnsafe,
            field_path,
            "evidence path is not a safe normalized relative path",
        );
    }
}

fn validate_required_non_claims(
    non_claims: &[String],
    required: &[&str],
    field_path: &str,
    issues: &mut Vec<ContentBoundRequirementIssueV1>,
) {
    for required_claim in required {
        if !non_claims.iter().any(|claim| claim == required_claim) {
            push_issue(
                issues,
                ContentBoundRequirementIssueCodeV1::NonClaimMissing,
                field_path,
                "required non-claim boundary is missing",
            );
        }
    }
}

#[allow(tigerstyle::ambiguous_params)]
fn validate_blake3(value: &str, field_path: &str, issues: &mut Vec<ContentBoundRequirementIssueV1>) {
    if crate::CheckedBlake3Hex::new(value).is_err() {
        push_issue(
            issues,
            ContentBoundRequirementIssueCodeV1::Blake3Invalid,
            field_path,
            "BLAKE3 identity must use 64 lowercase hexadecimal characters",
        );
    }
}

#[allow(tigerstyle::ambiguous_params)]
fn validate_collection_count(
    observed: usize,
    maximum: usize,
    field_path: &str,
    issues: &mut Vec<ContentBoundRequirementIssueV1>,
) {
    if observed > maximum {
        push_issue(
            issues,
            ContentBoundRequirementIssueCodeV1::ReleaseInputEmpty,
            field_path,
            "content-bound collection exceeds its fixed profile limit",
        );
    }
}

fn validate_binary_digest_values(
    digests: &[String],
    field_path: &str,
    issues: &mut Vec<ContentBoundRequirementIssueV1>,
) {
    debug_assert!(issues.capacity() >= issues.len());
    debug_assert!(issues.len() <= MAX_VALIDATION_ISSUE_COUNT);
    if digests.is_empty() {
        push_issue(
            issues,
            ContentBoundRequirementIssueCodeV1::ReleaseBinaryLinkMismatch,
            field_path,
            "content-bound release requires at least one binary identity",
        );
    }
    let mut seen = BTreeSet::new();
    for (index, digest) in digests.iter().enumerate() {
        validate_blake3(digest, &format!("{field_path}[{index}]"), issues);
        if !seen.insert(digest.as_str()) {
            push_issue(
                issues,
                ContentBoundRequirementIssueCodeV1::ReleaseBinaryLinkMismatch,
                &format!("{field_path}[{index}]"),
                "release binary identity occurs more than once",
            );
        }
    }
}

#[allow(tigerstyle::ambiguous_params)]
fn validate_release_id(release_id: &str, field_path: &str, issues: &mut Vec<ContentBoundRequirementIssueV1>) {
    if crate::ContentBoundReleaseId::new(release_id).is_err() {
        push_issue(
            issues,
            ContentBoundRequirementIssueCodeV1::ReleaseIdMismatch,
            field_path,
            "release ID must be bounded, trimmed, and free of control characters",
        );
    }
}

#[allow(tigerstyle::ambiguous_params)]
fn compare_digest(
    actual: &str,
    expected: &str,
    code: ContentBoundRequirementIssueCodeV1,
    field_path: &str,
    issues: &mut Vec<ContentBoundRequirementIssueV1>,
) {
    if actual != expected {
        push_issue(issues, code, field_path, "typed identity does not match its supplied producer artifact");
    }
}

fn is_lower_hex(value: &str) -> bool {
    !value.is_empty() && value.bytes().all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'))
}

fn empty_content_digest(domain: ContentBoundDigestDomainV1) -> ContentBoundDigestV1 {
    ContentBoundDigestV1 {
        domain,
        blake3: String::new(),
    }
}

fn required_non_claims(required: &[&str]) -> Vec<String> {
    required.iter().map(|claim| (*claim).to_string()).collect()
}

fn domain_hash_json<T: Serialize>(
    domain: &str,
    value: &T,
    field_path: &str,
) -> Result<String, Vec<ContentBoundRequirementIssueV1>> {
    debug_assert!(!domain.is_empty());
    debug_assert!(!field_path.is_empty());
    let value = serde_json::to_value(value).map_err(|_error| {
        vec![issue(
            ContentBoundRequirementIssueCodeV1::CanonicalSerializationFailed,
            field_path,
            "canonical hash material could not be serialized",
        )]
    })?;
    let bytes = serde_json::to_vec(&value).map_err(|_error| {
        vec![issue(
            ContentBoundRequirementIssueCodeV1::CanonicalSerializationFailed,
            field_path,
            "canonical hash material could not be encoded",
        )]
    })?;
    let mut hasher = blake3::Hasher::new();
    hasher.update(domain.as_bytes());
    hasher.update(&[DOMAIN_SEPARATOR]);
    hasher.update(&bytes);
    Ok(hasher.finalize().to_hex().to_string())
}

#[allow(tigerstyle::ambiguous_params)]
fn issue(code: ContentBoundRequirementIssueCodeV1, field_path: &str, message: &str) -> ContentBoundRequirementIssueV1 {
    ContentBoundRequirementIssueV1 {
        code,
        field_path: field_path.to_string(),
        message: message.to_string(),
    }
}

#[allow(tigerstyle::ambiguous_params)]
fn push_issue(
    issues: &mut Vec<ContentBoundRequirementIssueV1>,
    code: ContentBoundRequirementIssueCodeV1,
    field_path: &str,
    message: &str,
) {
    if issues.len() < MAX_VALIDATION_ISSUE_COUNT_BEFORE_SENTINEL {
        issues.push(issue(code, field_path, message));
        return;
    }
    if issues.len() == MAX_VALIDATION_ISSUE_COUNT_BEFORE_SENTINEL {
        issues.push(issue(
            ContentBoundRequirementIssueCodeV1::IssueLimitReached,
            "validation",
            "additional deterministic validation issues were omitted",
        ));
    }
}

fn sort_issues(issues: &mut Vec<ContentBoundRequirementIssueV1>) {
    issues.sort_by(|left, right| {
        left.field_path
            .cmp(&right.field_path)
            .then(left.code.cmp(&right.code))
            .then(left.message.cmp(&right.message))
    });
    issues.dedup();
}

#[cfg(test)]
mod tests {
    use super::*;

    const REPOSITORY_ID: &str = "OnixResearch/mantle";
    const REQUIREMENT_ID: &str = "mantle.release_provenance.content_bound_requirement_coverage";
    const SPECIFICATION_PATH: &str = "cairn/specs/release-provenance/spec.md";

    fn hash(domain: &str, bytes: &[u8]) -> String {
        let mut hasher = blake3::Hasher::new();
        hasher.update(domain.as_bytes());
        hasher.update(&[DOMAIN_SEPARATOR]);
        hasher.update(bytes);
        hasher.finalize().to_hex().to_string()
    }

    fn registry() -> CairnContentBoundRequirementRegistryV1 {
        let revision = CairnContentBoundRevisionV1 {
            kind: "opaque".to_string(),
            value: "mantle-fixture-v1".to_string(),
        };
        let specification_blake3 = hash("fixture.specification.v1", b"content-bound release specification");
        let source_span = CairnContentBoundSourceSpanV1 {
            start_byte: 1,
            end_byte: 64,
            start_line: 1,
            end_line: 2,
        };
        let requirement_body_blake3 = hash("fixture.requirement.v1", b"requirement body");
        let mut row = CairnContentBoundRequirementRowV1 {
            requirement_id: REQUIREMENT_ID.to_string(),
            specification_path: SPECIFICATION_PATH.to_string(),
            specification_blake3: specification_blake3.clone(),
            source_span,
            requirement_body_blake3,
            row_blake3: String::new(),
        };
        row.row_blake3 = domain_hash_json(
            CAIRN_REQUIREMENT_ROW_DOMAIN,
            &CairnRequirementRowHashMaterial {
                requirement_id: &row.requirement_id,
                specification_path: &row.specification_path,
                specification_blake3: &row.specification_blake3,
                source_span: &row.source_span,
                requirement_body_blake3: &row.requirement_body_blake3,
            },
            "row",
        )
        .expect("row hash");
        let specifications = vec![CairnContentBoundSpecificationIdentityV1 {
            path: SPECIFICATION_PATH.to_string(),
            content_blake3: specification_blake3,
        }];
        let specification_set_blake3 =
            domain_hash_json(CAIRN_SPECIFICATION_SET_DOMAIN, &specifications, "specifications")
                .expect("specification set hash");
        let policy_blake3 = hash("fixture.policy.v1", b"policy");
        let non_claims = required_non_claims(CAIRN_REGISTRY_NON_CLAIMS);
        let mut registry = CairnContentBoundRequirementRegistryV1 {
            schema_version: CAIRN_CONTENT_BOUND_REQUIREMENT_REGISTRY_SCHEMA_V1.to_string(),
            boundary: CAIRN_CONTENT_BOUND_REQUIREMENT_REGISTRY_BOUNDARY.to_string(),
            repository_id: REPOSITORY_ID.to_string(),
            revision,
            specification_set_blake3,
            specifications,
            policy_blake3,
            rows: vec![row],
            non_claims,
            registry_blake3: String::new(),
            receipt: CairnContentBoundRequirementRegistryReceiptV1 {
                schema_version: CAIRN_CONTENT_BOUND_REQUIREMENT_REGISTRY_RECEIPT_SCHEMA_V1.to_string(),
                registry_blake3: String::new(),
                repository_id: String::new(),
                revision_kind: String::new(),
                revision: String::new(),
                specification_set_blake3: String::new(),
                policy_blake3: String::new(),
                row_count: 0,
                non_claims: Vec::new(),
                receipt_blake3: String::new(),
            },
        };
        registry.registry_blake3 = domain_hash_json(
            CAIRN_REGISTRY_DOMAIN,
            &CairnRegistryHashMaterial {
                schema_version: &registry.schema_version,
                boundary: &registry.boundary,
                repository_id: &registry.repository_id,
                revision: &registry.revision,
                specification_set_blake3: &registry.specification_set_blake3,
                specifications: &registry.specifications,
                policy_blake3: &registry.policy_blake3,
                rows: &registry.rows,
                non_claims: &registry.non_claims,
            },
            "registry",
        )
        .expect("registry hash");
        registry.receipt = CairnContentBoundRequirementRegistryReceiptV1 {
            schema_version: CAIRN_CONTENT_BOUND_REQUIREMENT_REGISTRY_RECEIPT_SCHEMA_V1.to_string(),
            registry_blake3: registry.registry_blake3.clone(),
            repository_id: registry.repository_id.clone(),
            revision_kind: registry.revision.kind.clone(),
            revision: registry.revision.value.clone(),
            specification_set_blake3: registry.specification_set_blake3.clone(),
            policy_blake3: registry.policy_blake3.clone(),
            row_count: registry.rows.len(),
            non_claims: registry.non_claims.clone(),
            receipt_blake3: String::new(),
        };
        registry.receipt.receipt_blake3 = domain_hash_json(
            CAIRN_RECEIPT_DOMAIN,
            &CairnReceiptHashMaterial {
                schema_version: &registry.receipt.schema_version,
                registry_blake3: &registry.receipt.registry_blake3,
                repository_id: &registry.receipt.repository_id,
                revision_kind: &registry.receipt.revision_kind,
                revision: &registry.receipt.revision,
                specification_set_blake3: &registry.receipt.specification_set_blake3,
                policy_blake3: &registry.receipt.policy_blake3,
                row_count: registry.receipt.row_count,
                non_claims: &registry.receipt.non_claims,
            },
            "receipt",
        )
        .expect("receipt hash");
        registry
    }

    fn reference(registry: &CairnContentBoundRequirementRegistryV1) -> RequirementRefV1 {
        let row = &registry.rows[0];
        let mut reference = RequirementRefV1 {
            schema: VALENCE_REQUIREMENT_REF_SCHEMA_V1.to_string(),
            repository_id: registry.repository_id.clone(),
            revision: StackRevisionV1 {
                kind: StackRevisionKindV1::Opaque,
                value: registry.revision.value.clone(),
            },
            requirement_id: row.requirement_id.clone(),
            specification_path: row.specification_path.clone(),
            specification_identity: RequirementDigestV1 {
                domain: RequirementDigestDomainV1::Specification,
                blake3: row.specification_blake3.clone(),
            },
            requirement_body_identity: RequirementDigestV1 {
                domain: RequirementDigestDomainV1::RequirementBody,
                blake3: row.requirement_body_blake3.clone(),
            },
            policy_identity: RequirementDigestV1 {
                domain: RequirementDigestDomainV1::Policy,
                blake3: registry.policy_blake3.clone(),
            },
            row_identity: RequirementDigestV1 {
                domain: RequirementDigestDomainV1::RequirementRow,
                blake3: row.row_blake3.clone(),
            },
            registry_identity: RequirementDigestV1 {
                domain: RequirementDigestDomainV1::RequirementRegistry,
                blake3: registry.registry_blake3.clone(),
            },
            registry_receipt_identity: RequirementDigestV1 {
                domain: RequirementDigestDomainV1::RequirementRegistryReceipt,
                blake3: registry.receipt.receipt_blake3.clone(),
            },
            verification_role: ValenceVerificationRoleV1::RecordedOnly,
            non_claims: required_non_claims(VALENCE_REQUIREMENT_NON_CLAIMS),
            reference_identity: RequirementDigestV1 {
                domain: RequirementDigestDomainV1::RequirementReference,
                blake3: String::new(),
            },
        };
        reference.reference_identity.blake3 = requirement_ref_hash(&reference).expect("reference hash");
        reference
    }

    fn raw_evidence_row(
        path: &str,
        bundle: &str,
        role: ContentBoundEvidenceRoleV1,
        bytes: &[u8],
    ) -> ContentBoundMeasuredEvidenceRowV1 {
        ContentBoundMeasuredEvidenceRowV1 {
            schema: CONTENT_BOUND_EVIDENCE_ROW_SCHEMA_V1.to_string(),
            repository_id: REPOSITORY_ID.to_string(),
            repository_relative_path: path.to_string(),
            bundle_relative_path: bundle.to_string(),
            role,
            size_bytes: u64::try_from(bytes.len()).expect("fixture length"),
            content_identity: ContentBoundDigestV1 {
                domain: ContentBoundDigestDomainV1::EvidenceFile,
                blake3: blake3::hash(bytes).to_hex().to_string(),
            },
            span: Some(ContentBoundEvidenceSpanV1 {
                start_byte: 0,
                end_byte: u64::try_from(bytes.len()).expect("fixture length"),
                start_line: 1,
                end_line: 1,
            }),
            symbol: Some("fixture_symbol".to_string()),
            producer_receipt_row_identity: None,
            non_claims: required_non_claims(CONTENT_BOUND_EVIDENCE_ROW_NON_CLAIMS),
            row_identity: empty_content_digest(ContentBoundDigestDomainV1::EvidenceRow),
        }
    }

    fn evidence_rows() -> Vec<ContentBoundMeasuredEvidenceRowV1> {
        let receipt = seal_content_bound_evidence_row(raw_evidence_row(
            "evidence/receipt.json",
            "requirement-evidence/003-receipt.json",
            ContentBoundEvidenceRoleV1::Receipt,
            b"receipt",
        ))
        .expect("seal receipt");
        let receipt_identity = Some(receipt.row_identity.clone());
        let mut source = raw_evidence_row(
            "src/lib.rs",
            "requirement-evidence/001-source.rs",
            ContentBoundEvidenceRoleV1::Source,
            b"source",
        );
        source.producer_receipt_row_identity = receipt_identity.clone();
        let source = seal_content_bound_evidence_row(source).expect("seal source");
        let mut test = raw_evidence_row(
            "tests/release.rs",
            "requirement-evidence/002-test.rs",
            ContentBoundEvidenceRoleV1::Test,
            b"test",
        );
        test.producer_receipt_row_identity = receipt_identity;
        let test = seal_content_bound_evidence_row(test).expect("seal test");
        vec![source, test, receipt]
    }

    fn build_input() -> ContentBoundReleaseBuildInputV1 {
        let registry = registry();
        let reference = reference(&registry);
        ContentBoundReleaseBuildInputV1 {
            input: ContentBoundReleaseInputV1 {
                schema: CONTENT_BOUND_RELEASE_INPUT_SCHEMA_V1.to_string(),
                registries: vec![registry],
                requirement_refs: vec![reference.clone()],
                coverage: vec![
                    ContentBoundCoverageDraftV1 {
                        requirement_reference_blake3: reference.reference_identity.blake3.clone(),
                        evidence_role: ContentBoundEvidenceRoleV1::Source,
                        evidence_repository_paths: vec!["src/lib.rs".to_string()],
                    },
                    ContentBoundCoverageDraftV1 {
                        requirement_reference_blake3: reference.reference_identity.blake3,
                        evidence_role: ContentBoundEvidenceRoleV1::Test,
                        evidence_repository_paths: vec!["tests/release.rs".to_string()],
                    },
                ],
                non_claims: required_non_claims(CONTENT_BOUND_RELEASE_NON_CLAIMS),
            },
            evidence_rows: evidence_rows(),
            release_id: "mantle-content-bound-v1".to_string(),
            binary_digests_blake3: vec![blake3::hash(b"binary").to_hex().to_string()],
        }
    }

    fn external_links(evidence: &ContentBoundReleaseEvidenceV1) -> Vec<ContentBoundExternalEvidenceLinkV1> {
        evidence
            .evidence_manifest
            .rows
            .iter()
            .map(|row| ContentBoundExternalEvidenceLinkV1 {
                role: row.role.external_role().to_string(),
                schema: CONTENT_BOUND_EVIDENCE_EXTERNAL_SCHEMA_V1.to_string(),
                bundle_relative_path: row.bundle_relative_path.clone(),
                digest_blake3: row.content_identity.blake3.clone(),
            })
            .collect()
    }

    #[test]
    fn frozen_cairn_and_valence_contract_fixtures_match_selected_revisions() {
        let registry: CairnContentBoundRequirementRegistryV1 =
            serde_json::from_str(include_str!("../../../fixtures/content-bound-requirements/cairn-registry.json"))
                .expect("parse frozen Cairn registry");
        let reference: RequirementRefV1 =
            serde_json::from_str(include_str!("../../../fixtures/content-bound-requirements/requirement-ref.json"))
                .expect("parse frozen Valence reference");
        let mut issues = Vec::new();
        validate_cairn_registry(&registry, "registry", &mut issues);
        validate_requirement_ref(&reference, &registry, "reference", &mut issues);
        sort_issues(&mut issues);
        assert!(issues.is_empty(), "issues: {issues:?}");

        let mut stale_reference = reference;
        stale_reference.registry_identity.blake3 = blake3::hash(b"stale selected registry").to_hex().to_string();
        let mut stale_issues = Vec::new();
        validate_requirement_ref(&stale_reference, &registry, "reference", &mut stale_issues);
        let stale_codes = stale_issues.iter().map(|issue| issue.code).collect::<BTreeSet<_>>();
        assert!(stale_codes.contains(&ContentBoundRequirementIssueCodeV1::StaleRegistry));
        assert!(stale_codes.contains(&ContentBoundRequirementIssueCodeV1::ReferenceIdentityStale));
    }

    #[test]
    fn selected_revision_integration_receipt_binds_exact_fixture_bytes() {
        let receipt: serde_json::Value =
            serde_json::from_str(include_str!("../../../fixtures/content-bound-requirements/integration-receipt.json"))
                .expect("parse integration receipt");
        assert_eq!(receipt["cairn_revision"], CAIRN_CONTENT_BOUND_REGISTRY_CONTRACT_REVISION);
        assert_eq!(receipt["valence_revision"], VALENCE_CONTENT_BOUND_REQUIREMENT_CONTRACT_REVISION);
        let fixtures = [
            (
                "fixtures/content-bound-requirements/cairn-registry.json",
                include_bytes!("../../../fixtures/content-bound-requirements/cairn-registry.json").as_slice(),
            ),
            (
                "fixtures/content-bound-requirements/requirement-ref.json",
                include_bytes!("../../../fixtures/content-bound-requirements/requirement-ref.json").as_slice(),
            ),
            (
                "fixtures/content-bound-requirements/mantle-registry.json",
                include_bytes!("../../../fixtures/content-bound-requirements/mantle-registry.json").as_slice(),
            ),
            (
                "fixtures/content-bound-requirements/mantle-requirement-ref.json",
                include_bytes!("../../../fixtures/content-bound-requirements/mantle-requirement-ref.json").as_slice(),
            ),
        ];
        for (path, bytes) in fixtures {
            let expected = receipt["fixtures"]
                .as_array()
                .expect("fixture rows")
                .iter()
                .find(|fixture| fixture["path"] == path)
                .and_then(|fixture| fixture["blake3"].as_str())
                .expect("fixture hash");
            assert_eq!(blake3::hash(bytes).to_hex().as_str(), expected);
        }
    }

    #[test]
    fn valid_content_bound_release_is_order_independent() {
        // r[verify mantle.release_provenance.content_bound_requirement_coverage]
        let first = build_content_bound_release_evidence(build_input()).expect("valid content-bound release");
        let mut reordered = build_input();
        reordered.evidence_rows.reverse();
        reordered.input.coverage.reverse();
        let second = build_content_bound_release_evidence(reordered).expect("reordered content-bound release");
        assert_eq!(first.evidence_identity, second.evidence_identity);
        let verification = validate_content_bound_release_evidence(
            &first,
            &first.release_id,
            &first.binary_identities.iter().map(|identity| identity.blake3.clone()).collect::<Vec<_>>(),
            &external_links(&first),
            CONTENT_BOUND_REQUIREMENT_MODE_REQUIRED,
        );
        assert!(verification.valid, "issues: {:?}", verification.issues);
    }

    #[test]
    fn stale_registry_wrong_repository_and_domain_substitution_fail() {
        let mut wrong_repository = build_input();
        wrong_repository.input.requirement_refs[0].repository_id = "OnixResearch/other".to_string();
        let wrong_repository_issues =
            build_content_bound_release_evidence(wrong_repository).expect_err("cross-repository input must fail");
        let wrong_repository_codes = wrong_repository_issues.iter().map(|issue| issue.code).collect::<BTreeSet<_>>();
        assert!(wrong_repository_codes.contains(&ContentBoundRequirementIssueCodeV1::RegistryMissing));

        let mut stale = build_input();
        stale.input.requirement_refs[0].registry_identity.blake3 = blake3::hash(b"stale registry").to_hex().to_string();
        stale.input.requirement_refs[0].row_identity.domain = RequirementDigestDomainV1::Policy;
        let stale_issues =
            build_content_bound_release_evidence(stale).expect_err("stale domain-substituted input must fail");
        let stale_codes = stale_issues.iter().map(|issue| issue.code).collect::<BTreeSet<_>>();
        assert!(stale_codes.contains(&ContentBoundRequirementIssueCodeV1::StaleRegistry));
        assert!(stale_codes.contains(&ContentBoundRequirementIssueCodeV1::DigestDomainSubstitution));
        assert!(stale_codes.contains(&ContentBoundRequirementIssueCodeV1::ReferenceIdentityStale));
    }

    #[test]
    fn duplicate_coverage_missing_receipt_and_weakened_nonclaims_fail() {
        let mut duplicate = build_input();
        duplicate.input.coverage.push(duplicate.input.coverage[0].clone());
        let duplicate_issues =
            build_content_bound_release_evidence(duplicate).expect_err("duplicate coverage must fail");
        let duplicate_codes = duplicate_issues.iter().map(|issue| issue.code).collect::<BTreeSet<_>>();
        assert!(
            duplicate_codes.contains(&ContentBoundRequirementIssueCodeV1::CoverageDuplicate),
            "codes: {duplicate_codes:?}"
        );

        let mut weakened = build_input();
        weakened.evidence_rows[0].producer_receipt_row_identity = None;
        weakened.input.non_claims.clear();
        let weakened_issues = build_content_bound_release_evidence(weakened).expect_err("weakened coverage must fail");
        let weakened_codes = weakened_issues.iter().map(|issue| issue.code).collect::<BTreeSet<_>>();
        assert!(weakened_codes.contains(&ContentBoundRequirementIssueCodeV1::EvidenceReceiptLinkMissing));
        assert!(weakened_codes.contains(&ContentBoundRequirementIssueCodeV1::NonClaimMissing));
    }

    #[test]
    fn producer_receipt_from_a_different_repository_fails() {
        let mut input = build_input();
        let receipt_index = input
            .evidence_rows
            .iter()
            .position(|row| row.role == ContentBoundEvidenceRoleV1::Receipt)
            .expect("receipt row");
        let mut receipt = input.evidence_rows[receipt_index].clone();
        receipt.repository_id = "OnixResearch/other".to_string();
        receipt.row_identity = empty_content_digest(ContentBoundDigestDomainV1::EvidenceRow);
        let receipt = seal_content_bound_evidence_row(receipt).expect("reseal cross-repository receipt");
        for row in input.evidence_rows.iter_mut().filter(|row| row.role != ContentBoundEvidenceRoleV1::Receipt) {
            row.producer_receipt_row_identity = Some(receipt.row_identity.clone());
            row.row_identity = empty_content_digest(ContentBoundDigestDomainV1::EvidenceRow);
            *row = seal_content_bound_evidence_row(row.clone()).expect("reseal evidence row");
        }
        input.evidence_rows[receipt_index] = receipt;

        let issues =
            build_content_bound_release_evidence(input).expect_err("cross-repository producer receipt must fail");
        assert!(
            issues
                .iter()
                .any(|issue| issue.code == ContentBoundRequirementIssueCodeV1::EvidenceReceiptLinkInvalid)
        );
    }

    #[test]
    fn strict_mode_rejects_missing_and_invalid_optional_evidence() {
        let absent = evaluate_optional_content_bound_release_evidence(
            None,
            "release",
            &[],
            &[],
            CONTENT_BOUND_REQUIREMENT_MODE_REQUIRED,
        );
        assert!(!absent.valid);
        assert_eq!(absent.disposition, CONTENT_BOUND_REQUIREMENT_DISPOSITION_ABSENT);

        let evidence = build_content_bound_release_evidence(build_input()).expect("valid content-bound release");
        let mut stale_links = external_links(&evidence);
        stale_links[0].digest_blake3 = blake3::hash(b"stale evidence").to_hex().to_string();
        let invalid = evaluate_optional_content_bound_release_evidence(
            Some(&evidence),
            &evidence.release_id,
            &evidence.binary_identities.iter().map(|identity| identity.blake3.clone()).collect::<Vec<_>>(),
            &stale_links,
            CONTENT_BOUND_REQUIREMENT_MODE_OPTIONAL,
        );
        assert!(!invalid.valid);
        assert_eq!(invalid.disposition, CONTENT_BOUND_REQUIREMENT_DISPOSITION_INVALID);
    }

    #[test]
    fn legacy_optional_absence_remains_bounded_and_readable() {
        // r[verify mantle.release_provenance.legacy_coverage_boundary]
        let verification = evaluate_optional_content_bound_release_evidence(
            None,
            "legacy-release",
            &[],
            &[],
            CONTENT_BOUND_REQUIREMENT_MODE_OPTIONAL,
        );
        assert!(verification.valid);
        assert_eq!(verification.disposition, CONTENT_BOUND_REQUIREMENT_DISPOSITION_ABSENT);
        assert_eq!(verification.boundary, LEGACY_COVERAGE_BOUNDARY);
    }
}
