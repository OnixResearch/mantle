use alloc::collections::BTreeSet;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

use crate::digest::ACTION_REF_PREFIX;
use crate::digest::COHORT_REF_PREFIX;
use crate::digest::LOG_REF_PREFIX;
use crate::digest::OBJECT_REF_PREFIX;
use crate::digest::PROFILE_REF_PREFIX;
use crate::digest::SIMULATOR_REF_PREFIX;
use crate::digest::digest_ref;
use crate::digest::validate_identifier;
use crate::digest::validate_typed_ref;
use crate::model::BoundedLogRef;
use crate::model::HardwareSmokeResult;
use crate::model::SMOKE_RESULT_SCHEMA;
use crate::model::SmokeCase;
use crate::model::SmokeValidation;
use crate::model::SmokeVerdict;
use crate::profile::REQUIRED_NON_CLAIMS;

const LOG_DOMAIN: &[u8] = b"mantle.hardware.log.v1";

pub fn successful_smoke_result(
    case: &SmokeCase,
    simulator_ref: String,
    action_ref: String,
    profile_ref: String,
    cohort_ref: String,
    source_refs: Vec<String>,
    maximum_text_bytes: u32,
) -> Result<HardwareSmokeResult, Vec<String>> {
    let result = HardwareSmokeResult {
        schema: String::from(SMOKE_RESULT_SCHEMA),
        case_id: case.id.clone(),
        input: case.input.clone(),
        expected: case.expected.clone(),
        observed: case.expected.clone(),
        verdict: SmokeVerdict::Pass,
        process_exit_code: 0,
        simulator_ref,
        action_ref,
        profile_ref,
        cohort_ref,
        source_refs,
        stdout: bounded_log_ref(&case.expected)?,
        stderr: bounded_log_ref("")?,
        non_claims: REQUIRED_NON_CLAIMS.iter().map(|value| String::from(*value)).collect(),
    };
    let validation = validate_smoke_result(&result, maximum_text_bytes, case.max_log_bytes);
    if !validation.valid || !validation.publishable {
        return Err(validation.diagnostics);
    }
    debug_assert_eq!(result.verdict, SmokeVerdict::Pass);
    debug_assert_eq!(result.process_exit_code, 0);
    Ok(result)
}

pub fn bounded_log_ref(value: &str) -> Result<BoundedLogRef, Vec<String>> {
    let byte_count = u32::try_from(value.len()).map_err(|_| vec![String::from("log-byte-count-overflow")])?;
    let log_ref = digest_ref(LOG_REF_PREFIX, LOG_DOMAIN, &value).map_err(|error| vec![error])?;
    debug_assert!(log_ref.starts_with(LOG_REF_PREFIX));
    debug_assert_eq!(usize::try_from(byte_count).unwrap_or(usize::MAX), value.len());
    Ok(BoundedLogRef {
        log_ref,
        byte_count,
        truncated: false,
    })
}

pub fn validate_smoke_result(
    result: &HardwareSmokeResult,
    maximum_text_bytes: u32,
    maximum_log_bytes: u32,
) -> SmokeValidation {
    let mut diagnostics = Vec::new();
    if result.schema != SMOKE_RESULT_SCHEMA {
        diagnostics.push(String::from("smoke-result-schema-unsupported"));
    }
    push_result(&mut diagnostics, validate_identifier(&result.case_id, "smoke-case-id", maximum_text_bytes));
    validate_text(&result.input, "smoke-input", maximum_text_bytes, &mut diagnostics);
    validate_text(&result.expected, "smoke-expected", maximum_text_bytes, &mut diagnostics);
    validate_text(&result.observed, "smoke-observed", maximum_text_bytes, &mut diagnostics);
    push_result(&mut diagnostics, validate_typed_ref(&result.simulator_ref, SIMULATOR_REF_PREFIX, "simulator-ref"));
    push_result(&mut diagnostics, validate_typed_ref(&result.action_ref, ACTION_REF_PREFIX, "action-ref"));
    push_result(&mut diagnostics, validate_typed_ref(&result.profile_ref, PROFILE_REF_PREFIX, "profile-ref"));
    push_result(&mut diagnostics, validate_typed_ref(&result.cohort_ref, COHORT_REF_PREFIX, "cohort-ref"));
    validate_source_refs(&result.source_refs, &mut diagnostics);
    validate_log(&result.stdout, "stdout", maximum_log_bytes, &mut diagnostics);
    validate_log(&result.stderr, "stderr", maximum_log_bytes, &mut diagnostics);
    validate_non_claims(&result.non_claims, &mut diagnostics);
    validate_verdict_exit_agreement(result, &mut diagnostics);
    diagnostics.sort();
    diagnostics.dedup();
    let valid = diagnostics.is_empty();
    let publishable = valid && result.verdict == SmokeVerdict::Pass && result.process_exit_code == 0;
    debug_assert!(!publishable || valid);
    debug_assert!(!publishable || result.expected == result.observed);
    SmokeValidation {
        valid,
        publishable,
        diagnostics,
    }
}

