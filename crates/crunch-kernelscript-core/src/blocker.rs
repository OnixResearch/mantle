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

pub(crate) struct BlockerSubject<'a>(&'a str);

impl<'a> From<&'a str> for BlockerSubject<'a> {
    fn from(subject: &'a str) -> Self {
        Self(subject)
    }
}

impl<'a> From<&'a String> for BlockerSubject<'a> {
    fn from(subject: &'a String) -> Self {
        Self(subject.as_str())
    }
}

pub(crate) fn blocker<'a>(code: &str, subject: impl Into<BlockerSubject<'a>>, message: &str) -> ExperimentBlocker {
    debug_assert!(!code.is_empty());
    debug_assert!(!message.is_empty());
    ExperimentBlocker {
        code: String::from(code),
        subject: String::from(subject.into().0),
        message: String::from(message),
    }
}
