// machine-artifact-public: structured-refactor.command-reports
use std::fmt;
use std::fs;
use std::os::unix::fs::MetadataExt;
use std::path::Path;
use std::path::PathBuf;

use mantle_application_contract::ApplicationOutcome;
use mantle_application_contract::EffectId;
use mantle_application_contract::EffectKind;
use mantle_application_contract::EffectMeasure;
use mantle_application_contract::EffectOutput;
use mantle_application_contract::Observation;
use mantle_application_contract::ObservationStatus;
use mantle_application_contract::classify_observations;
use mantle_application_contract::diagnostics::REFACTOR_READ_EFFECT;
use mantle_application_contract::diagnostics::REFACTOR_READBACK_EFFECT;
use mantle_application_contract::diagnostics::REFACTOR_RENAME_EFFECT;
use mantle_application_contract::diagnostics::plan_refactor_inspection;
use mantle_application_contract::diagnostics::plan_refactor_rename;
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

pub fn plan_crunch_to_mantle(root: &Path, configured_store_prefixes: &[String]) -> Result<RefactorPlan, RefactorError> {
    const REFACTOR_PLAN_ENTRY_CAPACITY_COUNT: usize = 4;

    let session = crunch_to_mantle_session();
    let effect_plan =
        plan_refactor_inspection().map_err(|error| RefactorError::Observation(error.code().to_string()))?;
    let mut port = HostRefactorPort { reads: 0, writes: 0 };
    let mut operations = Vec::with_capacity(REFACTOR_PLAN_ENTRY_CAPACITY_COUNT);
    let mut diagnostics = Vec::with_capacity(REFACTOR_PLAN_ENTRY_CAPACITY_COUNT);

    for (legacy, canonical) in [
        ("crunch-project.ncl", "mantle-project.ncl"),
        ("crunch.lock", "mantle.lock"),
        (".crunch", ".mantle"),
    ] {
        let legacy_path = root.join(legacy);
        let canonical_path = root.join(canonical);
        let paths = (|| Ok((port.exists(&legacy_path)?, port.exists(&canonical_path)?)))();
        let paths = match paths {
            Ok(paths) => paths,
            Err(error) => {
                let _outcome = classify_refactor_inspection(&effect_plan, port.reads, false);
                return Err(error);
            }
        };
        match paths {
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

    let has_canonical = configured_store_prefixes.iter().any(|prefix| prefix == session.canonical_store_prefix);
    let has_legacy = configured_store_prefixes
        .iter()
        .any(|prefix| session.legacy_store_prefixes.contains(&prefix.as_str()));
    if has_canonical && has_legacy {
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
    } else if has_legacy && !has_canonical {
        operations.push(RefactorOperation {
            kind: "store-prefix-default",
            from: session.legacy_store_prefixes[0].to_string(),
            to: session.canonical_store_prefix.to_string(),
            apply_supported: false,
        });
    }

    debug_assert!(operations.len() <= REFACTOR_PLAN_ENTRY_CAPACITY_COUNT);
    debug_assert!(diagnostics.len() <= REFACTOR_PLAN_ENTRY_CAPACITY_COUNT);
    classify_refactor_inspection(&effect_plan, port.reads, true)?;
    Ok(RefactorPlan {
        schema: "mantle-refactor-plan-v1",
        session_id: session.id,
        root: root.display().to_string(),
        dry_run: true,
        operations,
        diagnostics,
    })
}

pub fn apply_crunch_to_mantle(
    root: &Path,
    configured_store_prefixes: &[String],
) -> Result<RefactorPlan, RefactorError> {
    let plan = plan_crunch_to_mantle(root, configured_store_prefixes)?;
    if plan.has_conflicts() {
        return Err(RefactorError::Conflicts(plan));
    }
    debug_assert!(plan.diagnostics.is_empty());
    debug_assert!(plan.dry_run);
    for operation in &plan.operations {
        if !operation.apply_supported {
            return Err(RefactorError::UnsupportedApply(operation.clone()));
        }
    }
    for operation in &plan.operations {
        rename_refactor_path(&root.join(&operation.from), &root.join(&operation.to))?;
    }
    let mut applied = plan;
    applied.dry_run = false;
    Ok(applied)
}

trait RefactorPort {
    fn exists(&mut self, path: &Path) -> Result<bool, RefactorError>;
    fn rename(&mut self, from: &Path, to: &Path) -> Result<(), RefactorError>;
    fn metadata(&mut self, path: &Path) -> Result<fs::Metadata, std::io::Error>;
}

struct HostRefactorPort {
    reads: u32,
    writes: u32,
}

impl RefactorPort for HostRefactorPort {
    fn exists(&mut self, path: &Path) -> Result<bool, RefactorError> {
        self.reads = self.reads.saturating_add(1);
        match fs::metadata(path) {
            Ok(_) => Ok(true),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
            Err(error) => Err(RefactorError::Io(path.to_path_buf(), error.to_string())),
        }
    }
    fn metadata(&mut self, path: &Path) -> Result<fs::Metadata, std::io::Error> {
        self.reads = self.reads.saturating_add(1);
        fs::symlink_metadata(path)
    }

    fn rename(&mut self, from: &Path, to: &Path) -> Result<(), RefactorError> {
        self.writes = self.writes.saturating_add(1);
        if let Some(parent) = to.parent() {
            fs::create_dir_all(parent).map_err(|error| RefactorError::Io(parent.to_path_buf(), error.to_string()))?;
        }
        fs::rename(from, to).map_err(|error| RefactorError::Io(from.to_path_buf(), error.to_string()))
    }
}

fn classify_refactor_inspection(
    plan: &mantle_application_contract::EffectPlan,
    calls: u32,
    succeeded: bool,
) -> Result<(), RefactorError> {
    let observation = Observation {
        effect_id: EffectId(REFACTOR_READ_EFFECT.to_string()),
        kind: EffectKind::ReadFiles,
        status: if succeeded {
            ObservationStatus::Succeeded
        } else {
            ObservationStatus::Failed
        },
        output: EffectOutput::None,
        usage: EffectMeasure::Calls(calls),
        diagnostics_code: (!succeeded).then(|| "refactor-inspection-failed".to_string()),
    };
    match classify_observations(plan, &[observation]) {
        ApplicationOutcome::Completed => Ok(()),
        _ => Err(RefactorError::Observation("refactor-inspection-failed".to_string())),
    }
}

fn rename_refactor_path(from: &Path, to: &Path) -> Result<(), RefactorError> {
    let destination = to.display().to_string();
    let plan =
        plan_refactor_rename(&destination).map_err(|error| RefactorError::Observation(error.code().to_string()))?;
    let mut port = HostRefactorPort { reads: 0, writes: 0 };
    let result: Result<bool, RefactorError> = (|| {
        let before = port.metadata(from).map_err(|error| RefactorError::Io(from.to_path_buf(), error.to_string()))?;
        port.rename(from, to)?;
        let after = port.metadata(to);
        let source = port.metadata(from);
        let is_same_object = after
            .as_ref()
            .is_ok_and(|metadata| metadata.dev() == before.dev() && metadata.ino() == before.ino());
        let is_source_absent = source.as_ref().is_err_and(|error| error.kind() == std::io::ErrorKind::NotFound);
        Ok(is_same_object && is_source_absent)
    })();
    let observed = result.as_ref().is_ok_and(|matched| *matched);
    let output = if observed {
        EffectOutput::Identity(destination)
    } else {
        EffectOutput::None
    };
    let status = if observed {
        ObservationStatus::Succeeded
    } else {
        ObservationStatus::Failed
    };
    let observations = [
        Observation {
            effect_id: EffectId(REFACTOR_RENAME_EFFECT.to_string()),
            kind: EffectKind::WriteFiles,
            status: if port.writes == 0 {
                ObservationStatus::Skipped
            } else {
                status
            },
            output: output.clone(),
            diagnostics_code: (!observed).then(|| "refactor-rename-mismatch".to_string()),
            usage: EffectMeasure::Calls(port.writes),
        },
        Observation {
            effect_id: EffectId(REFACTOR_READBACK_EFFECT.to_string()),
            kind: EffectKind::ReadFiles,
            status,
            output,
            diagnostics_code: (!observed).then(|| "refactor-rename-mismatch".to_string()),
            usage: EffectMeasure::Calls(port.reads),
        },
    ];
    let outcome = classify_observations(&plan, &observations);
    result?;
    if outcome != ApplicationOutcome::Completed {
        return Err(RefactorError::Observation("refactor-rename-mismatch".to_string()));
    }
    Ok(())
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum RefactorError {
    UnknownSession(String),
    MissingExplicitSession { available: Vec<&'static str> },
    Conflicts(RefactorPlan),
    UnsupportedApply(RefactorOperation),
    Io(PathBuf, String),
    Observation(String),
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
            Self::Observation(code) => write!(f, "refactor observation rejected: {code}"),
        }
    }
}

impl std::error::Error for RefactorError {}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use super::*;

    #[test]
    fn conflicting_apply_preserves_both_sources() {
        let directory = tempdir().unwrap();
        fs::write(directory.path().join("crunch-project.ncl"), "legacy").unwrap();
        fs::write(directory.path().join("mantle-project.ncl"), "canonical").unwrap();
        assert!(matches!(apply_crunch_to_mantle(directory.path(), &[]), Err(RefactorError::Conflicts(_))));
        assert_eq!(fs::read_to_string(directory.path().join("crunch-project.ncl")).unwrap(), "legacy");
        assert_eq!(fs::read_to_string(directory.path().join("mantle-project.ncl")).unwrap(), "canonical");
    }

    fn snapshot(root: &Path) -> Vec<String> {
        let mut entries = fs::read_dir(root)
            .unwrap()
            .map(|entry| entry.unwrap().file_name().to_string_lossy().to_string())
            .collect::<Vec<_>>();
        entries.sort();
        entries
    }

    #[test]
    fn plan_is_side_effect_free() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("crunch-project.ncl"), "legacy").unwrap();
        fs::write(dir.path().join("crunch.lock"), "{}").unwrap();
        let before = snapshot(dir.path());
        let plan = plan_crunch_to_mantle(dir.path(), &[]).unwrap();
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
        let plan = plan_crunch_to_mantle(dir.path(), &[]).unwrap();
        assert!(plan.has_conflicts());
        assert_eq!(plan.diagnostics[0].code, "mixed-file-conflict");
        assert!(plan.diagnostics[0].remediation.contains("choose one source of truth"));
    }

    #[test]
    fn ambiguous_store_prefixes_report_conflict() {
        let dir = tempdir().unwrap();
        let plan =
            plan_crunch_to_mantle(dir.path(), &["/mantle/store".to_string(), "/crunch/store".to_string()]).unwrap();
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
        assert_eq!(fs::read_to_string(dir.path().join("mantle-project.ncl")).unwrap(), "legacy");
    }
}
