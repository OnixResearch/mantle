use std::ffi::OsString;
use std::path::Path;
use std::path::PathBuf;

use crunch_build::signing;
use crunch_pipeline::BuildConfig;
use crunch_pipeline::HermeticityAuditEvent;
use crunch_pipeline::HermeticityMode;
use crunch_pipeline::PipelineResult;
use crunch_pipeline::drv_key_for;
use crunch_pipeline::label_for_key;
use crunch_pipeline::parse_drv_key;
use crunch_store::GcRootSource;
use nix_compat::store_path::StorePath;

use crate::build_failure::build_failure_envelopes;
use crate::build_failure::render_human_failure_summary;
use crate::build_failure::should_write_failure_log;
use crate::build_log::DiagnosticPersistenceFailure;
use crate::build_log::log_file_path;
use crate::build_log::write_log_file;
use crate::build_report::render_build_json_report;
use crate::errors::RunError;

const MAX_HUMAN_PRIORITY_ROWS: usize = 64;
const PRIORITY_DIGEST_PREFIX_BYTES: usize = 12;
const PRIORITY_SUMMARY_EXTRA_LINES: usize = 2;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BuildOutputMode {
    Human,
    Json,
}

impl BuildOutputMode {
    fn is_human(self) -> bool {
        matches!(self, Self::Human)
    }

    fn is_json(self) -> bool {
        matches!(self, Self::Json)
    }
}

#[allow(clippy::too_many_arguments)]
pub fn cmd_build(
    file: &Path,
    import_paths: &[OsString],
    output_dir: &Path,
    state_dir: &Path,
    store_dir: &str,
    verbose: bool,
    fix: bool,
    max_jobs: u32,
    substituter_urls: &[String],
    signing_key_path: Option<&Path>,
    trusted_public_keys: Option<&[nix_compat::narinfo::VerifyingKey]>,
    trust_unsigned: bool,
    hermeticity_mode: HermeticityMode,
    output_mode: BuildOutputMode,
) -> Result<(), RunError> {
    cmd_build_with_source_fetch_overrides(
        file,
        import_paths,
        output_dir,
        state_dir,
        store_dir,
        verbose,
        fix,
        max_jobs,
        substituter_urls,
        signing_key_path,
        trusted_public_keys,
        trust_unsigned,
        hermeticity_mode,
        output_mode,
        Vec::new(),
        Vec::new(),
    )
}

#[allow(clippy::too_many_arguments)]
pub fn cmd_build_with_source_fetch_overrides(
    file: &Path,
    import_paths: &[OsString],
    output_dir: &Path,
    state_dir: &Path,
    store_dir: &str,
    verbose: bool,
    fix: bool,
    max_jobs: u32,
    substituter_urls: &[String],
    signing_key_path: Option<&Path>,
    trusted_public_keys: Option<&[nix_compat::narinfo::VerifyingKey]>,
    trust_unsigned: bool,
    hermeticity_mode: HermeticityMode,
    output_mode: BuildOutputMode,
    source_fetch_overrides: Vec<crunch_build::FetchSourceOverride>,
    base_state_dirs: Vec<PathBuf>,
) -> Result<(), RunError> {
    let keypair = load_or_generate_signing_keypair(signing_key_path, state_dir, output_mode.is_human())?;
    let configured_trusted_keys = load_configured_trusted_public_keys(trusted_public_keys, state_dir)?;
    let trusted_keys = signing::build_trusted_keys(&keypair, configured_trusted_keys.as_deref());

    let config = BuildConfig {
        file: file.to_path_buf(),
        import_paths: import_paths.to_vec(),
        output_dir: output_dir.to_path_buf(),
        state_dir: state_dir.to_path_buf(),
        store_dir: store_dir.to_string(),
        verbose,
        max_jobs,
        scheduling_policy: crunch_pipeline::SchedulingPolicy::default(),
        substituter_urls: substituter_urls.to_vec(),
        hermeticity_mode,
        keypair,
        trusted_keys,
        trust_unsigned,
        root_retention_source: Some(GcRootSource::Build),
        source_fetch_overrides,
        remote_enabled: false,
        base_state_dirs,
    };

    let result = run_build(&config)?;
    report_build_result(&config, &result, fix, output_mode)
}

