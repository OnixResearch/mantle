//! Translate `nix_compat::Derivation` → `snix_build::BuildRequest`.
//!
//! Adapted from snix-glue's `derivation_into_build_request`, simplified:
//! no structured_attrs, no passAsFile (those are Nix-isms that crunch
//! doesn't need in v0).

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
    let environment_vars =
        declared_profile_environment(derivation, execution_profile).map_err(execution_profile_error)?;
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

    let command_args = build_command_args(derivation);
    let constraints = build_constraints(derivation, execution_profile, allow_network);
    let refscan_needles = build_refscan_needles(derivation, inputs);

    // Tiger Style: assert command_args has at least the builder.
    debug_assert!(!command_args.is_empty());
    debug_assert!(!environment_vars.is_empty(), "environment must include sandbox vars");

    let sandbox_outputs = map_outputs_to_sandbox_paths(derivation, store_dir);
    let input_map = map_inputs_to_components(inputs)?;

    Ok(BuildRequest {
        command_args,
        outputs: sandbox_outputs,
        environment_vars: environment_vars
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
        additional_files: vec![],
        refscan_needles,
        workspace: None,
    })
}

/// Build command args with placeholders expanded.
fn build_command_args(derivation: &Derivation) -> Vec<String> {
    let mut command_args: Vec<String> = Vec::with_capacity(derivation.arguments.len().saturating_add(1));
    command_args.push(derivation.builder.clone());
    for arg in &derivation.arguments {
        command_args.push(replace_placeholders(arg, &derivation.outputs));
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
        let replaced = replace_placeholders_bstr(value, &derivation.outputs);
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

/// Replace `hash_placeholder(outputName)` strings with actual output paths.
fn replace_placeholders(s: &str, outputs: &BTreeMap<String, Output>) -> String {
    let mut result = s.to_owned();
    for (name, output) in outputs {
        if let Some(path) = output.path.as_ref() {
            let placeholder = hash_placeholder(name.as_str());
            result = result.replace(&placeholder, &path.to_absolute_path());
        }
    }
    result
}

/// Replace placeholders in a BString.
fn replace_placeholders_bstr(s: &BString, outputs: &BTreeMap<String, Output>) -> BString {
    use bstr::ByteSlice;
    let mut result = s.clone();
    for (name, output) in outputs {
        if let Some(path) = output.path.as_ref() {
            let placeholder = hash_placeholder(name.as_str());
            result = result.replace(placeholder.as_bytes(), path.to_absolute_path().as_bytes()).into();
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use nix_compat::derivation::Derivation;

    use super::*;

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
        assert!(!request.constraints.contains(&BuildConstraints::NetworkAccess));
        assert_eq!(request.working_dir, PathBuf::from("build"));
        assert_eq!(request.scratch_paths, vec![PathBuf::from("build"), PathBuf::from("mantle/store")]);
        assert!(!request.environment_vars.iter().any(|entry| entry.key == "SHELL"));
        assert!(!request.environment_vars.iter().any(|entry| entry.key == crate::EXECUTION_PROFILE_BINDING_ENV));
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

        let result = replace_placeholders(&input, &drv.outputs);
        assert!(!result.contains(&placeholder), "placeholder should be gone");
        assert!(result.contains(&out_path.to_absolute_path()), "should contain output path: {result}");
    }

    #[test]
    fn replace_placeholders_noop_without_placeholder() {
        let drv = test_derivation();
        let input = "echo hello world";
        let result = replace_placeholders(input, &drv.outputs);
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

        let result = replace_placeholders(&input, &drv.outputs);
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

        let str_result = replace_placeholders(&input, &drv.outputs);
        let bstr_result = replace_placeholders_bstr(&BString::from(input.as_bytes()), &drv.outputs);
        assert_eq!(str_result.as_bytes(), bstr_result.as_ref() as &[u8]);
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
