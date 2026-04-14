use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::ffi::OsString;
use std::path::PathBuf;

/// Protected environment variables that must never be overwritten from sidecar data.
/// Changing this list requires a code change and a test update.
const PROTECTED_VARS: &[&str] = &["HOME", "USER", "TERM", "LOGNAME", "DISPLAY", "LANG", "SHELL"];

/// Maximum number of PATH entries after composition (fixed upper bound per Tiger Style).
const MAX_PATH_ENTRIES: u32 = 4096;

/// Maximum number of env vars in a sidecar (fixed upper bound per Tiger Style).
const MAX_ENV_VARS: u32 = 4096;

/// Maximum number of warnings recorded (fixed upper bound per Tiger Style).
const MAX_WARNINGS: u32 = 256;

// ── Sidecar ──────────────────────────────────────────────────────────────────

/// Machine-readable metadata written by `mkShell` at `$out/.crunch-shell.json`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ShellSidecar {
    pub version: u32,
    #[serde(default)]
    pub env: BTreeMap<String, String>,
    #[serde(default)]
    pub path_entries: Vec<PathBuf>,
    #[serde(default)]
    pub hook: Option<String>,
}

impl ShellSidecar {
    pub fn from_json(json: &str) -> Result<Self, ShellError> {
        let sidecar: Self = serde_json::from_str(json)
            .map_err(|e| ShellError::SidecarParse(e.to_string()))?;
        if sidecar.version != 1 {
            return Err(ShellError::UnsupportedSidecarVersion {
                version: sidecar.version,
            });
        }
        if sidecar.env.len() as u32 > MAX_ENV_VARS {
            return Err(ShellError::TooManyEnvVars {
                count: sidecar.env.len() as u32,
                limit: MAX_ENV_VARS,
            });
        }
        Ok(sidecar)
    }
}

// ── Host snapshot ────────────────────────────────────────────────────────────

/// Snapshot of host environment state. No live reads — the caller captures
/// this before calling `compute_activation`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HostEnv {
    pub env: BTreeMap<String, String>,
    pub shell: PathBuf,
}

// ── Exec target ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ExecTarget {
    Interactive { shell: PathBuf },
    Command { argv: Vec<OsString> },
    Run { shell: PathBuf, script: String },
}

// ── Warnings ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ShellWarning {
    ProtectedVarSkipped { key: String },
}

// ── Activation plan ──────────────────────────────────────────────────────────

/// Complete description of the environment to activate. Performs nothing —
/// the imperative shell reads this and execs.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ActivationPlan {
    pub env: BTreeMap<String, String>,
    pub path: Vec<PathBuf>,
    pub hook: Option<String>,
    pub exec_target: ExecTarget,
    pub warnings: Vec<ShellWarning>,
}

// ── Errors ───────────────────────────────────────────────────────────────────

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum ShellError {
    #[error("unsupported sidecar version {version} (expected 1)")]
    UnsupportedSidecarVersion { version: u32 },
    #[error("failed to parse sidecar JSON: {0}")]
    SidecarParse(String),
    #[error("PATH is empty after composition")]
    EmptyPath,
    #[error("too many env vars in sidecar ({count} > {limit})")]
    TooManyEnvVars { count: u32, limit: u32 },
    #[error("too many PATH entries after composition ({count} > {limit})")]
    TooManyPathEntries { count: u32, limit: u32 },
}

// ── Exec mode input ──────────────────────────────────────────────────────────

/// CLI-provided exec mode, resolved before calling compute_activation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExecMode {
    Interactive,
    Command { argv: Vec<OsString> },
    Run { script: String },
}

// ── Core ─────────────────────────────────────────────────────────────────────

