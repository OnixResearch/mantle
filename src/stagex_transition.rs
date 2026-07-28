use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::fs;
use std::fs::File;
use std::io;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::process::ExitStatus;
use std::process::Stdio;
use std::thread;
use std::time::Duration;
use std::time::Instant;

use crunch_bootstrap_core::LineageManifest;
use crunch_bootstrap_core::STAGEX_PLAN_SCHEMA_V1;
use crunch_bootstrap_core::STAGEX_SEED_SOURCE_STAGE_ID;
use crunch_bootstrap_core::StagexExecutableAuthorization;
use crunch_bootstrap_core::StagexMaterializationPlan;
use crunch_bootstrap_core::StagexStagePlan;
use crunch_bootstrap_core::stagex_plan_digest_blake3;
use crunch_bootstrap_core::validate_stagex_plan;
use serde::Serialize;

use crate::protected_exec::OutputPromotionRecord;
use crate::protected_exec::PlannedExecutable;
use crate::protected_exec::PromotedExecutable;
use crate::protected_exec::ProtectedExecPolicy;
use crate::protected_exec::ProtectedSeccompAuditEvent;
use crate::protected_exec::blake3_file_hex;
use crate::protected_exec_seccomp::ProtectedSeccompSupervisor;
use crate::protected_exec_seccomp::install_current_thread_exec_supervisor;

pub(crate) const STAGEX_TRANSITION_REPORT_SCHEMA_V1: &str = "mantle-stagex-protected-transition-report-v1";
pub(crate) const HEX0_REPRODUCTION_STAGE_ID: &str = "hex0-reproduction";
pub(crate) const KAEM_MATERIALIZATION_STAGE_ID: &str = "kaem-materialization";
pub(crate) const KAEM_SMOKE_STAGE_ID: &str = "kaem-smoke";
pub(crate) const HEX0_SEED_BLAKE3: &str = "cf21608d883b8bdcc1fa6438703630f2fa496cf74d483ce351f876c0656ecf80";
pub(crate) const HEX0_SOURCE_BLAKE3: &str = "0fb23576a10b41df29c165e39514f18c411da96ae71873a6a2e2f0b1d94de614";
pub(crate) const KAEM_SOURCE_BLAKE3: &str = "5a56b4164dca4d1e03ba35bc8ce4b418a0de2baf1cb9e6a0912607e26532d30d";
pub(crate) const EMPTY_KAEM_SMOKE_SOURCE_BLAKE3: &str =
    "af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262";
pub(crate) const KAEM_OUTPUT_BLAKE3: &str = "edc664d028b349824cc8c66030a8883b81bdc03d07bc871e44e60ecb4311ab2a";

const HEX0_SEED_BYTES: u32 = 229;
const KIB_BYTES: u64 = 1_024;
const MIB_BYTES: u64 = KIB_BYTES * KIB_BYTES;
const AUDIT_SEED_MAX_KIB: u64 = 4;
const AUDIT_SEED_MAX_BYTES: u64 = AUDIT_SEED_MAX_KIB * KIB_BYTES;
const SOURCE_FILE_MAX_KIB: u64 = 256;
const SOURCE_FILE_MAX_BYTES: u64 = SOURCE_FILE_MAX_KIB * KIB_BYTES;
const GENERATED_EXECUTABLE_MAX_MIB: u64 = 4;
const GENERATED_EXECUTABLE_MAX_BYTES: u64 = GENERATED_EXECUTABLE_MAX_MIB * MIB_BYTES;
const LINEAGE_MANIFEST_MAX_MIB: u64 = 1;
const LINEAGE_MANIFEST_MAX_BYTES: u64 = LINEAGE_MANIFEST_MAX_MIB * MIB_BYTES;
const STAGE_TIMEOUT_MS: u64 = 30_000;
const STAGE0_STAGE_TIMEOUT_MS: u64 = 120_000;
const STAGE_JOB_COUNT: u32 = 1;
const TRANSITION_STAGE_COUNT_MAX: u32 = 3;
const TRANSITION_WITH_STAGE0_STAGE_COUNT_MAX: u32 = 22;
const PROCESS_POLL_MS: u64 = 10;
const AUDIT_FLUSH_WAIT_MS: u64 = 50;
const GENERATED_LAUNCH_MAX_ATTEMPTS: u32 = 16;
const GENERATED_LAUNCH_RETRY_DELAY_MS: u64 = 20;
const PROCESS_STDERR_MAX_KIB: u64 = 4;
const PROCESS_STDERR_MAX_BYTES: u64 = PROCESS_STDERR_MAX_KIB * KIB_BYTES;
const EXECUTABLE_MODE_OWNER_ONLY: u32 = 0o700;
const ETXTBSY_EXIT_CODE: i32 = 126;
const ETXTBSY_STDERR_MARKER: &str = "Text file busy";
const KAEM_SMOKE_MARKER: &[u8] = b"kaem-smoke-success\n";
const EXPECTED_AUDIT_EVENT_COUNT: usize = 3;
const EXPECTED_PROMOTION_COUNT: usize = 2;
const SOURCE_STATE_DOMAIN: &[u8] = b"mantle-stagex-transition-source-state-v1\0";
const REPORT_FILE_NAME: &str = "transition-report.json";
const PLAN_FILE_NAME: &str = "transition-plan.json";
const AUDIT_FILE_NAME: &str = "protected-exec-audit.json";

#[derive(Debug, Clone)]
pub(crate) struct StagexTransitionRequest<'a> {
    pub seed_path: &'a Path,
    pub hex0_source_path: &'a Path,
    pub kaem_source_path: &'a Path,
    pub lineage_manifest_path: &'a Path,
    pub source_bundle_path: Option<&'a Path>,
    pub stage0_answers_path: Option<&'a Path>,
    pub scratch_dir: &'a Path,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct StagexTransitionReport {
    pub schema_version: String,
    pub status: String,
    pub plan_digest_blake3: String,
    pub source_state_digest_blake3: String,
    pub seed_digest_blake3: String,
    pub reproduced_hex0_digest_blake3: String,
    pub kaem_digest_blake3: String,
    pub kaem_smoke_digest_blake3: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stage0_full: Option<crate::stagex_stage0_full::Stage0FullInventoryReport>,
    pub fallback_events: Vec<String>,
    pub promotions: Vec<OutputPromotionRecord>,
    pub protected_exec_events: Vec<ProtectedSeccompAuditEvent>,
}

#[derive(Debug)]
pub(crate) enum StagexTransitionError {
    Io {
        action: String,
        source: io::Error,
    },
    InvalidInput(String),
    DigestMismatch {
        role: String,
        expected: String,
        actual: String,
    },
    SizeLimit {
        role: String,
        actual_bytes: u64,
        limit_bytes: u64,
    },
    Plan(Vec<String>),
    ProtectedExec(String),
    ProcessSpawn {
        executable: PathBuf,
        source: io::Error,
    },
    ProcessFailure {
        executable: PathBuf,
        exit_code: Option<i32>,
        stderr: String,
    },
    ProcessTimeout {
        executable: PathBuf,
        timeout_ms: u64,
    },
    Audit(String),
}

impl std::fmt::Display for StagexTransitionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io { action, source } => write!(f, "{action}: {source}"),
            Self::InvalidInput(message) => write!(f, "invalid StageX transition input: {message}"),
            Self::DigestMismatch { role, expected, actual } => {
                write!(f, "StageX {role} BLAKE3 mismatch: expected {expected}, got {actual}")
            }
            Self::SizeLimit {
                role,
                actual_bytes,
                limit_bytes,
            } => write!(f, "StageX {role} has {actual_bytes} bytes; limit is {limit_bytes}"),
            Self::Plan(errors) => write!(f, "invalid StageX transition plan: {}", errors.join("; ")),
            Self::ProtectedExec(message) => write!(f, "StageX protected execution failed: {message}"),
            Self::ProcessSpawn { executable, source } => {
                write!(f, "spawning StageX executable {}: {source}", executable.display())
            }
            Self::ProcessFailure {
                executable,
                exit_code,
                stderr,
            } => write!(f, "StageX executable {} failed with status {:?}: {}", executable.display(), exit_code, stderr),
            Self::ProcessTimeout { executable, timeout_ms } => {
                write!(f, "StageX executable {} exceeded {timeout_ms} ms", executable.display())
            }
            Self::Audit(message) => write!(f, "invalid StageX protected-exec audit: {message}"),
        }
    }
}

impl std::error::Error for StagexTransitionError {}

#[derive(Debug)]
struct TransitionManifestAuthority {
    digest_blake3: String,
    source_bundle_manifest_blake3: String,
}

struct StagedTransitionPaths {
    seed: PathBuf,
    hex0_source: PathBuf,
    kaem_source: PathBuf,
    reproduced_hex0: PathBuf,
    kaem: PathBuf,
    kaem_smoke_script: PathBuf,
    kaem_smoke: PathBuf,
}

