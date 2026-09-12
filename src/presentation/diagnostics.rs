//! Rendering for the doctor preflight report.

use mantle_application_contract::OutputFormat;

use crate::RunError;
use crate::operator_diagnostics;

/// One rendered doctor report together with the stream it belongs on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RenderedDoctorReport {
    /// Exact report text.
    pub(crate) text: String,
    /// Whether the text belongs on standard error.
    pub(crate) goes_to_stderr: bool,
}

/// Render one doctor report.
///
/// A failing human report belongs on standard error so scripts can separate it
/// from data output; JSON always goes to standard output.
pub(crate) fn render_doctor_report(
    report: &operator_diagnostics::PreflightReport,
    format: OutputFormat,
) -> Result<RenderedDoctorReport, RunError> {
    if format.is_machine_readable() {
        let text = report
            .render_json()
            .map_err(|error| RunError::Internal(format!("serializing doctor report: {error}")))?;
        let rendered = RenderedDoctorReport {
            text,
            goes_to_stderr: false,
        };
        debug_assert!(!rendered.text.is_empty());
        debug_assert!(!rendered.goes_to_stderr);
        return Ok(rendered);
    }
    let text = report.render_human();
    let rendered = RenderedDoctorReport {
        text,
        goes_to_stderr: !report.ok,
    };
    debug_assert!(!rendered.text.is_empty());
    debug_assert_eq!(rendered.goes_to_stderr, !report.ok);
    Ok(rendered)
}

/// Write one doctor report to its stream.
pub(crate) fn emit_doctor_report(
    report: &operator_diagnostics::PreflightReport,
    format: OutputFormat,
) -> Result<(), RunError> {
    let rendered = render_doctor_report(report, format)?;
    if rendered.goes_to_stderr {
        eprintln!("{}", rendered.text);
    } else {
        println!("{}", rendered.text);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn report(is_ok: bool) -> operator_diagnostics::PreflightReport {
        operator_diagnostics::PreflightReport {
            schema: "crunch-doctor-report-v1",
            command: "doctor",
            profile: operator_diagnostics::DoctorProfile::Build,
            ok: is_ok,
            checks: vec![operator_diagnostics::PreflightCheck {
                id: "store-exists",
                status: if is_ok {
                    operator_diagnostics::PreflightStatus::Ok
                } else {
                    operator_diagnostics::PreflightStatus::Failed
                },
                summary: String::from("store directory is present"),
                detail: None,
            }],
        }
    }

    #[test]
    fn a_json_report_goes_to_standard_output() {
        let rendered = render_doctor_report(&report(true), OutputFormat::Json).expect("json renders");
        assert!(!rendered.goes_to_stderr);
        let parsed: serde_json::Value = serde_json::from_str(&rendered.text).expect("report is JSON");
        assert_eq!(parsed["ok"], true);
    }

    #[test]
    fn a_failing_human_report_goes_to_standard_error() {
        let failing = render_doctor_report(&report(false), OutputFormat::Human).expect("human renders");
        assert!(failing.goes_to_stderr);
        assert!(failing.text.contains("doctor profile: build"));
        let passing = render_doctor_report(&report(true), OutputFormat::Human).expect("human renders");
        assert!(!passing.goes_to_stderr);
        assert!(!passing.text.is_empty());
    }
}