/// Compute a complete activation plan from structured inputs.
///
/// Pure function: no I/O, no subprocess, no ambient state reads.
pub fn compute_activation(
    sidecar: &ShellSidecar,
    host_env: &HostEnv,
    output_path: &str,
    with_paths: &[PathBuf],
    exec_mode: &ExecMode,
) -> Result<ActivationPlan, ShellError> {
    debug_assert_eq!(sidecar.version, 1, "caller must validate sidecar version");

    let mut env = BTreeMap::new();
    let mut warnings = Vec::new();

    // 1. Start with sidecar env, skipping protected vars.
    for (key, value) in &sidecar.env {
        if is_protected(key) {
            if warnings.len() < MAX_WARNINGS as usize {
                warnings.push(ShellWarning::ProtectedVarSkipped { key: key.clone() });
            }
        } else {
            env.insert(key.clone(), value.clone());
        }
    }

    // 2. Carry forward protected vars from host env.
    for key in PROTECTED_VARS {
        if let Some(value) = host_env.env.get(*key) {
            env.insert((*key).to_string(), value.clone());
        }
    }

    // 3. Always set CRUNCH_SHELL to the output path.
    env.insert("CRUNCH_SHELL".to_string(), output_path.to_string());

    // 4. Compose PATH: [--with bins] ++ [sidecar path_entries] ++ [host PATH].
    let host_path_entries: Vec<PathBuf> = host_env
        .env
        .get("PATH")
        .map(|p| std::env::split_paths(p).collect())
        .unwrap_or_default();

    let mut seen = std::collections::HashSet::new();
    let mut path = Vec::new();

    // --with bin dirs first
    for p in with_paths {
        let bin = p.join("bin");
        if seen.insert(bin.clone()) {
            path.push(bin);
        }
    }
    // sidecar entries
    for p in &sidecar.path_entries {
        if seen.insert(p.clone()) {
            path.push(p.clone());
        }
    }
    // host entries
    for p in &host_path_entries {
        if seen.insert(p.clone()) {
            path.push(p.clone());
        }
    }

    if path.is_empty() {
        return Err(ShellError::EmptyPath);
    }
    if path.len() as u32 > MAX_PATH_ENTRIES {
        return Err(ShellError::TooManyPathEntries {
            count: path.len() as u32,
            limit: MAX_PATH_ENTRIES,
        });
    }

    // 5. Resolve exec target.
    let exec_target = match exec_mode {
        ExecMode::Interactive => ExecTarget::Interactive {
            shell: host_env.shell.clone(),
        },
        ExecMode::Command { argv } => ExecTarget::Command { argv: argv.clone() },
        ExecMode::Run { script } => ExecTarget::Run {
            shell: host_env.shell.clone(),
            script: script.clone(),
        },
    };

    // 6. Pass hook through as data.
    let hook = sidecar.hook.clone();

    Ok(ActivationPlan {
        env,
        path,
        hook,
        exec_target,
        warnings,
    })
}

