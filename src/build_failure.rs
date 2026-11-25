use std::path::Path;

pub use crunch_build::BuildFailureClass as FailureClass;
pub use crunch_build::BuildFailurePhase as FailurePhase;
use crunch_build::FailedGoal;
use crunch_pipeline::PipelineResult;
use crunch_pipeline::label_for_key;
use serde::Serialize;

use crate::build_log::existing_log_file_path_from_drv_key;

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct BuildFailureEnvelope {
    pub root: String,
    pub drv_key: String,
    pub phase: FailurePhase,
    pub error_class: FailureClass,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub saved_log_path: Option<String>,
}

pub fn should_write_failure_log(failed: &FailedGoal) -> bool {
    failed.phase == FailurePhase::Build
}

pub fn build_failure_envelopes(result: &PipelineResult, store_dir: &str, logs_dir: &Path) -> Vec<BuildFailureEnvelope> {
    let failed_count = result.failed.len();
    let mut envelopes = result
        .failed
        .iter()
        .map(|failed| {
            let phase = failed.phase;
            let error_class = failed.error_class;
            let saved_log_path = if should_write_failure_log(failed) {
                existing_log_file_path_from_drv_key(logs_dir, store_dir, &failed.drv_key)
            } else {
                None
            };
            BuildFailureEnvelope {
                root: label_for_key(result, &failed.drv_key).unwrap_or(&failed.drv_key).to_string(),
                drv_key: failed.drv_key.clone(),
                phase,
                error_class,
                message: failed.error.clone(),
                saved_log_path,
            }
        })
        .collect::<Vec<_>>();
    envelopes.sort_by(|left, right| left.root.cmp(&right.root).then(left.drv_key.cmp(&right.drv_key)));
    debug_assert_eq!(envelopes.len(), failed_count);
    debug_assert!(envelopes.iter().all(|envelope| !envelope.drv_key.is_empty()));
    envelopes
}

pub fn render_human_failure_summary(envelope: &BuildFailureEnvelope) -> Vec<String> {
    let mut lines = Vec::with_capacity(5);
    lines.push(format!("FAILED root: {}", envelope.root));
    lines.push(format!("  phase: {}", envelope.phase.as_str()));
    lines.push(format!("  error_class: {}", envelope.error_class.as_str()));
    if let Some(path) = &envelope.saved_log_path {
        lines.push(format!("  saved_log_path: {path}"));
    }
    lines.push(format!("  message: {}", envelope.message));
    lines
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    fn sample_failed_goal(phase: FailurePhase, error_class: FailureClass, message: &str) -> FailedGoal {
        FailedGoal {
            drv_key: "/nix/store/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa-demo.drv".to_string(),
            origin_drv_key: "/nix/store/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa-demo.drv".to_string(),
            phase,
            error_class,
            fod_mismatch: None,
            error: message.to_string(),
            origin_error: message.to_string(),
            build_log: None,
        }
    }

    fn pipeline_result_with(failed: Vec<FailedGoal>) -> PipelineResult {
        let root_labels =
            failed.iter().map(|goal| (goal.drv_key.clone(), "demo".to_string())).collect::<HashMap<_, _>>();
        PipelineResult {
            outcomes: Vec::new(),
            failed,
            fod_mismatches: Vec::new(),
            root_labels,
            hermeticity_mode: crunch_pipeline::HermeticityMode::Strict,
            hermeticity_audit_events: Vec::new(),
            build_environment_reports: Vec::new(),
            finish_gate_reports: Vec::new(),
            network_policy_reports: Vec::new(),
            workspace_reports: Vec::new(),
            action_result_reports: Vec::new(),
            native_dynamic_plans: Vec::new(),
            plan_output_bindings: Vec::new(),
            priority_decisions: Vec::new(),
            pool_goal_observations: Vec::new(),
            overlay_report: None,
            store_layer_selections: Vec::new(),
        }
    }

    #[test]
    fn preflight_failure_omits_saved_log() {
        let envelope = BuildFailureEnvelope {
            root: "demo".to_string(),
            drv_key: "drv".to_string(),
            phase: FailurePhase::Preflight,
            error_class: FailureClass::Preflight,
            message: "opening store: broken".to_string(),
            saved_log_path: None,
        };
        let lines = render_human_failure_summary(&envelope);
        assert!(lines.iter().all(|line| !line.contains("saved_log_path")));
    }

    #[test]
    fn typed_classification_is_invariant_to_diagnostic_rewording() {
        let cases = [
            (FailurePhase::Preflight, FailureClass::Preflight, false),
            (FailurePhase::Preflight, FailureClass::UnsupportedPlatform, false),
            (FailurePhase::Build, FailureClass::FixedOutputHashMismatch, true),
            (FailurePhase::Build, FailureClass::Sandbox, true),
            (FailurePhase::Build, FailureClass::MissingOutput, true),
            (FailurePhase::Build, FailureClass::MissingSourceInput, true),
            (FailurePhase::Build, FailureClass::CacheVerification, true),
            (FailurePhase::Build, FailureClass::Builder, true),
        ];
        let logs_dir = tempfile::tempdir().unwrap();
        for (phase, error_class, expect_log) in cases {
            let reworded = sample_failed_goal(phase, error_class, "completely rephrased human message");
            assert_eq!(should_write_failure_log(&reworded), expect_log);
            let envelopes =
                build_failure_envelopes(&pipeline_result_with(vec![reworded]), "/nix/store", logs_dir.path());
            assert_eq!(envelopes.len(), 1);
            assert_eq!(envelopes[0].phase, phase);
            assert_eq!(envelopes[0].error_class, error_class);
            assert_eq!(envelopes[0].message, "completely rephrased human message");
        }
    }

    #[test]
    fn unrelated_builder_errors_containing_legacy_tokens_keep_builder_classification() {
        let logs_dir = tempfile::tempdir().unwrap();
        let legacy_tokens = [
            "compiler emitted warning mentioning bwrap and fusermount",
            "test fixture output: opening store: simulated text",
            "unit test log: trusted signature and unsigned PathInfo",
            "script echoed FOD hash mismatch in stdout",
            "readme says only supported on linux",
            "log mentions output not produced by build",
            "log mentions source input not found in store",
            "output store directory /foo does not exist",
        ];
        for message in legacy_tokens {
            let failed = sample_failed_goal(FailurePhase::Build, FailureClass::Builder, message);
            assert!(should_write_failure_log(&failed), "builder failure must allow log write: {message}");
            let envelopes = build_failure_envelopes(&pipeline_result_with(vec![failed]), "/nix/store", logs_dir.path());
            assert_eq!(envelopes[0].phase, FailurePhase::Build);
            assert_eq!(envelopes[0].error_class, FailureClass::Builder);
        }
    }
}
