use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

use serde::Deserialize;
use serde::Serialize;

pub const MAX_DIAGNOSTICS: u32 = 256;
const DIAGNOSTIC_LIMIT_INDEX: usize = 255;

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

pub(crate) fn error(code: &str, subject: &str, message: &str) -> Diagnostic {
    diagnostic(DiagnosticSeverity::Error, code, subject, message)
}

fn diagnostic(severity: DiagnosticSeverity, code: &str, subject: &str, message: &str) -> Diagnostic {
    debug_assert!(!code.is_empty());
    debug_assert!(!message.is_empty());
    Diagnostic {
        severity,
        code: String::from(code),
        subject: String::from(subject),
        message: String::from(message),
    }
}

pub(crate) fn ordered(mut diagnostics: Vec<Diagnostic>) -> Vec<Diagnostic> {
    diagnostics.sort();
    diagnostics.dedup();
    if diagnostics.len() > DIAGNOSTIC_LIMIT_INDEX {
        diagnostics.truncate(DIAGNOSTIC_LIMIT_INDEX);
        diagnostics.extend(vec![error(
            "diagnostic-limit-exceeded",
            "diagnostics",
            "diagnostic collection exceeded the fixed bound",
        )]);
        diagnostics.sort();
    }
    debug_assert!(diagnostics.windows(crate::ADJACENT_WINDOW_LENGTH).all(|pair| pair[0] <= pair[1]));
    debug_assert!(u32::try_from(diagnostics.len()).unwrap_or(u32::MAX) <= MAX_DIAGNOSTICS);
    diagnostics
}

pub(crate) fn has_errors(diagnostics: &[Diagnostic]) -> bool {
    diagnostics.iter().any(|diagnostic| diagnostic.severity == DiagnosticSeverity::Error)
}
