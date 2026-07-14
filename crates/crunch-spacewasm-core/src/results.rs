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
    let mut diagnostics = Vec::new();
    let mut observed_by_id = BTreeMap::new();
    for check in observed {
        let check_id = check.check_id.clone();
        if check_id.is_empty() || observed_by_id.insert(check_id.clone(), check).is_some() {
            diagnostics.push(error(
                "duplicate-observed-check",
                &check_id,
                "observed check ids must be unique and non-empty",
            ));
        }
    }
    let mut decisions = Vec::new();
    for expected in &profile.checks {
        let Some(check) = observed_by_id.get(&expected.check_id) else {
            diagnostics.push(error(
                "missing-declared-check",
                &expected.check_id,
                "declared check has no recorded outcome",
            ));
            continue;
        };
        let matched = check.status == expected.expected_status;
        if !matched {
            diagnostics.push(error(
                "check-status-mismatch",
                &expected.check_id,
                "recorded check status differs from the declared expected status",
            ));
        }
        decisions.push(CheckDecision {
            check_id: expected.check_id.clone(),
            expected_status: expected.expected_status,
            observed_status: check.status,
            result_code: check.result_code.clone(),
            matched,
            command_blake3: check.command_blake3.clone(),
            configuration_blake3: check.configuration_blake3.clone(),
            input_blake3: check.input_blake3.clone(),
            output_blake3: check.output_blake3.clone(),
        });
    }
    for check_id in observed_by_id.keys() {
        if !profile.checks.iter().any(|expected| &expected.check_id == check_id) {
            diagnostics.push(error(
                "undeclared-check-result",
                check_id,
                "recorded result does not correspond to a declared check",
            ));
        }
    }
    decisions.sort_by(|left, right| left.check_id.cmp(&right.check_id));
    let diagnostics = ordered(diagnostics);
    let complete = !has_errors(&diagnostics) && decisions.len() == profile.checks.len();
    debug_assert!(decisions.windows(crate::ADJACENT_WINDOW_LENGTH).all(|pair| pair[0].check_id <= pair[1].check_id));
    debug_assert_eq!(complete, decisions.len() == profile.checks.len() && diagnostics.is_empty());
    CheckEvaluation {
        complete,
        decisions,
        diagnostics,
    }
}
