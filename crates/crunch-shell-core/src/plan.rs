use alloc::collections::BTreeMap;
use alloc::collections::BTreeSet;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;

use crate::ActivationPlan;
use crate::HostEnv;
use crate::ShellError;
use crate::ShellSidecar;
use crate::ShellWarning;

const PROTECTED_VARS: &[&str] = &["HOME", "USER", "TERM", "LOGNAME", "DISPLAY", "LANG", "SHELL"];
const MAX_PATH_ENTRIES: u32 = 4096;
const MAX_ENV_VARS: u32 = 4096;
const MAX_WARNINGS: u32 = 256;

pub fn compute_activation(
    sidecar: ShellSidecar,
    host_env: HostEnv,
    output_path: String,
    with_paths: Vec<String>,
) -> Result<ActivationPlan, ShellError> {
    debug_assert_eq!(sidecar.version, 1, "caller must validate sidecar version");
    assert!(!output_path.is_empty(), "output path must not be empty");
    assert!(output_path.starts_with('/'), "output path must be absolute");
    assert!(entries_are_non_empty(&with_paths), "with path entries must not be empty");

    let (env, warnings) = build_activation_env(&sidecar, &host_env, &output_path)?;
    let path_entries = compose_activation_path(&with_paths, &sidecar.path_entries, &host_env.path_entries)?;
    let hook = sidecar.hook.clone();

    Ok(ActivationPlan {
        env,
        path_entries,
        hook,
        warnings,
    })
}

fn build_activation_env(
    sidecar: &ShellSidecar,
    host_env: &HostEnv,
    output_path: &str,
) -> Result<(BTreeMap<String, String>, Vec<ShellWarning>), ShellError> {
    let mut env = BTreeMap::new();
    let mut warnings = Vec::with_capacity(usize_limit_from_u32(MAX_WARNINGS));

    assert!(output_path.starts_with('/'), "output path must be absolute");
    assert!(warnings.is_empty(), "warnings must start empty");

    for (key, value) in &sidecar.env {
        if is_protected(key) {
            if warning_slot_available(warnings.len()) {
                warnings.push(ShellWarning::ProtectedVarSkipped { key: key.clone() });
            }
            continue;
        }
        checked_insert_env(&mut env, key.clone(), value.clone())?;
    }

    for protected_key in PROTECTED_VARS {
        if let Some(value) = host_env.env.get(*protected_key) {
            checked_insert_env(&mut env, (*protected_key).to_string(), value.clone())?;
        }
    }

    checked_insert_env(&mut env, "CRUNCH_SHELL".to_string(), output_path.to_string())?;
    Ok((env, warnings))
}

fn compose_activation_path(
    with_paths: &[String],
    sidecar_path_entries: &[String],
    host_path_entries: &[String],
) -> Result<Vec<String>, ShellError> {
    let path_entry_count_estimate =
        with_paths.len().saturating_add(sidecar_path_entries.len()).saturating_add(host_path_entries.len());
    let mut seen = BTreeSet::new();
    let mut path_entries = Vec::with_capacity(path_entry_count_estimate);

    assert!(path_entries.is_empty(), "activation path must start empty");
    assert!(seen.is_empty(), "path dedup set must start empty");

    append_unique_entries(&mut path_entries, &mut seen, with_paths);
    append_unique_entries(&mut path_entries, &mut seen, sidecar_path_entries);
    append_unique_entries(&mut path_entries, &mut seen, host_path_entries);

    if path_entries.is_empty() {
        return Err(ShellError::EmptyPath);
    }
    let path_entry_count = u32_count(path_entries.len());
    if path_entry_count > MAX_PATH_ENTRIES {
        return Err(ShellError::TooManyPathEntries {
            count: path_entry_count,
            limit: MAX_PATH_ENTRIES,
        });
    }
    Ok(path_entries)
}

fn append_unique_entries(path_entries: &mut Vec<String>, seen: &mut BTreeSet<String>, source_entries: &[String]) {
    for source_entry in source_entries {
        if seen.insert(source_entry.clone()) {
            path_entries.push(source_entry.clone());
        }
    }
}

fn is_protected(key: &str) -> bool {
    PROTECTED_VARS.contains(&key)
}