pub fn run_build(config: &BuildConfig) -> Result<PipelineResult, RunError> {
    let rt = tokio::runtime::Runtime::new().map_err(|e| RunError::Internal(format!("tokio runtime: {e}")))?;
    rt.block_on(crunch_pipeline::build(config)).map_err(Into::into)
}

pub fn report_build_result(
    config: &BuildConfig,
    result: &PipelineResult,
    fix: bool,
    output_mode: BuildOutputMode,
) -> Result<(), RunError> {
    let logs_dir = log_dir();
    let mut diagnostic_persistence_failures = Vec::new();
    let log_dir_ready = match prepare_logs_dir(&logs_dir) {
        Ok(()) => true,
        Err(failure) => {
            diagnostic_persistence_failures.push(failure);
            false
        }
    };

    if log_dir_ready {
        diagnostic_persistence_failures.extend(write_success_logs(config, result, &logs_dir, output_mode));
    }
    if output_mode.is_human() {
        print_hermeticity_summary(result);
        print_action_result_summary(result);
        if config.verbose {
            print_priority_summary(result);
        }
        print_success_outputs(config, result);
    }

    if result.failed.is_empty() {
        if output_mode.is_human() {
            print_diagnostic_persistence_failures(&diagnostic_persistence_failures);
        }
        if output_mode.is_json() {
            print_json_report(config, result, &logs_dir, &diagnostic_persistence_failures)?;
        }
        return Ok(());
    }

    if log_dir_ready {
        diagnostic_persistence_failures.extend(write_failure_logs(config, result, &logs_dir));
    }
    if output_mode.is_json() {
        print_json_report(config, result, &logs_dir, &diagnostic_persistence_failures)?;
    }

    if let Some(single_mismatch) = maybe_single_fod_mismatch(config, result, fix, output_mode) {
        return single_mismatch;
    }

    if output_mode.is_human() {
        print_failed_builds(result, &config.store_dir, &logs_dir);
        print_diagnostic_persistence_failures(&diagnostic_persistence_failures);
    }

    Err(RunError::Reported(1))
}

fn maybe_single_fod_mismatch(
    config: &BuildConfig,
    result: &PipelineResult,
    fix: bool,
    output_mode: BuildOutputMode,
) -> Option<Result<(), RunError>> {
    if result.failed.len() != 1 {
        return None;
    }
    if result.fod_mismatches.len() != 1 {
        return None;
    }

    let failed = &result.failed[0];
    let mismatch = &result.fod_mismatches[0];
    let drv_path = parse_drv_key(&config.store_dir, &failed.drv_key)?;
    let label = label_for_key(result, &failed.drv_key).unwrap_or(drv_path.name());

    Some(crate::fix::handle_fod_mismatch(
        mismatch,
        &drv_path,
        label,
        &config.file,
        fix,
        output_mode.is_human(),
    ))
}

fn write_success_logs(
    config: &BuildConfig,
    result: &PipelineResult,
    logs_dir: &Path,
    output_mode: BuildOutputMode,
) -> Vec<DiagnosticPersistenceFailure> {
    let mut failures = Vec::new();
    for outcome in &result.outcomes {
        let drv_key = drv_key_for(&config.store_dir, &outcome.drv_path);
        let label = label_for_key(result, &drv_key).unwrap_or(outcome.drv_path.name());

        if let Some(log) = &outcome.log {
            if let Err(failure) = write_log(logs_dir, &outcome.drv_path, label, true, log) {
                failures.push(failure);
            }
            if config.verbose && output_mode.is_human() {
                eprintln!("--- build log: {label} ---");
                eprintln!("{log}");
                eprintln!("--- end log ---");
            }
        } else if !outcome.cached
            && let Err(failure) = write_log(logs_dir, &outcome.drv_path, label, true, "(no output captured)")
        {
            failures.push(failure);
        }
    }
    failures
}

fn print_success_outputs(config: &BuildConfig, result: &PipelineResult) {
    let output_dir_str = config.output_dir.to_str().unwrap_or(&config.store_dir);

    for outcome in &result.outcomes {
        let multi = outcome.outputs.len() > 1;
        let mut outputs: Vec<_> = outcome.outputs.iter().collect();
        outputs.sort_by(|left, right| left.0.cmp(right.0));
        for (output_name, path_info) in outputs {
            let path = path_info.store_path.to_absolute_path_with_prefix(output_dir_str);
            let suffix = format_output_suffix(outcome, output_name, multi);
            println!("{path}{suffix}");
        }
    }
}

