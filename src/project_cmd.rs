//! CLI handlers for project management commands.
// machine-artifact-public: project.command-reports
//!
//! Each function handles one subcommand, delegating to crunch-project
//! for the real logic. This module owns argument parsing, file I/O,
//! and output formatting — nothing else.

use std::io::ErrorKind;
use std::io::Write;
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
use mantle_application_contract::ApplicationOutcome;
use mantle_application_contract::ProjectAuthority;
use mantle_application_contract::ProjectEffect;
use mantle_application_contract::ProjectEffectObservation;
use mantle_application_contract::ProjectEffectPlan;
use mantle_application_contract::ProjectEffectPort;
use mantle_application_contract::ProjectEffectStatus;
use mantle_application_contract::ProjectObservationKind;
use mantle_application_contract::ProjectOperation;
use mantle_application_contract::classify_project_effects;
use mantle_application_contract::project_effect_plan;

use crate::errors::RunError;
use crate::project_resolve::LiveResolver;

/// Project file names.
pub(crate) const MANIFEST_FILE: &str = "mantle-project.ncl";
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
const MANIFEST_TEMPLATE: &str = r#"# Mantle project manifest.
# See: lib/project.ncl for the contract definition.
{
  version = "1.0.0",
  inputs = [],
  patches = [],
}
"#;

struct ProjectFiles<'a> {
    dir: &'a Path,
    selected: &'a [String],
    no_network: bool,
}

impl ProjectEffectPort for ProjectFiles<'_> {
    type Manifest = ProjectManifest;
    type Lock = Lockfile;
    type Applied = crunch_project::ApplyResult;
    type Refresh = Vec<RefreshOutcome>;
    type Stale = crunch_project::StaleReport;
    type Retention = RetentionShellState;
    type Error = RunError;

    fn inspect_legacy(&mut self) -> Result<(), RunError> {
        reject_conflicting_legacy_project_files(self.dir)
    }

    fn inspect_init(&mut self) -> Result<(), RunError> {
        reject_conflicting_legacy_project_files(self.dir)?;
        if self.dir.join(MANIFEST_FILE).exists() {
            return Err(RunError::Internal(format!("{MANIFEST_FILE} already exists in {}", self.dir.display())));
        }
        Ok(())
    }
    fn inspect_manifest(&mut self) -> Result<ProjectManifest, RunError> {
        load_manifest(self.dir)
    }

    fn inspect_lock(&mut self) -> Result<Lockfile, RunError> {
        load_lockfile(self.dir)
    }

    fn inspect(&mut self) -> Result<(ProjectManifest, Lockfile), RunError> {
        reject_conflicting_legacy_project_files(self.dir)?;
        Ok((load_manifest(self.dir)?, load_lockfile(self.dir)?))
    }

    fn generated_inputs(&mut self) -> Option<String> {
        std::fs::read_to_string(self.dir.join(INPUTS_FILE)).ok()
    }

    fn retention(&mut self) -> RetentionShellState {
        load_retention_shell_state(self.dir)
    }

    fn write_manifest(&mut self) -> Result<(), RunError> {
        std::fs::write(self.dir.join(MANIFEST_FILE), MANIFEST_TEMPLATE)
            .map_err(|e| RunError::Internal(format!("writing {MANIFEST_FILE}: {e}")))
    }

    fn write_init_lock(&mut self) -> Result<(), RunError> {
        let lock_json = Lockfile::new().to_json().map_err(|e| RunError::Internal(format!("serializing lock: {e}")))?;
        std::fs::write(self.dir.join(LOCK_FILE), lock_json)
            .map_err(|e| RunError::Internal(format!("writing {LOCK_FILE}: {e}")))
    }

    fn write_init_inputs(&mut self) -> Result<(), RunError> {
        std::fs::create_dir_all(self.dir.join(INPUTS_DIR))
            .map_err(|e| RunError::Internal(format!("creating {INPUTS_DIR}/: {e}")))?;
        std::fs::write(self.dir.join(INPUTS_FILE), generate_inputs_ncl(Lockfile::new()))
            .map_err(|e| RunError::Internal(format!("writing {INPUTS_FILE}: {e}")))
    }

    fn write_init_retention(&mut self) -> Result<(), RunError> {
        write_retention_state_records(self.dir, Vec::new())
    }

    fn gitignore(&mut self) -> Result<(), RunError> {
        add_gitignore_entry(self.dir)
    }

    fn readback(&mut self) -> Result<bool, RunError> {
        let expected_lock =
            Lockfile::new().to_json().map_err(|e| RunError::Internal(format!("serializing lock: {e}")))?;
        let expected_inputs = generate_inputs_ncl(Lockfile::new());
        let expected_retention = ProjectRetentionState {
            schema: crunch_project::RETENTION_STATE_SCHEMA.into(),
            records: Vec::new(),
        };
        let read = |name| std::fs::read_to_string(self.dir.join(name)).ok();
        let gitignore = read(".gitignore").unwrap_or_default();
        Ok(read(MANIFEST_FILE).as_deref() == Some(MANIFEST_TEMPLATE)
            && read(LOCK_FILE).as_deref() == Some(expected_lock.as_str())
            && read(INPUTS_FILE).as_deref() == Some(expected_inputs.as_str())
            && read(RETENTION_STATE_FILE)
                .and_then(|text| serde_json::from_str::<ProjectRetentionState>(&text).ok())
                .as_ref()
                == Some(&expected_retention)
            && gitignore.lines().any(|line| line.trim() == GITIGNORE_ENTRY))
    }
    fn apply_refresh(
        &mut self,
        manifest: &ProjectManifest,
        lock: &Lockfile,
        outcomes: &Vec<RefreshOutcome>,
    ) -> crunch_project::ApplyResult {
        let resolver = LiveResolver::new(self.dir);
        apply_outcomes(manifest, lock, outcomes, &resolver)
    }

    fn refresh(&mut self, manifest: &ProjectManifest, lock: &Lockfile) -> Vec<RefreshOutcome> {
        let resolver = LiveResolver::new(self.dir);
        refresh_inputs_with_options(manifest, lock, self.selected, &resolver, self.no_network)
    }

    fn stale(&mut self, manifest: &ProjectManifest, lock: &Lockfile) -> crunch_project::StaleReport {
        let resolver = LiveResolver::new(self.dir);
        list_stale_with_options(manifest, lock, &resolver, self.no_network)
    }

    fn write_lock(&mut self, lock: &Lockfile) -> Result<(), RunError> {
        write_lockfile(self.dir, lock)
    }

    fn write_inputs(&mut self, lock: &Lockfile) -> Result<(), RunError> {
        write_inputs_ncl(self.dir, lock)
    }

    fn write_retention(&mut self, manifest: &ProjectManifest, lock: &Lockfile) -> Result<(), RunError> {
        write_retention_plan(self.dir, manifest, lock)
    }
}

