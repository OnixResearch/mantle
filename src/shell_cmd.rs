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

/// Read the sidecar from a built shell output, snapshot the host env,
/// compute the activation plan, and exec into the result.
#[expect(
    clippy::too_many_arguments,
    tigerstyle::too_many_parameters,
    reason = "the protected CLI dispatch caller retains this compatibility boundary"
)]
pub(crate) fn cmd_shell(
    name: Option<&str>,
    import_paths: &[PathBuf],
    jobs: Option<u32>,
    no_substitute: bool,
    signing_key: Option<&Path>,
    trust_unsigned: bool,
    command_argv: &[OsString],
    run_script: Option<&str>,
    with_paths: &[PathBuf],
    no_hook: impl Into<HookExecutionMode>,
    strict_hooks: impl Into<StrictHookMode>,
    output_dir: &Path,
    state_dir: &Path,
    store_prefix: &str,
    verbose: bool,
) -> Result<(), RunError> {
    debug_assert!(!SIDECAR_FILENAME.is_empty());
    debug_assert!(SIDECAR_FILENAME.starts_with('.'));
    let hook_execution_mode = no_hook.into();
    let strict_hook_mode = strict_hooks.into();
    // 1. Build the shell derivation.
    let out_path = build_shell_target(BuildShellTargetRequest {
        name,
        import_paths,
        jobs,
        no_substitute,
        signing_key,
        trust_unsigned,
        output_dir,
        state_dir,
        store_prefix,
        verbose,
    })?;

    // 2. Read sidecar from $out/.crunch-shell.json.
    let sidecar_path = out_path.join(SIDECAR_FILENAME);
    let sidecar_bytes = std::fs::read_to_string(&sidecar_path).map_err(|e| {
        RunError::Internal(format!(
            "missing {SIDECAR_FILENAME} in {} — was this built with mkShell? ({})",
            out_path.display(),
            e,
        ))
    })?;
    let sidecar =
        ShellSidecar::from_json(&sidecar_bytes).map_err(|e| RunError::Internal(format!("bad sidecar: {e}")))?;

    // 3. Snapshot host environment.
    let host_env = snapshot_host_env();

    // 4. Determine exec mode.
    let exec_mode = if !command_argv.is_empty() {
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

    // 5. Validate --with paths.
    for p in with_paths {
        if !p.exists() {
            return Err(RunError::Internal(format!("--with path does not exist: {}", p.display(),)));
        }
    }

    // 6. Compute activation plan (pure).
    let output_str = out_path.to_string_lossy();
    let plan = compute_activation(&sidecar, &host_env, &output_str, with_paths, &exec_mode)
        .map_err(|e| RunError::Internal(format!("activation: {e}")))?;

    // 7. Print warnings.
    for w in &plan.warnings {
        eprintln!("warning: {w:?}");
    }

    // 8. Execute hook (shell-side decision).
    if hook_execution_mode.should_execute
        && let Some(ref hook) = plan.hook
    {
        exec_hook(hook, &plan, strict_hook_mode.is_strict)?;
    }

    // 9. Exec into target.
    exec_plan(&plan)
}

struct BuildShellTargetRequest<'a> {
    name: Option<&'a str>,
    import_paths: &'a [PathBuf],
    jobs: Option<u32>,
    no_substitute: bool,
    signing_key: Option<&'a Path>,
    trust_unsigned: bool,
    output_dir: &'a Path,
    state_dir: &'a Path,
    store_prefix: &'a str,
    verbose: bool,
}

fn build_shell_target(request: BuildShellTargetRequest<'_>) -> Result<PathBuf, RunError> {
    debug_assert!(!SIDECAR_FILENAME.is_empty());
    debug_assert!(SIDECAR_FILENAME.starts_with('.'));
    let cwd = crate::current_dir_or_error()?;
    let target = crate::name_to_build_target(request.name);
    let resolved = project_build::resolve_project_target(&target, &cwd, request.import_paths)?;
    let shell_target = match &resolved.target {
        project_build::ProjectTarget::Default => project_build::ProjectTarget::DefaultShell,
        project_build::ProjectTarget::Attribute(segs) if segs.len() == 1 => {
            project_build::ProjectTarget::NamedShell(segs[0].clone())
        }
        other => other.clone(),
    };
    let expr = project_build::generate_extraction_expr(&resolved.root_file, &shell_target);
    let root_registration = shell_root_registration(&resolved, &shell_target)?;
    let max_jobs = crunch_pipeline::resolve_max_jobs(request.jobs);
    let result = crate::build_project_expr(
        &expr,
        resolved.import_paths,
        request.output_dir,
        request.state_dir,
        request.store_prefix,
        request.verbose,
        max_jobs,
        request.no_substitute,
        request.signing_key,
        request.trust_unsigned,
        Some(root_registration),
    )?;
    crate::first_output_path(&result, request.output_dir, request.store_prefix)
        .ok_or_else(|| RunError::Internal("no outputs built for shell".into()))
}

