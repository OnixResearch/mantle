use std::collections::BTreeMap;
use std::ffi::OsString;
use std::path::{Path, PathBuf};

use crunch_shell::{
    ActivationPlan, ExecMode, ExecTarget, HostEnv, ShellSidecar, compute_activation,
};

use crate::errors::RunError;
use crate::project_build;

const SIDECAR_FILENAME: &str = ".crunch-shell.json";

/// Read the sidecar from a built shell output, snapshot the host env,
/// compute the activation plan, and exec into the result.
#[allow(clippy::too_many_arguments)]
pub fn cmd_shell(
    name: Option<&str>,
    import_paths: &[PathBuf],
    jobs: Option<u32>,
    no_substitute: bool,
    signing_key: Option<&Path>,
    trust_unsigned: bool,
    command_argv: &[OsString],
    run_script: Option<&str>,
    with_paths: &[PathBuf],
    no_hook: bool,
    strict_hooks: bool,
    output_dir: &Path,
    state_dir: &Path,
    store_prefix: &str,
    verbose: bool,
) -> Result<(), RunError> {
    // 1. Build the shell derivation.
    let out_path = build_shell_target(
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
    )?;

    // 2. Read sidecar from $out/.crunch-shell.json.
    let sidecar_path = out_path.join(SIDECAR_FILENAME);
    let sidecar_bytes = std::fs::read_to_string(&sidecar_path).map_err(|e| {
        RunError::Internal(format!(
            "missing {SIDECAR_FILENAME} in {} — was this built with mkShell? ({})",
            out_path.display(),
            e,
        ))
    })?;
    let sidecar = ShellSidecar::from_json(&sidecar_bytes)
        .map_err(|e| RunError::Internal(format!("bad sidecar: {e}")))?;

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
            return Err(RunError::Internal(format!(
                "--with path does not exist: {}",
                p.display(),
            )));
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
    if !no_hook {
        if let Some(ref hook) = plan.hook {
            exec_hook(hook, &plan, strict_hooks)?;
        }
    }

    // 9. Exec into target.
    exec_plan(&plan)
}

fn build_shell_target(
    name: Option<&str>,
    import_paths: &[PathBuf],
    jobs: Option<u32>,
    no_substitute: bool,
    signing_key: Option<&Path>,
    trust_unsigned: bool,
    output_dir: &Path,
    state_dir: &Path,
    store_prefix: &str,
    verbose: bool,
) -> Result<PathBuf, RunError> {
    let cwd = crate::current_dir_or_error()?;
    let target = crate::name_to_build_target(name);
    let resolved = project_build::resolve_project_target(&target, &cwd, import_paths)?;
    let shell_target = match &resolved.target {
        project_build::ProjectTarget::Default => project_build::ProjectTarget::DefaultShell,
        project_build::ProjectTarget::Attribute(segs) if segs.len() == 1 => {
            project_build::ProjectTarget::NamedShell(segs[0].clone())
        }
        other => other.clone(),
    };
    let expr = project_build::generate_extraction_expr(&resolved.root_file, &shell_target);
    let max_jobs = crunch_pipeline::resolve_max_jobs(jobs);
    let result = crate::build_project_expr(
        &expr,
        resolved.import_paths,
        output_dir,
        state_dir,
        store_prefix,
        verbose,
        max_jobs,
        no_substitute,
        signing_key,
        trust_unsigned,
    )?;
    crate::first_output_path(&result, output_dir, store_prefix)
        .ok_or_else(|| RunError::Internal("no outputs built for shell".into()))
}

fn snapshot_host_env() -> HostEnv {
    let env: BTreeMap<String, String> = std::env::vars().collect();
    let shell = std::env::var("SHELL")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/bin/sh"));
    HostEnv { env, shell }
}

fn exec_hook(hook: &str, plan: &ActivationPlan, strict: bool) -> Result<(), RunError> {
    let shell = match &plan.exec_target {
        ExecTarget::Interactive { shell } => shell.clone(),
        ExecTarget::Run { shell, .. } => shell.clone(),
        ExecTarget::Command { .. } => {
            PathBuf::from(std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".into()))
        }
    };

    let env_path = std::env::join_paths(&plan.path)
        .unwrap_or_default();

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
    let env_path = std::env::join_paths(&plan.path)
        .unwrap_or_default();

    let status = match &plan.exec_target {
        ExecTarget::Interactive { shell } => {
            eprintln!("entering shell ({})", plan.env.get("CRUNCH_SHELL").unwrap_or(&String::new()));
            std::process::Command::new(shell)
                .envs(&plan.env)
                .env("PATH", &env_path)
                .status()
        }
        ExecTarget::Command { argv } => {
            let (cmd, args) = argv.split_first().ok_or_else(|| {
                RunError::Internal("--command requires at least one argument".into())
            })?;
            // Resolve the program against the activation PATH, not the host
            // PATH. Command::new uses execvp which searches the *parent's*
            // PATH, but we want the composed shell PATH.
            let resolved = resolve_in_path(cmd, &plan.path).unwrap_or_else(|| cmd.clone());
            std::process::Command::new(&resolved)
                .args(args)
                .envs(&plan.env)
                .env("PATH", &env_path)
                .status()
        }
        ExecTarget::Run { shell, script } => {
            std::process::Command::new(shell)
                .arg("-c")
                .arg(script)
                .envs(&plan.env)
                .env("PATH", &env_path)
                .status()
        }
    }
    .map_err(|e| RunError::Internal(format!("exec: {e}")))?;

    std::process::exit(status.code().unwrap_or(1));
}
