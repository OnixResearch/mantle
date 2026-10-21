//! Evaluation application: execute, verify output/report evidence, then render.
use std::path::Path;
use std::path::PathBuf;

use crate::RunError;
use crate::evaluator_budget;

pub(crate) fn stream_plan() -> Result<mantle_application_contract::EffectPlan, RunError> {
    mantle_application_contract::evaluation::evaluation_effect_plan(
        mantle_application_contract::evaluation::EvaluationEffect::Output,
    )
    .map_err(|error| RunError::Internal(format!("planning evaluation stream: {}", error.code())))
}

/// The native ledger's terminal state is evaluation/conversion evidence, not a build outcome.
pub(crate) fn classify_stream_record(
    plan: &mantle_application_contract::EffectPlan,
    record: &crunch_evaluation_stream_core::StreamRecordValue,
) -> Result<(), RunError> {
    use crunch_evaluation_stream_core::RunDisposition;
    use crunch_evaluation_stream_core::StreamRecordValue;
    use crunch_evaluation_stream_core::TerminalState;
    let succeeded = match record {
        StreamRecordValue::RootTerminal(root) => root.terminal_state() == TerminalState::Succeeded,
        StreamRecordValue::RunSummary(summary) => summary.disposition() == RunDisposition::Success,
        StreamRecordValue::RunStart(_) | StreamRecordValue::RootDiscovered(_) => return Ok(()),
    };
    let effect = mantle_application_contract::evaluation::EvaluationEffect::Output;
    crate::build_observation::classify(plan, effect.identity(), effect.kind(), succeeded)
}

pub(crate) fn run(
    file: &Path,
    import_paths: &[PathBuf],
    budget_policy: Option<&Path>,
    budget_report: Option<&Path>,
    roots: &[String],
    all_roots: bool,
) -> Result<(), RunError> {
    if budget_policy.is_some() != budget_report.is_some() {
        return Err(RunError::Eval("evaluation budget policy and report path must be provided together".to_string()));
    }
    let output = evaluate(file, import_paths, budget_policy.zip(budget_report), roots, all_roots)?;
    println!("{output}");
    Ok(())
}

pub(crate) fn verify_json(json: &str) -> Result<(), RunError> {
    serde_json::from_str::<serde::de::IgnoredAny>(json)
        .map(|_| ())
        .map_err(|error| RunError::Eval(format!("evaluation returned invalid JSON: {error}")))
}

fn evaluate(
    file: &Path,
    import_paths: &[PathBuf],
    budget: Option<(&Path, &Path)>,
    roots: &[String],
    all_roots: bool,
) -> Result<String, RunError> {
    let Some((policy, report)) = budget else {
        let effect = mantle_application_contract::evaluation::EvaluationEffect::Output;
        let plan = mantle_application_contract::evaluation::evaluation_effect_plan(effect)
            .map_err(|error| RunError::Internal(format!("planning evaluation: {}", error.code())))?;
        let output = (|| {
            let search_dirs = crate::build_cmd::build_import_paths(import_paths)?;
            let json =
                crunch_eval::evaluate_to_json(file, &search_dirs).map_err(|error| RunError::Eval(error.to_string()))?;
            verify_json(&json)?;
            Ok(json)
        })();
        crate::build_observation::classify(&plan, effect.identity(), effect.kind(), output.is_ok())?;
        return output;
    };
    let selection = if all_roots {
        evaluator_budget::EvaluationSelection::AllRoots
    } else if roots.is_empty() {
        evaluator_budget::EvaluationSelection::WholeValue
    } else {
        evaluator_budget::EvaluationSelection::SelectedRoots(roots.to_vec())
    };
    let evaluated = evaluator_budget::run_budgeted_file(file, import_paths, policy, report, selection)?;
    if let Some(error) = evaluator_budget::terminal_failure(&evaluated) {
        return Err(error);
    }
    evaluated
        .output_json
        .ok_or_else(|| RunError::Internal("evaluation worker succeeded without output".to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn json_scalar_null_and_empty_collections_are_valid_outputs() {
        for json in ["null", "false", "0", "\"\"", "[]", "{}"] {
            assert!(verify_json(json).is_ok(), "valid evaluator value {json}");
        }
    }

    #[test]
    fn truncated_and_multiple_values_are_not_successful_evaluations() {
        for json in ["", "{", "null null", "[1,]"] {
            assert!(verify_json(json).is_err(), "invalid evaluator value {json}");
        }
    }
}