fn format_output_suffix(outcome: &crunch_build::BuildOutcome, output_name: &str, multi: bool) -> String {
    let mut parts = Vec::<String>::new();
    if multi && output_name != "out" {
        parts.push(output_name.to_string());
    }
    if outcome.cached {
        parts.push("cached".to_string());
    }
    if let Some(report) = outcome.substitutions.get(output_name) {
        parts.push(format!(
            "substitution={}, transferred_bytes={}, reused_bytes={}",
            report.mode.as_str(),
            report.transferred_bytes,
            report.reused_bytes
        ));
        if let Some(reason) = &report.fallback_reason {
            parts.push(format!("fallback_reason={reason}"));
        }
    }
    if parts.is_empty() {
        return String::new();
    }
    format!(" ({})", parts.join(", "))
}

fn print_json_report(
    config: &BuildConfig,
    result: &PipelineResult,
    logs_dir: &Path,
    diagnostic_persistence_failures: &[DiagnosticPersistenceFailure],
) -> Result<(), RunError> {
    let report = render_build_json_report(config, result, logs_dir, diagnostic_persistence_failures)
        .map_err(|e| RunError::Internal(format!("serializing build report: {e}")))?;
    println!("{report}");
    Ok(())
}

fn print_hermeticity_summary(result: &PipelineResult) {
    for line in format_hermeticity_summary(result.hermeticity_mode, &result.hermeticity_audit_events) {
        eprintln!("{line}");
    }
}

fn print_priority_summary(result: &PipelineResult) {
    for line in format_priority_summary(&result.priority_decisions) {
        eprintln!("{line}");
    }
}

fn print_action_result_summary(result: &PipelineResult) {
    for line in format_action_result_summary(&result.action_result_reports) {
        eprintln!("{line}");
    }
}

fn format_action_result_summary(reports: &[crunch_build::ActionResultRuntimeReport]) -> Vec<String> {
    let mut lines = Vec::new();
    for report in reports {
        let selected = report.selected_result_ref.as_deref().unwrap_or("none");
        let source = report.selected_source_class.as_deref().unwrap_or("none");
        let source_id = report.selected_source_id.as_deref().unwrap_or("none");
        let conflict = report.conflict_class.as_deref().unwrap_or("none");
        let trust_basis = if report.trust_basis.is_empty() {
            "none".to_string()
        } else {
            report.trust_basis.join(",")
        };
        lines.push(format!(
            "shared-action-result: phase={} disposition={} action={} selected={} source={} source_id={} trust={} conflict={} non_claims={}",
            report.phase,
            report.disposition,
            report.action_ref,
            selected,
            source,
            source_id,
            trust_basis,
            conflict,
            report.non_claims.join(",")
        ));
        for decision in &report.candidate_decisions {
            if decision.admitted {
                continue;
            }
            lines.push(format!(
                "shared-action-result-rejected: result={} source={} source_id={} diagnostics={}",
                decision.result_ref,
                decision.source_class,
                decision.source_id,
                decision.diagnostics.join(",")
            ));
        }
    }
    lines
}

fn format_priority_summary(decisions: &[crunch_pipeline::PriorityDecisionEvidence]) -> Vec<String> {
    if decisions.is_empty() {
        return Vec::new();
    }
    let rendered_count = decisions.len().min(MAX_HUMAN_PRIORITY_ROWS);
    let mut lines = Vec::with_capacity(rendered_count.saturating_add(PRIORITY_SUMMARY_EXTRA_LINES));
    lines.push(format!(
        "scheduler priority: decisions={} scope={} non-claims={}",
        decisions.len(),
        decisions[0].claim_scope,
        decisions[0].non_claims.join(",")
    ));
    for decision in decisions.iter().take(rendered_count) {
        let selected = decision
            .selected_goal_key_blake3
            .get(..PRIORITY_DIGEST_PREFIX_BYTES)
            .unwrap_or(&decision.selected_goal_key_blake3);
        let snapshot = decision
            .candidate_snapshot_digest_blake3
            .get(..PRIORITY_DIGEST_PREFIX_BYTES)
            .unwrap_or(&decision.candidate_snapshot_digest_blake3);
        lines.push(format!(
            "  - epoch={} selected={} snapshot={} policy={} reason={} operator={} age={}/{} graph={}:path={},work={},roots={} resource={} locality={} transfer={} history={}",
            decision.scheduling_epoch,
            selected,
            snapshot,
            decision.policy_id,
            decision.selection_reason.as_str(),
            decision.operator_policy_class.as_str(),
            decision.starvation_class.as_str(),
            decision.age_epochs,
            decision.known_graph_basis,
            decision.known_critical_path_nodes,
            decision.known_critical_path_work_units,
            decision.blocked_root_count,
            decision.resource_fit_class.as_str(),
            decision.content_locality_class.as_str(),
            decision.transfer_cost_class.as_str(),
            decision.history_basis.as_str(),
        ));
    }
    if rendered_count < decisions.len() {
        lines.push(format!(
            "  - omitted={} (machine report retains all bounded decisions)",
            decisions.len().saturating_sub(rendered_count)
        ));
    }
    debug_assert!(lines.len() <= MAX_HUMAN_PRIORITY_ROWS.saturating_add(PRIORITY_SUMMARY_EXTRA_LINES));
    debug_assert!(lines.iter().all(|line| !line.contains("token=")));
    lines
}

