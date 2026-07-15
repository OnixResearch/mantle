use alloc::collections::BTreeMap;
use alloc::format;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;

use crate::ReleaseEvidenceError;
use crate::manifest::u32_count;
use crate::manifest::validation_error;

pub const RELEASE_SOURCE_ARCHIVE_PROFILE: &str = "mantle-release-source";
pub const RELEASE_SOURCE_ARCHIVE_VERSION: &str = "v1";

const MAX_RELEASE_SOURCE_MEMBERS_COUNT: u32 = 200_000;
const MAX_RELEASE_SOURCE_PATH_BYTES_COUNT: u32 = 4096;
const RELEASE_SOURCE_SKIP_ANYWHERE: &[&str] = &[".agent", ".git", ".jj", ".pi"];
const RELEASE_SOURCE_SKIP_AT_ROOT: &[&str] = &["cairn", "target"];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReleaseSourceEntryKind {
    File,
    Directory,
    Symlink,
    Submodule,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReleaseSourceCandidate {
    pub relative_path: String,
    pub kind: ReleaseSourceEntryKind,
    pub symlink_target: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReleaseSourceMember {
    pub relative_path: String,
    pub kind: ReleaseSourceEntryKind,
    pub symlink_target: Option<String>,
}

#[derive(Debug, Clone, Copy)]
struct RelativePathInput<'a> {
    path: &'a str,
    field_name: &'a str,
}

pub fn plan_release_source_archive_members(
    candidates: Vec<ReleaseSourceCandidate>,
) -> Result<Vec<ReleaseSourceMember>, ReleaseEvidenceError> {
    let candidate_count = u32_count(candidates.len(), "release source candidate count overflowed u32")?;
    if candidate_count > MAX_RELEASE_SOURCE_MEMBERS_COUNT {
        return Err(validation_error(format!(
            "release source candidate count {candidate_count} exceeds {MAX_RELEASE_SOURCE_MEMBERS_COUNT}"
        )));
    }

    let mut members = BTreeMap::new();
    for candidate in candidates {
        let Some(member) = normalize_release_source_candidate(candidate)? else {
            continue;
        };
        if members.insert(member.relative_path.clone(), member).is_some() {
            return Err(validation_error("release source archive contains duplicate member path".to_string()));
        }
    }
    Ok(members.into_values().collect())
}

pub fn release_source_path_is_releasable(path: &str) -> Result<bool, ReleaseEvidenceError> {
    let components = normalized_relative_components(RelativePathInput {
        path,
        field_name: "release source path",
    })?;
    if components.iter().any(|component| RELEASE_SOURCE_SKIP_ANYWHERE.contains(&component.as_str())) {
        return Ok(false);
    }
    let Some(first_component) = components.first() else {
        return Err(validation_error("release source path normalization produced no path components".to_string()));
    };
    if RELEASE_SOURCE_SKIP_AT_ROOT.contains(&first_component.as_str()) {
        return Ok(false);
    }
    Ok(true)
}

pub fn validate_release_source_symlink_target(target: &str) -> Result<(), ReleaseEvidenceError> {
    normalized_relative_components(RelativePathInput {
        path: target,
        field_name: "release source symlink target",
    })?;
    Ok(())
}

fn normalize_release_source_candidate(
    candidate: ReleaseSourceCandidate,
) -> Result<Option<ReleaseSourceMember>, ReleaseEvidenceError> {
    if candidate.kind == ReleaseSourceEntryKind::Submodule {
        return Err(validation_error(format!(
            "release source archive does not support Git submodule entry {}",
            candidate.relative_path
        )));
    }
    if !release_source_path_is_releasable(&candidate.relative_path)? {
        return Ok(None);
    }
    validate_release_source_candidate_symlink(&candidate)?;
    let member = ReleaseSourceMember {
        relative_path: candidate.relative_path,
        kind: candidate.kind,
        symlink_target: candidate.symlink_target,
    };
    debug_assert!(!member.relative_path.is_empty());
    debug_assert_ne!(member.kind, ReleaseSourceEntryKind::Submodule);
    Ok(Some(member))
}

fn validate_release_source_candidate_symlink(candidate: &ReleaseSourceCandidate) -> Result<(), ReleaseEvidenceError> {
    if candidate.kind != ReleaseSourceEntryKind::Symlink {
        return Ok(());
    }
    let Some(target) = candidate.symlink_target.as_deref() else {
        return Err(validation_error(format!(
            "release source symlink {} must record a target",
            candidate.relative_path
        )));
    };
    validate_release_source_symlink_target(target)
}

fn normalized_relative_components(input: RelativePathInput<'_>) -> Result<Vec<String>, ReleaseEvidenceError> {
    let path = input.path;
    let field_name = input.field_name;
    if path.trim().is_empty() {
        return Err(validation_error(format!("{field_name} must not be empty")));
    }
    if path.starts_with('/') {
        return Err(validation_error(format!("{field_name} must be relative: {path}")));
    }
    if path.contains('\\') {
        return Err(validation_error(format!("{field_name} must use '/' separators: {path}")));
    }
    let path_len_bytes = u32_count(path.len(), &format!("{field_name} length overflowed u32"))?;
    if path_len_bytes > MAX_RELEASE_SOURCE_PATH_BYTES_COUNT {
        return Err(validation_error(format!("{field_name} exceeds {MAX_RELEASE_SOURCE_PATH_BYTES_COUNT} bytes")));
    }

    let component_count = path.bytes().filter(|byte| *byte == b'/').count().saturating_add(1);
    let mut components = Vec::with_capacity(component_count);
    for component in path.split('/') {
        if component.is_empty() || component == "." {
            return Err(validation_error(format!("{field_name} contains an empty or current-dir component: {path}")));
        }
        if component == ".." {
            return Err(validation_error(format!("{field_name} must not contain parent-directory components: {path}")));
        }
        components.push(component.to_string());
    }
    if components.is_empty() {
        return Err(validation_error(format!("{field_name} must contain at least one path component")));
    }
    debug_assert_eq!(components.len(), component_count);
    debug_assert_eq!(components.join("/"), path);
    Ok(components)
}

#[cfg(test)]
mod tests {
    use alloc::vec;

    use super::*;

    fn candidate(path: &str, kind: ReleaseSourceEntryKind) -> ReleaseSourceCandidate {
        ReleaseSourceCandidate {
            relative_path: path.to_string(),
            kind,
            symlink_target: None,
        }
    }

    fn symlink(path: &str, target: &str) -> ReleaseSourceCandidate {
        ReleaseSourceCandidate {
            relative_path: path.to_string(),
            kind: ReleaseSourceEntryKind::Symlink,
            symlink_target: Some(target.to_string()),
        }
    }

    #[test]
    fn unordered_entries_produce_stable_member_order() {
        let members = plan_release_source_archive_members(vec![
            candidate("src/lib.rs", ReleaseSourceEntryKind::File),
            candidate("README.md", ReleaseSourceEntryKind::File),
            candidate("builders/default.ncl", ReleaseSourceEntryKind::File),
        ])
        .unwrap();

        let paths = members.into_iter().map(|member| member.relative_path).collect::<Vec<_>>();
        assert_eq!(paths, vec!["README.md", "builders/default.ncl", "src/lib.rs"]);
    }

    #[test]
    fn private_runtime_and_lifecycle_paths_are_excluded() {
        let members = plan_release_source_archive_members(vec![
            candidate("src/main.rs", ReleaseSourceEntryKind::File),
            candidate("target/release/mantle", ReleaseSourceEntryKind::File),
            candidate(".pi/private-note", ReleaseSourceEntryKind::File),
            candidate("vendor/.pi/prompt-history.jsonl", ReleaseSourceEntryKind::File),
            candidate("cairn/specs/demo/spec.md", ReleaseSourceEntryKind::File),
        ])
        .unwrap();

        assert_eq!(members.len(), 1);
        assert_eq!(members[0].relative_path, "src/main.rs");
    }

    #[test]
    fn source_paths_with_target_named_components_are_preserved() {
        let members = plan_release_source_archive_members(vec![
            candidate("vendor-deps/cc/src/target/apple.rs", ReleaseSourceEntryKind::File),
            candidate("vendor/patched/src/target.rs", ReleaseSourceEntryKind::File),
        ])
        .unwrap();

        let paths = members.into_iter().map(|member| member.relative_path).collect::<Vec<_>>();
        assert_eq!(paths, vec!["vendor-deps/cc/src/target/apple.rs", "vendor/patched/src/target.rs"]);
    }

    #[test]
    fn unsafe_paths_are_rejected() {
        let err = plan_release_source_archive_members(vec![candidate("../escape", ReleaseSourceEntryKind::File)])
            .unwrap_err();
        assert!(err.to_string().contains("parent-directory"));

        let err = plan_release_source_archive_members(vec![candidate("/absolute", ReleaseSourceEntryKind::File)])
            .unwrap_err();
        assert!(err.to_string().contains("must be relative"));
    }

    #[test]
    fn symlink_targets_must_stay_relative() {
        let err = plan_release_source_archive_members(vec![symlink("bin/tool", "/etc/passwd")]).unwrap_err();
        assert!(err.to_string().contains("symlink target must be relative"));

        let err = plan_release_source_archive_members(vec![symlink("bin/tool", "../outside")]).unwrap_err();
        assert!(err.to_string().contains("parent-directory"));
    }

    #[test]
    fn submodules_are_rejected() {
        let err =
            plan_release_source_archive_members(vec![candidate("vendor/submodule", ReleaseSourceEntryKind::Submodule)])
                .unwrap_err();

        assert!(err.to_string().contains("does not support Git submodule"));
    }
}
