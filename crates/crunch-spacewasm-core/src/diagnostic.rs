use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

use serde::Deserialize;
use serde::Serialize;

pub(crate) const MAX_DIAGNOSTIC_COUNT: usize = 256;
const DIAGNOSTIC_LIMIT_INDEX: usize = MAX_DIAGNOSTIC_COUNT.saturating_sub(1);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum DiagnosticSeverity {
    Info,
    Warning,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Diagnostic {
    pub severity: DiagnosticSeverity,
    pub code: String,
    pub subject: String,
    pub message: String,
}

pub(crate) struct ErrorDiagnostic<'a> {
    pub code: &'a str,
    pub subject: &'a str,
    pub message: &'a str,
}

pub(crate) fn error(input: ErrorDiagnostic<'_>) -> Diagnostic {
    debug_assert!(!input.code.is_empty());
    debug_assert!(!input.message.is_empty());
    Diagnostic {
        severity: DiagnosticSeverity::Error,
        code: String::from(input.code),
        subject: String::from(input.subject),
        message: String::from(input.message),
    }
}

pub(crate) fn ordered(mut diagnostics: Vec<Diagnostic>) -> Vec<Diagnostic> {
    diagnostics.sort();
    diagnostics.dedup();
    if diagnostics.len() > DIAGNOSTIC_LIMIT_INDEX {
        diagnostics.truncate(DIAGNOSTIC_LIMIT_INDEX);
        diagnostics.extend(vec![error(ErrorDiagnostic {
            code: "diagnostic-limit-exceeded",
            subject: "diagnostics",
            message: "diagnostic collection exceeded the fixed bound",
        })]);
        diagnostics.sort();
    }
    debug_assert!(diagnostics.windows(crate::ADJACENT_WINDOW_LENGTH).all(|pair| pair[0] <= pair[1]));
    debug_assert!(diagnostics.len() <= MAX_DIAGNOSTIC_COUNT);
    diagnostics
}

pub(crate) fn has_errors(diagnostics: &[Diagnostic]) -> bool {
    diagnostics.iter().any(|diagnostic| diagnostic.severity == DiagnosticSeverity::Error)
}
