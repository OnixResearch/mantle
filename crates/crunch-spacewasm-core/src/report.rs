use alloc::collections::BTreeSet;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

use serde::Deserialize;
use serde::Serialize;

use crate::Blake3Digest;
use crate::CheckDecision;
use crate::Diagnostic;
use crate::ReferenceProfile;
use crate::SourceAdmission;
use crate::SupportComparison;
use crate::diagnostic::error;
use crate::diagnostic::has_errors;
use crate::diagnostic::ordered;
use crate::digest::canonical_identity;
use crate::profile::REQUIRED_NON_CLAIMS;

pub const MATERIALIZATION_REPORT_SCHEMA: &str = "mantle-spacewasm-materialization-report-v1";
pub const REFERENCE_CLAIM_CLASS: &str = "exact-reference-materialization-facts";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ReportDisposition {
    Complete,
    Incomplete,
    Rejected,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReportBuildInput {
    pub profile: ReferenceProfile,
    pub profile_identity_blake3: Blake3Digest,
    pub cohort_identity_blake3: Blake3Digest,
    pub source_admission: SourceAdmission,
    pub support_comparison: SupportComparison,
    pub checks: Vec<CheckDecision>,
    pub check_evaluation_complete: bool,
    pub diagnostics: Vec<Diagnostic>,
    pub requested_claim_class: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MaterializationReport {
    pub schema: String,
    pub profile_id: String,
    pub profile_identity_blake3: Blake3Digest,
    pub cohort_identity_blake3: Blake3Digest,
    pub disposition: ReportDisposition,
    pub source_admitted: bool,
    pub support_matches_profile: bool,
    pub checks: Vec<CheckDecision>,
    pub diagnostics: Vec<Diagnostic>,
    pub claim_class: String,
    pub non_claims: Vec<String>,
    pub report_identity_blake3: Blake3Digest,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReportValidation {
    pub valid: bool,
    pub diagnostics: Vec<Diagnostic>,
}

pub fn build_materialization_report(mut input: ReportBuildInput) -> Result<MaterializationReport, Vec<Diagnostic>> {
    let mut diagnostics = input.diagnostics.clone();
    diagnostics.extend(input.source_admission.diagnostics.clone());
    diagnostics.extend(input.support_comparison.diagnostics.clone());
    if input.requested_claim_class != REFERENCE_CLAIM_CLASS {
        diagnostics.push(error(
            "unsupported-claim-promotion",
            "report.claim-class",
            "SpaceWasm reference evidence cannot be promoted beyond exact materialization facts",
        ));
    }
    input.checks.sort_by(|left, right| left.check_id.cmp(&right.check_id));
    let diagnostics = ordered(diagnostics);
    let disposition = report_disposition(&input, &diagnostics);
    let mut non_claims = input.profile.non_claims.clone();
    non_claims.sort();
    let identity_input = ReportIdentityInput {
        schema: String::from(MATERIALIZATION_REPORT_SCHEMA),
        profile_id: input.profile.profile_id.clone(),
        profile_identity_blake3: input.profile_identity_blake3.clone(),
        cohort_identity_blake3: input.cohort_identity_blake3.clone(),
        disposition,
        source_admitted: input.source_admission.admitted,
        support_matches_profile: input.support_comparison.matches_profile,
        checks: input.checks.clone(),
        diagnostics: diagnostics.clone(),
        claim_class: input.requested_claim_class.clone(),
        non_claims: non_claims.clone(),
    };
    if has_errors(&diagnostics) && input.requested_claim_class != REFERENCE_CLAIM_CLASS {
        return Err(diagnostics);
    }
    let report_identity_blake3 = canonical_identity(identity_input).map_err(|_| {
        vec![error(
            "report-identity-failed",
            "report",
            "materialization report could not be canonically identified",
        )]
    })?;
    debug_assert!(non_claims.windows(crate::ADJACENT_WINDOW_LENGTH).all(|pair| pair[0] <= pair[1]));
    debug_assert!(!input.requested_claim_class.is_empty());
    Ok(MaterializationReport {
        schema: String::from(MATERIALIZATION_REPORT_SCHEMA),
        profile_id: input.profile.profile_id,
        profile_identity_blake3: input.profile_identity_blake3,
        cohort_identity_blake3: input.cohort_identity_blake3,
        disposition,
        source_admitted: input.source_admission.admitted,
        support_matches_profile: input.support_comparison.matches_profile,
        checks: input.checks,
        diagnostics,
        claim_class: input.requested_claim_class,
        non_claims,
        report_identity_blake3,
    })
}

pub fn validate_materialization_report(report: MaterializationReport) -> ReportValidation {
    let mut diagnostics = Vec::new();
    if report.schema != MATERIALIZATION_REPORT_SCHEMA {
        diagnostics.push(error(
            "unsupported-report-schema",
            "report.schema",
            "materialization report schema is unsupported",
        ));
    }
    if report.claim_class != REFERENCE_CLAIM_CLASS {
        diagnostics.push(error(
            "unsupported-claim-promotion",
            "report.claim-class",
            "report requests an unsupported claim class",
        ));
    }
    validate_report_non_claims(&report.non_claims, &mut diagnostics);
    let expected_identity = report.report_identity_blake3.clone();
    let identity = canonical_identity(identity_input_from_report(report.clone()));
    if identity.as_ref() != Ok(&expected_identity) {
        diagnostics.push(error("report-tamper", "report", "report identity does not match canonical report fields"));
    }
    let diagnostics = ordered(diagnostics);
    let valid = !has_errors(&diagnostics);
    debug_assert_eq!(valid, diagnostics.is_empty());
    debug_assert!(diagnostics.iter().all(|item| !item.code.is_empty()));
    ReportValidation { valid, diagnostics }
}

fn report_disposition(input: &ReportBuildInput, diagnostics: &[Diagnostic]) -> ReportDisposition {
    if has_errors(diagnostics) {
        return ReportDisposition::Rejected;
    }
    if input.source_admission.admitted && input.support_comparison.matches_profile && input.check_evaluation_complete {
        return ReportDisposition::Complete;
    }
    ReportDisposition::Incomplete
}

fn validate_report_non_claims(non_claims: &[String], diagnostics: &mut Vec<Diagnostic>) {
    let values: BTreeSet<_> = non_claims.iter().map(String::as_str).collect();
    for required in REQUIRED_NON_CLAIMS {
        if !values.contains(required) {
            diagnostics.push(error("missing-required-non-claim", required, "report omits a required claim boundary"));
        }
    }
    if values.len() != non_claims.len() {
        diagnostics.push(error("duplicate-non-claim", "report.non-claims", "report non-claims must be unique"));
    }
    debug_assert!(values.len() <= non_claims.len());
    debug_assert!(non_claims.is_empty() || !values.is_empty());
}

#[derive(Serialize)]
struct ReportIdentityInput {
    schema: String,
    profile_id: String,
    profile_identity_blake3: Blake3Digest,
    cohort_identity_blake3: Blake3Digest,
    disposition: ReportDisposition,
    source_admitted: bool,
    support_matches_profile: bool,
    checks: Vec<CheckDecision>,
    diagnostics: Vec<Diagnostic>,
    claim_class: String,
    non_claims: Vec<String>,
}

fn identity_input_from_report(mut report: MaterializationReport) -> ReportIdentityInput {
    report.checks.sort_by(|left, right| left.check_id.cmp(&right.check_id));
    report.diagnostics = ordered(report.diagnostics);
    report.non_claims.sort();
    ReportIdentityInput {
        schema: report.schema,
        profile_id: report.profile_id,
        profile_identity_blake3: report.profile_identity_blake3,
        cohort_identity_blake3: report.cohort_identity_blake3,
        disposition: report.disposition,
        source_admitted: report.source_admitted,
        support_matches_profile: report.support_matches_profile,
        checks: report.checks,
        diagnostics: report.diagnostics,
        claim_class: report.claim_class,
        non_claims: report.non_claims,
    }
}
