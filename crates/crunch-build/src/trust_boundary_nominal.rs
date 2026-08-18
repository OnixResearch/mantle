// r[impl build_correctness.nominal_boundaries.identities]
// r[impl build_correctness.nominal_boundaries.units_and_paths]

use std::path::Path;
use std::path::PathBuf;

const MAX_FETCH_URL_BYTES: usize = 4_096;
const MAX_GIT_REVISION_BYTES: usize = 256;
const MAX_BUILD_NAME_BYTES: usize = 256;
const MAX_STORE_PATH_BYTES: usize = 4_096;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrustBoundaryNominalError {
    Empty,
    Oversized,
    ControlCharacter,
    PathContainsNul,
    PathNotAbsolute,
}

impl TrustBoundaryNominalError {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Empty => "empty",
            Self::Oversized => "oversized",
            Self::ControlCharacter => "control-character",
            Self::PathContainsNul => "path-contains-nul",
            Self::PathNotAbsolute => "path-not-absolute",
        }
    }
}

macro_rules! bounded_text {
    ($name:ident, $maximum:expr) => {
        #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $name(String);

        impl $name {
            pub fn new(value: impl Into<String>) -> Result<Self, TrustBoundaryNominalError> {
                let value = value.into();
                if value.is_empty() {
                    return Err(TrustBoundaryNominalError::Empty);
                }
                if value.len() > $maximum {
                    return Err(TrustBoundaryNominalError::Oversized);
                }
                if value.chars().any(char::is_control) {
                    return Err(TrustBoundaryNominalError::ControlCharacter);
                }
                Ok(Self(value))
            }

            pub fn as_str(&self) -> &str {
                &self.0
            }
        }
    };
}

bounded_text!(FetchUrl, MAX_FETCH_URL_BYTES);
bounded_text!(GitRevision, MAX_GIT_REVISION_BYTES);
bounded_text!(DerivationName, MAX_BUILD_NAME_BYTES);
bounded_text!(OutputName, MAX_BUILD_NAME_BYTES);
bounded_text!(DerivationKey, MAX_STORE_PATH_BYTES);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LogicalStorePrefix(String);

impl LogicalStorePrefix {
    pub fn new(value: impl Into<String>) -> Result<Self, TrustBoundaryNominalError> {
        absolute_store_path(value.into()).map(Self)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArchiveInputPath(PathBuf);

impl ArchiveInputPath {
    pub fn new(path: impl Into<PathBuf>) -> Result<Self, TrustBoundaryNominalError> {
        admit_path(path.into()).map(Self)
    }

    pub fn as_path(&self) -> &Path {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FetchOutputDirectory(PathBuf);

impl FetchOutputDirectory {
    pub fn new(path: impl Into<PathBuf>) -> Result<Self, TrustBoundaryNominalError> {
        admit_path(path.into()).map(Self)
    }

    pub fn as_path(&self) -> &Path {
        &self.0
    }
}

/// Provisional and final store paths cannot cross roles implicitly.
///
/// ```compile_fail
/// use crunch_build::{FinalStorePath, ProvisionalStorePath};
/// fn publish(_: FinalStorePath) {}
/// let provisional = ProvisionalStorePath::new("/mantle/store/tmp").unwrap();
/// publish(provisional);
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProvisionalStorePath(String);

impl ProvisionalStorePath {
    pub fn new(value: impl Into<String>) -> Result<Self, TrustBoundaryNominalError> {
        absolute_store_path(value.into()).map(Self)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FinalStorePath(String);

impl FinalStorePath {
    pub fn new(value: impl Into<String>) -> Result<Self, TrustBoundaryNominalError> {
        absolute_store_path(value.into()).map(Self)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GitFetchRequest {
    pub url: FetchUrl,
    pub revision: GitRevision,
    pub output: FetchOutputDirectory,
}

impl GitFetchRequest {
    pub fn new(url: &str, revision: &str, output: &str) -> Result<Self, TrustBoundaryNominalError> {
        Ok(Self {
            url: FetchUrl::new(url)?,
            revision: GitRevision::new(revision)?,
            output: FetchOutputDirectory::new(output)?,
        })
    }
}

fn admit_path(path: PathBuf) -> Result<PathBuf, TrustBoundaryNominalError> {
    if path.as_os_str().is_empty() {
        return Err(TrustBoundaryNominalError::Empty);
    }
    if path.to_string_lossy().contains('\0') {
        return Err(TrustBoundaryNominalError::PathContainsNul);
    }
    Ok(path)
}

fn absolute_store_path(value: String) -> Result<String, TrustBoundaryNominalError> {
    if !value.starts_with('/') {
        return Err(TrustBoundaryNominalError::PathNotAbsolute);
    }
    if value.len() > MAX_STORE_PATH_BYTES {
        return Err(TrustBoundaryNominalError::Oversized);
    }
    if value.contains('\0') {
        return Err(TrustBoundaryNominalError::PathContainsNul);
    }
    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    const VALID_URL: &str = "https://example.invalid/source.tar.gz";
    const VALID_REVISION: &str = "0123456789abcdef0123456789abcdef01234567";
    const VALID_OUTPUT: &str = "/tmp/output";

    #[test]
    fn fetch_and_store_roles_admit_valid_values() {
        let request = GitFetchRequest::new(VALID_URL, VALID_REVISION, VALID_OUTPUT).unwrap();
        let provisional = ProvisionalStorePath::new("/mantle/store/provisional").unwrap();
        let final_path = FinalStorePath::new("/mantle/store/final").unwrap();

        assert_eq!(request.url.as_str(), VALID_URL);
        assert_eq!(request.revision.as_str(), VALID_REVISION);
        assert_eq!(request.output.as_path(), Path::new(VALID_OUTPUT));
        assert_eq!(provisional.as_str(), "/mantle/store/provisional");
        assert_eq!(final_path.as_str(), "/mantle/store/final");
    }

    #[test]
    fn malformed_fetch_and_store_roles_fail_closed() {
        assert_eq!(FetchUrl::new(""), Err(TrustBoundaryNominalError::Empty));
        assert_eq!(GitRevision::new("bad\nrevision"), Err(TrustBoundaryNominalError::ControlCharacter));
        assert_eq!(FinalStorePath::new("relative"), Err(TrustBoundaryNominalError::PathNotAbsolute));
        assert_eq!(ArchiveInputPath::new(PathBuf::new()), Err(TrustBoundaryNominalError::Empty));
    }

    /// Provisional and final store paths cannot cross roles implicitly.
    ///
    /// ```compile_fail
    /// use crunch_build::{FinalStorePath, ProvisionalStorePath};
    /// fn publish(_: FinalStorePath) {}
    /// let provisional = ProvisionalStorePath::new("/mantle/store/tmp").unwrap();
    /// publish(provisional);
    /// ```
    #[allow(dead_code)]
    fn role_separation_compile_fixture() {}
}
