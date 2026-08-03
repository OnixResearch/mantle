// r[impl mantlepkgs_impact.compatible_snapshots]
// r[impl mantlepkgs_impact.catalog_dispositions]
// r[impl mantlepkgs_impact.outcome_transitions]
// r[impl mantlepkgs_impact.closure_deltas]
// r[impl mantlepkgs_impact.external_ci_boundary]
// r[impl mantlepkgs_impact.functional_core]
// r[impl mantlepkgs_impact.claim_boundary]
// r[verify mantlepkgs_impact.compatible_snapshots]
// r[verify mantlepkgs_impact.catalog_dispositions]
// r[verify mantlepkgs_impact.outcome_transitions]
// r[verify mantlepkgs_impact.closure_deltas]
// r[verify mantlepkgs_impact.functional_core]
// r[verify mantlepkgs_impact.claim_boundary]

use alloc::collections::BTreeMap;
use alloc::collections::BTreeSet;
use alloc::format;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec;
use alloc::vec::Vec;

use serde::Deserialize;
use serde::Serialize;

use crate::BLAKE3_HEX_LENGTH;
use crate::CoreFailure;
use crate::Diagnostic;

pub const IMPACT_POLICY_SCHEMA: &str = "mantlepkgs-impact-policy-v1";
pub const IMPACT_SNAPSHOT_SCHEMA: &str = "mantlepkgs-impact-snapshot-v1";
pub const IMPACT_PACKAGE_RECORD_SCHEMA: &str = "mantlepkgs-impact-package-record-v1";
pub const IMPACT_OBSERVATION_SCHEMA: &str = "mantlepkgs-impact-observation-v1";
pub const IMPACT_CLOSURE_SCHEMA: &str = "mantlepkgs-impact-closure-v1";
pub const IMPACT_REPORT_SCHEMA: &str = "mantle-package-impact-v1";
pub const IMPACT_IDENTITY_DOMAIN: &str = "mantle.mantlepkgs.impact-record.v1";
pub const ACTION_RESULT_RUNTIME_REPORT_SCHEMA: &str = "mantle-action-result-runtime-report-v1";
pub const REALIZATION_RECEIPT_SCHEMA: &str = "mantle-foreign-realization-receipt-v1";
pub const EXPLICIT_STATUS_SCHEMA: &str = "mantlepkgs-impact-explicit-status-v1";

