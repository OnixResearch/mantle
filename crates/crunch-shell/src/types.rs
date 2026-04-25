use std::collections::BTreeMap;
use std::ffi::OsString;
use std::path::PathBuf;

pub use crunch_shell_core::ShellError;
use crunch_shell_core::ShellSidecar as CoreShellSidecar;
pub use crunch_shell_core::ShellWarning;
use crunch_shell_core::parse_shell_sidecar_json;
use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ShellSidecar {
    pub version: u32,
    pub env: BTreeMap<String, String>,
    pub path_entries: Vec<PathBuf>,
    pub hook: Option<String>,
}

impl ShellSidecar {
    pub fn from_json(json: &str) -> Result<Self, ShellError> {
        let core_sidecar = parse_shell_sidecar_json(json.to_string())?;
        Ok(Self::from_core(core_sidecar))
    }

    pub(crate) fn from_core(core_sidecar: CoreShellSidecar) -> Self {
        Self {
            version: core_sidecar.version,
            env: core_sidecar.env,
            path_entries: core_sidecar.path_entries.into_iter().map(PathBuf::from).collect(),
            hook: core_sidecar.hook,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HostEnv {
    pub env: BTreeMap<String, String>,
    pub shell: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ExecTarget {
    Interactive { shell: PathBuf },
    Command { argv: Vec<OsString> },
    Run { shell: PathBuf, script: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ActivationPlan {
    pub env: BTreeMap<String, String>,
    pub path: Vec<PathBuf>,
    pub hook: Option<String>,
    pub exec_target: ExecTarget,
    pub warnings: Vec<ShellWarning>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExecMode {
    Interactive,
    Command { argv: Vec<OsString> },
    Run { script: String },
}
