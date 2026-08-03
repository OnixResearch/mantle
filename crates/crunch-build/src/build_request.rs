//! Translate `nix_compat::Derivation` → `snix_build::BuildRequest`.
//!
//! Adapted from snix-glue's `derivation_into_build_request`. Mantle supports
//! the bounded `passAsFile` and structured-attribute protocols needed by
//! imported Nix derivations.

use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::collections::HashSet;
use std::path::PathBuf;

use bstr::BString;
use bytes::Bytes;
use nix_compat::derivation::Derivation;
use nix_compat::derivation::Output;
use nix_compat::nixbase32;
use nix_compat::store_path::StorePath;
use nix_compat::store_path::hash_placeholder;
use sha2::Digest as _;
use sha2::Sha256;
use snix_build::buildservice::AdditionalFile;
use snix_build::buildservice::BuildConstraints;
use snix_build::buildservice::BuildRequest;
use snix_build::buildservice::EnvVar;
use snix_build::buildservice::StatefulWorkspaceLeaseBinding;
use snix_build::buildservice::StatefulWorkspaceMode;
use snix_build::buildservice::StatefulWorkspaceRequest;
use snix_castore::Node;

use crate::BuildEnvironmentReport;
use crate::ExecutionEnvironmentMode;
use crate::ExecutionNetworkMode;
use crate::ExecutionProfile;
use crate::HermeticityAuditEvent;
use crate::HermeticityAuditKind;
use crate::HermeticityMode;
use crate::declared_profile_environment;
use crate::environment_policy;
use crate::execution_profile::NIX_FOREIGN_PROFILE_ID;
use crate::network_policy::CompatibilityNetworkPolicy;
use crate::network_policy::plan_network_policy;
use crate::registry::DerivationRegistry;
use crate::validate_execution_profile;
use crate::verify_execution_profile_binding;

/// Environment variables that crunch sets in every sandbox build,
/// matching Nix's sandbox conventions for compatibility with build
/// scripts that expect them.
const SANDBOX_PATH_NOT_SET: &str = "/path-not-set";
const PATH_VARIABLE: &str = "PATH";
const PASS_AS_FILE_ENV: &str = "passAsFile";
const PASS_AS_FILE_PATH_SUFFIX: &str = "Path";
const PASS_AS_FILE_DIRECTORY: &str = "build";
const PASS_AS_FILE_NAME_PREFIX: &str = ".attr-";
const NIX_BUILD_DIRECTORY: &str = "/build";
const NIX_LOG_FILE_DESCRIPTOR: &str = "2";
const NIX_STRUCTURED_ATTRS_ENV: &str = "__json";
const NIX_ATTRS_SH_ENV: &str = "NIX_ATTRS_SH_FILE";
const NIX_ATTRS_JSON_ENV: &str = "NIX_ATTRS_JSON_FILE";
const NIX_ATTRS_SH_FILE_NAME: &str = ".attrs.sh";
const NIX_ATTRS_JSON_FILE_NAME: &str = ".attrs.json";
const NIX_STRUCTURED_SYNTHETIC_ENV_KEYS: &[&str] = &["builder", "name", "outputs", "system"];
const KIBIBYTE_BYTES: usize = 1_024;
const MAX_STRUCTURED_ATTRS_KIBIBYTES: usize = 1_024;
const MAX_STRUCTURED_ATTRS_BYTES: usize = MAX_STRUCTURED_ATTRS_KIBIBYTES * KIBIBYTE_BYTES;
const STRUCTURED_ATTRS_FILE_COUNT: usize = 2;
const SHELL_QUOTE_COUNT: usize = 2;
const NIX_PROTOCOL_ENV_VARS: [(&str, &str); 3] = [
    ("NIX_BUILD_TOP", NIX_BUILD_DIRECTORY),
    ("NIX_LOG_FD", NIX_LOG_FILE_DESCRIPTOR),
    ("PWD", NIX_BUILD_DIRECTORY),
];
pub const WORKSPACE_POLICY_ENV: &str = "__MANTLE_STATEFUL_WORKSPACE_POLICY";
pub const WORKSPACE_LEASE_ENV: &str = "__MANTLE_STATEFUL_WORKSPACE_LEASE";
const LOCAL_WORKSPACE_WORKER_ID: &str = "local-worker";
const LOCAL_WORKSPACE_ATTEMPT_ID: &str = "local-attempt";
const INITIAL_WORKSPACE_GENERATION: u64 = 1;
const INITIAL_WORKSPACE_FENCE_GENERATION: u64 = 1;
const PATH_ENTRY_SEPARATOR: char = ':';
const STORE_PATH_HASH_CHARS: usize = 32;
const STORE_COMPONENT_NAME_SEPARATOR_CHARS: usize = 1;
const MIN_STORE_COMPONENT_CHARS: usize = STORE_PATH_HASH_CHARS.saturating_add(STORE_COMPONENT_NAME_SEPARATOR_CHARS);
const STRICT_DETERMINISM_CONTROL_COUNT: usize = 8;
const DETERMINISM_POLICY_FIXED_ENV: &str = "fixed-env";
const DETERMINISM_POLICY_FIXED_EXECUTOR: &str = "fixed-executor";
const DETERMINISM_POLICY_MODELED: &str = "modeled";
const DETERMINISM_SURFACE_TIME: &str = "time";
const DETERMINISM_SURFACE_TIMEZONE: &str = "timezone";
const DETERMINISM_SURFACE_LOCALE: &str = "locale";
const DETERMINISM_SURFACE_TEMP_ROOTS: &str = "temp-roots";
const DETERMINISM_SURFACE_HOST_USER: &str = "host-user-metadata";
const DETERMINISM_SURFACE_UMASK: &str = "umask";
const DETERMINISM_SURFACE_RANDOMNESS: &str = "modeled-randomness";
const DETERMINISM_SURFACE_ORDERING: &str = "order-sensitive-output-processing";
const STRICT_UMASK_VALUE: &str = "0022";
const STRICT_RANDOMNESS_VALUE: &str = "no-modeled-random-seed";
const STRICT_ORDERING_VALUE: &str = "lexicographic-output-processing";
const SANDBOX_ENV_VARS: [(&str, &str); 19] = [
    ("HOME", "/homeless-shelter"),
    ("LANG", "C"),
    ("LC_ALL", "C"),
    ("LOGNAME", "nixbld"),
    ("NIX_BUILD_CORES", "1"),
    ("NIX_BUILD_TOP", "/build"),
    ("NIX_LOG_FD", "2"),
    ("NIX_STORE", "/nix/store"),
    ("PATH", SANDBOX_PATH_NOT_SET),
    ("PWD", "/build"),
    ("SHELL", "/bin/sh"),
    ("SOURCE_DATE_EPOCH", "1"),
    ("TEMP", "/build"),
    ("TEMPDIR", "/build"),
    ("TERM", "xterm-256color"),
    ("TMP", "/build"),
    ("TMPDIR", "/build"),
    ("TZ", "UTC"),
    ("USER", "nixbld"),
];

const ALLOWED_SANDBOX_ENV_OVERRIDES: [&str; 2] = ["NIX_BUILD_CORES", "SOURCE_DATE_EPOCH"];

/// Translate a `Derivation` into a `BuildRequest`.
///
/// `inputs` maps store path → castore Node for every input that must be
/// visible in the sandbox. The caller resolves these from the store
/// before calling this function.
///
/// `known_paths` is used to look up nested derivation outputs when
/// resolving `input_derivations`.
/// Compile-time: sandbox env vars must not be empty.
const _: () = assert!(!SANDBOX_ENV_VARS.is_empty());

#[derive(Debug)]
pub struct NormalizedBuildEnvironment {
    pub environment_vars: BTreeMap<String, Vec<u8>>,
    pub audit_events: Vec<HermeticityAuditEvent>,
    pub report: BuildEnvironmentReport,
}

#[derive(Debug)]
struct PassAsFileExpansion {
    environment_vars: BTreeMap<String, Vec<u8>>,
    additional_files: Vec<AdditionalFile>,
}

#[derive(Debug)]
struct StructuredAttrsExpansion {
    environment_vars: BTreeMap<String, Vec<u8>>,
    additional_files: Vec<AdditionalFile>,
}

#[derive(Debug)]
#[must_use = "inspect audit_events or consciously discard them"]
pub struct BuildRequestEnvelope {
    pub build_request: BuildRequest,
    pub audit_events: Vec<HermeticityAuditEvent>,
    pub build_environment_report: BuildEnvironmentReport,
    pub network_policy_report: crate::BuildNetworkPolicyReport,
}

struct StrictProtectedOverrideInput<'a> {
    derivation: &'a Derivation,
    store_dir: &'a str,
    accepted_variable_count: usize,
    key: &'a str,
    sandbox_value: &'a [u8],
    value: &'a [u8],
}

struct DeterminismControlInput<'a> {
    surface: &'a str,
    policy: &'a str,
    value: String,
}

struct ReceiptBoundSearchPathInput<'a> {
    value: &'a str,
    store_dir: &'a str,
}

struct DeclaredToolPathInput<'a> {
    entry: &'a str,
    store_dir: &'a str,
}

pub fn derivation_to_build_request(
    derivation: &Derivation,
    inputs: &BTreeMap<StorePath<String>, Node>,
    store_dir: &str,
    execution_profile: &ExecutionProfile,
    hermeticity_mode: HermeticityMode,
) -> Result<BuildRequestEnvelope, crate::Error> {
    debug_assert!(!derivation.builder.is_empty(), "builder must not be empty");
    debug_assert!(!store_dir.is_empty(), "store_dir must not be empty");
    debug_assert!(store_dir.starts_with('/'), "store_dir must be absolute path");
    validate_execution_profile(execution_profile).map_err(execution_profile_error)?;
    verify_execution_profile_binding(derivation, execution_profile).map_err(execution_profile_error)?;
    let normalized = normalized_environment_for_profile(derivation, store_dir, execution_profile, hermeticity_mode)?;
    let compatibility_network_policy = match execution_profile.network_mode {
        ExecutionNetworkMode::Deny => CompatibilityNetworkPolicy::DenyAll,
        ExecutionNetworkMode::AllowDeclared => CompatibilityNetworkPolicy::AllowDeclared,
    };
    let network_policy = plan_network_policy(derivation, compatibility_network_policy)?;
    let workspace = workspace_request_from_derivation(derivation)?;
    let mut build_request = build_request_from_environment(
        derivation,
        inputs,
        store_dir,
        execution_profile,
        normalized.environment_vars,
        network_policy.allow_network,
    )?;
    build_request.workspace = workspace;
    Ok(BuildRequestEnvelope {
        build_request,
        audit_events: normalized.audit_events,
        build_environment_report: normalized.report,
        network_policy_report: network_policy.report,
    })
}

fn normalized_environment_for_profile(
    derivation: &Derivation,
    store_dir: &str,
    execution_profile: &ExecutionProfile,
    hermeticity_mode: HermeticityMode,
) -> Result<NormalizedBuildEnvironment, crate::Error> {
    if execution_profile.environment_mode == ExecutionEnvironmentMode::MantleCompatibility {
        return normalize_build_environment(derivation, store_dir, hermeticity_mode);
    }
    let declared_environment =
        declared_profile_environment(derivation, execution_profile).map_err(execution_profile_error)?;
    let environment_vars = if execution_profile.profile_id == NIX_FOREIGN_PROFILE_ID {
        nix_protocol_environment(declared_environment, store_dir)
    } else {
        declared_environment
    };
    let action_name = environment_policy::action_name_from_environment(&derivation.environment);
    let report = environment_policy::success_report(action_name, &environment_vars);
    Ok(NormalizedBuildEnvironment {
        environment_vars,
        audit_events: Vec::new(),
        report,
    })
}

fn execution_profile_error(error: crate::ExecutionProfileError) -> crate::Error {
    crate::Error::Store(format!("invalid execution profile: {error}"))
}

fn nix_protocol_environment(
    mut declared_environment: BTreeMap<String, Vec<u8>>,
    store_dir: &str,
) -> BTreeMap<String, Vec<u8>> {
    assert!(!store_dir.is_empty(), "Nix protocol store directory must not be empty");
    assert!(store_dir.starts_with('/'), "Nix protocol store directory must be absolute");
    for (key, value) in NIX_PROTOCOL_ENV_VARS {
        declared_environment.insert(key.to_string(), value.as_bytes().to_vec());
    }
    declared_environment.insert("NIX_STORE".to_string(), store_dir.as_bytes().to_vec());
    debug_assert_eq!(declared_environment.get("NIX_BUILD_TOP").map(Vec::as_slice), Some(b"/build".as_slice()));
    debug_assert_eq!(declared_environment.get("NIX_STORE").map(Vec::as_slice), Some(store_dir.as_bytes()));
    declared_environment
}