fn format_hermeticity_summary(mode: HermeticityMode, events: &[HermeticityAuditEvent]) -> Vec<String> {
    let mut lines = Vec::with_capacity(events.len().saturating_add(1));
    if events.is_empty() {
        lines.push(format!("hermeticity: {mode} (no degraded facts)"));
        return lines;
    }

    lines.push(format!(
        "WARNING: degraded hermeticity: {mode} ({} {})",
        events.len(),
        audit_event_label(events.len())
    ));
    for event in events {
        lines.push(format!("  - {}: {}", event.kind, event.detail));
    }
    lines
}

fn audit_event_label(event_count: usize) -> &'static str {
    if event_count == 1 {
        return "audit event";
    }
    "audit events"
}

fn write_failure_logs(
    config: &BuildConfig,
    result: &PipelineResult,
    logs_dir: &Path,
) -> Vec<DiagnosticPersistenceFailure> {
    let mut failures = Vec::new();
    for failed in &result.failed {
        if !should_write_failure_log(&failed.error) {
            continue;
        }
        let Some(drv_path) = parse_drv_key(&config.store_dir, &failed.drv_key) else {
            continue;
        };
        let label = label_for_key(result, &failed.drv_key)
            .map(str::to_owned)
            .unwrap_or_else(|| drv_path.name().to_string());
        if let Err(failure) = write_log(logs_dir, &drv_path, &label, false, &failed.error) {
            failures.push(failure);
        }
    }
    failures
}

fn print_failed_builds(result: &PipelineResult, store_dir: &str, logs_dir: &Path) {
    for envelope in build_failure_envelopes(result, store_dir, logs_dir) {
        for line in render_human_failure_summary(&envelope) {
            eprintln!("{line}");
        }
    }
}

fn prepare_logs_dir(logs_dir: &Path) -> Result<(), DiagnosticPersistenceFailure> {
    std::fs::create_dir_all(logs_dir).map_err(|error| DiagnosticPersistenceFailure::create_log_dir(logs_dir, &error))
}

fn print_diagnostic_persistence_failures(failures: &[DiagnosticPersistenceFailure]) {
    for failure in failures {
        eprintln!("WARNING: diagnostic persistence failed");
        eprintln!("  operation: {}", failure.operation);
        eprintln!("  artifact: {}", failure.artifact);
        if let Some(label) = &failure.label {
            eprintln!("  label: {label}");
        }
        eprintln!("  attempted_path: {}", failure.attempted_path);
        eprintln!("  error: {}", failure.error);
    }
}

pub fn write_log(
    log_dir: &Path,
    drv_path: &StorePath<String>,
    label: &str,
    success: bool,
    body: &str,
) -> Result<PathBuf, DiagnosticPersistenceFailure> {
    let attempted_path = log_file_path(log_dir, drv_path);
    write_log_file(log_dir, drv_path, label, success, body)
        .map_err(|error| DiagnosticPersistenceFailure::write_build_log(label, &attempted_path, &error))
}

pub fn state_dir() -> PathBuf {
    std::env::var("CRUNCH_STATE_DIR").map(PathBuf::from).unwrap_or_else(|_| {
        let state = std::env::var("XDG_STATE_HOME").map(PathBuf::from).unwrap_or_else(|_| {
            let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
            PathBuf::from(home).join(".local/state")
        });
        state.join("crunch")
    })
}

