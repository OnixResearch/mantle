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
pub const MAX_INPUT_BYTES: u64 = 64 * 1024 * 1024;

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

pub fn admit_target(target: Target) -> Result<Target, String> {
    if target.implementation != "cpython" || target.abi != "cp312" || target.platform != "linux_x86_64" {
        return Err("python-target-unsupported: requires CPython cp312 Linux x86_64".into());
    }
    let patch = target.python.strip_prefix("3.12.").ok_or("python-version-unsupported")?;
    if patch.is_empty() || patch.len() > 4 || !patch.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err("python-version-unsupported".into());
    }
    if target.wheel_tags.is_empty()
        || target.wheel_tags.len() > 2
        || target
            .wheel_tags
            .iter()
            .any(|tag| !["py3-none-any", "cp312-cp312-linux_x86_64"].contains(&tag.as_str()))
        || (target.wheel_tags.len() == 2 && target.wheel_tags[0] == target.wheel_tags[1])
    {
        return Err("python-wheel-tags-unsupported".into());
    }
    debug_assert!(target.python.starts_with("3.12."));
    debug_assert!(!patch.is_empty());
    Ok(target)
}

pub fn admit_artifacts(artifacts: Vec<Artifact>) -> Result<Vec<Artifact>, String> {
    if artifacts.is_empty() || artifacts.len() > MAX_PACKAGES {
        return Err("python-artifact-limit".into());
    }
    for (index, artifact) in artifacts.iter().enumerate() {
        if artifact.filename.is_empty()
            || artifact.filename.len() > 240
            || artifact.filename.bytes().any(|byte| !byte.is_ascii_alphanumeric() && !b"._+-".contains(&byte))
        {
            return Err("python-artifact-name".into());
        }
        if artifact.url.len() > 4096 || !artifact.url.starts_with("https://") || artifact.url.contains('\0') {
            return Err("python-artifact-url".into());
        }
        if artifact.sha256.len() != 64
            || !artifact.sha256.bytes().all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
        {
            return Err("python-artifact-hash: expected lowercase SHA-256".into());
        }
        if artifacts[..index].iter().any(|prior| prior.filename == artifact.filename) {
            return Err("python-artifact-duplicate".into());
        }
    }
    debug_assert!(artifacts.len() <= MAX_PACKAGES);
    debug_assert!(!artifacts.is_empty());
    Ok(artifacts)
}

pub fn admit_selections(selections: Vec<String>) -> Result<Vec<String>, String> {
    if selections.len() > MAX_SELECTIONS {
        return Err("python-selection-limit".into());
    }
    for (index, selection) in selections.iter().enumerate() {
        if selection.is_empty()
            || selection.len() > 128
            || !selection
                .bytes()
                .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || b"-_.".contains(&byte))
            || selections[..index].contains(selection)
        {
            return Err("python-selection-invalid".into());
        }
    }
    debug_assert!(selections.len() <= MAX_SELECTIONS);
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
