//! Rendering for the remote-client build report.

use std::path::Path;

use crate::BuildOutputMode;
use crate::RunError;
use crate::remote_build;

/// Render one remote-client build report for the selected output mode.
///
/// JSON renders the full report document; human mode lists the logical paths of
/// imported outputs; the evaluation-stream mode is rejected because a remote
/// dispatch cannot stream an evaluation.
pub(crate) fn render_remote_client_build_report(
    report: &remote_build::RemoteClientBuildReport,
    file: &Path,
    output_dir: &Path,
    state_dir: &Path,
    output_mode: BuildOutputMode,
) -> Result<String, RunError> {
    match output_mode {
        BuildOutputMode::Json => {
            let json_document = crate::remote_client_build_json_report(report, file, output_dir, state_dir)?;
            let rendered = serde_json::to_string_pretty(&json_document)
                .map_err(|err| RunError::Internal(format!("serializing remote build report: {err}")))?;
            debug_assert!(rendered.starts_with('{'));
            debug_assert!(!rendered.is_empty());
            Ok(rendered)
        }
        BuildOutputMode::Human => {
            let mut lines: Vec<String> = Vec::new();
            for build in &report.imported {
                for output in &build.imported.outputs {
                    lines.push(output.logical_path.clone());
                }
            }
            debug_assert!(lines.iter().all(|line| !line.is_empty()) || lines.is_empty());
            Ok(lines.join("\n"))
        }
        BuildOutputMode::EvaluationStream => {
            Err(RunError::Internal("--evaluation-stream cannot be used with remote build dispatch".to_string()))
        }
    }
}

/// Write one remote-client build report to standard output.
pub(crate) fn print_remote_client_build_report(
    report: &remote_build::RemoteClientBuildReport,
    file: &Path,
    output_dir: &Path,
    state_dir: &Path,
    output_mode: BuildOutputMode,
) -> Result<(), RunError> {
    let rendered = render_remote_client_build_report(report, file, output_dir, state_dir, output_mode)?;
    if !rendered.is_empty() {
        println!("{rendered}");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn empty_report() -> remote_build::RemoteClientBuildReport {
        remote_build::RemoteClientBuildReport {
            schema: String::from("mantle-remote-client-build-report-v1"),
            builder: String::from("builder-a"),
            store_prefix: String::from("/mantle/store"),
            priority_decisions: Vec::new(),
            imported: Vec::new(),
        }
    }

    #[test]
    fn a_report_without_imports_renders_nothing_in_human_mode() {
        let rendered = render_remote_client_build_report(
            &empty_report(),
            Path::new("root.ncl"),
            Path::new("/out"),
            Path::new("/state"),
            BuildOutputMode::Human,
        )
        .expect("human renders");
        assert!(rendered.is_empty());
    }

    #[test]
    fn the_json_mode_renders_the_report_document() {
        let rendered = render_remote_client_build_report(
            &empty_report(),
            Path::new("root.ncl"),
            Path::new("/out"),
            Path::new("/state"),
            BuildOutputMode::Json,
        )
        .expect("json renders");
        let parsed: serde_json::Value = serde_json::from_str(&rendered).expect("output is JSON");
        assert_eq!(parsed["schema"], crate::BUILD_JSON_REPORT_SCHEMA);
        assert_eq!(parsed["store_dir"], "/mantle/store");
        assert_eq!(parsed["outcomes"].as_array().map(Vec::len), Some(0));
        assert_eq!(parsed["counts"]["succeeded_total"], 0);
    }

    #[test]
    fn the_evaluation_stream_mode_is_rejected() {
        let error = render_remote_client_build_report(
            &empty_report(),
            Path::new("root.ncl"),
            Path::new("/out"),
            Path::new("/state"),
            BuildOutputMode::EvaluationStream,
        )
        .expect_err("evaluation stream is rejected");
        assert!(format!("{error}").contains("--evaluation-stream cannot be used with remote build dispatch"));
    }
}
