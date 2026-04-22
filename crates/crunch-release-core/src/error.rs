use alloc::string::String;
use core::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReleaseEvidenceError {
    Parse(String),
    Validation(String),
}

impl ReleaseEvidenceError {
    pub fn message(&self) -> &str {
        match self {
            Self::Parse(message) | Self::Validation(message) => message,
        }
    }
}

impl fmt::Display for ReleaseEvidenceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.message())
    }
}
