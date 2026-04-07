//! CLI handlers for project management commands.
//!
//! Each function handles one subcommand, delegating to crunch-project
//! for the real logic. This module owns argument parsing, file I/O,
//! and output formatting — nothing else.

use crate::errors::RunError;
use crunch_project::{
    DriftStatus, Lockfile, ProjectManifest, SchemaVersion, check_drift,
    check_manifest_lock, generate_inputs_ncl, refresh_inputs, apply_outcomes,
    list_stale, upgrade_lockfile, RefreshOutcome, Severity,
};
use std::path::Path;

/// Project file names.
const MANIFEST_FILE: &str = "crunch-project.ncl";
const LOCK_FILE: &str = "crunch.lock";
const INPUTS_DIR: &str = ".crunch";
const INPUTS_FILE: &str = ".crunch/inputs.ncl";
const GITIGNORE_ENTRY: &str = ".crunch/";

/// `crunch init` — scaffold a new project.
pub fn cmd_init(dir: &Path) -> Result<(), RunError> {
    let manifest_path = dir.join(MANIFEST_FILE);
    let lock_path = dir.join(LOCK_FILE);
    let inputs_dir = dir.join(INPUTS_DIR);
    let inputs_path = dir.join(INPUTS_FILE);

    if manifest_path.exists() {
        return Err(RunError::Internal(format!(
            "{MANIFEST_FILE} already exists in {}",
            dir.display()
        )));
    }

    // Write manifest template
    let manifest_template = r#"# crunch project manifest.
# See: lib/project.ncl for the contract definition.
{
  version = "1.0.0",
  inputs = [],
  patches = [],
}
"#;
    std::fs::write(&manifest_path, manifest_template)
        .map_err(|e| RunError::Internal(format!("writing {MANIFEST_FILE}: {e}")))?;

    // Write empty lockfile
    let lock = Lockfile::new();
    let lock_json = lock
        .to_json()
        .map_err(|e| RunError::Internal(format!("serializing lock: {e}")))?;
    std::fs::write(&lock_path, &lock_json)
        .map_err(|e| RunError::Internal(format!("writing {LOCK_FILE}: {e}")))?;

    // Create .crunch/ and generate empty inputs
    std::fs::create_dir_all(&inputs_dir)
        .map_err(|e| RunError::Internal(format!("creating {INPUTS_DIR}/: {e}")))?;
    let inputs_ncl = generate_inputs_ncl(&lock);
    std::fs::write(&inputs_path, &inputs_ncl)
        .map_err(|e| RunError::Internal(format!("writing {INPUTS_FILE}: {e}")))?;

    // Add .crunch/ to .gitignore if not already there
    add_gitignore_entry(dir);

    eprintln!("Initialized crunch project:");
    eprintln!("  {MANIFEST_FILE}  (edit this)");
    eprintln!("  {LOCK_FILE}      (machine-managed)");
    eprintln!("  {INPUTS_FILE}    (generated)");
    Ok(())
}

/// `crunch check` — validate project state.
pub fn cmd_check(dir: &Path) -> Result<(), RunError> {
    let manifest = load_manifest(dir)?;
    let lock = load_lockfile(dir)?;

    // 1. Validate manifest internally
    let manifest_problems = manifest.validate();
    for p in &manifest_problems {
        eprintln!("manifest: {p}");
    }

    // 2. Validate lockfile internally
    let lock_problems = lock.validate();
    for p in &lock_problems {
        eprintln!("lockfile: {p}");
    }

    // 3. Check manifest-lock consistency
    let report = check_manifest_lock(&manifest, &lock);
    for issue in &report.issues {
        let prefix = match issue.severity {
            Severity::Error => "error",
            Severity::Warning => "warning",
        };
        eprintln!("{prefix}: {}", issue.message);
    }

    // 4. Check drift
    let inputs_path = dir.join(INPUTS_FILE);
    let actual_content = std::fs::read_to_string(&inputs_path).ok();
    let drift = check_drift(&lock, actual_content.as_deref());
    match &drift {
        DriftStatus::InSync => {}
        DriftStatus::Missing => {
            eprintln!("drift: {INPUTS_FILE} does not exist");
        }
        DriftStatus::Drifted { .. } => {
            eprintln!("drift: {INPUTS_FILE} is out of date");
        }
    }

    let has_errors = !manifest_problems.is_empty()
        || !lock_problems.is_empty()
        || report.has_errors()
        || !drift.is_ok();

    if has_errors {
        Err(RunError::Internal("project check failed".into()))
    } else {
        eprintln!("project check passed");
        Ok(())
    }
}

