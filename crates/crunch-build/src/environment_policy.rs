use std::collections::BTreeMap;

use serde::Serialize;

pub const BUILD_ENVIRONMENT_DIGEST_ALGORITHM: &str = "blake3";
pub const ENV_REJECTION_DYNAMIC_LINKER: &str = "dynamic-linker-control";
pub const ENV_REJECTION_COMPILER_WRAPPER: &str = "compiler-wrapper";
pub const ENV_REJECTION_PROXY: &str = "proxy";
pub const ENV_REJECTION_SECRET: &str = "secret-like";
pub const ENV_REJECTION_LOCALE: &str = "locale-override";
pub const ENV_REJECTION_TEMP_ROOT: &str = "temp-root-override";
const DENIED_ENV_DIAGNOSTIC: &str = "strict build environment rejects denied variable";
const ENV_NAME: &str = "name";
const UNKNOWN_ACTION_NAME: &str = "<unnamed>";
const DYNAMIC_LINKER_KEYS: [&str; 3] = ["LD_PRELOAD", "LD_LIBRARY_PATH", "LD_AUDIT"];
const COMPILER_WRAPPER_KEYS: [&str; 4] = ["RUSTC_WRAPPER", "RUSTC_WORKSPACE_WRAPPER", "CARGO", "RUSTC"];
const PROXY_KEYS: [&str; 8] = [
    "HTTP_PROXY",
    "HTTPS_PROXY",
    "ALL_PROXY",
    "NO_PROXY",
    "http_proxy",
    "https_proxy",
    "all_proxy",
    "no_proxy",
];
const LOCALE_KEYS: [&str; 3] = ["LANG", "LC_ALL", "LANGUAGE"];
const TEMP_ROOT_KEYS: [&str; 4] = ["TMPDIR", "TEMP", "TMP", "TEMPDIR"];
const SECRET_MARKERS: [&str; 6] = ["TOKEN", "SECRET", "PASSWORD", "CREDENTIAL", "PRIVATE_KEY", "AUTH"];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildEnvironmentReport {
    pub action_name: String,
    pub digest_blake3: Option<String>,
    pub variable_count: u32,
    pub rejections: Vec<BuildEnvironmentRejection>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildEnvironmentRejection {
    pub variable: String,
    pub class: String,
    pub diagnostic: String,
    pub redacted: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeniedEnvironmentVariable {
    pub action_name: String,
    pub rejection: BuildEnvironmentRejection,
    pub report: BuildEnvironmentReport,
}

#[derive(Debug, Serialize)]
struct CanonicalEnvEntry<'a> {
    key: &'a str,
    value_base64: String,
}

pub fn normalized_environment_digest_blake3(environment_vars: &BTreeMap<String, Vec<u8>>) -> String {
    assert!(!environment_vars.is_empty(), "environment digest requires at least one variable");
    let entries: Vec<_> = environment_vars
        .iter()
        .map(|(key, value)| CanonicalEnvEntry {
            key,
            value_base64: data_encoding::BASE64.encode(value),
        })
        .collect();
    let canonical = serde_json::to_vec(&entries).expect("canonical env digest serialization should not fail");
    blake3::hash(&canonical).to_hex().to_string()
}

pub fn success_report(action_name: String, environment_vars: &BTreeMap<String, Vec<u8>>) -> BuildEnvironmentReport {
    assert!(!action_name.is_empty(), "action name must not be empty");
    assert!(!environment_vars.is_empty(), "successful environment report must include variables");
    BuildEnvironmentReport {
        action_name,
        digest_blake3: Some(normalized_environment_digest_blake3(environment_vars)),
        variable_count: bounded_variable_count(environment_vars.len()),
        rejections: Vec::new(),
    }
}

pub fn denied_report(
    action_name: String,
    accepted_variable_count: usize,
    rejection: BuildEnvironmentRejection,
) -> BuildEnvironmentReport {
    assert!(!action_name.is_empty(), "action name must not be empty");
    assert!(!rejection.variable.is_empty(), "rejection variable must not be empty");
    BuildEnvironmentReport {
        action_name,
        digest_blake3: None,
        variable_count: bounded_variable_count(accepted_variable_count),
        rejections: vec![rejection],
    }
}

pub fn denied_environment_variable(
    action_name: String,
    accepted_variable_count: usize,
    variable: &str,
) -> Option<DeniedEnvironmentVariable> {
    let rejection = classify_denied_environment_variable(variable)?;
    let report = denied_report(action_name.clone(), accepted_variable_count, rejection.clone());
    Some(DeniedEnvironmentVariable {
        action_name,
        rejection,
        report,
    })
}

pub fn classify_denied_environment_variable(variable: &str) -> Option<BuildEnvironmentRejection> {
    assert!(!variable.is_empty(), "environment variable name must not be empty");
    if DYNAMIC_LINKER_KEYS.contains(&variable) {
        return Some(rejection(variable, ENV_REJECTION_DYNAMIC_LINKER, false));
    }
    if COMPILER_WRAPPER_KEYS.contains(&variable) {
        return Some(rejection(variable, ENV_REJECTION_COMPILER_WRAPPER, false));
    }
    if PROXY_KEYS.contains(&variable) {
        return Some(rejection(variable, ENV_REJECTION_PROXY, true));
    }
    if is_locale_key(variable) {
        return Some(rejection(variable, ENV_REJECTION_LOCALE, false));
    }
    if TEMP_ROOT_KEYS.contains(&variable) {
        return Some(rejection(variable, ENV_REJECTION_TEMP_ROOT, false));
    }
    if is_secret_like_key(variable) {
        return Some(rejection(variable, ENV_REJECTION_SECRET, true));
    }
    None
}

pub fn action_name_from_environment(environment_vars: &BTreeMap<String, bstr::BString>) -> String {
    environment_vars
        .get(ENV_NAME)
        .and_then(|value| std::str::from_utf8(value.as_ref()).ok())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(UNKNOWN_ACTION_NAME)
        .to_string()
}

fn rejection(variable: &str, class: &str, redacted: bool) -> BuildEnvironmentRejection {
    BuildEnvironmentRejection {
        variable: variable.to_string(),
        class: class.to_string(),
        diagnostic: DENIED_ENV_DIAGNOSTIC.to_string(),
        redacted,
    }
}

fn is_locale_key(variable: &str) -> bool {
    LOCALE_KEYS.contains(&variable) || variable.starts_with("LC_")
}

fn is_secret_like_key(variable: &str) -> bool {
    let upper = variable.to_ascii_uppercase();
    SECRET_MARKERS.iter().any(|marker| upper.contains(marker))
}

fn bounded_variable_count(count: usize) -> u32 {
    u32::try_from(count).expect("build environment variable count must fit in u32")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn digest_is_stable_for_ordered_environment_content() {
        const HEX_CHARS_PER_BYTE: usize = 2;

        let mut first = BTreeMap::new();
        first.insert("B".to_string(), b"two".to_vec());
        first.insert("A".to_string(), b"one".to_vec());
        let mut second = BTreeMap::new();
        second.insert("A".to_string(), b"one".to_vec());
        second.insert("B".to_string(), b"two".to_vec());

        let first_digest = normalized_environment_digest_blake3(&first);
        let second_digest = normalized_environment_digest_blake3(&second);

        assert_eq!(first_digest, second_digest);
        assert_eq!(first_digest.len(), blake3::OUT_LEN * HEX_CHARS_PER_BYTE);
    }

    #[test]
    fn classifier_rejects_poison_and_redacts_secrets() {
        let linker = classify_denied_environment_variable("LD_PRELOAD").expect("linker control rejected");
        let token = classify_denied_environment_variable("CARGO_REGISTRY_TOKEN").expect("token rejected");
        let proxy = classify_denied_environment_variable("HTTPS_PROXY").expect("proxy rejected");

        assert_eq!(linker.class, ENV_REJECTION_DYNAMIC_LINKER);
        assert!(!linker.redacted);
        assert_eq!(token.class, ENV_REJECTION_SECRET);
        assert!(token.redacted);
        assert_eq!(proxy.class, ENV_REJECTION_PROXY);
        assert!(proxy.redacted);
    }

    #[test]
    fn classifier_accepts_declared_build_variables() {
        assert!(classify_denied_environment_variable("CC").is_none());
        assert!(classify_denied_environment_variable("NIX_BUILD_CORES").is_none());
        assert!(classify_denied_environment_variable("SOURCE_DATE_EPOCH").is_none());
    }
}
