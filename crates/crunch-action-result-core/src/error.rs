//! Typed rejection for action-result records, indexes, and policies.

use std::fmt;

/// One rejected action-result input.
///
/// The rejection code is the machine-readable token the callers and receipts
/// already carry, so `Display` writes the code with no label of its own.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionResultError {
    code: String,
}

impl ActionResultError {
    /// Build one rejection from its code.
    pub fn new(code: impl Into<String>) -> Self {
        let error = Self { code: code.into() };
        debug_assert!(!error.code.is_empty());
        error
    }

    /// The machine-readable rejection code.
    pub fn code(&self) -> &str {
        debug_assert!(!self.code.is_empty());
        &self.code
    }
}

impl fmt::Display for ActionResultError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.code())
    }
}