/// `crunch show` — render resolved input state.
pub fn cmd_show(dir: &Path) -> Result<(), RunError> {
    let manifest = load_manifest(dir)?;
    let lock = load_lockfile(dir)?;

    println!("Project: {} (schema {})", MANIFEST_FILE, manifest.version);
    println!();

    if lock.inputs.is_empty() {
        println!("No locked inputs.");
        return Ok(());
    }

    for (name, entry) in &lock.inputs {
        let manifest_input = manifest.inputs.iter().find(|i| i.name == *name);
        let frozen = manifest_input.map_or(false, |i| i.frozen);
        let frozen_tag = if frozen { " [frozen]" } else { "" };

        let kind_str = match &entry.kind {
            crunch_project::LockedKind::File { url } => format!("file: {url}"),
            crunch_project::LockedKind::Tarball { url } => format!("tarball: {url}"),
            crunch_project::LockedKind::Git {
                repository,
                rev,
                ref_name,
            } => {
                let ref_str = ref_name
                    .as_deref()
                    .map(|r| format!(" ({r})"))
                    .unwrap_or_default();
                format!("git: {repository} @ {rev}{ref_str}")
            }
        };

        println!("{name}{frozen_tag}");
        println!("  {kind_str}");
        println!("  hash: {} {}", entry.hash.algo, entry.hash.value);

        if !entry.mirrors.is_empty() {
            println!("  mirrors: {}", entry.mirrors.join(", "));
        }
        if !entry.patches.is_empty() {
            println!("  patches: {}", entry.patches.join(", "));
        }
        println!();
    }

    Ok(())
}

/// `crunch refresh [names...]` — update inputs.
pub fn cmd_refresh(dir: &Path, selected: &[String]) -> Result<(), RunError> {
    let manifest = load_manifest(dir)?;
    let lock = load_lockfile(dir)?;

    // Use a stub resolver for now. Real network resolution will be
    // wired when the build pipeline exposes hash_url_content and
    // resolve_git_rev as standalone helpers.
    let resolver = StubResolver;
    let outcomes = refresh_inputs(&manifest, &lock, selected, &resolver);

    for outcome in &outcomes {
        match outcome {
            RefreshOutcome::Updated(r) => {
                eprintln!("updated: {}", r.name);
            }
            RefreshOutcome::Unchanged { name } => {
                eprintln!("unchanged: {name}");
            }
            RefreshOutcome::Frozen { name } => {
                eprintln!("frozen (skipped): {name}");
            }
            RefreshOutcome::Failed { name, reason } => {
                eprintln!("failed: {name}: {reason}");
            }
        }
    }

    // Always apply outcomes — this also resolves/repairs patch lock data
    // even when no inputs changed.
    let result = apply_outcomes(&manifest, &lock, &outcomes, &resolver);

    if result.has_changes() {
        write_lockfile(dir, &result.lock)?;
        write_inputs_ncl(dir, &result.lock)?;
        if result.inputs_changed > 0 {
            eprintln!("{} input(s) updated", result.inputs_changed);
        }
        if result.patches_changed {
            eprintln!("patch lock data updated");
        }
    } else {
        eprintln!("all inputs up to date");
    }

    Ok(())
}

/// `crunch list-stale` — show which inputs would change.
pub fn cmd_list_stale(dir: &Path) -> Result<(), RunError> {
    let manifest = load_manifest(dir)?;
    let lock = load_lockfile(dir)?;

    let resolver = StubResolver;
    let stale = list_stale(&manifest, &lock, &resolver);

    if stale.is_empty() {
        println!("all inputs up to date");
    } else {
        for name in &stale {
            println!("{name}");
        }
    }

    Ok(())
}

