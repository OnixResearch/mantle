use std::env::split_paths;
use std::path::Path;
use std::path::PathBuf;

use crunch_shell_core::ActivationPlan as CoreActivationPlan;
use crunch_shell_core::HostEnv as CoreHostEnv;
use crunch_shell_core::ShellSidecar as CoreShellSidecar;

use crate::ActivationPlan;
use crate::ExecMode;
use crate::ExecTarget;
use crate::HostEnv;
use crate::ShellError;
use crate::ShellSidecar;

const HOST_PATH_ENTRY_KIND: &str = "host PATH entry";
const SIDECAR_PATH_ENTRY_KIND: &str = "sidecar path entry";
const WITH_PATH_ENTRY_KIND: &str = "--with path";

pub fn compute_activation(
    sidecar: &ShellSidecar,
    host_env: &HostEnv,
    output_path: &str,
    with_paths: &[PathBuf],
    exec_mode: &ExecMode,
) -> Result<ActivationPlan, ShellError> {
    assert!(!output_path.is_empty(), "output path must not be empty");
    assert!(output_path.starts_with('/'), "output path must be absolute");
    assert!(host_env.shell.is_absolute(), "host shell path must be absolute");

    let core_sidecar = sidecar_to_core(sidecar)?;
    let core_host_env = host_env_to_core(host_env)?;
    let core_with_paths = with_paths_to_core(with_paths)?;
    let core_plan =
        crunch_shell_core::compute_activation(core_sidecar, core_host_env, output_path.to_string(), core_with_paths)?;
    let path = path_entries_from_core(&core_plan);
    let exec_target = resolve_exec_target(exec_mode, &host_env.shell);

    Ok(ActivationPlan {
        env: core_plan.env,
        path,
        hook: core_plan.hook,
        exec_target,
        warnings: core_plan.warnings,
    })
}

fn sidecar_to_core(sidecar: &ShellSidecar) -> Result<CoreShellSidecar, ShellError> {
    Ok(CoreShellSidecar {
        version: sidecar.version,
        env: sidecar.env.clone(),
        path_entries: path_bufs_to_strings(&sidecar.path_entries, SIDECAR_PATH_ENTRY_KIND)?,
        hook: sidecar.hook.clone(),
    })
}

fn host_env_to_core(host_env: &HostEnv) -> Result<CoreHostEnv, ShellError> {
    let host_path_entries = match host_env.env.get("PATH") {
        Some(host_path) => split_paths(host_path).collect::<Vec<_>>(),
        None => Vec::new(),
    };
    Ok(CoreHostEnv {
        env: host_env.env.clone(),
        path_entries: path_bufs_to_strings(&host_path_entries, HOST_PATH_ENTRY_KIND)?,
    })
}

fn with_paths_to_core(with_paths: &[PathBuf]) -> Result<Vec<String>, ShellError> {
    let with_bin_paths = with_paths.iter().map(|root_path| root_path.join("bin")).collect::<Vec<_>>();
    path_bufs_to_strings(&with_bin_paths, WITH_PATH_ENTRY_KIND)
}

fn path_bufs_to_strings(path_entries: &[PathBuf], kind: &str) -> Result<Vec<String>, ShellError> {
    path_entries.iter().map(|path_entry| path_to_string(path_entry, kind)).collect()
}

fn path_to_string(path: &Path, kind: &str) -> Result<String, ShellError> {
    match path.to_str() {
        Some(path_str) => Ok(path_str.to_string()),
        None => Err(ShellError::NonUtf8Path {
            kind: kind.to_string(),
            value: path.as_os_str().to_string_lossy().into_owned(),
        }),
    }
}

fn path_entries_from_core(core_plan: &CoreActivationPlan) -> Vec<PathBuf> {
    core_plan.path_entries.iter().map(PathBuf::from).collect()
}

