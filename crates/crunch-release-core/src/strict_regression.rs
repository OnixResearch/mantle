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
use crate::gauntlet::GauntletBlocker;
use crate::gauntlet::GauntletMode;
use crate::gauntlet::GauntletVerdict;
use crate::manifest::u32_count;
use crate::manifest::validate_blake3_hex;
use crate::manifest::validation_error;

pub const STRICT_HERMETICITY_REGRESSION_SUITE_PLAN_SCHEMA: &str = "mantle-strict-hermeticity-regression-suite-plan-v1";
pub const STRICT_HERMETICITY_REGRESSION_SUITE_REPORT_SCHEMA: &str =
    "mantle-strict-hermeticity-regression-suite-report-v1";

const MAX_STRICT_REGRESSION_CASE_COUNT: u32 = 64;
const MAX_STRICT_REGRESSION_CLASS_COUNT: u32 = 64;
const MAX_STRICT_REGRESSION_STRING_BYTES: u32 = 2048;
const ZERO_COUNT: u32 = 0;
const DIGEST_FIELD_PLAN: &str = "plan_digest_blake3";

const AXIS_CLEAN_STRICT: &str = "clean-strict";
const AXIS_ENVIRONMENT_POISONING: &str = "environment-poisoning";
const AXIS_PATH_POISONING: &str = "path-poisoning";
const AXIS_UNDECLARED_HOST_EXECUTION: &str = "undeclared-host-execution";
const AXIS_NETWORK_ACCESS: &str = "network-access";
const AXIS_MISSING_CLOSURE_FACTS: &str = "missing-closure-facts";
const AXIS_UMASK_DRIFT: &str = "umask-drift";
const AXIS_TEMP_ROOT_DEPENDENCE: &str = "temp-root-dependence";
const AXIS_NONDETERMINISTIC_OUTPUT: &str = "nondeterministic-output";

const NON_CLAIM_STRICT_REGRESSION_BLOCKED: &str = "strict-hermeticity-regression-blocked";

struct StringField<'a> {
    value: &'a str,
    name: &'a str,
}

struct SchemaRequirement<'a> {
    actual: &'a str,
    expected: &'a str,
    field_name: &'a str,
}

struct GauntletBlockerInput<'a> {
    evidence_class: &'a str,
    message: &'a str,
    next_action: &'a str,
    axis: Option<String>,
    expected_digest_blake3: Option<String>,
    observed_digest_blake3: Option<String>,
}

