// r[impl build_correctness.nominal_boundaries.admission]
// r[impl build_correctness.nominal_boundaries.digests]

use alloc::string::String;

const BLAKE3_HEX_LENGTH: usize = 64;
const MAX_REPOSITORY_ID_BYTES: usize = 256;
const MAX_REQUIREMENT_ID_BYTES: usize = 512;
const MAX_RELATIVE_PATH_BYTES: usize = 4_096;
const MAX_RELEASE_ID_BYTES: usize = 512;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContentBoundNominalError {
    RepositoryIdInvalid,
    RequirementIdInvalid,
    ReleaseIdInvalid,
    SpecificationPathInvalid,
    EvidencePathInvalid,
    Blake3Invalid,
}

impl ContentBoundNominalError {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::RepositoryIdInvalid => "repository-id-invalid",
            Self::RequirementIdInvalid => "requirement-id-invalid",
            Self::ReleaseIdInvalid => "release-id-invalid",
            Self::SpecificationPathInvalid => "specification-path-invalid",
            Self::EvidencePathInvalid => "evidence-path-invalid",
            Self::Blake3Invalid => "blake3-invalid",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ContentBoundRepositoryId(String);

impl ContentBoundRepositoryId {
    pub fn new(value: impl Into<String>) -> Result<Self, ContentBoundNominalError> {
        let value = value.into();
        let components = value.split('/');
        let is_valid = !value.is_empty()
            && value.len() <= MAX_REPOSITORY_ID_BYTES
            && value.trim() == value
            && value.bytes().all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-' | b'/'))
            && !value.starts_with('/')
            && components
                .into_iter()
                .all(|component| !component.is_empty() && component != "." && component != "..");
        if !is_valid {
            return Err(ContentBoundNominalError::RepositoryIdInvalid);
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ContentBoundRequirementId(String);

impl ContentBoundRequirementId {
    pub fn new(value: impl Into<String>) -> Result<Self, ContentBoundNominalError> {
        let value = value.into();
        let is_valid = !value.is_empty()
            && value.len() <= MAX_REQUIREMENT_ID_BYTES
            && value.contains('.')
            && value
                .bytes()
                .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'.' | b'_' | b'-'));
        if !is_valid {
            return Err(ContentBoundNominalError::RequirementIdInvalid);
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ContentBoundReleaseId(String);

impl ContentBoundReleaseId {
    pub fn new(value: impl Into<String>) -> Result<Self, ContentBoundNominalError> {
        let value = value.into();
        let is_valid = !value.is_empty()
            && value.len() <= MAX_RELEASE_ID_BYTES
            && value.trim() == value
            && !value.chars().any(char::is_control);
        if !is_valid {
            return Err(ContentBoundNominalError::ReleaseIdInvalid);
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ContentBoundSpecificationPath(String);

impl ContentBoundSpecificationPath {
    pub fn new(value: impl Into<String>) -> Result<Self, ContentBoundNominalError> {
        let value = value.into();
        let is_valid = safe_relative_path(&value)
            && value.len() <= MAX_RELATIVE_PATH_BYTES
            && value.starts_with("cairn/specs/")
            && value.ends_with("/spec.md");
        if !is_valid {
            return Err(ContentBoundNominalError::SpecificationPathInvalid);
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ContentBoundEvidencePath(String);

impl ContentBoundEvidencePath {
    pub fn new_repository_path(value: impl Into<String>) -> Result<Self, ContentBoundNominalError> {
        let value = value.into();
        if value.len() > MAX_RELATIVE_PATH_BYTES || !safe_relative_path(&value) {
            return Err(ContentBoundNominalError::EvidencePathInvalid);
        }
        Ok(Self(value))
    }

    pub fn new_bundle_path(value: impl Into<String>) -> Result<Self, ContentBoundNominalError> {
        let path = Self::new_repository_path(value)?;
        if !path.0.starts_with("requirement-evidence/") {
            return Err(ContentBoundNominalError::EvidencePathInvalid);
        }
        Ok(path)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CheckedBlake3Hex(String);

impl CheckedBlake3Hex {
    pub fn new(value: impl Into<String>) -> Result<Self, ContentBoundNominalError> {
        let value = value.into();
        if value.len() != BLAKE3_HEX_LENGTH
            || !value.bytes().all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'))
        {
            return Err(ContentBoundNominalError::Blake3Invalid);
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Requirement and content-evidence digest aggregates remain distinct.
///
/// ```compile_fail
/// use crunch_release_core::{CheckedContentBoundDigest, CheckedRequirementDigest};
/// fn requires_requirement(_: CheckedRequirementDigest) {}
/// fn reject_content(value: CheckedContentBoundDigest) { requires_requirement(value); }
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckedRequirementDigest {
    pub domain: crate::RequirementDigestDomainV1,
    pub blake3: CheckedBlake3Hex,
}

impl CheckedRequirementDigest {
    pub fn new(
        domain: crate::RequirementDigestDomainV1,
        value: impl Into<String>,
    ) -> Result<Self, ContentBoundNominalError> {
        Ok(Self {
            domain,
            blake3: CheckedBlake3Hex::new(value)?,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckedContentBoundDigest {
    pub domain: crate::ContentBoundDigestDomainV1,
    pub blake3: CheckedBlake3Hex,
}

impl CheckedContentBoundDigest {
    pub fn new(
        domain: crate::ContentBoundDigestDomainV1,
        value: impl Into<String>,
    ) -> Result<Self, ContentBoundNominalError> {
        Ok(Self {
            domain,
            blake3: CheckedBlake3Hex::new(value)?,
        })
    }
}

fn safe_relative_path(path: &str) -> bool {
    if path.is_empty() || path.starts_with('/') || path.contains('\\') {
        return false;
    }
    path.split('/').all(|component| !component.is_empty() && component != "." && component != "..")
}

#[cfg(test)]
mod tests {
    use super::*;

    const VALID_BLAKE3: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

    #[test]
    fn nominal_content_bound_values_admit_valid_roles() {
        assert_eq!(ContentBoundRepositoryId::new("onix/mantle").unwrap().as_str(), "onix/mantle");
        assert_eq!(ContentBoundRequirementId::new("mantle.release.bound").unwrap().as_str(), "mantle.release.bound");
        assert_eq!(
            ContentBoundSpecificationPath::new("cairn/specs/release/spec.md").unwrap().as_str(),
            "cairn/specs/release/spec.md"
        );
        assert_eq!(
            ContentBoundEvidencePath::new_bundle_path("requirement-evidence/test.json").unwrap().as_str(),
            "requirement-evidence/test.json"
        );
        assert_eq!(CheckedBlake3Hex::new(VALID_BLAKE3).unwrap().as_str(), VALID_BLAKE3);
    }

    #[test]
    fn malformed_roles_fail_closed() {
        assert_eq!(ContentBoundRepositoryId::new("../mantle"), Err(ContentBoundNominalError::RepositoryIdInvalid));
        assert_eq!(ContentBoundRequirementId::new("UPPER"), Err(ContentBoundNominalError::RequirementIdInvalid));
        assert_eq!(
            ContentBoundSpecificationPath::new("../spec.md"),
            Err(ContentBoundNominalError::SpecificationPathInvalid)
        );
        assert_eq!(
            ContentBoundEvidencePath::new_bundle_path("other/file"),
            Err(ContentBoundNominalError::EvidencePathInvalid)
        );
        assert_eq!(CheckedBlake3Hex::new("abcd"), Err(ContentBoundNominalError::Blake3Invalid));
    }
}
