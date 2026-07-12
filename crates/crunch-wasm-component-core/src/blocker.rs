use alloc::string::String;

use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ComponentBlocker {
    pub code: String,
    pub subject: String,
    pub message: String,
}

// The three strings map directly to the named DTO fields below; keeping this
// constructor tiny avoids duplicating allocation and invariant checks.
#[allow(tigerstyle::ambiguous_params)]
pub(crate) fn blocker(code: &str, subject: &str, message: &str) -> ComponentBlocker {
    debug_assert!(!code.is_empty());
    debug_assert!(!message.is_empty());
    ComponentBlocker {
        code: String::from(code),
        subject: String::from(subject),
        message: String::from(message),
    }
}
