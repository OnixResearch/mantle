use alloc::string::String;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RustPlanCoreError {
    InvalidFacts {
        code: &'static str,
        subject: String,
    },
    LimitExceeded {
        code: &'static str,
        observed: u32,
        maximum: u32,
    },
    MissingReference {
        code: &'static str,
        subject: String,
    },
    AmbiguousReference {
        code: &'static str,
        subject: String,
    },
    ObservationMismatch {
        code: &'static str,
        subject: String,
    },
    Serialization,
}

impl RustPlanCoreError {
    #[must_use]
    pub fn invalid(code: &'static str, subject: impl Into<String>) -> Self {
        let subject = subject.into();
        debug_assert!(!code.is_empty());
        debug_assert!(!subject.is_empty());
        Self::InvalidFacts { code, subject }
    }

    #[must_use]
    pub fn missing(code: &'static str, subject: impl Into<String>) -> Self {
        let subject = subject.into();
        debug_assert!(!code.is_empty());
        debug_assert!(!subject.is_empty());
        Self::MissingReference { code, subject }
    }

    #[must_use]
    pub fn mismatch(code: &'static str, subject: impl Into<String>) -> Self {
        let subject = subject.into();
        debug_assert!(!code.is_empty());
        debug_assert!(!subject.is_empty());
        Self::ObservationMismatch { code, subject }
    }
}

impl core::fmt::Display for RustPlanCoreError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::InvalidFacts { code, subject } => write!(formatter, "invalid Rust planning facts: {code}: {subject}"),
            Self::LimitExceeded {
                code,
                observed,
                maximum,
            } => write!(formatter, "Rust planning limit exceeded: {code}: {observed}>{maximum}"),
            Self::MissingReference { code, subject } => {
                write!(formatter, "missing Rust planning reference: {code}: {subject}")
            }
            Self::AmbiguousReference { code, subject } => {
                write!(formatter, "ambiguous Rust planning reference: {code}: {subject}")
            }
            Self::ObservationMismatch { code, subject } => {
                write!(formatter, "Rust planning observation mismatch: {code}: {subject}")
            }
            Self::Serialization => formatter.write_str("Rust planning serialization failed"),
        }
    }
}