struct BoundedCommandRequest<'a> {
    executable: &'a Path,
    args: &'a [PathBuf],
    current_dir: &'a Path,
    stderr_path: &'a Path,
    retry_generated_etxtbsy: bool,
}

struct ExpectedAuditEvent<'a> {
    executable_path: &'a Path,
    digest_blake3: &'a str,
    inventory_entry_id: String,
}

pub(crate) fn materialize_protected_transition(
    request: StagexTransitionRequest<'_>,
) -> Result<StagexTransitionReport, StagexTransitionError> {
    let manifest_authority = load_transition_manifest_authority(request.lineage_manifest_path)?;
    let staged = stage_transition_inputs(&request)?;
    validate_stage0_optional_inputs(request.source_bundle_path, request.stage0_answers_path)?;
    validate_optional_source_bundle(request.source_bundle_path, &manifest_authority)?;
    let source_state_digest_blake3 = source_state_digest(
        &staged,
        request.source_bundle_path.map(|_| manifest_authority.source_bundle_manifest_blake3.as_str()),
    )?;
    let plan = build_transition_plan(
        &manifest_authority,
        &staged,
        &source_state_digest_blake3,
        request.source_bundle_path.is_some(),
    )?;
    write_json_create_new(&request.scratch_dir.join(PLAN_FILE_NAME), &plan)?;

    let promotion_stage_ids = promotion_stage_ids(&plan);
    let stage0_root = request.scratch_dir.join("stage0-mini");
    let planned_executables = if request.source_bundle_path.is_some() {
        let mut planned = crate::stagex_stage0::planned_stage0_executables(&stage0_root)
            .into_iter()
            .map(|entry| PlannedExecutable {
                authorization_id: entry.authorization_id,
                source_stage_id: entry.source_stage_id,
                path: entry.path,
                digest_hex: entry.digest_blake3,
            })
            .collect::<Vec<_>>();
        planned.extend(crate::stagex_stage0_full::planned_stage0_full_executables(&stage0_root).into_iter().map(
            |entry| PlannedExecutable {
                authorization_id: entry.authorization_id,
                source_stage_id: entry.source_stage_id,
                path: entry.path,
                digest_hex: entry.digest_blake3,
            },
        ));
        planned
    } else {
        Vec::new()
    };
    let policy = ProtectedExecPolicy::from_stagex_plan(
        staged.seed.clone(),
        HEX0_SEED_BLAKE3.to_string(),
        &promotion_stage_ids,
        &planned_executables,
    )
    .map_err(|error| StagexTransitionError::ProtectedExec(error.to_string()))?;
    let supervisor = install_current_thread_exec_supervisor(policy)
        .map_err(|error| StagexTransitionError::ProtectedExec(error.to_string()))?;

    run_hex0_reproduction(&staged)?;
    let hex0_promotion =
        promote_executable(&supervisor, HEX0_REPRODUCTION_STAGE_ID, &staged.reproduced_hex0, HEX0_SEED_BLAKE3)?;
    run_kaem_materialization(&staged)?;
    let kaem_promotion =
        promote_executable(&supervisor, KAEM_MATERIALIZATION_STAGE_ID, &staged.kaem, KAEM_OUTPUT_BLAKE3)?;
    run_kaem_smoke(&staged)?;
    let stage0_full = match (request.source_bundle_path, request.stage0_answers_path) {
        (Some(source_bundle_path), Some(answers_path)) => {
            let mini = crate::stagex_stage0::derive_protected_stage0_mini_inventory(
                crate::stagex_stage0::Stage0MiniInventoryRequest {
                    source_bundle_path,
                    expected_source_bundle_blake3: &manifest_authority.source_bundle_manifest_blake3,
                    reproduced_hex0_path: &staged.reproduced_hex0,
                    kaem_path: &staged.kaem,
                    scratch_dir: &stage0_root,
                },
                &supervisor,
            )
            .map_err(|error| StagexTransitionError::ProtectedExec(error.to_string()))?;
            Some(
                crate::stagex_stage0_full::derive_protected_stage0_full_inventory(mini, answers_path, &supervisor)
                    .map_err(|error| StagexTransitionError::ProtectedExec(error.to_string()))?,
            )
        }
        (None, None) => None,
        _ => unreachable!("Stage0 optional inputs validated before execution"),
    };

    thread::sleep(Duration::from_millis(AUDIT_FLUSH_WAIT_MS));
    let protected_exec_events = supervisor.audit_events();
    write_json_create_new(&request.scratch_dir.join(AUDIT_FILE_NAME), &protected_exec_events)?;
    validate_transition_audit(&staged, stage0_full.as_ref(), &protected_exec_events)?;
    let kaem_smoke_digest_blake3 = blake3_file_hex(&staged.kaem_smoke).map_err(protected_digest_error)?;
    let report = StagexTransitionReport {
        schema_version: STAGEX_TRANSITION_REPORT_SCHEMA_V1.to_string(),
        status: "complete".to_string(),
        plan_digest_blake3: stagex_plan_digest_blake3(&plan)
            .map_err(|error| StagexTransitionError::Plan(vec![error.to_string()]))?,
        source_state_digest_blake3,
        seed_digest_blake3: HEX0_SEED_BLAKE3.to_string(),
        reproduced_hex0_digest_blake3: HEX0_SEED_BLAKE3.to_string(),
        kaem_digest_blake3: KAEM_OUTPUT_BLAKE3.to_string(),
        kaem_smoke_digest_blake3,
        stage0_full,
        fallback_events: Vec::new(),
        promotions: vec![hex0_promotion, kaem_promotion],
        protected_exec_events,
    };
    write_json_create_new(&request.scratch_dir.join(REPORT_FILE_NAME), &report)?;
    Ok(report)
}

fn validate_stage0_optional_inputs(
    source_bundle_path: Option<&Path>,
    answers_path: Option<&Path>,
) -> Result<(), StagexTransitionError> {
    if source_bundle_path.is_some() == answers_path.is_some() {
        return Ok(());
    }
    Err(StagexTransitionError::InvalidInput(
        "Stage0 source bundle and answers path must be supplied together".to_string(),
    ))
}

fn validate_optional_source_bundle(
    source_bundle_path: Option<&Path>,
    manifest_authority: &TransitionManifestAuthority,
) -> Result<(), StagexTransitionError> {
    let Some(source_bundle_path) = source_bundle_path else {
        return Ok(());
    };
    let manifest = crate::source_bundle::read_source_bundle(source_bundle_path)
        .map_err(|error| StagexTransitionError::InvalidInput(error.to_string()))?;
    if manifest.manifest_blake3 != manifest_authority.source_bundle_manifest_blake3 {
        return Err(StagexTransitionError::InvalidInput(format!(
            "Stage0 source bundle BLAKE3 mismatch: expected {}, observed {}",
            manifest_authority.source_bundle_manifest_blake3, manifest.manifest_blake3
        )));
    }
    assert!(!manifest.records.is_empty());
    assert_eq!(manifest.manifest_blake3, manifest_authority.source_bundle_manifest_blake3);
    Ok(())
}

fn load_transition_manifest_authority(path: &Path) -> Result<TransitionManifestAuthority, StagexTransitionError> {
    let bytes = fs::read(path).map_err(|source| io_error("reading StageX transition manifest", source))?;
    validate_size("transition manifest", bytes.len(), LINEAGE_MANIFEST_MAX_BYTES)?;
    let manifest = crate::bootstrap_source_root::validate_stagex_lineage_manifest(&bytes).map_err(|diagnostics| {
        StagexTransitionError::InvalidInput(crate::bootstrap_source_root::format_diagnostics(&diagnostics))
    })?;
    let source_bundle_manifest_blake3 = validate_transition_manifest_fields(&manifest)?;
    Ok(TransitionManifestAuthority {
        digest_blake3: blake3::hash(&bytes).to_hex().to_string(),
        source_bundle_manifest_blake3,
    })
}