struct ProjectRun {
    plan: ProjectEffectPlan,
    observed: Vec<ProjectEffectObservation>,
}

impl ProjectRun {
    fn new(operation: ProjectOperation, network_allowed: bool, process_allowed: bool) -> Self {
        let plan = project_effect_plan(operation, network_allowed, process_allowed);
        let observed = Vec::with_capacity(plan.steps.len());
        Self { plan, observed }
    }
    fn check_next(
        &self,
        effect: ProjectEffect,
        authority: ProjectAuthority,
        kind: ProjectObservationKind,
    ) -> Result<(), RunError> {
        let step = self
            .plan
            .steps
            .get(self.observed.len())
            .ok_or_else(|| RunError::Internal("project effect exceeded planned calls".into()))?;
        if (step.effect, step.authority, step.expected_observation) != (effect, authority, kind) {
            return Err(RunError::Internal(format!("project effect was not authorized: {effect:?}")));
        }
        Ok(())
    }

    fn record(
        &mut self,
        effect: ProjectEffect,
        authority: ProjectAuthority,
        kind: ProjectObservationKind,
        status: ProjectEffectStatus,
    ) -> Result<(), RunError> {
        self.record_items(effect, authority, kind, status, u32::from(status != ProjectEffectStatus::Skipped))
    }

    fn record_items(
        &mut self,
        effect: ProjectEffect,
        authority: ProjectAuthority,
        kind: ProjectObservationKind,
        status: ProjectEffectStatus,
        items: u32,
    ) -> Result<(), RunError> {
        self.check_next(effect, authority, kind)?;
        self.observed.push(ProjectEffectObservation {
            effect,
            authority,
            kind,
            items,
            readback_matches: None,
            calls: u8::from(status != ProjectEffectStatus::Skipped),
            target: effect.target(),
            status,
        });
        Ok(())
    }