fn checked_insert_env(env: &mut BTreeMap<String, String>, key: String, value: String) -> Result<(), ShellError> {
    let is_new_key = !env.contains_key(&key);
    if is_new_key && env.len() >= usize_limit_from_u32(MAX_ENV_VARS) {
        return Err(ShellError::TooManyEnvVars {
            count: u32_count(env.len()).saturating_add(1),
            limit: MAX_ENV_VARS,
        });
    }
    env.insert(key, value);
    Ok(())
}

fn warning_slot_available(current_warning_count: usize) -> bool {
    current_warning_count < usize_limit_from_u32(MAX_WARNINGS)
}

fn entries_are_non_empty(entries: &[String]) -> bool {
    entries.iter().all(|entry| !entry.is_empty())
}

fn usize_limit_from_u32(limit: u32) -> usize {
    match usize::try_from(limit) {
        Ok(limit_usize) => limit_usize,
        Err(_) => usize::MAX,
    }
}

fn u32_count(count: usize) -> u32 {
    u32::try_from(count).unwrap_or(u32::MAX)
}

#[cfg(test)]
mod tests {
    use alloc::collections::BTreeMap;
    use alloc::string::String;
    use alloc::string::ToString;
    use alloc::vec;
    use alloc::vec::Vec;

    use pretty_assertions::assert_eq;

    use super::compute_activation;
    use crate::HostEnv;
    use crate::ShellError;
    use crate::ShellSidecar;
    use crate::ShellWarning;

    fn sidecar_v1(env: BTreeMap<String, String>, path_entries: Vec<String>, hook: Option<String>) -> ShellSidecar {
        ShellSidecar {
            version: 1,
            env,
            path_entries,
            hook,
        }
    }

    fn host(env: &[(&str, &str)], path_entries: &[&str]) -> HostEnv {
        HostEnv {
            env: env.iter().map(|(key, value)| (key.to_string(), value.to_string())).collect(),
            path_entries: path_entries.iter().map(|entry| entry.to_string()).collect(),
        }
    }

    #[test]
    fn env_merge_basic() {
        let sidecar = sidecar_v1([("RUST_LOG".into(), "debug".into())].into(), vec!["/store/tool/bin".into()], None);
        let host_env = host(&[("PATH", "/usr/bin"), ("HOME", "/home/user")], &["/usr/bin"]);
        let plan = compute_activation(sidecar, host_env, "/store/myshell".to_string(), Vec::new()).unwrap();
        assert_eq!(plan.env.get("RUST_LOG").unwrap(), "debug");
        assert_eq!(plan.env.get("HOME").unwrap(), "/home/user");
        assert_eq!(plan.env.get("CRUNCH_SHELL").unwrap(), "/store/myshell");
        assert!(plan.warnings.is_empty());
    }

    #[test]
    fn protected_var_skipped_with_warning() {
        let sidecar = sidecar_v1(
            [("HOME".into(), "/override".into()), ("USER".into(), "root".into())].into(),
            vec!["/bin".into()],
            None,
        );
        let host_env = host(&[("PATH", "/usr/bin"), ("HOME", "/home/user"), ("USER", "alice")], &["/usr/bin"]);
        let plan = compute_activation(sidecar, host_env, "/store/sh".to_string(), Vec::new()).unwrap();
        assert_eq!(plan.env.get("HOME").unwrap(), "/home/user");
        assert_eq!(plan.env.get("USER").unwrap(), "alice");
        assert_eq!(plan.warnings.len(), 2);
        assert!(plan.warnings.contains(&ShellWarning::ProtectedVarSkipped { key: "HOME".into() }));
        assert!(plan.warnings.contains(&ShellWarning::ProtectedVarSkipped { key: "USER".into() }));
    }

    #[test]
    fn all_protected_vars_defended() {
        for &protected_var in super::PROTECTED_VARS {
            let sidecar = sidecar_v1([(protected_var.to_string(), "bad".into())].into(), vec!["/bin".into()], None);
            let host_env = host(&[("PATH", "/usr/bin"), (protected_var, "good")], &["/usr/bin"]);
            let plan = compute_activation(sidecar, host_env, "/out".to_string(), Vec::new()).unwrap();
            assert_eq!(plan.env.get(protected_var).unwrap(), "good", "protected var {protected_var} was overwritten");
            assert!(
                plan.warnings.contains(&ShellWarning::ProtectedVarSkipped {
                    key: protected_var.to_string(),
                }),
                "missing warning for {protected_var}"
            );
        }
    }