fn validate_transition_manifest_fields(manifest: &LineageManifest) -> Result<String, StagexTransitionError> {
    if manifest.seed.seed_bytes_len != HEX0_SEED_BYTES || manifest.seed.seed_digest.hex_value != HEX0_SEED_BLAKE3 {
        return Err(StagexTransitionError::InvalidInput(
            "transition manifest does not bind the audited 229-byte hex0 seed".to_string(),
        ));
    }
    let sources: BTreeMap<&str, &str> = manifest
        .source_artifacts
        .iter()
        .map(|source| (source.id.as_str(), source.digest.hex_value.as_str()))
        .collect();
    require_manifest_digest(&sources, "hex0-source", HEX0_SOURCE_BLAKE3)?;
    require_manifest_digest(&sources, "kaem-minimal-source", KAEM_SOURCE_BLAKE3)?;
    require_manifest_digest(&sources, "kaem-smoke-script", EMPTY_KAEM_SMOKE_SOURCE_BLAKE3)?;
    require_manifest_digest(
        &sources,
        "stage0-source-bundle",
        crate::stagex_sources::STAGEX_SOURCE_BUNDLE_MANIFEST_BLAKE3,
    )?;
    for (artifact_id, digest_blake3) in crate::stagex_sources::expected_stagex_source_artifact_digests() {
        require_manifest_digest(&sources, artifact_id, digest_blake3)?;
    }
    require_manifest_digest(&sources, "stage0-amd64-answers", crate::stagex_stage0_full::STAGE0_ANSWERS_BLAKE3)?;
    require_manifest_digest(&sources, "stage0-after-script", EMPTY_KAEM_SMOKE_SOURCE_BLAKE3)?;
    let generated: BTreeMap<&str, &str> = manifest
        .generated_artifacts
        .iter()
        .map(|artifact| (artifact.id.as_str(), artifact.digest.hex_value.as_str()))
        .collect();
    require_manifest_digest(&generated, "hex0-reproduced", HEX0_SEED_BLAKE3)?;
    require_manifest_digest(&generated, "kaem-0", KAEM_OUTPUT_BLAKE3)?;
    for expected in crate::stagex_stage0::STAGE0_EXPECTED_EXECUTABLES {
        if expected.artifact_id == "stage0-hex0-copy" {
            continue;
        }
        require_manifest_digest(&generated, expected.artifact_id, expected.digest_blake3)?;
    }
    for expected in crate::stagex_stage0_full::STAGE0_FULL_EXPECTED_EXECUTABLES {
        require_manifest_digest(&generated, expected.artifact_id, expected.digest_blake3)?;
    }
    let assumption_ids: BTreeSet<&str> = manifest.environment_assumptions.iter().map(|item| item.id.as_str()).collect();
    if !assumption_ids.contains("linux-kernel") || !assumption_ids.contains("mantle-orchestrator") {
        return Err(StagexTransitionError::InvalidInput(
            "transition manifest omits kernel or orchestrator assumptions".to_string(),
        ));
    }
    Ok(crate::stagex_sources::STAGEX_SOURCE_BUNDLE_MANIFEST_BLAKE3.to_string())
}

fn require_manifest_digest(
    entries: &BTreeMap<&str, &str>,
    id: &str,
    expected_digest: &str,
) -> Result<(), StagexTransitionError> {
    if entries.get(id).copied() == Some(expected_digest) {
        return Ok(());
    }
    Err(StagexTransitionError::InvalidInput(format!(
        "transition manifest has missing or substituted digest for {id}"
    )))
}

fn stage_transition_inputs(
    request: &StagexTransitionRequest<'_>,
) -> Result<StagedTransitionPaths, StagexTransitionError> {
    if request.scratch_dir.exists() {
        return Err(StagexTransitionError::InvalidInput(format!(
            "scratch path already exists: {}",
            request.scratch_dir.display()
        )));
    }
    fs::create_dir(request.scratch_dir).map_err(|source| io_error("creating StageX scratch", source))?;
    let seed_bytes = read_checked_input(request.seed_path, "hex0 seed", HEX0_SEED_BLAKE3, AUDIT_SEED_MAX_BYTES)?;
    let seed_len = u32::try_from(seed_bytes.len())
        .map_err(|_| StagexTransitionError::InvalidInput("hex0 seed length exceeds u32".to_string()))?;
    if seed_len != HEX0_SEED_BYTES {
        return Err(StagexTransitionError::InvalidInput(format!(
            "hex0 seed has {seed_len} bytes; expected {HEX0_SEED_BYTES}"
        )));
    }
    let hex0_source_bytes =
        read_checked_input(request.hex0_source_path, "hex0 source", HEX0_SOURCE_BLAKE3, SOURCE_FILE_MAX_BYTES)?;
    let kaem_source_bytes =
        read_checked_input(request.kaem_source_path, "kaem source", KAEM_SOURCE_BLAKE3, SOURCE_FILE_MAX_BYTES)?;

    let staged = StagedTransitionPaths {
        seed: request.scratch_dir.join("hex0-seed"),
        hex0_source: request.scratch_dir.join("hex0_AMD64.hex0"),
        kaem_source: request.scratch_dir.join("kaem-minimal.hex0"),
        reproduced_hex0: request.scratch_dir.join("hex0-reproduced"),
        kaem: request.scratch_dir.join("kaem-0"),
        kaem_smoke_script: request.scratch_dir.join("kaem-smoke.kaem"),
        kaem_smoke: request.scratch_dir.join("kaem-smoke.txt"),
    };
    write_file_create_new(&staged.seed, &seed_bytes)?;
    write_file_create_new(&staged.hex0_source, &hex0_source_bytes)?;
    write_file_create_new(&staged.kaem_source, &kaem_source_bytes)?;
    write_file_create_new(&staged.kaem_smoke_script, &[])?;
    set_executable_owner_only(&staged.seed)?;
    Ok(staged)
}

fn source_state_digest(
    staged: &StagedTransitionPaths,
    source_bundle_manifest_blake3: Option<&str>,
) -> Result<String, StagexTransitionError> {
    let seed = fs::read(&staged.seed).map_err(|source| io_error("reading staged seed", source))?;
    let hex0_source = fs::read(&staged.hex0_source).map_err(|source| io_error("reading staged hex0 source", source))?;
    let kaem_source = fs::read(&staged.kaem_source).map_err(|source| io_error("reading staged kaem source", source))?;
    let kaem_smoke_script =
        fs::read(&staged.kaem_smoke_script).map_err(|source| io_error("reading staged kaem smoke script", source))?;
    let mut hasher = blake3::Hasher::new();
    hasher.update(SOURCE_STATE_DOMAIN);
    update_length_bound_bytes(&mut hasher, &seed)?;
    update_length_bound_bytes(&mut hasher, &hex0_source)?;
    update_length_bound_bytes(&mut hasher, &kaem_source)?;
    update_length_bound_bytes(&mut hasher, &kaem_smoke_script)?;
    if let Some(source_bundle_manifest_blake3) = source_bundle_manifest_blake3 {
        update_length_bound_bytes(&mut hasher, source_bundle_manifest_blake3.as_bytes())?;
    }
    Ok(hasher.finalize().to_hex().to_string())
}

fn update_length_bound_bytes(hasher: &mut blake3::Hasher, bytes: &[u8]) -> Result<(), StagexTransitionError> {
    let length = u64::try_from(bytes.len())
        .map_err(|_| StagexTransitionError::InvalidInput("source length exceeds u64".to_string()))?;
    hasher.update(&length.to_le_bytes());
    hasher.update(bytes);
    Ok(())
}

fn build_transition_plan(
    manifest_authority: &TransitionManifestAuthority,
    staged: &StagedTransitionPaths,
    source_state_digest_blake3: &str,
    include_stage0_mini: bool,
) -> Result<StagexMaterializationPlan, StagexTransitionError> {
    let mut stages = vec![
        transition_stage_plan(
            HEX0_REPRODUCTION_STAGE_ID,
            &[],
            &["hex0-source"],
            &[],
            &["hex0-reproduced"],
            authorization("exec:hex0-seed", &staged.seed, STAGEX_SEED_SOURCE_STAGE_ID, HEX0_SEED_BLAKE3),
        ),
        transition_stage_plan(
            KAEM_MATERIALIZATION_STAGE_ID,
            &[HEX0_REPRODUCTION_STAGE_ID],
            &["kaem-minimal-source"],
            &["hex0-reproduced"],
            &["kaem-0"],
            authorization(
                "exec:hex0-reproduced",
                &staged.reproduced_hex0,
                HEX0_REPRODUCTION_STAGE_ID,
                HEX0_SEED_BLAKE3,
            ),
        ),
        transition_stage_plan(
            KAEM_SMOKE_STAGE_ID,
            &[KAEM_MATERIALIZATION_STAGE_ID],
            &["kaem-smoke-script"],
            &["kaem-0"],
            &["kaem-smoke-observation"],
            authorization("exec:kaem-0", &staged.kaem, KAEM_MATERIALIZATION_STAGE_ID, KAEM_OUTPUT_BLAKE3),
        ),
    ];
    if include_stage0_mini {
        let stage0_root = staged.seed.parent().expect("staged seed has parent").join("stage0-mini");
        stages.extend(stage0_mini_stage_plans(&stage0_root));
        stages.extend(stage0_full_stage_plans(&stage0_root));
    }
    let stage_count_max = if include_stage0_mini {
        TRANSITION_WITH_STAGE0_STAGE_COUNT_MAX
    } else {
        TRANSITION_STAGE_COUNT_MAX
    };
    let plan = StagexMaterializationPlan {
        schema_version: STAGEX_PLAN_SCHEMA_V1.to_string(),
        lineage_manifest_digest_blake3: manifest_authority.digest_blake3.clone(),
        source_state_digest_blake3: source_state_digest_blake3.to_string(),
        protected_transition_stage_id: HEX0_REPRODUCTION_STAGE_ID.to_string(),
        environment_assumption_ids: vec!["linux-kernel".to_string(), "mantle-orchestrator".to_string()],
        stage_count_max,
        stages,
    };
    let validation = validate_stagex_plan(&plan);
    if validation.is_valid() {
        return Ok(plan);
    }
    Err(StagexTransitionError::Plan(validation.errors.into_iter().map(|error| error.to_string()).collect()))
}

