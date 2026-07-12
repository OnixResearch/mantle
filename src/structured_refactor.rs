// machine-artifact-public: structured-refactor.command-reports
use std::fmt;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

use serde::Serialize;

pub const REFACTOR_SESSION_SCHEMA: &str = "mantle-structured-refactor-session-v1";
pub const CRUNCH_TO_MANTLE_SESSION_ID: &str = "crunch-to-mantle-project-identity";

#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
pub struct RefactorSession {
    pub schema: &'static str,
    pub id: &'static str,
    pub canonical_name: &'static str,
    pub legacy_aliases: &'static [&'static str],
    pub canonical_cli: &'static [&'static str],
    pub legacy_cli: &'static [&'static str],
    pub canonical_files: &'static [&'static str],
    pub legacy_files: &'static [&'static str],
    pub canonical_store_prefix: &'static str,
    pub legacy_store_prefixes: &'static [&'static str],
    pub compatibility_policy: &'static str,
    pub validation_checks: &'static [&'static str],
    pub risk_summary: &'static str,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
pub struct RefactorOperation {
    pub kind: &'static str,
    pub from: String,
    pub to: String,
    pub apply_supported: bool,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
pub struct RefactorDiagnostic {
    pub code: &'static str,
    pub message: String,
    pub remediation: String,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
pub struct RefactorPlan {
    pub schema: &'static str,
    pub session_id: &'static str,
    pub root: String,
    pub dry_run: bool,
    pub operations: Vec<RefactorOperation>,
    pub diagnostics: Vec<RefactorDiagnostic>,
}

impl RefactorPlan {
    pub fn has_conflicts(&self) -> bool {
        self.diagnostics.iter().any(|diag| diag.code.ends_with("conflict"))
    }
}

pub fn crunch_to_mantle_session() -> RefactorSession {
    RefactorSession {
        schema: REFACTOR_SESSION_SCHEMA,
        id: CRUNCH_TO_MANTLE_SESSION_ID,
        canonical_name: "mantle",
        legacy_aliases: &["crunch"],
        canonical_cli: &["mantle"],
        legacy_cli: &["crunch"],
        canonical_files: &["mantle-project.ncl", "mantle.lock", ".mantle/"],
        legacy_files: &["crunch-project.ncl", "crunch.lock", ".crunch/"],
        canonical_store_prefix: "/mantle/store",
        legacy_store_prefixes: &["/crunch/store"],
        compatibility_policy: "legacy crunch command, project files, and /crunch/store prefix are compatibility aliases; canonical identity is mantle and /mantle/store",
        validation_checks: &[
            "canonical and legacy project files must not both exist for the same surface",
            "dry-run plan/check must not mutate project files or store state",
            "apply requires explicit session id",
            "ambiguous /mantle/store and /crunch/store configuration fails with remediation",
        ],
        risk_summary: "bounded file/path identity migration; compatibility aliases retained",
    }
}

pub fn plan_crunch_to_mantle(root: &Path, configured_store_prefixes: &[String]) -> RefactorPlan {
    let session = crunch_to_mantle_session();
    let mut operations = Vec::new();
    let mut diagnostics = Vec::new();

    for (legacy, canonical) in [
        ("crunch-project.ncl", "mantle-project.ncl"),
        ("crunch.lock", "mantle.lock"),
        (".crunch", ".mantle"),
    ] {
        let legacy_path = root.join(legacy);
        let canonical_path = root.join(canonical);
        match (legacy_path.exists(), canonical_path.exists()) {
            (true, true) => diagnostics.push(RefactorDiagnostic {
                code: "mixed-file-conflict",
                message: format!("both legacy `{legacy}` and canonical `{canonical}` exist"),
                remediation: format!("choose one source of truth, then rerun `{}` plan/check", session.id),
            }),
            (true, false) => operations.push(RefactorOperation {
                kind: if legacy.starts_with('.') {
                    "rename-directory"
                } else {
                    "rename-file"
                },
                from: legacy.to_string(),
                to: canonical.to_string(),
                apply_supported: true,
            }),
            _ => {}
        }
    }

    let saw_canonical = configured_store_prefixes.iter().any(|prefix| prefix == session.canonical_store_prefix);
    let saw_legacy = configured_store_prefixes
        .iter()
        .any(|prefix| session.legacy_store_prefixes.contains(&prefix.as_str()));
    if saw_canonical && saw_legacy {
        diagnostics.push(RefactorDiagnostic {
            code: "ambiguous-store-prefix-conflict",
            message: format!(
                "both canonical `{}` and legacy `{}` store prefixes are configured",
                session.canonical_store_prefix,
                session.legacy_store_prefixes.join(", ")
            ),
            remediation: format!(
                "select `{}` as default and keep legacy prefixes only as explicit compatibility inputs",
                session.canonical_store_prefix
            ),
        });
    } else if saw_legacy && !saw_canonical {
        operations.push(RefactorOperation {
            kind: "store-prefix-default",
            from: session.legacy_store_prefixes[0].to_string(),
            to: session.canonical_store_prefix.to_string(),
            apply_supported: false,
        });
    }

    RefactorPlan {
        schema: "mantle-refactor-plan-v1",
        session_id: session.id,
        root: root.display().to_string(),
        dry_run: true,
        operations,
        diagnostics,
    }
}

pub fn apply_crunch_to_mantle(
    root: &Path,
    configured_store_prefixes: &[String],
) -> Result<RefactorPlan, RefactorError> {
    let plan = plan_crunch_to_mantle(root, configured_store_prefixes);
    if plan.has_conflicts() {
        return Err(RefactorError::Conflicts(plan));
    }
    for operation in &plan.operations {
        if !operation.apply_supported {
            return Err(RefactorError::UnsupportedApply(operation.clone()));
        }
    }
    for operation in &plan.operations {
        let from = root.join(&operation.from);
        let to = root.join(&operation.to);
        if let Some(parent) = to.parent() {
            fs::create_dir_all(parent).map_err(|err| RefactorError::Io(parent.to_path_buf(), err.to_string()))?;
        }
        fs::rename(&from, &to).map_err(|err| RefactorError::Io(from, err.to_string()))?;
    }
    let mut applied = plan;
    applied.dry_run = false;
    Ok(applied)
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum RefactorError {
    UnknownSession(String),
    MissingExplicitSession { available: Vec<&'static str> },
    Conflicts(RefactorPlan),
    UnsupportedApply(RefactorOperation),
    Io(PathBuf, String),
}

impl fmt::Display for RefactorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownSession(session) => write!(f, "unknown refactor session `{session}`"),
            Self::MissingExplicitSession { available } => {
                write!(f, "apply requires explicit refactor session; available: {}", available.join(", "))
            }
            Self::Conflicts(plan) => write!(f, "refactor session `{}` has conflicts", plan.session_id),
            Self::UnsupportedApply(operation) => write!(
                f,
                "operation `{}` from `{}` to `{}` is plan-only and cannot be applied automatically",
                operation.kind, operation.from, operation.to
            ),
            Self::Io(path, err) => write!(f, "refactor I/O error at {}: {err}", path.display()),
        }
    }
}

impl std::error::Error for RefactorError {}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use super::*;

