//! Resolve relocated Git dependencies from captured package identities, not layout.

use std::path::Path;

use super::DependencySourceResolutionError;
use super::NativeGitSourcePlanningSummary;
use super::RegistryVersionMatchInputs;
use super::dependency_package_name;
use super::dependency_path;
use super::dependency_source_error;
use super::registry_version_req_matches;

pub(super) struct GitPathDependencyInputs<'a> {
    pub(super) parent_package_id: Option<&'a str>,
    pub(super) name: &'a str,
    pub(super) value: &'a toml::Value,
    pub(super) sources: &'a NativeGitSourcePlanningSummary,
}

pub(super) fn captured_git_path_manifest(
    inputs: GitPathDependencyInputs<'_>,
) -> Result<Option<&str>, DependencySourceResolutionError> {
    let Some(parent_id) = inputs.parent_package_id.filter(|id| id.starts_with("git+")) else {
        return Ok(None);
    };
    if !inputs.sources.ready {
        return Err(dependency_source_error("native-git-source-planning-blocked", "Git source facts are not ready"));
    }
    let parent = inputs.sources.sources.iter().find(|source| source.package_id == parent_id).ok_or_else(|| {
        dependency_source_error("missing-captured-git-parent-source", "Git parent has no captured source fact")
    })?;
    let path = dependency_path(inputs.value).expect("caller selects a path dependency");
    if Path::new(path).is_absolute() {
        return Err(dependency_source_error(
            "unsupported-git-path-dependency",
            "Git path dependencies must be relative",
        ));
    }
    let requirement = version_requirement(inputs.value)?;
    let name = dependency_package_name(inputs.name, inputs.value);
    let mut candidates = inputs
        .sources
        .sources
        .iter()
        .filter(|source| source.source == parent.source)
        .filter(|source| source.name == name)
        .filter(|source| {
            requirement.is_none_or(|requirement| {
                registry_version_req_matches(RegistryVersionMatchInputs {
                    requirement,
                    source_version: &source.version,
                })
            })
        });
    let source = candidates.next().ok_or_else(|| {
        dependency_source_error(
            "missing-captured-git-path-dependency",
            format!("Git path dependency `{}` has no matching captured package in its parent source", inputs.name),
        )
    })?;
    if candidates.next().is_some() {
        return Err(dependency_source_error(
            "ambiguous-captured-git-path-dependency",
            format!("Git path dependency `{}` matches multiple packages in its parent source", inputs.name,),
        ));
    }
    assert_eq!(source.source, parent.source);
    assert_eq!(source.name, name);
    Ok(Some(&source.manifest_path))
}

fn version_requirement(value: &toml::Value) -> Result<Option<&str>, DependencySourceResolutionError> {
    let Some(version) = value.as_table().and_then(|table| table.get("version")) else {
        return Ok(None);
    };
    match version.as_str() {
        Some(version) if !version.trim().is_empty() => Ok(Some(version)),
        _ => Err(dependency_source_error(
            "invalid-git-path-dependency-version",
            "Git dependency version must be a nonempty string",
        )),
    }
}
