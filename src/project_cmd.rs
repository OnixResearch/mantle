//! CLI handlers for project management commands.
//!
//! Each function handles one subcommand, delegating to crunch-project
//! for the real logic. This module owns argument parsing, file I/O,
//! and output formatting — nothing else.

use std::io::ErrorKind;
use std::path::Path;
use std::path::PathBuf;

use crunch_project::Lockfile;
use crunch_project::ProjectManifest;
use crunch_project::ProjectRetentionState;
use crunch_project::ProjectSoundnessClass;
use crunch_project::ProjectSoundnessInput;
use crunch_project::ProjectSoundnessIssue;
use crunch_project::ProjectSoundnessMode;
use crunch_project::ProjectSoundnessReport;
use crunch_project::ProjectSoundnessSubject;
use crunch_project::RefreshFailure;
use crunch_project::RefreshOutcome;
use crunch_project::RetentionDiagnostic;
use crunch_project::RetentionDiagnosticKind;
use crunch_project::RetentionInputState;
use crunch_project::RetentionInputStatus;
use crunch_project::RetentionPlan;
use crunch_project::RetentionPlanRequest;
use crunch_project::RetentionRootActionKind;
use crunch_project::RetentionRootFact;
use crunch_project::RetentionRootRecord;
use crunch_project::SchemaVersion;
use crunch_project::apply_outcomes;
use crunch_project::check_project_soundness;
use crunch_project::generate_inputs_ncl;
use crunch_project::list_stale_with_options;
use crunch_project::plan_retention_roots;
use crunch_project::project_soundness_parse_error;
use crunch_project::refresh_inputs_with_options;
use crunch_project::upgrade_lockfile;

use crate::errors::RunError;
use crate::project_resolve::LiveResolver;

/// Project file names.
const MANIFEST_FILE: &str = "mantle-project.ncl";
const LOCK_FILE: &str = "mantle.lock";
const INPUTS_DIR: &str = ".mantle";
const INPUTS_FILE: &str = ".mantle/inputs.ncl";
const RETENTION_STATE_FILE: &str = ".mantle/retention.json";
const RETENTION_STATE_TMP_FILE: &str = ".mantle/retention.json.tmp";
const RETENTION_ROOTS_DIR: &str = ".mantle/retention-roots";
const ATOMIC_TMP_SUFFIX: &str = "tmp";
const GITIGNORE_ENTRY: &str = ".mantle/";
const LEGACY_MANIFEST_FILE: &str = "crunch-project.ncl";
const LEGACY_LOCK_FILE: &str = "crunch.lock";
const LEGACY_INPUTS_DIR: &str = ".crunch";
const PROJECT_CHECK_FAILURE_EXIT_CODE: u8 = 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectCheckOutput {
    Human,
    Json,
}

/// `crunch init` — scaffold a new project.
pub fn cmd_init(dir: &Path) -> Result<(), RunError> {
    reject_conflicting_legacy_project_files(dir)?;
    let manifest_path = dir.join(MANIFEST_FILE);
    let lock_path = dir.join(LOCK_FILE);
    let inputs_dir = dir.join(INPUTS_DIR);
    let inputs_path = dir.join(INPUTS_FILE);

    if manifest_path.exists() {
        return Err(RunError::Internal(format!("{MANIFEST_FILE} already exists in {}", dir.display())));
    }

    // Write manifest template
    let manifest_template = r#"# Mantle project manifest.
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
    let lock_json = lock.clone().to_json().map_err(|e| RunError::Internal(format!("serializing lock: {e}")))?;
    std::fs::write(&lock_path, &lock_json).map_err(|e| RunError::Internal(format!("writing {LOCK_FILE}: {e}")))?;

    // Create .mantle/ and generate empty inputs
    std::fs::create_dir_all(&inputs_dir).map_err(|e| RunError::Internal(format!("creating {INPUTS_DIR}/: {e}")))?;
    let inputs_ncl = generate_inputs_ncl(lock);
    std::fs::write(&inputs_path, &inputs_ncl).map_err(|e| RunError::Internal(format!("writing {INPUTS_FILE}: {e}")))?;
    write_retention_state_records(dir, Vec::new())?;

    // Add .mantle/ to .gitignore if not already there
    add_gitignore_entry(dir);

    eprintln!("Initialized Mantle project:");
    eprintln!("  {MANIFEST_FILE}  (edit this)");
    eprintln!("  {LOCK_FILE}      (machine-managed)");
    eprintln!("  {INPUTS_FILE}    (generated)");
    Ok(())
}