pub fn log_dir() -> PathBuf {
    std::env::var("CRUNCH_LOG_DIR").map(PathBuf::from).unwrap_or_else(|_| state_dir().join("logs"))
}

/// Load a signing keypair from the given path or the default config location,
/// but never generate a new key.
pub fn load_existing_signing_keypair(
    explicit_path: Option<&Path>,
    state_dir: &Path,
) -> Result<(signing::KeyPair, PathBuf), RunError> {
    if let Some(path) = explicit_path {
        let keypair = load_signing_keypair_from_path(path)?;
        return Ok((keypair, path.to_path_buf()));
    }

    let default_path = default_signing_key_path(state_dir);
    if default_path.exists() {
        let keypair = load_signing_keypair_from_path(&default_path)?;
        return Ok((keypair, default_path));
    }

    Err(RunError::Internal(format!("no signing key found at {}", default_path.display())))
}

/// Load a signing keypair from the given path, the default config location,
/// or generate one automatically.
pub fn load_or_generate_signing_keypair(
    explicit_path: Option<&Path>,
    state_dir: &Path,
    emit_human: bool,
) -> Result<signing::KeyPair, RunError> {
    match load_existing_signing_keypair(explicit_path, state_dir) {
        Ok((keypair, _path)) => return Ok(keypair),
        Err(err) => {
            if explicit_path.is_some() {
                return Err(err);
            }
        }
    }

    // 3. Auto-generate.
    let config_dir = config_dir_or(state_dir);
    let default_path = default_signing_key_path(state_dir);
    let (keypair, line): (signing::KeyPair, String) = signing::generate_keypair();
    std::fs::create_dir_all(&config_dir)
        .map_err(|e| RunError::Internal(format!("creating config dir {}: {e}", config_dir.display())))?;

    // Write with 0600 permissions.
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(&default_path)
            .and_then(|mut f| {
                use std::io::Write;
                f.write_all(line.as_bytes())?;
                f.write_all(b"\n")?;
                Ok(())
            })
            .map_err(|e| RunError::Internal(format!("writing signing key {}: {e}", default_path.display())))?;
    }
    #[cfg(not(unix))]
    {
        std::fs::write(&default_path, format!("{line}\n"))
            .map_err(|e| RunError::Internal(format!("writing signing key {}: {e}", default_path.display())))?;
    }

    if emit_human {
        eprintln!("Generated signing key: {} ({})", keypair.verifying_key.name(), default_path.display());
    }
    Ok(keypair)
}

fn load_signing_keypair_from_path(path: &Path) -> Result<signing::KeyPair, RunError> {
    let contents = std::fs::read_to_string(path)
        .map_err(|e| RunError::Internal(format!("reading signing key {}: {e}", path.display())))?;
    signing::load_keypair(&contents)
        .map_err(|e| RunError::Internal(format!("parsing signing key {}: {e}", path.display())))
}

fn default_signing_key_path(state_dir: &Path) -> PathBuf {
    config_dir_or(state_dir).join("signing-key")
}

pub fn load_configured_trusted_public_keys(
    explicit_keys: Option<&[nix_compat::narinfo::VerifyingKey]>,
    state_dir: &Path,
) -> Result<Option<Vec<nix_compat::narinfo::VerifyingKey>>, RunError> {
    if let Some(keys) = explicit_keys {
        return Ok(Some(keys.to_vec()));
    }

    let config_dir = config_dir_or(state_dir);
    let default_path = config_dir.join("trusted-public-keys");
    if !default_path.exists() {
        return Ok(None);
    }

    let contents = std::fs::read_to_string(&default_path)
        .map_err(|e| RunError::Internal(format!("reading trusted public keys {}: {e}", default_path.display())))?;

    let mut parsed = Vec::new();
    for raw_line in contents.lines() {
        let line = raw_line.trim();
        if line.is_empty() {
            continue;
        }
        if line.starts_with('#') {
            continue;
        }
        for key_str in line.split(',') {
            let trimmed = key_str.trim();
            if trimmed.is_empty() {
                continue;
            }
            parsed.push(nix_compat::narinfo::VerifyingKey::parse(trimmed).map_err(|e| {
                RunError::Internal(format!(
                    "invalid trusted public key '{}' in {}: {e}",
                    trimmed,
                    default_path.display()
                ))
            })?);
        }
    }

    if parsed.is_empty() {
        return Ok(None);
    }

    Ok(Some(parsed))
}