fn workspace_request_from_derivation(
    derivation: &Derivation,
) -> Result<Option<StatefulWorkspaceRequest>, crate::Error> {
    let Some(raw) = derivation.environment.get(WORKSPACE_POLICY_ENV) else {
        return Ok(None);
    };
    let policy: crate::WorkspacePolicy = serde_json::from_slice(raw.as_ref())
        .map_err(|error| crate::Error::Store(format!("invalid stateful workspace policy: {error}")))?;
    crate::validate_workspace_policy(&policy)
        .map_err(|reason| crate::Error::Store(format!("invalid stateful workspace policy: {}", reason.as_str())))?;
    let compatibility_digest_blake3 = crate::derive_workspace_compatibility_digest(&policy.compatibility)
        .map_err(|reason| crate::Error::Store(format!("invalid workspace compatibility facts: {}", reason.as_str())))?;
    let mode = match policy.mode {
        crate::WorkspaceMode::None => StatefulWorkspaceMode::None,
        crate::WorkspaceMode::ImmutableSnapshot => StatefulWorkspaceMode::ImmutableSnapshot,
        crate::WorkspaceMode::MutableSession => StatefulWorkspaceMode::MutableSession,
    };
    let snapshot_input_name = policy.snapshot_ref.as_deref().map(snapshot_input_name).transpose()?;
    let action_name = environment_policy::action_name_from_environment(&derivation.environment);
    let declared_lease = derivation
        .environment
        .get(WORKSPACE_LEASE_ENV)
        .map(|raw| serde_json::from_slice::<StatefulWorkspaceLeaseBinding>(raw.as_ref()))
        .transpose()
        .map_err(|error| crate::Error::Store(format!("invalid stateful workspace lease: {error}")))?;
    let lease = workspace_lease_binding(mode, &policy, &action_name, declared_lease)?;
    let is_mutable_session = mode == StatefulWorkspaceMode::MutableSession;
    let workspace_request = StatefulWorkspaceRequest {
        mode,
        workspace_id: policy.workspace_id,
        guest_path: PathBuf::from(policy.guest_path),
        snapshot_input_name,
        compatibility_digest_blake3,
        toolchain_refs: policy.compatibility.toolchain_refs,
        quota_bytes_max: policy.quota.bytes_max,
        quota_files_max: policy.quota.files_max,
        quota_snapshots_max: policy.quota.snapshots_max,
        retention_class: "declared".to_string(),
        retention_workspace_count_max: policy.retention.workspace_count_max,
        retention_idle_generations_max: policy.retention.idle_generations_max,
        retention_age_generations_max: policy.retention.age_generations_max,
        retention_quarantine_count_max: policy.retention.quarantine_count_max,
        generation: lease.as_ref().map(|binding| binding.fence_generation).unwrap_or(INITIAL_WORKSPACE_GENERATION),
        lease,
        sensitive_paths: policy.scrub.sensitive_paths,
        secret_markers: policy.scrub.secret_markers,
        scan_depth_max: policy.scrub.scan_depth_max,
        path_bytes_max: policy.scrub.path_bytes_max,
        snapshot_enabled: policy.snapshot.enabled,
        clean_rebuild_enabled: policy.clean_rebuild.enabled,
        clean_rebuild_require_declared_inputs: policy.clean_rebuild.require_declared_inputs,
        runtime_host_path: None,
    };
    debug_assert_eq!(workspace_request.lease.is_some(), is_mutable_session);
    debug_assert_eq!(
        workspace_request.generation,
        workspace_request
            .lease
            .as_ref()
            .map(|binding| binding.fence_generation)
            .unwrap_or(INITIAL_WORKSPACE_GENERATION)
    );
    Ok(Some(workspace_request))
}

fn workspace_lease_binding(
    mode: StatefulWorkspaceMode,
    policy: &crate::WorkspacePolicy,
    action_name: &str,
    declared_lease: Option<StatefulWorkspaceLeaseBinding>,
) -> Result<Option<StatefulWorkspaceLeaseBinding>, crate::Error> {
    if mode == StatefulWorkspaceMode::MutableSession {
        return Ok(Some(declared_lease.unwrap_or_else(|| StatefulWorkspaceLeaseBinding {
            worker_id: LOCAL_WORKSPACE_WORKER_ID.to_string(),
            authority_class: policy.compatibility.authority_class.clone(),
            job_id: sanitize_workspace_identity(action_name),
            attempt_id: LOCAL_WORKSPACE_ATTEMPT_ID.to_string(),
            fence_generation: INITIAL_WORKSPACE_FENCE_GENERATION,
        })));
    }
    if declared_lease.is_some() {
        return Err(crate::Error::Store("stateful workspace lease is only valid for mutable-session mode".to_string()));
    }
    Ok(None)
}

fn snapshot_input_name(value: &str) -> Result<PathBuf, crate::Error> {
    let path = PathBuf::from(value);
    if let Some(name) = path.file_name() {
        return Ok(PathBuf::from(name));
    }
    let digest = value
        .strip_prefix(crate::WORKSPACE_SNAPSHOT_REF_PREFIX)
        .ok_or_else(|| crate::Error::Store("workspace snapshot ref has no declared input name".to_string()))?;
    Ok(PathBuf::from(digest))
}

fn sanitize_workspace_identity(value: &str) -> String {
    let mut normalized = value
        .bytes()
        .map(|byte| {
            if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.') {
                char::from(byte)
            } else {
                '-'
            }
        })
        .take(crate::MAX_WORKSPACE_ID_BYTES)
        .collect::<String>();
    if normalized.is_empty() {
        normalized.push_str("action");
    }
    normalized
}

pub fn normalize_build_environment(
    derivation: &Derivation,
    store_dir: &str,
    hermeticity_mode: HermeticityMode,
) -> Result<NormalizedBuildEnvironment, crate::Error> {
    debug_assert!(!derivation.builder.is_empty(), "builder must not be empty");
    debug_assert!(!derivation.outputs.is_empty(), "must have at least one output");
    debug_assert!(!store_dir.is_empty(), "store_dir must not be empty");
    debug_assert!(store_dir.starts_with('/'), "store_dir must be absolute path");

    let mut environment_vars = default_sandbox_environment(store_dir);
    let audit_events = overlay_derivation_environment(derivation, store_dir, hermeticity_mode, &mut environment_vars)?;
    debug_assert!(!environment_vars.is_empty(), "environment must include sandbox vars");

    let action_name = environment_policy::action_name_from_environment(&derivation.environment);
    let search_path = strict_search_path_report(&environment_vars, store_dir, hermeticity_mode)?;
    let determinism = strict_determinism_report(&environment_vars, hermeticity_mode)?;
    let mut build_environment_evidence = environment_policy::success_report(action_name, &environment_vars);
    build_environment_evidence.search_path = search_path;
    build_environment_evidence.determinism = determinism;

    Ok(NormalizedBuildEnvironment {
        environment_vars,
        audit_events,
        report: build_environment_evidence,
    })
}

pub(crate) fn build_request_from_environment(
    derivation: &Derivation,
    inputs: &BTreeMap<StorePath<String>, Node>,
    store_dir: &str,
    execution_profile: &ExecutionProfile,
    environment_vars: BTreeMap<String, Vec<u8>>,
    allow_network: bool,
) -> Result<BuildRequest, crate::Error> {
    // Tiger Style: assert preconditions.
    debug_assert!(!derivation.builder.is_empty(), "builder must not be empty");
    debug_assert!(!derivation.outputs.is_empty(), "must have at least one output");
    debug_assert!(!store_dir.is_empty(), "store_dir must not be empty");
    debug_assert!(store_dir.starts_with('/'), "store_dir must be absolute path");

    let command_args = build_command_args(derivation, store_dir);
    let constraints = build_constraints(derivation, execution_profile, allow_network);
    let refscan_needles = build_refscan_needles(derivation, inputs);
    let environment_vars = replace_environment_placeholders(environment_vars, &derivation.outputs, store_dir);
    let structured_attrs = materialize_structured_attrs(environment_vars, &derivation.outputs, store_dir)?;
    let pass_as_file = expand_pass_as_file(structured_attrs.environment_vars)?;
    let additional_files = merge_additional_files(structured_attrs.additional_files, pass_as_file.additional_files)?;

    // Tiger Style: assert command_args has at least the builder.
    debug_assert!(!command_args.is_empty());
    debug_assert!(!pass_as_file.environment_vars.is_empty(), "environment must include sandbox vars");

    let sandbox_outputs = map_outputs_to_sandbox_paths(derivation, store_dir);
    let input_map = map_inputs_to_components(inputs)?;

    Ok(BuildRequest {
        command_args,
        outputs: sandbox_outputs,
        environment_vars: pass_as_file
            .environment_vars
            .into_iter()
            .map(|(key, value)| EnvVar {
                key,
                value: Bytes::from(value),
            })
            .collect(),
        inputs: input_map,
        inputs_dir: store_dir[1..].into(),
        constraints,
        working_dir: execution_profile.work_directory.clone().into(),
        scratch_paths: execution_profile_scratch_paths(execution_profile, store_dir),
        additional_files,
        refscan_needles,
        workspace: None,
    })
}

fn replace_environment_placeholders(
    mut environment_vars: BTreeMap<String, Vec<u8>>,
    outputs: &BTreeMap<String, Output>,
    store_dir: &str,
) -> BTreeMap<String, Vec<u8>> {
    assert!(!store_dir.is_empty(), "placeholder store directory must not be empty");
    assert!(store_dir.starts_with('/'), "placeholder store directory must be absolute");
    for value in environment_vars.values_mut() {
        let source = BString::from(value.as_slice());
        let replaced = replace_placeholders_bstr(&source, outputs, store_dir);
        *value = Vec::from(<BString as AsRef<[u8]>>::as_ref(&replaced));
    }
    environment_vars
}

/// Materialize Nix structured attributes without I/O.
fn materialize_structured_attrs(
    mut environment_vars: BTreeMap<String, Vec<u8>>,
    outputs: &BTreeMap<String, Output>,
    store_dir: &str,
) -> Result<StructuredAttrsExpansion, crate::Error> {
    let Some(raw_attrs) = environment_vars.remove(NIX_STRUCTURED_ATTRS_ENV) else {
        return Ok(StructuredAttrsExpansion {
            environment_vars,
            additional_files: Vec::new(),
        });
    };
    if raw_attrs.len() > MAX_STRUCTURED_ATTRS_BYTES {
        return Err(crate::Error::Store("Nix structured attributes exceed the byte limit".to_string()));
    }
    if environment_vars.contains_key(NIX_ATTRS_SH_ENV) || environment_vars.contains_key(NIX_ATTRS_JSON_ENV) {
        return Err(crate::Error::Store(
            "Nix structured attributes collide with protocol environment variables".to_string(),
        ));
    }

    let mut attrs: serde_json::Value = serde_json::from_slice(&raw_attrs)
        .map_err(|error| crate::Error::Store(format!("invalid Nix structured attributes: {error}")))?;
    let attrs_object = attrs
        .as_object_mut()
        .ok_or_else(|| crate::Error::Store("Nix structured attributes must be a JSON object".to_string()))?;
    let declared_outputs = attrs_object
        .remove("outputs")
        .ok_or_else(|| crate::Error::Store("Nix structured attributes are missing outputs".to_string()))?;
    validate_structured_output_names(&declared_outputs, outputs)?;
    attrs_object.insert(
        "outputs".to_string(),
        serde_json::Value::Object(structured_output_paths(outputs, &environment_vars, store_dir)?),
    );

    for output_name in outputs.keys() {
        environment_vars.remove(output_name);
    }
    for &key in NIX_STRUCTURED_SYNTHETIC_ENV_KEYS {
        environment_vars.remove(key);
    }
    let attrs = canonicalize_json_value(attrs);
    let attrs_json = serde_json::to_vec(&attrs)
        .map_err(|error| crate::Error::Store(format!("cannot encode Nix structured attributes: {error}")))?;
    let attrs_sh = render_structured_attrs_shell(&attrs)?;
    if attrs_json.len() > MAX_STRUCTURED_ATTRS_BYTES || attrs_sh.len() > MAX_STRUCTURED_ATTRS_BYTES {
        return Err(crate::Error::Store("materialized Nix structured attributes exceed the byte limit".to_string()));
    }

    let sh_sandbox_path = format!("{NIX_BUILD_DIRECTORY}/{NIX_ATTRS_SH_FILE_NAME}");
    let json_sandbox_path = format!("{NIX_BUILD_DIRECTORY}/{NIX_ATTRS_JSON_FILE_NAME}");
    environment_vars.insert(NIX_ATTRS_SH_ENV.to_string(), sh_sandbox_path.into_bytes());
    environment_vars.insert(NIX_ATTRS_JSON_ENV.to_string(), json_sandbox_path.into_bytes());
    let additional_files = vec![
        AdditionalFile {
            path: PathBuf::from(PASS_AS_FILE_DIRECTORY).join(NIX_ATTRS_JSON_FILE_NAME),
            contents: Bytes::from(attrs_json),
        },
        AdditionalFile {
            path: PathBuf::from(PASS_AS_FILE_DIRECTORY).join(NIX_ATTRS_SH_FILE_NAME),
            contents: Bytes::from(attrs_sh),
        },
    ];
    debug_assert_eq!(additional_files.len(), STRUCTURED_ATTRS_FILE_COUNT);
    Ok(StructuredAttrsExpansion {
        environment_vars,
        additional_files,
    })
}

fn validate_structured_output_names(
    declared_outputs: &serde_json::Value,
    outputs: &BTreeMap<String, Output>,
) -> Result<(), crate::Error> {
    let names = declared_outputs
        .as_array()
        .ok_or_else(|| crate::Error::Store("Nix structured attribute outputs must be an array".to_string()))?;
    let mut declared = BTreeSet::new();
    for value in names {
        let name = value
            .as_str()
            .ok_or_else(|| crate::Error::Store("Nix structured attribute output names must be strings".to_string()))?;
        if !declared.insert(name) {
            return Err(crate::Error::Store(format!("Nix structured attributes contain duplicate output '{name}'")));
        }
    }
    let actual = outputs.keys().map(String::as_str).collect::<BTreeSet<_>>();
    if declared != actual {
        return Err(crate::Error::Store(
            "Nix structured attribute outputs do not match the derivation outputs".to_string(),
        ));
    }
    Ok(())
}

fn structured_output_paths(
    outputs: &BTreeMap<String, Output>,
    environment_vars: &BTreeMap<String, Vec<u8>>,
    store_dir: &str,
) -> Result<serde_json::Map<String, serde_json::Value>, crate::Error> {
    let mut paths = serde_json::Map::new();
    for (name, output) in outputs {
        let declared_path = output.path_str_with_prefix(store_dir);
        let path = if declared_path.is_empty() {
            let raw = environment_vars
                .get(name)
                .ok_or_else(|| crate::Error::Store(format!("Nix structured attribute output '{name}' has no path")))?;
            std::str::from_utf8(raw)
                .map_err(|_| crate::Error::Store(format!("Nix structured attribute output '{name}' is not UTF-8")))?
                .to_string()
        } else {
            declared_path.into_owned()
        };
        let in_active_store = path.strip_prefix(store_dir).is_some_and(|suffix| suffix.starts_with('/'));
        if !in_active_store {
            return Err(crate::Error::Store(format!(
                "Nix structured attribute output '{name}' escapes the active store"
            )));
        }
        paths.insert(name.clone(), serde_json::Value::String(path));
    }
    debug_assert_eq!(paths.len(), outputs.len());
    Ok(paths)
}