    fn call<T>(
        &mut self,
        effect: ProjectEffect,
        authority: ProjectAuthority,
        kind: ProjectObservationKind,
        action: impl FnOnce() -> Result<T, RunError>,
    ) -> Result<T, RunError> {
        self.check_next(effect, authority, kind)?;
        let result = action();
        let status = if result.is_ok() {
            ProjectEffectStatus::Succeeded
        } else {
            ProjectEffectStatus::Failed
        };
        self.record(effect, authority, kind, status)?;
        if result.is_err() {
            let _ = self.classify()?;
        }
        result
    }

    fn classify(&mut self) -> Result<ApplicationOutcome, RunError> {
        while let Some(step) = self.plan.steps.get(self.observed.len()) {
            self.record(step.effect, step.authority, step.expected_observation, ProjectEffectStatus::Skipped)?;
        }
        match classify_project_effects(&self.plan, &self.observed) {
            outcome @ (ApplicationOutcome::Completed | ApplicationOutcome::Failed { .. }) => Ok(outcome),
            other => Err(RunError::Internal(format!("project effect observations were inconsistent: {other:?}"))),
        }
    }
}

fn admit_project_resolution(run: &mut ProjectRun, requested_count: usize, command: &str) -> Result<(), RunError> {
    let Ok(requested_count) = u32::try_from(requested_count) else {
        let _ = run.classify()?;
        return Err(RunError::Internal(format!("{command} input count overflowed u32")));
    };
    if let Err(blocker) = run.plan.admit_resolution_count(requested_count) {
        let _ = run.classify()?;
        return Err(RunError::Internal(format!(
            "{command} requested {requested_count} inputs beyond the {}-input limit ({blocker:?})",
            mantle_application_contract::MAX_PROJECT_LIFECYCLE_DECLARED_ENTRIES,
        )));
    }
    Ok(())
}

/// `mantle init` — scaffold a new project.
pub fn cmd_init(dir: &Path) -> Result<(), RunError> {
    let mut port = ProjectFiles {
        dir,
        selected: &[],
        no_network: true,
    };
    let mut run = ProjectRun::new(ProjectOperation::Init, false, false);
    run.call(ProjectEffect::Inspect, ProjectAuthority::ReadFiles, ProjectObservationKind::CallResult, || {
        port.inspect_init()
    })?;
    run.call(
        ProjectEffect::ManifestWrite,
        ProjectAuthority::WriteFiles,
        ProjectObservationKind::CallResult,
        || port.write_manifest(),
    )?;
    run.call(
        ProjectEffect::InitLockWrite,
        ProjectAuthority::WriteFiles,
        ProjectObservationKind::CallResult,
        || port.write_init_lock(),
    )?;
    run.call(
        ProjectEffect::InitInputsWrite,
        ProjectAuthority::WriteFiles,
        ProjectObservationKind::CallResult,
        || port.write_init_inputs(),
    )?;
    run.call(
        ProjectEffect::InitRetentionWrite,
        ProjectAuthority::WriteFiles,
        ProjectObservationKind::CallResult,
        || port.write_init_retention(),
    )?;
    run.call(ProjectEffect::Gitignore, ProjectAuthority::WriteFiles, ProjectObservationKind::CallResult, || {
        port.gitignore()
    })?;
    run.check_next(ProjectEffect::Readback, ProjectAuthority::ReadFiles, ProjectObservationKind::FileReadback)?;
    let readback = port.readback();
    let status = if matches!(readback, Ok(true)) {
        ProjectEffectStatus::Succeeded
    } else {
        ProjectEffectStatus::Failed
    };
    run.record(ProjectEffect::Readback, ProjectAuthority::ReadFiles, ProjectObservationKind::FileReadback, status)?;
    run.observed.last_mut().expect("readback observation recorded").readback_matches = readback.as_ref().ok().copied();
    let _ = run.classify()?;
    readback?;
    if status == ProjectEffectStatus::Failed {
        return Err(RunError::Internal(format!("project scaffold incomplete under {}", dir.display())));
    }
    eprintln!("Initialized Mantle project:");
    eprintln!("  {MANIFEST_FILE}  (edit this)");
    eprintln!("  {LOCK_FILE}      (machine-managed)");
    eprintln!("  {INPUTS_FILE}    (generated)");
    Ok(())
}