fn validate_source_refs(source_refs: &[String], diagnostics: &mut Vec<String>) {
    if source_refs.is_empty() {
        diagnostics.push(String::from("smoke-source-refs-empty"));
        return;
    }
    let mut unique = BTreeSet::new();
    for source_ref in source_refs {
        push_result(diagnostics, validate_typed_ref(source_ref, OBJECT_REF_PREFIX, "source-ref"));
        if !unique.insert(source_ref.as_str()) {
            diagnostics.push(String::from("smoke-source-ref-duplicate"));
        }
    }
    debug_assert!(unique.len() <= source_refs.len());
    debug_assert!(diagnostics.iter().all(|diagnostic| !diagnostic.is_empty()));
}

fn validate_verdict_exit_agreement(result: &HardwareSmokeResult, diagnostics: &mut Vec<String>) {
    let values_match = result.expected == result.observed;
    let exit_success = result.process_exit_code == 0;
    match result.verdict {
        SmokeVerdict::Pass if !values_match => diagnostics.push(String::from("smoke-pass-observation-mismatch")),
        SmokeVerdict::Pass if !exit_success => diagnostics.push(String::from("smoke-pass-exit-mismatch")),
        SmokeVerdict::Fail if values_match && exit_success => {
            diagnostics.push(String::from("smoke-fail-contradicts-success"))
        }
        SmokeVerdict::Pass | SmokeVerdict::Fail => {}
    }
    if values_match != exit_success {
        diagnostics.push(String::from("smoke-verdict-exit-observation-disagreement"));
    }
    debug_assert_eq!(values_match, result.expected.as_bytes() == result.observed.as_bytes());
    debug_assert_eq!(exit_success, result.process_exit_code == 0);
}

fn validate_text(value: &str, field: &str, maximum_bytes: u32, diagnostics: &mut Vec<String>) {
    if value.is_empty() {
        diagnostics.push(alloc::format!("{field}-empty"));
    }
    if value.len() > usize::try_from(maximum_bytes).unwrap_or(usize::MAX) {
        diagnostics.push(alloc::format!("{field}-too-large"));
    }
}

fn validate_log(log: &crate::model::BoundedLogRef, field: &str, maximum_bytes: u32, diagnostics: &mut Vec<String>) {
    push_result(diagnostics, validate_typed_ref(&log.log_ref, LOG_REF_PREFIX, field));
    if log.byte_count > maximum_bytes {
        diagnostics.push(alloc::format!("{field}-byte-bound-exceeded"));
    }
    if log.truncated && log.byte_count != maximum_bytes {
        diagnostics.push(alloc::format!("{field}-truncation-bound-mismatch"));
    }
    debug_assert!(log.byte_count <= maximum_bytes || !diagnostics.is_empty());
    debug_assert!(!log.log_ref.is_empty());
}

fn validate_non_claims(non_claims: &[String], diagnostics: &mut Vec<String>) {
    let declared = non_claims.iter().map(String::as_str).collect::<BTreeSet<_>>();
    for required in REQUIRED_NON_CLAIMS {
        if !declared.contains(required) {
            diagnostics.push(alloc::format!("smoke-non-claim-missing:{required}"));
        }
    }
}

fn push_result(diagnostics: &mut Vec<String>, result: Result<(), String>) {
    if let Err(error) = result {
        diagnostics.push(error);
    }
}
