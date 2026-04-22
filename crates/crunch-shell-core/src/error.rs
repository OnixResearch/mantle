use alloc::string::String;
use core::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ShellError {
    UnsupportedSidecarVersion { version: u32 },
    SidecarParse(String),
    EmptyPath,
    TooManyEnvVars { count: u32, limit: u32 },
    TooManyPathEntries { count: u32, limit: u32 },
    NonUtf8Path { kind: String, value: String },
}

impl fmt::Display for ShellError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedSidecarVersion { version } => {
                write!(formatter, "unsupported sidecar version {version} (expected 1)")
            }
            Self::SidecarParse(message) => write!(formatter, "failed to parse sidecar JSON: {message}"),
            Self::EmptyPath => formatter.write_str("PATH is empty after composition"),
            Self::TooManyEnvVars { count, limit } => {
                write!(formatter, "too many env vars in sidecar ({count} > {limit})")
            }
            Self::TooManyPathEntries { count, limit } => {
                write!(formatter, "too many PATH entries after composition ({count} > {limit})")
            }
            Self::NonUtf8Path { kind, value } => write!(formatter, "non-UTF-8 {kind}: {value}"),
        }
    }
}
