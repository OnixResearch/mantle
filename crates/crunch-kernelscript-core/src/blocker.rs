use alloc::string::String;

use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExperimentBlocker {
    pub code: String,
    pub subject: String,
    pub message: String,
}

#[allow(tigerstyle::ambiguous_params)]
pub(crate) fn blocker(code: &str, subject: &str, message: &str) -> ExperimentBlocker {
    debug_assert!(!code.is_empty());
    debug_assert!(!message.is_empty());
    ExperimentBlocker {
        code: String::from(code),
        subject: String::from(subject),
        message: String::from(message),
    }
}
