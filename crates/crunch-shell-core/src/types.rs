use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;

use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;

use crate::ShellError;

const SUPPORTED_SIDECAR_VERSION: u32 = 1;
const MAX_ENV_VARS: u32 = 4096;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ShellSidecar {
    pub version: u32,
    pub env: BTreeMap<String, String>,
    pub path_entries: Vec<String>,
    pub hook: Option<String>,
}

#[derive(Deserialize)]
struct RawShellSidecar {
    version: u32,
    env: Option<BTreeMap<String, String>>,
    path_entries: Option<Vec<String>>,
    hook: Option<String>,
}

impl<'de> Deserialize<'de> for ShellSidecar {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = RawShellSidecar::deserialize(deserializer)?;
        Ok(Self {
            version: raw.version,
            env: raw.env.unwrap_or_default(),
            path_entries: raw.path_entries.unwrap_or_default(),
            hook: raw.hook,
        })
    }
}

pub fn parse_shell_sidecar_json(json: String) -> Result<ShellSidecar, ShellError> {
    let sidecar: ShellSidecar = serde_json::from_str(&json).map_err(|err| ShellError::SidecarParse(err.to_string()))?;
    validate_shell_sidecar(&sidecar)?;
    Ok(sidecar)
}

fn validate_shell_sidecar(sidecar: &ShellSidecar) -> Result<(), ShellError> {
    if sidecar.version != SUPPORTED_SIDECAR_VERSION {
        return Err(ShellError::UnsupportedSidecarVersion {
            version: sidecar.version,
        });
    }
    let env_var_count = u32_count(sidecar.env.len());
    if env_var_count > MAX_ENV_VARS {
        return Err(ShellError::TooManyEnvVars {
            count: env_var_count,
            limit: MAX_ENV_VARS,
        });
    }
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HostEnv {
    pub env: BTreeMap<String, String>,
    pub path_entries: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ShellWarning {
    ProtectedVarSkipped { key: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ActivationPlan {
    pub env: BTreeMap<String, String>,
    pub path_entries: Vec<String>,
    pub hook: Option<String>,
    pub warnings: Vec<ShellWarning>,
}

fn u32_count(count: usize) -> u32 {
    match u32::try_from(count) {
        Ok(count_u32) => count_u32,
        Err(_) => u32::MAX,
    }
}

#[cfg(test)]
mod tests {
    use alloc::collections::BTreeMap;
    use alloc::string::String;
    use alloc::string::ToString;
    use alloc::vec;
    use alloc::vec::Vec;

    use pretty_assertions::assert_eq;

    use super::parse_shell_sidecar_json;
    use crate::ShellError;

    #[test]
    fn unsupported_version() {
        let json = r#"{"version": 2, "env": {}, "path_entries": []}"#;
        let err = parse_shell_sidecar_json(json.to_string()).unwrap_err();
        assert_eq!(err, ShellError::UnsupportedSidecarVersion { version: 2 });
    }

    #[test]
    fn missing_version() {
        let json = r#"{"env": {}, "path_entries": []}"#;
        let err = parse_shell_sidecar_json(json.to_string()).unwrap_err();
        assert!(matches!(err, ShellError::SidecarParse(_)));
    }

    #[test]
    fn version_zero_rejected() {
        let json = r#"{"version": 0, "env": {}, "path_entries": []}"#;
        let err = parse_shell_sidecar_json(json.to_string()).unwrap_err();
        assert_eq!(err, ShellError::UnsupportedSidecarVersion { version: 0 });
    }

    #[test]
    fn empty_sidecar_defaults() {
        let json = r#"{"version": 1}"#;
        let sidecar = parse_shell_sidecar_json(json.to_string()).unwrap();
        assert_eq!(sidecar.env, BTreeMap::new());
        assert_eq!(sidecar.path_entries, Vec::<String>::new());
        assert_eq!(sidecar.hook, None);
    }

    #[test]
    fn unknown_fields_ignored() {
        let json = r#"{"version": 1, "env": {}, "path_entries": [], "future_field": true}"#;
        let sidecar = parse_shell_sidecar_json(json.to_string()).unwrap();
        assert_eq!(sidecar.version, 1);
    }

    #[test]
    fn sidecar_full_json_parse() {
        let json = r#"{
            "version": 1,
            "env": {"RUST_LOG": "debug", "PGHOST": "localhost"},
            "path_entries": ["/crunch/store/rg/bin", "/crunch/store/fd/bin"],
            "hook": "echo welcome"
        }"#;
        let sidecar = parse_shell_sidecar_json(json.to_string()).unwrap();
        assert_eq!(sidecar.env.len(), 2);
        assert_eq!(sidecar.path_entries, vec!["/crunch/store/rg/bin".to_string(), "/crunch/store/fd/bin".to_string()]);
        assert_eq!(sidecar.hook.as_deref(), Some("echo welcome"));
    }
}
