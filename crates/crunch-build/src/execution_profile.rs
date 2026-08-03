//! Explicit execution policy for native and foreign derivations.
//!
//! The pure profile core validates bounded policy, derives canonical BLAKE3
//! identity, and verifies the reserved derivation binding. Build and CLI shells
//! only transport validated profiles.
// r[impl foreign_derivation_import.execution_profile]

use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::path::Component;
use std::path::Path;

use bstr::BString;
use nix_compat::derivation::Derivation;
use serde::Deserialize;
use serde::Serialize;
use thiserror::Error;

pub const EXECUTION_PROFILE_SCHEMA: &str = "mantle-foreign-execution-profile-v1";
pub const EXECUTION_PROFILE_BINDING_ENV: &str = "__MANTLE_FOREIGN_EXECUTION_PROFILE_V1";
const EXECUTION_PROFILE_DIGEST_DOMAIN: &[u8] = b"mantle-foreign-execution-profile-v1\0";
const NATIVE_COMPATIBILITY_PROFILE_ID: &str = "mantle-native-compatibility-v1";
pub(crate) const NIX_FOREIGN_PROFILE_ID: &str = "mantle-foreign-nix-v1";
const GUIX_FOREIGN_PROFILE_ID: &str = "mantle-foreign-guix-v1";
const BUILD_WORK_DIRECTORY: &str = "build";
const BIN_SH_PATH: &str = "/bin/sh";
const MAX_PROFILE_ID_BYTES: usize = 128;
const MAX_PROFILE_ENVIRONMENT_VARIABLES: usize = 128;
const MAX_PROFILE_ENVIRONMENT_KEY_BYTES: usize = 128;
const MAX_PROFILE_ENVIRONMENT_VALUE_BYTES: usize = 8_192;
const MAX_PROFILE_PROTECTED_VARIABLES: usize = 128;
const MAX_PROFILE_WRITABLE_PREFIXES: usize = 32;
const MAX_PROFILE_SYSCALL_EXCEPTIONS: usize = 64;
const MAX_PROFILE_UNSUPPORTED_CAPABILITIES: usize = 64;
const MAX_PROFILE_PATH_BYTES: usize = 4_096;
const MAX_PROFILE_MIN_MEMORY_BYTES: u64 = 1_099_511_627_776;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ExecutionProfileScope {
    NativeCompatibility,
    ForeignBound,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ExecutionEnvironmentMode {
    MantleCompatibility,
    DeclaredOnly,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ExecutionNetworkMode {
    Deny,
    AllowDeclared,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ExecutionResourceLimits {
    pub min_memory_bytes: u64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ExecutionProfile {
    pub schema: String,
    pub profile_id: String,
    pub scope: ExecutionProfileScope,
    pub environment_mode: ExecutionEnvironmentMode,
    pub fixed_environment: BTreeMap<String, String>,
    pub protected_variables: Vec<String>,
    pub provide_bin_sh: bool,
    pub work_directory: String,
    pub network_mode: ExecutionNetworkMode,
    pub allow_setid: bool,
    pub syscall_exceptions: Vec<String>,
    pub writable_prefixes: Vec<String>,
    pub resource_limits: ExecutionResourceLimits,
    pub unsupported_capabilities: Vec<String>,
}

#[derive(Debug, Error, Clone, Eq, PartialEq)]
pub enum ExecutionProfileError {
    #[error("execution profile schema is unsupported")]
    UnsupportedSchema,
    #[error("execution profile ID is invalid")]
    InvalidProfileId,
    #[error("foreign execution profile cannot use native compatibility environment mode")]
    ForeignCompatibilityEnvironment,
    #[error("execution profile collection exceeds its bound: {field}")]
    CollectionLimit { field: &'static str },
    #[error("execution profile environment key is invalid: {key}")]
    InvalidEnvironmentKey { key: String },
    #[error("execution profile environment value exceeds its bound: {key}")]
    EnvironmentValueLimit { key: String },
    #[error("execution profile protected variables must be sorted and unique")]
    ProtectedVariablesNotCanonical,
    #[error("execution profile protected variable has no fixed value: {key}")]
    ProtectedVariableMissingValue { key: String },
    #[error("execution profile path is invalid: {field}={path}")]
    InvalidPath { field: &'static str, path: String },
    #[error("execution profile writable prefixes must be sorted and unique")]
    WritablePrefixesNotCanonical,
    #[error("execution profile work directory is not writable")]
    WorkDirectoryNotWritable,
    #[error("setid behavior is unsupported by the local foreign adapter")]
    SetidUnsupported,
    #[error("syscall exceptions are unsupported by the local foreign adapter")]
    SyscallExceptionsUnsupported,
    #[error("execution profile declares unsupported capabilities")]
    UnsupportedCapabilities,
    #[error("execution profile minimum memory exceeds its bound")]
    MemoryLimitExceeded,
    #[error("native compatibility profile cannot bind a foreign derivation identity")]
    NativeProfileBinding,
    #[error("reserved execution profile field is already populated")]
    ReservedFieldCollision,
    #[error("execution profile binding is missing")]
    BindingMissing,
    #[error("execution profile binding is not UTF-8")]
    BindingNotUtf8,
    #[error("execution profile binding digest does not match the supplied profile")]
    BindingDigestMismatch,
    #[error("derivation overrides protected execution profile variable: {key}")]
    ProtectedVariableOverride { key: String },
    #[error("derivation requests /bin/sh while the execution profile forbids it")]
    BinShForbidden,
    #[error("execution profile serialization failed: {0}")]
    Serialization(String),
}

impl ExecutionProfile {
    #[must_use]
    pub fn native_compatibility() -> Self {
        Self {
            schema: EXECUTION_PROFILE_SCHEMA.to_string(),
            profile_id: NATIVE_COMPATIBILITY_PROFILE_ID.to_string(),
            scope: ExecutionProfileScope::NativeCompatibility,
            environment_mode: ExecutionEnvironmentMode::MantleCompatibility,
            fixed_environment: BTreeMap::new(),
            protected_variables: Vec::new(),
            provide_bin_sh: true,
            work_directory: BUILD_WORK_DIRECTORY.to_string(),
            network_mode: ExecutionNetworkMode::Deny,
            allow_setid: false,
            syscall_exceptions: Vec::new(),
            writable_prefixes: vec![BUILD_WORK_DIRECTORY.to_string()],
            resource_limits: ExecutionResourceLimits { min_memory_bytes: 0 },
            unsupported_capabilities: Vec::new(),
        }
    }

    #[must_use]
    pub fn foreign_nix() -> Self {
        let fixed_environment = BTreeMap::from([
            ("HOME".to_string(), "/homeless-shelter".to_string()),
            ("TEMP".to_string(), "/build".to_string()),
            ("TEMPDIR".to_string(), "/build".to_string()),
            ("TMP".to_string(), "/build".to_string()),
            ("TMPDIR".to_string(), "/build".to_string()),
        ]);
        Self::foreign_profile(NIX_FOREIGN_PROFILE_ID, fixed_environment, true)
    }

    #[must_use]
    pub fn foreign_guix() -> Self {
        let fixed_environment = BTreeMap::from([
            ("HOME".to_string(), "/homeless-shelter".to_string()),
            ("TEMP".to_string(), "/build".to_string()),
            ("TEMPDIR".to_string(), "/build".to_string()),
            ("TMP".to_string(), "/build".to_string()),
            ("TMPDIR".to_string(), "/build".to_string()),
        ]);
        Self::foreign_profile(GUIX_FOREIGN_PROFILE_ID, fixed_environment, false)
    }

    fn foreign_profile(profile_id: &str, fixed_environment: BTreeMap<String, String>, provide_bin_sh: bool) -> Self {
        let protected_variables = fixed_environment.keys().cloned().collect();
        Self {
            schema: EXECUTION_PROFILE_SCHEMA.to_string(),
            profile_id: profile_id.to_string(),
            scope: ExecutionProfileScope::ForeignBound,
            environment_mode: ExecutionEnvironmentMode::DeclaredOnly,
            fixed_environment,
            protected_variables,
            provide_bin_sh,
            work_directory: BUILD_WORK_DIRECTORY.to_string(),
            network_mode: ExecutionNetworkMode::Deny,
            allow_setid: false,
            syscall_exceptions: Vec::new(),
            writable_prefixes: vec![BUILD_WORK_DIRECTORY.to_string()],
            resource_limits: ExecutionResourceLimits { min_memory_bytes: 0 },
            unsupported_capabilities: Vec::new(),
        }
    }

    #[must_use]
    pub fn is_foreign_bound(&self) -> bool {
        self.scope == ExecutionProfileScope::ForeignBound
    }
}

pub fn foreign_profile_for_producer(producer_kind: &str) -> ExecutionProfile {
    if producer_kind.to_ascii_lowercase().contains("guix") {
        return ExecutionProfile::foreign_guix();
    }
    ExecutionProfile::foreign_nix()
}

pub fn validate_execution_profile(profile: &ExecutionProfile) -> Result<(), ExecutionProfileError> {
    if profile.schema != EXECUTION_PROFILE_SCHEMA {
        return Err(ExecutionProfileError::UnsupportedSchema);
    }
    validate_profile_id(&profile.profile_id)?;
    if profile.scope == ExecutionProfileScope::ForeignBound
        && profile.environment_mode == ExecutionEnvironmentMode::MantleCompatibility
    {
        return Err(ExecutionProfileError::ForeignCompatibilityEnvironment);
    }
    validate_environment(profile)?;
    validate_paths(profile)?;
    if profile.allow_setid {
        return Err(ExecutionProfileError::SetidUnsupported);
    }
    if profile.syscall_exceptions.len() > MAX_PROFILE_SYSCALL_EXCEPTIONS {
        return Err(ExecutionProfileError::CollectionLimit {
            field: "syscall_exceptions",
        });
    }
    if !profile.syscall_exceptions.is_empty() {
        return Err(ExecutionProfileError::SyscallExceptionsUnsupported);
    }
    if profile.unsupported_capabilities.len() > MAX_PROFILE_UNSUPPORTED_CAPABILITIES {
        return Err(ExecutionProfileError::CollectionLimit {
            field: "unsupported_capabilities",
        });
    }
    if !profile.unsupported_capabilities.is_empty() {
        return Err(ExecutionProfileError::UnsupportedCapabilities);
    }
    if profile.resource_limits.min_memory_bytes > MAX_PROFILE_MIN_MEMORY_BYTES {
        return Err(ExecutionProfileError::MemoryLimitExceeded);
    }
    Ok(())
}

fn validate_profile_id(profile_id: &str) -> Result<(), ExecutionProfileError> {
    if profile_id.is_empty()
        || profile_id.len() > MAX_PROFILE_ID_BYTES
        || !profile_id
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'-' | b'_' | b'.'))
    {
        return Err(ExecutionProfileError::InvalidProfileId);
    }
    Ok(())
}

fn validate_environment(profile: &ExecutionProfile) -> Result<(), ExecutionProfileError> {
    if profile.fixed_environment.len() > MAX_PROFILE_ENVIRONMENT_VARIABLES {
        return Err(ExecutionProfileError::CollectionLimit {
            field: "fixed_environment",
        });
    }
    if profile.protected_variables.len() > MAX_PROFILE_PROTECTED_VARIABLES {
        return Err(ExecutionProfileError::CollectionLimit {
            field: "protected_variables",
        });
    }
    for (key, value) in &profile.fixed_environment {
        if key.is_empty()
            || key.len() > MAX_PROFILE_ENVIRONMENT_KEY_BYTES
            || key.contains('=')
            || key.bytes().any(|byte| byte == 0)
        {
            return Err(ExecutionProfileError::InvalidEnvironmentKey { key: key.clone() });
        }
        if value.len() > MAX_PROFILE_ENVIRONMENT_VALUE_BYTES || value.bytes().any(|byte| byte == 0) {
            return Err(ExecutionProfileError::EnvironmentValueLimit { key: key.clone() });
        }
    }
    if !is_sorted_unique(&profile.protected_variables) {
        return Err(ExecutionProfileError::ProtectedVariablesNotCanonical);
    }
    for key in &profile.protected_variables {
        if !profile.fixed_environment.contains_key(key) {
            return Err(ExecutionProfileError::ProtectedVariableMissingValue { key: key.clone() });
        }
    }
    Ok(())
}

fn validate_paths(profile: &ExecutionProfile) -> Result<(), ExecutionProfileError> {
    validate_clean_relative_path("work_directory", &profile.work_directory)?;
    if profile.writable_prefixes.is_empty() || profile.writable_prefixes.len() > MAX_PROFILE_WRITABLE_PREFIXES {
        return Err(ExecutionProfileError::CollectionLimit {
            field: "writable_prefixes",
        });
    }
    if !is_sorted_unique(&profile.writable_prefixes) {
        return Err(ExecutionProfileError::WritablePrefixesNotCanonical);
    }
    for path in &profile.writable_prefixes {
        validate_clean_relative_path("writable_prefixes", path)?;
    }
    let work_directory = Path::new(&profile.work_directory);
    if !profile.writable_prefixes.iter().any(|prefix| work_directory.starts_with(prefix)) {
        return Err(ExecutionProfileError::WorkDirectoryNotWritable);
    }
    Ok(())
}

fn validate_clean_relative_path(field: &'static str, path: &str) -> Result<(), ExecutionProfileError> {
    let valid = !path.is_empty()
        && path.len() <= MAX_PROFILE_PATH_BYTES
        && Path::new(path).components().all(|component| matches!(component, Component::Normal(_)));
    if !valid {
        return Err(ExecutionProfileError::InvalidPath {
            field,
            path: path.to_string(),
        });
    }
    Ok(())
}

fn is_sorted_unique(values: &[String]) -> bool {
    values.windows(2).all(|pair| pair[0] < pair[1])
}

pub fn execution_profile_digest(profile: &ExecutionProfile) -> Result<String, ExecutionProfileError> {
    validate_execution_profile(profile)?;
    let canonical =
        serde_json::to_vec(profile).map_err(|error| ExecutionProfileError::Serialization(error.to_string()))?;
    let mut hasher = blake3::Hasher::new();
    hasher.update(EXECUTION_PROFILE_DIGEST_DOMAIN);
    hasher.update(&canonical);
    Ok(hasher.finalize().to_hex().to_string())
}

pub fn bind_execution_profile(
    derivation: &mut Derivation,
    profile: &ExecutionProfile,
) -> Result<String, ExecutionProfileError> {
    validate_execution_profile(profile)?;
    if !profile.is_foreign_bound() {
        return Err(ExecutionProfileError::NativeProfileBinding);
    }
    if derivation.environment.contains_key(EXECUTION_PROFILE_BINDING_ENV) {
        return Err(ExecutionProfileError::ReservedFieldCollision);
    }
    let digest = execution_profile_digest(profile)?;
    derivation
        .environment
        .insert(EXECUTION_PROFILE_BINDING_ENV.to_string(), BString::from(digest.clone()));
    Ok(digest)
}

pub fn verify_execution_profile_binding(
    derivation: &Derivation,
    profile: &ExecutionProfile,
) -> Result<(), ExecutionProfileError> {
    validate_execution_profile(profile)?;
    let binding = derivation.environment.get(EXECUTION_PROFILE_BINDING_ENV);
    if !profile.is_foreign_bound() {
        if binding.is_some() {
            return Err(ExecutionProfileError::ReservedFieldCollision);
        }
        return Ok(());
    }
    let binding = binding.ok_or(ExecutionProfileError::BindingMissing)?;
    let binding = std::str::from_utf8(binding).map_err(|_| ExecutionProfileError::BindingNotUtf8)?;
    if binding != execution_profile_digest(profile)? {
        return Err(ExecutionProfileError::BindingDigestMismatch);
    }
    validate_derivation_profile_surface(derivation, profile)
}

pub fn validate_derivation_profile_surface(
    derivation: &Derivation,
    profile: &ExecutionProfile,
) -> Result<(), ExecutionProfileError> {
    for key in &profile.protected_variables {
        if derivation.environment.contains_key(key) {
            return Err(ExecutionProfileError::ProtectedVariableOverride { key: key.clone() });
        }
    }
    if !profile.provide_bin_sh && derivation_mentions_bin_sh(derivation) {
        return Err(ExecutionProfileError::BinShForbidden);
    }
    Ok(())
}

fn derivation_mentions_bin_sh(derivation: &Derivation) -> bool {
    derivation.builder == BIN_SH_PATH
        || derivation.arguments.iter().any(|argument| argument.contains(BIN_SH_PATH))
        || derivation.environment.iter().any(|(key, value)| {
            key != EXECUTION_PROFILE_BINDING_ENV
                && std::str::from_utf8(value).is_ok_and(|text| text.contains(BIN_SH_PATH))
        })
}

pub fn declared_profile_environment(
    derivation: &Derivation,
    profile: &ExecutionProfile,
) -> Result<BTreeMap<String, Vec<u8>>, ExecutionProfileError> {
    verify_execution_profile_binding(derivation, profile)?;
    let mut environment = profile
        .fixed_environment
        .iter()
        .map(|(key, value)| (key.clone(), value.as_bytes().to_vec()))
        .collect::<BTreeMap<_, _>>();
    let protected = profile.protected_variables.iter().map(String::as_str).collect::<BTreeSet<_>>();
    for (key, value) in &derivation.environment {
        if key == EXECUTION_PROFILE_BINDING_ENV {
            continue;
        }
        if protected.contains(key.as_str()) {
            return Err(ExecutionProfileError::ProtectedVariableOverride { key: key.clone() });
        }
        environment.insert(key.clone(), Vec::from(<BString as AsRef<[u8]>>::as_ref(value)));
    }
    Ok(environment)
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::collections::BTreeSet;

    use nix_compat::derivation::Output;

    use super::*;

    const OVER_LIMIT_MEMORY_BYTES: u64 = MAX_PROFILE_MIN_MEMORY_BYTES + 1;

    fn derivation(builder: &str) -> Derivation {
        Derivation {
            arguments: Vec::new(),
            builder: builder.to_string(),
            environment: BTreeMap::from([("out".to_string(), BString::from("/mantle/store/out"))]),
            input_derivations: BTreeMap::new(),
            input_sources: BTreeSet::new(),
            outputs: BTreeMap::from([("out".to_string(), Output {
                path: None,
                ca_hash: None,
            })]),
            system: "x86_64-linux".to_string(),
        }
    }

    #[test]
    fn guix_profile_binds_and_strips_reserved_field_from_declared_environment() {
        let profile = ExecutionProfile::foreign_guix();
        let mut derivation = derivation("/mantle/store/builder");
        let digest = bind_execution_profile(&mut derivation, &profile).unwrap();

        verify_execution_profile_binding(&derivation, &profile).unwrap();
        let environment = declared_profile_environment(&derivation, &profile).unwrap();

        assert_eq!(digest, execution_profile_digest(&profile).unwrap());
        assert!(!environment.contains_key(EXECUTION_PROFILE_BINDING_ENV));
        assert!(!environment.contains_key("SHELL"));
        assert_eq!(environment.get("HOME").map(Vec::as_slice), Some(b"/homeless-shelter".as_slice()));
    }

    #[test]
    fn nickel_exports_match_rust_profile_defaults() {
        let guix: ExecutionProfile =
            serde_json::from_str(include_str!("../../../config/foreign-execution-profiles/generated/guix.json"))
                .unwrap();
        let nix: ExecutionProfile =
            serde_json::from_str(include_str!("../../../config/foreign-execution-profiles/generated/nix.json"))
                .unwrap();

        assert_eq!(guix, ExecutionProfile::foreign_guix());
        assert_eq!(nix, ExecutionProfile::foreign_nix());
    }

    #[test]
    fn profile_digest_changes_when_shell_policy_changes() {
        let guix = ExecutionProfile::foreign_guix();
        let nix = ExecutionProfile::foreign_nix();
        assert_ne!(execution_profile_digest(&guix).unwrap(), execution_profile_digest(&nix).unwrap());
    }

    #[test]
    fn profile_rejects_unknown_json_field() {
        let mut value = serde_json::to_value(ExecutionProfile::foreign_guix()).unwrap();
        value.as_object_mut().unwrap().insert("ambient_magic".to_string(), serde_json::Value::Bool(true));
        let error = serde_json::from_value::<ExecutionProfile>(value).unwrap_err();
        assert!(error.to_string().contains("unknown field"));
    }

    #[test]
    fn profile_rejects_limits_and_unsupported_authority() {
        let mut memory = ExecutionProfile::foreign_guix();
        memory.resource_limits.min_memory_bytes = OVER_LIMIT_MEMORY_BYTES;
        assert_eq!(validate_execution_profile(&memory), Err(ExecutionProfileError::MemoryLimitExceeded));

        let mut setid = ExecutionProfile::foreign_guix();
        setid.allow_setid = true;
        assert_eq!(validate_execution_profile(&setid), Err(ExecutionProfileError::SetidUnsupported));

        let mut syscalls = ExecutionProfile::foreign_guix();
        syscalls.syscall_exceptions.push("mount".to_string());
        assert_eq!(validate_execution_profile(&syscalls), Err(ExecutionProfileError::SyscallExceptionsUnsupported));

        let mut unsupported = ExecutionProfile::foreign_guix();
        unsupported.unsupported_capabilities.push("remote-execution".to_string());
        assert_eq!(validate_execution_profile(&unsupported), Err(ExecutionProfileError::UnsupportedCapabilities));
    }

    #[test]
    fn profile_rejects_bad_writable_prefix_and_environment_override() {
        let mut path = ExecutionProfile::foreign_guix();
        path.writable_prefixes = vec!["../escape".to_string()];
        assert!(matches!(
            validate_execution_profile(&path),
            Err(ExecutionProfileError::InvalidPath {
                field: "writable_prefixes",
                ..
            })
        ));

        let profile = ExecutionProfile::foreign_guix();
        let mut protected = derivation("/mantle/store/builder");
        protected.environment.insert("HOME".to_string(), BString::from("/tmp"));
        bind_execution_profile(&mut protected, &profile).unwrap();
        assert_eq!(
            verify_execution_profile_binding(&protected, &profile),
            Err(ExecutionProfileError::ProtectedVariableOverride {
                key: "HOME".to_string()
            })
        );
    }

    #[test]
    fn profile_rejects_reserved_collision_stale_digest_and_bin_sh() {
        let profile = ExecutionProfile::foreign_guix();
        let mut collision = derivation("/mantle/store/builder");
        collision.environment.insert(EXECUTION_PROFILE_BINDING_ENV.to_string(), BString::from("occupied"));
        assert_eq!(
            bind_execution_profile(&mut collision, &profile),
            Err(ExecutionProfileError::ReservedFieldCollision)
        );

        let mut stale = derivation("/mantle/store/builder");
        bind_execution_profile(&mut stale, &profile).unwrap();
        stale.environment.insert(EXECUTION_PROFILE_BINDING_ENV.to_string(), BString::from("0".repeat(64)));
        assert_eq!(
            verify_execution_profile_binding(&stale, &profile),
            Err(ExecutionProfileError::BindingDigestMismatch)
        );

        let mut shell = derivation(BIN_SH_PATH);
        bind_execution_profile(&mut shell, &profile).unwrap();
        assert_eq!(verify_execution_profile_binding(&shell, &profile), Err(ExecutionProfileError::BinShForbidden));
    }
}
