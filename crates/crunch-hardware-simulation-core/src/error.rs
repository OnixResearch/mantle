//! Typed rejection for digest and reference inputs.

use alloc::string::String;
use core::fmt;

/// One rejected digest, reference, or path input.
///
/// The rejection code is the machine-readable token the callers and receipts
/// already carry, so the type preserves it exactly: `Display` writes the code
/// with no label of its own.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DigestError {
    code: String,
}

impl DigestError {
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

impl fmt::Display for DigestError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.code())
    }
}
