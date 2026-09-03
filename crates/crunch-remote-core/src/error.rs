use alloc::string::String;
use core::fmt;

/// Deterministic rejection from the remote functional core.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RemoteCoreError {
    /// An admitted value violated a bounded core contract.
    Invalid { code: String },
    /// An observation did not belong to the pending effect.
    WrongEffectIdentity,
    /// An observation kind did not match the pending effect.
    WrongObservationKind,
    /// An observation carried a stale or foreign fence.
    StaleFence,
    /// An observation carried a stale or foreign attempt identity.
    StaleAttempt,
    /// A transition was requested after a terminal outcome.
    TerminalSession,
    /// The bounded application transition count was exhausted.
    TransitionLimitExceeded,
    /// Deterministic identity serialization failed.
    IdentitySerialization { reason: String },
}

impl RemoteCoreError {
    #[must_use]
    pub fn invalid(code: impl Into<String>) -> Self {
        Self::Invalid { code: code.into() }
    }

    #[must_use]
    pub fn code(&self) -> &str {
        match self {
            Self::Invalid { code } => code,
            Self::WrongEffectIdentity => "remote-effect-identity-mismatch",
            Self::WrongObservationKind => "remote-observation-kind-mismatch",
            Self::StaleFence => "stale-fence",
            Self::StaleAttempt => "stale-attempt",
            Self::TerminalSession => "remote-session-terminal",
            Self::TransitionLimitExceeded => "remote-transition-limit-exceeded",
            Self::IdentitySerialization { .. } => "remote-identity-serialization-failed",
        }
    }
}

impl core::error::Error for RemoteCoreError {}

impl fmt::Display for RemoteCoreError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::IdentitySerialization { reason } => {
                write!(formatter, "{}: {reason}", self.code())
            }
            _ => formatter.write_str(self.code()),
        }
    }
}

/// Compatibility error for realization-key derivation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RealizationKeyError {
    EmptyField { field: &'static str },
    NotSortedUnique { field: &'static str },
    Serialize { source: String },
}

impl core::error::Error for RealizationKeyError {}

impl fmt::Display for RealizationKeyError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyField { field } => write!(formatter, "realization key field {field} must not be empty"),
            Self::NotSortedUnique { field } => {
                write!(formatter, "realization key collection {field} must be sorted and unique")
            }
            Self::Serialize { source } => write!(formatter, "serializing realization key request: {source}"),
        }
    }
}
