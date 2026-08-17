// r[impl build_correctness.nominal_boundaries.admission]
// r[impl build_correctness.nominal_boundaries.identities]
// r[impl build_correctness.nominal_boundaries.units_and_paths]

use alloc::string::String;

const MAX_NOMINAL_IDENTIFIER_BYTES: usize = 4_096;
const MAX_ABSOLUTE_EXECUTABLE_PATH_BYTES: usize = 4_096;
const STAGEX_TIMEOUT_MILLIS_HARD_MAX: u64 = 604_800_000;
const STAGEX_OUTPUT_BYTES_HARD_MAX: u64 = 1_099_511_627_776;
pub const STAGEX_PARALLEL_JOB_COUNT_HARD_MAX: u32 = 64;
pub const STAGEX_STAGE_COUNT_HARD_MAX: u32 = 512;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NominalValueError {
    Empty,
    Oversized,
    ControlCharacter,
    UnsafeAbsolutePath,
    Zero,
    AboveHardLimit,
}

impl core::fmt::Display for NominalValueError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Empty => formatter.write_str("value is empty"),
            Self::Oversized => formatter.write_str("value exceeds its byte limit"),
            Self::ControlCharacter => formatter.write_str("value contains a control character"),
            Self::UnsafeAbsolutePath => formatter.write_str("path is not an admitted absolute executable path"),
            Self::Zero => formatter.write_str("bounded quantity is zero"),
            Self::AboveHardLimit => formatter.write_str("bounded quantity exceeds its hard limit"),
        }
    }
}

fn admit_identifier(value: String) -> Result<String, NominalValueError> {
    if value.trim().is_empty() {
        return Err(NominalValueError::Empty);
    }
    if value.len() > MAX_NOMINAL_IDENTIFIER_BYTES {
        return Err(NominalValueError::Oversized);
    }
    if value.chars().any(char::is_control) {
        return Err(NominalValueError::ControlCharacter);
    }
    Ok(value)
}

macro_rules! nominal_identifier {
    ($name:ident) => {
        #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $name(String);

        impl $name {
            pub fn new(value: impl Into<String>) -> Result<Self, NominalValueError> {
                admit_identifier(value.into()).map(Self)
            }

            pub fn as_str(&self) -> &str {
                &self.0
            }
        }
    };
}

nominal_identifier!(StagexStageId);
nominal_identifier!(StagexArtifactId);
nominal_identifier!(StagexExecutableAuthorizationId);
nominal_identifier!(StagexExecutableRole);
nominal_identifier!(StagexEnvironmentAssumptionId);
nominal_identifier!(LineageSourceArtifactId);
nominal_identifier!(LineageGeneratedArtifactId);
nominal_identifier!(LineageTransitionToolId);
nominal_identifier!(LineagePatchId);
nominal_identifier!(LineageEnvironmentAssumptionId);

/// Path and unit roles cannot be exchanged.
///
/// ```compile_fail
/// use crunch_bootstrap_core::{StagexAbsoluteExecutablePath, StagexTimeoutMillis};
/// fn requires_path(_: StagexAbsoluteExecutablePath) {}
/// let timeout = StagexTimeoutMillis::new(1).unwrap();
/// requires_path(timeout);
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StagexAbsoluteExecutablePath(String);