fn shell_root_registration(
    project: &project_build::ResolvedProject,
    shell_target: &project_build::ProjectTarget,
) -> Result<crunch_store::RootRegistration, RunError> {
    let registration = crate::project_output_root_registration(project)?;
    let selector = crate::project_retention_selector(shell_target);
    let now_unix_s_u64 = crate::unix_time_now_s()?;
    let now_unix_s =
        i64::try_from(now_unix_s_u64).map_err(|_| RunError::Internal("shell lease time exceeds i64".to_string()))?;
    let lease_seconds = crunch_store::store_retention_runtime_policy().limits.default_lease_seconds;
    plan_shell_root_registration(registration, selector, now_unix_s, lease_seconds)
}

fn plan_shell_root_registration(
    mut registration: crunch_store::RootRegistration,
    selector: String,
    now_unix_s: i64,
    lease_seconds: u64,
) -> Result<crunch_store::RootRegistration, RunError> {
    let lease_seconds_i64 =
        i64::try_from(lease_seconds).map_err(|_| RunError::Internal("shell lease duration exceeds i64".to_string()))?;
    let expires_unix_s = now_unix_s
        .checked_add(lease_seconds_i64)
        .ok_or_else(|| RunError::Internal("shell lease expiry overflowed i64".to_string()))?;
    let project_identity = registration
        .project_identity
        .as_deref()
        .ok_or_else(|| RunError::Internal("shell project identity is missing".to_string()))?;
    let lease_id = shell_lease_identity(project_identity, &selector, now_unix_s);
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
    Ok(registration)
}

fn shell_lease_identity(project_identity: &str, selector: &str, now_unix_s: i64) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(SHELL_LEASE_ID_DOMAIN);
    hash_shell_lease_field(&mut hasher, project_identity.as_bytes());
    hash_shell_lease_field(&mut hasher, selector.as_bytes());
    hasher.update(&now_unix_s.to_be_bytes());
    format!("b3:{}", hasher.finalize().to_hex())
}

fn hash_shell_lease_field(hasher: &mut blake3::Hasher, bytes: &[u8]) {
    hasher.update(&(bytes.len() as u128).to_be_bytes());
    hasher.update(bytes);
}

fn snapshot_host_env() -> HostEnv {
    let env: BTreeMap<String, String> = std::env::vars().collect();
    let shell = std::env::var("SHELL").map(PathBuf::from).unwrap_or_else(|_| PathBuf::from("/bin/sh"));
    HostEnv { env, shell }
}

fn exec_hook(hook: &str, plan: &ActivationPlan, strict: bool) -> Result<(), RunError> {
    debug_assert!(!SIDECAR_FILENAME.is_empty());
    debug_assert!(SIDECAR_FILENAME.starts_with('.'));
    let shell = match &plan.exec_target {
        ExecTarget::Interactive { shell } => shell.clone(),
        ExecTarget::Run { shell, .. } => shell.clone(),
        ExecTarget::Command { .. } => PathBuf::from(std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".into())),
    };

    let env_path = std::env::join_paths(&plan.path)
        .map_err(|error| RunError::Internal(format!("joining shell activation PATH: {error}")))?;

    let status = std::process::Command::new(&shell)
        .arg("-c")
        .arg(hook)
        .envs(&plan.env)
        .env("PATH", &env_path)
        .status()
        .map_err(|e| RunError::Internal(format!("hook exec: {e}")))?;

    if !status.success() {
        let code = status.code().unwrap_or(1);
        if strict {
            std::process::exit(code);
        }
        eprintln!("warning: shell hook exited with code {code}");
    }
    Ok(())
}

/// Search for a program name in the given PATH directories.
/// Returns the absolute path if found, None otherwise.
fn resolve_in_path(cmd: &OsString, path: &[PathBuf]) -> Option<OsString> {
    let cmd_path = Path::new(cmd);
    // If the command contains a path separator, it's already a path.
    if cmd_path.components().count() > 1 {
        return None;
    }
    for dir in path {
        let candidate = dir.join(cmd);
        if candidate.is_file() {
            return Some(candidate.into_os_string());
        }
    }
    None
}

fn exec_plan(plan: &ActivationPlan) -> Result<(), RunError> {
    debug_assert!(!SIDECAR_FILENAME.is_empty());
    debug_assert!(SIDECAR_FILENAME.starts_with('.'));
    let env_path = std::env::join_paths(&plan.path)
        .map_err(|error| RunError::Internal(format!("joining shell activation PATH: {error}")))?;

    let status = match &plan.exec_target {
        ExecTarget::Interactive { shell } => {
            eprintln!("entering shell ({})", plan.env.get("CRUNCH_SHELL").unwrap_or(&String::new()));
            std::process::Command::new(shell).envs(&plan.env).env("PATH", &env_path).status()
        }
        ExecTarget::Command { argv } => {
            let (cmd, args) = argv
                .split_first()
                .ok_or_else(|| RunError::Internal("--command requires at least one argument".into()))?;
            // Resolve the program against the activation PATH, not the host
            // PATH. Command::new uses execvp which searches the *parent's*
            // PATH, but we want the composed shell PATH.
            let resolved = resolve_in_path(cmd, &plan.path).unwrap_or_else(|| cmd.clone());
            std::process::Command::new(&resolved).args(args).envs(&plan.env).env("PATH", &env_path).status()
        }
        ExecTarget::Run { shell, script } => std::process::Command::new(shell)
            .arg("-c")
            .arg(script)
            .envs(&plan.env)
            .env("PATH", &env_path)
            .status(),
    }
    .map_err(|e| RunError::Internal(format!("exec: {e}")))?;

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
}