fn canonicalize_json_value(value: serde_json::Value) -> serde_json::Value {
    match value {
        serde_json::Value::Array(values) => {
            serde_json::Value::Array(values.into_iter().map(canonicalize_json_value).collect())
        }
        serde_json::Value::Object(values) => {
            let ordered = values.into_iter().collect::<BTreeMap<_, _>>();
            let mut canonical = serde_json::Map::new();
            for (key, value) in ordered {
                canonical.insert(key, canonicalize_json_value(value));
            }
            serde_json::Value::Object(canonical)
        }
        scalar => scalar,
    }
}

fn render_structured_attrs_shell(attrs: &serde_json::Value) -> Result<Vec<u8>, crate::Error> {
    let object = attrs
        .as_object()
        .ok_or_else(|| crate::Error::Store("Nix structured attributes must be a JSON object".to_string()))?;
    let ordered = object.iter().collect::<BTreeMap<_, _>>();
    let mut rendered = String::new();
    for (name, value) in ordered {
        if !is_shell_identifier(name) {
            continue;
        }
        if let Some(scalar) = shell_scalar(value) {
            rendered.push_str("declare ");
            rendered.push_str(name);
            rendered.push('=');
            rendered.push_str(&scalar);
            rendered.push('\n');
            continue;
        }
        if let Some(values) = value.as_array().filter(|values| values.iter().all(|value| shell_scalar(value).is_some()))
        {
            rendered.push_str("declare -a ");
            rendered.push_str(name);
            rendered.push_str("=(");
            for value in values {
                rendered.push_str(&shell_scalar(value).expect("validated scalar"));
                rendered.push(' ');
            }
            rendered.push(')');
            rendered.push('\n');
            continue;
        }
        if let Some(values) =
            value.as_object().filter(|values| values.values().all(|value| shell_scalar(value).is_some()))
        {
            let ordered_values = values.iter().collect::<BTreeMap<_, _>>();
            rendered.push_str("declare -A ");
            rendered.push_str(name);
            rendered.push_str("=(");
            for (key, value) in ordered_values {
                rendered.push('[');
                rendered.push_str(&shell_quote(key));
                rendered.push_str("]=");
                rendered.push_str(&shell_scalar(value).expect("validated scalar"));
                rendered.push(' ');
            }
            rendered.push_str(")\n");
        }
    }
    Ok(rendered.into_bytes())
}

fn is_shell_identifier(value: &str) -> bool {
    let mut bytes = value.bytes();
    let Some(first) = bytes.next() else {
        return false;
    };
    if !(first.is_ascii_alphabetic() || first == b'_') {
        return false;
    }
    bytes.all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
}

fn shell_scalar(value: &serde_json::Value) -> Option<String> {
    match value {
        serde_json::Value::Null => Some("''".to_string()),
        serde_json::Value::Bool(true) => Some("1".to_string()),
        serde_json::Value::Bool(false) => Some(String::new()),
        serde_json::Value::Number(value) => Some(value.to_string()),
        serde_json::Value::String(value) => Some(shell_quote(value)),
        serde_json::Value::Array(_) | serde_json::Value::Object(_) => None,
    }
}

fn shell_quote(value: &str) -> String {
    let mut quoted = String::with_capacity(value.len().saturating_add(SHELL_QUOTE_COUNT));
    quoted.push('\'');
    for character in value.chars() {
        if character == '\'' {
            quoted.push_str("'\\''");
        } else {
            quoted.push(character);
        }
    }
    quoted.push('\'');
    quoted
}

fn merge_additional_files(
    left: Vec<AdditionalFile>,
    right: Vec<AdditionalFile>,
) -> Result<Vec<AdditionalFile>, crate::Error> {
    let expected_count = left.len().saturating_add(right.len());
    let mut merged = BTreeMap::new();
    for file in left.into_iter().chain(right) {
        if merged.insert(file.path.clone(), file.contents).is_some() {
            return Err(crate::Error::Store(format!("additional build file path collision: {}", file.path.display())));
        }
    }
    let files = merged.into_iter().map(|(path, contents)| AdditionalFile { path, contents }).collect::<Vec<_>>();
    debug_assert_eq!(files.len(), expected_count);
    Ok(files)
}

/// Expand Nix `passAsFile` declarations without I/O.
///
/// The file name uses Nix's SHA-256 and nixbase32 protocol. This SHA-256 use is
/// required for Nix interoperability; Mantle identities continue to use BLAKE3.
fn expand_pass_as_file(mut environment_vars: BTreeMap<String, Vec<u8>>) -> Result<PassAsFileExpansion, crate::Error> {
    let Some(raw_names) = environment_vars.get(PASS_AS_FILE_ENV) else {
        return Ok(PassAsFileExpansion {
            environment_vars,
            additional_files: Vec::new(),
        });
    };
    let names = std::str::from_utf8(raw_names)
        .map_err(|_| crate::Error::Store("passAsFile contains non-UTF-8 bytes".to_string()))?
        .to_string();
    let mut additional_files = BTreeMap::new();

    for name in names.split(' ') {
        let contents = environment_vars.remove(name).ok_or_else(|| {
            crate::Error::Store(format!("passAsFile refers to missing environment variable '{name}'"))
        })?;
        let (path_variable, sandbox_path, relative_path) = pass_as_file_paths(name);
        if additional_files.insert(relative_path, Bytes::from(contents)).is_some() {
            return Err(crate::Error::Store(format!("passAsFile path collision for environment variable '{name}'")));
        }
        environment_vars.insert(path_variable, sandbox_path.into_bytes());
    }

    let additional_files: Vec<AdditionalFile> =
        additional_files.into_iter().map(|(path, contents)| AdditionalFile { path, contents }).collect();
    debug_assert_eq!(additional_files.len(), names.split(' ').count());
    debug_assert!(environment_vars.contains_key(PASS_AS_FILE_ENV));
    Ok(PassAsFileExpansion {
        environment_vars,
        additional_files,
    })
}

fn pass_as_file_paths(name: &str) -> (String, String, PathBuf) {
    let digest = nixbase32::encode(&Sha256::digest(name));
    let file_name = format!("{PASS_AS_FILE_NAME_PREFIX}{digest}");
    let path_variable = format!("{name}{PASS_AS_FILE_PATH_SUFFIX}");
    let sandbox_path = format!("/{PASS_AS_FILE_DIRECTORY}/{file_name}");
    let relative_path = PathBuf::from(PASS_AS_FILE_DIRECTORY).join(file_name);
    debug_assert!(sandbox_path.starts_with('/'));
    debug_assert!(!relative_path.is_absolute());
    (path_variable, sandbox_path, relative_path)
}

/// Build command args with placeholders expanded.
fn build_command_args(derivation: &Derivation, store_dir: &str) -> Vec<String> {
    let mut command_args: Vec<String> = Vec::with_capacity(derivation.arguments.len().saturating_add(1));
    command_args.push(derivation.builder.clone());
    for arg in &derivation.arguments {
        command_args.push(replace_placeholders(arg, &derivation.outputs, store_dir));
    }
    command_args
}

fn default_sandbox_environment(store_dir: &str) -> BTreeMap<String, Vec<u8>> {
    const MAX_SANDBOX_VARS: usize = 32;
    assert!(SANDBOX_ENV_VARS.len() <= MAX_SANDBOX_VARS, "SANDBOX_ENV_VARS exceeds bound");
    SANDBOX_ENV_VARS
        .iter()
        .map(|(key, value)| {
            let val = if *key == "NIX_STORE" {
                store_dir.as_bytes().to_vec()
            } else {
                value.as_bytes().to_vec()
            };
            (key.to_string(), val)
        })
        .collect()
}

fn overlay_derivation_environment(
    derivation: &Derivation,
    store_dir: &str,
    hermeticity_mode: HermeticityMode,
    environment_vars: &mut BTreeMap<String, Vec<u8>>,
) -> Result<Vec<HermeticityAuditEvent>, crate::Error> {
    assert!(!derivation.outputs.is_empty(), "derivation must have at least one output");
    assert!(!environment_vars.is_empty(), "sandbox env must be pre-populated");
    let mut audit_events = Vec::with_capacity(derivation.environment.len());
    for (key, value) in &derivation.environment {
        if key == WORKSPACE_POLICY_ENV || key == WORKSPACE_LEASE_ENV {
            continue;
        }
        reject_denied_strict_environment_key(derivation, hermeticity_mode, environment_vars.len(), key)?;
        let replaced = replace_placeholders_bstr(value, &derivation.outputs, store_dir);
        if let Some(sandbox_value) = environment_vars.get(key)
            && sandbox_value.as_slice() != <BString as AsRef<[u8]>>::as_ref(&replaced)
            && is_protected_sandbox_env_key(key)
        {
            let sandbox_value = sandbox_value.clone();
            if hermeticity_mode.is_strict() {
                validate_strict_protected_override(StrictProtectedOverrideInput {
                    derivation,
                    store_dir,
                    accepted_variable_count: environment_vars.len(),
                    key,
                    sandbox_value: &sandbox_value,
                    value: replaced.as_ref(),
                })?;
            } else {
                audit_events.push(HermeticityAuditEvent::new(
                    HermeticityAuditKind::EnvironmentOverride,
                    format_environment_override_detail(key, &sandbox_value, replaced.as_ref()),
                ));
            }
        }
        environment_vars.insert(key.clone(), Vec::from(replaced));
    }
    Ok(audit_events)
}

fn reject_denied_strict_environment_key(
    derivation: &Derivation,
    hermeticity_mode: HermeticityMode,
    accepted_variable_count: usize,
    key: &str,
) -> Result<(), crate::Error> {
    if !hermeticity_mode.is_strict() {
        return Ok(());
    }
    if key == PATH_VARIABLE {
        return Ok(());
    }
    let action_name = environment_policy::action_name_from_environment(&derivation.environment);
    if let Some(denied) = environment_policy::denied_environment_variable(action_name, accepted_variable_count, key) {
        return Err(denied_environment_error(denied));
    }
    Ok(())
}

fn validate_strict_protected_override(input: StrictProtectedOverrideInput<'_>) -> Result<(), crate::Error> {
    if input.key != PATH_VARIABLE {
        return Err(crate::Error::UnsafeEnvOverride {
            key: input.key.to_string(),
            sandbox_value: format_env_value(input.sandbox_value),
            derivation_value: format_env_value(input.value),
        });
    }
    let value = std::str::from_utf8(input.value).map_err(|_| {
        search_path_denied_error(
            input.derivation,
            input.accepted_variable_count,
            "receipt-bound-path-non-utf8".to_string(),
        )
    })?;
    let search_path_evidence = receipt_bound_search_path_from_value(ReceiptBoundSearchPathInput {
        value,
        store_dir: input.store_dir,
    })
    .map_err(|err| search_path_denied_error(input.derivation, input.accepted_variable_count, err.to_string()))?;
    debug_assert!(search_path_evidence.digest_blake3.is_some());
    debug_assert!(!search_path_evidence.entries.is_empty());
    Ok(())
}

fn strict_search_path_report(
    environment_vars: &BTreeMap<String, Vec<u8>>,
    store_dir: &str,
    hermeticity_mode: HermeticityMode,
) -> Result<Option<environment_policy::BuildSearchPathReport>, crate::Error> {
    if !hermeticity_mode.is_strict() {
        return Ok(None);
    }
    let path_value = environment_vars
        .get(PATH_VARIABLE)
        .ok_or_else(|| crate::Error::Store("strict search path missing PATH".to_string()))?;
    let path_value = std::str::from_utf8(path_value)
        .map_err(|_| crate::Error::Store("strict search path contains non-UTF-8 bytes".to_string()))?;
    let search_path_evidence = receipt_bound_search_path_from_value(ReceiptBoundSearchPathInput {
        value: path_value,
        store_dir,
    })
    .map_err(|err| crate::Error::Store(err.to_string()))?;
    Ok(Some(search_path_evidence))
}

fn strict_determinism_report(
    environment_vars: &BTreeMap<String, Vec<u8>>,
    hermeticity_mode: HermeticityMode,
) -> Result<Option<environment_policy::BuildDeterminismNormalizationReport>, crate::Error> {
    if !hermeticity_mode.is_strict() {
        return Ok(None);
    }
    let controls = strict_determinism_controls(environment_vars)?;
    let determinism_evidence =
        environment_policy::plan_determinism_normalization(environment_policy::DeterminismNormalizationRequest {
            controls,
            unsupported_controls: Vec::new(),
            divergence: None,
        })
        .map_err(|err| crate::Error::Store(err.to_string()))?;
    Ok(Some(determinism_evidence))
}

