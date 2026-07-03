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

use crate::ReleaseEvidenceError;
use crate::manifest::u32_count;
use crate::manifest::validate_blake3_hex;
use crate::manifest::validation_error;

pub const GLOBAL_REPRODUCIBILITY_UNIVERSE_SCHEMA: &str = "mantle-global-reproducibility-universe-v1";
pub const GLOBAL_REPRODUCIBILITY_POLICY_SCHEMA: &str = "mantle-global-reproducibility-policy-v1";
pub const GLOBAL_REPRODUCIBILITY_SURFACE_EVIDENCE_SCHEMA: &str = "mantle-global-reproducibility-surface-evidence-v1";
pub const GLOBAL_REPRODUCIBILITY_REPORT_SCHEMA: &str = "mantle-global-reproducibility-report-v1";

const MAX_GLOBAL_SURFACE_COUNT: u32 = 256;
const MAX_EXCLUDED_SURFACE_COUNT: u32 = 256;
const MAX_GLOBAL_WITNESS_COUNT: u32 = 64;
const MAX_GLOBAL_DIGEST_COUNT: u32 = 256;
const MAX_GLOBAL_PERTURBATION_AXIS_COUNT: u32 = 64;
const MAX_GLOBAL_STRING_SET_COUNT: u32 = 512;
const MAX_GLOBAL_STRING_BYTES_COUNT: u32 = 1024;
const MAX_GLOBAL_BLOCKER_COUNT: u32 = 512;
const MIN_REQUIRED_WITNESS_COUNT: u32 = 1;
const ZERO_COUNT: u32 = 0;

