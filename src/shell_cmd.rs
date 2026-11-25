use std::collections::BTreeMap;
use std::ffi::OsString;
use std::path::Path;
use std::path::PathBuf;

use crunch_shell::ActivationPlan;
use crunch_shell::ExecMode;
use crunch_shell::ExecTarget;
use crunch_shell::HostEnv;
use crunch_shell::ShellSidecar;
use crunch_shell::compute_activation;

use crate::errors::RunError;
use crate::project_build;

const SIDECAR_FILENAME: &str = ".crunch-shell.json";
const SHELL_LEASE_ID_DOMAIN: &[u8] = b"mantle.shell.retention-lease.v1";

const _: () = {
    assert!(!SIDECAR_FILENAME.is_empty());
    assert!(SIDECAR_FILENAME.as_bytes()[0] == b'.');
    assert!(!SHELL_LEASE_ID_DOMAIN.is_empty());
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct HookExecutionMode {
    should_execute: bool,
}

impl From<bool> for HookExecutionMode {
    fn from(is_hook_disabled: bool) -> Self {
        Self {
            should_execute: !is_hook_disabled,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct StrictHookMode {
    is_strict: bool,
}

impl From<bool> for StrictHookMode {
    fn from(is_strict: bool) -> Self {
        Self { is_strict }
    }
}

pub(crate) struct ShellCommandRequest<'a> {
    pub(crate) name: Option<&'a str>,
    pub(crate) import_paths: &'a [PathBuf],
    pub(crate) jobs: Option<u32>,
    pub(crate) signing_key: Option<&'a Path>,
    pub(crate) is_unsigned_trusted: bool,
    pub(crate) command_argv: &'a [OsString],
    pub(crate) run_script: Option<&'a str>,
    pub(crate) with_paths: &'a [PathBuf],
    pub(crate) hook_execution: HookExecutionMode,
    pub(crate) strict_hooks: StrictHookMode,
    pub(crate) output_dir: &'a Path,
    pub(crate) state_dir: &'a Path,
    pub(crate) store_prefix: &'a str,
    pub(crate) backend: crunch_store::StoreBackend,
    pub(crate) is_verbose: bool,
    pub(crate) source_admission: Option<&'a std::sync::Arc<dyn crunch_store::StoreSourceAdmission>>,
}

/// Read the sidecar from a built shell output, snapshot the host env,
/// compute the activation plan, and exec into the result.
pub(crate) fn cmd_shell(request: ShellCommandRequest<'_>) -> Result<(), RunError> {
    debug_assert!(!request.store_prefix.is_empty());
    debug_assert!(request.store_prefix.starts_with('/'));
    if matches!(request.command_argv.first(), Some(first) if first.is_empty()) {
        return Err(RunError::Internal("--command requires a non-empty program name".into()));
    }

    let out_path = build_shell_target(BuildShellTargetRequest {
        name: request.name,
        import_paths: request.import_paths,
        jobs: request.jobs,
        signing_key: request.signing_key,
        is_unsigned_trusted: request.is_unsigned_trusted,
        output_dir: request.output_dir,
        state_dir: request.state_dir,
        store_prefix: request.store_prefix,
        backend: request.backend,
        is_verbose: request.is_verbose,
        source_admission: request.source_admission,
    })?;
    let output_str = out_path.to_string_lossy();
    debug_assert!(!output_str.is_empty());

    let sidecar = load_shell_sidecar(&out_path)?;
    let host_env = snapshot_host_env()?;
    let exec_mode = resolve_shell_exec_mode(request.command_argv, request.run_script);
    validate_with_paths(request.with_paths)?;

    let plan = compute_activation(&sidecar, &host_env, &output_str, request.with_paths, &exec_mode)
        .map_err(|e| RunError::Internal(format!("activation: {e}")))?;
    debug_assert_eq!(
        matches!(&exec_mode, ExecMode::Interactive),
        matches!(&plan.exec_target, ExecTarget::Interactive { .. })
    );

    for warning in &plan.warnings {
        eprintln!("warning: {warning:?}");
    }

    if let Some(hook) = select_executable_hook(request.hook_execution, plan.hook.as_deref()) {
        exec_hook(hook, &plan, &host_env.shell, request.strict_hooks)?;
    }

    exec_plan(&plan)
}

fn select_executable_hook(hook_execution: HookExecutionMode, hook: Option<&str>) -> Option<&str> {
    let selected = if hook_execution.should_execute {
        hook.filter(|script| !script.is_empty())
    } else {
        None
    };
    if let Some(script) = selected {
        debug_assert!(!script.is_empty());
    }
    selected
}

fn load_shell_sidecar(out_path: &Path) -> Result<ShellSidecar, RunError> {
    debug_assert!(!out_path.as_os_str().is_empty());
    let sidecar_path = out_path.join(SIDECAR_FILENAME);
    debug_assert!(sidecar_path.starts_with(out_path));
    debug_assert!(sidecar_path.file_name().is_some());

    let effects =
        crate::build_observation::plan(mantle_application_contract::realization::RealizationEffect::Activation)?;
    let sidecar = std::fs::read_to_string(&sidecar_path)
        .map_err(|error| {
            RunError::Internal(format!(
                "missing {SIDECAR_FILENAME} in {} — was this built with mkShell? ({error})",
                out_path.display()
            ))
        })
        .and_then(|bytes| {
            ShellSidecar::from_json(&bytes).map_err(|error| RunError::Internal(format!("bad sidecar: {error}")))
        });
    crate::build_observation::classify(
        &effects,
        "realization-activation",
        mantle_application_contract::EffectKind::ReadFiles,
        sidecar.is_ok(),
    )?;
    sidecar
}

fn resolve_shell_exec_mode(command_argv: &[OsString], run_script: Option<&str>) -> ExecMode {
    let mode = if !command_argv.is_empty() {
        ExecMode::Command {
            argv: command_argv.to_vec(),
        }
    } else if let Some(script) = run_script {
        ExecMode::Run {
            script: script.to_string(),
        }
    } else {
        ExecMode::Interactive
    };
    debug_assert_eq!(matches!(&mode, ExecMode::Command { .. }), !command_argv.is_empty());
    if command_argv.is_empty() {
        debug_assert_eq!(matches!(&mode, ExecMode::Run { .. }), run_script.is_some());
        debug_assert_eq!(matches!(&mode, ExecMode::Interactive), run_script.is_none());
    }
    mode
}

fn shell_project_target(target: &project_build::ProjectTarget) -> project_build::ProjectTarget {
    let mapped = match target {
        project_build::ProjectTarget::Default => project_build::ProjectTarget::DefaultShell,
        project_build::ProjectTarget::Attribute(segs) if segs.len() == 1 => {
            project_build::ProjectTarget::NamedShell(segs[0].clone())
        }
        other => other.clone(),
    };
    debug_assert_eq!(
        matches!(target, project_build::ProjectTarget::Default),
        matches!(mapped, project_build::ProjectTarget::DefaultShell)
    );
    if let project_build::ProjectTarget::Attribute(segs) = target {
        debug_assert_eq!(segs.len() == 1, matches!(mapped, project_build::ProjectTarget::NamedShell(_)));
    }
    mapped
}

struct BuildShellTargetRequest<'a> {
    name: Option<&'a str>,
    import_paths: &'a [PathBuf],
    jobs: Option<u32>,
    signing_key: Option<&'a Path>,
    is_unsigned_trusted: bool,
    output_dir: &'a Path,
    state_dir: &'a Path,
    store_prefix: &'a str,
    backend: crunch_store::StoreBackend,
    is_verbose: bool,
    source_admission: Option<&'a std::sync::Arc<dyn crunch_store::StoreSourceAdmission>>,
}

fn build_shell_target(request: BuildShellTargetRequest<'_>) -> Result<PathBuf, RunError> {
    debug_assert!(!request.store_prefix.is_empty());
    debug_assert!(request.store_prefix.starts_with('/'));
    debug_assert!(!request.output_dir.as_os_str().is_empty());

    let cwd = crate::current_dir_or_error()?;
    let target = crate::name_to_build_target(request.name);
    let resolved = crate::build_application::resolve_project_target(&target, &cwd, request.import_paths)?;
    let shell_target = shell_project_target(&resolved.target);
    let expr = project_build::generate_extraction_expr(&resolved.root_file, &shell_target);
    let root_registration = shell_root_registration(&resolved, &shell_target)?;
    debug_assert!(root_registration.lease.is_some());

    let max_jobs = crunch_pipeline::resolve_max_jobs(request.jobs);
    let result = crate::build_project_expr(crate::BuildProjectExprRequest {
        expr: &expr,
        resolved_import_entries: resolved.import_paths,
        output_dir: request.output_dir,
        state_dir: request.state_dir,
        store_prefix: request.store_prefix,
        backend: request.backend,
        is_verbose: request.is_verbose,
        max_jobs,
        signing_key_path: request.signing_key,
        is_unsigned_trusted: request.is_unsigned_trusted,
        root_registration: Some(root_registration),
        source_admission: request.source_admission,
    })?;
    if !result.failed.is_empty() {
        return Err(RunError::Build("shell target has failed build roots".to_string()));
    }
    debug_assert!(result.failed.is_empty());
    crate::build_application::first_output_path(&result, request.output_dir)?
        .ok_or_else(|| RunError::Internal("no outputs built for shell".into()))
}

fn shell_root_registration(
    project: &project_build::ResolvedProject,
    shell_target: &project_build::ProjectTarget,
) -> Result<crunch_store::RootRegistration, RunError> {
    let registration = crate::build_application::project_output_root_registration(project)?;
    let selector = crate::build_application::project_retention_selector(shell_target);
    debug_assert!(!selector.is_empty());

    let now_unix_s_u64 = crate::unix_time_now_s()?;
    let now_unix_s =
        i64::try_from(now_unix_s_u64).map_err(|_| RunError::Internal("shell lease time exceeds i64".to_string()))?;
    let lease_seconds = crunch_store::store_retention_runtime_policy().limits.default_lease_seconds;
    let planned = plan_shell_root_registration(registration, selector, now_unix_s, lease_seconds)?;
    debug_assert!(planned.lease.is_some());
    Ok(planned)
}

struct ShellLeaseIdentityInput<'a> {
    project_identity: &'a str,
    selector: &'a str,
    now_unix_s: i64,
}

fn plan_shell_root_registration(
    mut registration: crunch_store::RootRegistration,
    selector: String,
    now_unix_s: i64,
    lease_seconds: u64,
) -> Result<crunch_store::RootRegistration, RunError> {
    debug_assert!(!selector.is_empty());
    let lease_seconds_i64 =
        i64::try_from(lease_seconds).map_err(|_| RunError::Internal("shell lease duration exceeds i64".to_string()))?;
    let expires_unix_s = now_unix_s
        .checked_add(lease_seconds_i64)
        .ok_or_else(|| RunError::Internal("shell lease expiry overflowed i64".to_string()))?;
    debug_assert!(expires_unix_s >= now_unix_s);

    let project_identity = registration
        .project_identity
        .as_deref()
        .ok_or_else(|| RunError::Internal("shell project identity is missing".to_string()))?;
    let lease_id = shell_lease_identity(&ShellLeaseIdentityInput {
        project_identity,
        selector: &selector,
        now_unix_s,
    });
    registration.class = crunch_store::GcRootClass::ActiveShellLease;
    registration.selector = Some(selector);
    registration.generation = None;
    registration.generation_identity = None;
    registration.lease = Some(crunch_store::GcRootLease {
        lease_id,
        expires_unix_s,
        last_observed_unix_s: now_unix_s,
        renewal_count: 0,
    });
    debug_assert!(registration.lease.is_some());
    Ok(registration)
}

fn shell_lease_identity(input: &ShellLeaseIdentityInput<'_>) -> String {
    debug_assert!(!input.project_identity.is_empty());
    debug_assert!(!input.selector.is_empty());
    let mut hasher = blake3::Hasher::new();
    hasher.update(SHELL_LEASE_ID_DOMAIN);
    hash_shell_lease_field(&mut hasher, input.project_identity.as_bytes());
    hash_shell_lease_field(&mut hasher, input.selector.as_bytes());
    hasher.update(&input.now_unix_s.to_be_bytes());
    let lease_id = format!("b3:{}", hasher.finalize().to_hex());
    debug_assert!(lease_id.starts_with("b3:"));
    lease_id
}

fn hash_shell_lease_field(hasher: &mut blake3::Hasher, bytes: &[u8]) {
    match u128::try_from(bytes.len()) {
        Ok(byte_count) => {
            debug_assert_eq!(usize::try_from(byte_count).ok(), Some(bytes.len()));
            hasher.update(&byte_count.to_be_bytes());
        }
        Err(_) => {
            hasher.update(b"len-overflow");
        }
    }
    hasher.update(bytes);
}

fn resolve_host_shell(env: &BTreeMap<String, String>) -> PathBuf {
    let raw_shell = env.get("SHELL").map(String::as_str).filter(|value| !value.is_empty());
    let shell = raw_shell.map(PathBuf::from).unwrap_or_else(|| PathBuf::from("/bin/sh"));
    match raw_shell {
        Some(configured) => {
            debug_assert_eq!(shell.as_os_str(), std::ffi::OsStr::new(configured));
        }
        None => {
            debug_assert_eq!(shell, Path::new("/bin/sh"));
        }
    }
    debug_assert!(!shell.as_os_str().is_empty());
    shell
}

fn snapshot_host_env() -> Result<HostEnv, RunError> {
    let effects =
        crate::build_observation::plan(mantle_application_contract::realization::RealizationEffect::Environment)?;
    let env: BTreeMap<String, String> = std::env::vars().collect();
    crate::build_observation::classify(
        &effects,
        "realization-environment",
        mantle_application_contract::EffectKind::ReadEnvironment,
        true,
    )?;
    let shell = resolve_host_shell(&env);
    debug_assert!(!shell.as_os_str().is_empty());
    Ok(HostEnv { env, shell })
}

fn resolve_hook_shell(exec_target: &ExecTarget, host_shell: &Path) -> PathBuf {
    debug_assert!(!host_shell.as_os_str().is_empty());
    let shell = match exec_target {
        ExecTarget::Interactive { shell } => shell.clone(),
        ExecTarget::Run { shell, .. } => shell.clone(),
        ExecTarget::Command { .. } => host_shell.to_path_buf(),
    };
    debug_assert!(!shell.as_os_str().is_empty());
    shell
}

fn exec_hook(
    hook: &str,
    plan: &ActivationPlan,
    host_shell: &Path,
    strict_mode: StrictHookMode,
) -> Result<(), RunError> {
    debug_assert!(!hook.is_empty());
    debug_assert!(!host_shell.as_os_str().is_empty());
    let shell = resolve_hook_shell(&plan.exec_target, host_shell);
    debug_assert!(!shell.as_os_str().is_empty());

    let env_path = std::env::join_paths(&plan.path)
        .map_err(|error| RunError::Internal(format!("joining shell activation PATH: {error}")))?;

    let effects = crate::build_observation::plan(mantle_application_contract::realization::RealizationEffect::Child)?;
    let status = std::process::Command::new(&shell)
        .arg("-c")
        .arg(hook)
        .envs(&plan.env)
        .env("PATH", &env_path)
        .status();
    crate::build_observation::classify(
        &effects,
        "realization-child",
        mantle_application_contract::EffectKind::RunProcess,
        matches!(&status, Ok(status) if status.success()),
    )?;
    let status = status.map_err(|e| RunError::Internal(format!("hook exec: {e}")))?;

    if status.success() {
        return Ok(());
    }
    let code = status.code().unwrap_or(1);
    if strict_mode.is_strict {
        std::process::exit(code);
    }
    debug_assert!(!strict_mode.is_strict);
    eprintln!("warning: shell hook exited with code {code}");
    Ok(())
}

fn validate_with_paths(paths: &[PathBuf]) -> Result<(), RunError> {
    if paths.is_empty() {
        return Ok(());
    }
    let path_count = u32::try_from(paths.len())
        .map_err(|_| RunError::Internal("activation inputs exceed path bound".to_string()))?;
    debug_assert!(path_count > 0);
    let effects = mantle_application_contract::realization::shell_with_paths_effect_plan(path_count)
        .map_err(|error| RunError::Internal(format!("planning activation inputs: {}", error.code())))?;
    let mut checked_count = 0_u32;
    let missing = paths.iter().find(|path| {
        checked_count = checked_count.saturating_add(1);
        !path.exists()
    });
    debug_assert!(checked_count > 0);
    debug_assert!(checked_count <= path_count);
    let observed = mantle_application_contract::Observation {
        effect_id: mantle_application_contract::EffectId("shell-with-paths".to_string()),
        kind: mantle_application_contract::EffectKind::ReadFiles,
        status: if missing.is_none() {
            mantle_application_contract::ObservationStatus::Succeeded
        } else {
            mantle_application_contract::ObservationStatus::Failed
        },
        output: mantle_application_contract::EffectOutput::None,
        usage: mantle_application_contract::EffectMeasure::Items(checked_count),
        diagnostics_code: missing.map(|_| "shell-with-path-missing".to_string()),
    };
    match (mantle_application_contract::classify_observations(&effects, &[observed]), missing) {
        (mantle_application_contract::ApplicationOutcome::Completed, None) => Ok(()),
        (mantle_application_contract::ApplicationOutcome::Failed { .. }, Some(path)) => {
            Err(RunError::Internal(format!("--with path does not exist: {}", path.display())))
        }
        (outcome, _) => Err(RunError::Internal(format!("activation inputs contradicted plan: {outcome:?}"))),
    }
}

/// Search for a program name in the given PATH directories.
/// Returns the absolute path if found, None otherwise.
fn resolve_in_path(cmd: &OsString, path: &[PathBuf]) -> Result<Option<OsString>, RunError> {
    let cmd_path = Path::new(cmd);
    if cmd_path.components().count() > 1 {
        return Ok(None);
    }
    if path.is_empty() {
        return Ok(None);
    }
    let dir_count = u32::try_from(path.len())
        .map_err(|_| RunError::Internal("activation PATH exceeds candidate bound".to_string()))?;
    debug_assert!(dir_count > 0);
    let effects = mantle_application_contract::realization::shell_path_search_effect_plan(dir_count)
        .map_err(|error| RunError::Internal(format!("planning activation PATH search: {}", error.code())))?;
    let mut checked_count = 0_u32;
    let mut selected = None;
    for dir in path {
        let candidate = dir.join(cmd);
        checked_count = checked_count.saturating_add(1);
        if candidate.is_file() {
            selected = Some(candidate.into_os_string());
            break;
        }
    }
    debug_assert!(checked_count > 0);
    debug_assert!(checked_count <= dir_count);
    let observed = mantle_application_contract::Observation {
        effect_id: mantle_application_contract::EffectId("shell-path-search".to_string()),
        kind: mantle_application_contract::EffectKind::ReadFiles,
        status: mantle_application_contract::ObservationStatus::Succeeded,
        output: mantle_application_contract::EffectOutput::None,
        usage: mantle_application_contract::EffectMeasure::Items(checked_count),
        diagnostics_code: None,
    };
    match mantle_application_contract::classify_observations(&effects, &[observed]) {
        mantle_application_contract::ApplicationOutcome::Completed => Ok(selected),
        outcome => Err(RunError::Internal(format!("activation PATH search contradicted plan: {outcome:?}"))),
    }
}

#[derive(Debug, PartialEq, Eq)]
struct PreparedExecCommand {
    program: OsString,
    args: Vec<OsString>,
    banner: Option<String>,
}

fn prepare_exec_command(
    exec_target: &ExecTarget,
    path: &[PathBuf],
    crunch_shell_label: Option<&str>,
) -> Result<PreparedExecCommand, RunError> {
    let prepared = match exec_target {
        ExecTarget::Interactive { shell } => {
            debug_assert!(!shell.as_os_str().is_empty());
            let label = crunch_shell_label.unwrap_or("");
            PreparedExecCommand {
                program: shell.as_os_str().to_os_string(),
                args: Vec::new(),
                banner: Some(format!("entering shell ({label})")),
            }
        }
        ExecTarget::Command { argv } => {
            let (cmd, args) = argv
                .split_first()
                .ok_or_else(|| RunError::Internal("--command requires at least one argument".into()))?;
            debug_assert!(!cmd.is_empty());
            // Resolve the program against the activation PATH, not the host
            // PATH. Command::new uses execvp which searches the *parent's*
            // PATH, but we want the composed shell PATH.
            let resolved = resolve_in_path(cmd, path)?.unwrap_or_else(|| cmd.clone());
            PreparedExecCommand {
                program: resolved,
                args: args.to_vec(),
                banner: None,
            }
        }
        ExecTarget::Run { shell, script } => {
            debug_assert!(!shell.as_os_str().is_empty());
            PreparedExecCommand {
                program: shell.as_os_str().to_os_string(),
                args: vec![OsString::from("-c"), OsString::from(script)],
                banner: None,
            }
        }
    };
    debug_assert!(!prepared.program.is_empty());
    Ok(prepared)
}

fn exec_plan(plan: &ActivationPlan) -> Result<(), RunError> {
    let env_path = std::env::join_paths(&plan.path)
        .map_err(|error| RunError::Internal(format!("joining shell activation PATH: {error}")))?;
    if plan.path.is_empty() {
        debug_assert!(env_path.is_empty());
    }

    let prepared =
        prepare_exec_command(&plan.exec_target, &plan.path, plan.env.get("CRUNCH_SHELL").map(String::as_str))?;
    debug_assert!(!prepared.program.is_empty());

    let effects = crate::build_observation::plan(mantle_application_contract::realization::RealizationEffect::Child)?;
    if let Some(banner) = prepared.banner {
        eprintln!("{banner}");
    }
    let status = std::process::Command::new(&prepared.program)
        .args(&prepared.args)
        .envs(&plan.env)
        .env("PATH", &env_path)
        .status();
    crate::build_observation::classify(
        &effects,
        "realization-child",
        mantle_application_contract::EffectKind::RunProcess,
        matches!(&status, Ok(status) if status.success()),
    )?;
    let status = status.map_err(|e| RunError::Internal(format!("exec: {e}")))?;

    std::process::exit(status.code().unwrap_or(1));
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_NOW_UNIX_S: i64 = 1_000;
    const TEST_LEASE_SECONDS: u64 = 600;
    const TEST_EXPECTED_EXPIRY_UNIX_S: i64 = 1_600;

    fn project_registration() -> crunch_store::RootRegistration {
        crunch_store::RootRegistration {
            class: crunch_store::GcRootClass::ProjectOutputGeneration,
            owner_scope: "project:b3:shell-test".to_string(),
            project_identity: Some("b3:shell-test".to_string()),
            selector: Some("default".to_string()),
            generation: Some(1),
            generation_identity: Some("b3:shell-generation".to_string()),
            lease: None,
            removal_requested: false,
        }
    }

    #[test]
    fn shell_lease_plan_is_deterministic_and_does_not_change_project_identity() {
        let first = plan_shell_root_registration(
            project_registration(),
            "shell:default".to_string(),
            TEST_NOW_UNIX_S,
            TEST_LEASE_SECONDS,
        )
        .expect("valid shell lease plan");
        let second = plan_shell_root_registration(
            project_registration(),
            "shell:default".to_string(),
            TEST_NOW_UNIX_S,
            TEST_LEASE_SECONDS,
        )
        .expect("equivalent shell lease plan");

        assert_eq!(first, second);
        assert_eq!(first.class, crunch_store::GcRootClass::ActiveShellLease);
        assert_eq!(first.project_identity.as_deref(), Some("b3:shell-test"));
        assert_eq!(first.selector.as_deref(), Some("shell:default"));
        assert!(first.generation.is_none());
        assert!(first.generation_identity.is_none());
        let lease = first.lease.expect("planned lease facts");
        assert_eq!(lease.expires_unix_s, TEST_EXPECTED_EXPIRY_UNIX_S);
        assert_eq!(lease.last_observed_unix_s, TEST_NOW_UNIX_S);
        assert_eq!(lease.renewal_count, 0);
        assert!(lease.lease_id.starts_with("b3:"));
    }

    #[test]
    fn shell_lease_plan_rejects_missing_identity_and_time_overflow() {
        let mut missing_identity = project_registration();
        missing_identity.project_identity = None;
        let missing_error = plan_shell_root_registration(
            missing_identity,
            "shell:default".to_string(),
            TEST_NOW_UNIX_S,
            TEST_LEASE_SECONDS,
        )
        .expect_err("missing project identity must fail");
        assert!(matches!(missing_error, RunError::Internal(message) if message.contains("project identity")));

        let overflow_error = plan_shell_root_registration(
            project_registration(),
            "shell:default".to_string(),
            i64::MAX,
            TEST_LEASE_SECONDS,
        )
        .expect_err("lease expiry overflow must fail");
        assert!(matches!(overflow_error, RunError::Internal(message) if message.contains("expiry overflow")));
    }

    #[test]
    fn resolve_shell_exec_mode_selects_command_run_or_interactive() {
        let cmd_mode = resolve_shell_exec_mode(&[OsString::from("env"), OsString::from("-i")], Some("ignored"));
        assert!(matches!(
            cmd_mode,
            ExecMode::Command { argv } if argv == vec![OsString::from("env"), OsString::from("-i")]
        ));

        let run_mode = resolve_shell_exec_mode(&[], Some("echo ok"));
        assert!(matches!(run_mode, ExecMode::Run { script } if script == "echo ok"));

        let interactive_mode = resolve_shell_exec_mode(&[], None);
        assert!(matches!(interactive_mode, ExecMode::Interactive));
    }

    #[test]
    fn shell_project_target_maps_default_and_single_attribute_shells() {
        assert_eq!(
            shell_project_target(&project_build::ProjectTarget::Default),
            project_build::ProjectTarget::DefaultShell
        );
        assert_eq!(
            shell_project_target(&project_build::ProjectTarget::Attribute(vec!["dev".to_string()])),
            project_build::ProjectTarget::NamedShell("dev".to_string())
        );
        let nested = project_build::ProjectTarget::Attribute(vec!["shells".to_string(), "dev".to_string()]);
        assert_eq!(shell_project_target(&nested), nested);
    }

    #[test]
    fn resolve_hook_shell_uses_target_shell_or_host_shell_for_commands() {
        let host_shell = Path::new("/bin/host-sh");
        let interactive = ExecTarget::Interactive {
            shell: PathBuf::from("/bin/nix-bash"),
        };
        assert_eq!(resolve_hook_shell(&interactive, host_shell), Path::new("/bin/nix-bash"));

        let run = ExecTarget::Run {
            shell: PathBuf::from("/bin/nix-zsh"),
            script: "true".to_string(),
        };
        assert_eq!(resolve_hook_shell(&run, host_shell), Path::new("/bin/nix-zsh"));

        let command = ExecTarget::Command {
            argv: vec![OsString::from("env")],
        };
        assert_eq!(resolve_hook_shell(&command, host_shell), host_shell);
    }

    #[test]
    fn validate_with_paths_accepts_empty_or_existing_and_rejects_missing() {
        validate_with_paths(&[]).expect("empty with-paths should succeed");
        let tmp = tempfile::tempdir().expect("create tempdir");
        let existing = tmp.path().to_path_buf();
        validate_with_paths(&[existing]).expect("existing directory should succeed");

        let missing = tmp.path().join("nonexistent-store-path");
        let err = validate_with_paths(&[missing]).expect_err("missing path must fail");
        assert!(matches!(err, RunError::Internal(message) if message.contains("--with path does not exist")));
    }

    #[test]
    fn resolve_in_path_finds_regular_file_and_ignores_multi_component_or_empty_path() {
        let tmp = tempfile::tempdir().expect("create tempdir");
        let bin_dir = tmp.path().join("bin");
        std::fs::create_dir_all(&bin_dir).expect("create bin dir");
        let tool_path = bin_dir.join("my-tool");
        std::fs::write(&tool_path, b"#!/bin/sh\n").expect("write tool file");

        let found = resolve_in_path(&OsString::from("my-tool"), std::slice::from_ref(&bin_dir))
            .expect("resolve tool")
            .expect("tool should be found");
        assert_eq!(found, tool_path.into_os_string());

        let multi = resolve_in_path(&OsString::from("sub/my-tool"), std::slice::from_ref(&bin_dir))
            .expect("multi-component command should not fail");
        assert_eq!(multi, None);

        let empty_path = resolve_in_path(&OsString::from("my-tool"), &[]).expect("empty path should not fail");
        assert_eq!(empty_path, None);
    }

    #[test]
    fn prepare_exec_command_handles_modes_and_rejects_empty_argv() {
        let interactive = prepare_exec_command(
            &ExecTarget::Interactive {
                shell: PathBuf::from("/bin/bash"),
            },
            &[],
            Some("dev"),
        )
        .expect("interactive prepare");
        assert_eq!(interactive, PreparedExecCommand {
            program: OsString::from("/bin/bash"),
            args: Vec::new(),
            banner: Some("entering shell (dev)".to_string()),
        });

        let tmp = tempfile::tempdir().expect("create tempdir");
        let tool = tmp.path().join("hello");
        std::fs::write(&tool, b"").expect("write tool");
        let cmd = prepare_exec_command(
            &ExecTarget::Command {
                argv: vec![OsString::from("hello"), OsString::from("--flag")],
            },
            &[tmp.path().to_path_buf()],
            None,
        )
        .expect("command prepare");
        assert_eq!(cmd, PreparedExecCommand {
            program: tool.into_os_string(),
            args: vec![OsString::from("--flag")],
            banner: None,
        });

        let run = prepare_exec_command(
            &ExecTarget::Run {
                shell: PathBuf::from("/bin/sh"),
                script: "echo hi".to_string(),
            },
            &[],
            None,
        )
        .expect("run prepare");
        assert_eq!(run, PreparedExecCommand {
            program: OsString::from("/bin/sh"),
            args: vec![OsString::from("-c"), OsString::from("echo hi")],
            banner: None,
        });

        let empty_argv = prepare_exec_command(&ExecTarget::Command { argv: Vec::new() }, &[], None)
            .expect_err("empty argv must fail");
        assert!(matches!(empty_argv, RunError::Internal(message) if message.contains("--command requires")));
    }

    #[cfg(debug_assertions)]
    #[test]
    #[should_panic]
    fn cmd_shell_rejects_relative_store_prefix_in_debug() {
        let _ = cmd_shell(ShellCommandRequest {
            name: None,
            import_paths: &[],
            jobs: None,
            signing_key: None,
            is_unsigned_trusted: false,
            command_argv: &[],
            run_script: None,
            with_paths: &[],
            hook_execution: HookExecutionMode::from(true),
            strict_hooks: StrictHookMode::from(false),
            output_dir: Path::new("/tmp/out"),
            state_dir: Path::new("/tmp/state"),
            store_prefix: "relative/store",
            backend: crunch_store::StoreBackend::Snix,
            is_verbose: false,
            source_admission: None,
        });
    }

    #[cfg(debug_assertions)]
    #[test]
    #[should_panic]
    fn build_shell_target_rejects_empty_output_dir_in_debug() {
        let _ = build_shell_target(BuildShellTargetRequest {
            name: None,
            import_paths: &[],
            jobs: None,
            signing_key: None,
            is_unsigned_trusted: false,
            output_dir: Path::new(""),
            state_dir: Path::new("/tmp/state"),
            store_prefix: "/nix/store",
            backend: crunch_store::StoreBackend::Snix,
            is_verbose: false,
            source_admission: None,
        });
    }

    #[cfg(debug_assertions)]
    #[test]
    #[should_panic]
    fn load_shell_sidecar_rejects_empty_out_path_in_debug() {
        let _ = load_shell_sidecar(Path::new(""));
    }

    #[cfg(debug_assertions)]
    #[test]
    #[should_panic]
    fn exec_hook_rejects_empty_hook_in_debug() {
        let plan = ActivationPlan {
            env: BTreeMap::new(),
            path: Vec::new(),
            hook: None,
            exec_target: ExecTarget::Command {
                argv: vec![OsString::from("true")],
            },
            warnings: Vec::new(),
        };
        let _ = exec_hook("", &plan, Path::new("/bin/sh"), StrictHookMode::from(false));
    }

    #[cfg(debug_assertions)]
    #[test]
    #[should_panic]
    fn exec_hook_rejects_empty_host_shell_in_debug() {
        let plan = ActivationPlan {
            env: BTreeMap::new(),
            path: Vec::new(),
            hook: None,
            exec_target: ExecTarget::Command {
                argv: vec![OsString::from("true")],
            },
            warnings: Vec::new(),
        };
        let _ = exec_hook("echo hi", &plan, Path::new(""), StrictHookMode::from(false));
    }

    #[cfg(debug_assertions)]
    #[test]
    #[should_panic]
    fn resolve_hook_shell_rejects_empty_target_shell_in_debug() {
        let target = ExecTarget::Interactive { shell: PathBuf::new() };
        let _ = resolve_hook_shell(&target, Path::new("/bin/sh"));
    }

    #[cfg(debug_assertions)]
    #[test]
    #[should_panic]
    fn prepare_exec_command_rejects_empty_command_name_in_debug() {
        let target = ExecTarget::Command {
            argv: vec![OsString::new()],
        };
        let _ = prepare_exec_command(&target, &[], None);
    }

    #[test]
    fn shell_lease_identity_distinguishes_field_boundaries_and_is_deterministic() {
        let first = shell_lease_identity(&ShellLeaseIdentityInput {
            project_identity: "ab",
            selector: "c",
            now_unix_s: TEST_NOW_UNIX_S,
        });
        let first_repeat = shell_lease_identity(&ShellLeaseIdentityInput {
            project_identity: "ab",
            selector: "c",
            now_unix_s: TEST_NOW_UNIX_S,
        });
        let shifted_boundary = shell_lease_identity(&ShellLeaseIdentityInput {
            project_identity: "a",
            selector: "bc",
            now_unix_s: TEST_NOW_UNIX_S,
        });
        let different_time = shell_lease_identity(&ShellLeaseIdentityInput {
            project_identity: "ab",
            selector: "c",
            now_unix_s: TEST_NOW_UNIX_S + 1,
        });

        assert_eq!(first, first_repeat);
        assert_ne!(first, shifted_boundary);
        assert_ne!(first, different_time);
        assert!(first.starts_with("b3:"));
        assert!(shifted_boundary.starts_with("b3:"));
    }

    #[test]
    fn resolve_host_shell_falls_back_to_bin_sh_when_missing_or_empty() {
        let missing = BTreeMap::new();
        assert_eq!(resolve_host_shell(&missing), Path::new("/bin/sh"));

        let empty = BTreeMap::from([("SHELL".to_string(), String::new())]);
        assert_eq!(resolve_host_shell(&empty), Path::new("/bin/sh"));

        let configured = BTreeMap::from([("SHELL".to_string(), "/bin/zsh".to_string())]);
        assert_eq!(resolve_host_shell(&configured), Path::new("/bin/zsh"));
    }

    #[test]
    fn select_executable_hook_skips_disabled_missing_and_empty_hooks() {
        assert_eq!(select_executable_hook(HookExecutionMode::from(true), Some("echo hi")), None);
        assert_eq!(select_executable_hook(HookExecutionMode::from(false), None), None);
        assert_eq!(select_executable_hook(HookExecutionMode::from(false), Some("")), None);
        assert_eq!(select_executable_hook(HookExecutionMode::from(false), Some("export FOO=1")), Some("export FOO=1"));
    }
}
