use std::fmt;
use std::path::Path;

#[derive(Debug)]
pub enum Error {
    Invalid(String),
    Io(String),
    Tool(String),
    Blocked(String),
}

impl Error {
    pub(crate) fn io(action: &str, path: &Path, error: impl fmt::Display) -> Self {
        Self::Io(format!("{action} {}: {error}", path.display()))
    }
}

impl fmt::Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Invalid(message) => write!(formatter, "invalid component pipeline input: {message}"),
            Self::Io(message) => write!(formatter, "component pipeline I/O failed: {message}"),
            Self::Tool(message) => write!(formatter, "component pipeline tool failed: {message}"),
            Self::Blocked(message) => write!(formatter, "component pipeline blocked: {message}"),
        }
    }
}

impl std::error::Error for Error {}