    #[test]
    fn path_ordering_with_before_sidecar_before_host() {
        let sidecar = sidecar_v1(BTreeMap::new(), vec!["/store/C/bin".into()], None);
        let host_env = host(&[("PATH", "/usr/bin:/bin")], &["/usr/bin", "/bin"]);
        let with_paths = vec!["/store/A/bin".into(), "/store/B/bin".into()];
        let plan = compute_activation(sidecar, host_env, "/out".to_string(), with_paths).unwrap();
        assert_eq!(plan.path_entries, vec![
            "/store/A/bin".to_string(),
            "/store/B/bin".to_string(),
            "/store/C/bin".to_string(),
            "/usr/bin".to_string(),
            "/bin".to_string(),
        ]);
    }

    #[test]
    fn path_dedup_first_wins() {
        let sidecar = sidecar_v1(BTreeMap::new(), vec!["/usr/bin".into(), "/foo/bin".into()], None);
        let host_env = host(&[("PATH", "/usr/bin:/bar/bin")], &["/usr/bin", "/bar/bin"]);
        let plan = compute_activation(sidecar, host_env, "/out".to_string(), Vec::new()).unwrap();
        assert_eq!(plan.path_entries, vec!["/usr/bin".to_string(), "/foo/bin".to_string(), "/bar/bin".to_string(),]);
    }

    #[test]
    fn empty_path_is_error() {
        let sidecar = sidecar_v1(BTreeMap::new(), vec![], None);
        let host_env = host(&[], &[]);
        let err = compute_activation(sidecar, host_env, "/out".to_string(), Vec::new()).unwrap_err();
        assert_eq!(err, ShellError::EmptyPath);
    }

    #[test]
    fn hook_passthrough() {
        let sidecar = sidecar_v1(BTreeMap::new(), vec!["/bin".into()], Some("echo hello".into()));
        let host_env = host(&[("PATH", "/usr/bin")], &["/usr/bin"]);
        let plan = compute_activation(sidecar, host_env, "/out".to_string(), Vec::new()).unwrap();
        assert_eq!(plan.hook, Some("echo hello".into()));
    }

    #[test]
    fn no_hook() {
        let sidecar = sidecar_v1(BTreeMap::new(), vec!["/bin".into()], None);
        let host_env = host(&[("PATH", "/usr/bin")], &["/usr/bin"]);
        let plan = compute_activation(sidecar, host_env, "/out".to_string(), Vec::new()).unwrap();
        assert_eq!(plan.hook, None);
    }

    #[test]
    fn crunch_shell_always_set() {
        let sidecar = sidecar_v1(BTreeMap::new(), vec!["/bin".into()], None);
        let host_env = host(&[("PATH", "/usr/bin")], &["/usr/bin"]);
        let plan = compute_activation(sidecar, host_env, "/store/myshell".to_string(), Vec::new()).unwrap();
        assert_eq!(plan.env.get("CRUNCH_SHELL").unwrap(), "/store/myshell");
    }

    #[test]
    fn crunch_shell_overrides_sidecar() {
        let sidecar = sidecar_v1([("CRUNCH_SHELL".into(), "wrong".into())].into(), vec!["/bin".into()], None);
        let host_env = host(&[("PATH", "/usr/bin")], &["/usr/bin"]);
        let plan = compute_activation(sidecar, host_env, "/store/correct".to_string(), Vec::new()).unwrap();
        assert_eq!(plan.env.get("CRUNCH_SHELL").unwrap(), "/store/correct");
    }

    #[test]
    fn protected_var_absent_from_host_not_injected() {
        let sidecar = sidecar_v1([("DISPLAY".into(), ":1".into())].into(), vec!["/bin".into()], None);
        let host_env = host(&[("PATH", "/usr/bin")], &["/usr/bin"]);
        let plan = compute_activation(sidecar, host_env, "/out".to_string(), Vec::new()).unwrap();
        assert!(!plan.env.contains_key("DISPLAY"));
        assert!(plan.warnings.contains(&ShellWarning::ProtectedVarSkipped { key: "DISPLAY".into() }));
    }
}