const REQUIRED_REGRESSION_AXES: &[&str] = &[
    AXIS_CLEAN_STRICT,
    AXIS_ENVIRONMENT_POISONING,
    AXIS_PATH_POISONING,
    AXIS_UNDECLARED_HOST_EXECUTION,
    AXIS_NETWORK_ACCESS,
    AXIS_MISSING_CLOSURE_FACTS,
    AXIS_UMASK_DRIFT,
    AXIS_TEMP_ROOT_DEPENDENCE,
    AXIS_NONDETERMINISTIC_OUTPUT,
];

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StrictHermeticityRegressionCasePlan {
    pub case_id: String,
    pub perturbation_axis: String,
    pub mode: GauntletMode,
    pub expected_verdict: GauntletVerdict,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required_blocker_or_audit_class: Option<String>,
    pub unsupported_host_non_claim: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expected_output_digest_blake3: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StrictHermeticityRegressionSuitePlan {
    pub schema: String,
    pub suite_id: String,
    pub plan_version: String,
    pub cases: Vec<StrictHermeticityRegressionCasePlan>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StrictHermeticityRegressionCaseEvidence {
    pub case_id: String,
    pub observed_verdict: GauntletVerdict,
    #[serde(default = "crate::empty_vec")]
    pub observed_blocker_or_audit_classes: Vec<String>,
    #[serde(default = "crate::default_false")]
    pub unsupported_host: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_digest_blake3: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repeated_output_digest_blake3: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StrictHermeticityRegressionCaseReport {
    pub case_id: String,
    pub perturbation_axis: String,
    pub mode: GauntletMode,
    pub expected_verdict: GauntletVerdict,
    pub verdict: GauntletVerdict,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required_blocker_or_audit_class: Option<String>,
    pub observed_blocker_or_audit_classes: Vec<String>,
    pub unsupported_host: bool,
    pub unsupported_host_non_claim: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expected_output_digest_blake3: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_digest_blake3: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repeated_output_digest_blake3: Option<String>,
    pub stable_output_digest: bool,
    pub blockers: Vec<GauntletBlocker>,
    pub non_claims: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StrictHermeticityRegressionSuiteReport {
    pub schema: String,
    pub run_id: String,
    pub suite_id: String,
    pub plan_version: String,
    pub plan_digest_blake3: String,
    pub cases: Vec<StrictHermeticityRegressionCaseReport>,
    pub strict_regression_evidence_eligible: bool,
    pub blockers: Vec<GauntletBlocker>,
    pub non_claims: Vec<String>,
}

pub fn strict_hermeticity_regression_required_axes() -> Vec<String> {
    REQUIRED_REGRESSION_AXES.iter().map(|axis| (*axis).to_string()).collect()
}

pub fn canonical_strict_hermeticity_regression_suite_plan(
    mut plan: StrictHermeticityRegressionSuitePlan,
) -> Result<StrictHermeticityRegressionSuitePlan, ReleaseEvidenceError> {
    require_schema(SchemaRequirement {
        actual: &plan.schema,
        expected: STRICT_HERMETICITY_REGRESSION_SUITE_PLAN_SCHEMA,
        field_name: "strict hermeticity regression plan schema",
    })?;
    plan.schema = STRICT_HERMETICITY_REGRESSION_SUITE_PLAN_SCHEMA.to_string();
    validate_non_empty_string(StringField {
        value: &plan.suite_id,
        name: "strict regression suite_id",
    })?;
    validate_non_empty_string(StringField {
        value: &plan.plan_version,
        name: "strict regression plan_version",
    })?;
    validate_case_count(plan.cases.len(), "strict regression plan cases")?;
    plan.cases.sort_by(|left, right| left.case_id.cmp(&right.case_id));
    let mut seen_case_ids = BTreeSet::new();
    for case in &mut plan.cases {
        validate_plan_case(case)?;
        if !seen_case_ids.insert(case.case_id.clone()) {
            return Err(validation_error(format!("duplicate strict regression case_id: {}", case.case_id)));
        }
    }
    debug_assert_eq!(seen_case_ids.len(), plan.cases.len());
    debug_assert!(plan.cases.windows(2).all(|pair| pair[0].case_id < pair[1].case_id));
    Ok(plan)
}

pub fn strict_hermeticity_regression_suite_plan_canonical_bytes(
    plan: StrictHermeticityRegressionSuitePlan,
) -> Result<Vec<u8>, ReleaseEvidenceError> {
    let canonical = canonical_strict_hermeticity_regression_suite_plan(plan)?;
    serialize_canonical(&canonical, "strict hermeticity regression plan")
}

pub fn strict_hermeticity_regression_suite_plan_digest_blake3(
    plan: StrictHermeticityRegressionSuitePlan,
) -> Result<String, ReleaseEvidenceError> {
    let bytes = strict_hermeticity_regression_suite_plan_canonical_bytes(plan)?;
    Ok(blake3::hash(&bytes).to_hex().to_string())
}

pub fn evaluate_strict_hermeticity_regression_suite(
    run_id: String,
    plan: StrictHermeticityRegressionSuitePlan,
    evidence: Vec<StrictHermeticityRegressionCaseEvidence>,
) -> Result<StrictHermeticityRegressionSuiteReport, ReleaseEvidenceError> {
    validate_non_empty_string(StringField {
        value: &run_id,
        name: "strict regression run_id",
    })?;
    let plan = canonical_strict_hermeticity_regression_suite_plan(plan)?;
    let plan_digest_blake3 = strict_hermeticity_regression_suite_plan_digest_blake3(plan.clone())?;
    let mut evidence_by_case = collect_case_evidence(evidence)?;
    let mut case_outcomes = Vec::with_capacity(plan.cases.len());
    let mut suite_blockers = plan_axis_blockers(&plan);

    for case in &plan.cases {
        let observed = evidence_by_case.remove(&case.case_id);
        case_outcomes.push(evaluate_regression_case(case, observed)?);
    }
    suite_blockers.extend(unplanned_case_blockers(&evidence_by_case));
    suite_blockers.extend(case_outcomes.iter().flat_map(|case| case.blockers.iter().cloned()));

    let mut non_claims = case_outcomes.iter().flat_map(|case| case.non_claims.iter().cloned()).collect::<Vec<_>>();
    if !suite_blockers.is_empty() || !non_claims.is_empty() {
        non_claims.push(NON_CLAIM_STRICT_REGRESSION_BLOCKED.to_string());
    }
    let is_strict_regression_evidence_eligible = suite_blockers.is_empty()
        && non_claims.is_empty()
        && case_outcomes.iter().all(|case| case.verdict == case.expected_verdict);
    debug_assert_eq!(case_outcomes.len(), plan.cases.len());
    debug_assert!(evidence_by_case.is_empty() || !suite_blockers.is_empty());

    canonical_strict_hermeticity_regression_suite_report(StrictHermeticityRegressionSuiteReport {
        schema: STRICT_HERMETICITY_REGRESSION_SUITE_REPORT_SCHEMA.to_string(),
        run_id,
        suite_id: plan.suite_id,
        plan_version: plan.plan_version,
        plan_digest_blake3,
        cases: case_outcomes,
        strict_regression_evidence_eligible: is_strict_regression_evidence_eligible,
        blockers: suite_blockers,
        non_claims,
    })
}

pub fn canonical_strict_hermeticity_regression_suite_report(
    mut report: StrictHermeticityRegressionSuiteReport,
) -> Result<StrictHermeticityRegressionSuiteReport, ReleaseEvidenceError> {
    require_schema(SchemaRequirement {
        actual: &report.schema,
        expected: STRICT_HERMETICITY_REGRESSION_SUITE_REPORT_SCHEMA,
        field_name: "strict hermeticity regression report schema",
    })?;
    report.schema = STRICT_HERMETICITY_REGRESSION_SUITE_REPORT_SCHEMA.to_string();
    validate_non_empty_string(StringField {
        value: &report.run_id,
        name: "strict regression run_id",
    })?;
    validate_non_empty_string(StringField {
        value: &report.suite_id,
        name: "strict regression suite_id",
    })?;
    validate_non_empty_string(StringField {
        value: &report.plan_version,
        name: "strict regression plan_version",
    })?;
    validate_blake3_hex(&report.plan_digest_blake3, DIGEST_FIELD_PLAN)?;
    validate_case_count(report.cases.len(), "strict regression report cases")?;
    report.cases.sort_by(|left, right| left.case_id.cmp(&right.case_id));
    for case in &mut report.cases {
        canonicalize_case_report(case)?;
    }
    canonicalize_blockers(&mut report.blockers)?;
    canonicalize_string_vec(&mut report.non_claims, "strict regression report non_claims")?;
    debug_assert!(report.cases.windows(2).all(|pair| pair[0].case_id <= pair[1].case_id));
    debug_assert!(report.non_claims.windows(2).all(|pair| pair[0] < pair[1]));
    Ok(report)
}

pub fn strict_hermeticity_regression_suite_report_canonical_bytes(
    report: StrictHermeticityRegressionSuiteReport,
) -> Result<Vec<u8>, ReleaseEvidenceError> {
    let canonical = canonical_strict_hermeticity_regression_suite_report(report)?;
    serialize_canonical(&canonical, "strict hermeticity regression report")
}

pub fn strict_hermeticity_regression_suite_report_digest_blake3(
    report: StrictHermeticityRegressionSuiteReport,
) -> Result<String, ReleaseEvidenceError> {
    let bytes = strict_hermeticity_regression_suite_report_canonical_bytes(report)?;
    Ok(blake3::hash(&bytes).to_hex().to_string())
}

fn validate_plan_case(case: &mut StrictHermeticityRegressionCasePlan) -> Result<(), ReleaseEvidenceError> {
    debug_assert!(!STRICT_HERMETICITY_REGRESSION_SUITE_PLAN_SCHEMA.is_empty());
    debug_assert!(!REQUIRED_REGRESSION_AXES.is_empty());
    validate_non_empty_string(StringField {
        value: &case.case_id,
        name: "strict regression case_id",
    })?;
    validate_non_empty_string(StringField {
        value: &case.perturbation_axis,
        name: "strict regression perturbation_axis",
    })?;
    if case.mode != GauntletMode::Strict {
        return Err(validation_error(format!("strict regression case {} must run in strict mode", case.case_id)));
    }
    validate_expected_verdict(case)?;
    validate_optional_string(
        &case.required_blocker_or_audit_class,
        "strict regression required_blocker_or_audit_class",
    )?;
    validate_non_empty_string(StringField {
        value: &case.unsupported_host_non_claim,
        name: "strict regression unsupported_host_non_claim",
    })?;
    validate_optional_digest(&case.expected_output_digest_blake3, "strict regression expected_output_digest_blake3")?;
    Ok(())
}

fn validate_expected_verdict(case: &StrictHermeticityRegressionCasePlan) -> Result<(), ReleaseEvidenceError> {
    if case.perturbation_axis == AXIS_CLEAN_STRICT {
        if case.expected_verdict == GauntletVerdict::Accepted {
            return Ok(());
        }
        return Err(validation_error("clean strict regression case must expect accepted verdict".to_string()));
    }
    if case.expected_verdict == GauntletVerdict::Blocked || case.expected_verdict == GauntletVerdict::Rejected {
        return Ok(());
    }
    Err(validation_error(format!(
        "negative strict regression case {} must expect blocked or rejected verdict",
        case.case_id
    )))
}

fn collect_case_evidence(
    evidence: Vec<StrictHermeticityRegressionCaseEvidence>,
) -> Result<BTreeMap<String, StrictHermeticityRegressionCaseEvidence>, ReleaseEvidenceError> {
    let mut by_case = BTreeMap::new();
    validate_case_limit(evidence.len(), "strict regression evidence cases")?;
    for mut observed in evidence {
        validate_case_evidence(&mut observed)?;
        let case_id = observed.case_id.clone();
        if by_case.insert(case_id.clone(), observed).is_some() {
            return Err(validation_error(format!("duplicate strict regression evidence case_id: {case_id}")));
        }
    }
    Ok(by_case)
}

fn validate_case_evidence(evidence: &mut StrictHermeticityRegressionCaseEvidence) -> Result<(), ReleaseEvidenceError> {
    validate_non_empty_string(StringField {
        value: &evidence.case_id,
        name: "strict regression evidence case_id",
    })?;
    canonicalize_string_vec(
        &mut evidence.observed_blocker_or_audit_classes,
        "strict regression observed_blocker_or_audit_classes",
    )?;
    validate_optional_digest(&evidence.output_digest_blake3, "strict regression output_digest_blake3")?;
    validate_optional_digest(
        &evidence.repeated_output_digest_blake3,
        "strict regression repeated_output_digest_blake3",
    )?;
    Ok(())
}

fn evaluate_regression_case(
    plan: &StrictHermeticityRegressionCasePlan,
    evidence: Option<StrictHermeticityRegressionCaseEvidence>,
) -> Result<StrictHermeticityRegressionCaseReport, ReleaseEvidenceError> {
    let Some(evidence) = evidence else {
        return missing_case_report(plan);
    };
    let mut blockers = Vec::new();
    let mut non_claims = Vec::new();
    let is_stable_output_digest = output_digests_stable(&evidence);
    let verdict = classify_case_verdict(plan, &evidence);

    if evidence.unsupported_host {
        non_claims.push(plan.unsupported_host_non_claim.clone());
    } else {
        push_expected_verdict_blocker(plan, &evidence, verdict, &mut blockers);
        push_required_class_blocker(plan, &evidence, &mut blockers);
        push_clean_output_blockers(plan, &evidence, is_stable_output_digest, &mut blockers);
    }

    canonicalize_blockers(&mut blockers)?;
    canonicalize_string_vec(&mut non_claims, "strict regression case non_claims")?;
    let case_outcome = StrictHermeticityRegressionCaseReport {
        case_id: plan.case_id.clone(),
        perturbation_axis: plan.perturbation_axis.clone(),
        mode: plan.mode,
        expected_verdict: plan.expected_verdict,
        verdict,
        required_blocker_or_audit_class: plan.required_blocker_or_audit_class.clone(),
        observed_blocker_or_audit_classes: evidence.observed_blocker_or_audit_classes,
        unsupported_host: evidence.unsupported_host,
        unsupported_host_non_claim: plan.unsupported_host_non_claim.clone(),
        expected_output_digest_blake3: plan.expected_output_digest_blake3.clone(),
        output_digest_blake3: evidence.output_digest_blake3,
        repeated_output_digest_blake3: evidence.repeated_output_digest_blake3,
        stable_output_digest: is_stable_output_digest,
        blockers,
        non_claims,
    };
    debug_assert_eq!(case_outcome.case_id, plan.case_id);
    debug_assert_eq!(case_outcome.unsupported_host, evidence.unsupported_host);
    Ok(case_outcome)
}

fn missing_case_report(
    plan: &StrictHermeticityRegressionCasePlan,
) -> Result<StrictHermeticityRegressionCaseReport, ReleaseEvidenceError> {
    let mut blockers = vec![blocker(GauntletBlockerInput {
        evidence_class: "missing-strict-regression-evidence",
        message: "strict hermeticity regression case has no evidence",
        next_action: "run the fixture or record the host axis as an unsupported non-claim",
        axis: Some(plan.perturbation_axis.clone()),
        expected_digest_blake3: None,
        observed_digest_blake3: None,
    })];
    canonicalize_blockers(&mut blockers)?;
    debug_assert_eq!(blockers.len(), 1);
    debug_assert!(!plan.case_id.is_empty());
    Ok(StrictHermeticityRegressionCaseReport {
        case_id: plan.case_id.clone(),
        perturbation_axis: plan.perturbation_axis.clone(),
        mode: plan.mode,
        expected_verdict: plan.expected_verdict,
        verdict: GauntletVerdict::Missing,
        required_blocker_or_audit_class: plan.required_blocker_or_audit_class.clone(),
        observed_blocker_or_audit_classes: Vec::new(),
        unsupported_host: false,
        unsupported_host_non_claim: plan.unsupported_host_non_claim.clone(),
        expected_output_digest_blake3: plan.expected_output_digest_blake3.clone(),
        output_digest_blake3: None,
        repeated_output_digest_blake3: None,
        stable_output_digest: false,
        blockers,
        non_claims: vec![NON_CLAIM_STRICT_REGRESSION_BLOCKED.to_string()],
    })
}

fn classify_case_verdict(
    plan: &StrictHermeticityRegressionCasePlan,
    evidence: &StrictHermeticityRegressionCaseEvidence,
) -> GauntletVerdict {
    if evidence.unsupported_host {
        return GauntletVerdict::Unsupported;
    }
    if plan.perturbation_axis == AXIS_CLEAN_STRICT {
        return evidence.observed_verdict;
    }
    if evidence.observed_verdict == GauntletVerdict::Accepted {
        return GauntletVerdict::Accepted;
    }
    evidence.observed_verdict
}

fn push_expected_verdict_blocker(
    plan: &StrictHermeticityRegressionCasePlan,
    evidence: &StrictHermeticityRegressionCaseEvidence,
    verdict: GauntletVerdict,
    blockers: &mut Vec<GauntletBlocker>,
) {
    let blocker_count_before = blockers.len();
    debug_assert!(!plan.case_id.is_empty());
    debug_assert_eq!(plan.mode, GauntletMode::Strict);
    if verdict == plan.expected_verdict {
        return;
    }
    if plan.perturbation_axis != AXIS_CLEAN_STRICT && evidence.observed_verdict == GauntletVerdict::Accepted {
        blockers.push(blocker(GauntletBlockerInput {
            evidence_class: "strict-regression-poison-admitted",
            message: "poisoned strict hermeticity fixture was admitted",
            next_action: "keep strict proof admission blocked until the poisoned fixture fails closed",
            axis: Some(plan.perturbation_axis.clone()),
            expected_digest_blake3: None,
            observed_digest_blake3: None,
        }));
        return;
    }
    blockers.push(blocker(GauntletBlockerInput {
        evidence_class: "strict-regression-verdict-mismatch",
        message: "strict hermeticity regression verdict did not match the declared expected outcome",
        next_action: "inspect the fixture result and update the implementation or plan only with review",
        axis: Some(plan.perturbation_axis.clone()),
        expected_digest_blake3: None,
        observed_digest_blake3: None,
    }));
    debug_assert!(blockers.len() > blocker_count_before);
}

fn push_required_class_blocker(
    plan: &StrictHermeticityRegressionCasePlan,
    evidence: &StrictHermeticityRegressionCaseEvidence,
    blockers: &mut Vec<GauntletBlocker>,
) {
    let Some(required_class) = &plan.required_blocker_or_audit_class else {
        return;
    };
    if evidence.observed_blocker_or_audit_classes.iter().any(|class| class == required_class) {
        return;
    }
    blockers.push(blocker(GauntletBlockerInput {
        evidence_class: "missing-required-regression-class",
        message: "strict regression evidence omitted the required blocker or audit class",
        next_action: "record the class before treating this axis as covered",
        axis: Some(plan.perturbation_axis.clone()),
        expected_digest_blake3: None,
        observed_digest_blake3: None,
    }));
}

fn push_clean_output_blockers(
    plan: &StrictHermeticityRegressionCasePlan,
    evidence: &StrictHermeticityRegressionCaseEvidence,
    is_stable_output_digest: bool,
    blockers: &mut Vec<GauntletBlocker>,
) {
    let blocker_count_before = blockers.len();
    debug_assert!(!plan.case_id.is_empty());
    debug_assert_eq!(plan.mode, GauntletMode::Strict);
    if plan.perturbation_axis != AXIS_CLEAN_STRICT {
        return;
    }
    if evidence.output_digest_blake3.is_none() || evidence.repeated_output_digest_blake3.is_none() {
        blockers.push(blocker(GauntletBlockerInput {
            evidence_class: "missing-clean-output-digest",
            message: "clean strict fixture did not record both output digest observations",
            next_action: "record first and repeated output BLAKE3 digests for the clean fixture",
            axis: Some(plan.perturbation_axis.clone()),
            expected_digest_blake3: plan.expected_output_digest_blake3.clone(),
            observed_digest_blake3: evidence.output_digest_blake3.clone(),
        }));
        return;
    }
    if !is_stable_output_digest {
        blockers.push(blocker(GauntletBlockerInput {
            evidence_class: "strict-regression-output-unstable",
            message: "clean strict fixture output digest changed across repeated observations",
            next_action: "rerun after removing nondeterminism or keep the suite blocked",
            axis: Some(plan.perturbation_axis.clone()),
            expected_digest_blake3: evidence.output_digest_blake3.clone(),
            observed_digest_blake3: evidence.repeated_output_digest_blake3.clone(),
        }));
    }
    if plan.expected_output_digest_blake3.is_some()
        && plan.expected_output_digest_blake3 != evidence.output_digest_blake3
    {
        blockers.push(blocker(GauntletBlockerInput {
            evidence_class: "strict-regression-output-digest-mismatch",
            message: "clean strict fixture output digest does not match the suite plan",
            next_action: "update the fixture deliberately or investigate the drift before claiming readiness",
            axis: Some(plan.perturbation_axis.clone()),
            expected_digest_blake3: plan.expected_output_digest_blake3.clone(),
            observed_digest_blake3: evidence.output_digest_blake3.clone(),
        }));
    }
    debug_assert!(blockers.len() >= blocker_count_before);
}

fn output_digests_stable(evidence: &StrictHermeticityRegressionCaseEvidence) -> bool {
    match (&evidence.output_digest_blake3, &evidence.repeated_output_digest_blake3) {
        (Some(left), Some(right)) => left == right,
        _ => false,
    }
}

fn plan_axis_blockers(plan: &StrictHermeticityRegressionSuitePlan) -> Vec<GauntletBlocker> {
    let present_axes = plan.cases.iter().map(|case| case.perturbation_axis.as_str()).collect::<BTreeSet<_>>();
    REQUIRED_REGRESSION_AXES
        .iter()
        .filter(|axis| !present_axes.contains(**axis))
        .map(|axis| {
            blocker(GauntletBlockerInput {
                evidence_class: "missing-strict-regression-axis",
                message: "strict hermeticity regression suite plan omits a required axis",
                next_action: "add the required fixture or mark the suite as incomplete",
                axis: Some((*axis).to_string()),
                expected_digest_blake3: None,
                observed_digest_blake3: None,
            })
        })
        .collect()
}

fn unplanned_case_blockers(
    evidence_by_case: &BTreeMap<String, StrictHermeticityRegressionCaseEvidence>,
) -> Vec<GauntletBlocker> {
    evidence_by_case
        .keys()
        .map(|case_id| {
            blocker(GauntletBlockerInput {
                evidence_class: "unplanned-strict-regression-evidence",
                message: "strict regression evidence references a case absent from the suite plan",
                next_action: "add the case to the plan or drop the stray evidence before promotion",
                axis: Some(case_id.clone()),
                expected_digest_blake3: None,
                observed_digest_blake3: None,
            })
        })
        .collect()
}

fn canonicalize_case_report(case: &mut StrictHermeticityRegressionCaseReport) -> Result<(), ReleaseEvidenceError> {
    validate_non_empty_string(StringField {
        value: &case.case_id,
        name: "strict regression report case_id",
    })?;
    validate_non_empty_string(StringField {
        value: &case.perturbation_axis,
        name: "strict regression report perturbation_axis",
    })?;
    validate_optional_string(
        &case.required_blocker_or_audit_class,
        "strict regression report required_blocker_or_audit_class",
    )?;
    validate_non_empty_string(StringField {
        value: &case.unsupported_host_non_claim,
        name: "strict regression report unsupported_host_non_claim",
    })?;
    validate_optional_digest(
        &case.expected_output_digest_blake3,
        "strict regression report expected_output_digest_blake3",
    )?;
    validate_optional_digest(&case.output_digest_blake3, "strict regression report output_digest_blake3")?;
    validate_optional_digest(
        &case.repeated_output_digest_blake3,
        "strict regression report repeated_output_digest_blake3",
    )?;
    canonicalize_string_vec(
        &mut case.observed_blocker_or_audit_classes,
        "strict regression report observed_blocker_or_audit_classes",
    )?;
    canonicalize_blockers(&mut case.blockers)?;
    canonicalize_string_vec(&mut case.non_claims, "strict regression report case non_claims")?;
    debug_assert!(case.observed_blocker_or_audit_classes.windows(2).all(|pair| pair[0] < pair[1]));
    debug_assert!(case.non_claims.windows(2).all(|pair| pair[0] < pair[1]));
    Ok(())
}

fn validate_case_count(count: usize, field_name: &str) -> Result<(), ReleaseEvidenceError> {
    let count_u32 = u32_count(count, &format!("{field_name} count overflowed u32"))?;
    if count_u32 == ZERO_COUNT {
        return Err(validation_error(format!("{field_name} must include at least one entry")));
    }
    validate_case_limit_count(count_u32, field_name)
}

fn validate_case_limit(count: usize, field_name: &str) -> Result<(), ReleaseEvidenceError> {
    let count_u32 = u32_count(count, &format!("{field_name} count overflowed u32"))?;
    validate_case_limit_count(count_u32, field_name)
}

fn validate_case_limit_count(count_u32: u32, field_name: &str) -> Result<(), ReleaseEvidenceError> {
    if count_u32 > MAX_STRICT_REGRESSION_CASE_COUNT {
        return Err(validation_error(format!(
            "{field_name} has {count_u32} entries, limit is {MAX_STRICT_REGRESSION_CASE_COUNT}"
        )));
    }
    Ok(())
}

fn canonicalize_string_vec(values: &mut Vec<String>, field_name: &str) -> Result<(), ReleaseEvidenceError> {
    values.sort();
    values.dedup();
    validate_string_slice(values, field_name)
}

fn validate_string_slice(values: &[String], field_name: &str) -> Result<(), ReleaseEvidenceError> {
    let count_u32 = u32_count(values.len(), &format!("{field_name} count overflowed u32"))?;
    if count_u32 > MAX_STRICT_REGRESSION_CLASS_COUNT {
        return Err(validation_error(format!(
            "{field_name} has {count_u32} entries, limit is {MAX_STRICT_REGRESSION_CLASS_COUNT}"
        )));
    }
    for value in values {
        validate_non_empty_string(StringField {
            value,
            name: field_name,
        })?;
    }
    Ok(())
}

fn canonicalize_blockers(blockers: &mut Vec<GauntletBlocker>) -> Result<(), ReleaseEvidenceError> {
    let count_u32 = u32_count(blockers.len(), "strict regression blocker count overflowed u32")?;
    if count_u32 > MAX_STRICT_REGRESSION_CLASS_COUNT {
        return Err(validation_error(format!(
            "strict regression blockers has {count_u32} entries, limit is {MAX_STRICT_REGRESSION_CLASS_COUNT}"
        )));
    }
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
    debug_assert!(blockers.windows(2).all(|pair| pair[0] != pair[1]));
    debug_assert!(
        u32_count(blockers.len(), "strict regression blocker count overflowed u32")
            .is_ok_and(|count| count <= MAX_STRICT_REGRESSION_CLASS_COUNT)
    );
    Ok(())
}

fn validate_blocker(blocker: &GauntletBlocker) -> Result<(), ReleaseEvidenceError> {
    validate_non_empty_string(StringField {
        value: &blocker.evidence_class,
        name: "strict regression blocker.evidence_class",
    })?;
    validate_non_empty_string(StringField {
        value: &blocker.message,
        name: "strict regression blocker.message",
    })?;
    validate_non_empty_string(StringField {
        value: &blocker.next_action,
        name: "strict regression blocker.next_action",
    })?;
    validate_optional_string(&blocker.axis, "strict regression blocker.axis")?;
    validate_optional_digest(&blocker.expected_digest_blake3, "strict regression blocker.expected_digest_blake3")?;
    validate_optional_digest(&blocker.observed_digest_blake3, "strict regression blocker.observed_digest_blake3")?;
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
        validate_non_empty_string(StringField {
            value,
            name: field_name,
        })?;
    }
    Ok(())
}

fn validate_non_empty_string(field: StringField<'_>) -> Result<(), ReleaseEvidenceError> {
    if field.value.is_empty() {
        return Err(validation_error(format!("{} must not be empty", field.name)));
    }
    let byte_count = u32_count(field.value.len(), &format!("{} length overflowed u32", field.name))?;
    if byte_count > MAX_STRICT_REGRESSION_STRING_BYTES {
        return Err(validation_error(format!("{} exceeds {MAX_STRICT_REGRESSION_STRING_BYTES} bytes", field.name)));
    }
    Ok(())
}

fn require_schema(requirement: SchemaRequirement<'_>) -> Result<(), ReleaseEvidenceError> {
    if requirement.actual != requirement.expected {
        return Err(validation_error(format!(
            "{} must be {}, got {}",
            requirement.field_name, requirement.expected, requirement.actual
        )));
    }
    Ok(())
}

fn serialize_canonical<T: Serialize>(value: &T, label: &str) -> Result<Vec<u8>, ReleaseEvidenceError> {
    serde_json::to_vec(value).map_err(|err| ReleaseEvidenceError::Parse(format!("serializing {label}: {err}")))
}

fn blocker(input: GauntletBlockerInput<'_>) -> GauntletBlocker {
    GauntletBlocker {
        evidence_class: input.evidence_class.to_string(),
        message: input.message.to_string(),
        next_action: input.next_action.to_string(),
        axis: input.axis,
        expected_digest_blake3: input.expected_digest_blake3,
        observed_digest_blake3: input.observed_digest_blake3,
    }
}

#[cfg(test)]
mod tests {
    use alloc::vec;

    use pretty_assertions::assert_eq;

    use super::*;

    const DIGEST_ONE_SEED: u8 = 1;
    const DIGEST_TWO_SEED: u8 = 2;
    const NEGATIVE_AXIS_COUNT: usize = 8;
    const REQUIRED_AXIS_COUNT: usize = 9;
    const DEFAULT_UNSUPPORTED_NON_CLAIM_PREFIX: &str = "untested-axis";

    fn digest(seed: u8) -> String {
        let byte = format!("{:x}", seed % crate::manifest::BLAKE3_HEX_LENGTH_CHARS as u8);
        byte.repeat(crate::manifest::BLAKE3_HEX_LENGTH_CHARS)
    }

    fn required_class(axis: &str) -> String {
        format!("{axis}-blocked")
    }

    fn non_claim(axis: &str) -> String {
        format!("{DEFAULT_UNSUPPORTED_NON_CLAIM_PREFIX}:{axis}")
    }

    fn suite_plan() -> StrictHermeticityRegressionSuitePlan {
        let mut cases = vec![StrictHermeticityRegressionCasePlan {
            case_id: AXIS_CLEAN_STRICT.to_string(),
            perturbation_axis: AXIS_CLEAN_STRICT.to_string(),
            mode: GauntletMode::Strict,
            expected_verdict: GauntletVerdict::Accepted,
            required_blocker_or_audit_class: None,
            unsupported_host_non_claim: non_claim(AXIS_CLEAN_STRICT),
            expected_output_digest_blake3: Some(digest(DIGEST_ONE_SEED)),
        }];
        for axis in REQUIRED_REGRESSION_AXES.iter().copied().filter(|axis| *axis != AXIS_CLEAN_STRICT) {
            cases.push(StrictHermeticityRegressionCasePlan {
                case_id: axis.to_string(),
                perturbation_axis: axis.to_string(),
                mode: GauntletMode::Strict,
                expected_verdict: GauntletVerdict::Blocked,
                required_blocker_or_audit_class: Some(required_class(axis)),
                unsupported_host_non_claim: non_claim(axis),
                expected_output_digest_blake3: None,
            });
        }
        StrictHermeticityRegressionSuitePlan {
            schema: STRICT_HERMETICITY_REGRESSION_SUITE_PLAN_SCHEMA.to_string(),
            suite_id: "edit-time-strict-hermeticity".to_string(),
            plan_version: "2026-07-04".to_string(),
            cases,
        }
    }

    fn successful_evidence() -> Vec<StrictHermeticityRegressionCaseEvidence> {
        let mut evidence = vec![StrictHermeticityRegressionCaseEvidence {
            case_id: AXIS_CLEAN_STRICT.to_string(),
            observed_verdict: GauntletVerdict::Accepted,
            observed_blocker_or_audit_classes: Vec::new(),
            unsupported_host: false,
            output_digest_blake3: Some(digest(DIGEST_ONE_SEED)),
            repeated_output_digest_blake3: Some(digest(DIGEST_ONE_SEED)),
        }];
        for axis in REQUIRED_REGRESSION_AXES.iter().copied().filter(|axis| *axis != AXIS_CLEAN_STRICT) {
            evidence.push(StrictHermeticityRegressionCaseEvidence {
                case_id: axis.to_string(),
                observed_verdict: GauntletVerdict::Blocked,
                observed_blocker_or_audit_classes: vec![required_class(axis)],
                unsupported_host: false,
                output_digest_blake3: None,
                repeated_output_digest_blake3: None,
            });
        }
        evidence
    }

    #[test]
    fn strict_regression_suite_accepts_clean_and_poisoned_fixture_outcomes() {
        let plan = suite_plan();
        let expected_plan_digest = strict_hermeticity_regression_suite_plan_digest_blake3(plan.clone()).unwrap();
        let report = evaluate_strict_hermeticity_regression_suite(
            "strict-regression-run".to_string(),
            plan,
            successful_evidence(),
        )
        .unwrap();

        assert!(report.strict_regression_evidence_eligible);
        assert_eq!(report.plan_digest_blake3, expected_plan_digest);
        assert_eq!(report.cases.len(), REQUIRED_AXIS_COUNT);
        assert!(report.blockers.is_empty());
        assert!(report.non_claims.is_empty());
        assert!(report.cases.iter().find(|case| case.case_id == AXIS_CLEAN_STRICT).unwrap().stable_output_digest);
        assert_eq!(
            report.cases.iter().filter(|case| case.expected_verdict == GauntletVerdict::Blocked).count(),
            NEGATIVE_AXIS_COUNT
        );
    }

    #[test]
    fn strict_regression_suite_blocks_when_poisoned_fixture_is_admitted() {
        let mut evidence = successful_evidence();
        let path_poisoning = evidence
            .iter_mut()
            .find(|case| case.case_id == AXIS_PATH_POISONING)
            .expect("path poisoning evidence");
        path_poisoning.observed_verdict = GauntletVerdict::Accepted;
        path_poisoning.observed_blocker_or_audit_classes.clear();

        let report =
            evaluate_strict_hermeticity_regression_suite("bad-run".to_string(), suite_plan(), evidence).unwrap();

        assert!(!report.strict_regression_evidence_eligible);
        assert!(report.blockers.iter().any(|blocker| blocker.evidence_class == "missing-required-regression-class"));
        assert!(report.blockers.iter().any(|blocker| blocker.evidence_class == "strict-regression-poison-admitted"));
        assert!(report.non_claims.contains(&NON_CLAIM_STRICT_REGRESSION_BLOCKED.to_string()));
    }

    #[test]
    fn strict_regression_suite_marks_unsupported_axis_as_non_claim() {
        let mut evidence = successful_evidence();
        let network = evidence.iter_mut().find(|case| case.case_id == AXIS_NETWORK_ACCESS).expect("network evidence");
        network.unsupported_host = true;
        network.observed_verdict = GauntletVerdict::Unsupported;
        network.observed_blocker_or_audit_classes.clear();

        let report =
            evaluate_strict_hermeticity_regression_suite("unsupported-run".to_string(), suite_plan(), evidence)
                .unwrap();

        assert!(!report.strict_regression_evidence_eligible);
        assert_eq!(
            report.cases.iter().find(|case| case.case_id == AXIS_NETWORK_ACCESS).unwrap().verdict,
            GauntletVerdict::Unsupported
        );
        assert!(report.non_claims.contains(&non_claim(AXIS_NETWORK_ACCESS)));
        assert!(report.non_claims.contains(&NON_CLAIM_STRICT_REGRESSION_BLOCKED.to_string()));
    }

    #[test]
    fn strict_regression_suite_blocks_missing_required_axis() {
        let mut plan = suite_plan();
        plan.cases.retain(|case| case.perturbation_axis != AXIS_UMASK_DRIFT);
        let report =
            evaluate_strict_hermeticity_regression_suite("missing-axis-run".to_string(), plan, successful_evidence())
                .unwrap();

        assert!(!report.strict_regression_evidence_eligible);
        assert!(report.blockers.iter().any(|blocker| blocker.evidence_class == "missing-strict-regression-axis"));
        assert!(
            report
                .blockers
                .iter()
                .any(|blocker| blocker.evidence_class == "unplanned-strict-regression-evidence")
        );
    }

    #[test]
    fn strict_regression_suite_blocks_unstable_clean_output_digest() {
        let mut evidence = successful_evidence();
        let clean = evidence.iter_mut().find(|case| case.case_id == AXIS_CLEAN_STRICT).expect("clean evidence");
        clean.repeated_output_digest_blake3 = Some(digest(DIGEST_TWO_SEED));

        let report =
            evaluate_strict_hermeticity_regression_suite("unstable-run".to_string(), suite_plan(), evidence).unwrap();

        assert!(!report.strict_regression_evidence_eligible);
        assert!(report.blockers.iter().any(|blocker| blocker.evidence_class == "strict-regression-output-unstable"));
        assert!(!report.cases.iter().find(|case| case.case_id == AXIS_CLEAN_STRICT).unwrap().stable_output_digest);
    }
}
