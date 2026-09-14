use alloc::string::String;
use core::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    Manifest(String),
    Lockfile(String),
    Validation(String),
    Upgrade(String),
}

impl Error {
    /// The message without the variant label.
    ///
    /// Diagnostic text that a caller renders into an operator-visible blocker
    /// uses this, so the label the `Display` implementation adds does not leak
    /// into a message the operator already reads elsewhere.
    pub(crate) fn message(&self) -> &str {
        match self {
            Self::Manifest(message) | Self::Lockfile(message) | Self::Validation(message) | Self::Upgrade(message) => {
                message
            }
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Manifest(message) => write!(f, "manifest: {message}"),
            Error::Lockfile(message) => write!(f, "lockfile: {message}"),
            Error::Validation(message) => write!(f, "validation: {message}"),
            Error::Upgrade(message) => write!(f, "upgrade: {message}"),
        }
    }
}