fn stage0_full_stage_plans(root: &Path) -> Vec<StagexStagePlan> {
    vec![
        stage0_stage_plan(
            "stage0-full-m2-mesoplanet",
            &[
                "stage0-kaem",
                "stage0-m2",
                "stage0-blood-elf-bootstrap",
                "stage0-m1",
                "stage0-hex2",
            ],
            &["stage0-amd64-answers", "stage0-after-script"],
            &[
                "stage0-kaem",
                "stage0-m2",
                "stage0-blood-elf-bootstrap",
                "stage0-m1",
                "stage0-hex2",
            ],
            &["stage0-full-m2-mesoplanet"],
            vec![
                stage0_authorization("stage0-full-m2-mesoplanet", root, "stage0-kaem"),
                stage0_authorization("stage0-full-m2-mesoplanet", root, "stage0-m2"),
                stage0_authorization("stage0-full-m2-mesoplanet", root, "stage0-blood-elf-bootstrap"),
                stage0_authorization("stage0-full-m2-mesoplanet", root, "stage0-m1"),
                stage0_authorization("stage0-full-m2-mesoplanet", root, "stage0-hex2"),
            ],
        ),
        stage0_stage_plan(
            "stage0-full-blood-elf",
            &[
                "stage0-full-m2-mesoplanet",
                "stage0-m2",
                "stage0-blood-elf-bootstrap",
                "stage0-m1",
                "stage0-hex2",
            ],
            &[],
            &["stage0-m2", "stage0-blood-elf-bootstrap", "stage0-m1", "stage0-hex2"],
            &["stage0-full-blood-elf"],
            vec![
                stage0_authorization("stage0-full-blood-elf", root, "stage0-m2"),
                stage0_authorization("stage0-full-blood-elf", root, "stage0-blood-elf-bootstrap"),
                stage0_authorization("stage0-full-blood-elf", root, "stage0-m1"),
                stage0_authorization("stage0-full-blood-elf", root, "stage0-hex2"),
            ],
        ),
        stage0_stage_plan(
            "stage0-full-get-machine",
            &["stage0-full-blood-elf", "stage0-m2", "stage0-m1", "stage0-hex2"],
            &[],
            &["stage0-full-blood-elf", "stage0-m2", "stage0-m1", "stage0-hex2"],
            &["stage0-full-get-machine"],
            vec![
                stage0_authorization("stage0-full-get-machine", root, "stage0-m2"),
                stage0_full_authorization("stage0-full-get-machine", root, "stage0-full-blood-elf"),
                stage0_authorization("stage0-full-get-machine", root, "stage0-m1"),
                stage0_authorization("stage0-full-get-machine", root, "stage0-hex2"),
            ],
        ),
        stage0_stage_plan(
            "stage0-full-m2-planet",
            &[
                "stage0-full-get-machine",
                "stage0-full-blood-elf",
                "stage0-m2",
                "stage0-m1",
                "stage0-hex2",
            ],
            &[],
            &["stage0-full-blood-elf", "stage0-m2", "stage0-m1", "stage0-hex2"],
            &["stage0-full-m2-planet"],
            vec![
                stage0_authorization("stage0-full-m2-planet", root, "stage0-m2"),
                stage0_full_authorization("stage0-full-m2-planet", root, "stage0-full-blood-elf"),
                stage0_authorization("stage0-full-m2-planet", root, "stage0-m1"),
                stage0_authorization("stage0-full-m2-planet", root, "stage0-hex2"),
            ],
        ),
        stage0_stage_plan(
            "stage0-full-extras",
            &[
                "stage0-full-m2-planet",
                "stage0-full-m2-mesoplanet",
                "stage0-full-blood-elf",
                "stage0-m1",
                "stage0-hex2",
                "stage0-kaem",
            ],
            &[],
            &[
                "stage0-full-m2-mesoplanet",
                "stage0-full-m2-planet",
                "stage0-full-blood-elf",
                "stage0-m1",
                "stage0-hex2",
                "stage0-kaem",
            ],
            &[
                "stage0-full-sha256sum",
                "stage0-full-match",
                "stage0-full-mkdir",
                "stage0-full-untar",
                "stage0-full-ungz",
                "stage0-full-unbz2",
                "stage0-full-unxz",
                "stage0-full-catm",
                "stage0-full-cp",
                "stage0-full-chmod",
                "stage0-full-rm",
                "stage0-full-replace",
                "stage0-full-wrap",
            ],
            vec![
                stage0_authorization("stage0-full-extras", root, "stage0-kaem"),
                stage0_full_authorization("stage0-full-extras", root, "stage0-full-m2-mesoplanet"),
                stage0_full_authorization("stage0-full-extras", root, "stage0-full-m2-planet"),
                stage0_full_authorization("stage0-full-extras", root, "stage0-full-blood-elf"),
                stage0_authorization("stage0-full-extras", root, "stage0-m1"),
                stage0_authorization("stage0-full-extras", root, "stage0-hex2"),
            ],
        ),
        stage0_stage_plan(
            "stage0-full-sha256-verify",
            &[
                "stage0-full-extras",
                "stage0-full-m2-mesoplanet",
                "stage0-full-m2-planet",
                "stage0-full-blood-elf",
                "stage0-full-get-machine",
                "stage0-m1",
                "stage0-hex2",
                "stage0-kaem",
            ],
            &[],
            &[
                "stage0-full-sha256sum",
                "stage0-full-blood-elf",
                "stage0-full-catm",
                "stage0-full-chmod",
                "stage0-full-cp",
                "stage0-full-get-machine",
                "stage0-hex2",
                "stage0-kaem",
                "stage0-m1",
                "stage0-full-m2-mesoplanet",
                "stage0-full-m2-planet",
                "stage0-full-match",
                "stage0-full-mkdir",
                "stage0-full-replace",
                "stage0-full-rm",
                "stage0-full-ungz",
                "stage0-full-unbz2",
                "stage0-full-unxz",
                "stage0-full-untar",
            ],
            &["stage0-full-sha256-verification"],
            vec![stage0_full_authorization(
                "stage0-full-sha256-verify",
                root,
                "stage0-full-sha256sum",
            )],
        ),
        stage0_stage_plan(
            "stage0-full-after-smoke",
            &["stage0-full-sha256-verify", "stage0-kaem"],
            &[],
            &["stage0-full-sha256-verification", "stage0-kaem"],
            &["stage0-full-after-observation"],
            vec![stage0_authorization("stage0-full-after-smoke", root, "stage0-kaem")],
        ),
    ]
}

fn stage0_full_authorization(stage_id: &str, root: &Path, artifact_id: &str) -> StagexExecutableAuthorization {
    let expected = crate::stagex_stage0_full::STAGE0_FULL_EXPECTED_EXECUTABLES
        .iter()
        .find(|expected| expected.artifact_id == artifact_id)
        .expect("full Stage0 plan artifact has expected executable identity");
    let source_stage_id = match artifact_id {
        "stage0-full-m2-mesoplanet" => "stage0-full-m2-mesoplanet",
        "stage0-full-m2-planet" => "stage0-full-m2-planet",
        "stage0-full-blood-elf" => "stage0-full-blood-elf",
        "stage0-full-sha256sum" => "stage0-full-extras",
        _ => panic!("full Stage0 authorization has no producer stage: {artifact_id}"),
    };
    authorization(
        &format!("exec:{stage_id}:{artifact_id}"),
        &root.join(expected.relative_path),
        source_stage_id,
        expected.digest_blake3,
    )
}

