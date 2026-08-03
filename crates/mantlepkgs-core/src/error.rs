use alloc::string::String;

use serde::Deserialize;
use serde::Serialize;

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Diagnostic {
    pub code: String,
    pub path: String,
    pub message: String,
}

impl Diagnostic {
    pub fn new(code: &str, path: &str, message: &str) -> Self {
        assert!(!code.is_empty(), "diagnostic code must not be empty");
        assert!(!path.is_empty(), "diagnostic path must not be empty");
        Self {
            code: code.into(),
            path: path.into(),
            message: message.into(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CoreFailure {
    pub diagnostics: alloc::vec::Vec<Diagnostic>,
}

impl CoreFailure {
    pub fn from_diagnostic(diagnostic: Diagnostic) -> Self {
        Self {
            diagnostics: alloc::vec![diagnostic],
        }
    }

    pub fn from_diagnostics(mut diagnostics: alloc::vec::Vec<Diagnostic>) -> Self {
        diagnostics.sort();
        diagnostics.dedup();
        assert!(!diagnostics.is_empty(), "core failure must contain a diagnostic");
        Self { diagnostics }
    }
}