/// `mantle check` — validate project state.
pub fn cmd_check<Probes, Trust>(
    dir: &Path,
    output: ProjectCheckOutput,
    probes: Probes,
    trust: Trust,
) -> Result<(), RunError>
where
    Probes: Into<bool>,
    Trust: Into<bool>,
{
    let should_run_probes = probes.into();
    let should_validate_trust = trust.into();
    let mut run =
        ProjectRun::new(ProjectOperation::Check, should_run_probes || should_validate_trust, should_run_probes);
    let mut port = ProjectFiles {
        dir,
        selected: &[],
        no_network: !(should_run_probes || should_validate_trust),
    };
    assert_ne!(MANIFEST_FILE, LOCK_FILE, "project manifest and lockfile names must differ");
    assert_ne!(INPUTS_FILE, RETENTION_STATE_FILE, "generated inputs and retention state names must differ");
    // Legacy conflicts remain a hard blocker rather than a soundness report.
    if let Err(error) = port.inspect_legacy() {
        run.record(
            ProjectEffect::Inspect,
            ProjectAuthority::ReadFiles,
            ProjectObservationKind::CallResult,
            ProjectEffectStatus::Failed,
        )?;
        let _ = run.classify()?;
        return Err(error);
    }
    let manifest = match port.inspect_manifest() {
        Ok(manifest) => manifest,
        Err(error) => {
            run.record(
                ProjectEffect::Inspect,
                ProjectAuthority::ReadFiles,
                ProjectObservationKind::CallResult,
                ProjectEffectStatus::Failed,
            )?;
            return finish_check_report(
                &mut run,
                project_soundness_parse_error(
                    ProjectSoundnessClass::ManifestParseError,
                    ProjectSoundnessSubject::file(MANIFEST_FILE),
                    error.message().to_string(),
                ),
                output,
            );
        }
    };
    let lock = match port.inspect_lock() {
        Ok(lock) => lock,
        Err(error) => {
            run.record(
                ProjectEffect::Inspect,
                ProjectAuthority::ReadFiles,
                ProjectObservationKind::CallResult,
                ProjectEffectStatus::Failed,
            )?;
            return finish_check_report(
                &mut run,
                project_soundness_parse_error(
                    ProjectSoundnessClass::LockfileParseError,
                    ProjectSoundnessSubject::file(LOCK_FILE),
                    error.message().to_string(),
                ),
                output,
            );
        }
    };
    let generated_inputs = port.generated_inputs();
    let retention_state = port.retention();
    run.record(
        ProjectEffect::Inspect,
        ProjectAuthority::ReadFiles,
        ProjectObservationKind::CallResult,
        ProjectEffectStatus::Succeeded,
    )?;
    let retention_plan = plan_retention_roots(RetentionPlanRequest {
        manifest: manifest.clone(),
        lock: lock.clone(),
        existing_roots: retention_state.facts,
    });
    let mut supplemental_facts = retention_state.supplemental_facts;
    supplemental_facts.extend(retention_soundness_facts(&retention_plan));
    let soundness = check_project_soundness(ProjectSoundnessInput {
        manifest,
        lock,
        generated_inputs,
        supplemental_facts,
        mode: ProjectSoundnessMode::from_dynamic_requests(should_run_probes, should_validate_trust),
    });
    finish_check_report(&mut run, soundness, output)
}

fn finish_check_report(
    run: &mut ProjectRun,
    report: ProjectSoundnessReport,
    output: ProjectCheckOutput,
) -> Result<(), RunError> {
    let _observed_outcome = run.classify()?;
    match output {
        ProjectCheckOutput::Human => render_check_report_human(&report),
        ProjectCheckOutput::Json => render_check_report_json(&report)?,
    }
    if !report.valid {
        return Err(RunError::Reported(PROJECT_CHECK_FAILURE_EXIT_CODE));
    }
    Ok(())
}

fn render_check_report_json(report: &ProjectSoundnessReport) -> Result<(), RunError> {
    render_check_report_json_to(report, &mut std::io::stdout().lock())
}

