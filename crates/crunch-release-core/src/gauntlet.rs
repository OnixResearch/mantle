use alloc::collections::BTreeMap;
use alloc::collections::BTreeSet;
use alloc::format;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec;
use alloc::vec::Vec;

use serde::Deserialize;
use serde::Serialize;

use crate::ReleaseEvidenceError;
use crate::manifest::u32_count;
use crate::manifest::validate_blake3_hex;
use crate::manifest::validation_error;

pub const ADVERSARIAL_HERMETICITY_GAUNTLET_REPORT_SCHEMA: &str = "mantle-adversarial-hermeticity-gauntlet-report-v1";
pub const BOOTSTRAP_PRESSURE_GAUNTLET_REPORT_SCHEMA: &str = "mantle-bootstrap-pressure-gauntlet-report-v1";
pub const SUBSTITUTION_CACHE_ATTACK_GAUNTLET_REPORT_SCHEMA: &str =
    "mantle-substitution-cache-attack-gauntlet-report-v1";
pub const NIX_MANTLE_COMPARISON_CORPUS_REPORT_SCHEMA: &str = "mantle-nix-mantle-comparison-corpus-report-v1";
pub const RELEASE_REPEATABILITY_MATRIX_REPORT_SCHEMA: &str = "mantle-release-repeatability-matrix-report-v1";
pub const CONTINUOUS_REPRODUCIBILITY_GAUNTLET_REPORT_SCHEMA: &str =
    "mantle-continuous-reproducibility-gauntlet-report-v1";

const MAX_GAUNTLET_CELL_COUNT: u32 = 256;
const MAX_GAUNTLET_AXIS_COUNT: u32 = 64;
const MAX_GAUNTLET_STRING_SET_COUNT: u32 = 512;
const MAX_GAUNTLET_BLOCKER_COUNT: u32 = 512;
const MAX_GAUNTLET_REPORT_DIGEST_COUNT: u32 = 512;
const MAX_GAUNTLET_TRACK_COUNT: u32 = 64;
const MAX_GAUNTLET_RUN_COUNT: u32 = 256;
const MAX_GAUNTLET_STRING_BYTES_COUNT: u32 = 2048;
const ZERO_COUNT: u32 = 0;
const MIN_REPEATABILITY_RUN_COUNT: u32 = 1;

const NON_CLAIM_STRICT_REPRODUCIBILITY_BLOCKED: &str = "strict-reproducibility-blocked";
const NON_CLAIM_FULL_SOURCE_BOOTSTRAP: &str = "not-full-source-bootstrap";
const NON_CLAIM_COMPILER_CORRECTNESS: &str = "not-compiler-correctness";
const NON_CLAIM_GLOBAL_SUPERIORITY: &str = "not-global-superiority";
const NON_CLAIM_STALE_EVIDENCE: &str = "stale-evidence-not-promoted";
const NON_CLAIM_UNTESTED_AXIS_PREFIX: &str = "untested-axis";

const PROFILE_NO_HOST_TOOLS: &str = "no-host-tools";
const PROFILE_REDUCED_SEED: &str = "reduced-seed";
const PROFILE_FULL_SOURCE_ROOT_ATTEMPT: &str = "full-source-root-attempt";
const PROFILE_SOURCE_BUILT_PROVIDER: &str = "source-built-provider";

