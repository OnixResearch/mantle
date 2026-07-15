//! Project-aware build resolution.
//!
//! Discovers `mantle-project.ncl`/`crunch.ncl`, parses `.#name` selectors,
//! and extracts derivations from the Project output schema.

use std::ffi::OsString;
use std::path::Path;
use std::path::PathBuf;

use crate::errors::RunError;

/// The canonical project root file name.
const CANONICAL_PROJECT_ROOT_FILE: &str = "mantle-project.ncl";
/// The legacy compatibility project root file name.
const LEGACY_PROJECT_ROOT_FILE: &str = "crunch.ncl";

/// Maximum ancestor directories to search for a project root.
const MAX_SEARCH_DEPTH: u32 = 64;

/// A parsed build target: either a file path or a project selector.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BuildTarget {
    /// Explicit file path: `crunch build foo.ncl`
    File(PathBuf),
    /// Project selector: `crunch build .#hello`
    Selector(Selector),
    /// Bare `crunch build` — use project defaults
    ProjectDefault,
}

/// A parsed `.#attr` selector.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Selector {
    /// Attribute path segments (e.g. `.#packages.hello` → `["packages", "hello"]`)
    pub segments: Vec<String>,
}

/// Resolved project with the root file and any narrowing.
pub struct ResolvedProject {
    /// Path to the project root Nickel file.
    pub root_file: PathBuf,
    /// Which attribute(s) to extract from the evaluated output.
    pub target: ProjectTarget,
    /// Import paths needed (project dir + any user-specified).
    pub import_paths: Vec<OsString>,
}

/// What to extract from the project output.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProjectTarget {
    /// Build the default package(s).
    Default,
    /// Build a specific attribute from the project.
    Attribute(Vec<String>),
    /// Build all packages.
    AllPackages,
    /// Build all checks.
    AllChecks,
    /// Enter a dev shell.
    DefaultShell,
    /// Enter a named shell.
    NamedShell(String),
}

/// Parse a CLI argument into a build target.
pub fn parse_build_target(arg: Option<&Path>) -> BuildTarget {
    match arg {
        None => BuildTarget::ProjectDefault,
        Some(path) => {
            let s = path.to_string_lossy();
            if let Some(selector) = parse_selector(&s) {
                BuildTarget::Selector(selector)
            } else {
                BuildTarget::File(path.to_path_buf())
            }
        }
    }
}

/// Parse a `.#name` or `.#category.name` selector string.
fn parse_selector(s: &str) -> Option<Selector> {
    let rest = s.strip_prefix(".#")?;
    if rest.is_empty() {
        return None;
    }
    let segments: Vec<String> = rest.split('.').map(String::from).collect();
    // Reject segments with empty parts (e.g. `.#foo..bar`)
    if segments.iter().any(String::is_empty) {
        return None;
    }
    Some(Selector { segments })
}

/// Find the nearest project root by walking up from `start_dir`.
#[allow(dead_code)]
pub fn find_project_root(start_dir: &Path) -> Option<PathBuf> {
    find_project_root_checked(start_dir).ok().flatten()
}

fn find_project_root_checked(start_dir: &Path) -> Result<Option<PathBuf>, RunError> {
    debug_assert!(MAX_SEARCH_DEPTH > 0);
    debug_assert_ne!(CANONICAL_PROJECT_ROOT_FILE, LEGACY_PROJECT_ROOT_FILE);
    let mut dir = start_dir.to_path_buf();
    for _ in 0..MAX_SEARCH_DEPTH {
        let canonical = dir.join(CANONICAL_PROJECT_ROOT_FILE);
        let legacy = dir.join(LEGACY_PROJECT_ROOT_FILE);
        let has_canonical = canonical.is_file();
        let has_legacy = legacy.is_file();
        if has_canonical && has_legacy {
            return Err(RunError::Internal(format!(
                "both {CANONICAL_PROJECT_ROOT_FILE} and {LEGACY_PROJECT_ROOT_FILE} exist in {}; keep one project surface",
                dir.display()
            )));
        }
        if has_canonical {
            return Ok(Some(canonical));
        }
        if has_legacy {
            return Ok(Some(legacy));
        }
        if !dir.pop() {
            break;
        }
    }
    Ok(None)
}