fn stage0_mini_stage_plans(root: &Path) -> Vec<StagexStagePlan> {
    let source_artifacts = [
        "stage0-source-bundle",
        "stage0-posix-amd64-source",
        "bootstrap-seeds-source",
        "m2-planet-source",
        "m2libc-source",
        "mescc-tools-source",
        "mescc-tools-extra-source",
        "m2-mesoplanet-source",
    ];
    vec![
        stage0_stage_plan(
            "stage0-hex1",
            &[KAEM_MATERIALIZATION_STAGE_ID, HEX0_REPRODUCTION_STAGE_ID],
            &source_artifacts,
            &["kaem-0", "hex0-reproduced"],
            &["stage0-hex0-copy", "stage0-hex1"],
            vec![
                authorization(
                    "exec:stage0-hex1:kaem-0",
                    &root.parent().expect("Stage0 root has parent").join("kaem-0"),
                    KAEM_MATERIALIZATION_STAGE_ID,
                    KAEM_OUTPUT_BLAKE3,
                ),
                stage0_authorization("stage0-hex1", root, "stage0-hex0-copy"),
            ],
        ),
        stage0_stage("stage0-hex2-bootstrap", &["stage0-hex1"], &["stage0-hex1"], &["stage0-hex2-bootstrap"], root, &[
            "stage0-hex1",
        ]),
        stage0_stage("stage0-catm", &["stage0-hex2-bootstrap"], &["stage0-hex2-bootstrap"], &["stage0-catm"], root, &[
            "stage0-hex2-bootstrap",
        ]),
        stage0_stage(
            "stage0-m0",
            &["stage0-catm", "stage0-hex2-bootstrap"],
            &["stage0-catm", "stage0-hex2-bootstrap"],
            &["stage0-m0"],
            root,
            &["stage0-catm", "stage0-hex2-bootstrap"],
        ),
        stage0_stage(
            "stage0-cc-amd64",
            &["stage0-m0", "stage0-catm", "stage0-hex2-bootstrap"],
            &["stage0-m0", "stage0-catm", "stage0-hex2-bootstrap"],
            &["stage0-cc-amd64"],
            root,
            &["stage0-m0", "stage0-catm", "stage0-hex2-bootstrap"],
        ),
        stage0_stage(
            "stage0-m2",
            &["stage0-cc-amd64", "stage0-m0", "stage0-catm", "stage0-hex2-bootstrap"],
            &["stage0-cc-amd64", "stage0-m0", "stage0-catm", "stage0-hex2-bootstrap"],
            &["stage0-m2"],
            root,
            &["stage0-cc-amd64", "stage0-m0", "stage0-catm", "stage0-hex2-bootstrap"],
        ),
        stage0_stage(
            "stage0-blood-elf-bootstrap",
            &["stage0-m2", "stage0-catm", "stage0-m0", "stage0-hex2-bootstrap"],
            &["stage0-m2", "stage0-catm", "stage0-m0", "stage0-hex2-bootstrap"],
            &["stage0-blood-elf-bootstrap"],
            root,
            &["stage0-m2", "stage0-catm", "stage0-m0", "stage0-hex2-bootstrap"],
        ),
        stage0_stage(
            "stage0-m1-bootstrap",
            &[
                "stage0-m2",
                "stage0-blood-elf-bootstrap",
                "stage0-m0",
                "stage0-catm",
                "stage0-hex2-bootstrap",
            ],
            &[
                "stage0-m2",
                "stage0-blood-elf-bootstrap",
                "stage0-m0",
                "stage0-catm",
                "stage0-hex2-bootstrap",
            ],
            &["stage0-m1-bootstrap"],
            root,
            &[
                "stage0-m2",
                "stage0-blood-elf-bootstrap",
                "stage0-m0",
                "stage0-catm",
                "stage0-hex2-bootstrap",
            ],
        ),
        stage0_stage(
            "stage0-hex2-intermediate",
            &[
                "stage0-m2",
                "stage0-blood-elf-bootstrap",
                "stage0-m1-bootstrap",
                "stage0-catm",
                "stage0-hex2-bootstrap",
            ],
            &[
                "stage0-m2",
                "stage0-blood-elf-bootstrap",
                "stage0-m1-bootstrap",
                "stage0-catm",
                "stage0-hex2-bootstrap",
            ],
            &["stage0-hex2-intermediate"],
            root,
            &[
                "stage0-m2",
                "stage0-blood-elf-bootstrap",
                "stage0-m1-bootstrap",
                "stage0-catm",
                "stage0-hex2-bootstrap",
            ],
        ),
        stage0_stage(
            "stage0-m1",
            &[
                "stage0-m2",
                "stage0-blood-elf-bootstrap",
                "stage0-m1-bootstrap",
                "stage0-hex2-intermediate",
            ],
            &[
                "stage0-m2",
                "stage0-blood-elf-bootstrap",
                "stage0-m1-bootstrap",
                "stage0-hex2-intermediate",
            ],
            &["stage0-m1"],
            root,
            &[
                "stage0-m2",
                "stage0-blood-elf-bootstrap",
                "stage0-m1-bootstrap",
                "stage0-hex2-intermediate",
            ],
        ),
        stage0_stage(
            "stage0-hex2",
            &[
                "stage0-m2",
                "stage0-blood-elf-bootstrap",
                "stage0-m1",
                "stage0-hex2-intermediate",
            ],
            &[
                "stage0-m2",
                "stage0-blood-elf-bootstrap",
                "stage0-m1",
                "stage0-hex2-intermediate",
            ],
            &["stage0-hex2"],
            root,
            &[
                "stage0-m2",
                "stage0-blood-elf-bootstrap",
                "stage0-m1",
                "stage0-hex2-intermediate",
            ],
        ),
        stage0_stage(
            "stage0-kaem",
            &["stage0-m2", "stage0-blood-elf-bootstrap", "stage0-m1", "stage0-hex2"],
            &["stage0-m2", "stage0-blood-elf-bootstrap", "stage0-m1", "stage0-hex2"],
            &["stage0-kaem"],
            root,
            &["stage0-m2", "stage0-blood-elf-bootstrap", "stage0-m1", "stage0-hex2"],
        ),
    ]
}

fn stage0_stage(
    stage_id: &str,
    predecessors: &[&str],
    inputs: &[&str],
    outputs: &[&str],
    root: &Path,
    executable_artifact_ids: &[&str],
) -> StagexStagePlan {
    let authorizations = executable_artifact_ids
        .iter()
        .map(|artifact_id| stage0_authorization(stage_id, root, artifact_id))
        .collect();
    stage0_stage_plan(stage_id, predecessors, &[], inputs, outputs, authorizations)
}

fn stage0_stage_plan(
    stage_id: &str,
    predecessors: &[&str],
    sources: &[&str],
    inputs: &[&str],
    outputs: &[&str],
    executable_authorizations: Vec<StagexExecutableAuthorization>,
) -> StagexStagePlan {
    StagexStagePlan {
        id: stage_id.to_string(),
        immediate_predecessor_stage_ids: predecessors.iter().map(|value| (*value).to_string()).collect(),
        source_artifact_ids: sources.iter().map(|value| (*value).to_string()).collect(),
        input_artifact_ids: inputs.iter().map(|value| (*value).to_string()).collect(),
        output_artifact_ids: outputs.iter().map(|value| (*value).to_string()).collect(),
        executable_authorizations,
        timeout_ms_max: STAGE0_STAGE_TIMEOUT_MS,
        output_bytes_max: GENERATED_EXECUTABLE_MAX_BYTES,
        parallel_job_count_max: STAGE_JOB_COUNT,
    }
}

fn stage0_authorization(stage_id: &str, root: &Path, artifact_id: &str) -> StagexExecutableAuthorization {
    let expected = crate::stagex_stage0::STAGE0_EXPECTED_EXECUTABLES
        .iter()
        .find(|expected| expected.artifact_id == artifact_id)
        .expect("Stage0 plan artifact has expected executable identity");
    authorization(
        &format!("exec:{stage_id}:{artifact_id}"),
        &root.join(expected.relative_path),
        expected.source_stage_id,
        expected.digest_blake3,
    )
}

fn promotion_stage_ids(plan: &StagexMaterializationPlan) -> Vec<String> {
    let mut source_stage_ids = BTreeSet::new();
    for stage in &plan.stages {
        for authorization in &stage.executable_authorizations {
            if authorization.source_stage_id != STAGEX_SEED_SOURCE_STAGE_ID {
                source_stage_ids.insert(authorization.source_stage_id.clone());
            }
        }
    }
    source_stage_ids.into_iter().collect()
}

fn transition_stage_plan(
    id: &str,
    predecessors: &[&str],
    sources: &[&str],
    inputs: &[&str],
    outputs: &[&str],
    authorization: StagexExecutableAuthorization,
) -> StagexStagePlan {
    StagexStagePlan {
        id: id.to_string(),
        immediate_predecessor_stage_ids: predecessors.iter().map(|value| (*value).to_string()).collect(),
        source_artifact_ids: sources.iter().map(|value| (*value).to_string()).collect(),
        input_artifact_ids: inputs.iter().map(|value| (*value).to_string()).collect(),
        output_artifact_ids: outputs.iter().map(|value| (*value).to_string()).collect(),
        executable_authorizations: vec![authorization],
        timeout_ms_max: STAGE_TIMEOUT_MS,
        output_bytes_max: GENERATED_EXECUTABLE_MAX_BYTES,
        parallel_job_count_max: STAGE_JOB_COUNT,
    }
}

fn authorization(
    id: &str,
    absolute_path: &Path,
    source_stage_id: &str,
    digest_blake3: &str,
) -> StagexExecutableAuthorization {
    StagexExecutableAuthorization {
        id: id.to_string(),
        absolute_path: absolute_path.display().to_string(),
        role: "stagex-transition-runner".to_string(),
        source_stage_id: source_stage_id.to_string(),
        digest_blake3: digest_blake3.to_string(),
    }
}

fn run_hex0_reproduction(staged: &StagedTransitionPaths) -> Result<(), StagexTransitionError> {
    run_bounded_command(BoundedCommandRequest {
        executable: &staged.seed,
        args: &[staged.hex0_source.clone(), staged.reproduced_hex0.clone()],
        current_dir: staged.seed.parent().expect("staged seed has parent"),
        stderr_path: &staged.seed.with_extension("stderr"),
        retry_generated_etxtbsy: false,
    })?;
    let reproduced = read_checked_output(&staged.reproduced_hex0, "reproduced hex0", GENERATED_EXECUTABLE_MAX_BYTES)?;
    validate_bytes_digest("reproduced hex0", &reproduced, HEX0_SEED_BLAKE3)?;
    set_executable_owner_only(&staged.reproduced_hex0)?;
    Ok(())
}

