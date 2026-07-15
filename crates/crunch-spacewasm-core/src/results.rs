use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;

use serde::Deserialize;
use serde::Serialize;

use crate::Blake3Digest;
use crate::CheckStatus;
use crate::Diagnostic;
use crate::ObservedCheck;
use crate::ReferenceProfile;
use crate::diagnostic::ErrorDiagnostic;
use crate::diagnostic::MAX_DIAGNOSTIC_COUNT;
use crate::diagnostic::error;
use crate::diagnostic::has_errors;
use crate::diagnostic::ordered;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CheckDecision {
    pub check_id: String,
    pub expected_status: CheckStatus,
    pub observed_status: CheckStatus,
    pub result_code: String,
    pub matched: bool,
    pub command_blake3: Blake3Digest,
    pub configuration_blake3: Blake3Digest,
    pub input_blake3: Blake3Digest,
    pub output_blake3: Option<Blake3Digest>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CheckEvaluation {
    pub complete: bool,
    pub decisions: Vec<CheckDecision>,
    pub diagnostics: Vec<Diagnostic>,
}

pub fn evaluate_checks(profile: ReferenceProfile, observed: Vec<ObservedCheck>) -> CheckEvaluation {
    let reserved_diagnostic_count = profile.checks.len().saturating_add(observed.len()).min(MAX_DIAGNOSTIC_COUNT);
    let mut diagnostics = Vec::with_capacity(reserved_diagnostic_count);
    let mut observed_by_id = BTreeMap::new();
    for check in observed {
        let check_id = check.check_id.clone();
        if check_id.is_empty() || observed_by_id.insert(check_id.clone(), check).is_some() {
            diagnostics.push(error(ErrorDiagnostic {
                code: "duplicate-observed-check",
                subject: &check_id,
                message: "observed check ids must be unique and non-empty",
            }));
        }
    }
    let mut decisions = Vec::with_capacity(profile.checks.len());
    for expected in &profile.checks {
        let Some(check) = observed_by_id.get(&expected.check_id) else {
            diagnostics.push(error(ErrorDiagnostic {
                code: "missing-declared-check",
                subject: &expected.check_id,
                message: "declared check has no recorded outcome",
            }));
            continue;
        };
        let is_matched = check.status == expected.expected_status;
        if !is_matched {
            diagnostics.push(error(ErrorDiagnostic {
                code: "check-status-mismatch",
                subject: &expected.check_id,
                message: "recorded check status differs from the declared expected status",
            }));
        }
        decisions.push(CheckDecision {
            check_id: expected.check_id.clone(),
            expected_status: expected.expected_status,
            observed_status: check.status,
            result_code: check.result_code.clone(),
            matched: is_matched,
            command_blake3: check.command_blake3.clone(),
            configuration_blake3: check.configuration_blake3.clone(),
            input_blake3: check.input_blake3.clone(),
            output_blake3: check.output_blake3.clone(),
        });
    }
    for check_id in observed_by_id.keys() {
        if !profile.checks.iter().any(|expected| &expected.check_id == check_id) {
            diagnostics.push(error(ErrorDiagnostic {
                code: "undeclared-check-result",
                subject: check_id,
                message: "recorded result does not correspond to a declared check",
            }));
        }
    }
    decisions.sort_by(|left, right| left.check_id.cmp(&right.check_id));
    let diagnostics = ordered(diagnostics);
    let has_all_decisions = decisions.len() == profile.checks.len();
    let is_complete = !has_errors(&diagnostics) && has_all_decisions;
    debug_assert!(decisions.windows(crate::ADJACENT_WINDOW_LENGTH).all(|pair| pair[0].check_id <= pair[1].check_id));
    debug_assert_eq!(is_complete, has_all_decisions && diagnostics.is_empty());
    CheckEvaluation {
        complete: is_complete,
        decisions,
        diagnostics,
    }
}