/// Resolve a build target into a project file + extraction plan.
pub fn resolve_project_target(
    target: &BuildTarget,
    cwd: &Path,
    user_import_paths: &[PathBuf],
) -> Result<ResolvedProject, RunError> {
    debug_assert!(MAX_SEARCH_DEPTH > 0);
    debug_assert_ne!(CANONICAL_PROJECT_ROOT_FILE, LEGACY_PROJECT_ROOT_FILE);
    let root_file = find_project_root_checked(cwd)?.ok_or_else(|| {
        RunError::Internal(format!(
            "no {CANONICAL_PROJECT_ROOT_FILE} or {LEGACY_PROJECT_ROOT_FILE} found in {} or any parent directory.\n\
             Either specify a .ncl file or run `mantle init` to create a project.",
            cwd.display()
        ))
    })?;

    let project_dir = root_file
        .parent()
        .ok_or_else(|| RunError::Internal(format!("project root file has no parent: {}", root_file.display())))?;
    let search_path_count = user_import_paths
        .len()
        .checked_add(1)
        .ok_or_else(|| RunError::Internal("project import path count overflowed usize".to_string()))?;
    let mut evaluation_search_paths: Vec<OsString> = Vec::with_capacity(search_path_count);
    evaluation_search_paths.push(project_dir.as_os_str().to_owned());
    for path in user_import_paths {
        evaluation_search_paths.push(path.as_os_str().to_owned());
    }

    let project_target = match target {
        BuildTarget::ProjectDefault => ProjectTarget::Default,
        BuildTarget::Selector(sel) => resolve_selector_to_target(sel),
        BuildTarget::File(_) => {
            // Caller should not route File targets through project resolution.
            return Err(RunError::Internal("file targets do not use project resolution".into()));
        }
    };

    Ok(ResolvedProject {
        root_file,
        target: project_target,
        import_paths: evaluation_search_paths,
    })
}

/// Map a selector's segments to a project target.
fn resolve_selector_to_target(sel: &Selector) -> ProjectTarget {
    ProjectTarget::Attribute(sel.segments.clone())
}