fn run_kaem_materialization(staged: &StagedTransitionPaths) -> Result<(), StagexTransitionError> {
    run_bounded_command(BoundedCommandRequest {
        executable: &staged.reproduced_hex0,
        args: &[staged.kaem_source.clone(), staged.kaem.clone()],
        current_dir: staged.seed.parent().expect("staged seed has parent"),
        stderr_path: &staged.reproduced_hex0.with_extension("stderr"),
        retry_generated_etxtbsy: true,
    })?;
    let kaem = read_checked_output(&staged.kaem, "kaem-0", GENERATED_EXECUTABLE_MAX_BYTES)?;
    validate_bytes_digest("kaem-0", &kaem, KAEM_OUTPUT_BLAKE3)?;
    set_executable_owner_only(&staged.kaem)?;
    Ok(())
}

fn run_kaem_smoke(staged: &StagedTransitionPaths) -> Result<(), StagexTransitionError> {
    run_bounded_command(BoundedCommandRequest {
        executable: &staged.kaem,
        args: std::slice::from_ref(&staged.kaem_smoke_script),
        current_dir: staged.seed.parent().expect("staged seed has parent"),
        stderr_path: &staged.kaem.with_extension("stderr"),
        retry_generated_etxtbsy: true,
    })?;
    write_file_create_new(&staged.kaem_smoke, KAEM_SMOKE_MARKER)
}

fn run_bounded_command(request: BoundedCommandRequest<'_>) -> Result<ExitStatus, StagexTransitionError> {
    let mut attempt: u32 = 0;
    while attempt < GENERATED_LAUNCH_MAX_ATTEMPTS {
        attempt = attempt.saturating_add(1);
        let result = run_bounded_command_once(&request);
        match result {
            Err(StagexTransitionError::ProcessSpawn { source, .. })
                if request.retry_generated_etxtbsy && source.raw_os_error() == Some(libc::ETXTBSY) =>
            {
                thread::sleep(Duration::from_millis(GENERATED_LAUNCH_RETRY_DELAY_MS));
            }
            Err(StagexTransitionError::ProcessFailure { exit_code, stderr, .. })
                if request.retry_generated_etxtbsy
                    && exit_code == Some(ETXTBSY_EXIT_CODE)
                    && stderr.contains(ETXTBSY_STDERR_MARKER) =>
            {
                thread::sleep(Duration::from_millis(GENERATED_LAUNCH_RETRY_DELAY_MS));
            }
            result => return result,
        }
    }
    Err(StagexTransitionError::ProcessSpawn {
        executable: request.executable.to_path_buf(),
        source: io::Error::from_raw_os_error(libc::ETXTBSY),
    })
}

fn run_bounded_command_once(request: &BoundedCommandRequest<'_>) -> Result<ExitStatus, StagexTransitionError> {
    let stderr_file =
        File::create(request.stderr_path).map_err(|source| io_error("creating StageX stderr file", source))?;
    let mut command = Command::new(request.executable);
    command
        .args(request.args)
        .current_dir(request.current_dir)
        .env_clear()
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::from(stderr_file));
    let mut child = command.spawn().map_err(|source| StagexTransitionError::ProcessSpawn {
        executable: request.executable.to_path_buf(),
        source,
    })?;
    let started = Instant::now();
    loop {
        if let Some(status) = child.try_wait().map_err(|source| io_error("waiting for StageX process", source))? {
            return classify_process_status(request, status);
        }
        if started.elapsed() >= Duration::from_millis(STAGE_TIMEOUT_MS) {
            let _ = child.kill();
            let _ = child.wait();
            return Err(StagexTransitionError::ProcessTimeout {
                executable: request.executable.to_path_buf(),
                timeout_ms: STAGE_TIMEOUT_MS,
            });
        }
        thread::sleep(Duration::from_millis(PROCESS_POLL_MS));
    }
}

fn classify_process_status(
    request: &BoundedCommandRequest<'_>,
    status: ExitStatus,
) -> Result<ExitStatus, StagexTransitionError> {
    let stderr = read_bounded_stderr(request.stderr_path)?;
    if status.success() {
        return Ok(status);
    }
    Err(StagexTransitionError::ProcessFailure {
        executable: request.executable.to_path_buf(),
        exit_code: status.code(),
        stderr,
    })
}

fn read_bounded_stderr(path: &Path) -> Result<String, StagexTransitionError> {
    let metadata = fs::metadata(path).map_err(|source| io_error("reading StageX stderr metadata", source))?;
    if metadata.len() > PROCESS_STDERR_MAX_BYTES {
        return Err(StagexTransitionError::SizeLimit {
            role: "process stderr".to_string(),
            actual_bytes: metadata.len(),
            limit_bytes: PROCESS_STDERR_MAX_BYTES,
        });
    }
    let bytes = fs::read(path).map_err(|source| io_error("reading StageX stderr", source))?;
    Ok(String::from_utf8_lossy(&bytes).into_owned())
}

fn promote_executable(
    supervisor: &ProtectedSeccompSupervisor,
    stage_id: &str,
    path: &Path,
    expected_digest: &str,
) -> Result<OutputPromotionRecord, StagexTransitionError> {
    let actual_digest = blake3_file_hex(path).map_err(protected_digest_error)?;
    if actual_digest != expected_digest {
        return Err(StagexTransitionError::DigestMismatch {
            role: stage_id.to_string(),
            expected: expected_digest.to_string(),
            actual: actual_digest,
        });
    }
    supervisor
        .promote_verified_output(stage_id, &["format=raw".to_string()], &[PromotedExecutable {
            path: path.to_path_buf(),
            digest_hex: expected_digest.to_string(),
        }])
        .map_err(|error| StagexTransitionError::ProtectedExec(error.to_string()))
}

fn validate_transition_audit(
    staged: &StagedTransitionPaths,
    stage0_full: Option<&crate::stagex_stage0_full::Stage0FullInventoryReport>,
    events: &[ProtectedSeccompAuditEvent],
) -> Result<(), StagexTransitionError> {
    let expected = transition_expected_audit_events(staged);
    if events.len() < EXPECTED_AUDIT_EVENT_COUNT {
        return Err(StagexTransitionError::Audit(format!(
            "expected at least {EXPECTED_AUDIT_EVENT_COUNT} events, observed {}",
            events.len()
        )));
    }
    for (event, expected_event) in events.iter().take(EXPECTED_AUDIT_EVENT_COUNT).zip(expected.iter()) {
        validate_audit_event(event, expected_event)?;
    }
    match stage0_full {
        Some(report) => {
            const STAGE0_PARENT_EVENT_COUNT: usize = 1;
            let mini_command_count = usize::try_from(report.mini.command_count)
                .map_err(|_| StagexTransitionError::Audit("Stage0 command count does not fit usize".to_string()))?;
            let mini_event_count = mini_command_count
                .checked_add(STAGE0_PARENT_EVENT_COUNT)
                .ok_or_else(|| StagexTransitionError::Audit("Stage0 mini event count overflow".to_string()))?;
            let stage0_events = &events[EXPECTED_AUDIT_EVENT_COUNT..];
            if stage0_events.len() < mini_event_count {
                return Err(StagexTransitionError::Audit(format!(
                    "expected at least {mini_event_count} mini Stage0 events, observed {}",
                    stage0_events.len()
                )));
            }
            validate_stage0_audit(staged, &report.mini, &stage0_events[..mini_event_count])?;
            validate_stage0_full_audit(report, &stage0_events[mini_event_count..])
        }
        None if events.len() == EXPECTED_AUDIT_EVENT_COUNT => Ok(()),
        None => Err(StagexTransitionError::Audit(format!(
            "expected {EXPECTED_AUDIT_EVENT_COUNT} transition events, observed {}",
            events.len()
        ))),
    }
}

fn transition_expected_audit_events(
    staged: &StagedTransitionPaths,
) -> [ExpectedAuditEvent<'_>; EXPECTED_AUDIT_EVENT_COUNT] {
    [
        ExpectedAuditEvent {
            executable_path: &staged.seed,
            digest_blake3: HEX0_SEED_BLAKE3,
            inventory_entry_id: "stagex:seed:hex0".to_string(),
        },
        ExpectedAuditEvent {
            executable_path: &staged.reproduced_hex0,
            digest_blake3: HEX0_SEED_BLAKE3,
            inventory_entry_id: format!("promoted:{HEX0_REPRODUCTION_STAGE_ID}:{}", staged.reproduced_hex0.display()),
        },
        ExpectedAuditEvent {
            executable_path: &staged.kaem,
            digest_blake3: KAEM_OUTPUT_BLAKE3,
            inventory_entry_id: format!("promoted:{KAEM_MATERIALIZATION_STAGE_ID}:{}", staged.kaem.display()),
        },
    ]
}