fn config_dir_or(state_dir: &Path) -> PathBuf {
    std::env::var("CRUNCH_CONFIG_DIR").map(PathBuf::from).unwrap_or_else(|_| state_dir.to_path_buf())
}

pub fn build_import_paths(extra: &[PathBuf]) -> Result<Vec<OsString>, RunError> {
    let stdlib_dir =
        crunch_eval::stdlib::stdlib_import_path().map_err(|e| RunError::Internal(format!("stdlib: {e}")))?;
    let mut paths: Vec<OsString> = vec![stdlib_dir.into()];
    for path in extra {
        paths.push(path.into());
    }
    Ok(paths)
}

#[cfg(test)]
mod tests {
    use crunch_pipeline::HermeticityAuditKind;

    use super::*;

    const TEST_PRIORITY_GOAL: &str = "/private/priority-secret/root.drv";
    const TEST_PRIORITY_EPOCH: u32 = 1;
    const TEST_PRIORITY_PATH_NODES: u32 = 2;
    const TEST_PRIORITY_SUMMARY_LINE_COUNT: usize = 2;

    fn sample_action_result_report(disposition: &str) -> crunch_build::ActionResultRuntimeReport {
        crunch_build::ActionResultRuntimeReport {
            schema: "mantle-action-result-runtime-report-v1".to_string(),
            phase: "discovery".to_string(),
            action_ref: "action-b3:demo".to_string(),
            disposition: disposition.to_string(),
            selected_result_ref: Some("result-b3:demo".to_string()),
            selected_source_id: Some("cache.example.invalid".to_string()),
            selected_source_class: Some("http".to_string()),
            trust_basis: vec!["record-signature-verified:builder-key-1".to_string()],
            conflict_class: None,
            candidate_decisions: Vec::new(),
            publication_result_refs: Vec::new(),
            transfer: None,
            diagnostics: Vec::new(),
            non_claims: vec!["index-presence-is-not-output-trust".to_string()],
        }
    }

    fn sample_priority_decision() -> crunch_pipeline::PriorityDecisionEvidence {
        let policy = crunch_build::SchedulingPolicy::default();
        let ready = crunch_build::ReadyGoalFacts::ordinary(TEST_PRIORITY_GOAL.to_string(), 0);
        let pressures =
            std::collections::BTreeMap::from([(TEST_PRIORITY_GOAL.to_string(), crunch_build::KnownGraphPressure {
                known_critical_path_nodes: TEST_PRIORITY_PATH_NODES,
                known_critical_path_work_units: TEST_PRIORITY_PATH_NODES,
                blocked_root_count: TEST_PRIORITY_EPOCH,
                blocked_root_count_saturated: false,
            })]);
        let ranked = crunch_build::rank_ready_goals(&policy, TEST_PRIORITY_EPOCH, &[ready], &pressures).unwrap();
        crunch_build::priority_decision_evidence(
            &policy,
            TEST_PRIORITY_EPOCH,
            &ranked,
            crunch_build::HistoryBasis::StructuralFallbackMissing,
            None,
        )
        .unwrap()
    }

    #[test]
    fn format_action_result_summary_reports_selected_source_and_omits_empty_input() {
        let lines = format_action_result_summary(&[sample_action_result_report("reused")]);

        assert_eq!(lines.len(), 1);
        assert!(lines[0].contains("disposition=reused"));
        assert!(lines[0].contains("selected=result-b3:demo"));
        assert!(lines[0].contains("source=http"));
        assert!(lines[0].contains("record-signature-verified:builder-key-1"));
        assert!(lines[0].contains("index-presence-is-not-output-trust"));
        assert!(format_action_result_summary(&[]).is_empty());
    }

    #[test]
    fn format_priority_summary_is_bounded_and_redacted() {
        let decisions = vec![sample_priority_decision()];
        let lines = format_priority_summary(&decisions);
        let rendered = lines.join("\n");

        assert_eq!(lines.len(), TEST_PRIORITY_SUMMARY_LINE_COUNT);
        assert!(rendered.contains("scope=configured-known-fact-ordering"));
        assert!(rendered.contains("graph=known-graph:path=2"));
        assert!(!rendered.contains(TEST_PRIORITY_GOAL));
        assert!(!rendered.contains("priority-secret/root.drv"));
    }

