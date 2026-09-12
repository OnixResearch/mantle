//! Output-format selection and its logging consequence.
//!
//! The root maps `--json` into one typed format, and this policy states what
//! that format means: JSON output is machine-readable, and in that mode
//! default logging stays quiet unless the operator asked for logs.

/// Requested output format.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum OutputFormat {
    /// Machine-readable JSON on standard output.
    Json,
    /// Human-readable text on standard output.
    Human,
}

impl OutputFormat {
    /// Whether output is meant for a machine reader.
    pub fn is_machine_readable(self) -> bool {
        let is_machine_readable = matches!(self, Self::Json);
        debug_assert!(is_machine_readable || matches!(self, Self::Human));
        debug_assert_eq!(is_machine_readable, self == Self::Json);
        is_machine_readable
    }
}

/// The logging decision that follows from an output format.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LoggingDecision {
    /// Whether tracing starts.
    pub is_initialized: bool,
    /// Whether default logging was suppressed to protect machine-readable stderr.
    pub is_suppressed_for_json: bool,
}

/// Select the output format from the parsed flag.
pub fn output_format(is_json: bool) -> OutputFormat {
    let format = if is_json {
        OutputFormat::Json
    } else {
        OutputFormat::Human
    };
    debug_assert_eq!(format.is_machine_readable(), is_json);
    debug_assert!(matches!(format, OutputFormat::Json | OutputFormat::Human));
    format
}

/// Decide whether tracing starts for one output format.
///
/// JSON mode keeps stderr machine-readable, so tracing starts only when the
/// operator explicitly asked for logs; the human format always starts tracing.
pub fn logging_decision(format: OutputFormat, is_logs_requested: bool) -> LoggingDecision {
    let is_suppressed_for_json = format.is_machine_readable() && !is_logs_requested;
    let decision = LoggingDecision {
        is_initialized: !is_suppressed_for_json,
        is_suppressed_for_json,
    };
    debug_assert_eq!(decision.is_initialized, !decision.is_suppressed_for_json);
    debug_assert!(decision.is_initialized || format.is_machine_readable());
    decision
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_flag_selects_the_matching_format() {
        assert_eq!(output_format(true), OutputFormat::Json);
        assert_eq!(output_format(false), OutputFormat::Human);
        assert!(OutputFormat::Json.is_machine_readable());
        assert!(!OutputFormat::Human.is_machine_readable());
    }

    #[test]
    fn json_without_requested_logs_suppresses_default_logging() {
        let decision = logging_decision(OutputFormat::Json, false);
        assert!(!decision.is_initialized);
        assert!(decision.is_suppressed_for_json);
    }

    #[test]
    fn requested_logs_start_tracing_in_either_format() {
        let requested_json = logging_decision(OutputFormat::Json, true);
        assert!(requested_json.is_initialized);
        assert!(!requested_json.is_suppressed_for_json);
        let human = logging_decision(OutputFormat::Human, false);
        assert!(human.is_initialized);
        assert!(!human.is_suppressed_for_json);
    }
}