fn strict_determinism_controls(
    environment_vars: &BTreeMap<String, Vec<u8>>,
) -> Result<Vec<environment_policy::BuildDeterminismControl>, crate::Error> {
    let mut controls = Vec::with_capacity(STRICT_DETERMINISM_CONTROL_COUNT);
    controls.push(determinism_control(DeterminismControlInput {
        surface: DETERMINISM_SURFACE_TIME,
        policy: DETERMINISM_POLICY_FIXED_ENV,
        value: format!("SOURCE_DATE_EPOCH={}", env_value(environment_vars, "SOURCE_DATE_EPOCH")?),
    }));
    controls.push(determinism_control(DeterminismControlInput {
        surface: DETERMINISM_SURFACE_TIMEZONE,
        policy: DETERMINISM_POLICY_FIXED_ENV,
        value: format!("TZ={}", env_value(environment_vars, "TZ")?),
    }));
    controls.push(determinism_control(DeterminismControlInput {
        surface: DETERMINISM_SURFACE_LOCALE,
        policy: DETERMINISM_POLICY_FIXED_ENV,
        value: format!(
            "LANG={};LC_ALL={}",
            env_value(environment_vars, "LANG")?,
            env_value(environment_vars, "LC_ALL")?
        ),
    }));
    controls.push(determinism_control(DeterminismControlInput {
        surface: DETERMINISM_SURFACE_TEMP_ROOTS,
        policy: DETERMINISM_POLICY_FIXED_ENV,
        value: format!(
            "TEMP={};TEMPDIR={};TMP={};TMPDIR={}",
            env_value(environment_vars, "TEMP")?,
            env_value(environment_vars, "TEMPDIR")?,
            env_value(environment_vars, "TMP")?,
            env_value(environment_vars, "TMPDIR")?
        ),
    }));
    controls.push(determinism_control(DeterminismControlInput {
        surface: DETERMINISM_SURFACE_HOST_USER,
        policy: DETERMINISM_POLICY_FIXED_ENV,
        value: format!(
            "HOME={};LOGNAME={};USER={}",
            env_value(environment_vars, "HOME")?,
            env_value(environment_vars, "LOGNAME")?,
            env_value(environment_vars, "USER")?
        ),
    }));
    controls.push(determinism_control(DeterminismControlInput {
        surface: DETERMINISM_SURFACE_UMASK,
        policy: DETERMINISM_POLICY_FIXED_EXECUTOR,
        value: STRICT_UMASK_VALUE.to_string(),
    }));
    controls.push(determinism_control(DeterminismControlInput {
        surface: DETERMINISM_SURFACE_RANDOMNESS,
        policy: DETERMINISM_POLICY_MODELED,
        value: STRICT_RANDOMNESS_VALUE.to_string(),
    }));
    controls.push(determinism_control(DeterminismControlInput {
        surface: DETERMINISM_SURFACE_ORDERING,
        policy: DETERMINISM_POLICY_MODELED,
        value: STRICT_ORDERING_VALUE.to_string(),
    }));
    debug_assert_eq!(controls.len(), STRICT_DETERMINISM_CONTROL_COUNT);
    Ok(controls)
}

fn determinism_control(input: DeterminismControlInput<'_>) -> environment_policy::BuildDeterminismControl {
    environment_policy::BuildDeterminismControl {
        surface: input.surface.to_string(),
        policy: input.policy.to_string(),
        value: input.value,
        enforcement: environment_policy::DETERMINISM_ENFORCEMENT_ENFORCED.to_string(),
    }
}

fn env_value(environment_vars: &BTreeMap<String, Vec<u8>>, key: &str) -> Result<String, crate::Error> {
    let value = environment_vars
        .get(key)
        .ok_or_else(|| crate::Error::Store(format!("strict determinism normalization missing {key}")))?;
    Ok(format_env_value(value))
}

fn receipt_bound_search_path_from_value(
    input: ReceiptBoundSearchPathInput<'_>,
) -> Result<environment_policy::BuildSearchPathReport, environment_policy::SearchPathPlanError> {
    let entry_count_max = input.value.split(PATH_ENTRY_SEPARATOR).count();
    debug_assert!(entry_count_max > 0);
    debug_assert!(entry_count_max <= input.value.len().saturating_add(1));
    if input.value == SANDBOX_PATH_NOT_SET {
        return environment_policy::plan_receipt_bound_search_path(environment_policy::SearchPathPlanRequest::default());
    }
    let mut entries = Vec::with_capacity(entry_count_max);
    let mut ambient_entries = Vec::with_capacity(entry_count_max);
    for entry in input.value.split(PATH_ENTRY_SEPARATOR) {
        match declared_tool_path_entry(DeclaredToolPathInput {
            entry,
            store_dir: input.store_dir,
        }) {
            Some(real_tool_ref) => entries.push(environment_policy::SearchPathEntryDeclaration {
                path: entry.to_string(),
                kind: environment_policy::SEARCH_PATH_ENTRY_DECLARED_TOOL.to_string(),
                real_tool_ref,
            }),
            None => ambient_entries.push(entry.to_string()),
        }
    }
    environment_policy::plan_receipt_bound_search_path(environment_policy::SearchPathPlanRequest {
        entries,
        ambient_entries,
    })
}

fn declared_tool_path_entry(input: DeclaredToolPathInput<'_>) -> Option<String> {
    if input.entry.is_empty() || has_non_normal_path_segment(input.entry) {
        return None;
    }
    let store_prefix = format!("{}/", input.store_dir);
    let rest = input.entry.strip_prefix(&store_prefix)?;
    let component = rest.split('/').next()?;
    if component.len() < MIN_STORE_COMPONENT_CHARS {
        return None;
    }
    Some(format!("{store_prefix}{component}"))
}

fn has_non_normal_path_segment(path: &str) -> bool {
    path.split('/').skip(1).any(|segment| segment.is_empty() || segment == "." || segment == "..")
}

fn search_path_denied_error(
    derivation: &Derivation,
    accepted_variable_count: usize,
    diagnostic: String,
) -> crate::Error {
    let action_name = environment_policy::action_name_from_environment(&derivation.environment);
    let denied = environment_policy::denied_search_path_variable(action_name, accepted_variable_count, diagnostic);
    denied_environment_error(denied)
}

fn denied_environment_error(denied: environment_policy::DeniedEnvironmentVariable) -> crate::Error {
    crate::Error::DeniedEnvironmentVariable {
        action_name: denied.action_name,
        variable: denied.rejection.variable,
        class: denied.rejection.class,
        diagnostic: denied.rejection.diagnostic,
        report: Box::new(denied.report),
    }
}

fn is_protected_sandbox_env_key(key: &str) -> bool {
    let is_sandbox_key = SANDBOX_ENV_VARS.iter().any(|(sandbox_key, _)| *sandbox_key == key);
    if !is_sandbox_key {
        return false;
    }
    !ALLOWED_SANDBOX_ENV_OVERRIDES.contains(&key)
}

fn format_environment_override_detail(key: &str, sandbox_value: &[u8], derivation_value: &[u8]) -> String {
    format!("{key}: {:?} -> {:?}", format_env_value(sandbox_value), format_env_value(derivation_value))
}

fn format_env_value(value: &[u8]) -> String {
    String::from_utf8_lossy(value).into_owned()
}

/// Build sandbox constraints from derivation properties.
fn build_constraints(
    derivation: &Derivation,
    execution_profile: &ExecutionProfile,
    allow_network: bool,
) -> HashSet<BuildConstraints> {
    let mut constraints = HashSet::from([BuildConstraints::System(derivation.system.clone())]);
    if execution_profile.provide_bin_sh {
        constraints.insert(BuildConstraints::ProvideBinSh);
    }
    if execution_profile.profile_id == NIX_FOREIGN_PROFILE_ID {
        constraints.insert(BuildConstraints::ProvideProcMetadata);
        constraints.insert(BuildConstraints::ProvideRandomDevices);
    }
    if execution_profile.resource_limits.min_memory_bytes > 0 {
        constraints.insert(BuildConstraints::MinMemory(execution_profile.resource_limits.min_memory_bytes));
    }
    if allow_network {
        constraints.insert(BuildConstraints::NetworkAccess);
    }
    constraints
}

fn execution_profile_scratch_paths(execution_profile: &ExecutionProfile, store_dir: &str) -> Vec<PathBuf> {
    let mut paths = execution_profile.writable_prefixes.iter().map(PathBuf::from).collect::<BTreeSet<_>>();
    paths.insert(PathBuf::from(&store_dir[1..]));
    paths.into_iter().collect()
}

/// Build refscan needles from output and input store path digests.
fn build_refscan_needles(derivation: &Derivation, inputs: &BTreeMap<StorePath<String>, Node>) -> Vec<String> {
    derivation
        .outputs
        .values()
        .filter_map(|o| o.path.as_ref())
        .map(|p| nixbase32::encode(p.digest()))
        .chain(inputs.keys().map(|p| nixbase32::encode(p.digest())))
        .collect()
}

/// Map derivation outputs to sandbox-relative paths.
/// CA outputs use the placeholder from the environment; input-addressed
/// outputs use the pre-computed store path.
fn map_outputs_to_sandbox_paths(derivation: &Derivation, store_dir: &str) -> Vec<PathBuf> {
    derivation
        .outputs
        .iter()
        .map(|(output_name, o)| {
            let path_str = o.path_str_with_prefix(store_dir);
            if path_str.is_empty() {
                let placeholder = derivation
                    .environment
                    .get(output_name)
                    .map(|v| String::from_utf8_lossy(v).to_string())
                    .unwrap_or_default();
                if let Some(stripped) = placeholder.strip_prefix('/') {
                    PathBuf::from(stripped)
                } else {
                    PathBuf::from(&placeholder)
                }
            } else {
                PathBuf::from(&path_str[1..])
            }
        })
        .collect()
}

/// Convert input store paths to castore PathComponent keys.
fn map_inputs_to_components(
    inputs: &BTreeMap<StorePath<String>, Node>,
) -> Result<BTreeMap<snix_castore::PathComponent, Node>, crate::Error> {
    inputs
        .iter()
        .map(|(path, node)| {
            let component = path
                .to_string()
                .as_str()
                .try_into()
                .map_err(|e| crate::Error::Store(format!("invalid store path component '{}': {e}", path)))?;
            Ok((component, node.clone()))
        })
        .collect()
}

/// Collect all store paths that must be visible in the sandbox.
///
/// For `input_sources`, the store path itself is needed.
/// For `input_derivations`, we need the output paths of each referenced
/// derivation output.
pub fn collect_input_paths(
    derivation: &Derivation,
    known_paths: &DerivationRegistry,
) -> Result<BTreeSet<StorePath<String>>, crate::Error> {
    assert!(!derivation.outputs.is_empty(), "derivation must have outputs");
    assert!(!known_paths.store_dir().is_empty(), "store dir must not be empty");
    let mut paths = BTreeSet::new();

    // Source inputs
    for source in &derivation.input_sources {
        paths.insert(source.clone());
    }

    // Derivation input outputs
    for (drv_path, output_names) in &derivation.input_derivations {
        let drv_abs = drv_path.to_absolute_path_with_prefix(known_paths.store_dir());
        // Verify the derivation is in DerivationRegistry
        if known_paths.get_by_drv_path(&drv_abs).is_none() {
            return Err(crate::Error::DerivationNotFound { path: drv_path.clone() });
        }
        for output_name in output_names {
            // Use get_output_path which handles both input-addressed
            // (reads from derivation.outputs[].path) and content-addressed
            // (reads from resolved_outputs).
            let output_path =
                known_paths.get_output_path(&drv_abs, output_name).ok_or_else(|| crate::Error::OutputNoPath {
                    output: output_name.clone(),
                    drv_name: drv_path.to_string(),
                })?;
            paths.insert(output_path);
        }
    }

    Ok(paths)
}

/// Compute the SHA-256 placeholder required by the Nix derivation protocol.
///
/// Mantle uses BLAKE3 for its own placeholder domain. Imported Nix ATerm bytes
/// retain this interoperability value and require replacement at execution.
fn nix_compatible_hash_placeholder(name: &str) -> String {
    let digest = Sha256::digest(format!("nix-output:{name}").as_bytes());
    format!("/{}", nixbase32::encode(&digest))
}

fn output_placeholders(name: &str) -> [String; 2] {
    [hash_placeholder(name), nix_compatible_hash_placeholder(name)]
}

/// Replace Mantle and Nix-compatible output placeholders with actual output paths.
fn replace_placeholders(s: &str, outputs: &BTreeMap<String, Output>, store_dir: &str) -> String {
    let mut result = s.to_owned();
    for (name, output) in outputs {
        if let Some(path) = output.path.as_ref() {
            let output_path = path.to_absolute_path_with_prefix(store_dir);
            for placeholder in output_placeholders(name) {
                result = result.replace(&placeholder, &output_path);
            }
        }
    }
    result
}