/// `crunch check` — validate project state.
pub fn cmd_check(dir: &Path, output: ProjectCheckOutput, probes: bool, trust: bool) -> Result<(), RunError> {
    reject_conflicting_legacy_project_files(dir)?;
    let manifest = match load_manifest(dir) {
        Ok(manifest) => manifest,
        Err(error) => {
            return finish_check_report(
                project_soundness_parse_error(
                    ProjectSoundnessClass::ManifestParseError,
                    ProjectSoundnessSubject::file(MANIFEST_FILE),
                    error.message().to_string(),
                ),
                output,
            );
        }
    };
    let lock = match load_lockfile(dir) {
        Ok(lock) => lock,
        Err(error) => {
            return finish_check_report(
                project_soundness_parse_error(
                    ProjectSoundnessClass::LockfileParseError,
                    ProjectSoundnessSubject::file(LOCK_FILE),
                    error.message().to_string(),
                ),
                output,
            );
        }
    };

    let inputs_path = dir.join(INPUTS_FILE);
    let generated_inputs = std::fs::read_to_string(&inputs_path).ok();
    let retention_state = load_retention_shell_state(dir);
    let retention_plan = plan_retention_roots(RetentionPlanRequest {
        manifest: manifest.clone(),
        lock: lock.clone(),
        existing_roots: retention_state.facts,
    });
    let mut supplemental_facts = retention_state.supplemental_facts;
    supplemental_facts.extend(retention_soundness_facts(&retention_plan));
    let report = check_project_soundness(ProjectSoundnessInput {
        manifest,
        lock,
        generated_inputs,
        supplemental_facts,
        mode: ProjectSoundnessMode::from_dynamic_requests(probes, trust),
    });
    finish_check_report(report, output)
}

fn finish_check_report(report: ProjectSoundnessReport, output: ProjectCheckOutput) -> Result<(), RunError> {
    match output {
        ProjectCheckOutput::Human => render_check_report_human(&report),
        ProjectCheckOutput::Json => render_check_report_json(&report)?,
    }
    if report.valid {
        Ok(())
    } else {
        Err(RunError::Reported(PROJECT_CHECK_FAILURE_EXIT_CODE))
    }
}

fn render_check_report_json(report: &ProjectSoundnessReport) -> Result<(), RunError> {
    let rendered = serde_json::to_string_pretty(report)
        .map_err(|err| RunError::Internal(format!("rendering project soundness JSON: {err}")))?;
    println!("{rendered}");
    Ok(())
}

fn render_check_report_human(report: &ProjectSoundnessReport) {
    for issue in &report.issues {
        render_check_issue_human(issue);
    }
    if report.valid {
        eprintln!(
            "project check passed (static project soundness only; build, source freshness, trust, and reproducibility are not proven)"
        );
    }
}

fn render_check_issue_human(issue: &ProjectSoundnessIssue) {
    eprintln!(
        "{} [{}] {} {}: {}",
        issue.severity.as_str(),
        issue.class.as_str(),
        issue.subject.kind,
        issue.subject.name,
        issue.message
    );
    if let Some(fix_guidance) = &issue.fix_guidance {
        eprintln!("  fix: {fix_guidance}");
    }
}

