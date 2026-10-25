#![no_std]
#![feature(register_tool)]
#![register_tool(tigerstyle)]

//! Bounded admission for the Python/uv adapter. No resolver, I/O, or process effects.
extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;

use serde::Deserialize;
use serde::Serialize;

pub const MAX_PACKAGES: usize = 256;
pub const MAX_SELECTIONS: usize = 64;
pub const MAX_CONFIG_BYTES: u64 = 1024 * 1024;
pub const MAX_INPUT_BYTES: u64 = (64u64 * 1024).saturating_mul(1024);

/// Rejections at the bounded Python package admission boundary.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AdmissionError {
    UnsupportedTarget,
    UnsupportedVersion,
    UnsupportedWheelTags,
    ArtifactLimit,
    ArtifactName,
    ArtifactUrl,
    ArtifactHash,
    DuplicateArtifact,
    SelectionLimit,
    InvalidSelection,
}

impl core::fmt::Display for AdmissionError {
    fn fmt(&self, output: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        output.write_str(match self {
            Self::UnsupportedTarget => "python-target-unsupported: requires CPython cp312 Linux x86_64",
            Self::UnsupportedVersion => "python-version-unsupported",
            Self::UnsupportedWheelTags => "python-wheel-tags-unsupported",
            Self::ArtifactLimit => "python-artifact-limit",
            Self::ArtifactName => "python-artifact-name",
            Self::ArtifactUrl => "python-artifact-url",
            Self::ArtifactHash => "python-artifact-hash: expected lowercase SHA-256",
            Self::DuplicateArtifact => "python-artifact-duplicate",
            Self::SelectionLimit => "python-selection-limit",
            Self::InvalidSelection => "python-selection-invalid",
        })
    }
}

impl core::error::Error for AdmissionError {}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Artifact {
    pub filename: String,
    pub url: String,
    pub path: String,
    /// SHA-256 is the Python package-index interoperability boundary.
    pub sha256: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Target {
    pub python: String,
    pub implementation: String,
    pub abi: String,
    pub platform: String,
    pub wheel_tags: Vec<String>,
}

pub fn admit_target(target: Target) -> Result<Target, AdmissionError> {
    if target.implementation != "cpython" || target.abi != "cp312" || target.platform != "linux_x86_64" {
        return Err(AdmissionError::UnsupportedTarget);
    }
    let patch = target.python.strip_prefix("3.12.").ok_or(AdmissionError::UnsupportedVersion)?;
    if patch.is_empty() || patch.len() > 4 || !patch.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(AdmissionError::UnsupportedVersion);
    }
    let is_tag_count_valid = !target.wheel_tags.is_empty() && target.wheel_tags.len() <= 2;
    let has_unknown_tag = target
        .wheel_tags
        .iter()
        .any(|tag| !["py3-none-any", "cp312-cp312-linux_x86_64"].contains(&tag.as_str()));
    let has_duplicate_tags = target.wheel_tags.len() == 2 && target.wheel_tags[0] == target.wheel_tags[1];
    if !is_tag_count_valid || has_unknown_tag || has_duplicate_tags {
        return Err(AdmissionError::UnsupportedWheelTags);
    }
    debug_assert!(target.python.starts_with("3.12."));
    debug_assert!(!patch.is_empty());
    Ok(target)
}

pub fn admit_artifacts(artifacts: Vec<Artifact>) -> Result<Vec<Artifact>, AdmissionError> {
    if artifacts.is_empty() || artifacts.len() > MAX_PACKAGES {
        return Err(AdmissionError::ArtifactLimit);
    }
    for (index, artifact) in artifacts.iter().enumerate() {
        if artifact.filename.is_empty()
            || artifact.filename.len() > 240
            || artifact.filename.bytes().any(|byte| !byte.is_ascii_alphanumeric() && !b"._+-".contains(&byte))
        {
            return Err(AdmissionError::ArtifactName);
        }
        if artifact.url.len() > 4096 || !artifact.url.starts_with("https://") || artifact.url.contains('\0') {
            return Err(AdmissionError::ArtifactUrl);
        }
        if artifact.sha256.len() != 64
            || !artifact.sha256.bytes().all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
        {
            return Err(AdmissionError::ArtifactHash);
        }
        if artifacts[..index].iter().any(|prior| prior.filename == artifact.filename) {
            return Err(AdmissionError::DuplicateArtifact);
        }
    }
    debug_assert!(artifacts.len() <= MAX_PACKAGES);
    debug_assert!(!artifacts.is_empty());
    Ok(artifacts)
}

pub fn admit_selections(selections: Vec<String>) -> Result<Vec<String>, AdmissionError> {
    if selections.len() > MAX_SELECTIONS {
        return Err(AdmissionError::SelectionLimit);
    }
    for (index, selection) in selections.iter().enumerate() {
        let is_empty = selection.is_empty();
        let is_too_long = selection.len() > 128;
        let has_invalid_char = !selection
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || b"-_.".contains(&byte));
        let is_duplicate = selections[..index].contains(selection);
        let is_invalid = is_empty || is_too_long || has_invalid_char || is_duplicate;
        if is_invalid {
            return Err(AdmissionError::InvalidSelection);
        }
    }
    debug_assert!(selections.len() <= MAX_SELECTIONS);
    debug_assert!(selections.iter().all(|selection| !selection.is_empty()));
    Ok(selections)
}

#[cfg(test)]
mod tests {
    use alloc::string::ToString;

    use super::*;

    #[test]
    fn target_and_artifact_identity_fail_closed() {
        let target = Target {
            python: "3.12.12".into(),
            implementation: "cpython".into(),
            abi: "cp312".into(),
            platform: "linux_x86_64".into(),
            wheel_tags: alloc::vec!["py3-none-any".into()],
        };
        let mut target = admit_target(target).unwrap();
        target.abi = "cp311".into();
        assert!(admit_target(target).is_err());
        let artifact = Artifact {
            filename: "demo-1-py3-none-any.whl".into(),
            url: "https://fixture.invalid/demo.whl".into(),
            path: "/declared/demo.whl".into(),
            sha256: "0".repeat(64),
        };
        let mut artifacts = admit_artifacts(alloc::vec![artifact]).unwrap();
        artifacts[0].filename = "../demo.whl".into();
        assert!(admit_artifacts(artifacts).is_err());
        assert!(admit_selections(alloc::vec!["gpu".to_string(), "gpu".to_string()]).is_err());
    }
}