/// Replace Mantle and Nix-compatible placeholders in a BString.
fn replace_placeholders_bstr(s: &BString, outputs: &BTreeMap<String, Output>, store_dir: &str) -> BString {
    use bstr::ByteSlice;
    let mut result = s.clone();
    for (name, output) in outputs {
        if let Some(path) = output.path.as_ref() {
            let output_path = path.to_absolute_path_with_prefix(store_dir);
            for placeholder in output_placeholders(name) {
                result = result.replace(placeholder.as_bytes(), output_path.as_bytes()).into();
            }
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use nix_compat::derivation::Derivation;

    use super::*;

    const NIX_OUT_PLACEHOLDER: &str = "/1rz4g4znpzjwh1xymhjpm42vipw92pr73vdgl6xs1hycac8kf2n9";
    const UNKNOWN_PLACEHOLDER: &str = "/0000000000000000000000000000000000000000000000000000";
    const NIX_STORE_DIR: &str = "/nix/store";
    const MANTLE_STORE_DIR: &str = "/mantle/store";
    const BAR_PASS_AS_FILE_PATH: &str = "/build/.attr-1fcgpy7vc4ammr7s17j2xq88scswkgz23dqzc04g8sx5vcp2pppw";
    const BAZ_PASS_AS_FILE_PATH: &str = "/build/.attr-15l04iksj1280dvhbzdq9ai3wlf8ac2188m9qv0gn81k9nba19ds";
    const INVALID_UTF8_BYTE: u8 = 0xff;

    fn derivation_to_build_request(
        derivation: &Derivation,
        inputs: &BTreeMap<StorePath<String>, Node>,
        store_dir: &str,
        hermeticity_mode: HermeticityMode,
    ) -> Result<BuildRequestEnvelope, crate::Error> {
        super::derivation_to_build_request(
            derivation,
            inputs,
            store_dir,
            &ExecutionProfile::native_compatibility(),
            hermeticity_mode,
        )
    }

    // ── Helper: build a derivation and register in DerivationRegistry ──

    fn make_drv_with_name(name: &str) -> Derivation {
        let mut outputs = BTreeMap::new();
        outputs.insert("out".to_string(), Output {
            path: None,
            ca_hash: None,
        });
        let mut environment = BTreeMap::new();
        environment.insert("name".to_string(), name.into());
        environment.insert("system".to_string(), "x86_64-linux".into());
        environment.insert("builder".to_string(), "/bin/sh".into());
        environment.insert("out".to_string(), "".into());

        let mut drv = Derivation {
            arguments: vec!["-c".into(), "echo > $out".into()],
            builder: "/bin/sh".to_string(),
            environment,
            input_derivations: BTreeMap::new(),
            input_sources: BTreeSet::new(),
            outputs,
            system: "x86_64-linux".to_string(),
        };
        let hdm = drv.hash_derivation_modulo(|_| panic!("no parent"));
        drv.calculate_output_paths(name, &hdm).unwrap();
        let _ = drv.calculate_derivation_path(name).unwrap();
        drv
    }

    fn register_drv(name: &str, kp: &mut DerivationRegistry) -> (StorePath<String>, Derivation) {
        let drv = make_drv_with_name(name);
        // Use drv name bytes as a unique fake aterm hash
        let mut fake_hash = [0u8; 32];
        for (i, b) in name.bytes().enumerate().take(32) {
            fake_hash[i] = b;
        }
        let hdm = drv.hash_derivation_modulo(|_| panic!("no parent"));
        let drv_path = drv.calculate_derivation_path(name).unwrap();
        kp.insert(drv_path.clone(), hdm, drv.clone(), false, None);
        (drv_path, drv)
    }

    /// Construct a minimal Derivation for testing.
    fn test_derivation() -> Derivation {
        let mut outputs = BTreeMap::new();
        outputs.insert("out".to_string(), Output {
            path: None,
            ca_hash: None,
        });

        let mut environment = BTreeMap::new();
        environment.insert("name".to_string(), "test".into());
        environment.insert("system".to_string(), "x86_64-linux".into());
        environment.insert("builder".to_string(), "/bin/sh".into());
        environment.insert("out".to_string(), "".into());

        let mut drv = Derivation {
            arguments: vec!["-c".to_string(), "echo hello > $out".to_string()],
            builder: "/bin/sh".to_string(),
            environment,
            input_derivations: BTreeMap::new(),
            input_sources: BTreeSet::new(),
            outputs,
            system: "x86_64-linux".to_string(),
        };

        // Compute paths so outputs have real values
        let hdm = drv.hash_derivation_modulo(|_| panic!("no parent derivations"));
        drv.calculate_output_paths("test", &hdm).unwrap();
        let _ = drv.calculate_derivation_path("test").unwrap();

        drv
    }

    const OVERRIDE_CORE_COUNT: &str = "4";
    const OVERRIDE_SOURCE_DATE_EPOCH: &str = "315532800";

    fn normalize_env_map(
        overrides: &[(&str, &str)],
        hermeticity_mode: HermeticityMode,
    ) -> Result<NormalizedBuildEnvironment, crate::Error> {
        let mut drv = test_derivation();
        for (key, value) in overrides {
            drv.environment.insert((*key).to_string(), (*value).into());
        }
        normalize_build_environment(&drv, "/nix/store", hermeticity_mode)
    }

    #[test]
    fn build_request_has_correct_builder() {
        let drv = test_derivation();
        let req = derivation_to_build_request(&drv, &BTreeMap::new(), "/nix/store", HermeticityMode::Practical)
            .unwrap()
            .build_request;

        assert_eq!(req.command_args[0], "/bin/sh");
        assert_eq!(req.command_args[1], "-c");
    }

    #[test]
    fn structured_attrs_materialization_matches_nix_shell_and_json_protocols() {
        let drv = test_derivation();
        let output_path = drv.outputs["out"].path_str_with_prefix(MANTLE_STORE_DIR).into_owned();
        let structured = serde_json::json!({
            "builder": "/mantle/store/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa-bash/bin/bash",
            "count": 42,
            "disabled": false,
            "emptyWords": [],
            "enabled": true,
            "mapScalars": {"count": 7, "enabled": true, "nullable": null},
            "nested": {"values": [1, false]},
            "nullable": null,
            "outputs": ["out"],
            "tricky": "a'b\n$HOME\\z",
            "words": ["one", "two words"],
        });
        let environment = BTreeMap::from([
            (NIX_STRUCTURED_ATTRS_ENV.to_string(), serde_json::to_vec(&structured).unwrap()),
            ("out".to_string(), output_path.as_bytes().to_vec()),
        ]);

        let expansion = materialize_structured_attrs(environment, &drv.outputs, MANTLE_STORE_DIR).unwrap();
        let files = expansion
            .additional_files
            .iter()
            .map(|file| (file.path.as_path(), file.contents.as_ref()))
            .collect::<BTreeMap<_, _>>();
        let json_path = PathBuf::from(PASS_AS_FILE_DIRECTORY).join(NIX_ATTRS_JSON_FILE_NAME);
        let shell_path = PathBuf::from(PASS_AS_FILE_DIRECTORY).join(NIX_ATTRS_SH_FILE_NAME);
        let attrs_json: serde_json::Value =
            serde_json::from_slice(files.get(json_path.as_path()).expect("structured JSON payload")).unwrap();
        let attrs_sh = std::str::from_utf8(files.get(shell_path.as_path()).expect("structured shell payload")).unwrap();
        let expected_shell = format!(
            "declare builder='/mantle/store/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa-bash/bin/bash'\n\
             declare count=42\n\
             declare disabled=\n\
             declare -a emptyWords=()\n\
             declare enabled=1\n\
             declare -A mapScalars=(['count']=7 ['enabled']=1 ['nullable']='' )\n\
             declare nullable=''\n\
             declare -A outputs=(['out']='{output_path}' )\n\
             declare tricky='a'\\''b\n$HOME\\z'\n\
             declare -a words=('one' 'two words' )\n"
        );

        assert_eq!(attrs_json["outputs"]["out"], output_path);
        assert_eq!(attrs_json["nested"]["values"], serde_json::json!([1, false]));
        assert_eq!(attrs_sh, expected_shell);
        assert_eq!(expansion.environment_vars.get(NIX_STRUCTURED_ATTRS_ENV), None);
        assert_eq!(expansion.environment_vars.get("out"), None);
        assert_eq!(expansion.environment_vars[NIX_ATTRS_SH_ENV], b"/build/.attrs.sh");
        assert_eq!(expansion.environment_vars[NIX_ATTRS_JSON_ENV], b"/build/.attrs.json");
    }

    #[test]
    fn structured_attrs_materialization_rejects_output_identity_mismatch() {
        let drv = test_derivation();
        let structured = serde_json::json!({"outputs": ["dev"]});
        let environment =
            BTreeMap::from([(NIX_STRUCTURED_ATTRS_ENV.to_string(), serde_json::to_vec(&structured).unwrap())]);

        let error = materialize_structured_attrs(environment, &drv.outputs, MANTLE_STORE_DIR).unwrap_err();

        assert!(error.to_string().contains("outputs do not match"));
    }

    #[test]
    fn structured_attrs_materialization_rejects_malformed_protocol_json() {
        let drv = test_derivation();
        let environment = BTreeMap::from([(NIX_STRUCTURED_ATTRS_ENV.to_string(), b"{\"outputs\":[\"out\"]".to_vec())]);

        let error = materialize_structured_attrs(environment, &drv.outputs, MANTLE_STORE_DIR).unwrap_err();

        assert!(error.to_string().contains("invalid Nix structured attributes"));
    }

    #[test]
    fn structured_attrs_payload_rewrites_output_placeholders_before_materialization() {
        let profile = ExecutionProfile::foreign_nix();
        let mut drv = test_derivation();
        let structured = serde_json::json!({
            "message": NIX_OUT_PLACEHOLDER,
            "outputs": ["out"],
        });
        drv.environment
            .insert(NIX_STRUCTURED_ATTRS_ENV.to_string(), serde_json::to_vec(&structured).unwrap().into());
        crate::bind_execution_profile(&mut drv, &profile).unwrap();
        let expected_output = drv.outputs["out"].path_str_with_prefix(MANTLE_STORE_DIR).into_owned();

        let request = super::derivation_to_build_request(
            &drv,
            &BTreeMap::new(),
            MANTLE_STORE_DIR,
            &profile,
            HermeticityMode::Strict,
        )
        .unwrap()
        .build_request;
        let attrs_json = request
            .additional_files
            .iter()
            .find(|file| file.path.ends_with(NIX_ATTRS_JSON_FILE_NAME))
            .expect("structured JSON file");
        let attrs: serde_json::Value = serde_json::from_slice(&attrs_json.contents).unwrap();

        assert_eq!(attrs["message"], expected_output);
        assert!(!request.environment_vars.iter().any(|entry| entry.key == NIX_STRUCTURED_ATTRS_ENV));
        assert!(!request.environment_vars.iter().any(|entry| entry.key == "out"));
        assert!(
            NIX_STRUCTURED_SYNTHETIC_ENV_KEYS
                .iter()
                .all(|key| !request.environment_vars.iter().any(|entry| entry.key == *key))
        );
    }

    #[test]
    fn pass_as_file_expansion_matches_nix_protocol_paths_and_contents() {
        let environment_vars = BTreeMap::from([
            ("bar".to_string(), b"baz".to_vec()),
            ("baz".to_string(), b"bar".to_vec()),
            (PASS_AS_FILE_ENV.to_string(), b"bar baz".to_vec()),
        ]);

        let expansion = expand_pass_as_file(environment_vars).unwrap();

        assert_eq!(expansion.environment_vars.get("bar"), None);
        assert_eq!(expansion.environment_vars.get("baz"), None);
        assert_eq!(expansion.environment_vars["barPath"], BAR_PASS_AS_FILE_PATH.as_bytes());
        assert_eq!(expansion.environment_vars["bazPath"], BAZ_PASS_AS_FILE_PATH.as_bytes());
        assert_eq!(expansion.environment_vars[PASS_AS_FILE_ENV], b"bar baz");
        assert_eq!(expansion.additional_files.len(), 2);
        assert_eq!(expansion.additional_files[0].path, PathBuf::from(&BAZ_PASS_AS_FILE_PATH[1..]));
        assert_eq!(expansion.additional_files[0].contents, Bytes::from_static(b"bar"));
        assert_eq!(expansion.additional_files[1].path, PathBuf::from(&BAR_PASS_AS_FILE_PATH[1..]));
        assert_eq!(expansion.additional_files[1].contents, Bytes::from_static(b"baz"));
    }

    #[test]
    fn pass_as_file_expansion_rejects_missing_environment_variable() {
        let environment_vars = BTreeMap::from([(PASS_AS_FILE_ENV.to_string(), b"missing".to_vec())]);

        let error = expand_pass_as_file(environment_vars).unwrap_err();

        assert!(error.to_string().contains("passAsFile refers to missing environment variable 'missing'"));
    }

    #[test]
    fn pass_as_file_expansion_rejects_non_utf8_name_list() {
        let environment_vars = BTreeMap::from([(PASS_AS_FILE_ENV.to_string(), vec![INVALID_UTF8_BYTE])]);

        let error = expand_pass_as_file(environment_vars).unwrap_err();

        assert!(error.to_string().contains("passAsFile contains non-UTF-8 bytes"));
    }

    #[test]
    fn build_request_materializes_pass_as_file_payload() {
        let mut drv = test_derivation();
        drv.environment.insert(PASS_AS_FILE_ENV.to_string(), "text".into());
        drv.environment.insert("text".to_string(), "Provide a source file".into());

        let request = derivation_to_build_request(&drv, &BTreeMap::new(), NIX_STORE_DIR, HermeticityMode::Practical)
            .unwrap()
            .build_request;

        let expected_text_path = pass_as_file_paths("text").1;
        assert!(!request.environment_vars.iter().any(|entry| entry.key == "text"));
        assert!(
            request
                .environment_vars
                .iter()
                .any(|entry| { entry.key == "textPath" && entry.value.as_ref() == expected_text_path.as_bytes() })
        );
        assert_eq!(request.additional_files.len(), 1);
        assert_eq!(request.additional_files[0].path, pass_as_file_paths("text").2);
        assert_eq!(request.additional_files[0].contents, Bytes::from_static(b"Provide a source file"));
    }

    #[test]
    fn pass_as_file_payload_rewrites_nix_output_placeholder_for_active_store() {
        let mut drv = test_derivation();
        let payload = format!("install --prefix={NIX_OUT_PLACEHOLDER}");
        drv.environment.insert(PASS_AS_FILE_ENV.to_string(), "buildCommand".into());
        drv.environment.insert("buildCommand".to_string(), payload.into());
        let expected_output = drv.outputs["out"].path.as_ref().unwrap().to_absolute_path_with_prefix(MANTLE_STORE_DIR);

        let request = derivation_to_build_request(&drv, &BTreeMap::new(), MANTLE_STORE_DIR, HermeticityMode::Practical)
            .unwrap()
            .build_request;
        let contents = &request.additional_files[0].contents;

        assert!(contents.as_ref().windows(expected_output.len()).any(|window| window == expected_output.as_bytes()));
        assert!(
            !contents
                .as_ref()
                .windows(NIX_OUT_PLACEHOLDER.len())
                .any(|window| window == NIX_OUT_PLACEHOLDER.as_bytes())
        );
        assert!(!contents.as_ref().windows(NIX_STORE_DIR.len()).any(|window| window == NIX_STORE_DIR.as_bytes()));
    }

    #[test]
    fn pass_as_file_payload_preserves_unknown_placeholder() {
        let mut drv = test_derivation();
        drv.environment.insert(PASS_AS_FILE_ENV.to_string(), "buildCommand".into());
        drv.environment.insert("buildCommand".to_string(), UNKNOWN_PLACEHOLDER.into());

        let request = derivation_to_build_request(&drv, &BTreeMap::new(), MANTLE_STORE_DIR, HermeticityMode::Practical)
            .unwrap()
            .build_request;

        assert_eq!(request.additional_files[0].contents, Bytes::from_static(UNKNOWN_PLACEHOLDER.as_bytes()));
    }

    #[test]
    fn foreign_guix_profile_controls_shell_environment_and_working_paths() {
        let profile = ExecutionProfile::foreign_guix();
        let mut drv = test_derivation();
        drv.builder = "/mantle/store/00000000000000000000000000000000-builder".to_string();
        drv.arguments = vec!["--build".to_string()];
        drv.environment.insert("builder".to_string(), drv.builder.clone().into());
        crate::bind_execution_profile(&mut drv, &profile).unwrap();

        let request = super::derivation_to_build_request(
            &drv,
            &BTreeMap::new(),
            "/mantle/store",
            &profile,
            HermeticityMode::Strict,
        )
        .unwrap()
        .build_request;

        assert!(!request.constraints.contains(&BuildConstraints::ProvideBinSh));
        assert!(!request.constraints.contains(&BuildConstraints::ProvideProcMetadata));
        assert!(!request.constraints.contains(&BuildConstraints::ProvideRandomDevices));
        assert!(!request.constraints.contains(&BuildConstraints::NetworkAccess));
        assert_eq!(request.working_dir, PathBuf::from("build"));
        assert_eq!(request.scratch_paths, vec![PathBuf::from("build"), PathBuf::from("mantle/store")]);
        assert!(!request.environment_vars.iter().any(|entry| entry.key == "SHELL"));
        assert!(!request.environment_vars.iter().any(|entry| entry.key == crate::EXECUTION_PROFILE_BINDING_ENV));
    }

    #[test]
    fn foreign_nix_profile_supplies_protocol_environment_for_active_store() {
        const ACTIVE_STORE_DIR: &str = "/mantle/store";
        let profile = ExecutionProfile::foreign_nix();
        let mut drv = test_derivation();
        drv.builder = format!("{ACTIVE_STORE_DIR}/00000000000000000000000000000000-builder");
        drv.arguments = vec!["--build".to_string()];
        drv.environment.insert("builder".to_string(), drv.builder.clone().into());
        crate::bind_execution_profile(&mut drv, &profile).unwrap();

        let request = super::derivation_to_build_request(
            &drv,
            &BTreeMap::new(),
            ACTIVE_STORE_DIR,
            &profile,
            HermeticityMode::Strict,
        )
        .unwrap()
        .build_request;
        let environment: BTreeMap<&str, &[u8]> =
            request.environment_vars.iter().map(|entry| (entry.key.as_str(), entry.value.as_ref())).collect();

        assert_eq!(environment.get("NIX_BUILD_TOP"), Some(&b"/build".as_slice()));
        assert_eq!(environment.get("PWD"), Some(&b"/build".as_slice()));
        assert_eq!(environment.get("NIX_STORE"), Some(&ACTIVE_STORE_DIR.as_bytes()));
        assert_eq!(environment.get("NIX_LOG_FD"), Some(&b"2".as_slice()));
        assert!(request.constraints.contains(&BuildConstraints::ProvideProcMetadata));
        assert!(request.constraints.contains(&BuildConstraints::ProvideRandomDevices));
    }

    #[test]
    fn foreign_nix_protocol_environment_prevents_declared_build_top_escape() {
        const ACTIVE_STORE_DIR: &str = "/mantle/store";
        const ESCAPING_BUILD_TOP: &str = "/escape";
        let profile = ExecutionProfile::foreign_nix();
        let mut drv = test_derivation();
        drv.builder = format!("{ACTIVE_STORE_DIR}/00000000000000000000000000000000-builder");
        drv.arguments = vec!["--build".to_string()];
        drv.environment.insert("builder".to_string(), drv.builder.clone().into());
        drv.environment.insert("NIX_BUILD_TOP".to_string(), ESCAPING_BUILD_TOP.into());
        crate::bind_execution_profile(&mut drv, &profile).unwrap();

        let request = super::derivation_to_build_request(
            &drv,
            &BTreeMap::new(),
            ACTIVE_STORE_DIR,
            &profile,
            HermeticityMode::Strict,
        )
        .unwrap()
        .build_request;
        let build_top = request
            .environment_vars
            .iter()
            .find(|entry| entry.key == "NIX_BUILD_TOP")
            .map(|entry| entry.value.as_ref());

        assert_eq!(build_top, Some(b"/build".as_slice()));
        assert_ne!(build_top, Some(ESCAPING_BUILD_TOP.as_bytes()));
    }

    #[test]
    fn foreign_profile_rejects_undeclared_network_authority() {
        let profile = ExecutionProfile::foreign_nix();
        let mut drv = test_derivation();
        drv.environment
            .insert(crate::ENV_NETWORK_CAPABILITY.to_string(), crate::NETWORK_CAPABILITY_BUILD_TIME.into());
        drv.environment.insert(crate::ENV_NETWORK_POLICY_BASIS.to_string(), "foreign-request".into());
        drv.environment.insert(crate::ENV_NETWORK_AUDIT_CLASS.to_string(), "foreign-network".into());
        crate::bind_execution_profile(&mut drv, &profile).unwrap();

        let error = super::derivation_to_build_request(
            &drv,
            &BTreeMap::new(),
            "/mantle/store",
            &profile,
            HermeticityMode::Strict,
        )
        .unwrap_err();

        assert!(matches!(error, crate::Error::NetworkPolicyDenied { .. }));
    }

    #[test]
    fn build_request_rejects_wrong_foreign_profile() {
        let guix = ExecutionProfile::foreign_guix();
        let nix = ExecutionProfile::foreign_nix();
        let mut drv = test_derivation();
        drv.builder = "/mantle/store/00000000000000000000000000000000-builder".to_string();
        drv.arguments = vec!["--build".to_string()];
        drv.environment.insert("builder".to_string(), drv.builder.clone().into());
        crate::bind_execution_profile(&mut drv, &guix).unwrap();

        let error =
            super::derivation_to_build_request(&drv, &BTreeMap::new(), "/mantle/store", &nix, HermeticityMode::Strict)
                .unwrap_err();

        assert!(error.to_string().contains("binding digest does not match"));
    }

    #[test]
    fn build_request_has_sandbox_env() {
        let drv = test_derivation();
        let req = derivation_to_build_request(&drv, &BTreeMap::new(), "/nix/store", HermeticityMode::Practical)
            .unwrap()
            .build_request;

        let env_map: BTreeMap<&str, &[u8]> =
            req.environment_vars.iter().map(|e| (e.key.as_str(), e.value.as_ref())).collect();

        assert_eq!(*env_map.get("HOME").unwrap(), &b"/homeless-shelter"[..]);
        assert_eq!(*env_map.get("LANG").unwrap(), &b"C"[..]);
        assert_eq!(*env_map.get("SHELL").unwrap(), &b"/bin/sh"[..]);
        assert_eq!(*env_map.get("SOURCE_DATE_EPOCH").unwrap(), &b"1"[..]);
        assert_eq!(*env_map.get("TMPDIR").unwrap(), &b"/build"[..]);
        assert_eq!(*env_map.get("TZ").unwrap(), &b"UTC"[..]);
        assert_eq!(*env_map.get("USER").unwrap(), &b"nixbld"[..]);
        assert_eq!(*env_map.get("NIX_BUILD_CORES").unwrap(), &b"1"[..]);
    }

    #[test]
    fn practical_mode_audits_protected_environment_override() {
        let normalized = normalize_env_map(&[("PATH", "/tmp/bin")], HermeticityMode::Practical).unwrap();
        let path_value = normalized.environment_vars.get("PATH").unwrap();

        assert_eq!(path_value.as_slice(), b"/tmp/bin");
        assert_eq!(normalized.audit_events.len(), 1);
        assert_eq!(normalized.audit_events[0].kind, HermeticityAuditKind::EnvironmentOverride);
        assert!(normalized.audit_events[0].detail.contains("PATH"));
    }

    #[test]
    fn strict_mode_rejects_protected_environment_override() {
        let err = normalize_env_map(&[("HOME", "/tmp/home")], HermeticityMode::Strict).unwrap_err();

        assert!(matches!(err, crate::Error::UnsafeEnvOverride { .. }));
        assert!(err.to_string().contains("HOME"));
    }

    #[test]
    fn practical_mode_allows_nix_build_cores_override_without_audit() {
        let normalized =
            normalize_env_map(&[("NIX_BUILD_CORES", OVERRIDE_CORE_COUNT)], HermeticityMode::Practical).unwrap();
        let nix_build_cores = normalized.environment_vars.get("NIX_BUILD_CORES").unwrap();

        assert_eq!(nix_build_cores.as_slice(), OVERRIDE_CORE_COUNT.as_bytes());
        assert!(normalized.audit_events.is_empty());
    }

    #[test]
    fn practical_mode_allows_source_date_epoch_override_without_audit() {
        let normalized =
            normalize_env_map(&[("SOURCE_DATE_EPOCH", OVERRIDE_SOURCE_DATE_EPOCH)], HermeticityMode::Practical)
                .unwrap();
        let source_date_epoch = normalized.environment_vars.get("SOURCE_DATE_EPOCH").unwrap();

        assert_eq!(source_date_epoch.as_slice(), OVERRIDE_SOURCE_DATE_EPOCH.as_bytes());
        assert!(normalized.audit_events.is_empty());
    }

    #[test]
    fn strict_mode_allows_source_date_epoch_override_without_audit() {
        let normalized =
            normalize_env_map(&[("SOURCE_DATE_EPOCH", OVERRIDE_SOURCE_DATE_EPOCH)], HermeticityMode::Strict).unwrap();
        let source_date_epoch = normalized.environment_vars.get("SOURCE_DATE_EPOCH").unwrap();

        assert_eq!(source_date_epoch.as_slice(), OVERRIDE_SOURCE_DATE_EPOCH.as_bytes());
        assert!(normalized.audit_events.is_empty());
    }

    #[test]
    fn strict_mode_reports_stable_environment_digest() {
        const BLAKE3_HEX_LENGTH_CHARS: usize = 64;

        let first = normalize_env_map(&[("CC", "cc")], HermeticityMode::Strict).unwrap();
        let second = normalize_env_map(&[("CC", "cc")], HermeticityMode::Strict).unwrap();

        assert_eq!(first.report.action_name, "test");
        assert_eq!(first.report.digest_blake3, second.report.digest_blake3);
        assert_eq!(first.report.digest_blake3.as_ref().unwrap().len(), BLAKE3_HEX_LENGTH_CHARS);
        assert_eq!(first.report.variable_count, u32::try_from(first.environment_vars.len()).unwrap());
        assert!(first.report.rejections.is_empty());
        assert!(first.report.search_path.as_ref().is_some_and(|search_path| search_path.entries.is_empty()));
        let determinism = first.report.determinism.as_ref().expect("strict determinism report");
        assert_eq!(determinism.controls.len(), STRICT_DETERMINISM_CONTROL_COUNT);
        assert!(!determinism.strong_claim_blocked);
        assert!(determinism.unsupported_controls.is_empty());
        assert!(determinism.controls.iter().any(|control| control.surface == DETERMINISM_SURFACE_UMASK));
        assert!(determinism.controls.iter().any(|control| control.surface == DETERMINISM_SURFACE_ORDERING));
    }

    #[test]
    fn strict_mode_determinism_policy_changes_with_declared_time_override() {
        let default = normalize_env_map(&[("CC", "cc")], HermeticityMode::Strict).unwrap();
        let overridden =
            normalize_env_map(&[("CC", "cc"), ("SOURCE_DATE_EPOCH", "12345")], HermeticityMode::Strict).unwrap();
        let default_determinism = default.report.determinism.expect("default determinism report");
        let overridden_determinism = overridden.report.determinism.expect("overridden determinism report");

        assert_ne!(default_determinism.policy_digest_blake3, overridden_determinism.policy_digest_blake3);
        assert!(!overridden_determinism.strong_claim_blocked);
        assert!(
            overridden_determinism.controls.iter().any(
                |control| control.surface == DETERMINISM_SURFACE_TIME && control.value == "SOURCE_DATE_EPOCH=12345"
            )
        );
    }

    #[test]
    fn strict_mode_accepts_declared_store_path_and_reports_tool_refs() {
        const BLAKE3_HEX_LENGTH_CHARS: usize = 64;
        const EXPECTED_SEARCH_PATH_ENTRIES: usize = 1;

        let tool_root = "/nix/store/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa-tool";
        let tool_bin = format!("{tool_root}/bin");
        let normalized = normalize_env_map(&[("PATH", tool_bin.as_str())], HermeticityMode::Strict).unwrap();
        let path_value = normalized.environment_vars.get(PATH_VARIABLE).expect("PATH is present");
        let search_path = normalized.report.search_path.as_ref().expect("strict search path report");

        assert_eq!(path_value.as_slice(), tool_bin.as_bytes());
        assert_eq!(search_path.entries.len(), EXPECTED_SEARCH_PATH_ENTRIES);
        assert_eq!(search_path.entries[0].path, tool_bin);
        assert_eq!(search_path.entries[0].real_tool_ref, tool_root);
        assert_eq!(search_path.real_tool_refs, vec![tool_root.to_string()]);
        assert!(search_path.digest_blake3.as_deref().is_some_and(|digest| digest.len() == BLAKE3_HEX_LENGTH_CHARS));
        assert!(normalized.audit_events.is_empty());
    }

    #[test]
    fn strict_mode_rejects_ambient_path_poisoning_with_receipt_bound_report() {
        let err = normalize_env_map(&[("PATH", "/tmp/poison/bin")], HermeticityMode::Strict).unwrap_err();

        let crate::Error::DeniedEnvironmentVariable {
            variable,
            class,
            report,
            ..
        } = err
        else {
            unreachable!("strict ambient PATH must produce denied environment error");
        };
        assert_eq!(variable, PATH_VARIABLE);
        assert_eq!(class, crate::environment_policy::ENV_REJECTION_SEARCH_PATH);
        assert_eq!(report.rejections.len(), 1);
        assert_eq!(report.rejections[0].variable, PATH_VARIABLE);
        assert!(report.rejections[0].diagnostic.contains("receipt-bound-path-ambient-entry"));
        let search_path = report.search_path.expect("blocked path report");
        assert!(search_path.digest_blake3.is_none());
        assert!(search_path.entries.is_empty());
    }

    #[test]
    fn strict_mode_rejects_secret_environment_variable_with_redacted_report() {
        const SECRET_VALUE: &str = "super-secret-token";

        let err = normalize_env_map(&[("CARGO_REGISTRY_TOKEN", SECRET_VALUE)], HermeticityMode::Strict).unwrap_err();

        let crate::Error::DeniedEnvironmentVariable {
            variable,
            class,
            report,
            ..
        } = err
        else {
            unreachable!("strict secret env must produce denied environment error");
        };
        assert_eq!(variable, "CARGO_REGISTRY_TOKEN");
        assert_eq!(class, crate::environment_policy::ENV_REJECTION_SECRET);
        assert_eq!(report.rejections.len(), 1);
        assert_eq!(report.rejections[0].variable, "CARGO_REGISTRY_TOKEN");
        assert!(report.rejections[0].redacted);
        assert!(!report.rejections[0].diagnostic.contains(SECRET_VALUE));
    }

    #[test]
    fn build_request_has_system_constraint() {
        let drv = test_derivation();
        let req = derivation_to_build_request(&drv, &BTreeMap::new(), "/nix/store", HermeticityMode::Practical)
            .unwrap()
            .build_request;

        assert!(req.constraints.contains(&BuildConstraints::System("x86_64-linux".to_string())));
        assert!(req.constraints.contains(&BuildConstraints::ProvideBinSh));
    }

    #[test]
    fn ordinary_build_request_denies_network_by_default() {
        let drv = test_derivation();
        let envelope =
            derivation_to_build_request(&drv, &BTreeMap::new(), "/nix/store", HermeticityMode::Practical).unwrap();

        assert!(!envelope.build_request.constraints.contains(&BuildConstraints::NetworkAccess));
        assert_eq!(envelope.network_policy_report.action_name, "test");
        assert_eq!(envelope.network_policy_report.mode, crate::network_policy::NETWORK_MODE_OFFLINE);
        assert_eq!(envelope.network_policy_report.result, crate::network_policy::NETWORK_RESULT_DENIED);
    }

    #[test]
    fn declared_network_capability_is_rejected_before_build_request() {
        let mut drv = test_derivation();
        drv.environment.insert(
            crate::network_policy::ENV_NETWORK_CAPABILITY.to_string(),
            crate::network_policy::NETWORK_CAPABILITY_BUILD_TIME.into(),
        );
        drv.environment.insert(
            crate::network_policy::ENV_NETWORK_POLICY_BASIS.to_string(),
            "compat-policy:legacy-upstream".into(),
        );
        drv.environment
            .insert(crate::network_policy::ENV_NETWORK_AUDIT_CLASS.to_string(), "legacy-network-build".into());

        let err =
            derivation_to_build_request(&drv, &BTreeMap::new(), "/nix/store", HermeticityMode::Strict).unwrap_err();

        assert!(matches!(err, crate::Error::NetworkPolicyDenied { .. }));
        assert!(err.to_string().contains("network policy denied for test"));
        assert!(err.to_string().contains("ordinary derivation network access is denied by default"));
        let crate::Error::NetworkPolicyDenied { report, .. } = err else {
            unreachable!("matches! above proved the error variant");
        };
        assert_eq!(report.result, crate::network_policy::NETWORK_RESULT_BLOCKED);
        assert_eq!(report.capability.as_deref(), Some(crate::network_policy::NETWORK_CAPABILITY_BUILD_TIME));
        assert_eq!(report.policy_basis.as_deref(), Some("compat-policy:legacy-upstream"));
        assert_eq!(report.audit_class.as_deref(), Some("legacy-network-build"));
    }

    #[test]
    fn build_request_outputs_are_relative() {
        let drv = test_derivation();
        let req = derivation_to_build_request(&drv, &BTreeMap::new(), "/nix/store", HermeticityMode::Practical)
            .unwrap()
            .build_request;

        for output in &req.outputs {
            assert!(!output.starts_with("/"), "output path must be relative: {output:?}");
            assert!(output.starts_with("nix/store"), "output must be under nix/store: {output:?}");
        }
    }

    #[test]
    fn build_request_has_refscan_needles() {
        let drv = test_derivation();
        let req = derivation_to_build_request(&drv, &BTreeMap::new(), "/nix/store", HermeticityMode::Practical)
            .unwrap()
            .build_request;

        // At least one needle for the output
        assert!(!req.refscan_needles.is_empty());
        // Needles are nixbase32 encoded (32 chars)
        for needle in &req.refscan_needles {
            assert_eq!(needle.len(), 32, "needle should be 32 chars: {needle}");
        }
    }

    // ── Phase 1: replace_placeholders tests ────────────────────

    #[test]
    fn replace_placeholders_substitutes_output_path() {
        let drv = test_derivation();
        let out_path = drv.outputs["out"].path.as_ref().unwrap();
        let placeholder = hash_placeholder("out");
        let input = format!("echo hello > {placeholder}");

        let result = replace_placeholders(&input, &drv.outputs, NIX_STORE_DIR);
        assert!(!result.contains(&placeholder), "placeholder should be gone");
        assert!(result.contains(&out_path.to_absolute_path()), "should contain output path: {result}");
    }

    #[test]
    fn replace_placeholders_substitutes_nix_compatible_output_at_selected_store_prefix() {
        let drv = test_derivation();
        let out_path = drv.outputs["out"].path.as_ref().unwrap();
        let placeholder = nix_compatible_hash_placeholder("out");
        let input = format!("hex0 source {placeholder}");

        assert_eq!(placeholder, NIX_OUT_PLACEHOLDER);
        let result = replace_placeholders(&input, &drv.outputs, MANTLE_STORE_DIR);
        assert!(!result.contains(&placeholder), "Nix-compatible placeholder should be gone");
        assert!(
            result.contains(&out_path.to_absolute_path_with_prefix(MANTLE_STORE_DIR)),
            "should contain selected output path: {result}"
        );
        assert!(!result.contains(NIX_STORE_DIR), "must not fall back to the Nix store: {result}");
    }

    #[test]
    fn replace_placeholders_noop_without_known_placeholder() {
        let drv = test_derivation();
        let input = format!("echo hello world {UNKNOWN_PLACEHOLDER}");
        let result = replace_placeholders(&input, &drv.outputs, MANTLE_STORE_DIR);
        assert_eq!(result, input);
    }

    #[test]
    fn replace_placeholders_multi_output() {
        // Build a fresh derivation with both outputs before computing paths
        let mut outputs = BTreeMap::new();
        outputs.insert("out".to_string(), Output {
            path: None,
            ca_hash: None,
        });
        outputs.insert("dev".to_string(), Output {
            path: None,
            ca_hash: None,
        });
        let mut environment = BTreeMap::new();
        environment.insert("name".to_string(), "test".into());
        environment.insert("system".to_string(), "x86_64-linux".into());
        environment.insert("builder".to_string(), "/bin/sh".into());
        environment.insert("out".to_string(), "".into());
        environment.insert("dev".to_string(), "".into());
        let mut drv = Derivation {
            arguments: vec!["-c".into(), "echo > $out".into()],
            builder: "/bin/sh".to_string(),
            environment,
            input_derivations: BTreeMap::new(),
            input_sources: BTreeSet::new(),
            outputs,
            system: "x86_64-linux".to_string(),
        };
        let hdm = drv.hash_derivation_modulo(|_| panic!("no parent"));
        drv.calculate_output_paths("test", &hdm).unwrap();

        let ph_out = hash_placeholder("out");
        let ph_dev = hash_placeholder("dev");
        let input = format!("install -D {ph_out}/bin/x {ph_dev}/include/x.h");

        let result = replace_placeholders(&input, &drv.outputs, NIX_STORE_DIR);
        assert!(!result.contains(&ph_out));
        assert!(!result.contains(&ph_dev));
        let out_path = drv.outputs["out"].path.as_ref().unwrap().to_absolute_path();
        let dev_path = drv.outputs["dev"].path.as_ref().unwrap().to_absolute_path();
        assert!(result.contains(&out_path));
        assert!(result.contains(&dev_path));
    }

    #[test]
    fn replace_placeholders_bstr_matches_string_variant() {
        let drv = test_derivation();
        let placeholder = hash_placeholder("out");
        let input = format!("echo > {placeholder}");

        let str_result = replace_placeholders(&input, &drv.outputs, MANTLE_STORE_DIR);
        let bstr_result = replace_placeholders_bstr(&BString::from(input.as_bytes()), &drv.outputs, MANTLE_STORE_DIR);
        assert_eq!(str_result.as_bytes(), bstr_result.as_ref() as &[u8]);
        assert!(str_result.contains(MANTLE_STORE_DIR));
        assert!(!str_result.contains(NIX_STORE_DIR));
    }

    // ── Phase 1: collect_input_paths tests ────────────────────

    #[test]
    fn collect_inputs_source_only() {
        let mut drv = test_derivation();
        let source: StorePath<String> =
            StorePath::from_absolute_path("/nix/store/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa-bash-5.2".as_bytes()).unwrap();
        drv.input_sources.insert(source.clone());

        let kp = DerivationRegistry::default();
        let paths = collect_input_paths(&drv, &kp).unwrap();
        assert!(paths.contains(&source));
        assert_eq!(paths.len(), 1);
    }

    #[test]
    fn collect_inputs_derivation_only() {
        let mut kp = DerivationRegistry::default();
        let (dep_drv_path, dep_drv) = register_drv("dep", &mut kp);

        let mut parent = test_derivation();
        let mut dep_outputs = BTreeSet::new();
        dep_outputs.insert("out".to_string());
        parent.input_derivations.insert(dep_drv_path.clone(), dep_outputs);

        let paths = collect_input_paths(&parent, &kp).unwrap();
        let dep_out = dep_drv.outputs["out"].path.as_ref().unwrap();
        assert!(paths.contains(dep_out), "should contain dep's output path");
    }

    #[test]
    fn collect_inputs_mixed() {
        let mut kp = DerivationRegistry::default();
        let (dep_drv_path, dep_drv) = register_drv("mixdep", &mut kp);

        let source: StorePath<String> =
            StorePath::from_absolute_path("/nix/store/bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb-src".as_bytes()).unwrap();

        let mut parent = test_derivation();
        parent.input_sources.insert(source.clone());
        let mut dep_outputs = BTreeSet::new();
        dep_outputs.insert("out".to_string());
        parent.input_derivations.insert(dep_drv_path, dep_outputs);

        let paths = collect_input_paths(&parent, &kp).unwrap();
        assert!(paths.contains(&source));
        let dep_out = dep_drv.outputs["out"].path.as_ref().unwrap();
        assert!(paths.contains(dep_out));
        assert_eq!(paths.len(), 2);
    }

    #[test]
    fn collect_inputs_missing_drv_returns_error() {
        let kp = DerivationRegistry::default();
        let mut parent = test_derivation();
        let fake_drv: StorePath<String> =
            StorePath::from_absolute_path("/nix/store/cccccccccccccccccccccccccccccccc-missing.drv".as_bytes())
                .unwrap();
        let mut outputs = BTreeSet::new();
        outputs.insert("out".to_string());
        parent.input_derivations.insert(fake_drv, outputs);

        let err = collect_input_paths(&parent, &kp).unwrap_err();
        assert!(matches!(err, crate::Error::DerivationNotFound { .. }));
    }

    // ── output selection integration tests ────────────────────────
    //
    // End-to-end: CrunchDerivation with OutputSelection → convert →
    // verify collect_input_paths resolves only the selected outputs.

    fn ia_drv(name: &str, outputs: &[&str]) -> crunch_glue::CrunchDerivation {
        crunch_glue::CrunchDerivation {
            name: name.to_string(),
            builder: "/bin/sh".to_string(),
            system: "x86_64-linux".to_string(),
            args: vec![],
            outputs: outputs.iter().map(|s| s.to_string()).collect(),
            dynamic_plan_outputs: vec![],
            env: Default::default(),
            inputs: vec![],
            fixed_output: None,
            addressing_mode: "input-addressed".to_string(),
            provenance: None,
        }
    }

    /// Convert via ConversionCache, then bridge to DerivationRegistry.
    fn convert_and_bridge(drv: &crunch_glue::CrunchDerivation) -> (DerivationRegistry, StorePath<String>, Derivation) {
        let mut cc = crunch_glue::ConversionCache::default();
        let (drv_path, nix_drv) = crunch_glue::convert(drv, &mut cc).unwrap();
        let mut reg = DerivationRegistry::default();
        crate::registry::populate_registry(&mut reg, cc.iter_entries());
        (reg, drv_path, nix_drv)
    }

    #[test]
    fn output_selection_only_selected_output_in_sandbox_inputs() {
        use crunch_glue::Input;
        use crunch_glue::OutputRef;

        let dep = ia_drv("libfoo", &["out", "dev", "lib"]);
        let consumer = crunch_glue::CrunchDerivation {
            inputs: vec![Input::OutputSelection(Box::new(OutputRef {
                drv: dep,
                output: "dev".to_string(),
            }))],
            ..ia_drv("myapp", &["out"])
        };

        let (reg, _, nix_drv) = convert_and_bridge(&consumer);

        // input_derivations has only "dev"
        let (dep_path, dep_outputs) = nix_drv.input_derivations.iter().next().unwrap();
        assert_eq!(dep_outputs.len(), 1);
        assert!(dep_outputs.contains("dev"));

        // collect_input_paths resolves to exactly one path: libfoo's dev output
        let paths = collect_input_paths(&nix_drv, &reg).unwrap();
        let dep_abs = dep_path.to_absolute_path();
        let dep_entry = reg.get_by_drv_path(&dep_abs).unwrap();
        let dev_path = dep_entry.derivation.outputs["dev"].path.as_ref().unwrap();

        assert!(paths.contains(dev_path), "sandbox should include dev output");
        let out_path = dep_entry.derivation.outputs["out"].path.as_ref().unwrap();
        let lib_path = dep_entry.derivation.outputs["lib"].path.as_ref().unwrap();
        assert!(!paths.contains(out_path), "sandbox should NOT include out output");
        assert!(!paths.contains(lib_path), "sandbox should NOT include lib output");
    }

    #[test]
    fn output_selection_coalescing_both_in_sandbox() {
        use crunch_glue::Input;
        use crunch_glue::OutputRef;

        let dep = ia_drv("libfoo", &["out", "dev", "lib"]);
        let consumer = crunch_glue::CrunchDerivation {
            inputs: vec![
                Input::OutputSelection(Box::new(OutputRef {
                    drv: dep.clone(),
                    output: "dev".to_string(),
                })),
                Input::OutputSelection(Box::new(OutputRef {
                    drv: dep,
                    output: "lib".to_string(),
                })),
            ],
            ..ia_drv("myapp", &["out"])
        };

        let (reg, _, nix_drv) = convert_and_bridge(&consumer);

        // Coalesced: both dev and lib
        let (dep_path, dep_outputs) = nix_drv.input_derivations.iter().next().unwrap();
        assert_eq!(dep_outputs.len(), 2);
        assert!(dep_outputs.contains("dev"));
        assert!(dep_outputs.contains("lib"));

        let paths = collect_input_paths(&nix_drv, &reg).unwrap();
        let dep_abs = dep_path.to_absolute_path();
        let dep_entry = reg.get_by_drv_path(&dep_abs).unwrap();
        let dev_path = dep_entry.derivation.outputs["dev"].path.as_ref().unwrap();
        let lib_path = dep_entry.derivation.outputs["lib"].path.as_ref().unwrap();

        assert!(paths.contains(dev_path), "sandbox should include dev");
        assert!(paths.contains(lib_path), "sandbox should include lib");
        let out_path = dep_entry.derivation.outputs["out"].path.as_ref().unwrap();
        assert!(!paths.contains(out_path), "sandbox should NOT include out");
    }

    // ── Fetcher BuildRequest encoding tests ────────────────────
    //
    // Verify that derivation_to_build_request preserves the builder
    // selector and fetch environment variables for builtin:fetchurl.

    /// Build a fetchurl derivation with the given env, similar to what
    /// crunch.fetchurl produces after convert().
    fn make_fetcher_drv(url: &str, hash: Option<nix_compat::nixhash::CAHash>) -> Derivation {
        let mut outputs = BTreeMap::new();
        outputs.insert("out".to_string(), Output {
            path: None,
            ca_hash: hash,
        });
        let mut environment = BTreeMap::new();
        environment.insert("name".to_string(), "test-fetch".into());
        environment.insert("system".to_string(), "builtin".into());
        environment.insert("builder".to_string(), "builtin:fetchurl".into());
        environment.insert("url".to_string(), url.into());
        environment.insert("out".to_string(), "".into());
        environment.insert("preferLocalBuild".to_string(), "1".into());

        let mut drv = Derivation {
            arguments: vec![],
            builder: "builtin:fetchurl".to_string(),
            environment,
            input_derivations: BTreeMap::new(),
            input_sources: BTreeSet::new(),
            outputs,
            system: "builtin".to_string(),
        };
        let hdm = drv.hash_derivation_modulo(|_| panic!("no parent"));
        drv.calculate_output_paths("test-fetch", &hdm).unwrap();
        drv
    }

    #[test]
    fn fetcher_build_request_has_builtin_builder() {
        use nix_compat::nixhash::CAHash;
        use nix_compat::nixhash::NixHash;

        const FETCH_HASH_BYTE: u8 = 0xBB;

        let drv = make_fetcher_drv(
            "https://example.com/foo.tar.gz",
            Some(CAHash::Flat(NixHash::Sha256([FETCH_HASH_BYTE; 32]))),
        );
        let envelope =
            derivation_to_build_request(&drv, &BTreeMap::new(), "/nix/store", HermeticityMode::Practical).unwrap();
        let req = envelope.build_request;

        assert_eq!(req.command_args[0], "builtin:fetchurl");
        // Fetcher derivations have no arguments.
        assert_eq!(req.command_args.len(), 1);
        assert!(req.constraints.contains(&BuildConstraints::NetworkAccess));
        assert_eq!(envelope.network_policy_report.mode, crate::network_policy::NETWORK_MODE_FIXED_OUTPUT_FETCHER);
    }

    #[test]
    fn fetcher_build_request_preserves_url_env_var() {
        let drv = make_fetcher_drv("https://example.com/foo.tar.gz", None);
        let req = derivation_to_build_request(&drv, &BTreeMap::new(), "/nix/store", HermeticityMode::Practical)
            .unwrap()
            .build_request;

        let env_map: BTreeMap<&str, &[u8]> =
            req.environment_vars.iter().map(|e| (e.key.as_str(), e.value.as_ref())).collect();

        assert_eq!(
            *env_map.get("url").unwrap(),
            b"https://example.com/foo.tar.gz",
            "url env var must be preserved in BuildRequest"
        );
        assert_eq!(*env_map.get("builder").unwrap(), b"builtin:fetchurl", "builder env var must be preserved");
    }

    #[test]
    fn fetcher_build_request_preserves_all_fetch_env_vars() {
        // All five fetch-specific env vars must survive conversion.
        let mut drv = make_fetcher_drv("https://example.com/src.tar.gz", None);
        drv.environment.insert("unpack".to_string(), "1".into());
        drv.environment.insert("type".to_string(), "git".into());
        drv.environment.insert("rev".to_string(), "abc123".into());
        drv.environment.insert("executable".to_string(), "1".into());

        let req = derivation_to_build_request(&drv, &BTreeMap::new(), "/nix/store", HermeticityMode::Practical)
            .unwrap()
            .build_request;

        let env_map: BTreeMap<&str, &[u8]> =
            req.environment_vars.iter().map(|e| (e.key.as_str(), e.value.as_ref())).collect();

        // url (set by make_fetcher_drv)
        assert_eq!(*env_map.get("url").unwrap(), b"https://example.com/src.tar.gz", "url must be preserved");
        // unpack
        assert_eq!(*env_map.get("unpack").unwrap(), b"1", "unpack must be preserved");
        // type
        assert_eq!(*env_map.get("type").unwrap(), b"git", "type must be preserved");
        // rev
        assert_eq!(*env_map.get("rev").unwrap(), b"abc123", "rev must be preserved");
        // executable
        assert_eq!(*env_map.get("executable").unwrap(), b"1", "executable must be preserved");
    }

    #[test]
    fn fetcher_build_request_is_recognized_by_is_fetch_request() {
        let drv = make_fetcher_drv("https://example.com/file.txt", None);
        let req = derivation_to_build_request(&drv, &BTreeMap::new(), "/nix/store", HermeticityMode::Practical)
            .unwrap()
            .build_request;

        assert!(
            crate::fetch_build_service::is_fetch_request(&req),
            "BuildRequest from a fetchurl derivation must be recognized as a fetch request"
        );
    }

    #[test]
    fn sandbox_build_request_is_not_fetch_request() {
        let drv = test_derivation();
        let req = derivation_to_build_request(&drv, &BTreeMap::new(), "/nix/store", HermeticityMode::Practical)
            .unwrap()
            .build_request;

        assert!(
            !crate::fetch_build_service::is_fetch_request(&req),
            "BuildRequest from a normal derivation must NOT be recognized as a fetch request"
        );
    }

    #[test]
    fn custom_fixed_output_builder_does_not_gain_network_access() {
        use nix_compat::nixhash::CAHash;
        use nix_compat::nixhash::NixHash;

        let mut drv = test_derivation();
        drv.outputs.get_mut("out").unwrap().ca_hash = Some(CAHash::Flat(NixHash::Sha256([0xBB; 32])));
        let envelope =
            derivation_to_build_request(&drv, &BTreeMap::new(), "/nix/store", HermeticityMode::Practical).unwrap();

        assert!(!envelope.build_request.constraints.contains(&BuildConstraints::NetworkAccess));
        assert_eq!(envelope.network_policy_report.mode, crate::network_policy::NETWORK_MODE_OFFLINE);
        assert!(envelope.network_policy_report.fixed_output.is_none());
    }

    #[test]
    fn fetcher_build_request_outputs_contain_fetch_output_path() {
        use nix_compat::nixhash::CAHash;
        use nix_compat::nixhash::NixHash;

        // Create a fetcher drv with a known flat hash so the output path
        // is deterministic (FOD path computation).
        let drv = make_fetcher_drv("https://example.com/foo.txt", Some(CAHash::Flat(NixHash::Sha256([0xBB; 32]))));
        let req = derivation_to_build_request(&drv, &BTreeMap::new(), "/nix/store", HermeticityMode::Practical)
            .unwrap()
            .build_request;

        // Must have exactly one output (fetchers produce "out" only).
        assert_eq!(req.outputs.len(), 1, "fetcher must have exactly one output");

        // The output path must be relative (no leading /) and under
        // the store dir, matching the derivation's computed output path.
        let out_path = &req.outputs[0];
        assert!(!out_path.starts_with("/"), "output path must be relative: {out_path:?}");
        assert!(out_path.starts_with("nix/store"), "output must be under nix/store: {out_path:?}");

        // Verify it matches the derivation's own output path (stripped of /).
        let drv_out = drv.outputs["out"].path.as_ref().unwrap();
        let expected_relative = &drv_out.to_absolute_path()[1..]; // strip leading /
        assert_eq!(
            out_path.to_str().unwrap(),
            expected_relative,
            "BuildRequest.outputs[0] must match the derivation's computed output path"
        );
    }

    #[test]
    fn declared_mutable_workspace_becomes_runtime_request_without_environment_leak() {
        let mut drv = test_derivation();
        let policy = crate::WorkspacePolicy {
            mode: crate::WorkspaceMode::MutableSession,
            workspace_id: Some("cargo-cache".to_string()),
            compatibility: crate::WorkspaceCompatibilityFacts {
                authority_class: "tenant-a".to_string(),
                action_class: "cargo-build".to_string(),
                toolchain_refs: vec!["mantle-object://blake3/rust".to_string()],
            },
            snapshot: crate::WorkspaceSnapshotPolicy {
                enabled: true,
                require_clean_scrub: true,
            },
            ..crate::WorkspacePolicy::default()
        };
        drv.environment
            .insert(WORKSPACE_POLICY_ENV.to_string(), serde_json::to_vec(&policy).unwrap().into());
        let request = derivation_to_build_request(&drv, &BTreeMap::new(), "/nix/store", HermeticityMode::Practical)
            .unwrap()
            .build_request;
        let workspace = request.workspace.unwrap();
        let lease = workspace.lease.as_ref().unwrap();
        assert_eq!(workspace.mode, StatefulWorkspaceMode::MutableSession);
        assert_eq!(workspace.workspace_id.as_deref(), Some("cargo-cache"));
        assert_eq!(lease.worker_id, LOCAL_WORKSPACE_WORKER_ID);
        assert_eq!(lease.attempt_id, LOCAL_WORKSPACE_ATTEMPT_ID);
        assert_eq!(workspace.generation, INITIAL_WORKSPACE_FENCE_GENERATION);
        assert!(request.environment_vars.iter().all(|item| item.key != WORKSPACE_POLICY_ENV));
    }

    #[test]
    fn non_mutable_workspace_rejects_declared_lease() {
        const TEST_FENCE_GENERATION: u64 = 7;

        let mut drv = test_derivation();
        let policy = crate::WorkspacePolicy {
            mode: crate::WorkspaceMode::None,
            ..crate::WorkspacePolicy::default()
        };
        let lease = StatefulWorkspaceLeaseBinding {
            worker_id: "worker-a".to_string(),
            authority_class: "tenant-a".to_string(),
            job_id: "job-a".to_string(),
            attempt_id: "attempt-a".to_string(),
            fence_generation: TEST_FENCE_GENERATION,
        };
        drv.environment
            .insert(WORKSPACE_POLICY_ENV.to_string(), serde_json::to_vec(&policy).unwrap().into());
        drv.environment.insert(WORKSPACE_LEASE_ENV.to_string(), serde_json::to_vec(&lease).unwrap().into());

        let error =
            derivation_to_build_request(&drv, &BTreeMap::new(), "/nix/store", HermeticityMode::Practical).unwrap_err();

        assert_eq!(error.to_string(), "store error: stateful workspace lease is only valid for mutable-session mode");
        assert!(!error.to_string().contains("runtime_host_path"));
    }

    #[test]
    fn invalid_workspace_policy_fails_closed() {
        let mut drv = test_derivation();
        drv.environment
            .insert(WORKSPACE_POLICY_ENV.to_string(), br#"{"schema":"unknown","mode":"ambient"}"#.to_vec().into());
        let error =
            derivation_to_build_request(&drv, &BTreeMap::new(), "/nix/store", HermeticityMode::Practical).unwrap_err();
        assert!(error.to_string().contains("invalid stateful workspace policy"));
        assert!(!error.to_string().contains("runtime_host_path"));
    }
}
