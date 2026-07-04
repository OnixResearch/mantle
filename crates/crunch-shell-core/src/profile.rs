use alloc::collections::BTreeMap;
use alloc::collections::BTreeSet;
use alloc::format;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec;
use alloc::vec::Vec;

use crate::ShellSidecar;

const SIDECAR_VERSION: u32 = 1;
const MAX_PROFILE_NAME_BYTES: usize = 64;
const MAX_SHELL_PROFILES: u32 = 256;
const MAX_PROFILE_BUILD_INPUTS: u32 = 1024;
const MAX_PROFILE_ENV_ENTRIES: u32 = 1024;
const MAX_PROFILE_PATH_ENTRIES: u32 = 1024;
const BYTES_PER_KIBIBYTE: usize = 1024;
const MAX_PROFILE_HOOK_KIBIBYTES: usize = 16;
const MAX_PROFILE_HOOK_BYTES: usize = MAX_PROFILE_HOOK_KIBIBYTES * BYTES_PER_KIBIBYTE;
const CLAIM_SHELL_ACTIVATION: &str = "shell-activation";
const DEFAULT_PROFILE_NAME: &str = "default";
const DEV_PROFILE_NAME: &str = "dev";

pub const NAMED_SHELL_PROFILE_NON_CLAIM: &str = "shell profile activation is convenience evidence only; it does not prove builds, tests, services, deployability, or release reproducibility";
pub const DEV_SHELL_DECOUPLING_NON_CLAIM: &str = "dev shell planning must not mutate generated files, lockfiles, project manifests, release evidence, or package build identity";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShellProfileEnvEntry {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShellProfileDeclaration {
    pub name: String,
    pub build_inputs: Vec<String>,
    pub env: Vec<ShellProfileEnvEntry>,
    pub path_entries: Vec<String>,
    pub hook: Option<String>,
    pub services: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NamedShellProfiles {
    pub profiles: Vec<ShellProfileDeclaration>,
    pub default_profile: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShellProfileSelectionRequest {
    pub profiles: NamedShellProfiles,
    pub requested_profile: Option<String>,
    pub requested_claims: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShellProfilePlan {
    pub selected_profile: String,
    pub build_inputs: Vec<String>,
    pub sidecar: ShellSidecar,
    pub non_claims: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShellProfileDiagnostic {
    pub code: &'static str,
    pub profile: Option<String>,
    pub message: String,
}

pub fn plan_named_shell_profile(
    request: ShellProfileSelectionRequest,
) -> Result<ShellProfilePlan, Vec<ShellProfileDiagnostic>> {
    assert!(request.profiles.profiles.len() <= usize_limit_from_u32(MAX_SHELL_PROFILES));
    assert!(request.requested_claims.len() <= usize_limit_from_u32(MAX_PROFILE_ENV_ENTRIES));

    let mut diagnostics = validate_profile_set(&request.profiles);
    diagnostics.extend(validate_requested_claims(&request.requested_claims));
    let selected_profile =
        select_profile_name(&request.profiles, request.requested_profile.as_deref(), &mut diagnostics);
    if !diagnostics.is_empty() {
        return Err(diagnostics);
    }

    let selected_name = selected_profile.expect("validated profile selection must be present");
    let profile = request
        .profiles
        .profiles
        .iter()
        .find(|profile| profile.name == selected_name)
        .expect("selected profile exists after validation");
    let sidecar = profile_to_sidecar(profile);
    Ok(ShellProfilePlan {
        selected_profile: selected_name,
        build_inputs: profile.build_inputs.clone(),
        sidecar,
        non_claims: vec![
            NAMED_SHELL_PROFILE_NON_CLAIM.to_string(),
            DEV_SHELL_DECOUPLING_NON_CLAIM.to_string(),
        ],
    })
}

fn validate_profile_set(profiles: &NamedShellProfiles) -> Vec<ShellProfileDiagnostic> {
    let mut diagnostics = Vec::new();
    let profile_count = u32_count(profiles.profiles.len());
    if profile_count > MAX_SHELL_PROFILES {
        diagnostics.push(diagnostic(
            "too-many-profiles",
            None,
            format!("too many shell profiles: {profile_count} > {MAX_SHELL_PROFILES}"),
        ));
    }

    let mut seen = BTreeSet::new();
    for profile in &profiles.profiles {
        if !seen.insert(profile.name.clone()) {
            diagnostics.push(diagnostic(
                "duplicate-profile",
                Some(profile.name.clone()),
                format!("duplicate shell profile `{}`", profile.name),
            ));
        }
        diagnostics.extend(validate_profile(profile));
    }

    if let Some(default_profile) = &profiles.default_profile {
        validate_profile_name(default_profile, "invalid-default-profile", &mut diagnostics);
        if !profiles.profiles.iter().any(|profile| profile.name == *default_profile) {
            diagnostics.push(diagnostic(
                "missing-default-profile",
                Some(default_profile.clone()),
                format!("default shell profile `{default_profile}` is not declared"),
            ));
        }
    }
    diagnostics
}

fn validate_profile(profile: &ShellProfileDeclaration) -> Vec<ShellProfileDiagnostic> {
    let mut diagnostics = Vec::new();
    validate_profile_name(&profile.name, "invalid-profile-name", &mut diagnostics);
    validate_count(
        profile.build_inputs.len(),
        MAX_PROFILE_BUILD_INPUTS,
        "too-many-build-inputs",
        &profile.name,
        "build inputs",
        &mut diagnostics,
    );
    validate_count(
        profile.env.len(),
        MAX_PROFILE_ENV_ENTRIES,
        "too-many-env-entries",
        &profile.name,
        "environment entries",
        &mut diagnostics,
    );
    validate_count(
        profile.path_entries.len(),
        MAX_PROFILE_PATH_ENTRIES,
        "too-many-path-entries",
        &profile.name,
        "PATH entries",
        &mut diagnostics,
    );

    for build_input in &profile.build_inputs {
        if build_input.is_empty() {
            diagnostics.push(diagnostic(
                "empty-build-input",
                Some(profile.name.clone()),
                "shell profile build input must not be empty".to_string(),
            ));
        }
    }
    diagnostics.extend(validate_env_entries(profile));
    diagnostics.extend(validate_path_entries(profile));
    diagnostics.extend(validate_hook(profile));
    if !profile.services.is_empty() {
        diagnostics.push(diagnostic(
            "services-not-supported",
            Some(profile.name.clone()),
            "service lifecycle declarations are not part of Mantle shell profiles".to_string(),
        ));
    }
    diagnostics
}

fn validate_profile_name(name: &str, code: &'static str, diagnostics: &mut Vec<ShellProfileDiagnostic>) {
    if name.is_empty() {
        diagnostics.push(diagnostic(code, None, "shell profile name must not be empty".to_string()));
        return;
    }
    if name.len() > MAX_PROFILE_NAME_BYTES {
        diagnostics.push(diagnostic(
            code,
            Some(name.to_string()),
            format!("shell profile name `{name}` exceeds {MAX_PROFILE_NAME_BYTES} bytes"),
        ));
    }
    let first_valid = name.bytes().next().is_some_and(|byte| byte.is_ascii_alphanumeric() || byte == b'_');
    let all_valid = name.bytes().all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'.'));
    if !first_valid || !all_valid {
        diagnostics.push(diagnostic(
            code,
            Some(name.to_string()),
            format!("shell profile name `{name}` must use ASCII letters, digits, `_`, `-`, or `.` and start with a letter, digit, or `_`"),
        ));
    }
}

fn validate_count(
    count: usize,
    limit: u32,
    code: &'static str,
    profile_name: &str,
    label: &str,
    diagnostics: &mut Vec<ShellProfileDiagnostic>,
) {
    let count_u32 = u32_count(count);
    if count_u32 > limit {
        diagnostics.push(diagnostic(
            code,
            Some(profile_name.to_string()),
            format!("shell profile `{profile_name}` has too many {label}: {count_u32} > {limit}"),
        ));
    }
}

fn validate_env_entries(profile: &ShellProfileDeclaration) -> Vec<ShellProfileDiagnostic> {
    let mut diagnostics = Vec::new();
    let mut seen = BTreeSet::new();
    for entry in &profile.env {
        if entry.key.is_empty() {
            diagnostics.push(diagnostic(
                "empty-env-key",
                Some(profile.name.clone()),
                "shell profile env key must not be empty".to_string(),
            ));
            continue;
        }
        let normalized = normalize_env_key(&entry.key);
        if !seen.insert(normalized.clone()) {
            diagnostics.push(diagnostic(
                "duplicate-env-key",
                Some(profile.name.clone()),
                format!("shell profile `{}` has duplicate env key after normalization: {normalized}", profile.name),
            ));
        }
    }
    diagnostics
}

fn validate_path_entries(profile: &ShellProfileDeclaration) -> Vec<ShellProfileDiagnostic> {
    let mut diagnostics = Vec::new();
    for entry in &profile.path_entries {
        if entry.is_empty() {
            diagnostics.push(diagnostic(
                "empty-path-entry",
                Some(profile.name.clone()),
                "shell profile path entry must not be empty".to_string(),
            ));
            continue;
        }
        if !entry.starts_with('/') {
            diagnostics.push(diagnostic(
                "unsupported-path-entry",
                Some(profile.name.clone()),
                format!("shell profile path entry `{entry}` must be absolute before activation"),
            ));
        }
        if entry.split('/').any(|component| component == "..") {
            diagnostics.push(diagnostic(
                "unsupported-path-entry",
                Some(profile.name.clone()),
                format!("shell profile path entry `{entry}` must not contain `..`"),
            ));
        }
    }
    diagnostics
}

fn validate_hook(profile: &ShellProfileDeclaration) -> Vec<ShellProfileDiagnostic> {
    let mut diagnostics = Vec::new();
    if let Some(hook) = &profile.hook {
        if hook.len() > MAX_PROFILE_HOOK_BYTES {
            diagnostics.push(diagnostic(
                "hook-too-large",
                Some(profile.name.clone()),
                format!("shell profile hook exceeds {MAX_PROFILE_HOOK_BYTES} bytes"),
            ));
        }
    }
    diagnostics
}

fn validate_requested_claims(claims: &[String]) -> Vec<ShellProfileDiagnostic> {
    let mut diagnostics = Vec::new();
    for claim in claims {
        if claim != CLAIM_SHELL_ACTIVATION {
            diagnostics.push(diagnostic(
                "unsupported-shell-claim",
                None,
                format!("shell activation cannot claim `{claim}` without separate evidence"),
            ));
        }
    }
    diagnostics
}

fn select_profile_name(
    profiles: &NamedShellProfiles,
    requested_profile: Option<&str>,
    diagnostics: &mut Vec<ShellProfileDiagnostic>,
) -> Option<String> {
    if let Some(requested) = requested_profile {
        if profiles.profiles.iter().any(|profile| profile.name == requested) {
            return Some(requested.to_string());
        }
        diagnostics.push(diagnostic(
            "missing-profile",
            Some(requested.to_string()),
            format!("requested shell profile `{requested}` is not declared"),
        ));
        return None;
    }

    if let Some(default_profile) = &profiles.default_profile {
        return Some(default_profile.clone());
    }
    if profiles.profiles.iter().any(|profile| profile.name == DEV_PROFILE_NAME) {
        return Some(DEV_PROFILE_NAME.to_string());
    }
    if profiles.profiles.iter().any(|profile| profile.name == DEFAULT_PROFILE_NAME) {
        return Some(DEFAULT_PROFILE_NAME.to_string());
    }
    if profiles.profiles.len() == 1 {
        return profiles.profiles.first().map(|profile| profile.name.clone());
    }

    diagnostics.push(diagnostic(
        "ambiguous-default-profile",
        None,
        "shell profile default is ambiguous; declare `default`, `dev`, or an explicit default profile".to_string(),
    ));
    None
}

fn profile_to_sidecar(profile: &ShellProfileDeclaration) -> ShellSidecar {
    let env = profile.env.iter().map(|entry| (entry.key.clone(), entry.value.clone())).collect::<BTreeMap<_, _>>();
    ShellSidecar {
        version: SIDECAR_VERSION,
        env,
        path_entries: profile.path_entries.clone(),
        hook: profile.hook.clone(),
    }
}

fn normalize_env_key(key: &str) -> String {
    key.bytes().map(|byte| byte.to_ascii_uppercase() as char).collect()
}

fn diagnostic(code: &'static str, profile: Option<String>, message: String) -> ShellProfileDiagnostic {
    assert!(!code.is_empty(), "diagnostic code must not be empty");
    assert!(!message.is_empty(), "diagnostic message must not be empty");
    ShellProfileDiagnostic { code, profile, message }
}

fn u32_count(count: usize) -> u32 {
    u32::try_from(count).unwrap_or(u32::MAX)
}

fn usize_limit_from_u32(limit: u32) -> usize {
    match usize::try_from(limit) {
        Ok(limit_usize) => limit_usize,
        Err(_) => usize::MAX,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn env(key: &str, value: &str) -> ShellProfileEnvEntry {
        ShellProfileEnvEntry {
            key: key.to_string(),
            value: value.to_string(),
        }
    }

    fn profile(name: &str) -> ShellProfileDeclaration {
        ShellProfileDeclaration {
            name: name.to_string(),
            build_inputs: Vec::new(),
            env: Vec::new(),
            path_entries: vec!["/tools/bin".to_string()],
            hook: None,
            services: Vec::new(),
        }
    }

    fn request(
        profiles: Vec<ShellProfileDeclaration>,
        requested_profile: Option<&str>,
    ) -> ShellProfileSelectionRequest {
        ShellProfileSelectionRequest {
            profiles: NamedShellProfiles {
                profiles,
                default_profile: None,
            },
            requested_profile: requested_profile.map(ToString::to_string),
            requested_claims: vec![CLAIM_SHELL_ACTIVATION.to_string()],
        }
    }

    fn diagnostic_codes(errs: &[ShellProfileDiagnostic]) -> Vec<&'static str> {
        errs.iter().map(|diag| diag.code).collect()
    }

    #[test]
    fn default_profile_resolution_prefers_dev_and_returns_activation_sidecar() {
        let mut build = profile("build");
        build.path_entries = vec!["/build/bin".to_string()];
        let mut dev = profile("dev");
        dev.build_inputs = vec!["toolchain".to_string()];
        dev.env = vec![env("RUST_LOG", "info")];
        dev.path_entries = vec!["/dev/bin".to_string()];
        dev.hook = Some("echo ready".to_string());

        let plan = plan_named_shell_profile(request(vec![build, dev], None)).unwrap();

        assert_eq!(plan.selected_profile, "dev");
        assert_eq!(plan.build_inputs, vec!["toolchain".to_string()]);
        assert_eq!(plan.sidecar.env.get("RUST_LOG"), Some(&"info".to_string()));
        assert_eq!(plan.sidecar.path_entries, vec!["/dev/bin".to_string()]);
        assert_eq!(plan.sidecar.hook.as_deref(), Some("echo ready"));
        assert!(plan.non_claims.iter().any(|claim| claim.contains("does not prove builds")));
        assert!(plan.non_claims.iter().any(|claim| claim.contains("must not mutate")));
    }

    #[test]
    fn explicit_profile_selection_chooses_build_over_dev() {
        let build = profile("build");
        let dev = profile("dev");

        let plan = plan_named_shell_profile(request(vec![build, dev], Some("build"))).unwrap();

        assert_eq!(plan.selected_profile, "build");
        assert_eq!(plan.sidecar.path_entries, vec!["/tools/bin".to_string()]);
    }

    #[test]
    fn explicit_default_field_selects_profile() {
        let profiles = NamedShellProfiles {
            profiles: vec![profile("build"), profile("dev")],
            default_profile: Some("build".to_string()),
        };
        let request = ShellProfileSelectionRequest {
            profiles,
            requested_profile: None,
            requested_claims: Vec::new(),
        };

        let plan = plan_named_shell_profile(request).unwrap();

        assert_eq!(plan.selected_profile, "build");
    }

    #[test]
    fn invalid_profile_name_and_missing_profile_fail_before_activation() {
        let bad = profile("bad profile");
        let err = plan_named_shell_profile(request(vec![bad], Some("dev"))).unwrap_err();
        let codes = diagnostic_codes(&err);

        assert!(codes.contains(&"invalid-profile-name"));
        assert!(codes.contains(&"missing-profile"));
    }

    #[test]
    fn duplicate_env_key_after_normalization_is_rejected() {
        let mut dev = profile("dev");
        dev.env = vec![env("path", "/a"), env("PATH", "/b")];

        let err = plan_named_shell_profile(request(vec![dev], None)).unwrap_err();

        assert!(diagnostic_codes(&err).contains(&"duplicate-env-key"));
    }

    #[test]
    fn services_relative_paths_and_overclaims_are_rejected() {
        let mut dev = profile("dev");
        dev.path_entries = vec!["relative/bin".to_string()];
        dev.services = vec!["postgres".to_string()];
        let mut request = request(vec![dev], None);
        request.requested_claims = vec!["build-success".to_string()];

        let err = plan_named_shell_profile(request).unwrap_err();
        let codes = diagnostic_codes(&err);

        assert!(codes.contains(&"unsupported-path-entry"));
        assert!(codes.contains(&"services-not-supported"));
        assert!(codes.contains(&"unsupported-shell-claim"));
    }

    #[test]
    fn ambiguous_default_is_rejected_without_silent_fallback() {
        let err = plan_named_shell_profile(request(vec![profile("build"), profile("test")], None)).unwrap_err();

        assert!(diagnostic_codes(&err).contains(&"ambiguous-default-profile"));
    }
}