impl StagexAbsoluteExecutablePath {
    pub fn new(value: impl Into<String>) -> Result<Self, NominalValueError> {
        let value = value.into();
        if value.len() > MAX_ABSOLUTE_EXECUTABLE_PATH_BYTES {
            return Err(NominalValueError::Oversized);
        }
        let has_control = value.chars().any(char::is_control);
        let has_parent = value.split('/').any(|component| component == "..");
        if !value.starts_with('/') || value == "/" || has_parent || has_control {
            return Err(NominalValueError::UnsafeAbsolutePath);
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

macro_rules! bounded_u64 {
    ($name:ident, $maximum:expr) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $name(u64);

        impl $name {
            pub fn new(value: u64) -> Result<Self, NominalValueError> {
                if value == 0 {
                    return Err(NominalValueError::Zero);
                }
                if value > $maximum {
                    return Err(NominalValueError::AboveHardLimit);
                }
                Ok(Self(value))
            }

            pub const fn get(self) -> u64 {
                self.0
            }
        }
    };
}

bounded_u64!(StagexTimeoutMillis, STAGEX_TIMEOUT_MILLIS_HARD_MAX);
bounded_u64!(StagexOutputByteLimit, STAGEX_OUTPUT_BYTES_HARD_MAX);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct StagexParallelJobLimit(u32);

impl StagexParallelJobLimit {
    pub fn new(value: u32) -> Result<Self, NominalValueError> {
        bounded_u32(value, STAGEX_PARALLEL_JOB_COUNT_HARD_MAX).map(Self)
    }

    pub const fn get(self) -> u32 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct StagexStageCountLimit(u32);

impl StagexStageCountLimit {
    pub fn new(value: u32) -> Result<Self, NominalValueError> {
        bounded_u32(value, STAGEX_STAGE_COUNT_HARD_MAX).map(Self)
    }

    pub const fn get(self) -> u32 {
        self.0
    }
}

fn bounded_u32(value: u32, maximum: u32) -> Result<u32, NominalValueError> {
    if value == 0 {
        return Err(NominalValueError::Zero);
    }
    if value > maximum {
        return Err(NominalValueError::AboveHardLimit);
    }
    Ok(value)
}

#[cfg(test)]
mod tests {
    use alloc::string::ToString;

    use super::*;

    const VALID_STAGE_ID: &str = "stage:hex0";
    const VALID_EXECUTABLE_PATH: &str = "/stage/bin/hex0";
    const VALID_TIMEOUT_MILLIS: u64 = 30_000;
    const VALID_OUTPUT_BYTES: u64 = 1_048_576;
    const VALID_PARALLEL_JOBS: u32 = 4;
    const VALID_STAGE_COUNT: u32 = 8;

    #[test]
    fn role_specific_identifiers_and_limits_admit_valid_values() {
        let stage = StagexStageId::new(VALID_STAGE_ID).unwrap();
        let artifact = StagexArtifactId::new(VALID_STAGE_ID).unwrap();
        let path = StagexAbsoluteExecutablePath::new(VALID_EXECUTABLE_PATH).unwrap();

        assert_eq!(stage.as_str(), VALID_STAGE_ID);
        assert_eq!(artifact.as_str(), VALID_STAGE_ID);
        assert_eq!(path.as_str(), VALID_EXECUTABLE_PATH);
        assert_eq!(StagexTimeoutMillis::new(VALID_TIMEOUT_MILLIS).unwrap().get(), VALID_TIMEOUT_MILLIS);
        assert_eq!(StagexOutputByteLimit::new(VALID_OUTPUT_BYTES).unwrap().get(), VALID_OUTPUT_BYTES);
        assert_eq!(StagexParallelJobLimit::new(VALID_PARALLEL_JOBS).unwrap().get(), VALID_PARALLEL_JOBS);
        assert_eq!(StagexStageCountLimit::new(VALID_STAGE_COUNT).unwrap().get(), VALID_STAGE_COUNT);
    }

    #[test]
    fn empty_oversized_control_path_and_limit_values_fail_closed() {
        let oversized = "a".repeat(MAX_NOMINAL_IDENTIFIER_BYTES.saturating_add(1));

        assert_eq!(StagexStageId::new(""), Err(NominalValueError::Empty));
        assert_eq!(StagexArtifactId::new(oversized), Err(NominalValueError::Oversized));
        assert_eq!(StagexExecutableAuthorizationId::new("bad\nid"), Err(NominalValueError::ControlCharacter));
        assert_eq!(StagexAbsoluteExecutablePath::new("../bin/tool"), Err(NominalValueError::UnsafeAbsolutePath));
        assert_eq!(StagexAbsoluteExecutablePath::new("/bin/../tool"), Err(NominalValueError::UnsafeAbsolutePath));
        assert_eq!(StagexTimeoutMillis::new(0), Err(NominalValueError::Zero));
        assert_eq!(
            StagexParallelJobLimit::new(STAGEX_PARALLEL_JOB_COUNT_HARD_MAX.saturating_add(1)),
            Err(NominalValueError::AboveHardLimit)
        );
        assert_eq!(
            StagexStageCountLimit::new(STAGEX_STAGE_COUNT_HARD_MAX.saturating_add(1)),
            Err(NominalValueError::AboveHardLimit)
        );
    }

    /// Stage and artifact identifiers are intentionally distinct roles.
    ///
    /// ```compile_fail
    /// use crunch_bootstrap_core::{StagexArtifactId, StagexStageId};
    /// fn requires_stage(_: StagexStageId) {}
    /// let artifact = StagexArtifactId::new("same-text").unwrap();
    /// requires_stage(artifact);
    /// ```
    #[allow(dead_code)]
    fn role_separation_compile_fixture() {
        let _ = VALID_STAGE_ID.to_string();
    }
}