/// Generate a Nickel expression that evaluates `crunch.ncl` and extracts
/// the requested derivation(s).
///
/// Returns a Nickel snippet that, when evaluated, produces either a single
/// derivation or a record of derivations suitable for the build pipeline.
pub fn generate_extraction_expr(root_file: &Path, target: &ProjectTarget) -> String {
    debug_assert!(MAX_SEARCH_DEPTH > 0);
    debug_assert_ne!(CANONICAL_PROJECT_ROOT_FILE, LEGACY_PROJECT_ROOT_FILE);
    let root_path = root_file.to_string_lossy();
    match target {
        ProjectTarget::Default => render_default_target(&root_path),
        ProjectTarget::Attribute(segments) => render_attribute_target(&root_path, segments),
        ProjectTarget::AllPackages => format!(r#"(import "{root_path}").packages"#),
        ProjectTarget::AllChecks => format!(r#"(import "{root_path}").checks"#),
        ProjectTarget::DefaultShell => render_default_shell_target(&root_path),
        ProjectTarget::NamedShell(name) => format!(
            r#"let _proj = import "{root_path}" in
if std.record.has_field "shells" _proj && std.record.has_field "{name}" _proj.shells then
  let _profile = _proj.shells."{name}" in
  if std.record.has_field "derivation" _profile then _profile.derivation else _profile
else
  _proj.devShells."{name}""#
        ),
    }
}

fn render_default_target(root_path: &str) -> String {
    format!(
        r#"let _proj = import "{root_path}" in
let _pkg_name = if std.record.has_field "default" _proj
  then (if std.record.has_field "package" _proj.default then _proj.default.package else null)
  else null
in
if _pkg_name != null then
  _proj.packages."%{{_pkg_name}}"
else
  _proj.packages"#
    )
}

fn render_attribute_target(root_path: &str, segments: &[String]) -> String {
    assert!(!segments.is_empty(), "selector must have at least one segment");
    debug_assert!(MAX_SEARCH_DEPTH > 0);
    if segments.len() == 1 {
        let name = &segments[0];
        return format!(
            r#"let _proj = import "{root_path}" in
if std.record.has_field "{name}" _proj.packages then
  _proj.packages."{name}"
else if std.record.has_field "{name}" _proj.checks then
  _proj.checks."{name}"
else
  _proj."{name}""#
        );
    }
    let path = segments.iter().map(|segment| format!("\"{segment}\"")).collect::<Vec<_>>().join(".");
    format!(
        r#"let _proj = import "{root_path}" in
_proj.{path}"#
    )
}

fn render_default_shell_target(root_path: &str) -> String {
    format!(
        r#"let _proj = import "{root_path}" in
let _has_profiles = std.record.has_field "shells" _proj in
let _explicit_shell_name = if std.record.has_field "default" _proj
  then (if std.record.has_field "shell" _proj.default then _proj.default.shell else null)
  else null in
let _profile_name = if _explicit_shell_name != null then
  _explicit_shell_name
else if _has_profiles && std.record.has_field "dev" _proj.shells then
  "dev"
else if _has_profiles && std.record.has_field "default" _proj.shells then
  "default"
else
  null in
let _profile_value = if _profile_name != null && _has_profiles && std.record.has_field _profile_name _proj.shells then
  _proj.shells."%{{_profile_name}}"
else
  null
in
if _profile_value != null then
  if std.record.has_field "derivation" _profile_value then _profile_value.derivation else _profile_value
else
  let _shell_name = _explicit_shell_name in
  if _shell_name != null then
    _proj.devShells."%{{_shell_name}}"
  else
    let _shells = std.record.to_array _proj.devShells in
    if std.array.length _shells == 1 then
      (std.array.first _shells).value
    else if std.array.length _shells == 0 then
      std.fail_with "no shells or devShells defined in mantle project"
    else
      std.fail_with "multiple shell profiles defined; set default.shell or use `mantle shell .#name`""#
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_selector_basic() {
        let sel = parse_selector(".#hello").unwrap();
        assert_eq!(sel.segments, vec!["hello"]);
    }

    #[test]
    fn parse_selector_nested() {
        let sel = parse_selector(".#packages.hello").unwrap();
        assert_eq!(sel.segments, vec!["packages", "hello"]);
    }

    #[test]
    fn parse_selector_rejects_empty() {
        assert!(parse_selector(".#").is_none());
    }

    #[test]
    fn parse_selector_rejects_double_dot() {
        assert!(parse_selector(".#foo..bar").is_none());
    }

    #[test]
    fn parse_selector_not_a_selector() {
        assert!(parse_selector("hello.ncl").is_none());
        assert!(parse_selector("./hello.ncl").is_none());
    }

    #[test]
    fn parse_build_target_none_is_default() {
        assert_eq!(parse_build_target(None), BuildTarget::ProjectDefault);
    }

    #[test]
    fn parse_build_target_selector() {
        let t = parse_build_target(Some(Path::new(".#hello")));
        assert!(matches!(t, BuildTarget::Selector(_)));
    }

    #[test]
    fn parse_build_target_file() {
        let t = parse_build_target(Some(Path::new("hello.ncl")));
        assert!(matches!(t, BuildTarget::File(_)));
    }

    #[test]
    fn generate_extraction_default() {
        let expr = generate_extraction_expr(Path::new("crunch.ncl"), &ProjectTarget::Default);
        assert!(expr.contains("_proj.packages"));
        assert!(expr.contains("default"));
    }

    #[test]
    fn generate_extraction_attribute_single() {
        let expr = generate_extraction_expr(Path::new("crunch.ncl"), &ProjectTarget::Attribute(vec!["hello".into()]));
        assert!(expr.contains("packages"));
        assert!(expr.contains("checks"));
        assert!(expr.contains("hello"));
    }

    #[test]
    fn generate_extraction_attribute_nested() {
        let expr = generate_extraction_expr(
            Path::new("crunch.ncl"),
            &ProjectTarget::Attribute(vec!["packages".into(), "hello".into()]),
        );
        assert!(expr.contains(r#"_proj."packages"."hello""#));
    }

    #[test]
    fn generate_extraction_all_checks() {
        let expr = generate_extraction_expr(Path::new("crunch.ncl"), &ProjectTarget::AllChecks);
        assert!(expr.contains(".checks"));
    }

    #[test]
    fn generate_default_shell_prefers_named_profiles_before_legacy_dev_shells() {
        let expr = generate_extraction_expr(Path::new("mantle-project.ncl"), &ProjectTarget::DefaultShell);

        assert!(expr.contains(r#"std.record.has_field "shells" _proj"#));
        assert!(expr.contains(r#"std.record.has_field "dev" _proj.shells"#));
        assert!(expr.contains("_profile_value.derivation"));
        assert!(expr.contains("_proj.devShells"));
    }

    #[test]
    fn generate_named_shell_uses_profile_derivation_fallback() {
        let expr =
            generate_extraction_expr(Path::new("mantle-project.ncl"), &ProjectTarget::NamedShell("build".into()));

        assert!(expr.contains(r#"std.record.has_field "shells" _proj"#));
        assert!(expr.contains(r#"std.record.has_field "build" _proj.shells"#));
        assert!(expr.contains("_profile.derivation"));
        assert!(expr.contains(r#"_proj.devShells."build""#));
    }

    #[test]
    fn package_default_extraction_does_not_depend_on_shell_profiles() {
        let expr = generate_extraction_expr(Path::new("mantle-project.ncl"), &ProjectTarget::Default);

        assert!(!expr.contains("devShells"));
        assert!(!expr.contains("shells"));
        assert!(expr.contains("_proj.packages"));
    }

    #[test]
    fn find_project_root_returns_none_for_empty() {
        let tmp = tempfile::tempdir().unwrap();
        assert!(find_project_root(tmp.path()).is_none());
    }

    #[test]
    fn find_project_root_finds_canonical_in_current_dir() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(tmp.path().join("mantle-project.ncl"), "{}").unwrap();
        let found = find_project_root(tmp.path()).unwrap();
        assert_eq!(found, tmp.path().join("mantle-project.ncl"));
    }

    #[test]
    fn find_project_root_still_finds_legacy_in_current_dir() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(tmp.path().join("crunch.ncl"), "{}").unwrap();
        let found = find_project_root(tmp.path()).unwrap();
        assert_eq!(found, tmp.path().join("crunch.ncl"));
    }

    #[test]
    fn find_project_root_walks_up() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(tmp.path().join("crunch.ncl"), "{}").unwrap();
        let child = tmp.path().join("sub").join("deep");
        std::fs::create_dir_all(&child).unwrap();
        let found = find_project_root(&child).unwrap();
        assert_eq!(found, tmp.path().join("crunch.ncl"));
    }

    #[test]
    fn find_project_root_rejects_mixed_canonical_and_legacy_surfaces() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(tmp.path().join("mantle-project.ncl"), "{}").unwrap();
        std::fs::write(tmp.path().join("crunch.ncl"), "{}").unwrap();

        let err = find_project_root_checked(tmp.path()).unwrap_err();
        assert!(err.to_string().contains("both mantle-project.ncl and crunch.ncl"));
    }
}