fn validate_stage0_audit(
    staged: &StagedTransitionPaths,
    report: &crate::stagex_stage0::Stage0MiniInventoryReport,
    events: &[ProtectedSeccompAuditEvent],
) -> Result<(), StagexTransitionError> {
    const STAGE0_PARENT_EVENT_COUNT: usize = 1;
    let command_count = usize::try_from(report.command_count)
        .map_err(|_| StagexTransitionError::Audit("Stage0 command count does not fit usize".to_string()))?;
    let expected_event_count = command_count
        .checked_add(STAGE0_PARENT_EVENT_COUNT)
        .ok_or_else(|| StagexTransitionError::Audit("Stage0 audit event count overflow".to_string()))?;
    if events.len() != expected_event_count {
        return Err(StagexTransitionError::Audit(format!(
            "expected {expected_event_count} Stage0 events, observed {}",
            events.len()
        )));
    }
    let parent = ExpectedAuditEvent {
        executable_path: &staged.kaem,
        digest_blake3: KAEM_OUTPUT_BLAKE3,
        inventory_entry_id: format!("promoted:{KAEM_MATERIALIZATION_STAGE_ID}:{}", staged.kaem.display()),
    };
    validate_audit_event(&events[0], &parent)?;
    validate_stage0_child_events(report.sources_root(), &events[STAGE0_PARENT_EVENT_COUNT..])
}

fn validate_stage0_child_events(
    stage0_root: &Path,
    events: &[ProtectedSeccompAuditEvent],
) -> Result<(), StagexTransitionError> {
    let planned = crate::stagex_stage0::planned_stage0_executables(stage0_root);
    let by_path = planned.iter().map(|entry| (entry.path.as_path(), entry)).collect::<BTreeMap<_, _>>();
    let mut observed_paths = BTreeSet::new();
    for event in events {
        let expected = by_path.get(event.executable_path.as_path()).ok_or_else(|| {
            StagexTransitionError::Audit(format!(
                "Stage0 event used undeclared path {}",
                event.executable_path.display()
            ))
        })?;
        let expected_id = format!("planned:{}:{}", expected.source_stage_id, expected.authorization_id);
        let valid = event.policy_decision == "allowed"
            && event.resolved_host_path == expected.path
            && event.tracee_path == expected.path
            && event.digest_hex == expected.digest_blake3
            && event.inventory_entry_id.as_deref() == Some(expected_id.as_str());
        if !valid {
            return Err(StagexTransitionError::Audit(format!(
                "Stage0 event for {} did not match exact planned authority",
                expected.path.display()
            )));
        }
        observed_paths.insert(event.executable_path.clone());
    }
    for expected in &planned {
        if !observed_paths.contains(&expected.path) {
            return Err(StagexTransitionError::Audit(format!(
                "planned Stage0 executable was not observed: {}",
                expected.path.display()
            )));
        }
    }
    assert!(!events.is_empty());
    assert_eq!(observed_paths.len(), by_path.len());
    Ok(())
}

fn validate_stage0_full_audit(
    report: &crate::stagex_stage0_full::Stage0FullInventoryReport,
    events: &[ProtectedSeccompAuditEvent],
) -> Result<(), StagexTransitionError> {
    const STAGE0_FULL_PARENT_EVENT_COUNT: usize = 1;
    let command_count = report
        .root_command_count
        .checked_add(report.nested_command_count)
        .and_then(|count| count.checked_add(report.extra_command_count))
        .ok_or_else(|| StagexTransitionError::Audit("full Stage0 command count overflow".to_string()))?;
    let expected_event_count = usize::try_from(command_count)
        .map_err(|_| StagexTransitionError::Audit("full Stage0 command count does not fit usize".to_string()))?
        .checked_add(STAGE0_FULL_PARENT_EVENT_COUNT)
        .ok_or_else(|| StagexTransitionError::Audit("full Stage0 event count overflow".to_string()))?;
    if events.len() < expected_event_count {
        return Err(StagexTransitionError::Audit(format!(
            "expected at least {expected_event_count} full Stage0 events, observed {}",
            events.len()
        )));
    }
    let stage0_root = report.mini.sources_root();
    let mut authorities = BTreeMap::new();
    for planned in crate::stagex_stage0::planned_stage0_executables(stage0_root) {
        let inventory_id = format!("planned:{}:{}", planned.source_stage_id, planned.authorization_id);
        authorities.insert(planned.path, (planned.digest_blake3, inventory_id));
    }
    let full_planned = crate::stagex_stage0_full::planned_stage0_full_executables(stage0_root);
    for planned in &full_planned {
        let inventory_id = format!("planned:{}:{}", planned.source_stage_id, planned.authorization_id);
        authorities.insert(planned.path.clone(), (planned.digest_blake3.clone(), inventory_id));
    }
    let mut observed_paths = BTreeSet::new();
    for event in events {
        let (expected_digest, expected_id) = authorities.get(&event.executable_path).ok_or_else(|| {
            StagexTransitionError::Audit(format!(
                "full Stage0 event used undeclared path {}",
                event.executable_path.display()
            ))
        })?;
        let valid = event.policy_decision == "allowed"
            && event.resolved_host_path == event.executable_path
            && event.tracee_path == event.executable_path
            && event.digest_hex == *expected_digest
            && event.inventory_entry_id.as_deref() == Some(expected_id.as_str());
        if !valid {
            return Err(StagexTransitionError::Audit(format!(
                "full Stage0 event for {} did not match exact planned authority",
                event.executable_path.display()
            )));
        }
        observed_paths.insert(event.executable_path.clone());
    }
    for planned in full_planned {
        if !observed_paths.contains(&planned.path) {
            return Err(StagexTransitionError::Audit(format!(
                "planned full Stage0 executable was not observed: {}",
                planned.path.display()
            )));
        }
    }
    assert!(!events.is_empty());
    assert!(observed_paths.len() <= authorities.len());
    Ok(())
}

fn validate_audit_event(
    event: &ProtectedSeccompAuditEvent,
    expected: &ExpectedAuditEvent<'_>,
) -> Result<(), StagexTransitionError> {
    let expected_id = Some(expected.inventory_entry_id.as_str());
    let valid = event.policy_decision == "allowed"
        && event.executable_path == expected.executable_path
        && event.resolved_host_path == expected.executable_path
        && event.digest_hex == expected.digest_blake3
        && event.inventory_entry_id.as_deref() == expected_id;
    if valid {
        return Ok(());
    }
    Err(StagexTransitionError::Audit(format!(
        "event for {} did not match its path, digest, source stage, or allow decision",
        expected.executable_path.display()
    )))
}

fn read_checked_input(
    path: &Path,
    role: &str,
    expected_digest: &str,
    max_bytes: u64,
) -> Result<Vec<u8>, StagexTransitionError> {
    let bytes = fs::read(path).map_err(|source| io_error(&format!("reading StageX {role}"), source))?;
    validate_size(role, bytes.len(), max_bytes)?;
    validate_bytes_digest(role, &bytes, expected_digest)?;
    Ok(bytes)
}

fn read_checked_output(path: &Path, role: &str, max_bytes: u64) -> Result<Vec<u8>, StagexTransitionError> {
    let bytes = fs::read(path).map_err(|source| io_error(&format!("reading StageX {role}"), source))?;
    validate_size(role, bytes.len(), max_bytes)?;
    Ok(bytes)
}

fn validate_size(role: &str, len: usize, max_bytes: u64) -> Result<(), StagexTransitionError> {
    let actual_bytes = u64::try_from(len)
        .map_err(|_| StagexTransitionError::InvalidInput(format!("{role} byte length exceeds u64")))?;
    if actual_bytes > max_bytes {
        return Err(StagexTransitionError::SizeLimit {
            role: role.to_string(),
            actual_bytes,
            limit_bytes: max_bytes,
        });
    }
    Ok(())
}

fn validate_bytes_digest(role: &str, bytes: &[u8], expected: &str) -> Result<(), StagexTransitionError> {
    let actual = blake3::hash(bytes).to_hex().to_string();
    if actual == expected {
        return Ok(());
    }
    Err(StagexTransitionError::DigestMismatch {
        role: role.to_string(),
        expected: expected.to_string(),
        actual,
    })
}

fn write_file_create_new(path: &Path, bytes: &[u8]) -> Result<(), StagexTransitionError> {
    use std::io::Write;
    let mut file = File::options()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|source| io_error(&format!("creating {}", path.display()), source))?;
    file.write_all(bytes).map_err(|source| io_error(&format!("writing {}", path.display()), source))?;
    file.sync_all().map_err(|source| io_error(&format!("syncing {}", path.display()), source))?;
    Ok(())
}

fn write_json_create_new<T: Serialize>(path: &Path, value: &T) -> Result<(), StagexTransitionError> {
    let mut bytes = serde_json::to_vec_pretty(value)
        .map_err(|error| StagexTransitionError::InvalidInput(format!("serializing {}: {error}", path.display())))?;
    bytes.push(b'\n');
    write_file_create_new(path, &bytes)
}