fn resolve_exec_target(exec_mode: &ExecMode, shell_path: &Path) -> ExecTarget {
    match exec_mode {
        ExecMode::Interactive => ExecTarget::Interactive {
            shell: shell_path.to_path_buf(),
        },
        ExecMode::Command { argv } => ExecTarget::Command { argv: argv.clone() },
        ExecMode::Run { script } => ExecTarget::Run {
            shell: shell_path.to_path_buf(),
            script: script.clone(),
        },
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::ffi::OsString;
    use std::path::PathBuf;

    use pretty_assertions::assert_eq;

    use super::compute_activation;
    use crate::ActivationPlan;
    use crate::ExecMode;
    use crate::ExecTarget;
    use crate::HostEnv;
    use crate::ShellError;
    use crate::ShellSidecar;

    fn sidecar_v1(env: BTreeMap<String, String>, path_entries: Vec<PathBuf>, hook: Option<String>) -> ShellSidecar {
        ShellSidecar {
            version: 1,
            env,
            path_entries,
            hook,
        }
    }

    fn host(env: &[(&str, &str)], shell: &str) -> HostEnv {
        HostEnv {
            env: env.iter().map(|(key, value)| (key.to_string(), value.to_string())).collect(),
            shell: PathBuf::from(shell),
        }
    }

    #[test]
    fn adapter_preserves_path_order_and_appends_bin() {
        let sidecar = sidecar_v1(BTreeMap::new(), vec![PathBuf::from("/store/tool/bin")], None);
        let host_env = host(&[("PATH", "/usr/bin:/bin")], "/bin/bash");
        let with_paths = vec![PathBuf::from("/store/alpha"), PathBuf::from("/store/beta")];
        let plan =
            compute_activation(&sidecar, &host_env, "/store/shell", &with_paths, &ExecMode::Interactive).unwrap();
        assert_eq!(plan.path, vec![
            PathBuf::from("/store/alpha/bin"),
            PathBuf::from("/store/beta/bin"),
            PathBuf::from("/store/tool/bin"),
            PathBuf::from("/usr/bin"),
            PathBuf::from("/bin"),
        ]);
        assert_eq!(plan.exec_target, ExecTarget::Interactive {
            shell: PathBuf::from("/bin/bash"),
        });
    }

    #[test]
    fn round_trip_serialization() {
        let sidecar = sidecar_v1(
            [("FOO".into(), "bar".into())].into(),
            vec![PathBuf::from("/store/x/bin")],
            Some("echo hi".into()),
        );
        let host_env = host(&[("PATH", "/usr/bin")], "/bin/bash");
        let plan = compute_activation(&sidecar, &host_env, "/store/shell", &[], &ExecMode::Interactive).unwrap();
        let json = serde_json::to_string(&plan).unwrap();
        let round_trip: ActivationPlan = serde_json::from_str(&json).unwrap();
        assert_eq!(plan, round_trip);
    }

    #[test]
    fn exec_target_command() {
        let sidecar = sidecar_v1(BTreeMap::new(), vec![PathBuf::from("/bin")], None);
        let host_env = host(&[("PATH", "/usr/bin")], "/bin/bash");
        let argv = vec![OsString::from("make"), OsString::from("test")];
        let plan =
            compute_activation(&sidecar, &host_env, "/out", &[], &ExecMode::Command { argv: argv.clone() }).unwrap();
        assert_eq!(plan.exec_target, ExecTarget::Command { argv });
    }

    #[test]
    fn exec_target_run() {
        let sidecar = sidecar_v1(BTreeMap::new(), vec![PathBuf::from("/bin")], None);
        let host_env = host(&[("PATH", "/usr/bin")], "/bin/bash");
        let plan = compute_activation(&sidecar, &host_env, "/out", &[], &ExecMode::Run {
            script: "echo $X".into(),
        })
        .unwrap();
        assert_eq!(plan.exec_target, ExecTarget::Run {
            shell: PathBuf::from("/bin/bash"),
            script: "echo $X".into(),
        });
    }

    #[cfg(unix)]
    #[test]
    fn non_utf8_with_path_is_rejected() {
        use std::ffi::OsString;
        use std::os::unix::ffi::OsStringExt;

        let sidecar = sidecar_v1(BTreeMap::new(), vec![PathBuf::from("/bin")], None);
        let host_env = host(&[("PATH", "/usr/bin")], "/bin/bash");
        let invalid_path = PathBuf::from(OsString::from_vec(vec![b'/', b't', 0x80]));
        let err = compute_activation(&sidecar, &host_env, "/out", &[invalid_path], &ExecMode::Interactive).unwrap_err();
        assert!(matches!(err, ShellError::NonUtf8Path { .. }));
    }
}
