use std::collections::BTreeMap;

use serde::Serialize;

pub const BUILD_ENVIRONMENT_DIGEST_ALGORITHM: &str = "blake3";
pub const ENV_REJECTION_DYNAMIC_LINKER: &str = "dynamic-linker-control";
pub const ENV_REJECTION_COMPILER_WRAPPER: &str = "compiler-wrapper";
pub const ENV_REJECTION_PROXY: &str = "proxy";
pub const ENV_REJECTION_SECRET: &str = "secret-like";
pub const ENV_REJECTION_LOCALE: &str = "locale-override";
pub const ENV_REJECTION_TEMP_ROOT: &str = "temp-root-override";
pub const ENV_REJECTION_SEARCH_PATH: &str = "receipt-bound-path";
pub const SEARCH_PATH_DIGEST_ALGORITHM: &str = "blake3";
pub const DETERMINISM_NORMALIZATION_DIGEST_ALGORITHM: &str = "blake3";
pub const DETERMINISM_NORMALIZATION_SCHEMA: &str = "mantle-determinism-normalization-policy-v1";
pub const DETERMINISM_ENFORCEMENT_ENFORCED: &str = "enforced";
pub const DETERMINISM_ENFORCEMENT_UNSUPPORTED: &str = "unsupported";
pub const DETERMINISM_DIVERGENCE_DIVERGED: &str = "diverged";
pub const SEARCH_PATH_ENTRY_DECLARED_TOOL: &str = "declared-tool-ref";
pub const SEARCH_PATH_ENTRY_ALIAS: &str = "alias-view";
pub const SEARCH_PATH_ENTRY_HOST_INVENTORY: &str = "host-inventory-record";
const DENIED_ENV_DIAGNOSTIC: &str = "strict build environment rejects denied variable";
const SEARCH_PATH_SCHEMA: &str = "mantle-search-path-plan-v1";
const SEARCH_PATH_VARIABLE: &str = "PATH";
const MAX_SEARCH_PATH_ENTRIES: usize = 256;
const MAX_DETERMINISM_CONTROLS: usize = 32;
const HEX_CHARS_PER_BYTE: usize = 2;
const BLAKE3_HEX_CHARS: usize = blake3::OUT_LEN * HEX_CHARS_PER_BYTE;
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
    pub search_path: Option<BuildSearchPathReport>,
    pub determinism: Option<BuildDeterminismNormalizationReport>,
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildSearchPathReport {
    pub digest_blake3: Option<String>,
    pub entries: Vec<BuildSearchPathEntry>,
    pub aliases: Vec<BuildSearchPathAlias>,
    pub real_tool_refs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct BuildSearchPathEntry {
    pub path: String,
    pub kind: String,
    pub real_tool_ref: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct BuildSearchPathAlias {
    pub alias_path: String,
    pub real_tool_ref: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchPathEntryDeclaration {
    pub path: String,
    pub kind: String,
    pub real_tool_ref: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SearchPathPlanRequest {
    pub entries: Vec<SearchPathEntryDeclaration>,
    pub ambient_entries: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildDeterminismNormalizationReport {
    pub policy_digest_blake3: String,
    pub controls: Vec<BuildDeterminismControl>,
    pub unsupported_controls: Vec<String>,
    pub divergence: Option<BuildOutputDivergenceDiagnostic>,
    pub strong_claim_blocked: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct BuildDeterminismControl {
    pub surface: String,
    pub policy: String,
    pub value: String,
    pub enforcement: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildOutputDivergenceDiagnostic {
    pub status: String,
    pub surface: String,
    pub left_digest_blake3: String,
    pub right_digest_blake3: String,
    pub diagnostic: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DeterminismNormalizationRequest {
    pub controls: Vec<BuildDeterminismControl>,
    pub unsupported_controls: Vec<String>,
    pub divergence: Option<BuildOutputDivergenceDiagnostic>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeterminismNormalizationError {
    TooManyControls { count: usize, max: usize },
    EmptyControlField { field: String },
    DuplicateControl { surface: String },
    InvalidUnsupportedControl { control: String },
    InvalidDivergence { diagnostic: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SearchPathPlanError {
    TooManyEntries { count: usize, max: usize },
    AmbientEntry { path: String },
    EmptyPath,
    RelativePath { path: String },
    UnclassifiedEntry { path: String },
    EmptyRealToolRef { path: String },
    DuplicateConflict { path: String },
    AliasTargetDrift { alias_path: String },
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

pub fn plan_receipt_bound_search_path(
    request: SearchPathPlanRequest,
) -> Result<BuildSearchPathReport, SearchPathPlanError> {
    validate_search_path_request_size(&request)?;
    reject_ambient_search_path_entries(&request)?;

    let mut entries = Vec::with_capacity(request.entries.len());
    let mut aliases = Vec::new();
    let mut real_tool_refs = Vec::new();
    let mut seen_paths: BTreeMap<String, (String, String)> = BTreeMap::new();

    for declaration in request.entries {
        validate_search_path_declaration(&declaration)?;
        reject_search_path_drift(&mut seen_paths, &declaration)?;
        if declaration.kind == SEARCH_PATH_ENTRY_ALIAS {
            aliases.push(BuildSearchPathAlias {
                alias_path: declaration.path.clone(),
                real_tool_ref: declaration.real_tool_ref.clone(),
            });
        }
        if !real_tool_refs.contains(&declaration.real_tool_ref) {
            real_tool_refs.push(declaration.real_tool_ref.clone());
        }
        entries.push(BuildSearchPathEntry {
            path: declaration.path,
            kind: declaration.kind,
            real_tool_ref: declaration.real_tool_ref,
        });
    }

    let digest_blake3 = Some(search_path_digest_blake3(&entries, &aliases, &real_tool_refs));
    Ok(BuildSearchPathReport {
        digest_blake3,
        entries,
        aliases,
        real_tool_refs,
    })
}

pub fn plan_determinism_normalization(
    request: DeterminismNormalizationRequest,
) -> Result<BuildDeterminismNormalizationReport, DeterminismNormalizationError> {
    validate_determinism_request_size(&request)?;
    let mut controls = request.controls;
    controls.sort();
    validate_determinism_controls(&controls)?;
    let unsupported_controls = sorted_unique_unsupported_controls(request.unsupported_controls)?;
    if let Some(divergence) = &request.divergence {
        validate_output_divergence(divergence)?;
    }
    let strong_claim_blocked = !unsupported_controls.is_empty()
        || request
            .divergence
            .as_ref()
            .is_some_and(|divergence| divergence.status == DETERMINISM_DIVERGENCE_DIVERGED);
    let policy_digest_blake3 = determinism_policy_digest_blake3(&controls, &unsupported_controls);
    Ok(BuildDeterminismNormalizationReport {
        policy_digest_blake3,
        controls,
        unsupported_controls,
        divergence: request.divergence,
        strong_claim_blocked,
    })
}

pub fn output_divergence_diagnostic(
    surface: String,
    left_digest_blake3: String,
    right_digest_blake3: String,
) -> Option<BuildOutputDivergenceDiagnostic> {
    if left_digest_blake3 == right_digest_blake3 {
        return None;
    }
    Some(BuildOutputDivergenceDiagnostic {
        status: DETERMINISM_DIVERGENCE_DIVERGED.to_string(),
        surface: surface.clone(),
        left_digest_blake3,
        right_digest_blake3,
        diagnostic: format!("deterministic-output-divergence:{surface}"),
    })
}

pub fn denied_search_path_variable(
    action_name: String,
    accepted_variable_count: usize,
    diagnostic: String,
) -> DeniedEnvironmentVariable {
    assert!(!diagnostic.is_empty(), "search-path rejection diagnostic must not be empty");
    let rejection = BuildEnvironmentRejection {
        variable: SEARCH_PATH_VARIABLE.to_string(),
        class: ENV_REJECTION_SEARCH_PATH.to_string(),
        diagnostic,
        redacted: false,
    };
    let mut report = denied_report(action_name.clone(), accepted_variable_count, rejection.clone());
    report.search_path = Some(BuildSearchPathReport {
        digest_blake3: None,
        entries: Vec::new(),
        aliases: Vec::new(),
        real_tool_refs: Vec::new(),
    });
    DeniedEnvironmentVariable {
        action_name,
        rejection,
        report,
    }
}

fn validate_determinism_request_size(
    request: &DeterminismNormalizationRequest,
) -> Result<(), DeterminismNormalizationError> {
    let count = request.controls.len().saturating_add(request.unsupported_controls.len());
    if count > MAX_DETERMINISM_CONTROLS {
        return Err(DeterminismNormalizationError::TooManyControls {
            count,
            max: MAX_DETERMINISM_CONTROLS,
        });
    }
    Ok(())
}

fn validate_determinism_controls(controls: &[BuildDeterminismControl]) -> Result<(), DeterminismNormalizationError> {
    let mut seen = BTreeMap::new();
    for control in controls {
        validate_determinism_control(control)?;
        if seen.insert(control.surface.clone(), control.policy.clone()).is_some() {
            return Err(DeterminismNormalizationError::DuplicateControl {
                surface: control.surface.clone(),
            });
        }
    }
    Ok(())
}

fn validate_determinism_control(control: &BuildDeterminismControl) -> Result<(), DeterminismNormalizationError> {
    for (field, value) in [
        ("surface", &control.surface),
        ("policy", &control.policy),
        ("value", &control.value),
        ("enforcement", &control.enforcement),
    ] {
        if value.trim().is_empty() {
            return Err(DeterminismNormalizationError::EmptyControlField {
                field: field.to_string(),
            });
        }
    }
    Ok(())
}

fn sorted_unique_unsupported_controls(mut controls: Vec<String>) -> Result<Vec<String>, DeterminismNormalizationError> {
    for control in &controls {
        if control.trim().is_empty() {
            return Err(DeterminismNormalizationError::InvalidUnsupportedControl {
                control: control.clone(),
            });
        }
    }
    controls.sort();
    controls.dedup();
    Ok(controls)
}

fn validate_output_divergence(
    divergence: &BuildOutputDivergenceDiagnostic,
) -> Result<(), DeterminismNormalizationError> {
    for (field, value) in [
        ("status", &divergence.status),
        ("surface", &divergence.surface),
        ("left_digest_blake3", &divergence.left_digest_blake3),
        ("right_digest_blake3", &divergence.right_digest_blake3),
        ("diagnostic", &divergence.diagnostic),
    ] {
        if value.trim().is_empty() {
            return Err(DeterminismNormalizationError::InvalidDivergence {
                diagnostic: format!("{field} must not be empty"),
            });
        }
    }
    if divergence.status != DETERMINISM_DIVERGENCE_DIVERGED {
        return Err(DeterminismNormalizationError::InvalidDivergence {
            diagnostic: format!("unsupported divergence status {}", divergence.status),
        });
    }
    for (field, digest) in [
        ("left_digest_blake3", &divergence.left_digest_blake3),
        ("right_digest_blake3", &divergence.right_digest_blake3),
    ] {
        if !is_lowercase_blake3_hex(digest) {
            return Err(DeterminismNormalizationError::InvalidDivergence {
                diagnostic: format!("{field} must be lowercase BLAKE3 hex"),
            });
        }
    }
    Ok(())
}

fn is_lowercase_blake3_hex(value: &str) -> bool {
    value.len() == BLAKE3_HEX_CHARS && value.chars().all(|ch| ch.is_ascii_hexdigit() && !ch.is_ascii_uppercase())
}

#[derive(Debug, Serialize)]
struct CanonicalDeterminismPolicy<'a> {
    schema: &'static str,
    controls: &'a [BuildDeterminismControl],
    unsupported_controls: &'a [String],
}

fn determinism_policy_digest_blake3(controls: &[BuildDeterminismControl], unsupported_controls: &[String]) -> String {
    let canonical = CanonicalDeterminismPolicy {
        schema: DETERMINISM_NORMALIZATION_SCHEMA,
        controls,
        unsupported_controls,
    };
    let bytes = serde_json::to_vec(&canonical).expect("canonical determinism policy serialization should not fail");
    blake3::hash(&bytes).to_hex().to_string()
}

fn validate_search_path_request_size(request: &SearchPathPlanRequest) -> Result<(), SearchPathPlanError> {
    let count = request.entries.len().saturating_add(request.ambient_entries.len());
    if count > MAX_SEARCH_PATH_ENTRIES {
        return Err(SearchPathPlanError::TooManyEntries {
            count,
            max: MAX_SEARCH_PATH_ENTRIES,
        });
    }
    Ok(())
}

fn reject_ambient_search_path_entries(request: &SearchPathPlanRequest) -> Result<(), SearchPathPlanError> {
    if let Some(path) = request.ambient_entries.first() {
        return Err(SearchPathPlanError::AmbientEntry { path: path.clone() });
    }
    Ok(())
}

fn validate_search_path_declaration(declaration: &SearchPathEntryDeclaration) -> Result<(), SearchPathPlanError> {
    if declaration.path.is_empty() {
        return Err(SearchPathPlanError::EmptyPath);
    }
    if !declaration.path.starts_with('/') {
        return Err(SearchPathPlanError::RelativePath {
            path: declaration.path.clone(),
        });
    }
    if !is_known_search_path_kind(&declaration.kind) {
        return Err(SearchPathPlanError::UnclassifiedEntry {
            path: declaration.path.clone(),
        });
    }
    if declaration.real_tool_ref.is_empty() {
        return Err(SearchPathPlanError::EmptyRealToolRef {
            path: declaration.path.clone(),
        });
    }
    Ok(())
}

fn reject_search_path_drift(
    seen_paths: &mut BTreeMap<String, (String, String)>,
    declaration: &SearchPathEntryDeclaration,
) -> Result<(), SearchPathPlanError> {
    if let Some((previous_kind, previous_ref)) = seen_paths.get(&declaration.path) {
        if previous_ref == &declaration.real_tool_ref {
            return Ok(());
        }
        if previous_kind == SEARCH_PATH_ENTRY_ALIAS || declaration.kind == SEARCH_PATH_ENTRY_ALIAS {
            return Err(SearchPathPlanError::AliasTargetDrift {
                alias_path: declaration.path.clone(),
            });
        }
        return Err(SearchPathPlanError::DuplicateConflict {
            path: declaration.path.clone(),
        });
    }
    seen_paths.insert(declaration.path.clone(), (declaration.kind.clone(), declaration.real_tool_ref.clone()));
    Ok(())
}

fn is_known_search_path_kind(kind: &str) -> bool {
    matches!(kind, SEARCH_PATH_ENTRY_DECLARED_TOOL | SEARCH_PATH_ENTRY_ALIAS | SEARCH_PATH_ENTRY_HOST_INVENTORY)
}

#[derive(Debug, Serialize)]
struct CanonicalSearchPathPlan<'a> {
    schema: &'static str,
    entries: &'a [BuildSearchPathEntry],
    aliases: &'a [BuildSearchPathAlias],
    real_tool_refs: &'a [String],
}

fn search_path_digest_blake3(
    entries: &[BuildSearchPathEntry],
    aliases: &[BuildSearchPathAlias],
    real_tool_refs: &[String],
) -> String {
    let canonical = CanonicalSearchPathPlan {
        schema: SEARCH_PATH_SCHEMA,
        entries,
        aliases,
        real_tool_refs,
    };
    let bytes = serde_json::to_vec(&canonical).expect("canonical search-path digest serialization should not fail");
    blake3::hash(&bytes).to_hex().to_string()
}

impl std::fmt::Display for DeterminismNormalizationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TooManyControls { count, max } => write!(f, "determinism-control-count-exceeds-{max}: {count}"),
            Self::EmptyControlField { field } => write!(f, "determinism-control-field-empty:{field}"),
            Self::DuplicateControl { surface } => write!(f, "determinism-control-duplicate:{surface}"),
            Self::InvalidUnsupportedControl { control } => {
                write!(f, "determinism-unsupported-control-invalid:{control}")
            }
            Self::InvalidDivergence { diagnostic } => write!(f, "determinism-divergence-invalid:{diagnostic}"),
        }
    }
}

impl std::fmt::Display for SearchPathPlanError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TooManyEntries { count, max } => {
                write!(f, "receipt-bound-path-entry-count-exceeds-{max}: {count}")
            }
            Self::AmbientEntry { path } => write!(f, "receipt-bound-path-ambient-entry: {path}"),
            Self::EmptyPath => f.write_str("receipt-bound-path-entry-empty"),
            Self::RelativePath { path } => write!(f, "receipt-bound-path-entry-relative: {path}"),
            Self::UnclassifiedEntry { path } => write!(f, "receipt-bound-path-unclassified-entry: {path}"),
            Self::EmptyRealToolRef { path } => write!(f, "receipt-bound-path-real-tool-ref-empty: {path}"),
            Self::DuplicateConflict { path } => write!(f, "receipt-bound-path-duplicate-conflict: {path}"),
            Self::AliasTargetDrift { alias_path } => write!(f, "receipt-bound-path-alias-target-drift: {alias_path}"),
        }
    }
}

pub fn success_report(action_name: String, environment_vars: &BTreeMap<String, Vec<u8>>) -> BuildEnvironmentReport {
    assert!(!action_name.is_empty(), "action name must not be empty");
    assert!(!environment_vars.is_empty(), "successful environment report must include variables");
    BuildEnvironmentReport {
        action_name,
        digest_blake3: Some(normalized_environment_digest_blake3(environment_vars)),
        variable_count: bounded_variable_count(environment_vars.len()),
        rejections: Vec::new(),
        search_path: None,
        determinism: None,
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
        search_path: None,
        determinism: None,
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

    #[test]
    fn receipt_bound_search_path_plan_records_ordered_digest_aliases_and_real_refs() {
        const EXPECTED_ENTRY_COUNT: usize = 3;
        const EXPECTED_ALIAS_COUNT: usize = 1;
        const EXPECTED_REAL_REF_COUNT: usize = 2;
        const HEX_CHARS_PER_BYTE: usize = 2;

        let report = plan_receipt_bound_search_path(SearchPathPlanRequest {
            entries: vec![
                SearchPathEntryDeclaration {
                    path: "/mantle/store/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa-tool/bin".to_string(),
                    kind: SEARCH_PATH_ENTRY_DECLARED_TOOL.to_string(),
                    real_tool_ref: "/mantle/store/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa-tool".to_string(),
                },
                SearchPathEntryDeclaration {
                    path: "/build/aliases".to_string(),
                    kind: SEARCH_PATH_ENTRY_ALIAS.to_string(),
                    real_tool_ref: "/mantle/store/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa-tool".to_string(),
                },
                SearchPathEntryDeclaration {
                    path: "/host-tools/gcc/bin".to_string(),
                    kind: SEARCH_PATH_ENTRY_HOST_INVENTORY.to_string(),
                    real_tool_ref: "host-tool:gcc:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"
                        .to_string(),
                },
            ],
            ambient_entries: Vec::new(),
        })
        .expect("receipt-bound path plan");
        let repeated = plan_receipt_bound_search_path(SearchPathPlanRequest {
            entries: report
                .entries
                .iter()
                .map(|entry| SearchPathEntryDeclaration {
                    path: entry.path.clone(),
                    kind: entry.kind.clone(),
                    real_tool_ref: entry.real_tool_ref.clone(),
                })
                .collect(),
            ambient_entries: Vec::new(),
        })
        .expect("repeated path plan");

        assert_eq!(report.entries.len(), EXPECTED_ENTRY_COUNT);
        assert_eq!(report.aliases.len(), EXPECTED_ALIAS_COUNT);
        assert_eq!(report.real_tool_refs.len(), EXPECTED_REAL_REF_COUNT);
        assert_eq!(report.digest_blake3, repeated.digest_blake3);
        assert!(
            report
                .digest_blake3
                .as_deref()
                .is_some_and(|digest| digest.len() == blake3::OUT_LEN * HEX_CHARS_PER_BYTE)
        );
    }

    #[test]
    fn receipt_bound_search_path_plan_rejects_ambient_entries_and_alias_drift() {
        let ambient = plan_receipt_bound_search_path(SearchPathPlanRequest {
            entries: Vec::new(),
            ambient_entries: vec!["/tmp/poison/bin".to_string()],
        })
        .expect_err("ambient path must fail");
        assert!(matches!(ambient, SearchPathPlanError::AmbientEntry { path } if path == "/tmp/poison/bin"));

        let alias_drift = plan_receipt_bound_search_path(SearchPathPlanRequest {
            entries: vec![
                SearchPathEntryDeclaration {
                    path: "/build/aliases".to_string(),
                    kind: SEARCH_PATH_ENTRY_ALIAS.to_string(),
                    real_tool_ref: "/mantle/store/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa-tool".to_string(),
                },
                SearchPathEntryDeclaration {
                    path: "/build/aliases".to_string(),
                    kind: SEARCH_PATH_ENTRY_ALIAS.to_string(),
                    real_tool_ref: "/mantle/store/bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb-tool".to_string(),
                },
            ],
            ambient_entries: Vec::new(),
        })
        .expect_err("alias target drift must fail");
        assert!(
            matches!(alias_drift, SearchPathPlanError::AliasTargetDrift { alias_path } if alias_path == "/build/aliases")
        );
    }

    #[test]
    fn determinism_normalization_plan_records_stable_policy_digest() {
        const CONTROL_COUNT: usize = 2;
        const HEX_CHARS_PER_BYTE: usize = 2;

        let request = DeterminismNormalizationRequest {
            controls: vec![
                BuildDeterminismControl {
                    surface: "timezone".to_string(),
                    policy: "fixed-env".to_string(),
                    value: "TZ=UTC".to_string(),
                    enforcement: DETERMINISM_ENFORCEMENT_ENFORCED.to_string(),
                },
                BuildDeterminismControl {
                    surface: "time".to_string(),
                    policy: "source-date-epoch".to_string(),
                    value: "SOURCE_DATE_EPOCH=1".to_string(),
                    enforcement: DETERMINISM_ENFORCEMENT_ENFORCED.to_string(),
                },
            ],
            unsupported_controls: Vec::new(),
            divergence: None,
        };
        let first = plan_determinism_normalization(request.clone()).expect("determinism policy");
        let repeated = plan_determinism_normalization(request).expect("repeat determinism policy");

        assert_eq!(first.controls.len(), CONTROL_COUNT);
        assert_eq!(first.policy_digest_blake3, repeated.policy_digest_blake3);
        assert_eq!(first.policy_digest_blake3.len(), blake3::OUT_LEN * HEX_CHARS_PER_BYTE);
        assert!(!first.strong_claim_blocked);
        assert!(first.unsupported_controls.is_empty());
    }

    #[test]
    fn determinism_normalization_plan_blocks_unsupported_controls_and_divergence() {
        let divergence =
            output_divergence_diagnostic("out".to_string(), DIGEST_LEFT.to_string(), DIGEST_RIGHT.to_string())
                .expect("different output digests diverge");
        let report = plan_determinism_normalization(DeterminismNormalizationRequest {
            controls: vec![BuildDeterminismControl {
                surface: "umask".to_string(),
                policy: "fixed".to_string(),
                value: "0022".to_string(),
                enforcement: DETERMINISM_ENFORCEMENT_UNSUPPORTED.to_string(),
            }],
            unsupported_controls: vec!["umask".to_string()],
            divergence: Some(divergence),
        })
        .expect("blocked determinism policy report");

        assert!(report.strong_claim_blocked);
        assert_eq!(report.unsupported_controls, vec!["umask".to_string()]);
        assert_eq!(
            report.divergence.as_ref().map(|value| value.status.as_str()),
            Some(DETERMINISM_DIVERGENCE_DIVERGED)
        );
        assert!(
            output_divergence_diagnostic("out".to_string(), DIGEST_LEFT.to_string(), DIGEST_LEFT.to_string()).is_none()
        );
    }

    const DIGEST_LEFT: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    const DIGEST_RIGHT: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
}