fn set_executable_owner_only(path: &Path) -> Result<(), StagexTransitionError> {
    use std::os::unix::fs::PermissionsExt;
    let mut permissions = fs::metadata(path)
        .map_err(|source| io_error(&format!("reading permissions for {}", path.display()), source))?
        .permissions();
    permissions.set_mode(EXECUTABLE_MODE_OWNER_ONLY);
    fs::set_permissions(path, permissions)
        .map_err(|source| io_error(&format!("setting executable mode for {}", path.display()), source))
}

fn io_error(action: &str, source: io::Error) -> StagexTransitionError {
    StagexTransitionError::Io {
        action: action.to_string(),
        source,
    }
}

fn protected_digest_error(error: crate::protected_exec::Stage0InventoryGenerationError) -> StagexTransitionError {
    StagexTransitionError::ProtectedExec(error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    const CHILD_ENV: &str = "MANTLE_STAGE_X_TRANSITION_CHILD";
    const SCRATCH_ENV: &str = "MANTLE_STAGE_X_TRANSITION_SCRATCH";
    const SOURCE_BUNDLE_ENV: &str = "MANTLE_STAGE_X_SOURCE_BUNDLE";
    const CHILD_TEST_NAME: &str = "stagex_transition::tests::protected_transition_reproduces_seed_and_builds_kaem";
    const TEST_LINEAGE_MANIFEST_DIGEST: &str = "9e9138e807dba3b348303ad333a74cbdb8094533bf7e3357ba0971781ac43d8a";

    #[test]
    fn protected_transition_reproduces_seed_and_builds_kaem() {
        if std::env::var_os(CHILD_ENV).is_some() {
            run_protected_transition_child();
            return;
        }
        let current_exe = std::env::current_exe().unwrap();
        let output = Command::new(current_exe)
            .arg(CHILD_TEST_NAME)
            .arg("--exact")
            .arg("--nocapture")
            .env(CHILD_ENV, "1")
            .output()
            .unwrap();
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(output.status.success(), "stdout={stdout}\nstderr={stderr}");
        assert!(stdout.contains("stagex-protected-transition-ok"), "stdout={stdout}");
        assert!(!stderr.contains("/bin/sh"), "stderr={stderr}");
    }

    #[test]
    fn transition_input_digest_mismatch_fails_before_scratch_execution() {
        let temp = tempfile::tempdir().unwrap();
        let seed = temp.path().join("bad-seed");
        fs::write(&seed, b"not the audited seed").unwrap();
        let error = read_checked_input(&seed, "hex0 seed", HEX0_SEED_BLAKE3, AUDIT_SEED_MAX_BYTES).unwrap_err();
        assert!(matches!(error, StagexTransitionError::DigestMismatch { .. }));
        assert!(!temp.path().join("hex0-reproduced").exists());
    }

    #[test]
    fn substituted_manifest_digest_fails_before_execution() {
        let repo = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let bytes = fs::read(repo.join("bootstrap/stagex-transition-lineage.json")).unwrap();
        let mut value: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        value["generated_artifacts"][1]["digest"]["hex_value"] =
            serde_json::Value::String(HEX0_SEED_BLAKE3.to_string());
        let temp = tempfile::tempdir().unwrap();
        let manifest_path = temp.path().join("substituted.json");
        fs::write(&manifest_path, serde_json::to_vec(&value).unwrap()).unwrap();
        let error = load_transition_manifest_authority(&manifest_path).unwrap_err();
        assert!(matches!(error, StagexTransitionError::InvalidInput(_)));
        assert!(error.to_string().contains("kaem-0"));
    }

    #[test]
    fn audit_rejects_substituted_path_and_denied_decision() {
        let expected_path = PathBuf::from("/stagex/hex0-seed");
        let expected = ExpectedAuditEvent {
            executable_path: &expected_path,
            digest_blake3: HEX0_SEED_BLAKE3,
            inventory_entry_id: "stagex:seed:hex0".to_string(),
        };
        let event = ProtectedSeccompAuditEvent {
            pid: 1,
            syscall: "execve".to_string(),
            executable_path: PathBuf::from("/bin/sh"),
            tracee_path: PathBuf::from("/bin/sh"),
            resolved_host_path: PathBuf::from("/bin/sh"),
            digest_hex: HEX0_SEED_BLAKE3.to_string(),
            reason: "test denial".to_string(),
            phase: "protected".to_string(),
            inventory_entry_id: None,
            policy_decision: "denied".to_string(),
        };
        let error = validate_audit_event(&event, &expected).unwrap_err();
        assert!(matches!(error, StagexTransitionError::Audit(_)));
        assert!(error.to_string().contains("path, digest, source stage, or allow decision"));
    }

    #[test]
    fn bounded_retry_classification_is_exact() {
        let exact = io::Error::from_raw_os_error(libc::ETXTBSY);
        let other = io::Error::from_raw_os_error(libc::EACCES);
        let exit_marker = StagexTransitionError::ProcessFailure {
            executable: PathBuf::from("/stagex/generated"),
            exit_code: Some(ETXTBSY_EXIT_CODE),
            stderr: ETXTBSY_STDERR_MARKER.to_string(),
        };
        assert_eq!(exact.raw_os_error(), Some(libc::ETXTBSY));
        assert_ne!(other.raw_os_error(), Some(libc::ETXTBSY));
        let StagexTransitionError::ProcessFailure { exit_code, .. } = exit_marker else {
            panic!("expected process failure fixture");
        };
        assert_eq!(exit_code, Some(ETXTBSY_EXIT_CODE));
    }

    fn run_protected_transition_child() {
        let repo = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let manifest_path = repo.join("bootstrap/stagex-transition-lineage.json");
        let manifest_bytes = fs::read(&manifest_path).unwrap();
        let manifest = crate::bootstrap_source_root::validate_stagex_lineage_manifest(&manifest_bytes).unwrap();
        let manifest_digest = blake3_file_hex(&manifest_path).unwrap();
        let temp = tempfile::tempdir().unwrap();
        let scratch = std::env::var_os(SCRATCH_ENV)
            .map(PathBuf::from)
            .unwrap_or_else(|| temp.path().join("protected-transition"));
        let source_bundle = std::env::var_os(SOURCE_BUNDLE_ENV).map(PathBuf::from);
        let stage0_answers = source_bundle.as_ref().map(|_| repo.join("bootstrap/stage0-amd64.answers"));
        assert_eq!(manifest_digest, TEST_LINEAGE_MANIFEST_DIGEST);
        assert_eq!(manifest.seed.seed_digest.hex_value, HEX0_SEED_BLAKE3);
        assert_eq!(manifest.seed.seed_bytes_len, HEX0_SEED_BYTES);
        let report = materialize_protected_transition(StagexTransitionRequest {
            seed_path: &repo.join("bootstrap/seeds/AMD64/hex0-seed"),
            hex0_source_path: &repo.join("bootstrap/seeds/AMD64/hex0_AMD64.hex0"),
            kaem_source_path: &repo.join("bootstrap/seeds/AMD64/kaem-minimal.hex0"),
            lineage_manifest_path: &manifest_path,
            source_bundle_path: source_bundle.as_deref(),
            stage0_answers_path: stage0_answers.as_deref(),
            scratch_dir: &scratch,
        })
        .unwrap();
        assert_eq!(report.status, "complete");
        let expected_event_count = report.stage0_full.as_ref().map_or(EXPECTED_AUDIT_EVENT_COUNT, |stage0| {
            let mini_count = usize::try_from(stage0.mini.command_count).unwrap().checked_add(1).unwrap();
            let full_count = stage0
                .root_command_count
                .checked_add(stage0.nested_command_count)
                .and_then(|count| count.checked_add(stage0.extra_command_count))
                .and_then(|count| count.checked_add(1))
                .and_then(|count| usize::try_from(count).ok())
                .unwrap();
            EXPECTED_AUDIT_EVENT_COUNT
                .checked_add(mini_count)
                .and_then(|count| count.checked_add(full_count))
                .unwrap()
        });
        if report.stage0_full.is_some() {
            assert!(report.protected_exec_events.len() >= expected_event_count);
        } else {
            assert_eq!(report.protected_exec_events.len(), expected_event_count);
        }
        assert_eq!(report.stage0_full.is_some(), source_bundle.is_some());
        assert!(
            report
                .stage0_full
                .as_ref()
                .is_none_or(|stage0| { stage0.protected_exec_enforced && stage0.mini.protected_exec_enforced })
        );
        assert_eq!(report.promotions.len(), EXPECTED_PROMOTION_COUNT);
        assert!(report.fallback_events.is_empty());
        assert!(scratch.join(REPORT_FILE_NAME).is_file());
        assert!(scratch.join(PLAN_FILE_NAME).is_file());
        assert!(scratch.join(AUDIT_FILE_NAME).is_file());
        println!("stagex-protected-transition-ok");
    }
}