fn render_check_report_json_to(report: &ProjectSoundnessReport, output: &mut impl Write) -> Result<(), RunError> {
    let rendered = serde_json::to_string_pretty(report)
        .map_err(|err| RunError::Internal(format!("rendering project soundness JSON: {err}")))?;
    writeln!(output, "{rendered}").map_err(|err| RunError::Internal(format!("writing project soundness JSON: {err}")))
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

/// `mantle show` — render resolved input state.
pub fn cmd_show(dir: &Path) -> Result<(), RunError> {
    let mut port = ProjectFiles {
        dir,
        selected: &[],
        no_network: true,
    };
    let mut run = ProjectRun::new(ProjectOperation::Show, false, false);
    assert_ne!(MANIFEST_FILE, LOCK_FILE, "project manifest and lockfile names must differ");
    assert_ne!(INPUTS_FILE, RETENTION_STATE_FILE, "generated inputs and retention state names must differ");
    let (manifest, lock) =
        run.call(ProjectEffect::Inspect, ProjectAuthority::ReadFiles, ProjectObservationKind::CallResult, || {
            port.inspect()
        })?;
    let retention_state = port.retention();
    let retention_plan = plan_retention_roots(RetentionPlanRequest {
        manifest: manifest.clone(),
        lock: lock.clone(),
        existing_roots: retention_state.facts,
    });
    let _ = run.classify()?;
    println!("Project: {} (schema {})", MANIFEST_FILE, manifest.version);
    println!();

    if lock.inputs.is_empty() {
        println!("No locked inputs.");
        return Ok(());
    }

    for (name, entry) in &lock.inputs {
        let manifest_input = manifest.inputs.iter().find(|i| i.name == *name);
        let is_frozen = manifest_input.is_some_and(|i| i.frozen);
        let frozen_tag = if is_frozen { " [frozen]" } else { "" };
        let kind_str = locked_kind_text(&entry.kind);

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

fn locked_kind_text(kind: &crunch_project::LockedKind) -> String {
    let text = match kind {
        crunch_project::LockedKind::File { url } => format!("file: {url}"),
        crunch_project::LockedKind::Tarball { url } => format!("tarball: {url}"),
        crunch_project::LockedKind::Git {
            repository,
            rev,
            ref_name,
        } => {
            let ref_str = ref_name.as_deref().map(|value| format!(" ({value})")).unwrap_or_default();
            format!("git: {repository} @ {rev}{ref_str}")
        }
        crunch_project::LockedKind::Darcs {
            repository,
            selector,
            context,
            weak_hash,
        } => {
            let identity = context.as_deref().or(weak_hash.as_deref()).unwrap_or("<unresolved>");
            format!("darcs: {repository} @ {} ({identity})", selector.identity_fragment())
        }
        crunch_project::LockedKind::Pijul {
            repository,
            selector,
            state,
            change,
        } => {
            let change_str = change.as_deref().map(|value| format!(" change {value}")).unwrap_or_default();
            format!("pijul: {repository} @ {} state {state}{change_str}", selector.identity_fragment())
        }
        crunch_project::LockedKind::Fossil {
            repository,
            selector,
            checkin,
        } => format!("fossil: {repository} @ {} check-in {checkin}", selector.identity_fragment()),
    };
    assert!(!text.is_empty(), "locked input kind rendering must not be empty");
    assert!(text.contains(':'), "locked input kind rendering must include its type prefix");
    text
}

/// `mantle refresh [names...]` — update inputs.
pub fn cmd_refresh(dir: &Path, selected: &[String], no_network: bool) -> Result<(), RunError> {
    let mut port = ProjectFiles {
        dir,
        selected,
        no_network,
    };
    let mut run = ProjectRun::new(ProjectOperation::Refresh, !no_network, true);
    assert_ne!(MANIFEST_FILE, LOCK_FILE, "project manifest and lockfile names must differ");
    assert_ne!(INPUTS_FILE, RETENTION_STATE_FILE, "generated inputs and retention state names must differ");
    let (manifest, lock) =
        run.call(ProjectEffect::Inspect, ProjectAuthority::ReadFiles, ProjectObservationKind::CallResult, || {
            port.inspect()
        })?;
    let requested_count = if selected.is_empty() {
        manifest.inputs.len()
    } else {
        selected.len()
    };
    admit_project_resolution(&mut run, requested_count, "refresh")?;
    let resolve_authority = ProjectAuthority::Resolve {
        network_allowed: !no_network,
        process_allowed: true,
    };
    run.check_next(ProjectEffect::Resolve, resolve_authority, ProjectObservationKind::CallResult)?;
    let outcomes = port.refresh(&manifest, &lock);
    let result = port.apply_refresh(&manifest, &lock, &outcomes);
    let input_failures = collect_outcome_failures(&outcomes);
    let Some(failure_count) = input_failures.len().checked_add(result.failures.len()) else {
        run.record(
            ProjectEffect::Resolve,
            resolve_authority,
            ProjectObservationKind::CallResult,
            ProjectEffectStatus::Failed,
        )?;
        let _ = run.classify()?;
        return Err(RunError::Internal("refresh failure count overflowed usize".to_string()));
    };
    let Ok(resolved_items) = u32::try_from(outcomes.len()) else {
        run.record(
            ProjectEffect::Resolve,
            resolve_authority,
            ProjectObservationKind::CallResult,
            ProjectEffectStatus::Failed,
        )?;
        let _ = run.classify()?;
        return Err(RunError::Internal("refresh outcome count overflowed u32".to_string()));
    };
    run.record_items(
        ProjectEffect::Resolve,
        resolve_authority,
        ProjectObservationKind::CallResult,
        if failure_count == 0 {
            ProjectEffectStatus::Succeeded
        } else {
            ProjectEffectStatus::Failed
        },
        resolved_items,
    )?;
    if resolved_items > mantle_application_contract::MAX_PROJECT_LIFECYCLE_DECLARED_ENTRIES {
        let _ = run.classify()?;
        return Err(RunError::Internal(format!("refresh observed {resolved_items} inputs beyond its plan limit")));
    }
    let problems = if result.has_changes {
        result.lock.clone().validate()
    } else {
        Vec::new()
    };
    if result.has_changes {
        run.call(ProjectEffect::LockWrite, ProjectAuthority::WriteFiles, ProjectObservationKind::CallResult, || {
            port.write_lock(&result.lock)
        })?;
        run.call(ProjectEffect::InputsWrite, ProjectAuthority::WriteFiles, ProjectObservationKind::CallResult, || {
            port.write_inputs(&result.lock)
        })?;
        run.call(
            ProjectEffect::RetentionWrite,
            ProjectAuthority::WriteFiles,
            ProjectObservationKind::CallResult,
            || port.write_retention(&manifest, &result.lock),
        )?;
    }
    let _ = run.classify()?;
    print_refresh_outcomes(&outcomes);
    print_patch_failures(&result.failures);
    for problem in &problems {
        eprintln!("lockfile warning: {problem}");
    }
    if result.has_changes {
        if result.inputs_changed > 0 {
            eprintln!("{} input(s) updated", result.inputs_changed);
        }
        if result.patches_changed {
            eprintln!("patch lock data updated");
        }
    } else if failure_count == 0 {
        eprintln!("all inputs up to date");
    }
    if failure_count > 0 {
        return Err(RunError::Internal(format!("refresh failed for {failure_count} item(s)")));
    }
    Ok(())
}

/// `mantle list-stale` — show which inputs would change.
pub fn cmd_list_stale(dir: &Path, no_network: bool) -> Result<(), RunError> {
    let mut port = ProjectFiles {
        dir,
        selected: &[],
        no_network,
    };
    let mut run = ProjectRun::new(ProjectOperation::ListStale, !no_network, true);
    assert_ne!(MANIFEST_FILE, LOCK_FILE, "project manifest and lockfile names must differ");
    assert_ne!(INPUTS_FILE, RETENTION_STATE_FILE, "generated inputs and retention state names must differ");
    let (manifest, lock) =
        run.call(ProjectEffect::Inspect, ProjectAuthority::ReadFiles, ProjectObservationKind::CallResult, || {
            port.inspect()
        })?;
    admit_project_resolution(&mut run, manifest.inputs.len(), "list-stale")?;
    let resolve_authority = ProjectAuthority::Resolve {
        network_allowed: !no_network,
        process_allowed: true,
    };
    run.check_next(ProjectEffect::Resolve, resolve_authority, ProjectObservationKind::CallResult)?;
    let outcome = port.stale(&manifest, &lock);
    let Some(blocker_count) = outcome.failed.len().checked_add(outcome.network_required.len()) else {
        run.record(
            ProjectEffect::Resolve,
            resolve_authority,
            ProjectObservationKind::CallResult,
            ProjectEffectStatus::Failed,
        )?;
        let _ = run.classify()?;
        return Err(RunError::Internal("stale-check blocker count overflowed usize".to_string()));
    };
    let observed_items = [
        outcome.stale.len(),
        outcome.unchanged.len(),
        outcome.skipped.len(),
        outcome.network_required.len(),
        outcome.failed.len(),
    ]
    .into_iter()
    .try_fold(0_u32, |count, len| u32::try_from(len).ok().and_then(|items| count.checked_add(items)));
    let Some(observed_items) = observed_items else {
        run.record(
            ProjectEffect::Resolve,
            resolve_authority,
            ProjectObservationKind::CallResult,
            ProjectEffectStatus::Failed,
        )?;
        let _ = run.classify()?;
        return Err(RunError::Internal("stale-check observation count overflowed u32".to_string()));
    };
    run.record_items(
        ProjectEffect::Resolve,
        resolve_authority,
        ProjectObservationKind::CallResult,
        if blocker_count == 0 {
            ProjectEffectStatus::Succeeded
        } else {
            ProjectEffectStatus::Failed
        },
        observed_items,
    )?;
    let _ = run.classify()?;
    for name in &outcome.stale {
        println!("{name}");
    }
    for name in &outcome.unchanged {
        eprintln!("unchanged: {name}");
    }
    for skipped in &outcome.skipped {
        eprintln!("skipped: {}: {}", skipped.name, skipped.reason);
    }
    for blocked in &outcome.network_required {
        eprintln!("network-required: {}: {}", blocked.name, blocked.reason);
    }
    for failure in &outcome.failed {
        eprintln!("failed: {}: {}", failure.name, failure.reason);
    }
    let is_current_state_clean = outcome.stale.is_empty() && outcome.failed.is_empty();
    let is_deferred_state_clean = outcome.skipped.is_empty() && outcome.network_required.is_empty();
    if is_current_state_clean && is_deferred_state_clean {
        println!("all inputs up to date");
    }
    if blocker_count > 0 {
        return Err(RunError::Internal(format!("stale check failed for {blocker_count} item(s)")));
    }
    Ok(())
}

/// `mantle upgrade` — migrate project files to current schema.
pub fn cmd_upgrade(dir: &Path) -> Result<(), RunError> {
    let mut port = ProjectFiles {
        dir,
        selected: &[],
        no_network: true,
    };
    let mut run = ProjectRun::new(ProjectOperation::Upgrade, false, false);
    let (lock, manifest) =
        run.call(ProjectEffect::Inspect, ProjectAuthority::ReadFiles, ProjectObservationKind::CallResult, || {
            port.inspect_legacy()?;
            let lock = port.inspect_lock()?;
            let manifest = if lock.version == SchemaVersion::CURRENT {
                None
            } else {
                Some(port.inspect_manifest()?)
            };
            Ok((lock, manifest))
        })?;
    if lock.version == SchemaVersion::CURRENT {
        let _ = run.classify()?;
        eprintln!("project files already at current version ({})", SchemaVersion::CURRENT);
        return Ok(());
    }
    let upgraded = upgrade_lockfile(lock.clone()).map_err(|e| RunError::Internal(format!("upgrade: {e}")))?;
    let manifest = manifest.expect("an outdated lock requires the manifest read");
    run.call(ProjectEffect::LockWrite, ProjectAuthority::WriteFiles, ProjectObservationKind::CallResult, || {
        port.write_lock(&upgraded)
    })?;
    run.call(ProjectEffect::InputsWrite, ProjectAuthority::WriteFiles, ProjectObservationKind::CallResult, || {
        port.write_inputs(&upgraded)
    })?;
    run.call(
        ProjectEffect::RetentionWrite,
        ProjectAuthority::WriteFiles,
        ProjectObservationKind::CallResult,
        || port.write_retention(&manifest, &upgraded),
    )?;
    let _ = run.classify()?;
    eprintln!("upgrading lockfile from {} to {}", lock.version, SchemaVersion::CURRENT);
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

    let evaluation_paths = vec![dir.as_os_str().to_owned()];
    crunch_eval::evaluate_and_deserialize(&path, &evaluation_paths)
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
            let is_root_present = retention_root_path(dir, &record).is_file();
            RetentionRootFact {
                record,
                root_exists: is_root_present,
            }
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
    assert_ne!(MANIFEST_FILE, LEGACY_MANIFEST_FILE, "current and legacy manifest names must differ");
    assert_ne!(LOCK_FILE, LEGACY_LOCK_FILE, "current and legacy lockfile names must differ");
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

fn add_gitignore_entry(dir: &Path) -> Result<(), RunError> {
    let gitignore = dir.join(".gitignore");
    let content = match std::fs::read_to_string(&gitignore) {
        Ok(content) => content,
        Err(error) if error.kind() == ErrorKind::NotFound => String::new(),
        Err(error) => return Err(RunError::Internal(format!("reading {}: {error}", gitignore.display()))),
    };
    if content.lines().any(|line| line.trim() == GITIGNORE_ENTRY) {
        return Ok(());
    }
    let mut new_content = content;
    if !new_content.is_empty() && !new_content.ends_with('\n') {
        new_content.push('\n');
    }
    new_content.push_str(GITIGNORE_ENTRY);
    new_content.push('\n');
    std::fs::write(&gitignore, &new_content)
        .map_err(|error| RunError::Internal(format!("writing {}: {error}", gitignore.display())))
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

#[cfg(test)]
mod lifecycle_effect_tests {
    use super::*;

    #[test]
    fn readback_rejects_mutated_scaffold_content_not_just_file_existence() {
        let dir = tempfile::tempdir().unwrap();
        let mut port = ProjectFiles {
            dir: dir.path(),
            selected: &[],
            no_network: true,
        };
        port.inspect_init().unwrap();
        port.write_manifest().unwrap();
        port.write_init_lock().unwrap();
        port.write_init_inputs().unwrap();
        port.write_init_retention().unwrap();
        port.gitignore().unwrap();
        assert!(port.readback().unwrap());
        std::fs::write(dir.path().join(INPUTS_FILE), "wrong inputs").unwrap();
        assert!(!port.readback().unwrap());
    }

    #[test]
    fn init_io_fault_never_reports_completion_after_partial_scaffold() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir(dir.path().join(".gitignore")).unwrap();
        let error = cmd_init(dir.path()).expect_err("gitignore read must reject a directory");
        assert!(error.to_string().contains(".gitignore"), "{error}");
        assert!(dir.path().join(MANIFEST_FILE).is_file());
        assert!(dir.path().join(LOCK_FILE).is_file());
    }

    #[test]
    fn existing_manifest_blocks_init_before_mutating_lock() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join(MANIFEST_FILE), "operator manifest").unwrap();
        let error = cmd_init(dir.path()).expect_err("existing project is a blocker");
        assert!(error.to_string().contains("already exists"), "{error}");
        assert!(!dir.path().join(LOCK_FILE).exists());
        assert_eq!(std::fs::read_to_string(dir.path().join(MANIFEST_FILE)).unwrap(), "operator manifest");
    }

    #[test]
    fn wrong_authority_is_rejected_before_port_mutates_the_project() {
        let dir = tempfile::tempdir().unwrap();
        let target = dir.path().join("should-not-exist");
        let mut run = ProjectRun::new(ProjectOperation::Init, false, false);
        let error = run
            .call(ProjectEffect::Inspect, ProjectAuthority::WriteFiles, ProjectObservationKind::CallResult, || {
                std::fs::write(&target, "unexpected write")
                    .map_err(|err| RunError::Internal(format!("writing sentinel: {err}")))
            })
            .expect_err("read authority cannot authorize a write");
        assert!(error.to_string().contains("not authorized"), "{error}");
        assert!(!target.exists());
    }

    struct FailingWriter;
    impl Write for FailingWriter {
        fn write(&mut self, _buf: &[u8]) -> std::io::Result<usize> {
            Err(std::io::Error::new(ErrorKind::BrokenPipe, "injected renderer failure"))
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    #[test]
    fn rendering_failure_is_not_a_successful_project_report() {
        let report = project_soundness_parse_error(
            ProjectSoundnessClass::ManifestParseError,
            ProjectSoundnessSubject::file(MANIFEST_FILE),
            "bad manifest".to_string(),
        );
        let error =
            render_check_report_json_to(&report, &mut FailingWriter).expect_err("failed output must not be reported");
        assert!(error.to_string().contains("injected renderer failure"), "{error}");
    }
}