const IMPACT_POLICY_IDENTITY_DOMAIN: &[u8] = b"mantle.mantlepkgs.impact-policy.v1";
const IMPACT_PACKAGE_IDENTITY_DOMAIN: &[u8] = b"mantle.mantlepkgs.impact-package-record.v1";
const IMPACT_OBSERVATION_IDENTITY_DOMAIN: &[u8] = b"mantle.mantlepkgs.impact-observation.v1";
const IMPACT_CLOSURE_IDENTITY_DOMAIN: &[u8] = b"mantle.mantlepkgs.impact-closure.v1";
const IMPACT_SNAPSHOT_IDENTITY_DOMAIN: &[u8] = b"mantle.mantlepkgs.impact-snapshot.v1";
const IMPACT_REPORT_IDENTITY_DOMAIN: &[u8] = b"mantle.mantlepkgs.impact-report.v1";
const DOMAIN_SEPARATOR: u8 = 0;
const MIN_ITEMS: u32 = 1;
const MIN_DIAGNOSTICS: u32 = 16;
const MAX_TEXT_BYTES: usize = 4_096;
const MAX_PACKAGES: u32 = 65_536;
const MAX_VARIANTS: u32 = 16_384;
const MAX_CLOSURE_MEMBERS: u32 = 1_048_576;
const MAX_DEPENDENCY_EDGES: u32 = 4_194_304;
const MAX_OBSERVATIONS: u32 = 65_536;
const MAX_DIAGNOSTICS: u32 = 4_096;
const MAX_REPORT_BYTES: u64 = 67_108_864;
const EXPECTED_NON_CLAIM_COUNT: usize = 6;
const GLOBAL_COMPATIBILITY_FACT_COUNT: usize = 8;
const CHANGED_DIMENSION_COUNT: usize = 4;
const CLOSURE_REASON_CAPACITY: usize = 16;
const MIN_STORE_PATH_SUFFIX_BYTES: usize = 2;
const ZERO_DIGEST_BYTE: u8 = b'0';

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ImpactComparisonPolicy {
    pub schema: String,
    pub policy_identity_blake3: String,
    pub max_packages: u32,
    pub max_variants: u32,
    pub max_closure_members: u32,
    pub max_dependency_edges: u32,
    pub max_observations: u32,
    pub max_diagnostics: u32,
    pub max_report_bytes: u64,
}

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ImpactPackageKey {
    pub public_selector: String,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ImpactPackageKind {
    Package,
    Variant,
}

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ImpactBlocker {
    pub code: String,
    pub detail_identity_blake3: String,
}

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ImpactVariantFacts {
    pub base_package_identity_blake3: String,
    pub base_root_identity_blake3: String,
    pub variant_name: String,
    pub provenance_identity_blake3: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ImpactPackageRecord {
    pub schema: String,
    pub record_identity_blake3: String,
    pub key: ImpactPackageKey,
    pub kind: ImpactPackageKind,
    pub package_identity_blake3: String,
    pub root_identity_blake3: String,
    pub recipe_identity_blake3: String,
    pub policy_identity_blake3: String,
    pub is_buildable: bool,
    pub blockers: Vec<ImpactBlocker>,
    pub variant: Option<ImpactVariantFacts>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum BuildObservationState {
    Success,
    Failure,
    Blocked,
    NotAttempted,
    Unavailable,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, tag = "kind", rename_all = "kebab-case")]
pub enum BuildObservationAuthority {
    AdmittedActionResult {
        runtime_report_schema: String,
        runtime_report_identity_blake3: String,
        action_ref: String,
        selected_result_ref: String,
        request_identity_blake3: String,
        policy_identity_blake3: String,
        platform_identity_blake3: String,
        signature_identity_blake3: String,
        output_set_identity_blake3: String,
        cas_identity_blake3: String,
    },
    RealizationReceipt {
        receipt_schema: String,
        receipt_identity_blake3: String,
    },
    ExplicitStatus {
        status_schema: String,
        status_identity_blake3: String,
    },
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct BuildObservation {
    pub schema: String,
    pub observation_identity_blake3: String,
    pub key: ImpactPackageKey,
    pub package_record_identity_blake3: String,
    pub state: BuildObservationState,
    pub terminal_reason_codes: Vec<String>,
    pub authority: BuildObservationAuthority,
}

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ClosureMember {
    pub member_identity_blake3: String,
    pub store_path: String,
    pub path_info_identity_blake3: Option<String>,
    pub logical_bytes: Option<u64>,
    pub references: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PackageClosure {
    pub schema: String,
    pub closure_identity_blake3: String,
    pub key: ImpactPackageKey,
    pub package_record_identity_blake3: String,
    pub root_member_identity_blake3: String,
    pub system: String,
    pub store_prefix: String,
    pub closure_semantics: String,
    pub byte_semantics: String,
    pub is_complete: bool,
    pub incomplete_reason_codes: Vec<String>,
    pub members: Vec<ClosureMember>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ImpactSnapshot {
    pub schema: String,
    pub snapshot_identity_blake3: String,
    pub comparison_policy_identity_blake3: String,
    pub catalog_schema: String,
    pub catalog_identity_blake3: String,
    pub system: String,
    pub store_prefix: String,
    pub conversion_policy_identity_blake3: String,
    pub package_record_schema: String,
    pub observation_schema: String,
    pub identity_domain: String,
    pub packages: Vec<ImpactPackageRecord>,
    pub observations: Vec<BuildObservation>,
    pub closures: Vec<PackageClosure>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum CatalogDisposition {
    Added,
    Removed,
    Unchanged,
    RecipeChanged,
    PolicyChanged,
    BlockerChanged,
    VariantChanged,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ChangedDimension {
    Recipe,
    Policy,
    Blocker,
    Variant,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum BuildTransitionDisposition {
    NewlySuccessful,
    StillSuccessful,
    NewlyFailed,
    StillFailed,
    Blocked,
    NotAttempted,
    Unavailable,
    Missing,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct BuildOutcomeTransition {
    pub disposition: BuildTransitionDisposition,
    pub base_state: Option<BuildObservationState>,
    pub head_state: Option<BuildObservationState>,
    pub base_observation_identity_blake3: Option<String>,
    pub head_observation_identity_blake3: Option<String>,
    pub base_terminal_reason_codes: Vec<String>,
    pub head_terminal_reason_codes: Vec<String>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum DeltaDirection {
    Increase,
    Decrease,
    Unchanged,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LogicalByteDelta {
    pub direction: DeltaDirection,
    pub magnitude_bytes: u64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ClosureDelta {
    pub is_comparable: bool,
    pub reason_codes: Vec<String>,
    pub base_closure_identity_blake3: Option<String>,
    pub head_closure_identity_blake3: Option<String>,
    pub added_member_identities: Option<Vec<String>>,
    pub removed_member_identities: Option<Vec<String>>,
    pub retained_dependency_identities: Option<Vec<String>>,
    pub member_count_delta: Option<i64>,
    pub logical_byte_delta: Option<LogicalByteDelta>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PackageImpact {
    pub key: ImpactPackageKey,
    pub catalog_disposition: CatalogDisposition,
    pub changed_dimensions: Vec<ChangedDimension>,
    pub base_record_identity_blake3: Option<String>,
    pub head_record_identity_blake3: Option<String>,
    pub build_transition: BuildOutcomeTransition,
    pub closure_delta: ClosureDelta,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MantlePackageImpactReport {
    pub schema: String,
    pub report_identity_blake3: String,
    pub comparison_policy_identity_blake3: String,
    pub base_snapshot_identity_blake3: String,
    pub head_snapshot_identity_blake3: String,
    pub base_catalog_identity_blake3: String,
    pub head_catalog_identity_blake3: String,
    pub system: String,
    pub store_prefix: String,
    pub conversion_policy_identity_blake3: String,
    pub package_impacts: Vec<PackageImpact>,
    pub non_claims: Vec<String>,
}

pub struct ImpactComparisonInput<'a> {
    pub policy: &'a ImpactComparisonPolicy,
    pub base: &'a ImpactSnapshot,
    pub head: &'a ImpactSnapshot,
}

struct NormalizedSnapshot {
    snapshot: ImpactSnapshot,
    packages: BTreeMap<ImpactPackageKey, ImpactPackageRecord>,
    observations: BTreeMap<ImpactPackageKey, BuildObservation>,
    closures: BTreeMap<ImpactPackageKey, PackageClosure>,
}

struct ActionResultAuthorityRef<'a> {
    runtime_report_schema: &'a str,
    runtime_report_identity_blake3: &'a str,
    action_ref: &'a str,
    selected_result_ref: &'a str,
    request_identity_blake3: &'a str,
    policy_identity_blake3: &'a str,
    platform_identity_blake3: &'a str,
    signature_identity_blake3: &'a str,
    output_set_identity_blake3: &'a str,
    cas_identity_blake3: &'a str,
}

#[derive(Serialize)]
struct PolicyIdentityPreimage<'a> {
    schema: &'a str,
    max_packages: u32,
    max_variants: u32,
    max_closure_members: u32,
    max_dependency_edges: u32,
    max_observations: u32,
    max_diagnostics: u32,
    max_report_bytes: u64,
}

#[derive(Serialize)]
struct PackageIdentityPreimage<'a> {
    schema: &'a str,
    key: &'a ImpactPackageKey,
    kind: ImpactPackageKind,
    package_identity_blake3: &'a str,
    root_identity_blake3: &'a str,
    recipe_identity_blake3: &'a str,
    policy_identity_blake3: &'a str,
    is_buildable: bool,
    blockers: &'a [ImpactBlocker],
    variant: &'a Option<ImpactVariantFacts>,
}

#[derive(Serialize)]
struct ObservationIdentityPreimage<'a> {
    schema: &'a str,
    key: &'a ImpactPackageKey,
    package_record_identity_blake3: &'a str,
    state: BuildObservationState,
    terminal_reason_codes: &'a [String],
    authority: &'a BuildObservationAuthority,
}

#[derive(Serialize)]
struct ClosureIdentityPreimage<'a> {
    schema: &'a str,
    key: &'a ImpactPackageKey,
    package_record_identity_blake3: &'a str,
    root_member_identity_blake3: &'a str,
    system: &'a str,
    store_prefix: &'a str,
    closure_semantics: &'a str,
    byte_semantics: &'a str,
    is_complete: bool,
    incomplete_reason_codes: &'a [String],
    members: &'a [ClosureMember],
}

#[derive(Serialize)]
struct SnapshotIdentityPreimage<'a> {
    schema: &'a str,
    comparison_policy_identity_blake3: &'a str,
    catalog_schema: &'a str,
    catalog_identity_blake3: &'a str,
    system: &'a str,
    store_prefix: &'a str,
    conversion_policy_identity_blake3: &'a str,
    package_record_schema: &'a str,
    observation_schema: &'a str,
    identity_domain: &'a str,
    packages: &'a [ImpactPackageRecord],
    observations: &'a [BuildObservation],
    closures: &'a [PackageClosure],
}

#[derive(Serialize)]
struct ReportIdentityPreimage<'a> {
    schema: &'a str,
    comparison_policy_identity_blake3: &'a str,
    base_snapshot_identity_blake3: &'a str,
    head_snapshot_identity_blake3: &'a str,
    base_catalog_identity_blake3: &'a str,
    head_catalog_identity_blake3: &'a str,
    system: &'a str,
    store_prefix: &'a str,
    conversion_policy_identity_blake3: &'a str,
    package_impacts: &'a [PackageImpact],
    non_claims: &'a [String],
}

pub fn impact_policy_identity_blake3(policy: &ImpactComparisonPolicy) -> Result<String, CoreFailure> {
    digest_serializable(
        IMPACT_POLICY_IDENTITY_DOMAIN,
        &PolicyIdentityPreimage {
            schema: &policy.schema,
            max_packages: policy.max_packages,
            max_variants: policy.max_variants,
            max_closure_members: policy.max_closure_members,
            max_dependency_edges: policy.max_dependency_edges,
            max_observations: policy.max_observations,
            max_diagnostics: policy.max_diagnostics,
            max_report_bytes: policy.max_report_bytes,
        },
        "impact-policy-identity-serialization-failed",
    )
}

pub fn seal_impact_policy(policy: &ImpactComparisonPolicy) -> Result<ImpactComparisonPolicy, CoreFailure> {
    validate_policy_limits(policy)?;
    let mut sealed = policy.clone();
    let observed = impact_policy_identity_blake3(&sealed)?;
    seal_digest_field(&mut sealed.policy_identity_blake3, &observed, "policy.policy_identity_blake3")?;
    Ok(sealed)
}

pub fn impact_package_record_identity_blake3(record: &ImpactPackageRecord) -> Result<String, CoreFailure> {
    let mut blockers = record.blockers.clone();
    blockers.sort();
    blockers.dedup();
    debug_assert!(blockers.len() <= record.blockers.len());
    debug_assert!(blockers.windows(2).all(|pair| pair[0] < pair[1]));
    digest_serializable(
        IMPACT_PACKAGE_IDENTITY_DOMAIN,
        &PackageIdentityPreimage {
            schema: &record.schema,
            key: &record.key,
            kind: record.kind,
            package_identity_blake3: &record.package_identity_blake3,
            root_identity_blake3: &record.root_identity_blake3,
            recipe_identity_blake3: &record.recipe_identity_blake3,
            policy_identity_blake3: &record.policy_identity_blake3,
            is_buildable: record.is_buildable,
            blockers: &blockers,
            variant: &record.variant,
        },
        "impact-package-identity-serialization-failed",
    )
}

pub fn impact_observation_identity_blake3(observation: &BuildObservation) -> Result<String, CoreFailure> {
    let mut terminal_reason_codes = observation.terminal_reason_codes.clone();
    terminal_reason_codes.sort();
    terminal_reason_codes.dedup();
    digest_serializable(
        IMPACT_OBSERVATION_IDENTITY_DOMAIN,
        &ObservationIdentityPreimage {
            schema: &observation.schema,
            key: &observation.key,
            package_record_identity_blake3: &observation.package_record_identity_blake3,
            state: observation.state,
            terminal_reason_codes: &terminal_reason_codes,
            authority: &observation.authority,
        },
        "impact-observation-identity-serialization-failed",
    )
}

pub fn impact_closure_identity_blake3(closure: &PackageClosure) -> Result<String, CoreFailure> {
    let mut normalized = closure.clone();
    normalize_closure_fields(&mut normalized);
    debug_assert_eq!(normalized.members.len(), closure.members.len());
    debug_assert!(normalized.members.windows(2).all(|pair| pair[0] <= pair[1]));
    digest_serializable(
        IMPACT_CLOSURE_IDENTITY_DOMAIN,
        &ClosureIdentityPreimage {
            schema: &normalized.schema,
            key: &normalized.key,
            package_record_identity_blake3: &normalized.package_record_identity_blake3,
            root_member_identity_blake3: &normalized.root_member_identity_blake3,
            system: &normalized.system,
            store_prefix: &normalized.store_prefix,
            closure_semantics: &normalized.closure_semantics,
            byte_semantics: &normalized.byte_semantics,
            is_complete: normalized.is_complete,
            incomplete_reason_codes: &normalized.incomplete_reason_codes,
            members: &normalized.members,
        },
        "impact-closure-identity-serialization-failed",
    )
}

pub fn seal_impact_snapshot(
    policy: &ImpactComparisonPolicy,
    snapshot: &ImpactSnapshot,
) -> Result<ImpactSnapshot, CoreFailure> {
    let sealed_policy = seal_impact_policy(policy)?;
    let normalized = normalize_snapshot(&sealed_policy, snapshot)?;
    Ok(normalized.snapshot)
}

pub fn build_package_impact_report(input: ImpactComparisonInput<'_>) -> Result<MantlePackageImpactReport, CoreFailure> {
    let policy = seal_impact_policy(input.policy)?;
    let base = normalize_snapshot(&policy, input.base)?;
    let head = normalize_snapshot(&policy, input.head)?;
    validate_comparison_diagnostic_facts(&policy, &base.snapshot, &head.snapshot)?;
    validate_snapshot_compatibility(&policy, &base.snapshot, &head.snapshot)?;
    let package_impacts = compare_packages(&base, &head)?;
    let non_claims = report_non_claims();
    debug_assert_eq!(non_claims.len(), EXPECTED_NON_CLAIM_COUNT);
    let mut impact_document = MantlePackageImpactReport {
        schema: IMPACT_REPORT_SCHEMA.into(),
        report_identity_blake3: String::new(),
        comparison_policy_identity_blake3: policy.policy_identity_blake3.clone(),
        base_snapshot_identity_blake3: base.snapshot.snapshot_identity_blake3.clone(),
        head_snapshot_identity_blake3: head.snapshot.snapshot_identity_blake3.clone(),
        base_catalog_identity_blake3: base.snapshot.catalog_identity_blake3.clone(),
        head_catalog_identity_blake3: head.snapshot.catalog_identity_blake3.clone(),
        system: base.snapshot.system.clone(),
        store_prefix: base.snapshot.store_prefix.clone(),
        conversion_policy_identity_blake3: base.snapshot.conversion_policy_identity_blake3.clone(),
        package_impacts,
        non_claims,
    };
    impact_document.report_identity_blake3 = impact_report_identity_blake3(&impact_document)?;
    enforce_report_size(&impact_document, policy.max_report_bytes)?;
    debug_assert_eq!(impact_document.report_identity_blake3.len(), BLAKE3_HEX_LENGTH);
    Ok(impact_document)
}

pub fn impact_report_identity_blake3(report: &MantlePackageImpactReport) -> Result<String, CoreFailure> {
    digest_serializable(
        IMPACT_REPORT_IDENTITY_DOMAIN,
        &ReportIdentityPreimage {
            schema: &report.schema,
            comparison_policy_identity_blake3: &report.comparison_policy_identity_blake3,
            base_snapshot_identity_blake3: &report.base_snapshot_identity_blake3,
            head_snapshot_identity_blake3: &report.head_snapshot_identity_blake3,
            base_catalog_identity_blake3: &report.base_catalog_identity_blake3,
            head_catalog_identity_blake3: &report.head_catalog_identity_blake3,
            system: &report.system,
            store_prefix: &report.store_prefix,
            conversion_policy_identity_blake3: &report.conversion_policy_identity_blake3,
            package_impacts: &report.package_impacts,
            non_claims: &report.non_claims,
        },
        "impact-report-identity-serialization-failed",
    )
}

fn normalize_snapshot(
    policy: &ImpactComparisonPolicy,
    snapshot: &ImpactSnapshot,
) -> Result<NormalizedSnapshot, CoreFailure> {
    validate_snapshot_header(policy, snapshot)?;
    validate_snapshot_counts(policy, snapshot)?;
    let packages = normalize_packages(policy, &snapshot.packages)?;
    let observations = normalize_observations(policy, &snapshot.observations, &packages)?;
    let closures = normalize_closures(policy, snapshot, &packages)?;
    let mut normalized = snapshot.clone();
    normalized.comparison_policy_identity_blake3 = policy.policy_identity_blake3.clone();
    normalized.packages = packages.values().cloned().collect();
    normalized.observations = observations.values().cloned().collect();
    normalized.closures = closures.values().cloned().collect();
    let observed_identity = snapshot_identity_blake3(&normalized)?;
    seal_digest_field(
        &mut normalized.snapshot_identity_blake3,
        &observed_identity,
        "snapshot.snapshot_identity_blake3",
    )?;
    debug_assert!(normalized.packages.windows(2).all(|pair| pair[0].key < pair[1].key));
    debug_assert!(normalized.observations.windows(2).all(|pair| pair[0].key < pair[1].key));
    debug_assert!(normalized.closures.windows(2).all(|pair| pair[0].key < pair[1].key));
    Ok(NormalizedSnapshot {
        snapshot: normalized,
        packages,
        observations,
        closures,
    })
}

fn validate_snapshot_header(policy: &ImpactComparisonPolicy, snapshot: &ImpactSnapshot) -> Result<(), CoreFailure> {
    require_equal(&snapshot.schema, IMPACT_SNAPSHOT_SCHEMA, "snapshot.schema", "unsupported-impact-snapshot-schema")?;
    require_equal(
        &snapshot.package_record_schema,
        IMPACT_PACKAGE_RECORD_SCHEMA,
        "snapshot.package_record_schema",
        "unsupported-impact-package-record-schema",
    )?;
    require_equal(
        &snapshot.observation_schema,
        IMPACT_OBSERVATION_SCHEMA,
        "snapshot.observation_schema",
        "unsupported-impact-observation-schema",
    )?;
    require_equal(
        &snapshot.identity_domain,
        IMPACT_IDENTITY_DOMAIN,
        "snapshot.identity_domain",
        "unsupported-impact-identity-domain",
    )?;
    require_digest(&snapshot.catalog_identity_blake3, "snapshot.catalog_identity_blake3")?;
    require_digest(&snapshot.conversion_policy_identity_blake3, "snapshot.conversion_policy_identity_blake3")?;
    require_text(&snapshot.catalog_schema, "snapshot.catalog_schema")?;
    require_text(&snapshot.system, "snapshot.system")?;
    require_store_prefix(&snapshot.store_prefix, "snapshot.store_prefix")?;
    if !is_placeholder_digest(&snapshot.comparison_policy_identity_blake3)
        && snapshot.comparison_policy_identity_blake3 != policy.policy_identity_blake3
    {
        return Err(failure(
            "impact-policy-binding-mismatch",
            "snapshot.comparison_policy_identity_blake3",
            "the snapshot names a different comparison policy",
        ));
    }
    debug_assert_eq!(snapshot.schema, IMPACT_SNAPSHOT_SCHEMA);
    debug_assert_eq!(snapshot.identity_domain, IMPACT_IDENTITY_DOMAIN);
    Ok(())
}

fn validate_snapshot_counts(policy: &ImpactComparisonPolicy, snapshot: &ImpactSnapshot) -> Result<(), CoreFailure> {
    let variant_count = snapshot.packages.iter().filter(|record| record.kind == ImpactPackageKind::Variant).count();
    require_count(snapshot.packages.len(), policy.max_packages, "snapshot.packages", "impact-package-limit-exceeded")?;
    require_count(variant_count, policy.max_variants, "snapshot.packages", "impact-variant-limit-exceeded")?;
    require_count(
        snapshot.observations.len(),
        policy.max_observations,
        "snapshot.observations",
        "impact-observation-limit-exceeded",
    )?;
    require_count(
        snapshot_diagnostic_fact_count(snapshot)?,
        policy.max_diagnostics,
        "snapshot.diagnostic_facts",
        "impact-diagnostic-limit-exceeded",
    )?;
    require_count(snapshot.closures.len(), policy.max_packages, "snapshot.closures", "impact-closure-limit-exceeded")?;
    let (member_count, edge_count) = closure_fact_counts(&snapshot.closures)?;
    require_count(
        member_count,
        policy.max_closure_members,
        "snapshot.closures.members",
        "impact-closure-member-limit-exceeded",
    )?;
    require_count(
        edge_count,
        policy.max_dependency_edges,
        "snapshot.closures.members.references",
        "impact-dependency-edge-limit-exceeded",
    )?;
    debug_assert!(variant_count <= snapshot.packages.len());
    debug_assert!(member_count >= snapshot.closures.len() || snapshot.closures.is_empty());
    Ok(())
}

fn snapshot_diagnostic_fact_count(snapshot: &ImpactSnapshot) -> Result<usize, CoreFailure> {
    let observation_reason_count = snapshot.observations.iter().try_fold(0usize, |count, observation| {
        count.checked_add(observation.terminal_reason_codes.len()).ok_or_else(|| {
            failure(
                "impact-diagnostic-count-overflow",
                "snapshot.observations.terminal_reason_codes",
                "the observation reason count overflowed usize",
            )
        })
    })?;
    let total_count = snapshot.closures.iter().try_fold(observation_reason_count, |count, closure| {
        count.checked_add(closure.incomplete_reason_codes.len()).ok_or_else(|| {
            failure(
                "impact-diagnostic-count-overflow",
                "snapshot.closures.incomplete_reason_codes",
                "the closure reason count overflowed usize",
            )
        })
    })?;
    debug_assert!(total_count >= observation_reason_count);
    debug_assert!(
        total_count >= snapshot.closures.iter().filter(|closure| !closure.incomplete_reason_codes.is_empty()).count()
    );
    Ok(total_count)
}

fn validate_comparison_diagnostic_facts(
    policy: &ImpactComparisonPolicy,
    base: &ImpactSnapshot,
    head: &ImpactSnapshot,
) -> Result<(), CoreFailure> {
    let base_count = snapshot_diagnostic_fact_count(base)?;
    let head_count = snapshot_diagnostic_fact_count(head)?;
    let total_count = base_count.checked_add(head_count).ok_or_else(|| {
        failure(
            "impact-diagnostic-count-overflow",
            "comparison.diagnostic_facts",
            "the comparison diagnostic count overflowed usize",
        )
    })?;
    require_count(
        total_count,
        policy.max_diagnostics,
        "comparison.diagnostic_facts",
        "impact-diagnostic-limit-exceeded",
    )?;
    debug_assert!(total_count >= base_count);
    debug_assert!(total_count >= head_count);
    Ok(())
}

fn closure_fact_counts(closures: &[PackageClosure]) -> Result<(usize, usize), CoreFailure> {
    let mut member_count = 0usize;
    let mut edge_count = 0usize;
    for closure in closures {
        member_count = member_count.checked_add(closure.members.len()).ok_or_else(|| {
            failure(
                "impact-closure-member-count-overflow",
                "snapshot.closures.members",
                "the closure member count overflowed usize",
            )
        })?;
        for member in &closure.members {
            edge_count = edge_count.checked_add(member.references.len()).ok_or_else(|| {
                failure(
                    "impact-dependency-edge-count-overflow",
                    "snapshot.closures.members.references",
                    "the dependency edge count overflowed usize",
                )
            })?;
        }
    }
    debug_assert!(member_count >= closures.len() || closures.is_empty());
    debug_assert!(
        edge_count
            >= closures
                .iter()
                .flat_map(|closure| &closure.members)
                .filter(|member| !member.references.is_empty())
                .count()
    );
    Ok((member_count, edge_count))
}

fn normalize_packages(
    policy: &ImpactComparisonPolicy,
    records: &[ImpactPackageRecord],
) -> Result<BTreeMap<ImpactPackageKey, ImpactPackageRecord>, CoreFailure> {
    let record_count_max = records.len();
    let mut packages = BTreeMap::new();
    let mut identities = BTreeMap::<String, ImpactPackageKey>::new();
    for record in records {
        let normalized = normalize_package_record(record)?;
        if let Some(existing) = packages.get(&normalized.key) {
            if existing != &normalized {
                return Err(failure(
                    "impact-package-key-conflict",
                    &normalized.key.public_selector,
                    "more than one different package record claims the key",
                ));
            }
            continue;
        }
        if let Some(existing_key) = identities.get(&normalized.record_identity_blake3)
            && existing_key != &normalized.key
        {
            return Err(failure(
                "impact-package-identity-conflict",
                &normalized.record_identity_blake3,
                "one record identity claims more than one package key",
            ));
        }
        if identities.len() >= record_count_max {
            return Err(failure(
                "impact-package-identity-bound-exceeded",
                "snapshot.packages",
                "normalized package identities exceed the supplied record count",
            ));
        }
        identities.insert(normalized.record_identity_blake3.clone(), normalized.key.clone());
        if packages.len() >= record_count_max {
            return Err(failure(
                "impact-package-normalization-bound-exceeded",
                "snapshot.packages",
                "normalized package records exceed the supplied record count",
            ));
        }
        packages.insert(normalized.key.clone(), normalized);
    }
    require_count(packages.len(), policy.max_packages, "snapshot.packages", "impact-package-limit-exceeded")?;
    debug_assert!(packages.len() <= records.len());
    Ok(packages)
}

fn normalize_package_record(record: &ImpactPackageRecord) -> Result<ImpactPackageRecord, CoreFailure> {
    require_equal(
        &record.schema,
        IMPACT_PACKAGE_RECORD_SCHEMA,
        "package.schema",
        "unsupported-impact-package-record-schema",
    )?;
    require_selector(&record.key.public_selector, "package.key.public_selector")?;
    require_digest(&record.package_identity_blake3, "package.package_identity_blake3")?;
    require_digest(&record.root_identity_blake3, "package.root_identity_blake3")?;
    require_digest(&record.recipe_identity_blake3, "package.recipe_identity_blake3")?;
    require_digest(&record.policy_identity_blake3, "package.policy_identity_blake3")?;
    validate_variant_shape(record)?;
    let mut normalized = record.clone();
    normalize_blockers(&mut normalized.blockers)?;
    validate_buildability(&normalized)?;
    let observed_identity = impact_package_record_identity_blake3(&normalized)?;
    seal_digest_field(&mut normalized.record_identity_blake3, &observed_identity, "package.record_identity_blake3")?;
    debug_assert!(normalized.blockers.windows(2).all(|pair| pair[0] < pair[1]));
    debug_assert_eq!(normalized.record_identity_blake3.len(), BLAKE3_HEX_LENGTH);
    Ok(normalized)
}

fn validate_variant_shape(record: &ImpactPackageRecord) -> Result<(), CoreFailure> {
    match (record.kind, &record.variant) {
        (ImpactPackageKind::Package, None) => Ok(()),
        (ImpactPackageKind::Variant, Some(variant)) => {
            require_digest(&variant.base_package_identity_blake3, "package.variant.base_package_identity_blake3")?;
            require_digest(&variant.base_root_identity_blake3, "package.variant.base_root_identity_blake3")?;
            require_text(&variant.variant_name, "package.variant.variant_name")?;
            require_digest(&variant.provenance_identity_blake3, "package.variant.provenance_identity_blake3")
        }
        (ImpactPackageKind::Package, Some(_)) => Err(failure(
            "impact-package-has-variant-facts",
            "package.variant",
            "an ordinary package record must not contain variant facts",
        )),
        (ImpactPackageKind::Variant, None) => Err(failure(
            "impact-variant-facts-missing",
            "package.variant",
            "a variant record must contain variant facts",
        )),
    }
}

fn normalize_blockers(blockers: &mut Vec<ImpactBlocker>) -> Result<(), CoreFailure> {
    blockers.sort();
    blockers.dedup();
    let mut by_code = BTreeMap::<&str, &str>::new();
    for blocker in blockers.iter() {
        require_text(&blocker.code, "package.blockers.code")?;
        require_digest(&blocker.detail_identity_blake3, "package.blockers.detail_identity_blake3")?;
        if let Some(existing) = by_code.insert(&blocker.code, &blocker.detail_identity_blake3)
            && existing != blocker.detail_identity_blake3
        {
            return Err(failure(
                "impact-blocker-code-conflict",
                &blocker.code,
                "one blocker code has more than one detail identity",
            ));
        }
    }
    Ok(())
}

fn validate_buildability(record: &ImpactPackageRecord) -> Result<(), CoreFailure> {
    if record.is_buildable && !record.blockers.is_empty() {
        return Err(failure(
            "impact-buildable-package-has-blockers",
            &record.key.public_selector,
            "a buildable package record must not contain blockers",
        ));
    }
    if !record.is_buildable && record.blockers.is_empty() {
        return Err(failure(
            "impact-blocked-package-has-no-blocker",
            &record.key.public_selector,
            "a non-buildable package record must contain a blocker",
        ));
    }
    Ok(())
}

fn normalize_observations(
    policy: &ImpactComparisonPolicy,
    observations: &[BuildObservation],
    packages: &BTreeMap<ImpactPackageKey, ImpactPackageRecord>,
) -> Result<BTreeMap<ImpactPackageKey, BuildObservation>, CoreFailure> {
    let observation_count_max = observations.len();
    let mut normalized_observations = BTreeMap::new();
    let mut identities = BTreeSet::new();
    for observation in observations {
        let package = packages.get(&observation.key).ok_or_else(|| {
            failure(
                "impact-observation-package-missing",
                &observation.key.public_selector,
                "the observation package is absent from the snapshot",
            )
        })?;
        let normalized = normalize_observation(observation, package)?;
        if let Some(existing) = normalized_observations.get(&normalized.key) {
            if existing != &normalized {
                return Err(failure(
                    "impact-observation-key-conflict",
                    &normalized.key.public_selector,
                    "more than one different observation claims the package key",
                ));
            }
            continue;
        }
        if !identities.insert(normalized.observation_identity_blake3.clone()) {
            return Err(failure(
                "impact-observation-identity-conflict",
                &normalized.observation_identity_blake3,
                "one observation identity claims more than one package key",
            ));
        }
        if normalized_observations.len() >= observation_count_max {
            return Err(failure(
                "impact-observation-normalization-bound-exceeded",
                "snapshot.observations",
                "normalized observations exceed the supplied observation count",
            ));
        }
        normalized_observations.insert(normalized.key.clone(), normalized);
    }
    require_count(
        normalized_observations.len(),
        policy.max_observations,
        "snapshot.observations",
        "impact-observation-limit-exceeded",
    )?;
    debug_assert!(normalized_observations.len() <= observation_count_max);
    debug_assert!(normalized_observations.keys().all(|key| packages.contains_key(key)));
    Ok(normalized_observations)
}

fn normalize_observation(
    observation: &BuildObservation,
    package: &ImpactPackageRecord,
) -> Result<BuildObservation, CoreFailure> {
    require_equal(
        &observation.schema,
        IMPACT_OBSERVATION_SCHEMA,
        "observation.schema",
        "unsupported-impact-observation-schema",
    )?;
    if observation.package_record_identity_blake3 != package.record_identity_blake3 {
        return Err(failure(
            "impact-observation-package-stale",
            &observation.key.public_selector,
            "the observation does not bind the current package record",
        ));
    }
    let mut normalized = observation.clone();
    normalize_reason_codes(&mut normalized.terminal_reason_codes, "observation.terminal_reason_codes")?;
    validate_observation_reason_shape(&normalized)?;
    validate_observation_authority(&normalized)?;
    let observed_identity = impact_observation_identity_blake3(&normalized)?;
    seal_digest_field(
        &mut normalized.observation_identity_blake3,
        &observed_identity,
        "observation.observation_identity_blake3",
    )?;
    debug_assert_eq!(normalized.observation_identity_blake3.len(), BLAKE3_HEX_LENGTH);
    debug_assert!(normalized.terminal_reason_codes.windows(2).all(|pair| pair[0] < pair[1]));
    Ok(normalized)
}

fn validate_observation_reason_shape(observation: &BuildObservation) -> Result<(), CoreFailure> {
    if observation.state == BuildObservationState::Success && !observation.terminal_reason_codes.is_empty() {
        return Err(failure(
            "impact-success-has-terminal-reason",
            "observation.terminal_reason_codes",
            "a successful observation must not contain terminal reason codes",
        ));
    }
    if observation.state != BuildObservationState::Success && observation.terminal_reason_codes.is_empty() {
        return Err(failure(
            "impact-terminal-reason-missing",
            "observation.terminal_reason_codes",
            "a non-success observation must contain a terminal reason code",
        ));
    }
    Ok(())
}

fn validate_observation_authority(observation: &BuildObservation) -> Result<(), CoreFailure> {
    debug_assert_eq!(observation.schema, IMPACT_OBSERVATION_SCHEMA);
    debug_assert_eq!(observation.package_record_identity_blake3.len(), BLAKE3_HEX_LENGTH);
    match &observation.authority {
        BuildObservationAuthority::AdmittedActionResult {
            runtime_report_schema,
            runtime_report_identity_blake3,
            action_ref,
            selected_result_ref,
            request_identity_blake3,
            policy_identity_blake3,
            platform_identity_blake3,
            signature_identity_blake3,
            output_set_identity_blake3,
            cas_identity_blake3,
        } => validate_action_result_authority(observation.state, ActionResultAuthorityRef {
            runtime_report_schema,
            runtime_report_identity_blake3,
            action_ref,
            selected_result_ref,
            request_identity_blake3,
            policy_identity_blake3,
            platform_identity_blake3,
            signature_identity_blake3,
            output_set_identity_blake3,
            cas_identity_blake3,
        }),
        BuildObservationAuthority::RealizationReceipt {
            receipt_schema,
            receipt_identity_blake3,
        } => validate_realization_authority(observation.state, receipt_schema, receipt_identity_blake3),
        BuildObservationAuthority::ExplicitStatus {
            status_schema,
            status_identity_blake3,
        } => validate_explicit_status_authority(observation.state, status_schema, status_identity_blake3),
    }
}

fn validate_action_result_authority(
    state: BuildObservationState,
    authority: ActionResultAuthorityRef<'_>,
) -> Result<(), CoreFailure> {
    if state != BuildObservationState::Success {
        return Err(failure(
            "impact-action-result-state-invalid",
            "observation.state",
            "an admitted action result can prove only a successful reused output",
        ));
    }
    require_equal(
        authority.runtime_report_schema,
        ACTION_RESULT_RUNTIME_REPORT_SCHEMA,
        "observation.authority.runtime_report_schema",
        "unsupported-action-result-runtime-report-schema",
    )?;
    require_digest(authority.runtime_report_identity_blake3, "observation.authority.runtime_report_identity_blake3")?;
    require_text(authority.action_ref, "observation.authority.action_ref")?;
    require_text(authority.selected_result_ref, "observation.authority.selected_result_ref")?;
    require_digest(authority.request_identity_blake3, "observation.authority.request_identity_blake3")?;
    require_digest(authority.policy_identity_blake3, "observation.authority.policy_identity_blake3")?;
    require_digest(authority.platform_identity_blake3, "observation.authority.platform_identity_blake3")?;
    require_digest(authority.signature_identity_blake3, "observation.authority.signature_identity_blake3")?;
    require_digest(authority.output_set_identity_blake3, "observation.authority.output_set_identity_blake3")?;
    require_digest(authority.cas_identity_blake3, "observation.authority.cas_identity_blake3")?;
    debug_assert_eq!(state, BuildObservationState::Success);
    debug_assert!(!authority.selected_result_ref.is_empty());
    Ok(())
}

fn validate_realization_authority(
    state: BuildObservationState,
    receipt_schema: &str,
    receipt_identity_blake3: impl AsRef<str>,
) -> Result<(), CoreFailure> {
    if !matches!(state, BuildObservationState::Success | BuildObservationState::Failure) {
        return Err(failure(
            "impact-realization-state-invalid",
            "observation.state",
            "a realization receipt can prove only a success or failure observation",
        ));
    }
    require_equal(
        receipt_schema,
        REALIZATION_RECEIPT_SCHEMA,
        "observation.authority.receipt_schema",
        "unsupported-realization-receipt-schema",
    )?;
    require_digest(receipt_identity_blake3.as_ref(), "observation.authority.receipt_identity_blake3")
}

fn validate_explicit_status_authority(
    state: BuildObservationState,
    status_schema: &str,
    status_identity_blake3: impl AsRef<str>,
) -> Result<(), CoreFailure> {
    if matches!(state, BuildObservationState::Success | BuildObservationState::Failure) {
        return Err(failure(
            "impact-explicit-status-state-invalid",
            "observation.state",
            "an explicit status must not claim a build success or failure",
        ));
    }
    require_equal(
        status_schema,
        EXPLICIT_STATUS_SCHEMA,
        "observation.authority.status_schema",
        "unsupported-explicit-status-schema",
    )?;
    require_digest(status_identity_blake3.as_ref(), "observation.authority.status_identity_blake3")
}

fn normalize_closures(
    policy: &ImpactComparisonPolicy,
    snapshot: &ImpactSnapshot,
    packages: &BTreeMap<ImpactPackageKey, ImpactPackageRecord>,
) -> Result<BTreeMap<ImpactPackageKey, PackageClosure>, CoreFailure> {
    let closure_count_max = snapshot.closures.len();
    let mut closures = BTreeMap::new();
    let mut identities = BTreeSet::new();
    for closure in &snapshot.closures {
        let package = packages.get(&closure.key).ok_or_else(|| {
            failure(
                "impact-closure-package-missing",
                &closure.key.public_selector,
                "the closure package is absent from the snapshot",
            )
        })?;
        let normalized = normalize_closure(closure, snapshot, package)?;
        if let Some(existing) = closures.get(&normalized.key) {
            if existing != &normalized {
                return Err(failure(
                    "impact-closure-key-conflict",
                    &normalized.key.public_selector,
                    "more than one different closure claims the package key",
                ));
            }
            continue;
        }
        if !identities.insert(normalized.closure_identity_blake3.clone()) {
            return Err(failure(
                "impact-closure-identity-conflict",
                &normalized.closure_identity_blake3,
                "one closure identity claims more than one package key",
            ));
        }
        if closures.len() >= closure_count_max {
            return Err(failure(
                "impact-closure-normalization-bound-exceeded",
                "snapshot.closures",
                "normalized closures exceed the supplied closure count",
            ));
        }
        closures.insert(normalized.key.clone(), normalized);
    }
    require_count(closures.len(), policy.max_packages, "snapshot.closures", "impact-closure-limit-exceeded")?;
    debug_assert!(closures.len() <= closure_count_max);
    debug_assert!(closures.keys().all(|key| packages.contains_key(key)));
    Ok(closures)
}

fn normalize_closure(
    closure: &PackageClosure,
    snapshot: &ImpactSnapshot,
    package: &ImpactPackageRecord,
) -> Result<PackageClosure, CoreFailure> {
    require_equal(&closure.schema, IMPACT_CLOSURE_SCHEMA, "closure.schema", "unsupported-impact-closure-schema")?;
    if closure.package_record_identity_blake3 != package.record_identity_blake3 {
        return Err(failure(
            "impact-closure-package-stale",
            &closure.key.public_selector,
            "the closure does not bind the current package record",
        ));
    }
    if closure.system != snapshot.system || closure.store_prefix != snapshot.store_prefix {
        return Err(failure(
            "impact-closure-snapshot-binding-mismatch",
            &closure.key.public_selector,
            "the closure system or store prefix differs from its snapshot",
        ));
    }
    require_digest(&closure.root_member_identity_blake3, "closure.root_member_identity_blake3")?;
    require_text(&closure.closure_semantics, "closure.closure_semantics")?;
    require_text(&closure.byte_semantics, "closure.byte_semantics")?;
    let mut normalized = closure.clone();
    normalize_closure_fields(&mut normalized);
    normalize_reason_codes(&mut normalized.incomplete_reason_codes, "closure.incomplete_reason_codes")?;
    validate_closure_completeness_declaration(&normalized)?;
    let observed_identity = impact_closure_identity_blake3(&normalized)?;
    seal_digest_field(&mut normalized.closure_identity_blake3, &observed_identity, "closure.closure_identity_blake3")?;
    debug_assert_eq!(normalized.closure_identity_blake3.len(), BLAKE3_HEX_LENGTH);
    debug_assert!(normalized.members.windows(2).all(|pair| pair[0] <= pair[1]));
    Ok(normalized)
}

fn normalize_closure_fields(closure: &mut PackageClosure) {
    for member in &mut closure.members {
        member.references.sort();
        member.references.dedup();
    }
    closure.members.sort();
    closure.incomplete_reason_codes.sort();
    closure.incomplete_reason_codes.dedup();
}

fn validate_closure_completeness_declaration(closure: &PackageClosure) -> Result<(), CoreFailure> {
    if closure.is_complete && !closure.incomplete_reason_codes.is_empty() {
        return Err(failure(
            "impact-complete-closure-has-incomplete-reasons",
            "closure.incomplete_reason_codes",
            "a complete closure must not contain incomplete reason codes",
        ));
    }
    if !closure.is_complete && closure.incomplete_reason_codes.is_empty() {
        return Err(failure(
            "impact-incomplete-closure-reason-missing",
            "closure.incomplete_reason_codes",
            "an incomplete closure must contain a reason code",
        ));
    }
    for member in &closure.members {
        require_digest(&member.member_identity_blake3, "closure.members.member_identity_blake3")?;
        require_text(&member.store_path, "closure.members.store_path")?;
        if let Some(identity) = &member.path_info_identity_blake3 {
            require_digest(identity, "closure.members.path_info_identity_blake3")?;
        }
        for reference in &member.references {
            require_digest(reference, "closure.members.references")?;
        }
    }
    debug_assert_eq!(closure.is_complete, closure.incomplete_reason_codes.is_empty());
    debug_assert!(closure.members.windows(2).all(|pair| pair[0] <= pair[1]));
    Ok(())
}

fn snapshot_identity_blake3(snapshot: &ImpactSnapshot) -> Result<String, CoreFailure> {
    digest_serializable(
        IMPACT_SNAPSHOT_IDENTITY_DOMAIN,
        &SnapshotIdentityPreimage {
            schema: &snapshot.schema,
            comparison_policy_identity_blake3: &snapshot.comparison_policy_identity_blake3,
            catalog_schema: &snapshot.catalog_schema,
            catalog_identity_blake3: &snapshot.catalog_identity_blake3,
            system: &snapshot.system,
            store_prefix: &snapshot.store_prefix,
            conversion_policy_identity_blake3: &snapshot.conversion_policy_identity_blake3,
            package_record_schema: &snapshot.package_record_schema,
            observation_schema: &snapshot.observation_schema,
            identity_domain: &snapshot.identity_domain,
            packages: &snapshot.packages,
            observations: &snapshot.observations,
            closures: &snapshot.closures,
        },
        "impact-snapshot-identity-serialization-failed",
    )
}

fn validate_snapshot_compatibility(
    policy: &ImpactComparisonPolicy,
    base: &ImpactSnapshot,
    head: &ImpactSnapshot,
) -> Result<(), CoreFailure> {
    let mut diagnostics = Vec::with_capacity(GLOBAL_COMPATIBILITY_FACT_COUNT);
    compare_global_fact(&base.system, &head.system, "impact-system-mismatch", "snapshot.system", &mut diagnostics);
    compare_global_fact(
        &base.store_prefix,
        &head.store_prefix,
        "impact-store-prefix-mismatch",
        "snapshot.store_prefix",
        &mut diagnostics,
    );
    compare_global_fact(
        &base.conversion_policy_identity_blake3,
        &head.conversion_policy_identity_blake3,
        "impact-conversion-policy-mismatch",
        "snapshot.conversion_policy_identity_blake3",
        &mut diagnostics,
    );
    compare_global_fact(
        &base.catalog_schema,
        &head.catalog_schema,
        "impact-catalog-schema-mismatch",
        "snapshot.catalog_schema",
        &mut diagnostics,
    );
    compare_global_fact(
        &base.package_record_schema,
        &head.package_record_schema,
        "impact-package-record-schema-mismatch",
        "snapshot.package_record_schema",
        &mut diagnostics,
    );
    compare_global_fact(
        &base.observation_schema,
        &head.observation_schema,
        "impact-observation-schema-mismatch",
        "snapshot.observation_schema",
        &mut diagnostics,
    );
    compare_global_fact(
        &base.identity_domain,
        &head.identity_domain,
        "impact-identity-domain-mismatch",
        "snapshot.identity_domain",
        &mut diagnostics,
    );
    compare_global_fact(
        &base.comparison_policy_identity_blake3,
        &head.comparison_policy_identity_blake3,
        "impact-policy-mismatch",
        "snapshot.comparison_policy_identity_blake3",
        &mut diagnostics,
    );
    let diagnostics_count = u32::try_from(diagnostics.len()).map_err(|_| {
        failure("impact-diagnostic-count-overflow", "comparison.diagnostics", "the diagnostic count exceeds u32")
    })?;
    if diagnostics_count > policy.max_diagnostics {
        return Err(failure(
            "impact-diagnostic-limit-exceeded",
            "comparison.diagnostics",
            "compatibility diagnostics exceed the comparison policy",
        ));
    }
    debug_assert_eq!(base.schema, head.schema);
    debug_assert!(diagnostics.len() <= GLOBAL_COMPATIBILITY_FACT_COUNT);
    if diagnostics.is_empty() {
        return Ok(());
    }
    Err(CoreFailure::from_diagnostics(diagnostics))
}

fn compare_global_fact(
    base: &str,
    head: impl AsRef<str>,
    code: impl AsRef<str>,
    path: impl AsRef<str>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if base != head.as_ref() {
        diagnostics.push(Diagnostic::new(code.as_ref(), path.as_ref(), "the base and head facts are incompatible"));
    }
}

fn compare_packages(base: &NormalizedSnapshot, head: &NormalizedSnapshot) -> Result<Vec<PackageImpact>, CoreFailure> {
    let keys = base.packages.keys().chain(head.packages.keys()).cloned().collect::<BTreeSet<_>>();
    let mut impacts = Vec::with_capacity(keys.len());
    for key in keys {
        let base_record = base.packages.get(&key);
        let head_record = head.packages.get(&key);
        let (catalog_disposition, changed_dimensions) = catalog_disposition(base_record, head_record);
        let build_transition = build_transition(base.observations.get(&key), head.observations.get(&key));
        let closure_delta = compare_closures(base.closures.get(&key), head.closures.get(&key));
        impacts.push(PackageImpact {
            key,
            catalog_disposition,
            changed_dimensions,
            base_record_identity_blake3: base_record.map(|record| record.record_identity_blake3.clone()),
            head_record_identity_blake3: head_record.map(|record| record.record_identity_blake3.clone()),
            build_transition,
            closure_delta,
        });
    }
    debug_assert!(impacts.windows(2).all(|pair| pair[0].key < pair[1].key));
    Ok(impacts)
}

fn catalog_disposition(
    base: Option<&ImpactPackageRecord>,
    head: Option<&ImpactPackageRecord>,
) -> (CatalogDisposition, Vec<ChangedDimension>) {
    let (Some(base), Some(head)) = (base, head) else {
        let disposition = if base.is_none() {
            CatalogDisposition::Added
        } else {
            CatalogDisposition::Removed
        };
        return (disposition, Vec::new());
    };
    let mut dimensions = Vec::with_capacity(CHANGED_DIMENSION_COUNT);
    let is_recipe_changed = base.recipe_identity_blake3 != head.recipe_identity_blake3
        || base.root_identity_blake3 != head.root_identity_blake3
        || base.package_identity_blake3 != head.package_identity_blake3;
    let is_policy_changed = base.policy_identity_blake3 != head.policy_identity_blake3;
    let is_blocker_changed = base.is_buildable != head.is_buildable || base.blockers != head.blockers;
    let is_variant_changed = base.kind != head.kind || base.variant != head.variant;
    if is_recipe_changed {
        dimensions.push(ChangedDimension::Recipe);
    }
    if is_policy_changed {
        dimensions.push(ChangedDimension::Policy);
    }
    if is_blocker_changed {
        dimensions.push(ChangedDimension::Blocker);
    }
    if is_variant_changed {
        dimensions.push(ChangedDimension::Variant);
    }
    let disposition = primary_catalog_disposition(&dimensions);
    debug_assert!(dimensions.len() <= CHANGED_DIMENSION_COUNT);
    debug_assert!(dimensions.windows(2).all(|pair| pair[0] < pair[1]));
    (disposition, dimensions)
}

fn primary_catalog_disposition(dimensions: &[ChangedDimension]) -> CatalogDisposition {
    if dimensions.contains(&ChangedDimension::Variant) {
        return CatalogDisposition::VariantChanged;
    }
    if dimensions.contains(&ChangedDimension::Policy) {
        return CatalogDisposition::PolicyChanged;
    }
    if dimensions.contains(&ChangedDimension::Blocker) {
        return CatalogDisposition::BlockerChanged;
    }
    if dimensions.contains(&ChangedDimension::Recipe) {
        return CatalogDisposition::RecipeChanged;
    }
    CatalogDisposition::Unchanged
}

fn build_transition(base: Option<&BuildObservation>, head: Option<&BuildObservation>) -> BuildOutcomeTransition {
    let disposition = match head.map(|observation| observation.state) {
        None => BuildTransitionDisposition::Missing,
        Some(BuildObservationState::Blocked) => BuildTransitionDisposition::Blocked,
        Some(BuildObservationState::NotAttempted) => BuildTransitionDisposition::NotAttempted,
        Some(BuildObservationState::Unavailable) => BuildTransitionDisposition::Unavailable,
        Some(BuildObservationState::Success) => {
            if base.is_some_and(|observation| observation.state == BuildObservationState::Success) {
                BuildTransitionDisposition::StillSuccessful
            } else {
                BuildTransitionDisposition::NewlySuccessful
            }
        }
        Some(BuildObservationState::Failure) => {
            if base.is_some_and(|observation| observation.state == BuildObservationState::Failure) {
                BuildTransitionDisposition::StillFailed
            } else {
                BuildTransitionDisposition::NewlyFailed
            }
        }
    };
    debug_assert!(base.zip(head).is_none_or(|(base, head)| base.key == head.key));
    debug_assert!(head.is_none_or(|observation| observation.state == BuildObservationState::Success
        || !observation.terminal_reason_codes.is_empty()));
    BuildOutcomeTransition {
        disposition,
        base_state: base.map(|observation| observation.state),
        head_state: head.map(|observation| observation.state),
        base_observation_identity_blake3: base.map(|observation| observation.observation_identity_blake3.clone()),
        head_observation_identity_blake3: head.map(|observation| observation.observation_identity_blake3.clone()),
        base_terminal_reason_codes: base.map_or_else(Vec::new, |observation| observation.terminal_reason_codes.clone()),
        head_terminal_reason_codes: head.map_or_else(Vec::new, |observation| observation.terminal_reason_codes.clone()),
    }
}

fn compare_closures(base: Option<&PackageClosure>, head: Option<&PackageClosure>) -> ClosureDelta {
    let mut reason_codes = closure_comparability_reasons(base, head);
    reason_codes.sort();
    reason_codes.dedup();
    if !reason_codes.is_empty() {
        return non_comparable_closure_delta(base, head, reason_codes);
    }
    let (Some(base), Some(head)) = (base, head) else {
        return non_comparable_closure_delta(base, head, vec!["closure-missing".into()]);
    };
    let base_members = base.members.iter().map(|member| member.member_identity_blake3.clone()).collect::<BTreeSet<_>>();
    let head_members = head.members.iter().map(|member| member.member_identity_blake3.clone()).collect::<BTreeSet<_>>();
    let added = head_members.difference(&base_members).cloned().collect::<Vec<_>>();
    let removed = base_members.difference(&head_members).cloned().collect::<Vec<_>>();
    let retained = retained_dependencies(base, head, &base_members, &head_members);
    let Some(base_bytes) = closure_logical_bytes(base) else {
        return non_comparable_closure_delta(base.into(), head.into(), vec!["base-logical-byte-overflow".into()]);
    };
    let Some(head_bytes) = closure_logical_bytes(head) else {
        return non_comparable_closure_delta(base.into(), head.into(), vec!["head-logical-byte-overflow".into()]);
    };
    let Some(member_count_delta) = member_count_delta(base_members.len(), head_members.len()) else {
        return non_comparable_closure_delta(base.into(), head.into(), vec!["member-count-delta-overflow".into()]);
    };
    debug_assert_eq!(base_members.len(), base.members.len());
    debug_assert_eq!(head_members.len(), head.members.len());
    ClosureDelta {
        is_comparable: true,
        reason_codes: Vec::new(),
        base_closure_identity_blake3: Some(base.closure_identity_blake3.clone()),
        head_closure_identity_blake3: Some(head.closure_identity_blake3.clone()),
        added_member_identities: Some(added),
        removed_member_identities: Some(removed),
        retained_dependency_identities: Some(retained),
        member_count_delta: Some(member_count_delta),
        logical_byte_delta: Some(logical_byte_delta(base_bytes, head_bytes)),
    }
}

fn closure_comparability_reasons(base: Option<&PackageClosure>, head: Option<&PackageClosure>) -> Vec<String> {
    let mut reasons = Vec::with_capacity(CLOSURE_REASON_CAPACITY);
    let Some(base) = base else {
        reasons.push("base-closure-missing".into());
        return reasons;
    };
    let Some(head) = head else {
        reasons.push("head-closure-missing".into());
        return reasons;
    };
    if base.system != head.system {
        reasons.push("closure-system-mismatch".into());
    }
    if base.store_prefix != head.store_prefix {
        reasons.push("closure-store-prefix-mismatch".into());
    }
    if base.closure_semantics != head.closure_semantics {
        reasons.push("closure-semantics-mismatch".into());
    }
    if base.byte_semantics != head.byte_semantics {
        reasons.push("closure-byte-semantics-mismatch".into());
    }
    append_closure_reasons("base", base, &mut reasons);
    append_closure_reasons("head", head, &mut reasons);
    debug_assert!(reasons.len() >= base.incomplete_reason_codes.len());
    debug_assert!(reasons.len() >= head.incomplete_reason_codes.len());
    reasons
}

fn append_closure_reasons(side: &str, closure: &PackageClosure, reasons: &mut Vec<String>) {
    let initial_reason_count = reasons.len();
    debug_assert!(!side.is_empty());
    debug_assert_eq!(closure.schema, IMPACT_CLOSURE_SCHEMA);
    if !closure.is_complete {
        reasons.push(format!("{side}-closure-incomplete"));
        reasons.extend(closure.incomplete_reason_codes.iter().map(|code| format!("{side}-{code}")));
    }
    let member_ids =
        closure.members.iter().map(|member| member.member_identity_blake3.as_str()).collect::<BTreeSet<_>>();
    let store_paths = closure.members.iter().map(|member| member.store_path.as_str()).collect::<BTreeSet<_>>();
    if member_ids.len() != closure.members.len() {
        reasons.push(format!("{side}-duplicate-member-identity"));
    }
    if store_paths.len() != closure.members.len() {
        reasons.push(format!("{side}-duplicate-store-path"));
    }
    if !member_ids.contains(closure.root_member_identity_blake3.as_str()) {
        reasons.push(format!("{side}-root-member-missing"));
    }
    for member in &closure.members {
        append_member_reasons(side, closure, member, &member_ids, reasons);
    }
    debug_assert!(reasons.len() >= initial_reason_count);
}

fn append_member_reasons(
    side: &str,
    closure: &PackageClosure,
    member: &ClosureMember,
    member_ids: &BTreeSet<&str>,
    reasons: &mut Vec<String>,
) {
    if member.path_info_identity_blake3.is_none() {
        push_reason_once(reasons, format!("{side}-path-info-missing"));
    }
    if member.logical_bytes.is_none() {
        push_reason_once(reasons, format!("{side}-logical-size-missing"));
    }
    if !path_is_in_store_prefix(&member.store_path, &closure.store_prefix) {
        push_reason_once(reasons, format!("{side}-store-path-outside-prefix"));
    }
    if member.references.iter().any(|reference| !member_ids.contains(reference.as_str())) {
        push_reason_once(reasons, format!("{side}-reference-unresolved"));
    }
}

fn push_reason_once(reasons: &mut Vec<String>, reason: String) {
    if !reasons.contains(&reason) {
        reasons.push(reason);
    }
}

fn retained_dependencies(
    base: &PackageClosure,
    head: &PackageClosure,
    base_members: &BTreeSet<String>,
    head_members: &BTreeSet<String>,
) -> Vec<String> {
    base_members
        .intersection(head_members)
        .filter(|identity| {
            identity.as_str() != base.root_member_identity_blake3
                && identity.as_str() != head.root_member_identity_blake3
        })
        .cloned()
        .collect()
}

fn closure_logical_bytes(closure: &PackageClosure) -> Option<u64> {
    closure.members.iter().try_fold(0u64, |total, member| total.checked_add(member.logical_bytes?))
}

fn member_count_delta(base_count: usize, head_count: impl Into<usize>) -> Option<i64> {
    let base = i64::try_from(base_count).ok()?;
    let head = i64::try_from(head_count.into()).ok()?;
    head.checked_sub(base)
}

fn logical_byte_delta(base_bytes: u64, head_bytes: impl Into<u64>) -> LogicalByteDelta {
    let head_bytes = head_bytes.into();
    if head_bytes > base_bytes {
        LogicalByteDelta {
            direction: DeltaDirection::Increase,
            magnitude_bytes: head_bytes.saturating_sub(base_bytes),
        }
    } else if base_bytes > head_bytes {
        LogicalByteDelta {
            direction: DeltaDirection::Decrease,
            magnitude_bytes: base_bytes.saturating_sub(head_bytes),
        }
    } else {
        LogicalByteDelta {
            direction: DeltaDirection::Unchanged,
            magnitude_bytes: 0,
        }
    }
}

fn non_comparable_closure_delta(
    base: Option<&PackageClosure>,
    head: Option<&PackageClosure>,
    mut reason_codes: Vec<String>,
) -> ClosureDelta {
    reason_codes.sort();
    reason_codes.dedup();
    ClosureDelta {
        is_comparable: false,
        reason_codes,
        base_closure_identity_blake3: base.map(|closure| closure.closure_identity_blake3.clone()),
        head_closure_identity_blake3: head.map(|closure| closure.closure_identity_blake3.clone()),
        added_member_identities: None,
        removed_member_identities: None,
        retained_dependency_identities: None,
        member_count_delta: None,
        logical_byte_delta: None,
    }
}

fn validate_policy_limits(policy: &ImpactComparisonPolicy) -> Result<(), CoreFailure> {
    require_equal(&policy.schema, IMPACT_POLICY_SCHEMA, "policy.schema", "unsupported-impact-policy-schema")?;
    require_limit(policy.max_packages, MIN_ITEMS, MAX_PACKAGES, "policy.max_packages")?;
    require_limit(policy.max_variants, MIN_ITEMS, MAX_VARIANTS, "policy.max_variants")?;
    require_limit(policy.max_closure_members, MIN_ITEMS, MAX_CLOSURE_MEMBERS, "policy.max_closure_members")?;
    require_limit(policy.max_dependency_edges, MIN_ITEMS, MAX_DEPENDENCY_EDGES, "policy.max_dependency_edges")?;
    require_limit(policy.max_observations, MIN_ITEMS, MAX_OBSERVATIONS, "policy.max_observations")?;
    require_limit(policy.max_diagnostics, MIN_DIAGNOSTICS, MAX_DIAGNOSTICS, "policy.max_diagnostics")?;
    if policy.max_report_bytes == 0 || policy.max_report_bytes > MAX_REPORT_BYTES {
        return Err(failure(
            "impact-policy-limit-invalid",
            "policy.max_report_bytes",
            "the report byte limit is outside the supported range",
        ));
    }
    Ok(())
}

fn require_limit(value: u32, minimum: impl Into<u32>, maximum: impl Into<u32>, path: &str) -> Result<(), CoreFailure> {
    if value < minimum.into() || value > maximum.into() {
        return Err(failure("impact-policy-limit-invalid", path, "the policy limit is outside the supported range"));
    }
    Ok(())
}

fn require_count(count: usize, limit: u32, path: impl AsRef<str>, code: impl AsRef<str>) -> Result<(), CoreFailure> {
    let path = path.as_ref();
    let code = code.as_ref();
    let count = u32::try_from(count).map_err(|_| failure(code, path, "the item count exceeds u32"))?;
    if count > limit {
        return Err(failure(code, path, "the item count exceeds the comparison policy"));
    }
    Ok(())
}

fn enforce_report_size(report: &MantlePackageImpactReport, max_bytes: u64) -> Result<(), CoreFailure> {
    let bytes = serde_json::to_vec(report).map_err(|error| {
        failure("impact-report-serialization-failed", "report", format!("the report cannot be serialized: {error}"))
    })?;
    let byte_count = u64::try_from(bytes.len())
        .map_err(|_| failure("impact-report-byte-count-overflow", "report", "the report byte count exceeds u64"))?;
    if byte_count > max_bytes {
        return Err(failure(
            "impact-report-size-limit-exceeded",
            "report",
            "the complete report exceeds the comparison policy",
        ));
    }
    Ok(())
}

fn report_non_claims() -> Vec<String> {
    vec![
        "This report does not prove package correctness.".into(),
        "This report does not prove unchanged package behavior.".into(),
        "This report does not prove reproducibility.".into(),
        "This report does not prove deployment safety.".into(),
        "This report does not prove release eligibility.".into(),
        "This report does not grant forge, approval, publication, or network authority.".into(),
    ]
}

fn normalize_reason_codes(codes: &mut Vec<String>, path: &str) -> Result<(), CoreFailure> {
    for code in codes.iter() {
        require_text(code, path)?;
    }
    codes.sort();
    codes.dedup();
    Ok(())
}

fn require_equal(
    observed: &str,
    expected: impl AsRef<str>,
    path: impl AsRef<str>,
    code: impl AsRef<str>,
) -> Result<(), CoreFailure> {
    if observed != expected.as_ref() {
        return Err(failure(code, path, "the schema or identity domain is not supported"));
    }
    Ok(())
}

fn require_text(value: &str, path: impl AsRef<str>) -> Result<(), CoreFailure> {
    let path = path.as_ref();
    if value.is_empty() || value.len() > MAX_TEXT_BYTES || value.chars().any(char::is_control) {
        return Err(failure(
            "impact-text-invalid",
            path,
            "the text must be nonempty, bounded, and free of control characters",
        ));
    }
    Ok(())
}

fn require_selector(value: &str, path: impl AsRef<str>) -> Result<(), CoreFailure> {
    let path = path.as_ref();
    require_text(value, path)?;
    if selector_has_edge_dot(value) || selector_has_path_component(value) {
        return Err(failure(
            "impact-selector-invalid",
            path,
            "the package selector contains a path or ambiguous component",
        ));
    }
    Ok(())
}

fn selector_has_edge_dot(value: &str) -> bool {
    value.starts_with('.') || value.ends_with('.')
}

fn selector_has_path_component(value: &str) -> bool {
    value.contains("..") || value.contains('/') || value.contains('\\')
}

fn require_store_prefix(value: &str, path: impl AsRef<str>) -> Result<(), CoreFailure> {
    let path = path.as_ref();
    require_text(value, path)?;
    if store_prefix_is_invalid(value) {
        return Err(failure(
            "impact-store-prefix-invalid",
            path,
            "the store prefix must be an absolute normalized path without a trailing slash",
        ));
    }
    Ok(())
}

fn store_prefix_is_invalid(value: &str) -> bool {
    if !value.starts_with('/') {
        return true;
    }
    if value.ends_with('/') {
        return true;
    }
    if value.contains("//") {
        return true;
    }
    value.contains("/../") || value.ends_with("/..")
}

fn path_is_in_store_prefix(path: &str, store_prefix: impl AsRef<str>) -> bool {
    path.strip_prefix(store_prefix.as_ref())
        .is_some_and(|suffix| suffix.starts_with('/') && suffix.len() >= MIN_STORE_PATH_SUFFIX_BYTES)
}

fn require_digest(value: &str, path: impl AsRef<str>) -> Result<(), CoreFailure> {
    if value.len() != BLAKE3_HEX_LENGTH
        || !value.bytes().all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
    {
        return Err(failure("impact-digest-invalid", path, "the value must be a lowercase BLAKE3 hexadecimal digest"));
    }
    Ok(())
}

fn is_placeholder_digest(value: &str) -> bool {
    value.is_empty() || (value.len() == BLAKE3_HEX_LENGTH && value.bytes().all(|byte| byte == ZERO_DIGEST_BYTE))
}

fn seal_digest_field(field: &mut String, observed: &str, path: impl AsRef<str>) -> Result<(), CoreFailure> {
    let path = path.as_ref();
    if is_placeholder_digest(field) {
        *field = observed.into();
        return Ok(());
    }
    require_digest(field, path)?;
    if field != observed {
        return Err(failure(
            "impact-identity-mismatch",
            path,
            "the supplied identity differs from the canonical fields",
        ));
    }
    Ok(())
}

fn digest_serializable<T: Serialize>(domain: &[u8], value: &T, code: &str) -> Result<String, CoreFailure> {
    let bytes = serde_json::to_vec(value)
        .map_err(|error| failure(code, "identity", format!("the identity preimage cannot be serialized: {error}")))?;
    let mut hasher = blake3::Hasher::new();
    hasher.update(domain);
    hasher.update(&[DOMAIN_SEPARATOR]);
    hasher.update(&bytes);
    Ok(hasher.finalize().to_hex().to_string())
}

fn failure(code: impl AsRef<str>, path: impl AsRef<str>, message: impl AsRef<str>) -> CoreFailure {
    CoreFailure::from_diagnostic(Diagnostic::new(code.as_ref(), path.as_ref(), message.as_ref()))
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_MAX_PACKAGES: u32 = 64;
    const TEST_MAX_VARIANTS: u32 = 32;
    const TEST_MAX_CLOSURE_MEMBERS: u32 = 256;
    const TEST_MAX_DEPENDENCY_EDGES: u32 = 512;
    const TEST_MAX_OBSERVATIONS: u32 = 64;
    const TEST_MAX_DIAGNOSTICS: u32 = 64;
    const TEST_MAX_REPORT_BYTES: u64 = 1_048_576;
    const BASE_DIGEST_BYTE: u8 = 1;
    const HEAD_DIGEST_BYTE: u8 = 2;
    const OTHER_DIGEST_BYTE: u8 = 3;
    const ROOT_LOGICAL_BYTES: u64 = 80;
    const DEPENDENCY_LOGICAL_BYTES: u64 = 20;
    const GROWN_DEPENDENCY_LOGICAL_BYTES: u64 = 30;
    const EXPECTED_MEMBER_COUNT: usize = 2;
    const EXPECTED_ADDED_MEMBER_COUNT: usize = 1;
    const EXPECTED_COMPATIBILITY_DIAGNOSTICS: usize = 2;
    const HEX_CHARS_PER_BYTE: usize = 2;

    fn digest(byte: u8) -> String {
        format!("{byte:02x}").repeat(BLAKE3_HEX_LENGTH / HEX_CHARS_PER_BYTE)
    }

    fn policy() -> ImpactComparisonPolicy {
        seal_impact_policy(&ImpactComparisonPolicy {
            schema: IMPACT_POLICY_SCHEMA.into(),
            policy_identity_blake3: String::new(),
            max_packages: TEST_MAX_PACKAGES,
            max_variants: TEST_MAX_VARIANTS,
            max_closure_members: TEST_MAX_CLOSURE_MEMBERS,
            max_dependency_edges: TEST_MAX_DEPENDENCY_EDGES,
            max_observations: TEST_MAX_OBSERVATIONS,
            max_diagnostics: TEST_MAX_DIAGNOSTICS,
            max_report_bytes: TEST_MAX_REPORT_BYTES,
        })
        .unwrap()
    }

    fn package(selector: &str, digest_byte: u8) -> ImpactPackageRecord {
        let mut record = ImpactPackageRecord {
            schema: IMPACT_PACKAGE_RECORD_SCHEMA.into(),
            record_identity_blake3: String::new(),
            key: ImpactPackageKey {
                public_selector: selector.into(),
            },
            kind: ImpactPackageKind::Package,
            package_identity_blake3: digest(digest_byte),
            root_identity_blake3: digest(digest_byte),
            recipe_identity_blake3: digest(digest_byte),
            policy_identity_blake3: digest(BASE_DIGEST_BYTE),
            is_buildable: true,
            blockers: Vec::new(),
            variant: None,
        };
        record.record_identity_blake3 = impact_package_record_identity_blake3(&record).unwrap();
        record
    }

    fn observation(record: &ImpactPackageRecord, state: BuildObservationState) -> BuildObservation {
        let terminal_reason_codes = if state == BuildObservationState::Success {
            Vec::new()
        } else {
            vec!["compiler-exit-nonzero".into()]
        };
        let authority = if matches!(state, BuildObservationState::Success | BuildObservationState::Failure) {
            BuildObservationAuthority::RealizationReceipt {
                receipt_schema: REALIZATION_RECEIPT_SCHEMA.into(),
                receipt_identity_blake3: digest(OTHER_DIGEST_BYTE),
            }
        } else {
            BuildObservationAuthority::ExplicitStatus {
                status_schema: EXPLICIT_STATUS_SCHEMA.into(),
                status_identity_blake3: digest(OTHER_DIGEST_BYTE),
            }
        };
        let mut observation = BuildObservation {
            schema: IMPACT_OBSERVATION_SCHEMA.into(),
            observation_identity_blake3: String::new(),
            key: record.key.clone(),
            package_record_identity_blake3: record.record_identity_blake3.clone(),
            state,
            terminal_reason_codes,
            authority,
        };
        observation.observation_identity_blake3 = impact_observation_identity_blake3(&observation).unwrap();
        observation
    }

    fn action_result_observation(record: &ImpactPackageRecord) -> BuildObservation {
        let mut observation = BuildObservation {
            schema: IMPACT_OBSERVATION_SCHEMA.into(),
            observation_identity_blake3: String::new(),
            key: record.key.clone(),
            package_record_identity_blake3: record.record_identity_blake3.clone(),
            state: BuildObservationState::Success,
            terminal_reason_codes: Vec::new(),
            authority: BuildObservationAuthority::AdmittedActionResult {
                runtime_report_schema: ACTION_RESULT_RUNTIME_REPORT_SCHEMA.into(),
                runtime_report_identity_blake3: digest(BASE_DIGEST_BYTE),
                action_ref: "mantle-action://blake3/fixture".into(),
                selected_result_ref: "mantle-action-result://blake3/fixture".into(),
                request_identity_blake3: digest(BASE_DIGEST_BYTE),
                policy_identity_blake3: digest(HEAD_DIGEST_BYTE),
                platform_identity_blake3: digest(OTHER_DIGEST_BYTE),
                signature_identity_blake3: digest(BASE_DIGEST_BYTE),
                output_set_identity_blake3: digest(HEAD_DIGEST_BYTE),
                cas_identity_blake3: digest(OTHER_DIGEST_BYTE),
            },
        };
        observation.observation_identity_blake3 = impact_observation_identity_blake3(&observation).unwrap();
        observation
    }

    fn closure(record: &ImpactPackageRecord, dependency_byte: u8, dependency_bytes: u64) -> PackageClosure {
        let root_identity = digest(BASE_DIGEST_BYTE);
        let dependency_identity = digest(dependency_byte);
        let mut closure = PackageClosure {
            schema: IMPACT_CLOSURE_SCHEMA.into(),
            closure_identity_blake3: String::new(),
            key: record.key.clone(),
            package_record_identity_blake3: record.record_identity_blake3.clone(),
            root_member_identity_blake3: root_identity.clone(),
            system: "x86_64-linux".into(),
            store_prefix: "/mantle/store".into(),
            closure_semantics: "path-info-transitive-v1".into(),
            byte_semantics: "nar-logical-bytes-v1".into(),
            is_complete: true,
            incomplete_reason_codes: Vec::new(),
            members: vec![
                ClosureMember {
                    member_identity_blake3: root_identity.clone(),
                    store_path: "/mantle/store/root".into(),
                    path_info_identity_blake3: Some(digest(BASE_DIGEST_BYTE)),
                    logical_bytes: Some(ROOT_LOGICAL_BYTES),
                    references: vec![dependency_identity.clone()],
                },
                ClosureMember {
                    member_identity_blake3: dependency_identity,
                    store_path: "/mantle/store/dependency".into(),
                    path_info_identity_blake3: Some(digest(dependency_byte)),
                    logical_bytes: Some(dependency_bytes),
                    references: Vec::new(),
                },
            ],
        };
        normalize_closure_fields(&mut closure);
        closure.closure_identity_blake3 = impact_closure_identity_blake3(&closure).unwrap();
        closure
    }

    fn snapshot(record: ImpactPackageRecord, observation: Option<BuildObservation>) -> ImpactSnapshot {
        let policy = policy();
        let closure = closure(&record, HEAD_DIGEST_BYTE, DEPENDENCY_LOGICAL_BYTES);
        ImpactSnapshot {
            schema: IMPACT_SNAPSHOT_SCHEMA.into(),
            snapshot_identity_blake3: String::new(),
            comparison_policy_identity_blake3: policy.policy_identity_blake3,
            catalog_schema: "mantlepkgs-domain-catalog-v1".into(),
            catalog_identity_blake3: digest(BASE_DIGEST_BYTE),
            system: "x86_64-linux".into(),
            store_prefix: "/mantle/store".into(),
            conversion_policy_identity_blake3: digest(BASE_DIGEST_BYTE),
            package_record_schema: IMPACT_PACKAGE_RECORD_SCHEMA.into(),
            observation_schema: IMPACT_OBSERVATION_SCHEMA.into(),
            identity_domain: IMPACT_IDENTITY_DOMAIN.into(),
            packages: vec![record],
            observations: observation.into_iter().collect(),
            closures: vec![closure],
        }
    }

    fn report(base: &ImpactSnapshot, head: &ImpactSnapshot) -> MantlePackageImpactReport {
        build_package_impact_report(ImpactComparisonInput {
            policy: &policy(),
            base,
            head,
        })
        .unwrap()
    }

    fn first_code(error: &CoreFailure) -> &str {
        &error.diagnostics[0].code
    }

    #[test]
    fn compatible_order_independent_snapshots_have_one_identity() {
        let first = package("alpha", BASE_DIGEST_BYTE);
        let second = package("beta", HEAD_DIGEST_BYTE);
        let mut base = snapshot(first.clone(), Some(observation(&first, BuildObservationState::Success)));
        base.packages.push(second.clone());
        base.observations.push(observation(&second, BuildObservationState::Success));
        base.closures.push(closure(&second, OTHER_DIGEST_BYTE, DEPENDENCY_LOGICAL_BYTES));
        let mut head = base.clone();
        head.packages.reverse();
        head.observations.reverse();
        head.closures.reverse();

        let first_report = report(&base, &head);
        let second_report = report(&head, &base);

        assert_eq!(first_report.report_identity_blake3, second_report.report_identity_blake3);
        assert!(
            first_report
                .package_impacts
                .iter()
                .all(|impact| impact.catalog_disposition == CatalogDisposition::Unchanged)
        );
    }

    #[test]
    fn global_incompatibilities_are_ordered_failures_without_report() {
        let record = package("alpha", BASE_DIGEST_BYTE);
        let base = snapshot(record.clone(), None);
        let mut head = snapshot(record, None);
        head.system = "aarch64-linux".into();
        head.store_prefix = "/other/store".into();
        head.closures[0].system = head.system.clone();
        head.closures[0].store_prefix = head.store_prefix.clone();
        head.closures[0].members[0].store_path = "/other/store/root".into();
        head.closures[0].members[1].store_path = "/other/store/dependency".into();
        head.closures[0].closure_identity_blake3 = impact_closure_identity_blake3(&head.closures[0]).unwrap();

        let error = build_package_impact_report(ImpactComparisonInput {
            policy: &policy(),
            base: &base,
            head: &head,
        })
        .unwrap_err();

        assert_eq!(error.diagnostics.len(), EXPECTED_COMPATIBILITY_DIAGNOSTICS);
        assert_eq!(error.diagnostics[0].code, "impact-store-prefix-mismatch");
        assert_eq!(error.diagnostics[1].code, "impact-system-mismatch");
    }

    #[test]
    fn conflicting_package_keys_are_rejected() {
        let first = package("alpha", BASE_DIGEST_BYTE);
        let mut conflicting = first.clone();
        conflicting.recipe_identity_blake3 = digest(HEAD_DIGEST_BYTE);
        conflicting.record_identity_blake3 = impact_package_record_identity_blake3(&conflicting).unwrap();
        let mut base = snapshot(first, None);
        base.packages.push(conflicting);

        let error = seal_impact_snapshot(&policy(), &base).unwrap_err();

        assert_eq!(first_code(&error), "impact-package-key-conflict");
    }

    #[test]
    fn combined_recipe_and_policy_change_preserves_both_dimensions() {
        let base_record = package("alpha", BASE_DIGEST_BYTE);
        let mut head_record = package("alpha", HEAD_DIGEST_BYTE);
        head_record.policy_identity_blake3 = digest(OTHER_DIGEST_BYTE);
        head_record.record_identity_blake3 = impact_package_record_identity_blake3(&head_record).unwrap();
        let base = snapshot(base_record, None);
        let head = snapshot(head_record, None);

        let impact = report(&base, &head).package_impacts.remove(0);

        assert_eq!(impact.catalog_disposition, CatalogDisposition::PolicyChanged);
        assert_eq!(impact.changed_dimensions, vec![ChangedDimension::Recipe, ChangedDimension::Policy]);
    }

    #[test]
    fn missing_head_observation_is_not_a_failure() {
        let record = package("alpha", BASE_DIGEST_BYTE);
        let base = snapshot(record.clone(), Some(observation(&record, BuildObservationState::Success)));
        let head = snapshot(record, None);

        let transition = report(&base, &head).package_impacts.remove(0).build_transition;

        assert_eq!(transition.disposition, BuildTransitionDisposition::Missing);
        assert_eq!(transition.base_state, Some(BuildObservationState::Success));
        assert_eq!(transition.head_state, None);
    }

    #[test]
    fn admitted_success_to_failed_receipt_is_newly_failed() {
        let record = package("alpha", BASE_DIGEST_BYTE);
        let base = snapshot(record.clone(), Some(observation(&record, BuildObservationState::Success)));
        let head = snapshot(record.clone(), Some(observation(&record, BuildObservationState::Failure)));

        let transition = report(&base, &head).package_impacts.remove(0).build_transition;

        assert_eq!(transition.disposition, BuildTransitionDisposition::NewlyFailed);
        assert_eq!(transition.head_terminal_reason_codes, vec!["compiler-exit-nonzero"]);
        assert!(transition.base_observation_identity_blake3.is_some());
        assert!(transition.head_observation_identity_blake3.is_some());
    }

    #[test]
    fn stale_observation_binding_is_rejected() {
        let record = package("alpha", BASE_DIGEST_BYTE);
        let mut stale = observation(&record, BuildObservationState::Success);
        stale.package_record_identity_blake3 = digest(HEAD_DIGEST_BYTE);
        stale.observation_identity_blake3 = impact_observation_identity_blake3(&stale).unwrap();
        let snapshot = snapshot(record, Some(stale));

        let error = seal_impact_snapshot(&policy(), &snapshot).unwrap_err();

        assert_eq!(first_code(&error), "impact-observation-package-stale");
    }

    #[test]
    fn explicit_status_cannot_invent_success() {
        let record = package("alpha", BASE_DIGEST_BYTE);
        let mut invalid = observation(&record, BuildObservationState::Success);
        invalid.authority = BuildObservationAuthority::ExplicitStatus {
            status_schema: EXPLICIT_STATUS_SCHEMA.into(),
            status_identity_blake3: digest(OTHER_DIGEST_BYTE),
        };
        invalid.observation_identity_blake3 = impact_observation_identity_blake3(&invalid).unwrap();
        let snapshot = snapshot(record, Some(invalid));

        let error = seal_impact_snapshot(&policy(), &snapshot).unwrap_err();

        assert_eq!(first_code(&error), "impact-explicit-status-state-invalid");
    }

    #[test]
    fn complete_closure_reports_members_bytes_and_retained_dependencies() {
        let record = package("alpha", BASE_DIGEST_BYTE);
        let base = snapshot(record.clone(), None);
        let mut head = snapshot(record, None);
        let head_record = &head.packages[0];
        let mut grown = closure(head_record, OTHER_DIGEST_BYTE, GROWN_DEPENDENCY_LOGICAL_BYTES);
        grown.members.push(ClosureMember {
            member_identity_blake3: digest(HEAD_DIGEST_BYTE),
            store_path: "/mantle/store/new-dependency".into(),
            path_info_identity_blake3: Some(digest(HEAD_DIGEST_BYTE)),
            logical_bytes: Some(DEPENDENCY_LOGICAL_BYTES),
            references: Vec::new(),
        });
        normalize_closure_fields(&mut grown);
        grown.closure_identity_blake3 = impact_closure_identity_blake3(&grown).unwrap();
        head.closures = vec![grown];

        let delta = report(&base, &head).package_impacts.remove(0).closure_delta;

        assert!(delta.is_comparable);
        assert_eq!(delta.added_member_identities.as_ref().unwrap().len(), EXPECTED_ADDED_MEMBER_COUNT);
        assert_eq!(delta.member_count_delta, Some(1));
        assert_eq!(delta.logical_byte_delta.unwrap().direction, DeltaDirection::Increase);
    }

    #[test]
    fn incomplete_closure_has_reasons_and_no_numeric_claims() {
        let record = package("alpha", BASE_DIGEST_BYTE);
        let base = snapshot(record.clone(), None);
        let mut head = snapshot(record, None);
        head.closures[0].is_complete = false;
        head.closures[0].incomplete_reason_codes = vec!["path-info-unavailable".into()];
        head.closures[0].members[0].path_info_identity_blake3 = None;
        head.closures[0].closure_identity_blake3 = impact_closure_identity_blake3(&head.closures[0]).unwrap();

        let delta = report(&base, &head).package_impacts.remove(0).closure_delta;

        assert!(!delta.is_comparable);
        assert!(delta.reason_codes.contains(&"head-closure-incomplete".into()));
        assert!(delta.reason_codes.contains(&"head-path-info-missing".into()));
        assert!(delta.added_member_identities.is_none());
        assert!(delta.member_count_delta.is_none());
        assert!(delta.logical_byte_delta.is_none());
    }

    #[test]
    fn unresolved_reference_prevents_numeric_closure_claims() {
        let record = package("alpha", BASE_DIGEST_BYTE);
        let base = snapshot(record.clone(), None);
        let mut head = snapshot(record, None);
        head.closures[0].members[0].references = vec![digest(OTHER_DIGEST_BYTE)];
        head.closures[0].closure_identity_blake3 = impact_closure_identity_blake3(&head.closures[0]).unwrap();

        let delta = report(&base, &head).package_impacts.remove(0).closure_delta;

        assert!(!delta.is_comparable);
        assert!(delta.reason_codes.contains(&"head-reference-unresolved".into()));
        assert!(delta.removed_member_identities.is_none());
    }

    #[test]
    fn closure_byte_overflow_prevents_numeric_claims() {
        let record = package("alpha", BASE_DIGEST_BYTE);
        let base = snapshot(record.clone(), None);
        let mut head = snapshot(record, None);
        head.closures[0].members[0].logical_bytes = Some(u64::MAX);
        head.closures[0].members[1].logical_bytes = Some(1);
        head.closures[0].closure_identity_blake3 = impact_closure_identity_blake3(&head.closures[0]).unwrap();

        let delta = report(&base, &head).package_impacts.remove(0).closure_delta;

        assert!(!delta.is_comparable);
        assert_eq!(delta.reason_codes, vec!["head-logical-byte-overflow"]);
        assert!(delta.logical_byte_delta.is_none());
    }

    #[test]
    fn exceeded_package_limit_fails_before_comparison() {
        let record = package("alpha", BASE_DIGEST_BYTE);
        let snapshot = snapshot(record, None);
        let mut limited_policy = policy();
        limited_policy.max_packages = 0;
        limited_policy.policy_identity_blake3.clear();

        let error = build_package_impact_report(ImpactComparisonInput {
            policy: &limited_policy,
            base: &snapshot,
            head: &snapshot,
        })
        .unwrap_err();

        assert_eq!(first_code(&error), "impact-policy-limit-invalid");
    }

    #[test]
    fn catalog_dispositions_cover_added_removed_recipe_blocker_and_variant_changes() {
        let base_record = package("alpha", BASE_DIGEST_BYTE);
        let mut empty = snapshot(base_record.clone(), None);
        empty.packages.clear();
        empty.closures.clear();
        let ordinary = snapshot(base_record.clone(), None);
        let added = report(&empty, &ordinary).package_impacts.remove(0).catalog_disposition;
        let removed = report(&ordinary, &empty).package_impacts.remove(0).catalog_disposition;

        let recipe_record = package("alpha", HEAD_DIGEST_BYTE);
        let recipe = report(&ordinary, &snapshot(recipe_record, None)).package_impacts.remove(0).catalog_disposition;

        let mut blocked_record = base_record.clone();
        blocked_record.is_buildable = false;
        blocked_record.blockers = vec![ImpactBlocker {
            code: "unsupported-builder".into(),
            detail_identity_blake3: digest(OTHER_DIGEST_BYTE),
        }];
        blocked_record.record_identity_blake3 = impact_package_record_identity_blake3(&blocked_record).unwrap();
        let blocker = report(&ordinary, &snapshot(blocked_record, None)).package_impacts.remove(0).catalog_disposition;

        let mut variant_record = base_record;
        variant_record.kind = ImpactPackageKind::Variant;
        variant_record.variant = Some(ImpactVariantFacts {
            base_package_identity_blake3: digest(BASE_DIGEST_BYTE),
            base_root_identity_blake3: digest(BASE_DIGEST_BYTE),
            variant_name: "debug".into(),
            provenance_identity_blake3: digest(OTHER_DIGEST_BYTE),
        });
        variant_record.record_identity_blake3 = impact_package_record_identity_blake3(&variant_record).unwrap();
        let variant = report(&ordinary, &snapshot(variant_record, None)).package_impacts.remove(0).catalog_disposition;

        assert_eq!(added, CatalogDisposition::Added);
        assert_eq!(removed, CatalogDisposition::Removed);
        assert_eq!(recipe, CatalogDisposition::RecipeChanged);
        assert_eq!(blocker, CatalogDisposition::BlockerChanged);
        assert_eq!(variant, CatalogDisposition::VariantChanged);
    }

    #[test]
    fn explicit_non_build_states_remain_distinct() {
        let record = package("alpha", BASE_DIGEST_BYTE);
        let base = snapshot(record.clone(), Some(observation(&record, BuildObservationState::Success)));
        let states = [
            (BuildObservationState::Blocked, BuildTransitionDisposition::Blocked),
            (BuildObservationState::NotAttempted, BuildTransitionDisposition::NotAttempted),
            (BuildObservationState::Unavailable, BuildTransitionDisposition::Unavailable),
        ];

        for (state, expected) in states {
            let head = snapshot(record.clone(), Some(observation(&record, state)));
            let transition = report(&base, &head).package_impacts.remove(0).build_transition;
            assert_eq!(transition.disposition, expected);
            assert_eq!(transition.head_state, Some(state));
        }
    }

    #[test]
    fn admitted_action_result_requires_all_current_authority_bindings() {
        let record = package("alpha", BASE_DIGEST_BYTE);
        let admitted = action_result_observation(&record);
        let valid = snapshot(record.clone(), Some(admitted.clone()));
        seal_impact_snapshot(&policy(), &valid).unwrap();
        let mut stale = admitted;
        let BuildObservationAuthority::AdmittedActionResult {
            cas_identity_blake3, ..
        } = &mut stale.authority
        else {
            panic!("fixture must use action-result authority");
        };
        *cas_identity_blake3 = "missing-cas-admission".into();
        stale.observation_identity_blake3 = impact_observation_identity_blake3(&stale).unwrap();
        let invalid = snapshot(record, Some(stale));

        let error = seal_impact_snapshot(&policy(), &invalid).unwrap_err();

        assert_eq!(first_code(&error), "impact-digest-invalid");
    }

    #[test]
    fn closure_semantic_mismatch_emits_no_numeric_delta() {
        let record = package("alpha", BASE_DIGEST_BYTE);
        let base = snapshot(record.clone(), None);
        let mut head = snapshot(record, None);
        head.closures[0].byte_semantics = "allocated-block-bytes-v1".into();
        head.closures[0].closure_identity_blake3 = impact_closure_identity_blake3(&head.closures[0]).unwrap();

        let delta = report(&base, &head).package_impacts.remove(0).closure_delta;

        assert!(!delta.is_comparable);
        assert_eq!(delta.reason_codes, vec!["closure-byte-semantics-mismatch"]);
        assert!(delta.member_count_delta.is_none());
        assert!(delta.logical_byte_delta.is_none());
    }

    #[test]
    fn missing_logical_size_and_duplicate_member_prevent_numeric_deltas() {
        let record = package("alpha", BASE_DIGEST_BYTE);
        let base = snapshot(record.clone(), None);
        let mut head = snapshot(record, None);
        head.closures[0].members[0].logical_bytes = None;
        let duplicate = head.closures[0].members[0].clone();
        head.closures[0].members.push(duplicate);
        normalize_closure_fields(&mut head.closures[0]);
        head.closures[0].closure_identity_blake3 = impact_closure_identity_blake3(&head.closures[0]).unwrap();

        let delta = report(&base, &head).package_impacts.remove(0).closure_delta;

        assert!(!delta.is_comparable);
        assert!(delta.reason_codes.contains(&"head-logical-size-missing".into()));
        assert!(delta.reason_codes.contains(&"head-duplicate-member-identity".into()));
        assert!(delta.added_member_identities.is_none());
    }

    #[test]
    fn dependency_edge_limit_fails_before_closure_work() {
        let record = package("alpha", BASE_DIGEST_BYTE);
        let mut snapshot = snapshot(record, None);
        snapshot.closures[0].members[0].references.push(digest(BASE_DIGEST_BYTE));
        snapshot.closures[0].closure_identity_blake3 = impact_closure_identity_blake3(&snapshot.closures[0]).unwrap();
        snapshot.comparison_policy_identity_blake3.clear();
        let mut limited_policy = policy();
        limited_policy.max_dependency_edges = MIN_ITEMS;
        limited_policy.policy_identity_blake3.clear();

        let error = seal_impact_snapshot(&limited_policy, &snapshot).unwrap_err();

        assert_eq!(first_code(&error), "impact-dependency-edge-limit-exceeded");
    }

    #[test]
    fn complete_report_is_not_truncated_to_fit_byte_limit() {
        let record = package("alpha", BASE_DIGEST_BYTE);
        let mut base = snapshot(record.clone(), None);
        let mut head = snapshot(record, None);
        base.comparison_policy_identity_blake3.clear();
        head.comparison_policy_identity_blake3.clear();
        let mut small_policy = policy();
        small_policy.max_report_bytes = 1;
        small_policy.policy_identity_blake3.clear();

        let error = build_package_impact_report(ImpactComparisonInput {
            policy: &small_policy,
            base: &base,
            head: &head,
        })
        .unwrap_err();

        assert_eq!(first_code(&error), "impact-report-size-limit-exceeded");
    }

    #[test]
    fn report_identity_changes_after_semantic_mutation() {
        let record = package("alpha", BASE_DIGEST_BYTE);
        let base = snapshot(record.clone(), None);
        let mut head = snapshot(record, None);
        let first = report(&base, &head);
        head.catalog_identity_blake3 = digest(HEAD_DIGEST_BYTE);
        let second = report(&base, &head);

        assert_ne!(first.report_identity_blake3, second.report_identity_blake3);
        assert_eq!(first.package_impacts.len(), 1);
        assert_eq!(first.package_impacts[0].closure_delta.added_member_identities.as_ref().unwrap().len(), 0);
        assert_eq!(first.package_impacts[0].closure_delta.removed_member_identities.as_ref().unwrap().len(), 0);
        assert_eq!(first.package_impacts[0].closure_delta.retained_dependency_identities.as_ref().unwrap().len(), 1);
        assert_eq!(base.closures[0].members.len(), EXPECTED_MEMBER_COUNT);
    }
}
