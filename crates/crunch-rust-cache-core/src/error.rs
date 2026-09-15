//! Typed rejection for Rust unit result envelopes and policies.

use std::fmt;

/// One rejected Rust cache input.
///
/// The rejection code is the machine-readable token the callers and receipts
/// already carry, so `Display` writes the code with no label of its own.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RustCacheError {
    code: String,
}

impl RustCacheError {
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

impl fmt::Display for RustCacheError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.code())
    }
}
