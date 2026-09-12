//! Rendered reports written to standard output.
//!
//! Every report has a pure renderer that returns the exact bytes and a thin
//! writer that emits them, so the format is testable without capturing a
//! process stream.

use mantle_application_contract::OutputFormat;

use crate::RunError;

/// Render one operator JSON payload.
///
/// The label names the payload in diagnostics and must describe real content.
pub(crate) fn render_operator_json<T: serde::Serialize>(value: &T, label: &str) -> Result<String, RunError> {
    assert!(!label.is_empty(), "operator JSON label must not be empty");
    let rendered = serde_json::to_string_pretty(value)
        .map_err(|error| RunError::Internal(format!("rendering {label}: {error}")))?;
    debug_assert!(!rendered.is_empty());
    debug_assert!(rendered.starts_with('{') || rendered.starts_with('['));
    Ok(rendered)
}

/// Write one operator JSON payload to standard output.
pub(crate) fn print_operator_json<T: serde::Serialize>(value: &T, label: &str) -> Result<(), RunError> {
    let rendered = render_operator_json(value, label)?;
    println!("{rendered}");
    Ok(())
}

/// Render one remote-failure replay report.
///
/// JSON output is the compact machine payload; human output is four labelled
/// lines whose missing fields read as `invalid`.
pub(crate) fn render_remote_failure_replay_result(
    result: &serde_json::Value,
    format: OutputFormat,
) -> Result<String, RunError> {
    if format.is_machine_readable() {
        let rendered = serde_json::to_string(result)
            .map_err(|error| RunError::Internal(format!("serializing remote failure replay report: {error}")))?;
        debug_assert!(!rendered.is_empty());
        return Ok(rendered);
    }
    let lines = [
        format!("source-bundle: {}", result["source_bundle_blake3"].as_str().unwrap_or("invalid")),
        format!("replay-attempt: {}", result["replay_attempt_identity"].as_str().unwrap_or("invalid")),
        format!("comparison: {}", result["comparison"]["class"].as_str().unwrap_or("invalid")),
        format!("output-count: {}", result["admitted_output_count"].as_u64().unwrap_or(0)),
    ];
    debug_assert_eq!(lines.len(), 4);
    debug_assert!(lines.iter().all(|line| line.contains(':')));
    Ok(lines.join("\n"))
}

/// Write one remote-failure replay report to standard output.
pub(crate) fn emit_remote_failure_replay_result(
    result: &serde_json::Value,
    format: OutputFormat,
) -> Result<(), RunError> {
    let rendered = render_remote_failure_replay_result(result, format)?;
    println!("{rendered}");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_result() -> serde_json::Value {
        serde_json::json!({
            "source_bundle_blake3": "abc",
            "replay_attempt_identity": "def",
            "comparison": { "class": "reproduced" },
            "admitted_output_count": 3,
        })
    }

    #[test]
    fn the_operator_json_renderer_emits_pretty_json() {
        let rendered = render_operator_json(&serde_json::json!({ "b": 1, "a": 2 }), "sample").expect("renders");
        let parsed: serde_json::Value = serde_json::from_str(&rendered).expect("output is JSON");
        assert_eq!(parsed["a"], 2);
        assert_eq!(parsed["b"], 1);
        assert!(rendered.contains('\n'), "pretty output is multi-line");
    }

    #[test]
    #[should_panic(expected = "operator JSON label must not be empty")]
    fn an_empty_operator_json_label_is_rejected() {
        let _ = render_operator_json(&serde_json::json!({}), "");
    }

    #[test]
    fn the_replay_renderer_emits_one_compact_json_payload() {
        let rendered = render_remote_failure_replay_result(&sample_result(), OutputFormat::Json).expect("json renders");
        assert!(!rendered.contains('\n'));
        let parsed: serde_json::Value = serde_json::from_str(&rendered).expect("output is JSON");
        assert_eq!(parsed["admitted_output_count"], 3);
    }

    #[test]
    fn the_replay_renderer_emits_four_human_lines() {
        let rendered =
            render_remote_failure_replay_result(&sample_result(), OutputFormat::Human).expect("human renders");
        let lines: Vec<&str> = rendered.lines().collect();
        assert_eq!(lines.len(), 4);
        assert_eq!(lines[0], "source-bundle: abc");
        assert_eq!(lines[3], "output-count: 3");
        assert!(serde_json::from_str::<serde_json::Value>(&rendered).is_err());
    }

    #[test]
    fn missing_replay_fields_read_as_invalid() {
        let rendered =
            render_remote_failure_replay_result(&serde_json::json!({}), OutputFormat::Human).expect("human renders");
        assert!(rendered.contains("source-bundle: invalid"));
        assert!(rendered.contains("replay-attempt: invalid"));
        assert!(rendered.contains("comparison: invalid"));
        assert!(rendered.contains("output-count: 0"));
    }
}