/// `crunch show` — render resolved input state.
pub fn cmd_show(dir: &Path) -> Result<(), RunError> {
    reject_conflicting_legacy_project_files(dir)?;
    let manifest = load_manifest(dir)?;
    let lock = load_lockfile(dir)?;
    let retention_state = load_retention_shell_state(dir);
    let retention_plan = plan_retention_roots(RetentionPlanRequest {
        manifest: manifest.clone(),
        lock: lock.clone(),
        existing_roots: retention_state.facts,
    });

    println!("Project: {} (schema {})", MANIFEST_FILE, manifest.version);
    println!();

    if lock.inputs.is_empty() {
        println!("No locked inputs.");
        return Ok(());
    }

    for (name, entry) in &lock.inputs {
        let manifest_input = manifest.inputs.iter().find(|i| i.name == *name);
        let frozen = manifest_input.is_some_and(|i| i.frozen);
        let frozen_tag = if frozen { " [frozen]" } else { "" };

        let kind_str = match &entry.kind {
            crunch_project::LockedKind::File { url } => format!("file: {url}"),
            crunch_project::LockedKind::Tarball { url } => format!("tarball: {url}"),
            crunch_project::LockedKind::Git {
                repository,
                rev,
                ref_name,
            } => {
                let ref_str = ref_name.as_deref().map(|r| format!(" ({r})")).unwrap_or_default();
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
        if let Some(trust) = &entry.trust {
            println!("  trust: {} signer(s) via {} ({})", trust.signers.len(), trust.verifier.as_str(), trust.claim);
        }
        println!("  retention: {}", retention_status_text(name, &retention_plan));
        println!();
    }

    Ok(())
}

/// `crunch refresh [names...]` — update inputs.
pub fn cmd_refresh(dir: &Path, selected: &[String], no_network: bool) -> Result<(), RunError> {
    reject_conflicting_legacy_project_files(dir)?;
    let manifest = load_manifest(dir)?;
    let lock = load_lockfile(dir)?;
    let resolver = LiveResolver::new(dir);
    let outcomes = refresh_inputs_with_options(&manifest, &lock, selected, &resolver, no_network);
    print_refresh_outcomes(&outcomes);

    let result = apply_outcomes(&manifest, &lock, &outcomes, &resolver);
    let input_failures = collect_outcome_failures(&outcomes);
    print_patch_failures(&result.failures);

    if result.has_changes {
        let problems = result.lock.clone().validate();
        if !problems.is_empty() {
            for problem in &problems {
                eprintln!("lockfile warning: {problem}");
            }
        }
        write_lockfile(dir, &result.lock)?;
        write_inputs_ncl(dir, &result.lock)?;
        write_retention_plan(dir, &manifest, &result.lock)?;
        if result.inputs_changed > 0 {
            eprintln!("{} input(s) updated", result.inputs_changed);
        }
        if result.patches_changed {
            eprintln!("patch lock data updated");
        }
    } else if input_failures.is_empty() && result.failures.is_empty() {
        eprintln!("all inputs up to date");
    }

    let failure_count = input_failures.len() + result.failures.len();
    if failure_count > 0 {
        return Err(RunError::Internal(format!("refresh failed for {failure_count} item(s)")));
    }

    Ok(())
}

/// `crunch list-stale` — show which inputs would change.
pub fn cmd_list_stale(dir: &Path, no_network: bool) -> Result<(), RunError> {
    reject_conflicting_legacy_project_files(dir)?;
    let manifest = load_manifest(dir)?;
    let lock = load_lockfile(dir)?;
    let resolver = LiveResolver::new(dir);
    let report = list_stale_with_options(&manifest, &lock, &resolver, no_network);

    for name in &report.stale {
        println!("{name}");
    }
    for name in &report.unchanged {
        eprintln!("unchanged: {name}");
    }
    for skipped in &report.skipped {
        eprintln!("skipped: {}: {}", skipped.name, skipped.reason);
    }
    for blocked in &report.network_required {
        eprintln!("network-required: {}: {}", blocked.name, blocked.reason);
    }
    for failure in &report.failed {
        eprintln!("failed: {}: {}", failure.name, failure.reason);
    }
    if report.stale.is_empty()
        && report.failed.is_empty()
        && report.skipped.is_empty()
        && report.network_required.is_empty()
    {
        println!("all inputs up to date");
    }
    let blocker_count = report.failed.len() + report.network_required.len();
    if blocker_count > 0 {
        return Err(RunError::Internal(format!("stale check failed for {blocker_count} item(s)")));
    }

    Ok(())
}

/// `crunch upgrade` — migrate project files to current schema.
pub fn cmd_upgrade(dir: &Path) -> Result<(), RunError> {
    reject_conflicting_legacy_project_files(dir)?;
    let lock = load_lockfile(dir)?;

    if lock.version == SchemaVersion::CURRENT {
        eprintln!("project files already at current version ({})", SchemaVersion::CURRENT);
        return Ok(());
    }

    eprintln!("upgrading lockfile from {} to {}", lock.version, SchemaVersion::CURRENT);
    let upgraded = upgrade_lockfile(lock).map_err(|e| RunError::Internal(format!("upgrade: {e}")))?;

    let manifest = load_manifest(dir)?;
    write_lockfile(dir, &upgraded)?;
    write_inputs_ncl(dir, &upgraded)?;
    write_retention_plan(dir, &manifest, &upgraded)?;
    eprintln!("upgrade complete");
    Ok(())
}

// ── Helpers ─────────────────────────────────────────────────────

fn load_manifest(dir: &Path) -> Result<ProjectManifest, RunError> {
    let path = dir.join(MANIFEST_FILE);
    if !path.exists() {
        return Err(RunError::Internal(format!(
            "{MANIFEST_FILE} not found in {}. Run `mantle init` first.",
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
            "{LOCK_FILE} not found in {}. Run `mantle init` first.",
            dir.display()
        )));
    }

    let content =
        std::fs::read_to_string(&path).map_err(|e| RunError::Internal(format!("reading {LOCK_FILE}: {e}")))?;
    Lockfile::from_json(content).map_err(|e| RunError::Internal(format!("parsing {LOCK_FILE}: {e}")))
}

fn write_lockfile(dir: &Path, lock: &Lockfile) -> Result<(), RunError> {
    let path = dir.join(LOCK_FILE);
    let json = lock.clone().to_json().map_err(|e| RunError::Internal(format!("serializing lock: {e}")))?;
    write_file_atomic(&path, json.as_bytes(), LOCK_FILE)
}

fn write_inputs_ncl(dir: &Path, lock: &Lockfile) -> Result<(), RunError> {
    let inputs_dir = dir.join(INPUTS_DIR);
    std::fs::create_dir_all(&inputs_dir).map_err(|e| RunError::Internal(format!("creating {INPUTS_DIR}/: {e}")))?;
    let ncl = generate_inputs_ncl(lock.clone());
    let path = dir.join(INPUTS_FILE);
    write_file_atomic(&path, ncl.as_bytes(), INPUTS_FILE)
}

#[derive(Debug)]
struct RetentionShellState {
    facts: Vec<RetentionRootFact>,
    supplemental_facts: Vec<crunch_project::ProjectSoundnessFact>,
}

fn load_retention_shell_state(dir: &Path) -> RetentionShellState {
    let mut supplemental_facts = Vec::new();
    let mut facts = load_retention_facts_from_file(dir, &dir.join(RETENTION_STATE_FILE), true, &mut supplemental_facts);
    facts.extend(load_retention_facts_from_file(
        dir,
        &dir.join(RETENTION_STATE_TMP_FILE),
        false,
        &mut supplemental_facts,
    ));
    RetentionShellState {
        facts,
        supplemental_facts,
    }
}

fn load_retention_facts_from_file(
    dir: &Path,
    path: &Path,
    committed: bool,
    supplemental_facts: &mut Vec<crunch_project::ProjectSoundnessFact>,
) -> Vec<RetentionRootFact> {
    if !path.exists() {
        return Vec::new();
    }
    let state = match read_retention_state(path) {
        Ok(state) => state,
        Err(message) => {
            supplemental_facts.push(retention_shell_fact(path, message));
            return Vec::new();
        }
    };
    retention_state_facts(dir, state, committed)
}

fn read_retention_state(path: &Path) -> Result<ProjectRetentionState, String> {
    let text = std::fs::read_to_string(path).map_err(|err| format!("reading {}: {err}", path.display()))?;
    let state = serde_json::from_str::<ProjectRetentionState>(&text)
        .map_err(|err| format!("parsing {}: {err}", path.display()))?;
    let problems = state.validate();
    if problems.is_empty() {
        Ok(state)
    } else {
        Err(format!("invalid retention state {}: {}", path.display(), problems.join(", ")))
    }
}

fn retention_state_facts(dir: &Path, state: ProjectRetentionState, committed: bool) -> Vec<RetentionRootFact> {
    state
        .records
        .into_iter()
        .map(|mut record| {
            record.committed = committed && record.committed;
            let root_exists = retention_root_path(dir, &record).is_file();
            RetentionRootFact { record, root_exists }
        })
        .collect()
}

fn retention_shell_fact(path: &Path, message: String) -> crunch_project::ProjectSoundnessFact {
    crunch_project::ProjectSoundnessFact::warning(
        ProjectSoundnessClass::RetentionRootMismatch,
        ProjectSoundnessSubject::file(path.display().to_string()),
        message,
    )
}

fn retention_soundness_facts(plan: &RetentionPlan) -> Vec<crunch_project::ProjectSoundnessFact> {
    plan.diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.kind != RetentionDiagnosticKind::GcEligible)
        .map(retention_diagnostic_fact)
        .collect()
}

fn retention_diagnostic_fact(diagnostic: &RetentionDiagnostic) -> crunch_project::ProjectSoundnessFact {
    let subject = ProjectSoundnessSubject::input(diagnostic.input_name.clone());
    match diagnostic.kind {
        RetentionDiagnosticKind::InvalidPolicy => crunch_project::ProjectSoundnessFact::error(
            ProjectSoundnessClass::RetentionRootMismatch,
            subject,
            diagnostic.message.clone(),
        ),
        _ => crunch_project::ProjectSoundnessFact::warning(
            ProjectSoundnessClass::RetentionRootMismatch,
            subject,
            diagnostic.message.clone(),
        ),
    }
}

fn write_retention_plan(dir: &Path, manifest: &ProjectManifest, lock: &Lockfile) -> Result<(), RunError> {
    let shell_state = load_retention_shell_state(dir);
    let plan = plan_retention_roots(RetentionPlanRequest {
        manifest: manifest.clone(),
        lock: lock.clone(),
        existing_roots: shell_state.facts,
    });
    for action in plan.actions.iter().filter(|action| action.kind == RetentionRootActionKind::CreateRoot) {
        write_retention_root_marker(dir, &action.record)?;
    }
    write_retention_state_records(dir, plan.retained_records.clone())?;
    remove_retention_roots(dir, &plan)
}

fn write_retention_state_records(dir: &Path, records: Vec<RetentionRootRecord>) -> Result<(), RunError> {
    let inputs_dir = dir.join(INPUTS_DIR);
    std::fs::create_dir_all(&inputs_dir).map_err(|err| RunError::Internal(format!("creating {INPUTS_DIR}/: {err}")))?;
    let state = ProjectRetentionState {
        schema: crunch_project::RETENTION_STATE_SCHEMA.into(),
        records,
    };
    let json = serde_json::to_string_pretty(&state)
        .map_err(|err| RunError::Internal(format!("serializing retention state: {err}")))?;
    write_file_atomic(&dir.join(RETENTION_STATE_FILE), json.as_bytes(), RETENTION_STATE_FILE)
}

fn write_retention_root_marker(dir: &Path, record: &RetentionRootRecord) -> Result<(), RunError> {
    let roots_dir = dir.join(RETENTION_ROOTS_DIR);
    std::fs::create_dir_all(&roots_dir)
        .map_err(|err| RunError::Internal(format!("creating {RETENTION_ROOTS_DIR}/: {err}")))?;
    let json = serde_json::to_string_pretty(record)
        .map_err(|err| RunError::Internal(format!("serializing retention root marker: {err}")))?;
    write_file_atomic(&retention_root_path(dir, record), json.as_bytes(), RETENTION_ROOTS_DIR)
}

fn remove_retention_roots(dir: &Path, plan: &RetentionPlan) -> Result<(), RunError> {
    for action in plan.actions.iter().filter(|action| action.kind != RetentionRootActionKind::CreateRoot) {
        let path = retention_root_path(dir, &action.record);
        match std::fs::remove_file(&path) {
            Ok(()) => {}
            Err(err) if err.kind() == ErrorKind::NotFound => {}
            Err(err) => {
                return Err(RunError::Internal(format!("removing stale retention root {}: {err}", path.display())));
            }
        }
    }
    Ok(())
}

fn retention_root_path(dir: &Path, record: &RetentionRootRecord) -> PathBuf {
    dir.join(RETENTION_ROOTS_DIR).join(format!("{}.json", record.root_id))
}

fn retention_status_text(input_name: &str, plan: &RetentionPlan) -> String {
    let Some(status) = plan.statuses.iter().find(|status| status.input_name == input_name) else {
        return "unpinned (undeclared input)".into();
    };
    render_retention_status(status)
}

fn render_retention_status(status: &RetentionInputStatus) -> String {
    match status.state {
        RetentionInputState::Pinned => format!(
            "pinned (root {}, lock {})",
            status.root_id.as_deref().unwrap_or("<missing>"),
            status.lock_digest.as_deref().unwrap_or("<unknown>")
        ),
        RetentionInputState::MissingRoot => "missing-root".into(),
        RetentionInputState::StaleRoot => "stale-root".into(),
        RetentionInputState::GcEligible => "gc-eligible".into(),
        RetentionInputState::Unpinned => "unpinned".into(),
    }
}

fn write_file_atomic(path: &Path, content: &[u8], label: &str) -> Result<(), RunError> {
    let tmp_path = atomic_tmp_path(path);
    std::fs::write(&tmp_path, content)
        .map_err(|err| RunError::Internal(format!("writing temporary {label}: {err}")))?;
    std::fs::rename(&tmp_path, path).map_err(|err| RunError::Internal(format!("committing {label}: {err}")))?;
    Ok(())
}

fn atomic_tmp_path(path: &Path) -> PathBuf {
    let file_name = path.file_name().and_then(|name| name.to_str()).unwrap_or("project-file");
    path.with_file_name(format!("{file_name}.{ATOMIC_TMP_SUFFIX}"))
}

fn reject_conflicting_legacy_project_files(dir: &Path) -> Result<(), RunError> {
    let conflicts = [
        (MANIFEST_FILE, LEGACY_MANIFEST_FILE),
        (LOCK_FILE, LEGACY_LOCK_FILE),
        (INPUTS_DIR, LEGACY_INPUTS_DIR),
    ]
    .into_iter()
    .filter_map(|(current, legacy)| {
        let has_current = dir.join(current).exists();
        let has_legacy = dir.join(legacy).exists();
        if has_current && has_legacy {
            Some(format!("{legacy} conflicts with {current}"))
        } else {
            None
        }
    })
    .collect::<Vec<_>>();

    if conflicts.is_empty() {
        Ok(())
    } else {
        Err(RunError::Internal(format!(
            "conflicting legacy Crunch project files: {}. Move or migrate the legacy files before running mantle.",
            conflicts.join(", ")
        )))
    }
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

fn print_refresh_outcomes(outcomes: &[RefreshOutcome]) {
    for outcome in outcomes {
        match outcome {
            RefreshOutcome::Updated(resolved) => eprintln!("updated: {}", resolved.name),
            RefreshOutcome::Unchanged { name } => eprintln!("unchanged: {name}"),
            RefreshOutcome::Frozen { name } => eprintln!("frozen (skipped): {name}"),
            RefreshOutcome::Skipped { name, reason } => eprintln!("skipped: {name}: {reason}"),
            RefreshOutcome::NetworkRequired { name, reason } => eprintln!("network-required: {name}: {reason}"),
            RefreshOutcome::Failed { name, reason } => eprintln!("failed: {name}: {reason}"),
        }
    }
}

fn collect_outcome_failures(outcomes: &[RefreshOutcome]) -> Vec<RefreshFailure> {
    outcomes
        .iter()
        .filter_map(|outcome| match outcome {
            RefreshOutcome::Failed { name, reason }
            | RefreshOutcome::NetworkRequired { name, reason }
            | RefreshOutcome::Skipped { name, reason } => Some(RefreshFailure {
                name: name.clone(),
                reason: reason.clone(),
            }),
            _ => None,
        })
        .collect()
}

fn print_patch_failures(failures: &[RefreshFailure]) {
    for failure in failures {
        eprintln!("failed: patch {}: {}", failure.name, failure.reason);
    }
}
