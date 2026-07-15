use std::path::Path;

use crunch_pipeline::PipelineResult;
use crunch_pipeline::label_for_key;
use serde::Serialize;

use crate::build_log::existing_log_file_path_from_drv_key;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum FailurePhase {
    Preflight,
    Build,
}

impl FailurePhase {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Preflight => "preflight",
            Self::Build => "build",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum FailureClass {
    Preflight,
    FixedOutputHashMismatch,
    Sandbox,
    MissingOutput,
    MissingSourceInput,
    CacheVerification,
    UnsupportedPlatform,
    Builder,
}

impl FailureClass {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Preflight => "preflight",
            Self::FixedOutputHashMismatch => "fixed-output-hash-mismatch",
            Self::Sandbox => "sandbox",
            Self::MissingOutput => "missing-output",
            Self::MissingSourceInput => "missing-source-input",
            Self::CacheVerification => "cache-verification",
            Self::UnsupportedPlatform => "unsupported-platform",
            Self::Builder => "builder",
        }
    }
}

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

pub fn should_write_failure_log(error: &str) -> bool {
    classify_phase(error) == FailurePhase::Build
}

pub fn build_failure_envelopes(result: &PipelineResult, store_dir: &str, logs_dir: &Path) -> Vec<BuildFailureEnvelope> {
    let mut envelopes = result
        .failed
        .iter()
        .map(|failed| {
            let phase = classify_phase(&failed.error);
            let error_class = classify_error_class(&failed.error);
            let saved_log_path = if phase == FailurePhase::Build {
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

fn classify_phase(error: &str) -> FailurePhase {
    let lower = error.to_lowercase();
    if lower.contains("output store directory") && lower.contains("does not exist") {
        return FailurePhase::Preflight;
    }
    if lower.contains("store_dir must") {
        return FailurePhase::Preflight;
    }
    if lower.contains("opening store:") {
        return FailurePhase::Preflight;
    }
    if lower.contains("pathinfo") && lower.contains("fallback") {
        return FailurePhase::Preflight;
    }
    if lower.contains("only supported on linux") {
        return FailurePhase::Preflight;
    }
    FailurePhase::Build
}

fn classify_error_class(error: &str) -> FailureClass {
    let lower = error.to_lowercase();
    let error_class = if classify_phase(error) == FailurePhase::Preflight {
        if lower.contains("only supported on linux") {
            FailureClass::UnsupportedPlatform
        } else {
            FailureClass::Preflight
        }
    } else if lower.contains("fod hash mismatch") {
        FailureClass::FixedOutputHashMismatch
    } else if lower.contains("bwrap") || lower.contains("fusermount") {
        FailureClass::Sandbox
    } else if lower.contains("output not produced by build") {
        FailureClass::MissingOutput
    } else if lower.contains("source input not found in store") {
        FailureClass::MissingSourceInput
    } else if lower.contains("unsigned pathinfo") || lower.contains("trusted signature") {
        FailureClass::CacheVerification
    } else {
        FailureClass::Builder
    };
    debug_assert!(!error_class.as_str().is_empty());
    debug_assert!(!error_class.as_str().contains(' '));
    error_class
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn phase_and_classify_detect_fod_mismatch() {
        let error = "FOD hash mismatch for foo: expected sha256-a, got sha256-b";
        assert_eq!(classify_phase(error), FailurePhase::Build);
        assert_eq!(classify_error_class(error), FailureClass::FixedOutputHashMismatch);
        assert!(should_write_failure_log(error));
    }

    #[test]
    fn preflight_errors_do_not_write_logs() {
        let error = "output store directory /missing does not exist";
        assert_eq!(classify_phase(error), FailurePhase::Preflight);
        assert_eq!(classify_error_class(error), FailureClass::Preflight);
        assert!(!should_write_failure_log(error));
    }
}
