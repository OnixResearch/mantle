//! Rendering for the runtime fingerprint.
//!
//! The trigger policy and the fingerprint shape live in `operator_diagnostics`;
//! this adapter only assembles the typed request and decides where the rendered
//! text goes.

use mantle_application_contract::OutputFormat;

use crate::RunError;
use crate::operator_diagnostics;

/// Everything the runtime fingerprint renderer needs.
#[derive(Debug, Clone)]
pub(crate) struct RuntimeFingerprintRequest<'a> {
    /// Whether the operator asked for verbose output.
    pub(crate) is_verbose: bool,
    /// Explicit log level, when one was requested.
    pub(crate) log_level: Option<&'a str>,
    /// Requested output format.
    pub(crate) format: OutputFormat,
    /// Command label reported in the fingerprint.
    pub(crate) command_label: &'a str,
    /// Logical store prefix in use.
    pub(crate) logical_store_prefix: &'a str,
    /// Physical store directory in use.
    pub(crate) physical_store_dir: &'a str,
    /// Resolved state directory in use.
    pub(crate) state_dir: &'a str,
    /// Command-specific mode fields.
    pub(crate) modes: operator_diagnostics::RuntimeFingerprintModeFields,
}

/// Render the runtime fingerprint when its trigger fires.
///
/// A quiet invocation returns `None`, so the caller emits nothing at all.
pub(crate) fn render_runtime_fingerprint(request: &RuntimeFingerprintRequest<'_>) -> Result<Option<String>, RunError> {
    let Some(verbosity_source) = operator_diagnostics::runtime_fingerprint_verbosity_source(
        operator_diagnostics::RuntimeFingerprintTriggerInput {
            verbose: request.is_verbose,
            log_level: request.log_level,
            diagnostic_mode: false,
        },
    ) else {
        debug_assert!(!request.is_verbose);
        return Ok(None);
    };
    let fingerprint = operator_diagnostics::build_runtime_fingerprint(operator_diagnostics::RuntimeFingerprintInput {
        mantle_version: env!("CARGO_PKG_VERSION").to_string(),
        command: request.command_label.to_string(),
        logical_store_prefix: request.logical_store_prefix.to_string(),
        physical_store_dir: request.physical_store_dir.to_string(),
        state_dir: request.state_dir.to_string(),
        json_mode: request.format.is_machine_readable(),
        verbosity_source,
        modes: request.modes.clone(),
    });
    let rendered = operator_diagnostics::render_runtime_fingerprint(&fingerprint)
        .map_err(|err| RunError::Internal(format!("rendering runtime fingerprint: {err}")))?;
    debug_assert!(!rendered.is_empty());
    debug_assert!(rendered.contains(request.command_label));
    Ok(Some(rendered))
}

/// Write the runtime fingerprint to standard error when its trigger fires.
///
/// The fingerprint is diagnostic, so it never writes to standard output.
pub(crate) fn emit_runtime_fingerprint(request: &RuntimeFingerprintRequest<'_>) -> Result<(), RunError> {
    if let Some(rendered) = render_runtime_fingerprint(request)? {
        eprintln!("{rendered}");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request(is_verbose: bool, log_level: Option<&str>) -> RuntimeFingerprintRequest<'_> {
        RuntimeFingerprintRequest {
            is_verbose,
            log_level,
            format: OutputFormat::Human,
            command_label: "build",
            logical_store_prefix: "/mantle/store",
            physical_store_dir: "/var/lib/mantle/store",
            state_dir: "/var/lib/mantle/state",
            modes: operator_diagnostics::RuntimeFingerprintModeFields::default(),
        }
    }

    #[test]
    fn a_quiet_invocation_renders_nothing() {
        assert!(render_runtime_fingerprint(&request(false, None)).expect("renders").is_none());
        assert!(render_runtime_fingerprint(&request(false, Some("warn"))).expect("renders").is_none());
    }

    #[test]
    fn the_verbose_flag_renders_the_command_and_version() {
        let rendered = render_runtime_fingerprint(&request(true, None)).expect("renders").expect("trigger fires");
        assert!(rendered.contains("build"));
        assert!(rendered.contains(env!("CARGO_PKG_VERSION")));
        assert!(rendered.contains("/mantle/store"));
        assert!(rendered.contains("/var/lib/mantle/state"));
    }

    #[test]
    fn a_talkative_log_level_renders_and_json_mode_follows_the_format() {
        let rendered =
            render_runtime_fingerprint(&request(false, Some("debug"))).expect("renders").expect("trigger fires");
        assert!(rendered.contains("debug") || rendered.contains("build"));
        let mut json_request = request(true, None);
        json_request.format = OutputFormat::Json;
        let json_rendered = render_runtime_fingerprint(&json_request).expect("renders").expect("trigger fires");
        let payload = json_rendered
            .split_once(' ')
            .map(|(_, payload)| payload)
            .expect("fingerprint renders a prefix and a payload");
        let parsed: serde_json::Value = serde_json::from_str(payload).expect("payload is JSON");
        assert_eq!(parsed["json_mode"], true);
        assert_eq!(parsed["command"], "build");
    }
}
