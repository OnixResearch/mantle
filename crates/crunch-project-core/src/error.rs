use alloc::string::String;
use core::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    Manifest(String),
    Lockfile(String),
    Validation(String),
    Upgrade(String),
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