fn is_protected(key: &str) -> bool {
    PROTECTED_VARS.iter().any(|&k| k == key)
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    fn sidecar_v1(env: BTreeMap<String, String>, path_entries: Vec<PathBuf>, hook: Option<String>) -> ShellSidecar {
        ShellSidecar { version: 1, env, path_entries, hook }
    }

    fn host(env: &[(&str, &str)], shell: &str) -> HostEnv {
        HostEnv {
            env: env.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect(),
            shell: PathBuf::from(shell),
        }
    }

    #[test]
    fn env_merge_basic() {
        let sidecar = sidecar_v1(
            [("RUST_LOG".into(), "debug".into())].into(),
            vec![PathBuf::from("/store/tool/bin")],
            None,
        );
        let host_env = host(&[("PATH", "/usr/bin"), ("HOME", "/home/user")], "/bin/bash");
        let plan = compute_activation(&sidecar, &host_env, "/store/myshell", &[], &ExecMode::Interactive).unwrap();
        assert_eq!(plan.env.get("RUST_LOG").unwrap(), "debug");
        assert_eq!(plan.env.get("HOME").unwrap(), "/home/user");
        assert_eq!(plan.env.get("CRUNCH_SHELL").unwrap(), "/store/myshell");
        assert!(plan.warnings.is_empty());
    }

    #[test]
    fn protected_var_skipped_with_warning() {
        let sidecar = sidecar_v1(
            [("HOME".into(), "/override".into()), ("USER".into(), "root".into())].into(),
            vec![PathBuf::from("/bin")],
            None,
        );
        let host_env = host(&[("PATH", "/usr/bin"), ("HOME", "/home/user"), ("USER", "alice")], "/bin/bash");
        let plan = compute_activation(&sidecar, &host_env, "/store/sh", &[], &ExecMode::Interactive).unwrap();
        assert_eq!(plan.env.get("HOME").unwrap(), "/home/user");
        assert_eq!(plan.env.get("USER").unwrap(), "alice");
        assert_eq!(plan.warnings.len(), 2);
        assert!(plan.warnings.contains(&ShellWarning::ProtectedVarSkipped { key: "HOME".into() }));
        assert!(plan.warnings.contains(&ShellWarning::ProtectedVarSkipped { key: "USER".into() }));
    }

    #[test]
    fn all_protected_vars_defended() {
        for &var in PROTECTED_VARS {
            let sidecar = sidecar_v1(
                [(var.to_string(), "bad".into())].into(),
                vec![PathBuf::from("/bin")],
                None,
            );
            let host_env = host(&[("PATH", "/usr/bin"), (var, "good")], "/bin/sh");
            let plan = compute_activation(&sidecar, &host_env, "/out", &[], &ExecMode::Interactive).unwrap();
            assert_eq!(plan.env.get(var).unwrap(), "good", "protected var {var} was overwritten");
            assert!(
                plan.warnings.contains(&ShellWarning::ProtectedVarSkipped { key: var.to_string() }),
                "missing warning for {var}"
            );
        }
    }

    #[test]
    fn path_ordering_with_before_sidecar_before_host() {
        let sidecar = sidecar_v1(
            BTreeMap::new(),
            vec![PathBuf::from("/store/C/bin")],
            None,
        );
        let host_env = host(&[("PATH", "/usr/bin:/bin")], "/bin/sh");
        let with = vec![PathBuf::from("/store/A"), PathBuf::from("/store/B")];
        let plan = compute_activation(&sidecar, &host_env, "/out", &with, &ExecMode::Interactive).unwrap();
        assert_eq!(
            plan.path,
            vec![
                PathBuf::from("/store/A/bin"),
                PathBuf::from("/store/B/bin"),
                PathBuf::from("/store/C/bin"),
                PathBuf::from("/usr/bin"),
                PathBuf::from("/bin"),
            ]
        );
    }

    #[test]
    fn path_dedup_first_wins() {
        let sidecar = sidecar_v1(
            BTreeMap::new(),
            vec![PathBuf::from("/usr/bin"), PathBuf::from("/foo/bin")],
            None,
        );
        let host_env = host(&[("PATH", "/usr/bin:/bar/bin")], "/bin/sh");
        let plan = compute_activation(&sidecar, &host_env, "/out", &[], &ExecMode::Interactive).unwrap();
        // /usr/bin appears once, in sidecar position
        assert_eq!(
            plan.path,
            vec![
                PathBuf::from("/usr/bin"),
                PathBuf::from("/foo/bin"),
                PathBuf::from("/bar/bin"),
            ]
        );
    }

    #[test]
    fn empty_path_is_error() {
        let sidecar = sidecar_v1(BTreeMap::new(), vec![], None);
        let host_env = host(&[], "/bin/sh");
        let err = compute_activation(&sidecar, &host_env, "/out", &[], &ExecMode::Interactive).unwrap_err();
        assert_eq!(err, ShellError::EmptyPath);
    }

    #[test]
    fn hook_passthrough() {
        let sidecar = sidecar_v1(BTreeMap::new(), vec![PathBuf::from("/bin")], Some("echo hello".into()));
        let host_env = host(&[("PATH", "/usr/bin")], "/bin/bash");
        let plan = compute_activation(&sidecar, &host_env, "/out", &[], &ExecMode::Interactive).unwrap();
        assert_eq!(plan.hook, Some("echo hello".into()));
    }

    #[test]
    fn no_hook() {
        let sidecar = sidecar_v1(BTreeMap::new(), vec![PathBuf::from("/bin")], None);
        let host_env = host(&[("PATH", "/usr/bin")], "/bin/bash");
        let plan = compute_activation(&sidecar, &host_env, "/out", &[], &ExecMode::Interactive).unwrap();
        assert_eq!(plan.hook, None);
    }

    #[test]
    fn unsupported_version() {
        let json = r#"{"version": 2, "env": {}, "path_entries": []}"#;
        let err = ShellSidecar::from_json(json).unwrap_err();
        assert_eq!(err, ShellError::UnsupportedSidecarVersion { version: 2 });
    }

    #[test]
    fn missing_version() {
        let json = r#"{"env": {}, "path_entries": []}"#;
        let err = ShellSidecar::from_json(json).unwrap_err();
        assert!(matches!(err, ShellError::SidecarParse(_)));
    }

    #[test]
    fn version_zero_rejected() {
        let json = r#"{"version": 0, "env": {}, "path_entries": []}"#;
        let err = ShellSidecar::from_json(json).unwrap_err();
        assert_eq!(err, ShellError::UnsupportedSidecarVersion { version: 0 });
    }

    #[test]
    fn empty_sidecar_defaults() {
        let json = r#"{"version": 1}"#;
        let sidecar = ShellSidecar::from_json(json).unwrap();
        assert_eq!(sidecar.env, BTreeMap::new());
        assert_eq!(sidecar.path_entries, Vec::<PathBuf>::new());
        assert_eq!(sidecar.hook, None);
    }

    #[test]
    fn unknown_fields_ignored() {
        let json = r#"{"version": 1, "env": {}, "path_entries": [], "future_field": true}"#;
        let sidecar = ShellSidecar::from_json(json).unwrap();
        assert_eq!(sidecar.version, 1);
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
        let back: ActivationPlan = serde_json::from_str(&json).unwrap();
        assert_eq!(plan, back);
    }

    #[test]
    fn exec_target_interactive() {
        let sidecar = sidecar_v1(BTreeMap::new(), vec![PathBuf::from("/bin")], None);
        let host_env = host(&[("PATH", "/usr/bin")], "/bin/fish");
        let plan = compute_activation(&sidecar, &host_env, "/out", &[], &ExecMode::Interactive).unwrap();
        assert_eq!(plan.exec_target, ExecTarget::Interactive { shell: PathBuf::from("/bin/fish") });
    }

    #[test]
    fn exec_target_command() {
        let sidecar = sidecar_v1(BTreeMap::new(), vec![PathBuf::from("/bin")], None);
        let host_env = host(&[("PATH", "/usr/bin")], "/bin/bash");
        let argv = vec![OsString::from("make"), OsString::from("test")];
        let plan = compute_activation(&sidecar, &host_env, "/out", &[], &ExecMode::Command { argv: argv.clone() }).unwrap();
        assert_eq!(plan.exec_target, ExecTarget::Command { argv });
    }

    #[test]
    fn exec_target_run() {
        let sidecar = sidecar_v1(BTreeMap::new(), vec![PathBuf::from("/bin")], None);
        let host_env = host(&[("PATH", "/usr/bin")], "/bin/bash");
        let plan = compute_activation(&sidecar, &host_env, "/out", &[], &ExecMode::Run { script: "echo $X".into() }).unwrap();
        assert_eq!(
            plan.exec_target,
            ExecTarget::Run { shell: PathBuf::from("/bin/bash"), script: "echo $X".into() }
        );
    }

    #[test]
    fn crunch_shell_always_set() {
        let sidecar = sidecar_v1(BTreeMap::new(), vec![PathBuf::from("/bin")], None);
        let host_env = host(&[("PATH", "/usr/bin")], "/bin/sh");
        let plan = compute_activation(&sidecar, &host_env, "/store/myshell", &[], &ExecMode::Interactive).unwrap();
        assert_eq!(plan.env.get("CRUNCH_SHELL").unwrap(), "/store/myshell");
    }

    #[test]
    fn crunch_shell_overrides_sidecar() {
        let sidecar = sidecar_v1(
            [("CRUNCH_SHELL".into(), "wrong".into())].into(),
            vec![PathBuf::from("/bin")],
            None,
        );
        let host_env = host(&[("PATH", "/usr/bin")], "/bin/sh");
        let plan = compute_activation(&sidecar, &host_env, "/store/correct", &[], &ExecMode::Interactive).unwrap();
        assert_eq!(plan.env.get("CRUNCH_SHELL").unwrap(), "/store/correct");
    }

    #[test]
    fn sidecar_full_json_parse() {
        let json = r#"{
            "version": 1,
            "env": {"RUST_LOG": "debug", "PGHOST": "localhost"},
            "path_entries": ["/crunch/store/rg/bin", "/crunch/store/fd/bin"],
            "hook": "echo welcome"
        }"#;
        let sidecar = ShellSidecar::from_json(json).unwrap();
        assert_eq!(sidecar.env.len(), 2);
        assert_eq!(sidecar.path_entries.len(), 2);
        assert_eq!(sidecar.hook.as_deref(), Some("echo welcome"));
    }

    #[test]
    fn protected_var_absent_from_host_not_injected() {
        // If sidecar declares DISPLAY but host has no DISPLAY, it should be skipped (not injected).
        let sidecar = sidecar_v1(
            [("DISPLAY".into(), ":1".into())].into(),
            vec![PathBuf::from("/bin")],
            None,
        );
        let host_env = host(&[("PATH", "/usr/bin")], "/bin/sh");
        let plan = compute_activation(&sidecar, &host_env, "/out", &[], &ExecMode::Interactive).unwrap();
        assert!(!plan.env.contains_key("DISPLAY"));
        assert!(plan.warnings.contains(&ShellWarning::ProtectedVarSkipped { key: "DISPLAY".into() }));
    }
}