    fn snapshot(root: &Path) -> Vec<String> {
        let mut entries = fs::read_dir(root)
            .unwrap()
            .map(|entry| entry.unwrap().file_name().to_string_lossy().to_string())
            .collect::<Vec<_>>();
        entries.sort();
        entries
    }

    #[test]
    fn crunch_to_mantle_fixture_names_compatibility_surfaces() {
        let session = crunch_to_mantle_session();
        assert_eq!(session.canonical_name, "mantle");
        assert!(session.legacy_aliases.contains(&"crunch"));
        assert!(session.canonical_cli.contains(&"mantle"));
        assert!(session.legacy_cli.contains(&"crunch"));
        assert!(session.canonical_files.contains(&"mantle-project.ncl"));
        assert!(session.legacy_files.contains(&"crunch-project.ncl"));
        assert_eq!(session.canonical_store_prefix, "/mantle/store");
        assert!(session.legacy_store_prefixes.contains(&"/crunch/store"));
    }

    #[test]
    fn plan_is_side_effect_free() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("crunch-project.ncl"), "legacy").unwrap();
        fs::write(dir.path().join("crunch.lock"), "{}").unwrap();
        let before = snapshot(dir.path());
        let plan = plan_crunch_to_mantle(dir.path(), &[]);
        let after = snapshot(dir.path());
        assert_eq!(before, after);
        assert!(plan.dry_run);
        assert_eq!(plan.operations.len(), 2);
    }

    #[test]
    fn mixed_canonical_and_legacy_files_report_conflict() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("crunch-project.ncl"), "legacy").unwrap();
        fs::write(dir.path().join("mantle-project.ncl"), "canonical").unwrap();
        let plan = plan_crunch_to_mantle(dir.path(), &[]);
        assert!(plan.has_conflicts());
        assert_eq!(plan.diagnostics[0].code, "mixed-file-conflict");
        assert!(plan.diagnostics[0].remediation.contains("choose one source of truth"));
    }

    #[test]
    fn ambiguous_store_prefixes_report_conflict() {
        let dir = tempdir().unwrap();
        let plan = plan_crunch_to_mantle(dir.path(), &["/mantle/store".to_string(), "/crunch/store".to_string()]);
        assert!(plan.has_conflicts());
        assert_eq!(plan.diagnostics[0].code, "ambiguous-store-prefix-conflict");
    }

    #[test]
    fn apply_renames_only_after_explicit_session_planning() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("crunch-project.ncl"), "legacy").unwrap();
        let applied = apply_crunch_to_mantle(dir.path(), &[]).unwrap();
        assert!(!applied.dry_run);
        assert!(!dir.path().join("crunch-project.ncl").exists());
        assert!(dir.path().join("mantle-project.ncl").exists());
    }
}
