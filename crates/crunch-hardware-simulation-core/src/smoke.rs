use alloc::collections::BTreeSet;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

use crate::digest::ACTION_REF_PREFIX;
use crate::digest::BoundedValue;
use crate::digest::COHORT_REF_PREFIX;
use crate::digest::LOG_REF_PREFIX;
use crate::digest::OBJECT_REF_PREFIX;
use crate::digest::PROFILE_REF_PREFIX;
use crate::digest::SIMULATOR_REF_PREFIX;
use crate::digest::TypedRefValidation;
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

pub struct SuccessfulSmokeResultInput<'a> {
    pub case: &'a SmokeCase,
    pub simulator_ref: String,
    pub action_ref: String,
    pub profile_ref: String,
    pub cohort_ref: String,
    pub source_refs: Vec<String>,
    pub maximum_text_bytes: u32,
}

#[derive(Clone, Copy)]
pub struct SmokeValidationLimits {
    pub maximum_text_bytes: u32,
    pub maximum_log_bytes: u32,
}

#[derive(Clone, Copy)]
struct TextValidation<'a> {
    value: &'a str,
    field: &'a str,
    maximum_bytes: u32,
}

pub fn successful_smoke_result(input: SuccessfulSmokeResultInput<'_>) -> Result<HardwareSmokeResult, Vec<String>> {
    let SuccessfulSmokeResultInput {
        case,
        simulator_ref,
        action_ref,
        profile_ref,
        cohort_ref,
        source_refs,
        maximum_text_bytes,
    } = input;
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
    let validation = validate_smoke_result(&result, SmokeValidationLimits {
        maximum_text_bytes,
        maximum_log_bytes: case.max_log_bytes,
    });
    if !validation.valid || !validation.publishable {
        return Err(validation.diagnostics);
    }
    debug_assert_eq!(result.verdict, SmokeVerdict::Pass);
    debug_assert_eq!(result.process_exit_code, 0);
    Ok(result)
}

pub fn bounded_log_ref(value: &str) -> Result<BoundedLogRef, Vec<String>> {
    let byte_count = u32::try_from(value.len()).map_err(|_| vec![String::from("log-byte-count-overflow")])?;
    let log_ref = digest_ref(LOG_REF_PREFIX, LOG_DOMAIN, &value).map_err(|error| vec![String::from(error.code())])?;
    debug_assert!(log_ref.starts_with(LOG_REF_PREFIX));
    debug_assert_eq!(u32::try_from(value.len()).ok(), Some(byte_count));
    Ok(BoundedLogRef {
        log_ref,
        byte_count,
        truncated: false,
    })
}

pub fn validate_smoke_result(result: &HardwareSmokeResult, limits: SmokeValidationLimits) -> SmokeValidation {
    let mut diagnostics = Vec::new();
    validate_smoke_content(result, limits.maximum_text_bytes, &mut diagnostics);
    validate_smoke_identity_refs(result, &mut diagnostics);
    validate_source_refs(&result.source_refs, &mut diagnostics);
    validate_log(&result.stdout, "stdout", limits.maximum_log_bytes, &mut diagnostics);
    validate_log(&result.stderr, "stderr", limits.maximum_log_bytes, &mut diagnostics);
    validate_non_claims(&result.non_claims, &mut diagnostics);
    validate_verdict_exit_agreement(result, &mut diagnostics);
    diagnostics.sort();
    diagnostics.dedup();
    let is_valid = diagnostics.is_empty();
    let is_publishable = is_valid && result.verdict == SmokeVerdict::Pass && result.process_exit_code == 0;
    debug_assert!(!is_publishable || is_valid);
    debug_assert!(!is_publishable || result.expected == result.observed);
    SmokeValidation {
        valid: is_valid,
        publishable: is_publishable,
        diagnostics,
    }
}

fn validate_smoke_content(result: &HardwareSmokeResult, maximum_text_bytes: u32, diagnostics: &mut Vec<String>) {
    let diagnostics_before = diagnostics.len();
    if result.schema != SMOKE_RESULT_SCHEMA {
        diagnostics.push(String::from("smoke-result-schema-unsupported"));
    }
    push_result(
        diagnostics,
        validate_identifier(BoundedValue {
            value: &result.case_id,
            field: "smoke-case-id",
            maximum_bytes: maximum_text_bytes,
        }),
    );
    for input in [
        TextValidation {
            value: &result.input,
            field: "smoke-input",
            maximum_bytes: maximum_text_bytes,
        },
        TextValidation {
            value: &result.expected,
            field: "smoke-expected",
            maximum_bytes: maximum_text_bytes,
        },
        TextValidation {
            value: &result.observed,
            field: "smoke-observed",
            maximum_bytes: maximum_text_bytes,
        },
    ] {
        validate_text(input, diagnostics);
    }
    debug_assert!(diagnostics.len() >= diagnostics_before);
    debug_assert!(diagnostics[diagnostics_before..].iter().all(|diagnostic| !diagnostic.is_empty()));
}