const DIGEST_FIELD_SOURCE: &str = "source_digest_blake3";
const DIGEST_FIELD_POLICY: &str = "policy_digest_blake3";
const DIGEST_FIELD_UNIVERSE: &str = "universe_digest_blake3";
const DIGEST_FIELD_TOOLCHAIN: &str = "toolchain_digest_blake3";
const DIGEST_FIELD_WITNESS_SET: &str = "witness_set_digest_blake3";
const DIGEST_FIELD_REPORT: &str = "report_digest_blake3";

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum GauntletMode {
    Strict,
    Practical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum GauntletVerdict {
    Accepted,
    Matched,
    Rejected,
    Degraded,
    Blocked,
    Unsupported,
    Stale,
    Flaky,
    Mismatched,
    Missing,
}

impl GauntletVerdict {
    pub fn is_blocking(self) -> bool {
        matches!(
            self,
            Self::Blocked | Self::Unsupported | Self::Stale | Self::Flaky | Self::Mismatched | Self::Missing
        )
    }

    pub fn is_strict_evidence(self) -> bool {
        matches!(self, Self::Accepted | Self::Matched | Self::Rejected)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GauntletBlocker {
    pub evidence_class: String,
    pub message: String,
    pub next_action: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub axis: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expected_digest_blake3: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub observed_digest_blake3: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HermeticityGauntletProfile {
    #[serde(default)]
    pub host_tool_axes: Vec<String>,
    #[serde(default)]
    pub environment_axes: Vec<String>,
    #[serde(default)]
    pub network_axes: Vec<String>,
    #[serde(default)]
    pub timestamp_axes: Vec<String>,
    #[serde(default)]
    pub locale_axes: Vec<String>,
    #[serde(default)]
    pub umask_axes: Vec<String>,
    #[serde(default)]
    pub temp_path_axes: Vec<String>,
    #[serde(default)]
    pub store_path_axes: Vec<String>,
    #[serde(default)]
    pub randomness_axes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HermeticityViolation {
    pub class: String,
    pub detail: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub axis: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HermeticityCellEvidence {
    pub cell_id: String,
    pub mode: GauntletMode,
    #[serde(default)]
    pub observed_violations: Vec<HermeticityViolation>,
    #[serde(default)]
    pub audit_events: Vec<String>,
    #[serde(default)]
    pub unsupported_axes: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output_digest_blake3: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HermeticityCellReport {
    pub cell_id: String,
    pub mode: GauntletMode,
    pub verdict: GauntletVerdict,
    #[serde(default)]
    pub output_digest_blake3: Option<String>,
    pub blockers: Vec<GauntletBlocker>,
    pub non_claims: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AdversarialHermeticityGauntletReport {
    pub schema: String,
    pub run_id: String,
    pub source_digest_blake3: String,
    pub profile: HermeticityGauntletProfile,
    pub cells: Vec<HermeticityCellReport>,
    pub strict_evidence_eligible: bool,
    pub blockers: Vec<GauntletBlocker>,
    pub non_claims: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProtectedExecObservation {
    pub path: String,
    pub executable_class: String,
    pub declared: bool,
    pub digest_blake3: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BootstrapPressureCellEvidence {
    pub cell_id: String,
    pub profile: String,
    pub fixed_point: bool,
    pub host_tool_policy: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub seed_inventory_digest_blake3: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub protected_exec_audit_digest_blake3: Option<String>,
    #[serde(default)]
    pub stage_output_digest_set_blake3: Vec<String>,
    #[serde(default)]
    pub observed_execs: Vec<ProtectedExecObservation>,
    #[serde(default)]
    pub remaining_trusted_root: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unsupported_reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BootstrapPressureCellReport {
    pub cell_id: String,
    pub profile: String,
    pub verdict: GauntletVerdict,
    pub fixed_point: bool,
    pub host_tool_policy: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub seed_inventory_digest_blake3: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub protected_exec_audit_digest_blake3: Option<String>,
    pub stage_output_digest_set_blake3: Vec<String>,
    pub remaining_trusted_root: Vec<String>,
    pub observed_execs: Vec<ProtectedExecObservation>,
    pub blockers: Vec<GauntletBlocker>,
    pub non_claims: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BootstrapPressureGauntletReport {
    pub schema: String,
    pub run_id: String,
    pub source_digest_blake3: String,
    pub cells: Vec<BootstrapPressureCellReport>,
    pub blockers: Vec<GauntletBlocker>,
    pub non_claims: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SubstitutionAttackCaseEvidence {
    pub case_id: String,
    pub strict_mode: bool,
    pub accepted: bool,
    pub closure_complete: bool,
    pub artifact_attestation_fresh: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trusted_key_material: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fallback_mode: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expected_digest_blake3: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub observed_digest_blake3: Option<String>,
    #[serde(default)]
    pub failed_trust_edges: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SubstitutionAttackCaseReport {
    pub case_id: String,
    pub verdict: GauntletVerdict,
    pub strict_mode: bool,
    pub accepted: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trusted_key_material: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fallback_mode: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expected_digest_blake3: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub observed_digest_blake3: Option<String>,
    pub failed_trust_edges: Vec<String>,
    pub blockers: Vec<GauntletBlocker>,
    pub non_claims: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SubstitutionCacheAttackGauntletReport {
    pub schema: String,
    pub run_id: String,
    pub policy_digest_blake3: String,
    pub cases: Vec<SubstitutionAttackCaseReport>,
    pub strict_cache_evidence_eligible: bool,
    pub blockers: Vec<GauntletBlocker>,
    pub non_claims: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NixMantleComparisonCaseEvidence {
    pub case_id: String,
    pub equivalence_declared: bool,
    pub build_recipe_identity: String,
    pub normalization_policy: String,
    #[serde(default)]
    pub source_refs: Vec<String>,
    #[serde(default)]
    pub toolchain_refs: Vec<String>,
    #[serde(default)]
    pub dependency_refs: Vec<String>,
    #[serde(default)]
    pub output_surfaces: Vec<String>,
    #[serde(default)]
    pub allowed_differences: Vec<String>,
    #[serde(default)]
    pub nix_digest_set_blake3: Vec<String>,
    #[serde(default)]
    pub mantle_digest_set_blake3: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unsupported_reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NixMantleComparisonCaseReport {
    pub case_id: String,
    pub verdict: GauntletVerdict,
    pub build_recipe_identity: String,
    pub normalization_policy: String,
    pub source_refs: Vec<String>,
    pub toolchain_refs: Vec<String>,
    pub dependency_refs: Vec<String>,
    pub output_surfaces: Vec<String>,
    pub allowed_differences: Vec<String>,
    pub nix_digest_set_blake3: Vec<String>,
    pub mantle_digest_set_blake3: Vec<String>,
    pub blockers: Vec<GauntletBlocker>,
    pub non_claims: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NixMantleComparisonCorpusReport {
    pub schema: String,
    pub run_id: String,
    pub policy_digest_blake3: String,
    pub cases: Vec<NixMantleComparisonCaseReport>,
    pub blockers: Vec<GauntletBlocker>,
    pub non_claims: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RepeatabilityMatrixProfile {
    pub release_id: String,
    pub matrix_profile_digest_blake3: String,
    pub run_count: u32,
    #[serde(default)]
    pub artifact_surfaces: Vec<String>,
    #[serde(default)]
    pub expected_output_digest_set_blake3: Vec<String>,
    #[serde(default)]
    pub cache_modes: Vec<String>,
    #[serde(default)]
    pub store_isolation_modes: Vec<String>,
    #[serde(default)]
    pub environment_controls: Vec<String>,
    #[serde(default)]
    pub temp_root_controls: Vec<String>,
    #[serde(default)]
    pub user_controls: Vec<String>,
    #[serde(default)]
    pub host_classes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RepeatabilityMatrixCellEvidence {
    pub cell_id: String,
    pub cache_mode: String,
    pub store_isolation_mode: String,
    pub fresh_store: bool,
    pub explicit_reuse_test: bool,
    pub reused_store: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output_root_identity: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub store_root_identity: Option<String>,
    #[serde(default)]
    pub expected_output_digest_set_blake3: Vec<String>,
    #[serde(default)]
    pub observed_output_digest_set_blake3: Vec<String>,
    #[serde(default)]
    pub missing_outputs: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unsupported_reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RepeatabilityMatrixCellReport {
    pub cell_id: String,
    pub verdict: GauntletVerdict,
    pub cache_mode: String,
    pub store_isolation_mode: String,
    pub fresh_store: bool,
    pub explicit_reuse_test: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output_root_identity: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub store_root_identity: Option<String>,
    pub expected_output_digest_set_blake3: Vec<String>,
    pub observed_output_digest_set_blake3: Vec<String>,
    pub missing_outputs: Vec<String>,
    pub blockers: Vec<GauntletBlocker>,
    pub non_claims: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReleaseRepeatabilityMatrixReport {
    pub schema: String,
    pub profile: RepeatabilityMatrixProfile,
    pub cells: Vec<RepeatabilityMatrixCellReport>,
    pub repeatability_evidence_eligible: bool,
    pub blockers: Vec<GauntletBlocker>,
    pub non_claims: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrackSchemaVersion {
    pub track_id: String,
    pub schema_version: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContinuousGauntletContext {
    pub source_digest_blake3: String,
    pub policy_digest_blake3: String,
    pub universe_digest_blake3: String,
    pub toolchain_digest_blake3: String,
    pub host_class: String,
    pub witness_set_digest_blake3: String,
    #[serde(default)]
    pub required_track_schema_versions: Vec<TrackSchemaVersion>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrackRunEvidence {
    pub run_id: String,
    pub status: GauntletVerdict,
    #[serde(default)]
    pub blockers: Vec<String>,
    #[serde(default)]
    pub next_actions: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GauntletTrackEvidence {
    pub track_id: String,
    pub track_schema_version: String,
    pub report_digest_blake3: String,
    pub source_digest_blake3: String,
    pub policy_digest_blake3: String,
    pub universe_digest_blake3: String,
    pub toolchain_digest_blake3: String,
    pub host_class: String,
    pub witness_set_digest_blake3: String,
    #[serde(default)]
    pub runs: Vec<TrackRunEvidence>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrackAggregateReport {
    pub track_id: String,
    pub track_schema_version: String,
    pub report_digest_blake3: String,
    pub status: GauntletVerdict,
    pub first_run_id: String,
    pub last_run_id: String,
    pub blockers: Vec<GauntletBlocker>,
    pub next_actions: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContinuousReproducibilityGauntletReport {
    pub schema: String,
    pub context: ContinuousGauntletContext,
    pub tracks: Vec<TrackAggregateReport>,
    pub current_claim_status: GauntletVerdict,
    pub track_report_digests_blake3: Vec<String>,
    pub blockers: Vec<GauntletBlocker>,
    pub non_claims: Vec<String>,
}

pub fn evaluate_adversarial_hermeticity_gauntlet(
    run_id: String,
    source_digest_blake3: String,
    profile: HermeticityGauntletProfile,
    cells: Vec<HermeticityCellEvidence>,
) -> Result<AdversarialHermeticityGauntletReport, ReleaseEvidenceError> {
    validate_blake3_hex(&source_digest_blake3, DIGEST_FIELD_SOURCE)?;
    validate_non_empty_string(&run_id, "hermeticity run_id")?;
    let profile = canonicalize_hermeticity_profile(profile)?;
    let mut reports = Vec::new();
    for cell in cells {
        reports.push(evaluate_hermeticity_cell(cell)?);
    }
    let mut blockers = collect_cell_blockers(reports.iter().map(|cell| &cell.blockers))?;
    blockers.extend(profile_non_claim_blockers(&profile));
    let strict_evidence_eligible = reports.iter().all(|cell| cell.verdict.is_strict_evidence()) && blockers.is_empty();
    let mut non_claims = hermeticity_non_claims(&profile);
    if !strict_evidence_eligible {
        non_claims.push(NON_CLAIM_STRICT_REPRODUCIBILITY_BLOCKED.to_string());
    }
    canonical_adversarial_hermeticity_gauntlet_report(AdversarialHermeticityGauntletReport {
        schema: ADVERSARIAL_HERMETICITY_GAUNTLET_REPORT_SCHEMA.to_string(),
        run_id,
        source_digest_blake3,
        profile,
        cells: reports,
        strict_evidence_eligible,
        blockers,
        non_claims,
    })
}

pub fn canonical_adversarial_hermeticity_gauntlet_report(
    mut report: AdversarialHermeticityGauntletReport,
) -> Result<AdversarialHermeticityGauntletReport, ReleaseEvidenceError> {
    require_schema(
        &report.schema,
        ADVERSARIAL_HERMETICITY_GAUNTLET_REPORT_SCHEMA,
        "adversarial hermeticity report schema",
    )?;
    validate_blake3_hex(&report.source_digest_blake3, DIGEST_FIELD_SOURCE)?;
    report.schema = ADVERSARIAL_HERMETICITY_GAUNTLET_REPORT_SCHEMA.to_string();
    report.profile = canonicalize_hermeticity_profile(report.profile)?;
    report.cells.sort_by(|left, right| left.cell_id.cmp(&right.cell_id));
    for cell in &mut report.cells {
        canonicalize_hermeticity_cell_report(cell)?;
    }
    canonicalize_blockers(&mut report.blockers)?;
    canonicalize_string_vec(&mut report.non_claims, "hermeticity non_claims")?;
    validate_report_cell_count(report.cells.len(), "hermeticity cells")?;
    validate_non_empty_string(&report.run_id, "hermeticity run_id")?;
    Ok(report)
}

pub fn adversarial_hermeticity_gauntlet_report_canonical_bytes(
    report: AdversarialHermeticityGauntletReport,
) -> Result<Vec<u8>, ReleaseEvidenceError> {
    let canonical = canonical_adversarial_hermeticity_gauntlet_report(report)?;
    serialize_canonical(&canonical, "adversarial hermeticity report")
}

pub fn adversarial_hermeticity_gauntlet_report_digest_blake3(
    report: AdversarialHermeticityGauntletReport,
) -> Result<String, ReleaseEvidenceError> {
    Ok(blake3::hash(&adversarial_hermeticity_gauntlet_report_canonical_bytes(report)?).to_hex().to_string())
}

pub fn evaluate_bootstrap_pressure_gauntlet(
    run_id: String,
    source_digest_blake3: String,
    cells: Vec<BootstrapPressureCellEvidence>,
) -> Result<BootstrapPressureGauntletReport, ReleaseEvidenceError> {
    validate_blake3_hex(&source_digest_blake3, DIGEST_FIELD_SOURCE)?;
    validate_non_empty_string(&run_id, "bootstrap pressure run_id")?;
    let mut reports = Vec::new();
    for cell in cells {
        reports.push(evaluate_bootstrap_pressure_cell(cell)?);
    }
    let blockers = collect_cell_blockers(reports.iter().map(|cell| &cell.blockers))?;
    let mut non_claims = vec![
        NON_CLAIM_FULL_SOURCE_BOOTSTRAP.to_string(),
        NON_CLAIM_COMPILER_CORRECTNESS.to_string(),
    ];
    if !blockers.is_empty() {
        non_claims.push(NON_CLAIM_STRICT_REPRODUCIBILITY_BLOCKED.to_string());
    }
    canonical_bootstrap_pressure_gauntlet_report(BootstrapPressureGauntletReport {
        schema: BOOTSTRAP_PRESSURE_GAUNTLET_REPORT_SCHEMA.to_string(),
        run_id,
        source_digest_blake3,
        cells: reports,
        blockers,
        non_claims,
    })
}

pub fn canonical_bootstrap_pressure_gauntlet_report(
    mut report: BootstrapPressureGauntletReport,
) -> Result<BootstrapPressureGauntletReport, ReleaseEvidenceError> {
    require_schema(&report.schema, BOOTSTRAP_PRESSURE_GAUNTLET_REPORT_SCHEMA, "bootstrap pressure report schema")?;
    report.schema = BOOTSTRAP_PRESSURE_GAUNTLET_REPORT_SCHEMA.to_string();
    validate_non_empty_string(&report.run_id, "bootstrap pressure run_id")?;
    validate_blake3_hex(&report.source_digest_blake3, DIGEST_FIELD_SOURCE)?;
    report.cells.sort_by(|left, right| left.cell_id.cmp(&right.cell_id));
    for cell in &mut report.cells {
        canonicalize_bootstrap_pressure_cell_report(cell)?;
    }
    canonicalize_blockers(&mut report.blockers)?;
    canonicalize_string_vec(&mut report.non_claims, "bootstrap pressure non_claims")?;
    validate_report_cell_count(report.cells.len(), "bootstrap pressure cells")?;
    Ok(report)
}

pub fn bootstrap_pressure_gauntlet_report_canonical_bytes(
    report: BootstrapPressureGauntletReport,
) -> Result<Vec<u8>, ReleaseEvidenceError> {
    serialize_canonical(&canonical_bootstrap_pressure_gauntlet_report(report)?, "bootstrap pressure report")
}

pub fn bootstrap_pressure_gauntlet_report_digest_blake3(
    report: BootstrapPressureGauntletReport,
) -> Result<String, ReleaseEvidenceError> {
    Ok(blake3::hash(&bootstrap_pressure_gauntlet_report_canonical_bytes(report)?).to_hex().to_string())
}

pub fn evaluate_substitution_cache_attack_gauntlet(
    run_id: String,
    policy_digest_blake3: String,
    cases: Vec<SubstitutionAttackCaseEvidence>,
) -> Result<SubstitutionCacheAttackGauntletReport, ReleaseEvidenceError> {
    validate_non_empty_string(&run_id, "substitution attack run_id")?;
    validate_blake3_hex(&policy_digest_blake3, DIGEST_FIELD_POLICY)?;
    let mut reports = Vec::new();
    for case in cases {
        reports.push(evaluate_substitution_attack_case(case)?);
    }
    let blockers = collect_cell_blockers(reports.iter().map(|case| &case.blockers))?;
    let strict_cache_evidence_eligible = reports.iter().all(|case| !case.verdict.is_blocking()) && blockers.is_empty();
    let mut non_claims = vec![NON_CLAIM_COMPILER_CORRECTNESS.to_string()];
    if !strict_cache_evidence_eligible {
        non_claims.push(NON_CLAIM_STRICT_REPRODUCIBILITY_BLOCKED.to_string());
    }
    canonical_substitution_cache_attack_gauntlet_report(SubstitutionCacheAttackGauntletReport {
        schema: SUBSTITUTION_CACHE_ATTACK_GAUNTLET_REPORT_SCHEMA.to_string(),
        run_id,
        policy_digest_blake3,
        cases: reports,
        strict_cache_evidence_eligible,
        blockers,
        non_claims,
    })
}

pub fn canonical_substitution_cache_attack_gauntlet_report(
    mut report: SubstitutionCacheAttackGauntletReport,
) -> Result<SubstitutionCacheAttackGauntletReport, ReleaseEvidenceError> {
    require_schema(
        &report.schema,
        SUBSTITUTION_CACHE_ATTACK_GAUNTLET_REPORT_SCHEMA,
        "substitution cache attack report schema",
    )?;
    report.schema = SUBSTITUTION_CACHE_ATTACK_GAUNTLET_REPORT_SCHEMA.to_string();
    validate_non_empty_string(&report.run_id, "substitution attack run_id")?;
    validate_blake3_hex(&report.policy_digest_blake3, DIGEST_FIELD_POLICY)?;
    report.cases.sort_by(|left, right| left.case_id.cmp(&right.case_id));
    for case in &mut report.cases {
        canonicalize_substitution_case_report(case)?;
    }
    canonicalize_blockers(&mut report.blockers)?;
    canonicalize_string_vec(&mut report.non_claims, "substitution attack non_claims")?;
    validate_report_cell_count(report.cases.len(), "substitution attack cases")?;
    Ok(report)
}

pub fn substitution_cache_attack_gauntlet_report_canonical_bytes(
    report: SubstitutionCacheAttackGauntletReport,
) -> Result<Vec<u8>, ReleaseEvidenceError> {
    serialize_canonical(&canonical_substitution_cache_attack_gauntlet_report(report)?, "substitution attack report")
}

pub fn substitution_cache_attack_gauntlet_report_digest_blake3(
    report: SubstitutionCacheAttackGauntletReport,
) -> Result<String, ReleaseEvidenceError> {
    Ok(blake3::hash(&substitution_cache_attack_gauntlet_report_canonical_bytes(report)?)
        .to_hex()
        .to_string())
}

pub fn evaluate_nix_mantle_comparison_corpus(
    run_id: String,
    policy_digest_blake3: String,
    cases: Vec<NixMantleComparisonCaseEvidence>,
) -> Result<NixMantleComparisonCorpusReport, ReleaseEvidenceError> {
    validate_non_empty_string(&run_id, "Nix Mantle comparison run_id")?;
    validate_blake3_hex(&policy_digest_blake3, DIGEST_FIELD_POLICY)?;
    let mut reports = Vec::new();
    for case in cases {
        reports.push(evaluate_nix_mantle_comparison_case(case)?);
    }
    let blockers = collect_cell_blockers(reports.iter().map(|case| &case.blockers))?;
    let mut non_claims = vec![
        NON_CLAIM_GLOBAL_SUPERIORITY.to_string(),
        NON_CLAIM_COMPILER_CORRECTNESS.to_string(),
    ];
    if !blockers.is_empty() {
        non_claims.push(NON_CLAIM_STRICT_REPRODUCIBILITY_BLOCKED.to_string());
    }
    canonical_nix_mantle_comparison_corpus_report(NixMantleComparisonCorpusReport {
        schema: NIX_MANTLE_COMPARISON_CORPUS_REPORT_SCHEMA.to_string(),
        run_id,
        policy_digest_blake3,
        cases: reports,
        blockers,
        non_claims,
    })
}

pub fn canonical_nix_mantle_comparison_corpus_report(
    mut report: NixMantleComparisonCorpusReport,
) -> Result<NixMantleComparisonCorpusReport, ReleaseEvidenceError> {
    require_schema(&report.schema, NIX_MANTLE_COMPARISON_CORPUS_REPORT_SCHEMA, "Nix Mantle comparison report schema")?;
    report.schema = NIX_MANTLE_COMPARISON_CORPUS_REPORT_SCHEMA.to_string();
    validate_non_empty_string(&report.run_id, "Nix Mantle comparison run_id")?;
    validate_blake3_hex(&report.policy_digest_blake3, DIGEST_FIELD_POLICY)?;
    report.cases.sort_by(|left, right| left.case_id.cmp(&right.case_id));
    for case in &mut report.cases {
        canonicalize_nix_mantle_case_report(case)?;
    }
    canonicalize_blockers(&mut report.blockers)?;
    canonicalize_string_vec(&mut report.non_claims, "Nix Mantle comparison non_claims")?;
    validate_report_cell_count(report.cases.len(), "Nix Mantle comparison cases")?;
    Ok(report)
}

pub fn nix_mantle_comparison_corpus_report_canonical_bytes(
    report: NixMantleComparisonCorpusReport,
) -> Result<Vec<u8>, ReleaseEvidenceError> {
    serialize_canonical(&canonical_nix_mantle_comparison_corpus_report(report)?, "Nix Mantle comparison report")
}

pub fn nix_mantle_comparison_corpus_report_digest_blake3(
    report: NixMantleComparisonCorpusReport,
) -> Result<String, ReleaseEvidenceError> {
    Ok(blake3::hash(&nix_mantle_comparison_corpus_report_canonical_bytes(report)?).to_hex().to_string())
}

pub fn evaluate_release_repeatability_matrix(
    profile: RepeatabilityMatrixProfile,
    cells: Vec<RepeatabilityMatrixCellEvidence>,
) -> Result<ReleaseRepeatabilityMatrixReport, ReleaseEvidenceError> {
    let profile = canonicalize_repeatability_profile(profile)?;
    let mut reports = Vec::new();
    for cell in cells {
        reports.push(evaluate_repeatability_cell(&profile, cell)?);
    }
    let blockers = collect_cell_blockers(reports.iter().map(|cell| &cell.blockers))?;
    let repeatability_evidence_eligible =
        reports.iter().all(|cell| cell.verdict == GauntletVerdict::Matched) && blockers.is_empty();
    let mut non_claims = vec![NON_CLAIM_COMPILER_CORRECTNESS.to_string()];
    if !repeatability_evidence_eligible {
        non_claims.push(NON_CLAIM_STRICT_REPRODUCIBILITY_BLOCKED.to_string());
    }
    canonical_release_repeatability_matrix_report(ReleaseRepeatabilityMatrixReport {
        schema: RELEASE_REPEATABILITY_MATRIX_REPORT_SCHEMA.to_string(),
        profile,
        cells: reports,
        repeatability_evidence_eligible,
        blockers,
        non_claims,
    })
}

pub fn canonical_release_repeatability_matrix_report(
    mut report: ReleaseRepeatabilityMatrixReport,
) -> Result<ReleaseRepeatabilityMatrixReport, ReleaseEvidenceError> {
    require_schema(
        &report.schema,
        RELEASE_REPEATABILITY_MATRIX_REPORT_SCHEMA,
        "release repeatability matrix report schema",
    )?;
    report.schema = RELEASE_REPEATABILITY_MATRIX_REPORT_SCHEMA.to_string();
    report.profile = canonicalize_repeatability_profile(report.profile)?;
    report.cells.sort_by(|left, right| left.cell_id.cmp(&right.cell_id));
    for cell in &mut report.cells {
        canonicalize_repeatability_cell_report(cell)?;
    }
    canonicalize_blockers(&mut report.blockers)?;
    canonicalize_string_vec(&mut report.non_claims, "release repeatability non_claims")?;
    validate_report_cell_count(report.cells.len(), "release repeatability cells")?;
    Ok(report)
}

pub fn release_repeatability_matrix_report_canonical_bytes(
    report: ReleaseRepeatabilityMatrixReport,
) -> Result<Vec<u8>, ReleaseEvidenceError> {
    serialize_canonical(&canonical_release_repeatability_matrix_report(report)?, "release repeatability report")
}

pub fn release_repeatability_matrix_report_digest_blake3(
    report: ReleaseRepeatabilityMatrixReport,
) -> Result<String, ReleaseEvidenceError> {
    Ok(blake3::hash(&release_repeatability_matrix_report_canonical_bytes(report)?).to_hex().to_string())
}

pub fn evaluate_continuous_reproducibility_gauntlet(
    context: ContinuousGauntletContext,
    tracks: Vec<GauntletTrackEvidence>,
) -> Result<ContinuousReproducibilityGauntletReport, ReleaseEvidenceError> {
    let context = canonicalize_continuous_context(context)?;
    let required_versions = required_track_versions(&context)?;
    let mut reports = Vec::new();
    for track in tracks {
        reports.push(evaluate_track_evidence(&context, &required_versions, track)?);
    }
    let blockers = collect_cell_blockers(reports.iter().map(|track| &track.blockers))?;
    let current_claim_status = classify_continuous_status(&reports, &blockers);
    let mut digests = reports.iter().map(|track| track.report_digest_blake3.clone()).collect::<Vec<_>>();
    canonicalize_digest_vec(&mut digests, "continuous track report digests")?;
    let mut non_claims = vec![NON_CLAIM_GLOBAL_SUPERIORITY.to_string()];
    if current_claim_status != GauntletVerdict::Accepted {
        non_claims.push(NON_CLAIM_STALE_EVIDENCE.to_string());
    }
    canonical_continuous_reproducibility_gauntlet_report(ContinuousReproducibilityGauntletReport {
        schema: CONTINUOUS_REPRODUCIBILITY_GAUNTLET_REPORT_SCHEMA.to_string(),
        context,
        tracks: reports,
        current_claim_status,
        track_report_digests_blake3: digests,
        blockers,
        non_claims,
    })
}

pub fn canonical_continuous_reproducibility_gauntlet_report(
    mut report: ContinuousReproducibilityGauntletReport,
) -> Result<ContinuousReproducibilityGauntletReport, ReleaseEvidenceError> {
    require_schema(
        &report.schema,
        CONTINUOUS_REPRODUCIBILITY_GAUNTLET_REPORT_SCHEMA,
        "continuous gauntlet report schema",
    )?;
    report.schema = CONTINUOUS_REPRODUCIBILITY_GAUNTLET_REPORT_SCHEMA.to_string();
    report.context = canonicalize_continuous_context(report.context)?;
    report.tracks.sort_by(|left, right| left.track_id.cmp(&right.track_id));
    for track in &mut report.tracks {
        canonicalize_track_report(track)?;
    }
    canonicalize_digest_vec(&mut report.track_report_digests_blake3, "continuous track report digests")?;
    canonicalize_blockers(&mut report.blockers)?;
    canonicalize_string_vec(&mut report.non_claims, "continuous non_claims")?;
    validate_continuous_track_count(report.tracks.len())?;
    Ok(report)
}

pub fn continuous_reproducibility_gauntlet_report_canonical_bytes(
    report: ContinuousReproducibilityGauntletReport,
) -> Result<Vec<u8>, ReleaseEvidenceError> {
    serialize_canonical(&canonical_continuous_reproducibility_gauntlet_report(report)?, "continuous gauntlet report")
}

pub fn continuous_reproducibility_gauntlet_report_digest_blake3(
    report: ContinuousReproducibilityGauntletReport,
) -> Result<String, ReleaseEvidenceError> {
    Ok(blake3::hash(&continuous_reproducibility_gauntlet_report_canonical_bytes(report)?)
        .to_hex()
        .to_string())
}

fn evaluate_hermeticity_cell(mut cell: HermeticityCellEvidence) -> Result<HermeticityCellReport, ReleaseEvidenceError> {
    validate_non_empty_string(&cell.cell_id, "hermeticity cell_id")?;
    canonicalize_violations(&mut cell.observed_violations)?;
    canonicalize_string_vec(&mut cell.audit_events, "hermeticity audit_events")?;
    canonicalize_string_vec(&mut cell.unsupported_axes, "hermeticity unsupported_axes")?;
    validate_optional_digest(&cell.output_digest_blake3, "hermeticity output_digest_blake3")?;
    let mut blockers = hermeticity_cell_blockers(&cell)?;
    let verdict = hermeticity_verdict(&cell, &blockers);
    let mut non_claims = unsupported_axis_non_claims(&cell.unsupported_axes);
    if verdict == GauntletVerdict::Degraded || verdict.is_blocking() {
        non_claims.push(NON_CLAIM_STRICT_REPRODUCIBILITY_BLOCKED.to_string());
    }
    canonicalize_blockers(&mut blockers)?;
    canonicalize_string_vec(&mut non_claims, "hermeticity cell non_claims")?;
    Ok(HermeticityCellReport {
        cell_id: cell.cell_id,
        mode: cell.mode,
        verdict,
        output_digest_blake3: cell.output_digest_blake3,
        blockers,
        non_claims,
    })
}

fn hermeticity_cell_blockers(cell: &HermeticityCellEvidence) -> Result<Vec<GauntletBlocker>, ReleaseEvidenceError> {
    let mut blockers = Vec::new();
    for violation in &cell.observed_violations {
        validate_non_empty_string(&violation.class, "hermeticity violation class")?;
        validate_non_empty_string(&violation.detail, "hermeticity violation detail")?;
        blockers.push(blocker(
            "hermeticity-violation",
            &violation.detail,
            "rerun with the hidden host influence removed or keep the cell as a blocker",
            violation.axis.clone(),
            None,
            None,
        ));
    }
    if cell.mode == GauntletMode::Strict && !cell.unsupported_axes.is_empty() {
        blockers.push(blocker(
            "unsupported-hermeticity-axis",
            "strict hermeticity cell has unsupported perturbation axes",
            "record the unsupported axis as a non-claim or add host support before using strict evidence",
            None,
            None,
            None,
        ));
    }
    Ok(blockers)
}

fn hermeticity_verdict(cell: &HermeticityCellEvidence, blockers: &[GauntletBlocker]) -> GauntletVerdict {
    if blockers.is_empty() {
        return GauntletVerdict::Accepted;
    }
    match cell.mode {
        GauntletMode::Strict => GauntletVerdict::Blocked,
        GauntletMode::Practical => GauntletVerdict::Degraded,
    }
}

fn evaluate_bootstrap_pressure_cell(
    mut cell: BootstrapPressureCellEvidence,
) -> Result<BootstrapPressureCellReport, ReleaseEvidenceError> {
    validate_bootstrap_pressure_cell(&mut cell)?;
    let mut blockers = bootstrap_pressure_cell_blockers(&cell);
    let verdict = bootstrap_pressure_verdict(&cell, &blockers);
    let mut non_claims = vec![
        NON_CLAIM_FULL_SOURCE_BOOTSTRAP.to_string(),
        NON_CLAIM_COMPILER_CORRECTNESS.to_string(),
    ];
    if verdict.is_blocking() {
        non_claims.push(NON_CLAIM_STRICT_REPRODUCIBILITY_BLOCKED.to_string());
    }
    canonicalize_blockers(&mut blockers)?;
    canonicalize_string_vec(&mut non_claims, "bootstrap pressure cell non_claims")?;
    Ok(BootstrapPressureCellReport {
        cell_id: cell.cell_id,
        profile: cell.profile,
        verdict,
        fixed_point: cell.fixed_point,
        host_tool_policy: cell.host_tool_policy,
        seed_inventory_digest_blake3: cell.seed_inventory_digest_blake3,
        protected_exec_audit_digest_blake3: cell.protected_exec_audit_digest_blake3,
        stage_output_digest_set_blake3: cell.stage_output_digest_set_blake3,
        remaining_trusted_root: cell.remaining_trusted_root,
        observed_execs: cell.observed_execs,
        blockers,
        non_claims,
    })
}

fn validate_bootstrap_pressure_cell(cell: &mut BootstrapPressureCellEvidence) -> Result<(), ReleaseEvidenceError> {
    validate_non_empty_string(&cell.cell_id, "bootstrap pressure cell_id")?;
    validate_non_empty_string(&cell.profile, "bootstrap pressure profile")?;
    validate_non_empty_string(&cell.host_tool_policy, "bootstrap pressure host_tool_policy")?;
    validate_optional_digest(&cell.seed_inventory_digest_blake3, "seed_inventory_digest_blake3")?;
    validate_optional_digest(&cell.protected_exec_audit_digest_blake3, "protected_exec_audit_digest_blake3")?;
    canonicalize_digest_vec(&mut cell.stage_output_digest_set_blake3, "stage_output_digest_set_blake3")?;
    canonicalize_string_vec(&mut cell.remaining_trusted_root, "remaining_trusted_root")?;
    for observed in &cell.observed_execs {
        validate_observed_exec(observed)?;
    }
    cell.observed_execs.sort_by(|left, right| left.path.cmp(&right.path));
    validate_optional_string(&cell.unsupported_reason, "bootstrap pressure unsupported_reason")?;
    Ok(())
}

fn bootstrap_pressure_cell_blockers(cell: &BootstrapPressureCellEvidence) -> Vec<GauntletBlocker> {
    let mut blockers = Vec::new();
    if let Some(reason) = &cell.unsupported_reason {
        blockers.push(blocker(
            "unsupported-bootstrap-profile",
            reason,
            "rerun on a supported host profile",
            None,
            None,
            None,
        ));
    }
    push_protected_profile_blockers(cell, &mut blockers);
    push_bootstrap_exec_blockers(cell, &mut blockers);
    push_bootstrap_fixed_point_blockers(cell, &mut blockers);
    blockers
}

fn push_protected_profile_blockers(cell: &BootstrapPressureCellEvidence, blockers: &mut Vec<GauntletBlocker>) {
    if !profile_requires_protected_exec(&cell.profile) {
        return;
    }
    if cell.seed_inventory_digest_blake3.is_none() {
        blockers.push(blocker(
            "missing-seed-inventory",
            "protected bootstrap profile is missing seed inventory digest",
            "bind the protected phase to a declared seed inventory digest",
            None,
            None,
            None,
        ));
    }
    if cell.protected_exec_audit_digest_blake3.is_none() {
        blockers.push(blocker(
            "missing-protected-exec-audit",
            "protected bootstrap profile is missing protected-exec audit digest",
            "capture protected-exec audit evidence before promoting this profile",
            None,
            None,
            None,
        ));
    }
}

fn push_bootstrap_exec_blockers(cell: &BootstrapPressureCellEvidence, blockers: &mut Vec<GauntletBlocker>) {
    for observed in &cell.observed_execs {
        if observed.declared {
            continue;
        }
        blockers.push(blocker(
            "undeclared-protected-exec",
            &format!("undeclared protected executable observed: {}", observed.executable_class),
            "add the executable to the inventory or remove it from the protected phase",
            Some(observed.path.clone()),
            None,
            Some(observed.digest_blake3.clone()),
        ));
    }
}

fn push_bootstrap_fixed_point_blockers(cell: &BootstrapPressureCellEvidence, blockers: &mut Vec<GauntletBlocker>) {
    if !cell.fixed_point {
        blockers.push(blocker(
            "bootstrap-fixed-point",
            "bootstrap pressure profile did not prove a fixed point",
            "rerun the profile until stage outputs match or record the mismatch as evidence debt",
            None,
            None,
            None,
        ));
    }
    if cell.profile == PROFILE_FULL_SOURCE_ROOT_ATTEMPT && !cell.remaining_trusted_root.is_empty() {
        blockers.push(blocker(
            "remaining-trusted-root",
            "full-source-root attempt still has trusted seed or host blockers",
            "derive the remaining root from accepted source evidence before making a full-source claim",
            None,
            None,
            None,
        ));
    }
}

fn bootstrap_pressure_verdict(cell: &BootstrapPressureCellEvidence, blockers: &[GauntletBlocker]) -> GauntletVerdict {
    if cell.unsupported_reason.is_some() {
        return GauntletVerdict::Unsupported;
    }
    if blockers.is_empty() {
        return GauntletVerdict::Accepted;
    }
    GauntletVerdict::Blocked
}

fn evaluate_substitution_attack_case(
    mut case: SubstitutionAttackCaseEvidence,
) -> Result<SubstitutionAttackCaseReport, ReleaseEvidenceError> {
    validate_substitution_attack_case(&mut case)?;
    let attack_observed = substitution_attack_observed(&case);
    let mut blockers = substitution_attack_blockers(&case, attack_observed);
    let verdict = substitution_attack_verdict(&case, attack_observed, &blockers);
    let mut non_claims = Vec::new();
    if verdict == GauntletVerdict::Degraded || verdict.is_blocking() {
        non_claims.push(NON_CLAIM_STRICT_REPRODUCIBILITY_BLOCKED.to_string());
    }
    canonicalize_blockers(&mut blockers)?;
    canonicalize_string_vec(&mut non_claims, "substitution attack case non_claims")?;
    Ok(SubstitutionAttackCaseReport {
        case_id: case.case_id,
        verdict,
        strict_mode: case.strict_mode,
        accepted: case.accepted,
        trusted_key_material: case.trusted_key_material,
        fallback_mode: case.fallback_mode,
        expected_digest_blake3: case.expected_digest_blake3,
        observed_digest_blake3: case.observed_digest_blake3,
        failed_trust_edges: case.failed_trust_edges,
        blockers,
        non_claims,
    })
}

fn validate_substitution_attack_case(case: &mut SubstitutionAttackCaseEvidence) -> Result<(), ReleaseEvidenceError> {
    validate_non_empty_string(&case.case_id, "substitution attack case_id")?;
    validate_optional_string(&case.trusted_key_material, "trusted_key_material")?;
    validate_optional_string(&case.fallback_mode, "fallback_mode")?;
    validate_optional_digest(&case.expected_digest_blake3, "expected_digest_blake3")?;
    validate_optional_digest(&case.observed_digest_blake3, "observed_digest_blake3")?;
    canonicalize_string_vec(&mut case.failed_trust_edges, "failed_trust_edges")?;
    Ok(())
}

fn substitution_attack_observed(case: &SubstitutionAttackCaseEvidence) -> bool {
    !case.failed_trust_edges.is_empty()
        || !case.closure_complete
        || !case.artifact_attestation_fresh
        || digest_options_mismatch(&case.expected_digest_blake3, &case.observed_digest_blake3)
}

fn substitution_attack_blockers(case: &SubstitutionAttackCaseEvidence, attack_observed: bool) -> Vec<GauntletBlocker> {
    let mut blockers = Vec::new();
    if !attack_observed && !case.accepted {
        blockers.push(blocker(
            "trusted-substitute-rejected",
            "trusted substitute with matching evidence was not accepted",
            "inspect signature, closure, and content evidence for the trusted fixture",
            None,
            case.expected_digest_blake3.clone(),
            case.observed_digest_blake3.clone(),
        ));
    }
    if attack_observed && case.strict_mode && case.accepted {
        blockers.push(blocker(
            "malicious-cache-accepted",
            "strict mode accepted malicious or incomplete cache material",
            "reject the substitute before reuse evidence is emitted",
            None,
            case.expected_digest_blake3.clone(),
            case.observed_digest_blake3.clone(),
        ));
    }
    if attack_observed && !case.strict_mode && case.fallback_mode.is_some() {
        blockers.push(blocker(
            "practical-cache-fallback",
            "practical mode used fallback after invalid cache evidence",
            "keep fallback evidence out of strict reproducibility admission",
            None,
            case.expected_digest_blake3.clone(),
            case.observed_digest_blake3.clone(),
        ));
    }
    blockers
}

fn substitution_attack_verdict(
    case: &SubstitutionAttackCaseEvidence,
    attack_observed: bool,
    blockers: &[GauntletBlocker],
) -> GauntletVerdict {
    if !blockers.is_empty() && case.strict_mode {
        return GauntletVerdict::Blocked;
    }
    if attack_observed && case.strict_mode && !case.accepted {
        return GauntletVerdict::Rejected;
    }
    if attack_observed && !case.strict_mode {
        return GauntletVerdict::Degraded;
    }
    if blockers.is_empty() && case.accepted {
        return GauntletVerdict::Accepted;
    }
    GauntletVerdict::Blocked
}

fn evaluate_nix_mantle_comparison_case(
    mut case: NixMantleComparisonCaseEvidence,
) -> Result<NixMantleComparisonCaseReport, ReleaseEvidenceError> {
    validate_nix_mantle_case(&mut case)?;
    let mut blockers = nix_mantle_case_blockers(&case);
    let verdict = nix_mantle_case_verdict(&case, &blockers);
    let mut non_claims = vec![NON_CLAIM_GLOBAL_SUPERIORITY.to_string()];
    if verdict.is_blocking() || verdict == GauntletVerdict::Mismatched {
        non_claims.push(NON_CLAIM_STRICT_REPRODUCIBILITY_BLOCKED.to_string());
    }
    canonicalize_blockers(&mut blockers)?;
    canonicalize_string_vec(&mut non_claims, "Nix Mantle case non_claims")?;
    Ok(NixMantleComparisonCaseReport {
        case_id: case.case_id,
        verdict,
        build_recipe_identity: case.build_recipe_identity,
        normalization_policy: case.normalization_policy,
        source_refs: case.source_refs,
        toolchain_refs: case.toolchain_refs,
        dependency_refs: case.dependency_refs,
        output_surfaces: case.output_surfaces,
        allowed_differences: case.allowed_differences,
        nix_digest_set_blake3: case.nix_digest_set_blake3,
        mantle_digest_set_blake3: case.mantle_digest_set_blake3,
        blockers,
        non_claims,
    })
}

fn validate_nix_mantle_case(case: &mut NixMantleComparisonCaseEvidence) -> Result<(), ReleaseEvidenceError> {
    validate_non_empty_string(&case.case_id, "comparison case_id")?;
    validate_non_empty_string(&case.build_recipe_identity, "build_recipe_identity")?;
    validate_non_empty_string(&case.normalization_policy, "normalization_policy")?;
    canonicalize_string_vec(&mut case.source_refs, "source_refs")?;
    canonicalize_string_vec(&mut case.toolchain_refs, "toolchain_refs")?;
    canonicalize_string_vec(&mut case.dependency_refs, "dependency_refs")?;
    canonicalize_string_vec(&mut case.output_surfaces, "output_surfaces")?;
    canonicalize_string_vec(&mut case.allowed_differences, "allowed_differences")?;
    canonicalize_digest_vec(&mut case.nix_digest_set_blake3, "nix_digest_set_blake3")?;
    canonicalize_digest_vec(&mut case.mantle_digest_set_blake3, "mantle_digest_set_blake3")?;
    validate_optional_string(&case.unsupported_reason, "unsupported_reason")?;
    Ok(())
}

fn nix_mantle_case_blockers(case: &NixMantleComparisonCaseEvidence) -> Vec<GauntletBlocker> {
    let mut blockers = Vec::new();
    if let Some(reason) = &case.unsupported_reason {
        blockers.push(blocker(
            "unsupported-comparison-case",
            reason,
            "record a next action or remove the case from the required corpus",
            None,
            None,
            None,
        ));
    }
    if !case.equivalence_declared {
        blockers.push(blocker(
            "missing-equivalence-policy",
            "comparison case lacks equivalent input policy",
            "declare equivalent source, toolchain, dependency, recipe, output, and normalization policy",
            None,
            None,
            None,
        ));
    }
    if case.equivalence_declared && !digest_vecs_match(&case.nix_digest_set_blake3, &case.mantle_digest_set_blake3) {
        blockers.push(blocker(
            "comparison-digest-mismatch",
            "Nix and Mantle output digest sets differ",
            "inspect byte-level outputs before treating the case as matched",
            None,
            first_digest(&case.nix_digest_set_blake3),
            first_digest(&case.mantle_digest_set_blake3),
        ));
    }
    blockers
}

fn nix_mantle_case_verdict(case: &NixMantleComparisonCaseEvidence, blockers: &[GauntletBlocker]) -> GauntletVerdict {
    if case.unsupported_reason.is_some() {
        return GauntletVerdict::Unsupported;
    }
    if !case.equivalence_declared {
        return GauntletVerdict::Blocked;
    }
    if blockers.iter().any(|blocker| blocker.evidence_class == "comparison-digest-mismatch") {
        return GauntletVerdict::Mismatched;
    }
    if blockers.is_empty() {
        return GauntletVerdict::Matched;
    }
    GauntletVerdict::Blocked
}

fn evaluate_repeatability_cell(
    profile: &RepeatabilityMatrixProfile,
    mut cell: RepeatabilityMatrixCellEvidence,
) -> Result<RepeatabilityMatrixCellReport, ReleaseEvidenceError> {
    validate_repeatability_cell(&mut cell)?;
    let mut blockers = repeatability_cell_blockers(profile, &cell);
    let verdict = repeatability_cell_verdict(&cell, &blockers);
    let mut non_claims = Vec::new();
    if verdict.is_blocking() || verdict == GauntletVerdict::Mismatched {
        non_claims.push(NON_CLAIM_STRICT_REPRODUCIBILITY_BLOCKED.to_string());
    }
    canonicalize_blockers(&mut blockers)?;
    canonicalize_string_vec(&mut non_claims, "repeatability cell non_claims")?;
    Ok(RepeatabilityMatrixCellReport {
        cell_id: cell.cell_id,
        verdict,
        cache_mode: cell.cache_mode,
        store_isolation_mode: cell.store_isolation_mode,
        fresh_store: cell.fresh_store,
        explicit_reuse_test: cell.explicit_reuse_test,
        output_root_identity: cell.output_root_identity,
        store_root_identity: cell.store_root_identity,
        expected_output_digest_set_blake3: cell.expected_output_digest_set_blake3,
        observed_output_digest_set_blake3: cell.observed_output_digest_set_blake3,
        missing_outputs: cell.missing_outputs,
        blockers,
        non_claims,
    })
}

fn validate_repeatability_cell(cell: &mut RepeatabilityMatrixCellEvidence) -> Result<(), ReleaseEvidenceError> {
    validate_non_empty_string(&cell.cell_id, "repeatability cell_id")?;
    validate_non_empty_string(&cell.cache_mode, "repeatability cache_mode")?;
    validate_non_empty_string(&cell.store_isolation_mode, "repeatability store_isolation_mode")?;
    validate_optional_string(&cell.output_root_identity, "output_root_identity")?;
    validate_optional_string(&cell.store_root_identity, "store_root_identity")?;
    canonicalize_digest_vec(&mut cell.expected_output_digest_set_blake3, "expected_output_digest_set_blake3")?;
    canonicalize_digest_vec(&mut cell.observed_output_digest_set_blake3, "observed_output_digest_set_blake3")?;
    canonicalize_string_vec(&mut cell.missing_outputs, "missing_outputs")?;
    validate_optional_string(&cell.unsupported_reason, "repeatability unsupported_reason")?;
    Ok(())
}

fn repeatability_cell_blockers(
    profile: &RepeatabilityMatrixProfile,
    cell: &RepeatabilityMatrixCellEvidence,
) -> Vec<GauntletBlocker> {
    let mut blockers = Vec::new();
    if let Some(reason) = &cell.unsupported_reason {
        blockers.push(blocker(
            "unsupported-repeatability-cell",
            reason,
            "rerun on a supported host or record the axis as a non-claim",
            None,
            None,
            None,
        ));
    }
    push_fresh_store_blockers(cell, &mut blockers);
    push_repeatability_digest_blockers(profile, cell, &mut blockers);
    blockers
}

fn push_fresh_store_blockers(cell: &RepeatabilityMatrixCellEvidence, blockers: &mut Vec<GauntletBlocker>) {
    if !cell.fresh_store {
        return;
    }
    if cell.output_root_identity.is_none() || cell.store_root_identity.is_none() {
        blockers.push(blocker(
            "missing-isolated-roots",
            "fresh-store matrix cell lacks output/store root identities",
            "record isolated output and store root identities for the cell",
            None,
            None,
            None,
        ));
    }
    if cell.reused_store && !cell.explicit_reuse_test {
        blockers.push(blocker(
            "reused-store",
            "fresh-store matrix cell reused prior store state",
            "rerun with a fresh store or mark this as an explicit reuse experiment",
            None,
            None,
            None,
        ));
    }
}

fn push_repeatability_digest_blockers(
    profile: &RepeatabilityMatrixProfile,
    cell: &RepeatabilityMatrixCellEvidence,
    blockers: &mut Vec<GauntletBlocker>,
) {
    if !cell.missing_outputs.is_empty() {
        blockers.push(blocker(
            "missing-repeatability-output",
            "matrix cell produced missing outputs",
            "inspect the failing axis and keep global admission blocked",
            None,
            first_digest(&cell.expected_output_digest_set_blake3),
            None,
        ));
    }
    if !digest_vecs_match(&cell.expected_output_digest_set_blake3, &cell.observed_output_digest_set_blake3) {
        blockers.push(blocker(
            "repeatability-digest-mismatch",
            "matrix cell output digest set did not match expected release digests",
            "inspect the mismatch and keep affected release surfaces blocked",
            None,
            first_digest(&cell.expected_output_digest_set_blake3),
            first_digest(&cell.observed_output_digest_set_blake3),
        ));
    }
    if !digest_vecs_match(&profile.expected_output_digest_set_blake3, &cell.expected_output_digest_set_blake3) {
        blockers.push(blocker(
            "profile-digest-mismatch",
            "matrix cell expected digest set does not match profile expected digest set",
            "regenerate the cell plan from the matrix profile",
            None,
            first_digest(&profile.expected_output_digest_set_blake3),
            first_digest(&cell.expected_output_digest_set_blake3),
        ));
    }
}

fn repeatability_cell_verdict(cell: &RepeatabilityMatrixCellEvidence, blockers: &[GauntletBlocker]) -> GauntletVerdict {
    if cell.unsupported_reason.is_some() {
        return GauntletVerdict::Unsupported;
    }
    if blockers.iter().any(|blocker| blocker.evidence_class.contains("digest-mismatch")) {
        return GauntletVerdict::Mismatched;
    }
    if !cell.missing_outputs.is_empty() {
        return GauntletVerdict::Missing;
    }
    if blockers.is_empty() {
        return GauntletVerdict::Matched;
    }
    GauntletVerdict::Blocked
}

fn evaluate_track_evidence(
    context: &ContinuousGauntletContext,
    required_versions: &BTreeMap<String, String>,
    mut track: GauntletTrackEvidence,
) -> Result<TrackAggregateReport, ReleaseEvidenceError> {
    validate_track_evidence(&mut track)?;
    let mut blockers = track_staleness_blockers(context, required_versions, &track);
    let run_summary = summarize_track_runs(&track.runs, &mut blockers)?;
    let status = classify_track_status(&blockers, &run_summary.statuses);
    let mut next_actions = run_summary.next_actions;
    if status == GauntletVerdict::Stale {
        next_actions.push("regenerate the track report for the current claim boundary".to_string());
    }
    canonicalize_blockers(&mut blockers)?;
    canonicalize_string_vec(&mut next_actions, "track next_actions")?;
    Ok(TrackAggregateReport {
        track_id: track.track_id,
        track_schema_version: track.track_schema_version,
        report_digest_blake3: track.report_digest_blake3,
        status,
        first_run_id: run_summary.first_run_id,
        last_run_id: run_summary.last_run_id,
        blockers,
        next_actions,
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct TrackRunSummary {
    first_run_id: String,
    last_run_id: String,
    statuses: BTreeSet<GauntletVerdict>,
    next_actions: Vec<String>,
}

fn summarize_track_runs(
    runs: &[TrackRunEvidence],
    blockers: &mut Vec<GauntletBlocker>,
) -> Result<TrackRunSummary, ReleaseEvidenceError> {
    let run_count = u32_count(runs.len(), "continuous run count overflowed u32")?;
    if run_count == ZERO_COUNT {
        blockers.push(blocker(
            "missing-track-run",
            "track evidence has no recorded runs",
            "record at least one track run before aggregation",
            None,
            None,
            None,
        ));
        return Ok(TrackRunSummary::empty());
    }
    if run_count > MAX_GAUNTLET_RUN_COUNT {
        return Err(validation_error(format!(
            "continuous track has {run_count} runs, limit is {MAX_GAUNTLET_RUN_COUNT}"
        )));
    }
    collect_track_run_summary(runs, blockers)
}

fn collect_track_run_summary(
    runs: &[TrackRunEvidence],
    blockers: &mut Vec<GauntletBlocker>,
) -> Result<TrackRunSummary, ReleaseEvidenceError> {
    let mut sorted_runs = runs.to_vec();
    sorted_runs.sort_by(|left, right| left.run_id.cmp(&right.run_id));
    let mut statuses = BTreeSet::new();
    let mut next_actions = Vec::new();
    for run in &sorted_runs {
        validate_track_run(run)?;
        statuses.insert(run.status);
        push_run_blockers(run, blockers);
        next_actions.extend(run.next_actions.iter().cloned());
    }
    Ok(TrackRunSummary {
        first_run_id: sorted_runs.first().map(|run| run.run_id.clone()).unwrap_or_default(),
        last_run_id: sorted_runs.last().map(|run| run.run_id.clone()).unwrap_or_default(),
        statuses,
        next_actions,
    })
}

impl TrackRunSummary {
    fn empty() -> Self {
        Self {
            first_run_id: "none".to_string(),
            last_run_id: "none".to_string(),
            statuses: BTreeSet::new(),
            next_actions: Vec::new(),
        }
    }
}

fn track_staleness_blockers(
    context: &ContinuousGauntletContext,
    required_versions: &BTreeMap<String, String>,
    track: &GauntletTrackEvidence,
) -> Vec<GauntletBlocker> {
    let mut blockers = Vec::new();
    push_digest_boundary_blocker(
        &mut blockers,
        DIGEST_FIELD_SOURCE,
        &context.source_digest_blake3,
        &track.source_digest_blake3,
    );
    push_digest_boundary_blocker(
        &mut blockers,
        DIGEST_FIELD_POLICY,
        &context.policy_digest_blake3,
        &track.policy_digest_blake3,
    );
    push_digest_boundary_blocker(
        &mut blockers,
        DIGEST_FIELD_UNIVERSE,
        &context.universe_digest_blake3,
        &track.universe_digest_blake3,
    );
    push_digest_boundary_blocker(
        &mut blockers,
        DIGEST_FIELD_TOOLCHAIN,
        &context.toolchain_digest_blake3,
        &track.toolchain_digest_blake3,
    );
    push_digest_boundary_blocker(
        &mut blockers,
        DIGEST_FIELD_WITNESS_SET,
        &context.witness_set_digest_blake3,
        &track.witness_set_digest_blake3,
    );
    if context.host_class != track.host_class {
        blockers.push(blocker(
            "host-class-stale",
            "track evidence was produced for a different host class",
            "rerun or mark the track out-of-scope for this host class",
            None,
            Some(context.host_class.clone()),
            Some(track.host_class.clone()),
        ));
    }
    push_schema_version_blocker(&mut blockers, required_versions, track);
    blockers
}

fn push_schema_version_blocker(
    blockers: &mut Vec<GauntletBlocker>,
    required_versions: &BTreeMap<String, String>,
    track: &GauntletTrackEvidence,
) {
    let Some(required_version) = required_versions.get(&track.track_id) else {
        return;
    };
    if required_version == &track.track_schema_version {
        return;
    }
    blockers.push(blocker(
        "track-schema-stale",
        "track evidence schema version does not match the current claim boundary",
        "regenerate the track report with the required schema version",
        None,
        Some(required_version.clone()),
        Some(track.track_schema_version.clone()),
    ));
}

fn push_digest_boundary_blocker(blockers: &mut Vec<GauntletBlocker>, field: &str, expected: &str, observed: &str) {
    if expected == observed {
        return;
    }
    blockers.push(blocker(
        "track-boundary-stale",
        &format!("track evidence {field} does not match the current claim boundary"),
        "regenerate the track report for the current claim boundary",
        None,
        Some(expected.to_string()),
        Some(observed.to_string()),
    ));
}

fn classify_track_status(blockers: &[GauntletBlocker], statuses: &BTreeSet<GauntletVerdict>) -> GauntletVerdict {
    if blockers.iter().any(|blocker| blocker.evidence_class.contains("stale")) {
        return GauntletVerdict::Stale;
    }
    if statuses.len() > 1 {
        return GauntletVerdict::Flaky;
    }
    if !blockers.is_empty() {
        return GauntletVerdict::Blocked;
    }
    statuses.iter().next().copied().unwrap_or(GauntletVerdict::Missing)
}

fn classify_continuous_status(tracks: &[TrackAggregateReport], blockers: &[GauntletBlocker]) -> GauntletVerdict {
    if !blockers.is_empty() {
        return GauntletVerdict::Blocked;
    }
    if tracks.is_empty() {
        return GauntletVerdict::Missing;
    }
    if tracks.iter().any(|track| track.status == GauntletVerdict::Stale) {
        return GauntletVerdict::Stale;
    }
    if tracks.iter().any(|track| track.status == GauntletVerdict::Flaky) {
        return GauntletVerdict::Flaky;
    }
    if tracks
        .iter()
        .all(|track| matches!(track.status, GauntletVerdict::Accepted | GauntletVerdict::Matched))
    {
        return GauntletVerdict::Accepted;
    }
    GauntletVerdict::Blocked
}

fn canonicalize_hermeticity_profile(
    mut profile: HermeticityGauntletProfile,
) -> Result<HermeticityGauntletProfile, ReleaseEvidenceError> {
    canonicalize_axis_vec(&mut profile.host_tool_axes, "host_tool_axes")?;
    canonicalize_axis_vec(&mut profile.environment_axes, "environment_axes")?;
    canonicalize_axis_vec(&mut profile.network_axes, "network_axes")?;
    canonicalize_axis_vec(&mut profile.timestamp_axes, "timestamp_axes")?;
    canonicalize_axis_vec(&mut profile.locale_axes, "locale_axes")?;
    canonicalize_axis_vec(&mut profile.umask_axes, "umask_axes")?;
    canonicalize_axis_vec(&mut profile.temp_path_axes, "temp_path_axes")?;
    canonicalize_axis_vec(&mut profile.store_path_axes, "store_path_axes")?;
    canonicalize_axis_vec(&mut profile.randomness_axes, "randomness_axes")?;
    Ok(profile)
}

fn profile_non_claim_blockers(profile: &HermeticityGauntletProfile) -> Vec<GauntletBlocker> {
    missing_profile_axes(profile)
        .into_iter()
        .map(|axis| {
            blocker(
                "omitted-hermeticity-axis",
                "hermeticity profile omits a perturbation axis",
                "record the omission as a non-claim or add the axis to the gauntlet profile",
                Some(axis),
                None,
                None,
            )
        })
        .collect()
}

fn hermeticity_non_claims(profile: &HermeticityGauntletProfile) -> Vec<String> {
    missing_profile_axes(profile)
        .into_iter()
        .map(|axis| format!("{NON_CLAIM_UNTESTED_AXIS_PREFIX}:{axis}"))
        .collect()
}

fn missing_profile_axes(profile: &HermeticityGauntletProfile) -> Vec<String> {
    let mut missing = Vec::new();
    push_missing_axis(&mut missing, "host-tool", &profile.host_tool_axes);
    push_missing_axis(&mut missing, "environment", &profile.environment_axes);
    push_missing_axis(&mut missing, "network", &profile.network_axes);
    push_missing_axis(&mut missing, "timestamp", &profile.timestamp_axes);
    push_missing_axis(&mut missing, "locale", &profile.locale_axes);
    push_missing_axis(&mut missing, "umask", &profile.umask_axes);
    push_missing_axis(&mut missing, "temp-path", &profile.temp_path_axes);
    push_missing_axis(&mut missing, "store-path", &profile.store_path_axes);
    push_missing_axis(&mut missing, "randomness", &profile.randomness_axes);
    missing
}

fn push_missing_axis(missing: &mut Vec<String>, axis: &str, values: &[String]) {
    if values.is_empty() {
        missing.push(axis.to_string());
    }
}

fn canonicalize_hermeticity_cell_report(cell: &mut HermeticityCellReport) -> Result<(), ReleaseEvidenceError> {
    validate_non_empty_string(&cell.cell_id, "hermeticity cell report cell_id")?;
    validate_optional_digest(&cell.output_digest_blake3, "hermeticity cell output_digest_blake3")?;
    canonicalize_blockers(&mut cell.blockers)?;
    canonicalize_string_vec(&mut cell.non_claims, "hermeticity cell non_claims")?;
    Ok(())
}

fn canonicalize_bootstrap_pressure_cell_report(
    cell: &mut BootstrapPressureCellReport,
) -> Result<(), ReleaseEvidenceError> {
    validate_non_empty_string(&cell.cell_id, "bootstrap pressure cell_id")?;
    validate_non_empty_string(&cell.profile, "bootstrap pressure profile")?;
    validate_non_empty_string(&cell.host_tool_policy, "bootstrap pressure host_tool_policy")?;
    validate_optional_digest(&cell.seed_inventory_digest_blake3, "seed_inventory_digest_blake3")?;
    validate_optional_digest(&cell.protected_exec_audit_digest_blake3, "protected_exec_audit_digest_blake3")?;
    canonicalize_digest_vec(&mut cell.stage_output_digest_set_blake3, "stage_output_digest_set_blake3")?;
    canonicalize_string_vec(&mut cell.remaining_trusted_root, "remaining_trusted_root")?;
    for observed in &cell.observed_execs {
        validate_observed_exec(observed)?;
    }
    cell.observed_execs.sort_by(|left, right| left.path.cmp(&right.path));
    canonicalize_blockers(&mut cell.blockers)?;
    canonicalize_string_vec(&mut cell.non_claims, "bootstrap pressure cell non_claims")?;
    Ok(())
}

fn canonicalize_substitution_case_report(case: &mut SubstitutionAttackCaseReport) -> Result<(), ReleaseEvidenceError> {
    validate_non_empty_string(&case.case_id, "substitution attack case_id")?;
    validate_optional_string(&case.trusted_key_material, "trusted_key_material")?;
    validate_optional_string(&case.fallback_mode, "fallback_mode")?;
    validate_optional_digest(&case.expected_digest_blake3, "expected_digest_blake3")?;
    validate_optional_digest(&case.observed_digest_blake3, "observed_digest_blake3")?;
    canonicalize_string_vec(&mut case.failed_trust_edges, "failed_trust_edges")?;
    canonicalize_blockers(&mut case.blockers)?;
    canonicalize_string_vec(&mut case.non_claims, "substitution attack non_claims")?;
    Ok(())
}

fn canonicalize_nix_mantle_case_report(case: &mut NixMantleComparisonCaseReport) -> Result<(), ReleaseEvidenceError> {
    validate_non_empty_string(&case.case_id, "comparison case_id")?;
    validate_non_empty_string(&case.build_recipe_identity, "build_recipe_identity")?;
    validate_non_empty_string(&case.normalization_policy, "normalization_policy")?;
    canonicalize_string_vec(&mut case.source_refs, "source_refs")?;
    canonicalize_string_vec(&mut case.toolchain_refs, "toolchain_refs")?;
    canonicalize_string_vec(&mut case.dependency_refs, "dependency_refs")?;
    canonicalize_string_vec(&mut case.output_surfaces, "output_surfaces")?;
    canonicalize_string_vec(&mut case.allowed_differences, "allowed_differences")?;
    canonicalize_digest_vec(&mut case.nix_digest_set_blake3, "nix_digest_set_blake3")?;
    canonicalize_digest_vec(&mut case.mantle_digest_set_blake3, "mantle_digest_set_blake3")?;
    canonicalize_blockers(&mut case.blockers)?;
    canonicalize_string_vec(&mut case.non_claims, "Nix Mantle case non_claims")?;
    Ok(())
}

fn canonicalize_repeatability_profile(
    mut profile: RepeatabilityMatrixProfile,
) -> Result<RepeatabilityMatrixProfile, ReleaseEvidenceError> {
    validate_non_empty_string(&profile.release_id, "repeatability release_id")?;
    validate_blake3_hex(&profile.matrix_profile_digest_blake3, "matrix_profile_digest_blake3")?;
    if profile.run_count < MIN_REPEATABILITY_RUN_COUNT {
        return Err(validation_error(format!(
            "repeatability run_count must be at least {MIN_REPEATABILITY_RUN_COUNT}"
        )));
    }
    canonicalize_string_vec(&mut profile.artifact_surfaces, "artifact_surfaces")?;
    canonicalize_digest_vec(&mut profile.expected_output_digest_set_blake3, "expected_output_digest_set_blake3")?;
    canonicalize_string_vec(&mut profile.cache_modes, "cache_modes")?;
    canonicalize_string_vec(&mut profile.store_isolation_modes, "store_isolation_modes")?;
    canonicalize_string_vec(&mut profile.environment_controls, "environment_controls")?;
    canonicalize_string_vec(&mut profile.temp_root_controls, "temp_root_controls")?;
    canonicalize_string_vec(&mut profile.user_controls, "user_controls")?;
    canonicalize_string_vec(&mut profile.host_classes, "host_classes")?;
    Ok(profile)
}

fn canonicalize_repeatability_cell_report(
    cell: &mut RepeatabilityMatrixCellReport,
) -> Result<(), ReleaseEvidenceError> {
    validate_non_empty_string(&cell.cell_id, "repeatability cell_id")?;
    validate_non_empty_string(&cell.cache_mode, "repeatability cache_mode")?;
    validate_non_empty_string(&cell.store_isolation_mode, "repeatability store_isolation_mode")?;
    validate_optional_string(&cell.output_root_identity, "output_root_identity")?;
    validate_optional_string(&cell.store_root_identity, "store_root_identity")?;
    canonicalize_digest_vec(&mut cell.expected_output_digest_set_blake3, "expected_output_digest_set_blake3")?;
    canonicalize_digest_vec(&mut cell.observed_output_digest_set_blake3, "observed_output_digest_set_blake3")?;
    canonicalize_string_vec(&mut cell.missing_outputs, "missing_outputs")?;
    canonicalize_blockers(&mut cell.blockers)?;
    canonicalize_string_vec(&mut cell.non_claims, "repeatability cell non_claims")?;
    Ok(())
}

fn canonicalize_continuous_context(
    mut context: ContinuousGauntletContext,
) -> Result<ContinuousGauntletContext, ReleaseEvidenceError> {
    validate_blake3_hex(&context.source_digest_blake3, DIGEST_FIELD_SOURCE)?;
    validate_blake3_hex(&context.policy_digest_blake3, DIGEST_FIELD_POLICY)?;
    validate_blake3_hex(&context.universe_digest_blake3, DIGEST_FIELD_UNIVERSE)?;
    validate_blake3_hex(&context.toolchain_digest_blake3, DIGEST_FIELD_TOOLCHAIN)?;
    validate_blake3_hex(&context.witness_set_digest_blake3, DIGEST_FIELD_WITNESS_SET)?;
    validate_non_empty_string(&context.host_class, "continuous host_class")?;
    context.required_track_schema_versions.sort_by(|left, right| left.track_id.cmp(&right.track_id));
    for version in &context.required_track_schema_versions {
        validate_non_empty_string(&version.track_id, "required track_id")?;
        validate_non_empty_string(&version.schema_version, "required track schema_version")?;
    }
    Ok(context)
}

fn required_track_versions(
    context: &ContinuousGauntletContext,
) -> Result<BTreeMap<String, String>, ReleaseEvidenceError> {
    let mut versions = BTreeMap::new();
    for version in &context.required_track_schema_versions {
        if versions.insert(version.track_id.clone(), version.schema_version.clone()).is_some() {
            return Err(validation_error(format!("duplicate required track schema version: {}", version.track_id)));
        }
    }
    Ok(versions)
}

fn validate_track_evidence(track: &mut GauntletTrackEvidence) -> Result<(), ReleaseEvidenceError> {
    validate_non_empty_string(&track.track_id, "track_id")?;
    validate_non_empty_string(&track.track_schema_version, "track_schema_version")?;
    validate_blake3_hex(&track.report_digest_blake3, DIGEST_FIELD_REPORT)?;
    validate_blake3_hex(&track.source_digest_blake3, DIGEST_FIELD_SOURCE)?;
    validate_blake3_hex(&track.policy_digest_blake3, DIGEST_FIELD_POLICY)?;
    validate_blake3_hex(&track.universe_digest_blake3, DIGEST_FIELD_UNIVERSE)?;
    validate_blake3_hex(&track.toolchain_digest_blake3, DIGEST_FIELD_TOOLCHAIN)?;
    validate_blake3_hex(&track.witness_set_digest_blake3, DIGEST_FIELD_WITNESS_SET)?;
    validate_non_empty_string(&track.host_class, "track host_class")?;
    track.runs.sort_by(|left, right| left.run_id.cmp(&right.run_id));
    Ok(())
}

fn validate_track_run(run: &TrackRunEvidence) -> Result<(), ReleaseEvidenceError> {
    validate_non_empty_string(&run.run_id, "track run_id")?;
    validate_string_slice(&run.blockers, "track run blockers")?;
    validate_string_slice(&run.next_actions, "track run next_actions")?;
    Ok(())
}

fn push_run_blockers(run: &TrackRunEvidence, blockers: &mut Vec<GauntletBlocker>) {
    for message in &run.blockers {
        blockers.push(blocker(
            "track-run-blocker",
            message,
            "complete the track-specific next action before promoting the aggregate",
            None,
            None,
            None,
        ));
    }
}

fn canonicalize_track_report(track: &mut TrackAggregateReport) -> Result<(), ReleaseEvidenceError> {
    validate_non_empty_string(&track.track_id, "track report track_id")?;
    validate_non_empty_string(&track.track_schema_version, "track report schema_version")?;
    validate_blake3_hex(&track.report_digest_blake3, DIGEST_FIELD_REPORT)?;
    validate_non_empty_string(&track.first_run_id, "track first_run_id")?;
    validate_non_empty_string(&track.last_run_id, "track last_run_id")?;
    canonicalize_blockers(&mut track.blockers)?;
    canonicalize_string_vec(&mut track.next_actions, "track next_actions")?;
    Ok(())
}

fn validate_observed_exec(observed: &ProtectedExecObservation) -> Result<(), ReleaseEvidenceError> {
    validate_non_empty_string(&observed.path, "observed exec path")?;
    validate_non_empty_string(&observed.executable_class, "observed exec class")?;
    validate_blake3_hex(&observed.digest_blake3, "observed exec digest_blake3")?;
    Ok(())
}

fn profile_requires_protected_exec(profile: &str) -> bool {
    matches!(
        profile,
        PROFILE_NO_HOST_TOOLS | PROFILE_SOURCE_BUILT_PROVIDER | PROFILE_REDUCED_SEED | PROFILE_FULL_SOURCE_ROOT_ATTEMPT
    )
}

fn canonicalize_violations(violations: &mut Vec<HermeticityViolation>) -> Result<(), ReleaseEvidenceError> {
    validate_collection_limit(violations.len(), MAX_GAUNTLET_BLOCKER_COUNT, "hermeticity violations")?;
    for violation in violations.iter() {
        validate_non_empty_string(&violation.class, "hermeticity violation class")?;
        validate_non_empty_string(&violation.detail, "hermeticity violation detail")?;
        validate_optional_string(&violation.axis, "hermeticity violation axis")?;
    }
    violations.sort_by(|left, right| {
        (&left.class, &left.axis, &left.detail).cmp(&(&right.class, &right.axis, &right.detail))
    });
    Ok(())
}

fn unsupported_axis_non_claims(axes: &[String]) -> Vec<String> {
    axes.iter().map(|axis| format!("{NON_CLAIM_UNTESTED_AXIS_PREFIX}:{axis}")).collect()
}

fn collect_cell_blockers<'a, I>(blocker_sets: I) -> Result<Vec<GauntletBlocker>, ReleaseEvidenceError>
where I: IntoIterator<Item = &'a Vec<GauntletBlocker>> {
    let mut blockers = Vec::new();
    for set in blocker_sets {
        blockers.extend(set.iter().cloned());
    }
    canonicalize_blockers(&mut blockers)?;
    Ok(blockers)
}

fn canonicalize_axis_vec(values: &mut Vec<String>, field_name: &str) -> Result<(), ReleaseEvidenceError> {
    canonicalize_string_vec(values, field_name)?;
    validate_collection_limit(values.len(), MAX_GAUNTLET_AXIS_COUNT, field_name)
}

fn canonicalize_digest_vec(values: &mut Vec<String>, field_name: &str) -> Result<(), ReleaseEvidenceError> {
    values.sort();
    values.dedup();
    validate_collection_limit(values.len(), MAX_GAUNTLET_REPORT_DIGEST_COUNT, field_name)?;
    for value in values.iter() {
        validate_blake3_hex(value, field_name)?;
    }
    Ok(())
}

fn canonicalize_string_vec(values: &mut Vec<String>, field_name: &str) -> Result<(), ReleaseEvidenceError> {
    values.sort();
    values.dedup();
    validate_string_slice(values, field_name)
}

fn validate_string_slice(values: &[String], field_name: &str) -> Result<(), ReleaseEvidenceError> {
    validate_collection_limit(values.len(), MAX_GAUNTLET_STRING_SET_COUNT, field_name)?;
    for value in values {
        validate_non_empty_string(value, field_name)?;
    }
    Ok(())
}

fn canonicalize_blockers(blockers: &mut Vec<GauntletBlocker>) -> Result<(), ReleaseEvidenceError> {
    validate_collection_limit(blockers.len(), MAX_GAUNTLET_BLOCKER_COUNT, "gauntlet blockers")?;
    for blocker in blockers.iter() {
        validate_blocker(blocker)?;
    }
    blockers.sort_by(|left, right| {
        (
            &left.evidence_class,
            &left.axis,
            &left.expected_digest_blake3,
            &left.observed_digest_blake3,
            &left.message,
            &left.next_action,
        )
            .cmp(&(
                &right.evidence_class,
                &right.axis,
                &right.expected_digest_blake3,
                &right.observed_digest_blake3,
                &right.message,
                &right.next_action,
            ))
    });
    blockers.dedup();
    Ok(())
}

fn validate_blocker(blocker: &GauntletBlocker) -> Result<(), ReleaseEvidenceError> {
    validate_non_empty_string(&blocker.evidence_class, "blocker.evidence_class")?;
    validate_non_empty_string(&blocker.message, "blocker.message")?;
    validate_non_empty_string(&blocker.next_action, "blocker.next_action")?;
    validate_optional_string(&blocker.axis, "blocker.axis")?;
    validate_optional_digest(&blocker.expected_digest_blake3, "blocker.expected_digest_blake3")?;
    validate_optional_digest(&blocker.observed_digest_blake3, "blocker.observed_digest_blake3")?;
    Ok(())
}

fn validate_optional_digest(value: &Option<String>, field_name: &str) -> Result<(), ReleaseEvidenceError> {
    if let Some(value) = value {
        validate_blake3_hex(value, field_name)?;
    }
    Ok(())
}

fn validate_optional_string(value: &Option<String>, field_name: &str) -> Result<(), ReleaseEvidenceError> {
    if let Some(value) = value {
        validate_non_empty_string(value, field_name)?;
    }
    Ok(())
}

fn validate_non_empty_string(value: &str, field_name: &str) -> Result<(), ReleaseEvidenceError> {
    if value.is_empty() {
        return Err(validation_error(format!("{field_name} must not be empty")));
    }
    let byte_count = u32_count(value.len(), &format!("{field_name} length overflowed u32"))?;
    if byte_count > MAX_GAUNTLET_STRING_BYTES_COUNT {
        return Err(validation_error(format!("{field_name} exceeds {MAX_GAUNTLET_STRING_BYTES_COUNT} bytes")));
    }
    Ok(())
}

fn validate_collection_limit(count: usize, limit: u32, field_name: &str) -> Result<(), ReleaseEvidenceError> {
    let count_u32 = u32_count(count, &format!("{field_name} count overflowed u32"))?;
    if count_u32 > limit {
        return Err(validation_error(format!("{field_name} has {count_u32} entries, limit is {limit}")));
    }
    Ok(())
}

fn validate_report_cell_count(count: usize, field_name: &str) -> Result<(), ReleaseEvidenceError> {
    let count_u32 = u32_count(count, &format!("{field_name} count overflowed u32"))?;
    if count_u32 == ZERO_COUNT {
        return Err(validation_error(format!("{field_name} must include at least one entry")));
    }
    if count_u32 > MAX_GAUNTLET_CELL_COUNT {
        return Err(validation_error(format!(
            "{field_name} has {count_u32} entries, limit is {MAX_GAUNTLET_CELL_COUNT}"
        )));
    }
    Ok(())
}

fn validate_continuous_track_count(count: usize) -> Result<(), ReleaseEvidenceError> {
    let count_u32 = u32_count(count, "continuous track count overflowed u32")?;
    if count_u32 == ZERO_COUNT {
        return Err(validation_error("continuous tracks must include at least one entry".to_string()));
    }
    if count_u32 > MAX_GAUNTLET_TRACK_COUNT {
        return Err(validation_error(format!(
            "continuous tracks has {count_u32} entries, limit is {MAX_GAUNTLET_TRACK_COUNT}"
        )));
    }
    Ok(())
}

fn require_schema(actual: &str, expected: &str, field_name: &str) -> Result<(), ReleaseEvidenceError> {
    if actual != expected {
        return Err(validation_error(format!("{field_name} must be {expected}, got {actual}")));
    }
    Ok(())
}

fn serialize_canonical<T: Serialize>(value: &T, label: &str) -> Result<Vec<u8>, ReleaseEvidenceError> {
    serde_json::to_vec(value).map_err(|err| ReleaseEvidenceError::Parse(format!("serializing {label}: {err}")))
}

fn blocker(
    evidence_class: &str,
    message: &str,
    next_action: &str,
    axis: Option<String>,
    expected_digest_blake3: Option<String>,
    observed_digest_blake3: Option<String>,
) -> GauntletBlocker {
    GauntletBlocker {
        evidence_class: evidence_class.to_string(),
        message: message.to_string(),
        next_action: next_action.to_string(),
        axis,
        expected_digest_blake3,
        observed_digest_blake3,
    }
}

fn digest_options_mismatch(expected: &Option<String>, observed: &Option<String>) -> bool {
    match (expected, observed) {
        (Some(expected), Some(observed)) => expected != observed,
        (Some(_), None) => true,
        _ => false,
    }
}

fn digest_vecs_match(left: &[String], right: &[String]) -> bool {
    let left_set = left.iter().collect::<BTreeSet<_>>();
    let right_set = right.iter().collect::<BTreeSet<_>>();
    left_set == right_set && !left_set.is_empty()
}

fn first_digest(values: &[String]) -> Option<String> {
    values.first().cloned()
}

#[cfg(test)]
mod tests {
    use alloc::string::ToString;
    use alloc::vec;

    use pretty_assertions::assert_eq;

    use super::*;

    const DIGEST_ONE_SEED: u8 = 1;
    const DIGEST_TWO_SEED: u8 = 2;
    const DIGEST_THREE_SEED: u8 = 3;
    const DIGEST_FOUR_SEED: u8 = 4;
    const RUN_COUNT_TWO: u32 = 2;

    fn digest(seed: u8) -> String {
        let byte = format!("{:x}", seed % crate::manifest::BLAKE3_HEX_LENGTH_CHARS as u8);
        byte.repeat(crate::manifest::BLAKE3_HEX_LENGTH_CHARS)
    }

    fn full_hermeticity_profile() -> HermeticityGauntletProfile {
        HermeticityGauntletProfile {
            host_tool_axes: vec!["hidden-host-exec".to_string()],
            environment_axes: vec!["ambient-env".to_string()],
            network_axes: vec!["network-denied".to_string()],
            timestamp_axes: vec!["timestamp".to_string()],
            locale_axes: vec!["locale".to_string()],
            umask_axes: vec!["umask".to_string()],
            temp_path_axes: vec!["temp-path".to_string()],
            store_path_axes: vec!["store-path".to_string()],
            randomness_axes: vec!["randomness".to_string()],
        }
    }

    #[test]
    fn hermeticity_gauntlet_accepts_clean_strict_cell() {
        let report = evaluate_adversarial_hermeticity_gauntlet(
            "run-a".to_string(),
            digest(DIGEST_ONE_SEED),
            full_hermeticity_profile(),
            vec![HermeticityCellEvidence {
                cell_id: "clean".to_string(),
                mode: GauntletMode::Strict,
                observed_violations: Vec::new(),
                audit_events: Vec::new(),
                unsupported_axes: Vec::new(),
                output_digest_blake3: Some(digest(DIGEST_TWO_SEED)),
            }],
        )
        .unwrap();

        assert!(report.strict_evidence_eligible);
        assert_eq!(report.cells[0].verdict, GauntletVerdict::Accepted);
        assert!(report.blockers.is_empty());
    }

    #[test]
    fn hermeticity_gauntlet_blocks_strict_violation() {
        let report = evaluate_adversarial_hermeticity_gauntlet(
            "run-b".to_string(),
            digest(DIGEST_ONE_SEED),
            full_hermeticity_profile(),
            vec![HermeticityCellEvidence {
                cell_id: "hidden-exec".to_string(),
                mode: GauntletMode::Strict,
                observed_violations: vec![HermeticityViolation {
                    class: "host-tool".to_string(),
                    detail: "undeclared cc executed".to_string(),
                    axis: Some("hidden-host-exec".to_string()),
                }],
                audit_events: Vec::new(),
                unsupported_axes: Vec::new(),
                output_digest_blake3: None,
            }],
        )
        .unwrap();

        assert!(!report.strict_evidence_eligible);
        assert_eq!(report.cells[0].verdict, GauntletVerdict::Blocked);
        assert_eq!(report.blockers[0].evidence_class, "hermeticity-violation");
    }

    #[test]
    fn bootstrap_pressure_blocks_undeclared_protected_exec() {
        let report = evaluate_bootstrap_pressure_gauntlet("run-bootstrap".to_string(), digest(DIGEST_ONE_SEED), vec![
            BootstrapPressureCellEvidence {
                cell_id: "no-host-tools".to_string(),
                profile: PROFILE_NO_HOST_TOOLS.to_string(),
                fixed_point: true,
                host_tool_policy: "deny-host-tools".to_string(),
                seed_inventory_digest_blake3: Some(digest(DIGEST_TWO_SEED)),
                protected_exec_audit_digest_blake3: Some(digest(DIGEST_THREE_SEED)),
                stage_output_digest_set_blake3: vec![digest(DIGEST_FOUR_SEED)],
                observed_execs: vec![ProtectedExecObservation {
                    path: "/usr/bin/gcc".to_string(),
                    executable_class: "compiler".to_string(),
                    declared: false,
                    digest_blake3: digest(DIGEST_FOUR_SEED),
                }],
                remaining_trusted_root: Vec::new(),
                unsupported_reason: None,
            },
        ])
        .unwrap();

        assert_eq!(report.cells[0].verdict, GauntletVerdict::Blocked);
        assert_eq!(report.blockers[0].evidence_class, "undeclared-protected-exec");
    }

    #[test]
    fn substitution_attack_accepts_trusted_case_and_rejects_malicious_strict_case() {
        let report =
            evaluate_substitution_cache_attack_gauntlet("run-cache".to_string(), digest(DIGEST_ONE_SEED), vec![
                SubstitutionAttackCaseEvidence {
                    case_id: "trusted".to_string(),
                    strict_mode: true,
                    accepted: true,
                    closure_complete: true,
                    artifact_attestation_fresh: true,
                    trusted_key_material: Some("builder:pub".to_string()),
                    fallback_mode: None,
                    expected_digest_blake3: Some(digest(DIGEST_TWO_SEED)),
                    observed_digest_blake3: Some(digest(DIGEST_TWO_SEED)),
                    failed_trust_edges: Vec::new(),
                },
                SubstitutionAttackCaseEvidence {
                    case_id: "bad-signature".to_string(),
                    strict_mode: true,
                    accepted: false,
                    closure_complete: true,
                    artifact_attestation_fresh: true,
                    trusted_key_material: Some("attacker:pub".to_string()),
                    fallback_mode: None,
                    expected_digest_blake3: Some(digest(DIGEST_TWO_SEED)),
                    observed_digest_blake3: Some(digest(DIGEST_TWO_SEED)),
                    failed_trust_edges: vec!["wrong-key".to_string()],
                },
            ])
            .unwrap();

        assert_eq!(report.cases[0].verdict, GauntletVerdict::Rejected);
        assert_eq!(report.cases[1].verdict, GauntletVerdict::Accepted);
        assert!(report.blockers.is_empty());
    }

    #[test]
    fn substitution_attack_blocks_malicious_acceptance() {
        let report =
            evaluate_substitution_cache_attack_gauntlet("run-cache-bad".to_string(), digest(DIGEST_ONE_SEED), vec![
                SubstitutionAttackCaseEvidence {
                    case_id: "corrupt-accepted".to_string(),
                    strict_mode: true,
                    accepted: true,
                    closure_complete: false,
                    artifact_attestation_fresh: false,
                    trusted_key_material: Some("builder:pub".to_string()),
                    fallback_mode: None,
                    expected_digest_blake3: Some(digest(DIGEST_TWO_SEED)),
                    observed_digest_blake3: Some(digest(DIGEST_THREE_SEED)),
                    failed_trust_edges: vec!["nar-digest".to_string()],
                },
            ])
            .unwrap();

        assert_eq!(report.cases[0].verdict, GauntletVerdict::Blocked);
        assert_eq!(report.blockers[0].evidence_class, "malicious-cache-accepted");
    }

    #[test]
    fn nix_mantle_comparison_matches_equivalent_digest_sets() {
        let report = evaluate_nix_mantle_comparison_corpus("run-compare".to_string(), digest(DIGEST_ONE_SEED), vec![
            comparison_case(true, None, vec![digest(DIGEST_TWO_SEED)], vec![digest(DIGEST_TWO_SEED)]),
        ])
        .unwrap();

        assert_eq!(report.cases[0].verdict, GauntletVerdict::Matched);
        assert!(report.blockers.is_empty());
    }

    #[test]
    fn nix_mantle_comparison_blocks_missing_equivalence_policy() {
        let report =
            evaluate_nix_mantle_comparison_corpus("run-compare-bad".to_string(), digest(DIGEST_ONE_SEED), vec![
                comparison_case(false, None, vec![digest(DIGEST_TWO_SEED)], vec![digest(DIGEST_TWO_SEED)]),
            ])
            .unwrap();

        assert_eq!(report.cases[0].verdict, GauntletVerdict::Blocked);
        assert_eq!(report.blockers[0].evidence_class, "missing-equivalence-policy");
    }

    #[test]
    fn repeatability_matrix_matches_fresh_isolated_cell() {
        let profile = repeatability_profile();
        let report =
            evaluate_release_repeatability_matrix(profile.clone(), vec![repeatability_cell(&profile, true)]).unwrap();

        assert!(report.repeatability_evidence_eligible);
        assert_eq!(report.cells[0].verdict, GauntletVerdict::Matched);
    }

    #[test]
    fn repeatability_matrix_blocks_reused_fresh_store() {
        let profile = repeatability_profile();
        let mut cell = repeatability_cell(&profile, true);
        cell.reused_store = true;
        let report = evaluate_release_repeatability_matrix(profile, vec![cell]).unwrap();

        assert!(!report.repeatability_evidence_eligible);
        assert_eq!(report.cells[0].verdict, GauntletVerdict::Blocked);
        assert_eq!(report.blockers[0].evidence_class, "reused-store");
    }

    #[test]
    fn continuous_gauntlet_accepts_current_track() {
        let context = continuous_context();
        let report = evaluate_continuous_reproducibility_gauntlet(context.clone(), vec![track_evidence(
            &context,
            GauntletVerdict::Matched,
        )])
        .unwrap();

        assert_eq!(report.current_claim_status, GauntletVerdict::Accepted);
        assert_eq!(report.tracks[0].status, GauntletVerdict::Matched);
        assert!(report.blockers.is_empty());
    }

    #[test]
    fn continuous_gauntlet_marks_stale_source_digest() {
        let context = continuous_context();
        let mut track = track_evidence(&context, GauntletVerdict::Matched);
        track.source_digest_blake3 = digest(DIGEST_FOUR_SEED);
        let report = evaluate_continuous_reproducibility_gauntlet(context, vec![track]).unwrap();

        assert_eq!(report.tracks[0].status, GauntletVerdict::Stale);
        assert_eq!(report.current_claim_status, GauntletVerdict::Blocked);
        assert_eq!(report.blockers[0].evidence_class, "track-boundary-stale");
    }

    #[test]
    fn canonical_bytes_are_stable_for_equivalent_continuous_reports() {
        let context = continuous_context();
        let mut first = evaluate_continuous_reproducibility_gauntlet(context.clone(), vec![track_evidence(
            &context,
            GauntletVerdict::Matched,
        )])
        .unwrap();
        let second = first.clone();
        first.tracks.reverse();

        assert_eq!(
            continuous_reproducibility_gauntlet_report_canonical_bytes(first).unwrap(),
            continuous_reproducibility_gauntlet_report_canonical_bytes(second).unwrap()
        );
    }

    #[test]
    fn canonical_report_rejects_malformed_digest() {
        let context = continuous_context();
        let mut report = evaluate_continuous_reproducibility_gauntlet(context.clone(), vec![track_evidence(
            &context,
            GauntletVerdict::Matched,
        )])
        .unwrap();
        report.track_report_digests_blake3[0] = "ABC".to_string();

        let err = continuous_reproducibility_gauntlet_report_canonical_bytes(report).unwrap_err();

        assert!(err.to_string().contains("lowercase hex") || err.to_string().contains("64 lowercase"));
    }

    fn comparison_case(
        equivalence_declared: bool,
        unsupported_reason: Option<String>,
        nix_digest_set_blake3: Vec<String>,
        mantle_digest_set_blake3: Vec<String>,
    ) -> NixMantleComparisonCaseEvidence {
        NixMantleComparisonCaseEvidence {
            case_id: "hello".to_string(),
            equivalence_declared,
            build_recipe_identity: "hello-v1".to_string(),
            normalization_policy: "nar-content".to_string(),
            source_refs: vec!["src".to_string()],
            toolchain_refs: vec!["toolchain".to_string()],
            dependency_refs: vec!["dep".to_string()],
            output_surfaces: vec!["out".to_string()],
            allowed_differences: vec!["store-prefix".to_string()],
            nix_digest_set_blake3,
            mantle_digest_set_blake3,
            unsupported_reason,
        }
    }

    fn repeatability_profile() -> RepeatabilityMatrixProfile {
        RepeatabilityMatrixProfile {
            release_id: "mantle-test".to_string(),
            matrix_profile_digest_blake3: digest(DIGEST_ONE_SEED),
            run_count: RUN_COUNT_TWO,
            artifact_surfaces: vec!["bin/mantle".to_string()],
            expected_output_digest_set_blake3: vec![digest(DIGEST_TWO_SEED)],
            cache_modes: vec!["no-substitute".to_string()],
            store_isolation_modes: vec!["fresh".to_string()],
            environment_controls: vec!["scrubbed".to_string()],
            temp_root_controls: vec!["fresh-temp".to_string()],
            user_controls: vec!["same-user".to_string()],
            host_classes: vec!["nixos".to_string()],
        }
    }

    fn repeatability_cell(profile: &RepeatabilityMatrixProfile, fresh_store: bool) -> RepeatabilityMatrixCellEvidence {
        RepeatabilityMatrixCellEvidence {
            cell_id: "cell-a".to_string(),
            cache_mode: "no-substitute".to_string(),
            store_isolation_mode: "fresh".to_string(),
            fresh_store,
            explicit_reuse_test: false,
            reused_store: false,
            output_root_identity: Some("out-a".to_string()),
            store_root_identity: Some("store-a".to_string()),
            expected_output_digest_set_blake3: profile.expected_output_digest_set_blake3.clone(),
            observed_output_digest_set_blake3: profile.expected_output_digest_set_blake3.clone(),
            missing_outputs: Vec::new(),
            unsupported_reason: None,
        }
    }

    fn continuous_context() -> ContinuousGauntletContext {
        ContinuousGauntletContext {
            source_digest_blake3: digest(DIGEST_ONE_SEED),
            policy_digest_blake3: digest(DIGEST_TWO_SEED),
            universe_digest_blake3: digest(DIGEST_THREE_SEED),
            toolchain_digest_blake3: digest(DIGEST_FOUR_SEED),
            host_class: "nixos".to_string(),
            witness_set_digest_blake3: digest(DIGEST_ONE_SEED),
            required_track_schema_versions: vec![TrackSchemaVersion {
                track_id: "repeatability".to_string(),
                schema_version: RELEASE_REPEATABILITY_MATRIX_REPORT_SCHEMA.to_string(),
            }],
        }
    }

    fn track_evidence(context: &ContinuousGauntletContext, status: GauntletVerdict) -> GauntletTrackEvidence {
        GauntletTrackEvidence {
            track_id: "repeatability".to_string(),
            track_schema_version: RELEASE_REPEATABILITY_MATRIX_REPORT_SCHEMA.to_string(),
            report_digest_blake3: digest(DIGEST_TWO_SEED),
            source_digest_blake3: context.source_digest_blake3.clone(),
            policy_digest_blake3: context.policy_digest_blake3.clone(),
            universe_digest_blake3: context.universe_digest_blake3.clone(),
            toolchain_digest_blake3: context.toolchain_digest_blake3.clone(),
            host_class: context.host_class.clone(),
            witness_set_digest_blake3: context.witness_set_digest_blake3.clone(),
            runs: vec![TrackRunEvidence {
                run_id: "run-1".to_string(),
                status,
                blockers: Vec::new(),
                next_actions: Vec::new(),
            }],
        }
    }
}