/// `crunch upgrade` — migrate project files to current schema.
pub fn cmd_upgrade(dir: &Path) -> Result<(), RunError> {
    let lock = load_lockfile(dir)?;

    if lock.version == SchemaVersion::CURRENT {
        eprintln!(
            "project files already at current version ({})",
            SchemaVersion::CURRENT
        );
        return Ok(());
    }

    eprintln!(
        "upgrading lockfile from {} to {}",
        lock.version,
        SchemaVersion::CURRENT
    );
    let upgraded = upgrade_lockfile(lock)
        .map_err(|e| RunError::Internal(format!("upgrade: {e}")))?;

    write_lockfile(dir, &upgraded)?;
    write_inputs_ncl(dir, &upgraded)?;
    eprintln!("upgrade complete");
    Ok(())
}

// ── Helpers ─────────────────────────────────────────────────────

fn load_manifest(dir: &Path) -> Result<ProjectManifest, RunError> {
    let path = dir.join(MANIFEST_FILE);
    if !path.exists() {
        return Err(RunError::Internal(format!(
            "{MANIFEST_FILE} not found in {}. Run `crunch init` first.",
            dir.display()
        )));
    }

    let import_paths = vec![dir.as_os_str().to_owned()];
    crunch_eval::evaluate_and_deserialize(&path, &import_paths)
        .map_err(|e| RunError::Eval(format!("loading {MANIFEST_FILE}: {e}")))
}

fn load_lockfile(dir: &Path) -> Result<Lockfile, RunError> {
    let path = dir.join(LOCK_FILE);
    if !path.exists() {
        return Err(RunError::Internal(format!(
            "{LOCK_FILE} not found in {}. Run `crunch init` first.",
            dir.display()
        )));
    }

    let content = std::fs::read_to_string(&path)
        .map_err(|e| RunError::Internal(format!("reading {LOCK_FILE}: {e}")))?;
    Lockfile::from_json(&content)
        .map_err(|e| RunError::Internal(format!("parsing {LOCK_FILE}: {e}")))
}

fn write_lockfile(dir: &Path, lock: &Lockfile) -> Result<(), RunError> {
    let path = dir.join(LOCK_FILE);
    let json = lock
        .to_json()
        .map_err(|e| RunError::Internal(format!("serializing lock: {e}")))?;
    std::fs::write(&path, &json)
        .map_err(|e| RunError::Internal(format!("writing {LOCK_FILE}: {e}")))?;
    Ok(())
}

fn write_inputs_ncl(dir: &Path, lock: &Lockfile) -> Result<(), RunError> {
    let inputs_dir = dir.join(INPUTS_DIR);
    std::fs::create_dir_all(&inputs_dir)
        .map_err(|e| RunError::Internal(format!("creating {INPUTS_DIR}/: {e}")))?;
    let ncl = generate_inputs_ncl(lock);
    let path = dir.join(INPUTS_FILE);
    std::fs::write(&path, &ncl)
        .map_err(|e| RunError::Internal(format!("writing {INPUTS_FILE}: {e}")))?;
    Ok(())
}

fn add_gitignore_entry(dir: &Path) {
    let gitignore = dir.join(".gitignore");
    let content = std::fs::read_to_string(&gitignore).unwrap_or_default();
    if !content.lines().any(|line| line.trim() == GITIGNORE_ENTRY) {
        let mut new_content = content;
        if !new_content.is_empty() && !new_content.ends_with('\n') {
            new_content.push('\n');
        }
        new_content.push_str(GITIGNORE_ENTRY);
        new_content.push('\n');
        let _ = std::fs::write(&gitignore, &new_content);
    }
}

/// Stub resolver that returns None for all queries.
///
/// Real implementations will call `git ls-remote` and fetch+hash
/// content through the existing build pipeline helpers.
struct StubResolver;

impl crunch_project::RefreshResolver for StubResolver {
    fn resolve_git_rev(
        &self,
        _repository: &str,
        _reference: &crunch_project::GitReference,
    ) -> Result<Option<String>, crunch_project::Error> {
        Ok(None)
    }

    fn hash_url_content(
        &self,
        _url: &str,
        _algo: &crunch_project::HashAlgo,
    ) -> Result<Option<String>, crunch_project::Error> {
        Ok(None)
    }
}