const NON_CLAIM_FUTURE_CODE: &str = "not-future-code";
const NON_CLAIM_UNDECLARED_FRONTENDS: &str = "not-undeclared-frontends";
const NON_CLAIM_UNDECLARED_TARGET_SYSTEMS: &str = "not-undeclared-target-systems";
const NON_CLAIM_COMPILER_CORRECTNESS: &str = "not-compiler-correctness";
const NON_CLAIM_DEPLOY_SUCCESS: &str = "not-deploy-success";
const NON_CLAIM_PHYSICAL_TARGET_DETERMINISM: &str = "not-physical-target-determinism";
const NON_CLAIM_GLOBAL_BLOCKED: &str = "global-reproducibility-blocked";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GlobalReproducibilityUniverse {
    pub schema: String,
    pub name: String,
    pub included_surfaces: Vec<GlobalBuildSurface>,
    #[serde(default)]
    pub excluded_surfaces: Vec<GlobalExcludedSurface>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GlobalBuildSurface {
    pub id: String,
    pub target_system: String,
    pub source_acquisition_mode: String,
    pub toolchain_route: String,
    pub cache_substitution_mode: String,
    pub release_artifact_set: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GlobalExcludedSurface {
    pub id: String,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GlobalReproducibilityPolicy {
    pub schema: String,
    pub policy_id: String,
    pub witness_policy: String,
    pub required_operator_domains: u32,
    pub required_host_classes: u32,
    #[serde(default)]
    pub required_perturbation_axes: Vec<String>,
    pub require_strict_hermeticity: bool,
    pub require_fresh_rebuild_store: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GlobalSurfaceEvidence {
    pub schema: String,
    pub surface_id: String,
    pub universe_digest_blake3: String,
    pub policy_digest_blake3: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub action_receipt_digest_blake3: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_acquisition_digest_blake3: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub toolchain_provenance_digest_blake3: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hermeticity_evidence_digest_blake3: Option<String>,
    #[serde(default)]
    pub output_digest_set_blake3: Vec<String>,
    pub strict_hermeticity: bool,
    pub fresh_rebuild_store: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unsupported_reason: Option<String>,
    #[serde(default)]
    pub witnesses: Vec<GlobalWitnessEvidence>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GlobalWitnessEvidence {
    pub identity: String,
    pub operator_domain: String,
    pub host_class: String,
    #[serde(default)]
    pub perturbation_axes: Vec<String>,
    pub trust_status: GlobalWitnessTrustStatus,
    #[serde(default)]
    pub output_digest_set_blake3: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum GlobalWitnessTrustStatus {
    Valid,
    UnknownKey,
    Revoked,
    InvalidSignature,
    PolicyInsufficient,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum GlobalReproducibilityClaimClass {
    Eligible,
    Blocked,
    NonGlobal,
}

impl GlobalReproducibilityClaimClass {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Eligible => "eligible",
            Self::Blocked => "blocked",
            Self::NonGlobal => "non-global",
        }
    }

    pub fn is_global_claim(self) -> bool {
        matches!(self, Self::Eligible)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GlobalWitnessCounts {
    pub accepted: u32,
    pub skipped_unknown_key: u32,
    pub skipped_revoked: u32,
    pub skipped_same_domain: u32,
    pub failed_invalid_signature: u32,
    pub failed_digest_mismatched: u32,
    pub policy_insufficient: u32,
}

impl GlobalWitnessCounts {
    fn empty() -> Self {
        Self {
            accepted: ZERO_COUNT,
            skipped_unknown_key: ZERO_COUNT,
            skipped_revoked: ZERO_COUNT,
            skipped_same_domain: ZERO_COUNT,
            failed_invalid_signature: ZERO_COUNT,
            failed_digest_mismatched: ZERO_COUNT,
            policy_insufficient: ZERO_COUNT,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GlobalReproducibilityBlocker {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub surface_id: Option<String>,
    pub evidence_class: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expected_digest_blake3: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub observed_digest_blake3: Option<String>,
    pub message: String,
    pub next_action: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GlobalSurfaceReport {
    pub surface_id: String,
    pub target_system: String,
    pub source_acquisition_mode: String,
    pub toolchain_route: String,
    pub cache_substitution_mode: String,
    pub release_artifact_set: String,
    pub output_digest_set_blake3: Vec<String>,
    pub accepted_witness_identities: Vec<String>,
    pub accepted_operator_domains: Vec<String>,
    pub accepted_host_classes: Vec<String>,
    pub covered_perturbation_axes: Vec<String>,
    pub witness_counts: GlobalWitnessCounts,
    pub blockers: Vec<GlobalReproducibilityBlocker>,
    pub non_claims: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GlobalReproducibilityReport {
    pub schema: String,
    pub universe_name: String,
    pub universe_digest_blake3: String,
    pub policy_id: String,
    pub policy_digest_blake3: String,
    pub witness_policy: String,
    pub included_surface_count: u32,
    pub claim_class: GlobalReproducibilityClaimClass,
    pub evidence_digests_blake3: Vec<String>,
    pub accepted_witness_identities: Vec<String>,
    pub surfaces: Vec<GlobalSurfaceReport>,
    pub blockers: Vec<GlobalReproducibilityBlocker>,
    pub non_claims: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GlobalReproducibilityEvaluationInput {
    pub universe: GlobalReproducibilityUniverse,
    pub policy: GlobalReproducibilityPolicy,
    pub universe_digest_blake3: String,
    pub policy_digest_blake3: String,
    pub surface_evidence: Vec<GlobalSurfaceEvidence>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SurfaceWitnessEvaluation {
    accepted_witness_identities: Vec<String>,
    accepted_operator_domains: Vec<String>,
    accepted_host_classes: Vec<String>,
    covered_perturbation_axes: Vec<String>,
    witness_counts: GlobalWitnessCounts,
    blockers: Vec<GlobalReproducibilityBlocker>,
}

pub fn canonical_global_reproducibility_universe(
    mut universe: GlobalReproducibilityUniverse,
) -> Result<GlobalReproducibilityUniverse, ReleaseEvidenceError> {
    validate_universe_header(&universe)?;
    universe.schema = GLOBAL_REPRODUCIBILITY_UNIVERSE_SCHEMA.to_string();
    universe.included_surfaces.sort_by(|left, right| left.id.cmp(&right.id));
    universe.excluded_surfaces.sort_by(|left, right| left.id.cmp(&right.id));
    validate_included_surfaces(&universe.included_surfaces)?;
    validate_excluded_surfaces(&universe.excluded_surfaces)?;
    Ok(universe)
}

pub fn global_reproducibility_universe_canonical_bytes(
    universe: GlobalReproducibilityUniverse,
) -> Result<Vec<u8>, ReleaseEvidenceError> {
    let canonical = canonical_global_reproducibility_universe(universe)?;
    serde_json::to_vec(&canonical)
        .map_err(|err| ReleaseEvidenceError::Parse(format!("serializing global reproducibility universe: {err}")))
}

pub fn global_reproducibility_universe_digest_blake3(
    universe: GlobalReproducibilityUniverse,
) -> Result<String, ReleaseEvidenceError> {
    let canonical = global_reproducibility_universe_canonical_bytes(universe)?;
    Ok(blake3::hash(&canonical).to_hex().to_string())
}

pub fn canonical_global_reproducibility_policy(
    mut policy: GlobalReproducibilityPolicy,
) -> Result<GlobalReproducibilityPolicy, ReleaseEvidenceError> {
    validate_policy_header(&policy)?;
    policy.schema = GLOBAL_REPRODUCIBILITY_POLICY_SCHEMA.to_string();
    policy.required_perturbation_axes.sort();
    validate_perturbation_axes(&policy.required_perturbation_axes)?;
    Ok(policy)
}

pub fn global_reproducibility_policy_canonical_bytes(
    policy: GlobalReproducibilityPolicy,
) -> Result<Vec<u8>, ReleaseEvidenceError> {
    let canonical = canonical_global_reproducibility_policy(policy)?;
    serde_json::to_vec(&canonical)
        .map_err(|err| ReleaseEvidenceError::Parse(format!("serializing global reproducibility policy: {err}")))
}

pub fn global_reproducibility_policy_digest_blake3(
    policy: GlobalReproducibilityPolicy,
) -> Result<String, ReleaseEvidenceError> {
    let canonical = global_reproducibility_policy_canonical_bytes(policy)?;
    Ok(blake3::hash(&canonical).to_hex().to_string())
}

pub fn evaluate_global_reproducibility(
    input: GlobalReproducibilityEvaluationInput,
) -> Result<GlobalReproducibilityReport, ReleaseEvidenceError> {
    validate_blake3_hex(&input.universe_digest_blake3, "universe_digest_blake3")?;
    validate_blake3_hex(&input.policy_digest_blake3, "policy_digest_blake3")?;
    let universe = canonical_global_reproducibility_universe(input.universe)?;
    let policy = canonical_global_reproducibility_policy(input.policy)?;
    let mut report_blockers = Vec::new();
    let evidence_by_surface = collect_surface_evidence(input.surface_evidence, &mut report_blockers)?;
    let mut surface_reports = Vec::new();
    let mut report_evidence_digests = BTreeSet::new();
    let mut report_witness_identities = BTreeSet::new();

    let included_surface_ids =
        universe.included_surfaces.iter().map(|surface| surface.id.clone()).collect::<BTreeSet<_>>();
    for surface in &universe.included_surfaces {
        let evidence = evidence_by_surface.get(&surface.id);
        let surface_report =
            evaluate_surface(surface, evidence, &policy, &input.universe_digest_blake3, &input.policy_digest_blake3)?;
        for digest in collect_surface_report_digests(evidence) {
            report_evidence_digests.insert(digest);
        }
        for identity in &surface_report.accepted_witness_identities {
            report_witness_identities.insert(identity.clone());
        }
        report_blockers.extend(surface_report.blockers.clone());
        surface_reports.push(surface_report);
    }
    push_unknown_surface_evidence_blockers(&included_surface_ids, &evidence_by_surface, &mut report_blockers);

    let claim_class = classify_global_claim(&report_blockers);
    let mut non_claims = default_global_non_claims(&universe.excluded_surfaces);
    if !claim_class.is_global_claim() {
        non_claims.insert(NON_CLAIM_GLOBAL_BLOCKED.to_string());
    }

    let report = GlobalReproducibilityReport {
        schema: GLOBAL_REPRODUCIBILITY_REPORT_SCHEMA.to_string(),
        universe_name: universe.name,
        universe_digest_blake3: input.universe_digest_blake3,
        policy_id: policy.policy_id,
        policy_digest_blake3: input.policy_digest_blake3,
        witness_policy: policy.witness_policy,
        included_surface_count: u32_count(
            surface_reports.len(),
            "global reproducibility surface count overflowed u32",
        )?,
        claim_class,
        evidence_digests_blake3: report_evidence_digests.into_iter().collect(),
        accepted_witness_identities: report_witness_identities.into_iter().collect(),
        surfaces: surface_reports,
        blockers: report_blockers,
        non_claims: non_claims.into_iter().collect(),
    };
    canonical_global_reproducibility_report(report)
}

pub fn canonical_global_reproducibility_report(
    mut report: GlobalReproducibilityReport,
) -> Result<GlobalReproducibilityReport, ReleaseEvidenceError> {
    validate_report_header(&report)?;
    report.schema = GLOBAL_REPRODUCIBILITY_REPORT_SCHEMA.to_string();
    report.evidence_digests_blake3.sort();
    report.accepted_witness_identities.sort();
    report.surfaces.sort_by(|left, right| left.surface_id.cmp(&right.surface_id));
    for surface in &mut report.surfaces {
        canonicalize_surface_report(surface)?;
    }
    sort_blockers(&mut report.blockers);
    report.non_claims.sort();
    validate_report_body(&report)?;
    Ok(report)
}

pub fn global_reproducibility_report_canonical_bytes(
    report: GlobalReproducibilityReport,
) -> Result<Vec<u8>, ReleaseEvidenceError> {
    let canonical = canonical_global_reproducibility_report(report)?;
    serde_json::to_vec(&canonical)
        .map_err(|err| ReleaseEvidenceError::Parse(format!("serializing global reproducibility report: {err}")))
}

pub fn global_reproducibility_report_digest_blake3(
    report: GlobalReproducibilityReport,
) -> Result<String, ReleaseEvidenceError> {
    let canonical = global_reproducibility_report_canonical_bytes(report)?;
    Ok(blake3::hash(&canonical).to_hex().to_string())
}

fn validate_universe_header(universe: &GlobalReproducibilityUniverse) -> Result<(), ReleaseEvidenceError> {
    if universe.schema != GLOBAL_REPRODUCIBILITY_UNIVERSE_SCHEMA {
        return Err(validation_error(format!(
            "global reproducibility universe schema must be {GLOBAL_REPRODUCIBILITY_UNIVERSE_SCHEMA}, got {}",
            universe.schema
        )));
    }
    validate_non_empty_string(&universe.name, "universe.name")?;
    Ok(())
}

fn validate_included_surfaces(surfaces: &[GlobalBuildSurface]) -> Result<(), ReleaseEvidenceError> {
    let surface_count = u32_count(surfaces.len(), "global reproducibility included surface count overflowed u32")?;
    if surface_count == ZERO_COUNT {
        return Err(validation_error(
            "global reproducibility universe must include at least one build surface".to_string(),
        ));
    }
    if surface_count > MAX_GLOBAL_SURFACE_COUNT {
        return Err(validation_error(format!(
            "global reproducibility universe includes {surface_count} surfaces, limit is {MAX_GLOBAL_SURFACE_COUNT}"
        )));
    }
    let mut ids = BTreeSet::new();
    for surface in surfaces {
        validate_surface(surface)?;
        insert_unique(&mut ids, &surface.id, "included surface id")?;
    }
    Ok(())
}

fn validate_excluded_surfaces(surfaces: &[GlobalExcludedSurface]) -> Result<(), ReleaseEvidenceError> {
    let surface_count = u32_count(surfaces.len(), "global reproducibility excluded surface count overflowed u32")?;
    if surface_count > MAX_EXCLUDED_SURFACE_COUNT {
        return Err(validation_error(format!(
            "global reproducibility universe excludes {surface_count} surfaces, limit is {MAX_EXCLUDED_SURFACE_COUNT}"
        )));
    }
    let mut ids = BTreeSet::new();
    for surface in surfaces {
        validate_non_empty_string(&surface.id, "excluded surface id")?;
        validate_non_empty_string(&surface.reason, "excluded surface reason")?;
        insert_unique(&mut ids, &surface.id, "excluded surface id")?;
    }
    Ok(())
}

fn validate_surface(surface: &GlobalBuildSurface) -> Result<(), ReleaseEvidenceError> {
    validate_non_empty_string(&surface.id, "surface.id")?;
    validate_non_empty_string(&surface.target_system, "surface.target_system")?;
    validate_non_empty_string(&surface.source_acquisition_mode, "surface.source_acquisition_mode")?;
    validate_non_empty_string(&surface.toolchain_route, "surface.toolchain_route")?;
    validate_non_empty_string(&surface.cache_substitution_mode, "surface.cache_substitution_mode")?;
    validate_non_empty_string(&surface.release_artifact_set, "surface.release_artifact_set")?;
    Ok(())
}

fn validate_policy_header(policy: &GlobalReproducibilityPolicy) -> Result<(), ReleaseEvidenceError> {
    if policy.schema != GLOBAL_REPRODUCIBILITY_POLICY_SCHEMA {
        return Err(validation_error(format!(
            "global reproducibility policy schema must be {GLOBAL_REPRODUCIBILITY_POLICY_SCHEMA}, got {}",
            policy.schema
        )));
    }
    validate_non_empty_string(&policy.policy_id, "policy.policy_id")?;
    validate_non_empty_string(&policy.witness_policy, "policy.witness_policy")?;
    validate_required_count(policy.required_operator_domains, "required_operator_domains")?;
    validate_required_count(policy.required_host_classes, "required_host_classes")?;
    Ok(())
}

fn validate_required_count(count: u32, field_name: &str) -> Result<(), ReleaseEvidenceError> {
    if count < MIN_REQUIRED_WITNESS_COUNT {
        return Err(validation_error(format!(
            "global reproducibility policy {field_name} must be at least {MIN_REQUIRED_WITNESS_COUNT}"
        )));
    }
    Ok(())
}

fn collect_surface_evidence(
    evidence_items: Vec<GlobalSurfaceEvidence>,
    report_blockers: &mut Vec<GlobalReproducibilityBlocker>,
) -> Result<BTreeMap<String, GlobalSurfaceEvidence>, ReleaseEvidenceError> {
    let evidence_count =
        u32_count(evidence_items.len(), "global reproducibility surface evidence count overflowed u32")?;
    if evidence_count > MAX_GLOBAL_SURFACE_COUNT {
        return Err(validation_error(format!(
            "global reproducibility has {evidence_count} surface evidence entries, limit is {MAX_GLOBAL_SURFACE_COUNT}"
        )));
    }
    let mut map = BTreeMap::new();
    for evidence in evidence_items {
        validate_surface_evidence(&evidence)?;
        let surface_id = evidence.surface_id.clone();
        if map.insert(surface_id.clone(), evidence).is_some() {
            report_blockers.push(blocker(
                Some(surface_id),
                "surface-evidence",
                None,
                None,
                "duplicate surface evidence entry",
                "keep exactly one evidence entry for each included surface",
            ));
        }
    }
    Ok(map)
}

fn validate_surface_evidence(evidence: &GlobalSurfaceEvidence) -> Result<(), ReleaseEvidenceError> {
    if evidence.schema != GLOBAL_REPRODUCIBILITY_SURFACE_EVIDENCE_SCHEMA {
        return Err(validation_error(format!(
            "global reproducibility surface evidence schema must be {GLOBAL_REPRODUCIBILITY_SURFACE_EVIDENCE_SCHEMA}, got {}",
            evidence.schema
        )));
    }
    validate_non_empty_string(&evidence.surface_id, "surface_evidence.surface_id")?;
    validate_blake3_hex(&evidence.universe_digest_blake3, "surface_evidence.universe_digest_blake3")?;
    validate_blake3_hex(&evidence.policy_digest_blake3, "surface_evidence.policy_digest_blake3")?;
    validate_optional_digest(&evidence.action_receipt_digest_blake3, "action_receipt_digest_blake3")?;
    validate_optional_digest(&evidence.source_acquisition_digest_blake3, "source_acquisition_digest_blake3")?;
    validate_optional_digest(&evidence.toolchain_provenance_digest_blake3, "toolchain_provenance_digest_blake3")?;
    validate_optional_digest(&evidence.hermeticity_evidence_digest_blake3, "hermeticity_evidence_digest_blake3")?;
    validate_digest_set(&evidence.output_digest_set_blake3, "output_digest_set_blake3")?;
    validate_witnesses(&evidence.witnesses)?;
    if let Some(reason) = &evidence.unsupported_reason {
        validate_non_empty_string(reason, "unsupported_reason")?;
    }
    Ok(())
}

fn evaluate_surface(
    surface: &GlobalBuildSurface,
    evidence: Option<&GlobalSurfaceEvidence>,
    policy: &GlobalReproducibilityPolicy,
    universe_digest_blake3: &str,
    policy_digest_blake3: &str,
) -> Result<GlobalSurfaceReport, ReleaseEvidenceError> {
    let mut blockers = Vec::new();
    let mut output_digest_set = Vec::new();
    let witness_eval = if let Some(evidence) = evidence {
        output_digest_set = evidence.output_digest_set_blake3.clone();
        evaluate_surface_evidence_blockers(
            evidence,
            policy,
            universe_digest_blake3,
            policy_digest_blake3,
            &mut blockers,
        );
        evaluate_witness_matrix(surface, evidence, policy)?
    } else {
        blockers.push(blocker(
            Some(surface.id.clone()),
            "surface-evidence",
            None,
            None,
            "missing evidence for included global reproducibility surface",
            "record action receipt, source, toolchain, hermeticity, output digest, and witness evidence",
        ));
        SurfaceWitnessEvaluation::empty()
    };
    blockers.extend(witness_eval.blockers.clone());
    Ok(GlobalSurfaceReport {
        surface_id: surface.id.clone(),
        target_system: surface.target_system.clone(),
        source_acquisition_mode: surface.source_acquisition_mode.clone(),
        toolchain_route: surface.toolchain_route.clone(),
        cache_substitution_mode: surface.cache_substitution_mode.clone(),
        release_artifact_set: surface.release_artifact_set.clone(),
        output_digest_set_blake3: output_digest_set,
        accepted_witness_identities: witness_eval.accepted_witness_identities,
        accepted_operator_domains: witness_eval.accepted_operator_domains,
        accepted_host_classes: witness_eval.accepted_host_classes,
        covered_perturbation_axes: witness_eval.covered_perturbation_axes,
        witness_counts: witness_eval.witness_counts,
        blockers,
        non_claims: surface_non_claims(),
    })
}

fn evaluate_surface_evidence_blockers(
    evidence: &GlobalSurfaceEvidence,
    policy: &GlobalReproducibilityPolicy,
    universe_digest_blake3: &str,
    policy_digest_blake3: &str,
    blockers: &mut Vec<GlobalReproducibilityBlocker>,
) {
    push_digest_mismatch_blocker(
        blockers,
        &evidence.surface_id,
        "universe-digest",
        universe_digest_blake3,
        &evidence.universe_digest_blake3,
        "surface evidence was produced for a different universe digest",
        "regenerate the surface evidence for the current universe manifest",
    );
    push_digest_mismatch_blocker(
        blockers,
        &evidence.surface_id,
        "policy-digest",
        policy_digest_blake3,
        &evidence.policy_digest_blake3,
        "surface evidence was produced for a different policy digest",
        "regenerate the surface evidence under the current global reproducibility policy",
    );
    push_required_digest_blockers(evidence, blockers);
    push_policy_blockers(evidence, policy, blockers);
}

fn push_required_digest_blockers(evidence: &GlobalSurfaceEvidence, blockers: &mut Vec<GlobalReproducibilityBlocker>) {
    require_digest(blockers, evidence, &evidence.action_receipt_digest_blake3, "action-receipt");
    require_digest(blockers, evidence, &evidence.source_acquisition_digest_blake3, "source-acquisition");
    require_digest(blockers, evidence, &evidence.toolchain_provenance_digest_blake3, "toolchain-provenance");
    require_digest(blockers, evidence, &evidence.hermeticity_evidence_digest_blake3, "strict-hermeticity");
    if evidence.output_digest_set_blake3.is_empty() {
        blockers.push(blocker(
            Some(evidence.surface_id.clone()),
            "output-digest-evidence",
            None,
            None,
            "missing output digest evidence for included surface",
            "record the expected BLAKE3 output digest set for this surface",
        ));
    }
}

fn push_policy_blockers(
    evidence: &GlobalSurfaceEvidence,
    policy: &GlobalReproducibilityPolicy,
    blockers: &mut Vec<GlobalReproducibilityBlocker>,
) {
    if let Some(reason) = &evidence.unsupported_reason {
        blockers.push(blocker(
            Some(evidence.surface_id.clone()),
            "unsupported-surface",
            None,
            None,
            reason,
            "remove the surface from the included universe or add supported evidence for it",
        ));
    }
    if policy.require_strict_hermeticity && !evidence.strict_hermeticity {
        blockers.push(blocker(
            Some(evidence.surface_id.clone()),
            "weak-hermeticity",
            None,
            None,
            "surface evidence is not strict-hermetic",
            "rerun the surface under strict hermeticity and record the resulting receipt",
        ));
    }
    if policy.require_fresh_rebuild_store && !evidence.fresh_rebuild_store {
        blockers.push(blocker(
            Some(evidence.surface_id.clone()),
            "reused-store",
            None,
            None,
            "surface evidence reused a store where fresh replay was required",
            "rerun independent replay with a fresh store identity",
        ));
    }
}

fn evaluate_witness_matrix(
    surface: &GlobalBuildSurface,
    evidence: &GlobalSurfaceEvidence,
    policy: &GlobalReproducibilityPolicy,
) -> Result<SurfaceWitnessEvaluation, ReleaseEvidenceError> {
    let mut counts = GlobalWitnessCounts::empty();
    let mut identities = BTreeSet::new();
    let mut domains = BTreeSet::new();
    let mut host_classes = BTreeSet::new();
    let mut axes = BTreeSet::new();
    let mut blockers = Vec::new();
    let mut witnesses = evidence.witnesses.clone();
    witnesses.sort_by(|left, right| left.identity.cmp(&right.identity));

    for witness in &witnesses {
        evaluate_one_witness(
            evidence,
            witness,
            &mut counts,
            &mut identities,
            &mut domains,
            &mut host_classes,
            &mut axes,
            &mut blockers,
        );
    }
    push_witness_policy_blockers(surface, policy, &domains, &host_classes, &axes, &mut blockers)?;
    Ok(SurfaceWitnessEvaluation {
        accepted_witness_identities: identities.into_iter().collect(),
        accepted_operator_domains: domains.into_iter().collect(),
        accepted_host_classes: host_classes.into_iter().collect(),
        covered_perturbation_axes: axes.into_iter().collect(),
        witness_counts: counts,
        blockers,
    })
}

#[allow(clippy::too_many_arguments)]
fn evaluate_one_witness(
    evidence: &GlobalSurfaceEvidence,
    witness: &GlobalWitnessEvidence,
    counts: &mut GlobalWitnessCounts,
    identities: &mut BTreeSet<String>,
    domains: &mut BTreeSet<String>,
    host_classes: &mut BTreeSet<String>,
    axes: &mut BTreeSet<String>,
    blockers: &mut Vec<GlobalReproducibilityBlocker>,
) {
    match witness.trust_status {
        GlobalWitnessTrustStatus::UnknownKey => {
            counts.skipped_unknown_key = counts.skipped_unknown_key.saturating_add(1);
        }
        GlobalWitnessTrustStatus::Revoked => {
            counts.skipped_revoked = counts.skipped_revoked.saturating_add(1);
        }
        GlobalWitnessTrustStatus::InvalidSignature => {
            counts.failed_invalid_signature = counts.failed_invalid_signature.saturating_add(1);
        }
        GlobalWitnessTrustStatus::PolicyInsufficient => {
            counts.policy_insufficient = counts.policy_insufficient.saturating_add(1);
        }
        GlobalWitnessTrustStatus::Valid => {
            accept_valid_witness(evidence, witness, counts, identities, domains, host_classes, axes, blockers);
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn accept_valid_witness(
    evidence: &GlobalSurfaceEvidence,
    witness: &GlobalWitnessEvidence,
    counts: &mut GlobalWitnessCounts,
    identities: &mut BTreeSet<String>,
    domains: &mut BTreeSet<String>,
    host_classes: &mut BTreeSet<String>,
    axes: &mut BTreeSet<String>,
    blockers: &mut Vec<GlobalReproducibilityBlocker>,
) {
    if !digest_sets_match(&witness.output_digest_set_blake3, &evidence.output_digest_set_blake3) {
        counts.failed_digest_mismatched = counts.failed_digest_mismatched.saturating_add(1);
        blockers.push(blocker(
            Some(evidence.surface_id.clone()),
            "witness-output-digest",
            single_digest_field(&evidence.output_digest_set_blake3),
            single_digest_field(&witness.output_digest_set_blake3),
            "witness output digest set did not match the surface output digest set",
            "rerun witness replay or inspect the mismatched artifact output",
        ));
        return;
    }
    if domains.contains(&witness.operator_domain) {
        counts.skipped_same_domain = counts.skipped_same_domain.saturating_add(1);
        return;
    }
    counts.accepted = counts.accepted.saturating_add(1);
    identities.insert(witness.identity.clone());
    domains.insert(witness.operator_domain.clone());
    host_classes.insert(witness.host_class.clone());
    for axis in &witness.perturbation_axes {
        axes.insert(axis.clone());
    }
}

fn push_witness_policy_blockers(
    surface: &GlobalBuildSurface,
    policy: &GlobalReproducibilityPolicy,
    domains: &BTreeSet<String>,
    host_classes: &BTreeSet<String>,
    axes: &BTreeSet<String>,
    blockers: &mut Vec<GlobalReproducibilityBlocker>,
) -> Result<(), ReleaseEvidenceError> {
    let domain_count = u32_count(domains.len(), "global reproducibility accepted domain count overflowed u32")?;
    if domain_count < policy.required_operator_domains {
        blockers.push(blocker(
            Some(surface.id.clone()),
            "witness-operator-domain-quorum",
            None,
            None,
            "accepted witness operator domains do not satisfy the policy quorum",
            "add independent valid witnesses from additional operator domains",
        ));
    }
    push_host_class_quorum_blocker(surface, policy, host_classes, blockers)?;
    push_missing_axis_blockers(surface, policy, axes, blockers);
    Ok(())
}

fn push_host_class_quorum_blocker(
    surface: &GlobalBuildSurface,
    policy: &GlobalReproducibilityPolicy,
    host_classes: &BTreeSet<String>,
    blockers: &mut Vec<GlobalReproducibilityBlocker>,
) -> Result<(), ReleaseEvidenceError> {
    let host_class_count = u32_count(host_classes.len(), "global reproducibility host class count overflowed u32")?;
    if host_class_count < policy.required_host_classes {
        blockers.push(blocker(
            Some(surface.id.clone()),
            "witness-host-class-quorum",
            None,
            None,
            "accepted witness host classes do not satisfy the policy quorum",
            "add valid witnesses from the required host class diversity",
        ));
    }
    Ok(())
}

fn push_missing_axis_blockers(
    surface: &GlobalBuildSurface,
    policy: &GlobalReproducibilityPolicy,
    axes: &BTreeSet<String>,
    blockers: &mut Vec<GlobalReproducibilityBlocker>,
) {
    for axis in &policy.required_perturbation_axes {
        if !axes.contains(axis) {
            blockers.push(blocker(
                Some(surface.id.clone()),
                "witness-perturbation-axis",
                None,
                None,
                &format!("missing accepted witness coverage for perturbation axis {axis}"),
                "rerun an accepted witness with the missing perturbation axis recorded",
            ));
        }
    }
}

impl SurfaceWitnessEvaluation {
    fn empty() -> Self {
        Self {
            accepted_witness_identities: Vec::new(),
            accepted_operator_domains: Vec::new(),
            accepted_host_classes: Vec::new(),
            covered_perturbation_axes: Vec::new(),
            witness_counts: GlobalWitnessCounts::empty(),
            blockers: Vec::new(),
        }
    }
}

fn classify_global_claim(blockers: &[GlobalReproducibilityBlocker]) -> GlobalReproducibilityClaimClass {
    if blockers.is_empty() {
        GlobalReproducibilityClaimClass::Eligible
    } else {
        GlobalReproducibilityClaimClass::Blocked
    }
}

fn push_unknown_surface_evidence_blockers(
    included_surface_ids: &BTreeSet<String>,
    evidence_by_surface: &BTreeMap<String, GlobalSurfaceEvidence>,
    blockers: &mut Vec<GlobalReproducibilityBlocker>,
) {
    for surface_id in evidence_by_surface.keys() {
        if !included_surface_ids.contains(surface_id) {
            blockers.push(blocker(
                Some(surface_id.clone()),
                "unknown-surface-evidence",
                None,
                None,
                "surface evidence is not declared in the global reproducibility universe",
                "remove the extra evidence or add the surface to the digest-bound universe",
            ));
        }
    }
}

fn collect_surface_report_digests(evidence: Option<&GlobalSurfaceEvidence>) -> Vec<String> {
    let mut digests = BTreeSet::new();
    if let Some(evidence) = evidence {
        insert_optional_digest(&mut digests, &evidence.action_receipt_digest_blake3);
        insert_optional_digest(&mut digests, &evidence.source_acquisition_digest_blake3);
        insert_optional_digest(&mut digests, &evidence.toolchain_provenance_digest_blake3);
        insert_optional_digest(&mut digests, &evidence.hermeticity_evidence_digest_blake3);
        for digest in &evidence.output_digest_set_blake3 {
            digests.insert(digest.clone());
        }
        for witness in &evidence.witnesses {
            for digest in &witness.output_digest_set_blake3 {
                digests.insert(digest.clone());
            }
        }
    }
    digests.into_iter().collect()
}

fn default_global_non_claims(excluded: &[GlobalExcludedSurface]) -> BTreeSet<String> {
    let mut non_claims = BTreeSet::new();
    for claim in [
        NON_CLAIM_FUTURE_CODE,
        NON_CLAIM_UNDECLARED_FRONTENDS,
        NON_CLAIM_UNDECLARED_TARGET_SYSTEMS,
        NON_CLAIM_COMPILER_CORRECTNESS,
        NON_CLAIM_DEPLOY_SUCCESS,
        NON_CLAIM_PHYSICAL_TARGET_DETERMINISM,
    ] {
        non_claims.insert(claim.to_string());
    }
    for excluded_surface in excluded {
        non_claims.insert(format!("excluded-surface:{}", excluded_surface.id));
    }
    non_claims
}

fn surface_non_claims() -> Vec<String> {
    vec![
        NON_CLAIM_FUTURE_CODE.to_string(),
        NON_CLAIM_COMPILER_CORRECTNESS.to_string(),
        NON_CLAIM_DEPLOY_SUCCESS.to_string(),
    ]
}

fn canonicalize_surface_report(surface: &mut GlobalSurfaceReport) -> Result<(), ReleaseEvidenceError> {
    surface.output_digest_set_blake3.sort();
    surface.accepted_witness_identities.sort();
    surface.accepted_operator_domains.sort();
    surface.accepted_host_classes.sort();
    surface.covered_perturbation_axes.sort();
    sort_blockers(&mut surface.blockers);
    surface.non_claims.sort();
    validate_non_empty_string(&surface.surface_id, "surface_report.surface_id")?;
    validate_digest_set(&surface.output_digest_set_blake3, "surface_report.output_digest_set_blake3")?;
    validate_blockers(&surface.blockers)?;
    validate_string_set(&surface.non_claims, "surface_report.non_claims")?;
    Ok(())
}

fn validate_report_header(report: &GlobalReproducibilityReport) -> Result<(), ReleaseEvidenceError> {
    if report.schema != GLOBAL_REPRODUCIBILITY_REPORT_SCHEMA {
        return Err(validation_error(format!(
            "global reproducibility report schema must be {GLOBAL_REPRODUCIBILITY_REPORT_SCHEMA}, got {}",
            report.schema
        )));
    }
    validate_non_empty_string(&report.universe_name, "report.universe_name")?;
    validate_blake3_hex(&report.universe_digest_blake3, "report.universe_digest_blake3")?;
    validate_non_empty_string(&report.policy_id, "report.policy_id")?;
    validate_blake3_hex(&report.policy_digest_blake3, "report.policy_digest_blake3")?;
    validate_non_empty_string(&report.witness_policy, "report.witness_policy")?;
    Ok(())
}

fn validate_report_body(report: &GlobalReproducibilityReport) -> Result<(), ReleaseEvidenceError> {
    let surface_count = u32_count(report.surfaces.len(), "global reproducibility report surface count overflowed u32")?;
    if report.included_surface_count != surface_count {
        return Err(validation_error(format!(
            "global reproducibility report included_surface_count mismatch: expected {surface_count}, got {}",
            report.included_surface_count
        )));
    }
    validate_digest_set(&report.evidence_digests_blake3, "report.evidence_digests_blake3")?;
    validate_string_set(&report.accepted_witness_identities, "report.accepted_witness_identities")?;
    validate_blockers(&report.blockers)?;
    validate_report_claim_class(report)?;
    validate_string_set(&report.non_claims, "report.non_claims")?;
    Ok(())
}

fn validate_report_claim_class(report: &GlobalReproducibilityReport) -> Result<(), ReleaseEvidenceError> {
    if report.claim_class == GlobalReproducibilityClaimClass::Eligible && !report.blockers.is_empty() {
        return Err(validation_error(
            "global reproducibility report cannot be eligible while blockers are present".to_string(),
        ));
    }
    if report.claim_class == GlobalReproducibilityClaimClass::Blocked && report.blockers.is_empty() {
        return Err(validation_error(
            "global reproducibility blocked report must preserve at least one blocker".to_string(),
        ));
    }
    Ok(())
}

fn validate_witnesses(witnesses: &[GlobalWitnessEvidence]) -> Result<(), ReleaseEvidenceError> {
    let witness_count = u32_count(witnesses.len(), "global reproducibility witness count overflowed u32")?;
    if witness_count > MAX_GLOBAL_WITNESS_COUNT {
        return Err(validation_error(format!(
            "global reproducibility has {witness_count} witnesses, limit is {MAX_GLOBAL_WITNESS_COUNT}"
        )));
    }
    let mut identities = BTreeSet::new();
    for witness in witnesses {
        validate_non_empty_string(&witness.identity, "witness.identity")?;
        validate_non_empty_string(&witness.operator_domain, "witness.operator_domain")?;
        validate_non_empty_string(&witness.host_class, "witness.host_class")?;
        validate_digest_set(&witness.output_digest_set_blake3, "witness.output_digest_set_blake3")?;
        validate_string_set(&witness.perturbation_axes, "witness.perturbation_axes")?;
        insert_unique(&mut identities, &witness.identity, "witness identity")?;
    }
    Ok(())
}

fn validate_digest_set(digests: &[String], field_name: &str) -> Result<(), ReleaseEvidenceError> {
    let digest_count = u32_count(digests.len(), "global reproducibility digest count overflowed u32")?;
    if digest_count > MAX_GLOBAL_DIGEST_COUNT {
        return Err(validation_error(format!(
            "global reproducibility {field_name} contains {digest_count} digests, limit is {MAX_GLOBAL_DIGEST_COUNT}"
        )));
    }
    let mut seen = BTreeSet::new();
    for digest in digests {
        validate_blake3_hex(digest, field_name)?;
        insert_unique(&mut seen, digest, field_name)?;
    }
    Ok(())
}

fn validate_optional_digest(digest: &Option<String>, field_name: &str) -> Result<(), ReleaseEvidenceError> {
    if let Some(digest) = digest {
        validate_blake3_hex(digest, field_name)?;
    }
    Ok(())
}

fn validate_perturbation_axes(values: &[String]) -> Result<(), ReleaseEvidenceError> {
    let axis_count = u32_count(values.len(), "global reproducibility perturbation axis count overflowed u32")?;
    if axis_count > MAX_GLOBAL_PERTURBATION_AXIS_COUNT {
        return Err(validation_error(format!(
            "global reproducibility required_perturbation_axes contains {axis_count} values, limit is {MAX_GLOBAL_PERTURBATION_AXIS_COUNT}"
        )));
    }
    validate_string_set(values, "required_perturbation_axes")
}

fn validate_string_set(values: &[String], field_name: &str) -> Result<(), ReleaseEvidenceError> {
    let value_count = u32_count(values.len(), "global reproducibility string-set count overflowed u32")?;
    if value_count > MAX_GLOBAL_STRING_SET_COUNT {
        return Err(validation_error(format!(
            "global reproducibility {field_name} contains {value_count} values, limit is {MAX_GLOBAL_STRING_SET_COUNT}"
        )));
    }
    let mut seen = BTreeSet::new();
    for value in values {
        validate_non_empty_string(value, field_name)?;
        insert_unique(&mut seen, value, field_name)?;
    }
    Ok(())
}

fn validate_blockers(blockers: &[GlobalReproducibilityBlocker]) -> Result<(), ReleaseEvidenceError> {
    let blocker_count = u32_count(blockers.len(), "global reproducibility blocker count overflowed u32")?;
    if blocker_count > MAX_GLOBAL_BLOCKER_COUNT {
        return Err(validation_error(format!(
            "global reproducibility report has {blocker_count} blockers, limit is {MAX_GLOBAL_BLOCKER_COUNT}"
        )));
    }
    for blocker in blockers {
        validate_non_empty_string(&blocker.evidence_class, "blocker.evidence_class")?;
        validate_non_empty_string(&blocker.message, "blocker.message")?;
        validate_non_empty_string(&blocker.next_action, "blocker.next_action")?;
        validate_optional_digest(&blocker.expected_digest_blake3, "blocker.expected_digest_blake3")?;
        validate_optional_digest(&blocker.observed_digest_blake3, "blocker.observed_digest_blake3")?;
    }
    Ok(())
}

fn validate_non_empty_string(value: &str, field_name: &str) -> Result<(), ReleaseEvidenceError> {
    if value.trim().is_empty() {
        return Err(validation_error(format!("global reproducibility {field_name} must not be empty")));
    }
    let value_bytes = u32_count(value.len(), "global reproducibility string byte count overflowed u32")?;
    if value_bytes > MAX_GLOBAL_STRING_BYTES_COUNT {
        return Err(validation_error(format!(
            "global reproducibility {field_name} is {value_bytes} bytes, limit is {MAX_GLOBAL_STRING_BYTES_COUNT}"
        )));
    }
    Ok(())
}

fn insert_unique(seen: &mut BTreeSet<String>, value: &str, field_name: &str) -> Result<(), ReleaseEvidenceError> {
    if !seen.insert(value.to_string()) {
        return Err(validation_error(format!("global reproducibility duplicate {field_name}: {value}")));
    }
    Ok(())
}

fn insert_optional_digest(digests: &mut BTreeSet<String>, digest: &Option<String>) {
    if let Some(digest) = digest {
        digests.insert(digest.clone());
    }
}

fn require_digest(
    blockers: &mut Vec<GlobalReproducibilityBlocker>,
    evidence: &GlobalSurfaceEvidence,
    digest: &Option<String>,
    evidence_class: &str,
) {
    if digest.is_none() {
        blockers.push(blocker(
            Some(evidence.surface_id.clone()),
            evidence_class,
            None,
            None,
            &format!("missing {evidence_class} evidence digest for included surface"),
            "record the required evidence digest for this global reproducibility surface",
        ));
    }
}

fn push_digest_mismatch_blocker(
    blockers: &mut Vec<GlobalReproducibilityBlocker>,
    surface_id: &str,
    evidence_class: &str,
    expected: &str,
    observed: &str,
    message: &str,
    next_action: &str,
) {
    if expected != observed {
        blockers.push(blocker(
            Some(surface_id.to_string()),
            evidence_class,
            Some(expected.to_string()),
            Some(observed.to_string()),
            message,
            next_action,
        ));
    }
}

fn blocker(
    surface_id: Option<String>,
    evidence_class: &str,
    expected_digest_blake3: Option<String>,
    observed_digest_blake3: Option<String>,
    message: &str,
    next_action: &str,
) -> GlobalReproducibilityBlocker {
    GlobalReproducibilityBlocker {
        surface_id,
        evidence_class: evidence_class.to_string(),
        expected_digest_blake3,
        observed_digest_blake3,
        message: message.to_string(),
        next_action: next_action.to_string(),
    }
}

fn sort_blockers(blockers: &mut [GlobalReproducibilityBlocker]) {
    blockers.sort_by(compare_blockers);
}

fn compare_blockers(left: &GlobalReproducibilityBlocker, right: &GlobalReproducibilityBlocker) -> Ordering {
    left.surface_id
        .cmp(&right.surface_id)
        .then_with(|| left.evidence_class.cmp(&right.evidence_class))
        .then_with(|| left.message.cmp(&right.message))
}

fn digest_sets_match(left: &[String], right: &[String]) -> bool {
    if left.len() != right.len() {
        return false;
    }
    let left_set = left.iter().collect::<BTreeSet<_>>();
    let right_set = right.iter().collect::<BTreeSet<_>>();
    left_set == right_set
}

fn single_digest_field(digests: &[String]) -> Option<String> {
    if digests.len() == 1 {
        digests.first().cloned()
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use alloc::vec;

    use pretty_assertions::assert_eq;

    use super::*;
    use crate::manifest::BLAKE3_HEX_LENGTH_CHARS;

    const SAMPLE_SIZE_BYTES: u64 = 42;

    fn sample_digest(seed: u8) -> String {
        let bytes = [seed; SAMPLE_SIZE_BYTES as usize];
        blake3::hash(&bytes).to_hex().to_string()
    }

    fn sample_surface() -> GlobalBuildSurface {
        GlobalBuildSurface {
            id: "mantle-release-x86_64".to_string(),
            target_system: "x86_64-linux".to_string(),
            source_acquisition_mode: "git-archive".to_string(),
            toolchain_route: "source-built-rust".to_string(),
            cache_substitution_mode: "no-substitute".to_string(),
            release_artifact_set: "binaries/01-mantle".to_string(),
        }
    }

    fn sample_universe() -> GlobalReproducibilityUniverse {
        GlobalReproducibilityUniverse {
            schema: GLOBAL_REPRODUCIBILITY_UNIVERSE_SCHEMA.to_string(),
            name: "tiny admitted universe".to_string(),
            included_surfaces: vec![sample_surface()],
            excluded_surfaces: vec![GlobalExcludedSurface {
                id: "undeclared-frontends".to_string(),
                reason: "no replay receipts yet".to_string(),
            }],
        }
    }

    fn sample_policy() -> GlobalReproducibilityPolicy {
        GlobalReproducibilityPolicy {
            schema: GLOBAL_REPRODUCIBILITY_POLICY_SCHEMA.to_string(),
            policy_id: "single-independent-domain".to_string(),
            witness_policy: "one-domain-one-host-with-path-perturbation".to_string(),
            required_operator_domains: 1,
            required_host_classes: 1,
            required_perturbation_axes: vec!["PATH".to_string()],
            require_strict_hermeticity: true,
            require_fresh_rebuild_store: true,
        }
    }

    fn sample_witness(identity: &str, domain: &str, digest_set: Vec<String>) -> GlobalWitnessEvidence {
        GlobalWitnessEvidence {
            identity: identity.to_string(),
            operator_domain: domain.to_string(),
            host_class: "nixos-25.05".to_string(),
            perturbation_axes: vec!["PATH".to_string()],
            trust_status: GlobalWitnessTrustStatus::Valid,
            output_digest_set_blake3: digest_set,
        }
    }

    fn sample_evidence(universe_digest: String, policy_digest: String) -> GlobalSurfaceEvidence {
        let output_digest = sample_digest(10);
        GlobalSurfaceEvidence {
            schema: GLOBAL_REPRODUCIBILITY_SURFACE_EVIDENCE_SCHEMA.to_string(),
            surface_id: sample_surface().id,
            universe_digest_blake3: universe_digest,
            policy_digest_blake3: policy_digest,
            action_receipt_digest_blake3: Some(sample_digest(1)),
            source_acquisition_digest_blake3: Some(sample_digest(2)),
            toolchain_provenance_digest_blake3: Some(sample_digest(3)),
            hermeticity_evidence_digest_blake3: Some(sample_digest(4)),
            output_digest_set_blake3: vec![output_digest.clone()],
            strict_hermeticity: true,
            fresh_rebuild_store: true,
            unsupported_reason: None,
            witnesses: vec![sample_witness("aspen1", "aspen", vec![output_digest])],
        }
    }

    fn sample_input() -> GlobalReproducibilityEvaluationInput {
        let universe = sample_universe();
        let policy = sample_policy();
        let universe_digest = global_reproducibility_universe_digest_blake3(universe.clone()).unwrap();
        let policy_digest = global_reproducibility_policy_digest_blake3(policy.clone()).unwrap();
        let evidence = sample_evidence(universe_digest.clone(), policy_digest.clone());
        GlobalReproducibilityEvaluationInput {
            universe,
            policy,
            universe_digest_blake3: universe_digest,
            policy_digest_blake3: policy_digest,
            surface_evidence: vec![evidence],
        }
    }

    #[test]
    fn global_reproducibility_report_accepts_tiny_eligible_universe() {
        let input = sample_input();

        let report = evaluate_global_reproducibility(input).unwrap();
        let digest = global_reproducibility_report_digest_blake3(report.clone()).unwrap();

        assert_eq!(report.claim_class, GlobalReproducibilityClaimClass::Eligible);
        assert_eq!(report.included_surface_count, 1);
        assert_eq!(report.accepted_witness_identities, vec!["aspen1".to_string()]);
        assert!(report.blockers.is_empty());
        assert!(report.non_claims.contains(&NON_CLAIM_FUTURE_CODE.to_string()));
        assert_eq!(digest.len(), BLAKE3_HEX_LENGTH_CHARS);
    }

    #[test]
    fn global_reproducibility_report_canonical_bytes_are_stable() {
        let mut input = sample_input();
        let output_digest_set = input.surface_evidence[0].output_digest_set_blake3.clone();
        input.surface_evidence[0]
            .witnesses
            .push(sample_witness("same-domain-extra", "aspen", output_digest_set));
        input.universe.included_surfaces.reverse();

        let first = evaluate_global_reproducibility(input.clone()).unwrap();
        let second = evaluate_global_reproducibility(input).unwrap();
        let first_bytes = global_reproducibility_report_canonical_bytes(first).unwrap();
        let second_bytes = global_reproducibility_report_canonical_bytes(second).unwrap();

        assert_eq!(first_bytes, second_bytes);
        assert!(!first_bytes.contains(&b'\n'));
    }

    #[test]
    fn global_reproducibility_blocks_required_negative_fixtures() {
        let cases = vec![
            ("action-receipt", make_missing_receipt_fixture as fn(GlobalReproducibilityEvaluationInput) -> _),
            ("weak-hermeticity", make_weak_hermeticity_fixture),
            ("policy-digest", make_stale_policy_fixture),
            ("unsupported-surface", make_unsupported_fixture),
            ("witness-output-digest", make_digest_mismatch_fixture),
            ("witness-operator-domain-quorum", make_insufficient_quorum_fixture),
        ];
        for (expected_class, mutate) in cases {
            let report = evaluate_global_reproducibility(mutate(sample_input())).unwrap();

            assert_eq!(report.claim_class, GlobalReproducibilityClaimClass::Blocked);
            assert!(
                report.blockers.iter().any(|blocker| blocker.evidence_class == expected_class),
                "missing expected blocker {expected_class}: {:#?}",
                report.blockers
            );
            assert!(report.non_claims.contains(&NON_CLAIM_GLOBAL_BLOCKED.to_string()));
        }
    }

    #[test]
    fn release_scoped_witness_evidence_alone_does_not_become_global() {
        let mut input = sample_input();
        input.surface_evidence[0].action_receipt_digest_blake3 = None;
        input.surface_evidence[0].source_acquisition_digest_blake3 = None;
        input.surface_evidence[0].toolchain_provenance_digest_blake3 = None;
        input.surface_evidence[0].hermeticity_evidence_digest_blake3 = None;

        let report = evaluate_global_reproducibility(input).unwrap();

        assert_eq!(report.claim_class, GlobalReproducibilityClaimClass::Blocked);
        assert!(report.accepted_witness_identities.contains(&"aspen1".to_string()));
        assert!(report.blockers.iter().any(|blocker| blocker.evidence_class == "action-receipt"));
        assert!(report.blockers.iter().any(|blocker| blocker.evidence_class == "toolchain-provenance"));
    }

    fn make_missing_receipt_fixture(
        mut input: GlobalReproducibilityEvaluationInput,
    ) -> GlobalReproducibilityEvaluationInput {
        input.surface_evidence[0].action_receipt_digest_blake3 = None;
        input
    }

    fn make_weak_hermeticity_fixture(
        mut input: GlobalReproducibilityEvaluationInput,
    ) -> GlobalReproducibilityEvaluationInput {
        input.surface_evidence[0].strict_hermeticity = false;
        input
    }

    fn make_stale_policy_fixture(
        mut input: GlobalReproducibilityEvaluationInput,
    ) -> GlobalReproducibilityEvaluationInput {
        input.surface_evidence[0].policy_digest_blake3 = sample_digest(99);
        input
    }

    fn make_unsupported_fixture(
        mut input: GlobalReproducibilityEvaluationInput,
    ) -> GlobalReproducibilityEvaluationInput {
        input.surface_evidence[0].unsupported_reason = Some("source transport lacks replay receipts".to_string());
        input
    }

    fn make_digest_mismatch_fixture(
        mut input: GlobalReproducibilityEvaluationInput,
    ) -> GlobalReproducibilityEvaluationInput {
        input.surface_evidence[0].witnesses[0].output_digest_set_blake3 = vec![sample_digest(77)];
        input
    }

    fn make_insufficient_quorum_fixture(
        mut input: GlobalReproducibilityEvaluationInput,
    ) -> GlobalReproducibilityEvaluationInput {
        input.policy.required_operator_domains = 2;
        input
    }
}