fn validate_smoke_identity_refs(result: &HardwareSmokeResult, diagnostics: &mut Vec<String>) {
    let diagnostics_before = diagnostics.len();
    for input in [
        TypedRefValidation {
            value: &result.simulator_ref,
            prefix: SIMULATOR_REF_PREFIX,
            field: "simulator-ref",
        },
        TypedRefValidation {
            value: &result.action_ref,
            prefix: ACTION_REF_PREFIX,
            field: "action-ref",
        },
        TypedRefValidation {
            value: &result.profile_ref,
            prefix: PROFILE_REF_PREFIX,
            field: "profile-ref",
        },
        TypedRefValidation {
            value: &result.cohort_ref,
            prefix: COHORT_REF_PREFIX,
            field: "cohort-ref",
        },
    ] {
        push_result(diagnostics, validate_typed_ref(input));
    }
    debug_assert!(diagnostics.len() >= diagnostics_before);
    debug_assert!(diagnostics[diagnostics_before..].iter().all(|diagnostic| !diagnostic.is_empty()));
}

fn validate_source_refs(source_refs: &[String], diagnostics: &mut Vec<String>) {
    if source_refs.is_empty() {
        diagnostics.push(String::from("smoke-source-refs-empty"));
        return;
    }
    let mut unique = BTreeSet::new();
    for source_ref in source_refs {
        push_result(
            diagnostics,
            validate_typed_ref(TypedRefValidation {
                value: source_ref,
                prefix: OBJECT_REF_PREFIX,
                field: "source-ref",
            }),
        );
        if !unique.insert(source_ref.as_str()) {
            diagnostics.push(String::from("smoke-source-ref-duplicate"));
        }
    }
    debug_assert!(unique.len() <= source_refs.len());
    debug_assert!(diagnostics.iter().all(|diagnostic| !diagnostic.is_empty()));
}

fn validate_verdict_exit_agreement(result: &HardwareSmokeResult, diagnostics: &mut Vec<String>) {
    let is_values_match = result.expected == result.observed;
    let is_exit_success = result.process_exit_code == 0;
    match result.verdict {
        SmokeVerdict::Pass if !is_values_match => diagnostics.push(String::from("smoke-pass-observation-mismatch")),
        SmokeVerdict::Pass if !is_exit_success => diagnostics.push(String::from("smoke-pass-exit-mismatch")),
        SmokeVerdict::Fail if is_values_match && is_exit_success => {
            diagnostics.push(String::from("smoke-fail-contradicts-success"));
        }
        SmokeVerdict::Pass | SmokeVerdict::Fail => {}
    }
    if is_values_match != is_exit_success {
        diagnostics.push(String::from("smoke-verdict-exit-observation-disagreement"));
    }
    debug_assert_eq!(is_values_match, result.expected.as_bytes() == result.observed.as_bytes());
    debug_assert_eq!(is_exit_success, result.process_exit_code == 0);
}

fn validate_text(input: TextValidation<'_>, diagnostics: &mut Vec<String>) {
    if input.value.is_empty() {
        diagnostics.push(alloc::format!("{}-empty", input.field));
    }
    let is_too_large = match u32::try_from(input.value.len()) {
        Ok(value_bytes) => value_bytes > input.maximum_bytes,
        Err(_) => true,
    };
    if is_too_large {
        diagnostics.push(alloc::format!("{}-too-large", input.field));
    }
}

fn validate_log(log: &crate::model::BoundedLogRef, field: &str, maximum_bytes: u32, diagnostics: &mut Vec<String>) {
    push_result(
        diagnostics,
        validate_typed_ref(TypedRefValidation {
            value: &log.log_ref,
            prefix: LOG_REF_PREFIX,
            field,
        }),
    );
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

fn push_result<E: core::fmt::Display>(diagnostics: &mut Vec<String>, result: Result<(), E>) {
    if let Err(error) = result {
        diagnostics.push(alloc::format!("{error}"));
    }
}