    #[test]
    fn format_priority_summary_caps_human_rows_without_dropping_machine_evidence() {
        let decision_count = MAX_HUMAN_PRIORITY_ROWS.saturating_add(1);
        let decisions = vec![sample_priority_decision(); decision_count];

        let lines = format_priority_summary(&decisions);

        assert_eq!(lines.len(), MAX_HUMAN_PRIORITY_ROWS.saturating_add(PRIORITY_SUMMARY_EXTRA_LINES));
        assert!(lines.last().is_some_and(|line| line.contains("omitted=1")));
        assert_eq!(decisions.len(), decision_count);
    }

    #[test]
    fn format_priority_summary_omits_empty_evidence() {
        assert!(format_priority_summary(&[]).is_empty());
        assert_eq!(format_priority_summary(&[]).len(), 0);
    }

    #[test]
    fn format_hermeticity_summary_reports_clean_mode() {
        let lines = format_hermeticity_summary(HermeticityMode::Strict, &[]);
        assert_eq!(lines, vec!["hermeticity: strict (no degraded facts)".to_string()]);
    }

    #[test]
    fn format_hermeticity_summary_reports_explicit_impure_mode() {
        let events = vec![HermeticityAuditEvent::new(
            HermeticityAuditKind::ImpureModeSelected,
            "explicit --impure mode permits ambient host dependencies",
        )];
        let lines = format_hermeticity_summary(HermeticityMode::Impure, &events);
        assert_eq!(lines[0], "WARNING: degraded hermeticity: impure (1 audit event)");
        assert_eq!(lines[1], "  - impure-mode-selected: explicit --impure mode permits ambient host dependencies");
    }

    #[test]
    fn format_hermeticity_summary_reports_degraded_events() {
        let events = vec![HermeticityAuditEvent::new(
            HermeticityAuditKind::HostToolFallback,
            "using external bwrap",
        )];
        let lines = format_hermeticity_summary(HermeticityMode::Practical, &events);
        assert_eq!(lines[0], "WARNING: degraded hermeticity: practical (1 audit event)");
        assert_eq!(lines[1], "  - host-tool-fallback: using external bwrap");
    }

    #[test]
    fn audit_event_label_pluralizes_count() {
        assert_eq!(audit_event_label(1), "audit event");
        assert_eq!(audit_event_label(2), "audit events");
    }

    #[test]
    fn format_output_suffix_includes_full_substitution_report() {
        let outcome = crunch_build::BuildOutcome {
            drv_path: nix_compat::store_path::StorePath::from_name_and_digest_fixed("demo.drv", [1u8; 20]).unwrap(),
            outputs: std::collections::BTreeMap::new(),
            substitutions: std::collections::BTreeMap::from([(
                "out".to_string(),
                crunch_store::OutputSubstitutionReport {
                    mode: crunch_store::OutputSubstitutionMode::Full,
                    transferred_bytes: 55,
                    reused_bytes: 0,
                    metadata_reused: false,
                    fallback_reason: Some("stream_application_failed".to_string()),
                },
            )]),
            cached: true,
            log: None,
        };

        let suffix = format_output_suffix(&outcome, "out", false);
        assert_eq!(
            suffix,
            " (cached, substitution=full, transferred_bytes=55, reused_bytes=0, fallback_reason=stream_application_failed)"
        );
    }

    #[test]
    fn format_output_suffix_includes_delta_substitution_report_without_fallback_reason() {
        let outcome = crunch_build::BuildOutcome {
            drv_path: nix_compat::store_path::StorePath::from_name_and_digest_fixed("demo.drv", [2u8; 20]).unwrap(),
            outputs: std::collections::BTreeMap::new(),
            substitutions: std::collections::BTreeMap::from([(
                "out".to_string(),
                crunch_store::OutputSubstitutionReport {
                    mode: crunch_store::OutputSubstitutionMode::Delta,
                    transferred_bytes: 12,
                    reused_bytes: 34,
                    metadata_reused: false,
                    fallback_reason: None,
                },
            )]),
            cached: true,
            log: None,
        };

        let suffix = format_output_suffix(&outcome, "out", false);
        assert_eq!(suffix, " (cached, substitution=delta, transferred_bytes=12, reused_bytes=34)");
    }
}
