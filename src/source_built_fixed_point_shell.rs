use std::collections::BTreeMap;
#[cfg(test)]
use std::ffi::OsString;
use std::fs;
use std::io::Write as _;
use std::os::unix::fs::MetadataExt as _;
use std::os::unix::fs::PermissionsExt as _;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::process::Output;
use std::time::Duration;
use std::time::Instant;

use serde::Deserialize;
use serde::Serialize;

use crate::cargo_free_self_build::CargoFreeSelfBuildOptions;
use crate::errors::RunError;
use crate::full_source_rust_binding_shell::FullSourceRustHostToolMaterializationRequest;
use crate::native_toolchain_closure::NativeToolchainClosureOptions;
use crate::source_built_fixed_point::plan_source_built_fixed_point;
use crate::source_built_fixed_point::InitialOutputAuthorityState;
use crate::source_built_fixed_point::ProofHermeticityMode;
use crate::source_built_fixed_point::ProofOutputRole;
use crate::source_built_fixed_point::SourceAuthorityInput;
use crate::source_built_fixed_point::SourceAuthorityRole;
use crate::source_built_fixed_point::SourceBuiltFixedPointPlan;
use crate::source_built_fixed_point::SourceBuiltFixedPointPlanInput;
use crate::source_built_fixed_point::SourceBuiltFixedPointPolicies;
use crate::source_built_fixed_point::SourceBuiltFixedPointResourceBounds;
use crate::source_built_fixed_point::SourceContentKind;
use crate::source_built_fixed_point::SOURCE_BUILT_FIXED_POINT_OPEN_FILE_DESCRIPTORS_MAX;
use crate::source_built_fixed_point_dev_cache::dev_provider_cache_key;
use crate::source_built_fixed_point_dev_cache::evaluate_fast_fail;
use crate::source_built_fixed_point_dev_cache::evaluate_provider_cache_lookup;
use crate::source_built_fixed_point_dev_cache::validate_stage_marker;
use crate::source_built_fixed_point_dev_cache::DevCachePolicies;
use crate::source_built_fixed_point_dev_cache::DevProviderCacheEntry;
use crate::source_built_fixed_point_dev_cache::DevProviderCacheLookup;
use crate::source_built_fixed_point_dev_cache::FastFailDecision;
use crate::source_built_fixed_point_dev_cache::StageCompletionMarker;
use crate::source_built_fixed_point_dev_cache::StageMarkerValidation;
use crate::source_built_fixed_point_dev_cache::DEV_CACHE_ENTRY_FILE;
use crate::source_built_fixed_point_dev_cache::FAST_FAIL_SCHEMA;
use crate::source_bundle::assemble_source_bundle;
use crate::source_bundle::materialize_source_record_payload;
use crate::source_bundle::source_built_fixed_point_profile_records;
use crate::source_bundle::SourceBuiltFixedPointProfileRecords;
use crate::source_bundle::SourceRecord;
use crate::stagex_provider::StagexProviderRequest;
use crate::stagex_transition::StagexTransitionRequest;

mod checkpoint_integration;
mod dev_resume;

use checkpoint_integration::import_provider_checkpoint_attempt;
use checkpoint_integration::print_checkpoint_import_completion;
use checkpoint_integration::publish_constructed_provider_checkpoint;
use checkpoint_integration::restore_constructed_provider_checkpoint;
use checkpoint_integration::restore_native_provider_prefix_attempt;
#[cfg(test)]
use checkpoint_integration::validate_imported_attempt_status;
#[cfg(test)]
use checkpoint_integration::validate_imported_provider_authority;

const LOGICAL_STORE_PREFIX: &str = "/mantle/store";
const PROFILE_SCHEMA: &str = "mantle-source-bundle-v1";
const LEGACY_PROVIDER_CHECKPOINT_IMPORT_PLAN_SCHEMA: &str = "mantle-source-built-fixed-point-plan-v2";
const BUILD_REPORT_SCHEMA: &str = "crunch-build-report-v1";
const OFFLINE_PREFLIGHT_REPORT_FORMAT: &str = "mantle-source-offline-preflight-v1";
const NATIVE_FAILURE_IDENTITY_COUNT_MAX: usize = 8;
const _: () = assert!(NATIVE_FAILURE_IDENTITY_COUNT_MAX > 0);
const PROOF_STATUS_SCHEMA: &str = "mantle-source-built-fixed-point-attempt-v1";
const PROOF_STATUS_RUNNING: &str = "running";
const PROOF_STATUS_FAILED: &str = "failed";
const PROOF_STATUS_COMPLETE: &str = "complete";
const STAGEX_TRANSITION_EXECUTION_DIR: &str = "stagex-transition-execution";
const STAGEX_TRANSITION_HANDOFF_REPLAY_DIR: &str = "stagex-transition-handoff-replay";
const STAGEX_PROVIDER_REPLAY_DIR: &str = "stagex-provider-replay";
pub(crate) const STAGEX_TRANSITION_REPORT_FILE: &str = "transition-report.json";
pub(crate) const STAGEX_TRANSITION_PLAN_FILE: &str = "transition-plan.json";
pub(crate) const STAGEX_TRANSITION_AUDIT_FILE: &str = "protected-exec-audit.json";
const STAGEX_TRANSITION_HANDOFF_REPORT_FILE: &str = "stagex-transition-handoff.json";
const STAGEX_TRANSITION_HANDOFF_REPORT_FORMAT: &str = "mantle-stagex-transition-handoff-v1";
const STAGEX_TRANSITION_HANDOFF_NON_CLAIM: &str = "this projection exposes only declared StageX runtime outputs; the preserved execution tree owns transition evidence";
const STAGEX_TRANSITION_HANDOFF_DIRECTORY_COUNT: usize = 8;
const STAGEX_TRANSITION_HANDOFF_DIRECTORIES: [&str; STAGEX_TRANSITION_HANDOFF_DIRECTORY_COUNT] = [
    "bash-full-stage/runtime/output",
    "coreutils-stage/runtime/output",
    "diffutils-stage/runtime/output",
    "gawk-stage/runtime/output",
    "grep-stage/runtime/output",
    "m4-stage/runtime/output",
    "make-stage/runtime/output",
    "sed-stage/runtime/output",
];
const STAGEX_TRANSITION_HANDOFF_REQUIRED_FILE_COUNT: usize = 9;
const STAGEX_TRANSITION_HANDOFF_REQUIRED_FILES: [&str; STAGEX_TRANSITION_HANDOFF_REQUIRED_FILE_COUNT] = [
    "bash-full-stage/runtime/output/bin/bash-full",
    "coreutils-stage/runtime/output/bin/cp",
    "diffutils-stage/runtime/output/bin/cmp",
    "diffutils-stage/runtime/output/bin/diff",
    "gawk-stage/runtime/output/bin/gawk",
    "grep-stage/runtime/output/bin/grep",
    "m4-stage/runtime/output/bin/m4",
    "make-stage/runtime/output/bin/make",
    "sed-stage/runtime/output/bin/sed",
];
pub(crate) const STAGEX_TRANSITION_STORE_BASENAME: &str = "ki5gkg5d6si77dl5k4mav4s6x9s8l25r-mantle-stagex-transition";
const STAGEX_TRANSITION_LOGICAL_PATH: &str = "/mantle/store/ki5gkg5d6si77dl5k4mav4s6x9s8l25r-mantle-stagex-transition";
pub(crate) const STAGEX_PROVIDER_STORE_BASENAME: &str =
    "snzd91n8dv6l21xa89vml67229n9svkg-mantle-stagex-intermediate-provider";
const STAGEX_PROVIDER_LOGICAL_PATH: &str =
    "/mantle/store/snzd91n8dv6l21xa89vml67229n9svkg-mantle-stagex-intermediate-provider";
const STAGEX_PROVIDER_EXPECTED_NORMALIZED_DIGEST: &str =
    "e1039a3c844d709f51586f7afa2aacdbbe92aa224f1e20a778ea80573603ada3";
const NATIVE_PROVIDER_ID: &str = "full-source-native-provider";
const NATIVE_ACTION_PLAN_FILE: &str = "native-provider-action-plan.json";
const NATIVE_ACTION_RECONCILIATION_FILE: &str = "native-provider-action-reconciliation.json";
const NATIVE_ADMISSION_REPORT_FILE: &str = "full-source-provider-admission.json";
const NATIVE_SOURCE_MANIFEST_FILE: &str = "native-source-closure.json";
const NATIVE_SOURCE_AUTHORITY_DIR: &str = "native-source-authority";
const TOOLCHAIN_CLOSURE_FILE: &str = "source-built-toolchain-closure.json";
const PLAN_FILE: &str = "source-built-fixed-point-plan.json";
const ATTEMPT_STATUS_FILE: &str = "attempt-status.json";
pub(crate) const FINAL_RECEIPT_FILE: &str = "deterministic-build-proof.json";
pub(crate) const FINAL_BUNDLE_DIGEST_FILE: &str = "proof-bundle-blake3.txt";
pub(crate) const PROVIDER_KIND_LINKAGE_FILE: &str = "provider-kind-linkage.json";
pub(crate) const RELEASE_INPUTS_FILE: &str = "release-evidence-inputs.json";
const LATEST_ALIAS: &str = "latest";
const LATEST_SOURCE_BUILT_ALIAS: &str = "latest-source-built-fixed-point";
const STAGING_SUFFIX: &str = ".source-built-fixed-point-staging";
const INPUTS_DIR: &str = "inputs";
const TRANSCRIPTS_DIR: &str = "transcripts";
const NATIVE_STORE_DIR: &str = "native-store";
const NATIVE_STATE_DIR: &str = "native-state";
const HOST_TOOLS_EVIDENCE_DIR: &str = "full-source-rust-host-tools";
const RUST_PROVIDER_DIR: &str = "rust-provider";
const RUST_PROVIDER_SCRATCH_DIR: &str = "rust-provider-scratch";
const FIXED_POINT_DIR: &str = "cargo-free-fixed-point";
const SOURCE_ROOT_DIR: &str = "mantle-source";
const VENDOR_RELATIVE_PATH: &str = "vendor-deps";
const STAGEX_SEED_DIR: &str = "stagex-seed";
const STAGEX_LINEAGE_DIR: &str = "stagex-lineage";
const STAGEX_SOURCE_BUNDLE_DIR: &str = "stagex-source-bundle";
const RUST_SOURCE_DIR: &str = "rust-source-archives";
const HOME_DIR: &str = "home";
const TMP_DIR: &str = "tmp";
const BUILD_OUTPUT_NAME: &str = "out";
const NATIVE_PROVIDER_NCL: &str = "bootstrap/seed-full-toolchain.ncl";
const HOST_MAKE_NCL: &str = "bootstrap/make-4.4.1-gcc10.ncl";
const HOST_LINUX_HEADERS_NCL: &str = "bootstrap/linux-headers-6.6-gcc10.ncl";
const HOST_BUSYBOX_NCL: &str = "bootstrap/busybox-1.37.0-gcc10.ncl";
const HOST_CMAKE_NCL: &str = "bootstrap/cmake-3.31.8-gcc10.ncl";
const HOST_PYTHON_NCL: &str = "bootstrap/python-3.13.5-gcc10.ncl";
const HOST_PERL_NCL: &str = "bootstrap/perl-5.10.1-gcc10.ncl";
const RUST_RECIPE_NCL: &str = "bootstrap/rust-source.ncl";
const RUST_ROUTE_PLAN_NCL: &str = "bootstrap/rust-source-musl-host-plan.ncl";
const RUSTC_RELATIVE_PATH: &str = "bin/rustc";
const TARGET_TRIPLE: &str = "x86_64-unknown-linux-musl";
const SOURCE_RECORD_COUNT_MIN: usize = 1;
const DEV_CACHE_PROVIDERS_SUBDIR: &str = "providers";
const STAGE_MARKERS_SUBDIR: &str = ".stage-markers";
const PROVIDER_CHECKPOINT_TRANSCRIPT_FILE: &str = "provider-checkpoint.txt";
const PROVIDER_CHECKPOINT_RESTORE_TRANSCRIPT_FILE: &str = "provider-checkpoint-restore.txt";
const PROVIDER_CHECKPOINT_IMPORT_REPORT_FILE: &str = "provider-checkpoint-import.json";
const CHECKPOINT_ORIGIN_EVIDENCE_DIR: &str = "provider-checkpoint-origin";
const CHECKPOINT_STAGEX_TRANSITION_PATH: &str = "payload/stagex-transition-execution";
const CHECKPOINT_STAGEX_PROVIDER_PREFIX: &str = "payload/native-store";
const CHECKPOINT_NATIVE_PROVIDER_PREFIX: &str = "payload/native-store";
const CHECKPOINT_RUST_PROVIDER_PATH: &str = "payload/rust-provider";
const CHECKPOINT_NATIVE_ADMISSION_PATH: &str = "payload/evidence/native-admission.json";
const CHECKPOINT_NATIVE_TRANSCRIPT_PATH: &str = "payload/evidence/native-provider.json";
const CHECKPOINT_NATIVE_ACTION_PLAN_PATH: &str = "payload/evidence/native-provider-action-plan.json";
const CHECKPOINT_NATIVE_ACTION_RECONCILIATION_PATH: &str =
    "payload/evidence/native-provider-action-reconciliation.json";
const CHECKPOINT_TOOLCHAIN_CLOSURE_PATH: &str = "payload/evidence/source-built-toolchain-closure.json";
const CHECKPOINT_RUST_HOST_TOOL_PREFIX: &str = "payload/rust-host-tools";
const CHECKPOINT_RUST_HOST_EVIDENCE_PATH: &str = "payload/evidence/rust-host-tools";
const CHECKPOINT_RUST_ACTION_TRUST_PATH: &str = "payload/evidence/rust-provider-action-trust";
const RUST_PROVIDER_RUSTC_RELATIVE: &str = "bin/rustc";
const RUST_PROVIDER_BUILD_RECEIPT_RELATIVE: &str = "share/mantle-rust-provider/receipts/build.json";
const RUST_PROVIDER_BINDING_RECEIPT_RELATIVE: &str = "share/mantle-rust-provider/receipts/full-source-binding.json";
const CHECKPOINT_EXECUTION_EVIDENCE_CONTEXT: &str = "mantle-source-built-provider-checkpoint-execution-evidence-v1";
const PROVIDER_RECIPE_PROJECTION_CONTEXT: &str = "mantle-source-built-provider-recipe-projection-v1";
const PROVIDER_RECIPE_PROJECTION_ROOT_COUNT: usize = 3;
const PROVIDER_RECIPE_PROJECTION_ROOTS: [&str; PROVIDER_RECIPE_PROJECTION_ROOT_COUNT] =
    ["bootstrap", "builders", "lib"];
const FAST_FAIL_NOTICE: &str = "dev fast-fail: source profile unchanged from the last published fixed-point receipt; reporting the prior success without a fresh rebuild";
const EXPECTED_SINGLE_OUTPUT_COUNT: usize = 1;
const EXPECTED_STAGE_COUNT: usize = 6;
const IMPORTED_PROVIDER_SOURCE_ROLE_COUNT: usize = 5;
const BLAKE3_HEX_LENGTH: usize = 64;
const MAX_JOBS_MIN: u32 = 1;
const MAX_JOBS_MAX: u32 = 16;
const MATERIALIZED_SOURCE_TREE_ENTRY_COUNT_MAX: u32 = 1_000_000;
const FILE_MODE_EXECUTABLE_MASK: u32 = 0o111;
const ATTEMPT_DIRECTORY_MODE: u32 = 0o700;
const SUCCESS_ALIAS_COUNT: usize = 2;
const POLICY_DIGEST_DOMAIN: &[u8] = b"mantle-source-built-fixed-point-policy-v1\0";
const CLOSURE_POLICY_TEXT: &str = "zero-seed receipt-bound native and Rust toolchain closure";
const HERMETICITY_POLICY_TEXT: &str = "strict no-fetch no-fallback no-substitution build envelope";
const EFFECT_POLICY_TEXT: &str = "mantle-build-effects-v1:read-store,write-output,environment";
const NORMALIZATION_POLICY_TEXT: &str =
    "time,timezone,locale,temp-roots,host-user-metadata,umask,modeled-randomness,order-sensitive-output-processing";

#[derive(Debug, Clone)]
pub(crate) struct SourceBuiltFixedPointOptions<'a> {
    pub(crate) source_profile: &'a Path,
    pub(crate) expected_source_profile_blake3: &'a str,
    pub(crate) expected_stagex_lineage_blake3: &'a str,
    pub(crate) expected_native_provider_blake3: &'a str,
    pub(crate) output_dir: &'a Path,
    pub(crate) bwrap: &'a Path,
    pub(crate) sandbox_shell: &'a Path,
    pub(crate) jobs: u32,
    pub(crate) elapsed_seconds_max: u64,
    pub(crate) disk_bytes_max: u64,
    pub(crate) protected_exec_events_max: u32,
    pub(crate) source_records_max: u32,
    pub(crate) proof_checkpoint_store: Option<&'a Path>,
    pub(crate) proof_checkpoint_import_attempt: Option<&'a Path>,
    pub(crate) proof_native_checkpoint_attempt: Option<&'a Path>,
    pub(crate) dev_provider_cache: Option<&'a Path>,
    pub(crate) dev_resume: bool,
    pub(crate) dev_fast_fail: bool,
    pub(crate) verbose: bool,
    pub(crate) json: bool,
}

struct MaterializedSourceDigests {
    stagex_seed: String,
    stagex_source_bundle: String,
    native_source_manifest: String,
    rust_source_archive_set: String,
    provider_recipe_projection: String,
    provider_recipe_projection_bytes: u64,
    mantle_source: String,
    vendor_inputs: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct OpenFileDescriptorLimitPlan {
    soft_limit: u64,
    hard_limit: u64,
    update_required: bool,
}

#[derive(Debug)]
struct PreparedAttempt {
    final_dir: PathBuf,
    staging_dir: PathBuf,
    source_root: PathBuf,
    stagex_seed: PathBuf,
    stagex_lineage: PathBuf,
    stagex_source_bundle: PathBuf,
    native_source_manifest: PathBuf,
    rust_source_archive_dir: PathBuf,
    native_store_dir: PathBuf,
    native_state_dir: PathBuf,
    transcripts_dir: PathBuf,
    disk_available_bytes_before: u64,
    started_at: Instant,
    plan: SourceBuiltFixedPointPlan,
}

#[derive(Debug, Clone, Serialize)]
struct AttemptStatus {
    schema: &'static str,
    status: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    plan_digest_blake3: Option<String>,
    blocker: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ImportedAttemptStatus {
    status: String,
    plan_digest_blake3: Option<String>,
}

#[derive(Debug, Deserialize)]
struct BuildJsonReport {
    schema: String,
    hermeticity_mode: String,
    hermeticity_audit_events: Vec<serde_json::Value>,
    #[serde(default)]
    scheduler_priority_decisions: Vec<BuildJsonPriorityDecision>,
    outcomes: Vec<BuildJsonOutcome>,
    failed: Vec<BuildJsonFailure>,
}

#[derive(Debug, Deserialize)]
struct BuildJsonPriorityDecision {
    selected_goal_key_blake3: String,
}

#[derive(Debug, Deserialize)]
struct BuildJsonOutcome {
    label: String,
    cached: bool,
    outputs: Vec<BuildJsonOutput>,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct BuildJsonOutput {
    pub(crate) name: String,
    pub(crate) path: PathBuf,
    pub(crate) artifact_attestation: BuildJsonAttestationReference,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct BuildJsonAttestationReference {
    pub(crate) logical_path: String,
    pub(crate) path: PathBuf,
}

#[derive(Debug, Deserialize)]
struct BuildJsonFailure {
    root: String,
    phase: String,
    error_class: String,
    message: String,
}

#[derive(Debug)]
pub(crate) struct BuildObservation {
    pub(crate) output: BuildJsonOutput,
    pub(crate) transcript_path: PathBuf,
    pub(crate) transcript_digest_blake3: String,
    pub(crate) action_trust: Option<NativeBuildActionTrustEvidence>,
}

#[derive(Debug)]
pub(crate) struct NativeBuildActionTrustEvidence {
    pub(crate) plan_path: PathBuf,
    pub(crate) reconciliation_path: PathBuf,
    pub(crate) plan: crate::source_built_derivation_action_plan::EagerDerivationActionPlan,
    pub(crate) reconciliation: crate::source_built_derivation_action_plan::EagerDerivationReconciliation,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct StagexTransitionHandoffReport {
    format: &'static str,
    copied_directories: Vec<&'static str>,
    non_claim: &'static str,
}

#[derive(Debug)]
pub(crate) struct NativeProviderPrefix {
    pub(crate) stagex_transition_execution_dir: PathBuf,
    pub(crate) stagex_provider_report: crate::stagex_provider::StagexProviderPublicationReport,
    pub(crate) native_provider: BuildObservation,
    pub(crate) native_action_trust: Option<NativeBuildActionTrustEvidence>,
    pub(crate) native_admission: crate::full_source_provider::FullSourceProviderAdmissionReport,
    pub(crate) native_admission_report_path: PathBuf,
    pub(crate) rust_host_tools: BTreeMap<String, BuildObservation>,
    pub(crate) rust_host_tool_evidence_dir: PathBuf,
    pub(crate) rust_host_tool_manifest_path: PathBuf,
}

#[derive(Debug)]
pub(crate) struct ConstructedProviders {
    pub(crate) stagex_transition_execution_dir: PathBuf,
    pub(crate) stagex_provider_report: crate::stagex_provider::StagexProviderPublicationReport,
    pub(crate) native_provider: BuildObservation,
    pub(crate) native_action_trust: Option<NativeBuildActionTrustEvidence>,
    pub(crate) native_admission: crate::full_source_provider::FullSourceProviderAdmissionReport,
    pub(crate) native_admission_report_path: PathBuf,
    pub(crate) rust_provider: crate::rust_source_provider::RustSourceProviderMaterialization,
    pub(crate) rust_host_tools: BTreeMap<String, BuildObservation>,
    pub(crate) rust_host_tool_evidence_dir: PathBuf,
    pub(crate) toolchain_closure_path: PathBuf,
    pub(crate) provider_checkpoint:
        Option<crate::source_built_fixed_point_checkpoint_shell::RestoredProviderCheckpoint>,
}

pub(crate) fn cmd_source_built_fixed_point(options: SourceBuiltFixedPointOptions<'_>) -> Result<(), RunError> {
    validate_options(&options)?;
    let started_at = Instant::now();
    let expected_staging_dir = staging_path(options.output_dir)?;
    let prepared = match prepare_attempt(&options, started_at) {
        Ok(prepared) => prepared,
        Err(error) => {
            let blocker = error.to_string();
            let preservation = if expected_staging_dir.is_dir() {
                write_attempt_status(&expected_staging_dir, None, PROOF_STATUS_FAILED, Some(&blocker))?;
                format!("preserved_attempt={}", expected_staging_dir.display())
            } else {
                "attempt_not_started=true".to_string()
            };
            return Err(RunError::Build(format!(
                "source-built fixed-point preparation failed closed: {blocker}; {preservation}"
            )));
        }
    };
    write_attempt_status(&prepared.staging_dir, Some(&prepared.plan.plan_digest_blake3), PROOF_STATUS_RUNNING, None)?;
    if let Some(import_attempt) = options.proof_checkpoint_import_attempt {
        if let Err(error) = import_provider_checkpoint_attempt(&options, &prepared, import_attempt) {
            let blocker = error.to_string();
            write_attempt_status(
                &prepared.staging_dir,
                Some(&prepared.plan.plan_digest_blake3),
                PROOF_STATUS_FAILED,
                Some(&blocker),
            )?;
            return Err(RunError::Build(format!(
                "source-built provider checkpoint import failed closed: {blocker}; preserved_attempt={}",
                prepared.staging_dir.display()
            )));
        }
        write_attempt_status(
            &prepared.staging_dir,
            Some(&prepared.plan.plan_digest_blake3),
            PROOF_STATUS_COMPLETE,
            None,
        )?;
        print_checkpoint_import_completion(&options, &prepared)?;
        return Ok(());
    }
    if options.dev_fast_fail
        && matches!(dev_fast_fail_check(&options, &prepared)?, FastFailDecision::ReportPriorSuccess { .. })
    {
        // Fast-fail: do not launch a fresh rebuild. Report the exact prior digest.
        let prior = last_published_source_digest(&options)?;
        write_attempt_status(
            &prepared.staging_dir,
            Some(&prepared.plan.plan_digest_blake3),
            PROOF_STATUS_COMPLETE,
            None,
        )?;
        emit_fast_fail_notice(&options, &prepared, prior.as_deref());
        return Ok(());
    }
    let adopted = match run_attempt(&options, &prepared) {
        Ok(adopted) => adopted,
        Err(error) => {
            let blocker = error.to_string();
            write_attempt_status(
                &prepared.staging_dir,
                Some(&prepared.plan.plan_digest_blake3),
                PROOF_STATUS_FAILED,
                Some(&blocker),
            )?;
            return Err(RunError::Build(format!(
                "source-built fixed-point proof failed closed: {blocker}; preserved_attempt={}",
                prepared.staging_dir.display()
            )));
        }
    };
    write_attempt_status(&prepared.staging_dir, Some(&prepared.plan.plan_digest_blake3), PROOF_STATUS_COMPLETE, None)?;
    if adopted {
        // Dev-only provider adoption: never publish to the success aliases and never
        // update `latest`/`latest-source-built-fixed-point`.
        print_dev_adopted_completion(&options, &prepared)?;
        return Ok(());
    }
    if let Err(error) = publish_attempt(&prepared) {
        let blocker = error.to_string();
        let status_root = if prepared.staging_dir.is_dir() {
            &prepared.staging_dir
        } else {
            &prepared.final_dir
        };
        write_attempt_status(
            status_root,
            Some(&prepared.plan.plan_digest_blake3),
            PROOF_STATUS_FAILED,
            Some(&blocker),
        )?;
        return Err(RunError::Build(format!(
            "source-built fixed-point publication failed closed: {blocker}; preserved_attempt={}",
            status_root.display()
        )));
    }
    print_completion(&options, &prepared)
}

fn validate_options(options: &SourceBuiltFixedPointOptions<'_>) -> Result<(), RunError> {
    for (label, path) in [
        ("source profile", options.source_profile),
        ("output", options.output_dir),
        ("bwrap", options.bwrap),
        ("sandbox shell", options.sandbox_shell),
    ] {
        if !path.is_absolute() {
            return Err(proof_error(format!("{label} path must be absolute: {}", path.display())));
        }
    }
    validate_expected_digest("source profile", options.expected_source_profile_blake3)?;
    validate_expected_digest("StageX lineage", options.expected_stagex_lineage_blake3)?;
    validate_expected_digest("native provider", options.expected_native_provider_blake3)?;
    if options.output_dir.exists() {
        return Err(proof_error(format!(
            "proof output must be absent before execution: {}",
            options.output_dir.display()
        )));
    }
    validate_executable("bwrap", options.bwrap)?;
    if options.bwrap.file_name().and_then(|name| name.to_str()) != Some("bwrap") {
        return Err(proof_error(format!(
            "proof bwrap executable must have basename bwrap: {}",
            options.bwrap.display()
        )));
    }
    validate_executable("sandbox shell", options.sandbox_shell)?;
    validate_static_executable("sandbox shell", options.sandbox_shell)?;
    if let Some(checkpoint_store) = options.proof_checkpoint_store {
        if !checkpoint_store.is_absolute() {
            return Err(proof_error(format!(
                "proof checkpoint store must be absolute: {}",
                checkpoint_store.display()
            )));
        }
        if options.dev_provider_cache.is_some() || options.dev_resume || options.dev_fast_fail {
            return Err(proof_error(
                "promoted proof checkpoints conflict with dev cache, resume, and fast-fail state".to_string(),
            ));
        }
        validate_checkpoint_store_filesystem(options.output_dir, checkpoint_store)?;
    }
    if options.dev_resume && options.dev_provider_cache.is_none() {
        return Err(proof_error("dev resume requires --dev-provider-cache".to_string()));
    }
    if let Some(import_attempt) = options.proof_checkpoint_import_attempt {
        if options.proof_checkpoint_store.is_none() {
            return Err(proof_error("checkpoint import requires a checkpoint store".to_string()));
        }
        if !import_attempt.is_absolute() || !import_attempt.is_dir() {
            return Err(proof_error(format!(
                "checkpoint import attempt must be an absolute directory: {}",
                import_attempt.display()
            )));
        }
    }
    if let Some(native_attempt) = options.proof_native_checkpoint_attempt {
        if options.proof_checkpoint_store.is_none() {
            return Err(proof_error("native checkpoint reuse requires a checkpoint store".to_string()));
        }
        if options.proof_checkpoint_import_attempt.is_some() {
            return Err(proof_error("native checkpoint reuse conflicts with full checkpoint import".to_string()));
        }
        if !native_attempt.is_absolute() || !native_attempt.is_dir() {
            return Err(proof_error(format!(
                "native checkpoint attempt must be an absolute directory: {}",
                native_attempt.display()
            )));
        }
    }
    if !(MAX_JOBS_MIN..=MAX_JOBS_MAX).contains(&options.jobs) {
        return Err(proof_error(format!("jobs must be within {MAX_JOBS_MIN}..={MAX_JOBS_MAX}, got {}", options.jobs)));
    }
    assert!(options.elapsed_seconds_max > 0);
    assert!(options.disk_bytes_max > 0);
    validate_disk_preflight(options.output_dir, options.disk_bytes_max)
}

fn validate_checkpoint_store_filesystem(output_dir: &Path, checkpoint_store: &Path) -> Result<(), RunError> {
    let output_parent = output_dir
        .parent()
        .ok_or_else(|| proof_error("proof output has no parent for checkpoint filesystem validation".to_string()))?;
    let checkpoint_parent = checkpoint_store
        .parent()
        .ok_or_else(|| proof_error("proof checkpoint store has no parent for filesystem validation".to_string()))?;
    let output_metadata = fs::metadata(output_parent).map_err(|error| {
        proof_error(format!("reading proof output filesystem {}: {error}", output_parent.display()))
    })?;
    let checkpoint_metadata = fs::metadata(checkpoint_parent).map_err(|error| {
        proof_error(format!("reading checkpoint filesystem {}: {error}", checkpoint_parent.display()))
    })?;
    if output_metadata.dev() != checkpoint_metadata.dev() {
        return Err(proof_error(
            "proof checkpoint store must share the proof output filesystem so disk accounting remains complete"
                .to_string(),
        ));
    }
    assert!(output_metadata.is_dir());
    assert!(checkpoint_metadata.is_dir());
    Ok(())
}

fn validate_disk_preflight(output_dir: &Path, disk_bytes_max: u64) -> Result<(), RunError> {
    let parent = output_dir
        .parent()
        .ok_or_else(|| proof_error(format!("proof output has no parent directory: {}", output_dir.display())))?;
    if !parent.is_dir() {
        return Err(proof_error(format!("proof output parent must exist before execution: {}", parent.display())));
    }
    let available_bytes = fs2::available_space(parent)
        .map_err(|error| proof_error(format!("probing proof output capacity {}: {error}", parent.display())))?;
    validate_disk_capacity(available_bytes, disk_bytes_max)
}

fn validate_disk_capacity(available_bytes: u64, required_bytes: u64) -> Result<(), RunError> {
    assert!(required_bytes > 0);
    if available_bytes < required_bytes {
        return Err(proof_error(format!(
            "proof disk preflight requires {required_bytes} bytes, only {available_bytes} bytes are available"
        )));
    }
    Ok(())
}

fn plan_open_file_descriptor_limit(
    current_soft_limit: u64,
    current_hard_limit: u64,
    required_limit: u64,
) -> Result<OpenFileDescriptorLimitPlan, String> {
    if required_limit == 0 {
        return Err("proof open-file descriptor limit must be nonzero".to_string());
    }
    if current_soft_limit > current_hard_limit {
        return Err(format!(
            "observed open-file descriptor soft limit {current_soft_limit} exceeds hard limit {current_hard_limit}"
        ));
    }
    if current_hard_limit < required_limit {
        return Err(format!(
            "proof requires open-file descriptor limit {required_limit}, but the hard limit is {current_hard_limit}"
        ));
    }
    let plan = OpenFileDescriptorLimitPlan {
        soft_limit: required_limit,
        hard_limit: current_hard_limit,
        update_required: current_soft_limit != required_limit,
    };
    assert!(plan.soft_limit > 0);
    assert!(plan.soft_limit <= plan.hard_limit);
    Ok(plan)
}

#[cfg(target_os = "linux")]
fn read_open_file_descriptor_limits() -> Result<(u64, u64), RunError> {
    let mut limits = libc::rlimit {
        rlim_cur: 0,
        rlim_max: 0,
    };
    // SAFETY: `limits` points to initialized writable storage for one `rlimit` value.
    let status = unsafe { libc::getrlimit(libc::RLIMIT_NOFILE, &mut limits) };
    if status != 0 {
        return Err(proof_error(format!("reading open-file descriptor limit: {}", std::io::Error::last_os_error())));
    }
    if limits.rlim_cur > limits.rlim_max {
        return Err(proof_error(format!(
            "observed open-file descriptor soft limit {} exceeds hard limit {}",
            limits.rlim_cur, limits.rlim_max
        )));
    }
    assert!(limits.rlim_max > 0);
    assert!(limits.rlim_cur <= limits.rlim_max);
    Ok((limits.rlim_cur, limits.rlim_max))
}

#[cfg(target_os = "linux")]
fn enforce_open_file_descriptor_limit(required_limit: u64) -> Result<(), RunError> {
    let (current_soft_limit, current_hard_limit) = read_open_file_descriptor_limits()?;
    let plan =
        plan_open_file_descriptor_limit(current_soft_limit, current_hard_limit, required_limit).map_err(proof_error)?;
    if plan.update_required {
        let limits = libc::rlimit {
            rlim_cur: plan.soft_limit,
            rlim_max: plan.hard_limit,
        };
        // SAFETY: `limits` is a valid immutable `rlimit` value, and the pure plan
        // proves that its soft limit is nonzero and no greater than its hard limit.
        let status = unsafe { libc::setrlimit(libc::RLIMIT_NOFILE, &limits) };
        if status != 0 {
            return Err(proof_error(format!(
                "setting open-file descriptor limit to {}: {}",
                plan.soft_limit,
                std::io::Error::last_os_error()
            )));
        }
    }
    let (observed_soft_limit, observed_hard_limit) = read_open_file_descriptor_limits()?;
    if observed_soft_limit != plan.soft_limit || observed_hard_limit != plan.hard_limit {
        return Err(proof_error(format!(
            "open-file descriptor limit verification failed: expected soft={} hard={}, observed soft={} hard={}",
            plan.soft_limit, plan.hard_limit, observed_soft_limit, observed_hard_limit
        )));
    }
    assert_eq!(observed_soft_limit, required_limit);
    assert!(observed_soft_limit <= observed_hard_limit);
    Ok(())
}

#[cfg(not(target_os = "linux"))]
fn enforce_open_file_descriptor_limit(_required_limit: u64) -> Result<(), RunError> {
    Err(proof_error("source-built fixed-point open-file descriptor enforcement requires Linux".to_string()))
}

fn hash_provider_recipe_projection(source_root: &Path) -> Result<(u64, String), RunError> {
    let mut total_bytes = 0_u64;
    let mut hasher = blake3::Hasher::new_derive_key(PROVIDER_RECIPE_PROJECTION_CONTEXT);
    for relative in PROVIDER_RECIPE_PROJECTION_ROOTS {
        let path = source_root.join(relative);
        let (bytes, digest) = crate::release_tree_copy::hash_directory_tree(&path)
            .map_err(|error| proof_error(format!("hashing provider recipe projection {relative}: {error}")))?;
        total_bytes = total_bytes
            .checked_add(bytes)
            .ok_or_else(|| proof_error("provider recipe projection byte count overflowed".to_string()))?;
        hasher.update(relative.as_bytes());
        hasher.update(b"\0");
        hasher.update(&bytes.to_le_bytes());
        hasher.update(digest.as_bytes());
        hasher.update(b"\n");
    }
    let digest = hasher.finalize().to_hex().to_string();
    assert!(total_bytes > 0);
    assert_eq!(digest.len(), BLAKE3_HEX_LENGTH);
    Ok((total_bytes, digest))
}

fn validate_materialized_vendor_inputs(source_root: &Path) -> Result<(), RunError> {
    crate::self_build::require_checked_vendor_inputs(source_root)
        .map_err(|error| proof_error(format!("validating materialized vendored Cargo inputs: {error}")))
}

fn prepare_attempt(
    options: &SourceBuiltFixedPointOptions<'_>,
    started_at: Instant,
) -> Result<PreparedAttempt, RunError> {
    let profile = crate::source_bundle::read_source_bundle(options.source_profile)?;
    if profile.format != PROFILE_SCHEMA {
        return Err(proof_error(format!("source profile schema must be {PROFILE_SCHEMA}")));
    }
    if profile.manifest_blake3 != options.expected_source_profile_blake3 {
        return Err(proof_error(format!(
            "source profile digest mismatch: expected {}, observed {}",
            options.expected_source_profile_blake3, profile.manifest_blake3
        )));
    }
    let records = source_built_fixed_point_profile_records(&profile)?;
    validate_profile_records(&records, options)?;
    let staging_dir = staging_path(options.output_dir)?;
    let disk_available_bytes_before =
        fs2::available_space(options.output_dir.parent().expect("validated absolute output has an existing parent"))
            .map_err(|error| proof_error(format!("recording proof disk baseline: {error}")))?;
    create_private_attempt_dir(&staging_dir)?;
    let inputs_dir = staging_dir.join(INPUTS_DIR);
    fs::create_dir(&inputs_dir)
        .map_err(|error| proof_error(format!("creating proof inputs {}: {error}", inputs_dir.display())))?;
    let source_root = inputs_dir.join(SOURCE_ROOT_DIR);
    materialize_source_record_payload(records.mantle_source, &source_root)?;
    let mantle_source_digest = hash_materialized_source(&source_root)?;
    let (provider_recipe_projection_bytes, provider_recipe_projection) = hash_provider_recipe_projection(&source_root)?;
    let vendor_root = source_root.join(VENDOR_RELATIVE_PATH);
    materialize_source_record_payload(records.vendor_inputs, &vendor_root)?;
    validate_materialized_vendor_inputs(&source_root)?;
    let vendor_inputs_digest = hash_materialized_source(&vendor_root)?;
    let stagex_seed_root = inputs_dir.join(STAGEX_SEED_DIR);
    materialize_source_record_payload(records.stagex_seed, &stagex_seed_root)?;
    let stagex_seed = sole_materialized_file(records.stagex_seed, &stagex_seed_root)?;
    let stagex_seed_digest = hash_materialized_source(&stagex_seed)?;
    let stagex_lineage_root = inputs_dir.join(STAGEX_LINEAGE_DIR);
    materialize_source_record_payload(records.stagex_lineage, &stagex_lineage_root)?;
    let stagex_lineage = sole_materialized_file(records.stagex_lineage, &stagex_lineage_root)?;
    let observed_stagex_lineage_blake3 = crate::protected_exec::blake3_file_hex(&stagex_lineage)
        .map_err(|error| proof_error(format!("hashing StageX lineage {}: {error}", stagex_lineage.display())))?;
    if observed_stagex_lineage_blake3 != options.expected_stagex_lineage_blake3 {
        return Err(proof_error(format!(
            "StageX lineage digest mismatch: expected {}, observed {}",
            options.expected_stagex_lineage_blake3, observed_stagex_lineage_blake3
        )));
    }
    let stagex_source_bundle_root = inputs_dir.join(STAGEX_SOURCE_BUNDLE_DIR);
    materialize_source_record_payload(records.stagex_source_bundle, &stagex_source_bundle_root)?;
    let stagex_source_bundle = sole_materialized_file(records.stagex_source_bundle, &stagex_source_bundle_root)?;
    let stagex_source_bundle_digest = hash_materialized_source(&stagex_source_bundle)?;
    let bound_stagex_manifest = crate::source_bundle::read_source_bundle(&stagex_source_bundle)
        .map_err(|error| proof_error(format!("validating StageX source bundle: {error}")))?;
    let rust_source_archive_dir = inputs_dir.join(RUST_SOURCE_DIR);
    materialize_source_record_payload(records.rust_source_archive_set, &rust_source_archive_dir)?;
    let rust_source_archive_set_digest = hash_materialized_source(&rust_source_archive_dir)?;
    let native_source_authority_root = inputs_dir.join(NATIVE_SOURCE_AUTHORITY_DIR);
    materialize_source_record_payload(records.native_source_manifest, &native_source_authority_root)?;
    let native_source_authority_manifest =
        sole_materialized_file(records.native_source_manifest, &native_source_authority_root)?;
    let bound_native_manifest = crate::source_bundle::read_source_bundle(&native_source_authority_manifest)?;
    validate_materialized_source_records(
        &bound_native_manifest,
        &bound_stagex_manifest,
        &records.native_source_records,
    )?;
    let native_source_manifest_digest = hash_materialized_source(&native_source_authority_manifest)?;
    let native_source_manifest = staging_dir.join(NATIVE_SOURCE_MANIFEST_FILE);
    let combined_native_manifest = assemble_source_bundle(
        records.native_source_records.iter().map(|record| (*record).clone()).collect(),
        LOGICAL_STORE_PREFIX,
    )?;
    crate::source_bundle::write_source_bundle(&native_source_manifest, &combined_native_manifest)?;
    let source_digests = MaterializedSourceDigests {
        stagex_seed: stagex_seed_digest,
        stagex_source_bundle: stagex_source_bundle_digest,
        native_source_manifest: native_source_manifest_digest,
        rust_source_archive_set: rust_source_archive_set_digest,
        provider_recipe_projection,
        provider_recipe_projection_bytes,
        mantle_source: mantle_source_digest,
        vendor_inputs: vendor_inputs_digest,
    };
    let plan = prepare_plan(options, &records, &source_digests, &native_source_authority_manifest)?;
    let plan_path = staging_dir.join(PLAN_FILE);
    write_json_create_new(&plan_path, &plan)?;
    let (native_store_dir, native_state_dir) = match options.dev_provider_cache {
        // Dev runs share a persistent content-addressed store so each
        // successfully built package is a no-op hit on the next dev run.
        Some(cache) => {
            let store = cache.join("dev-store");
            let state = cache.join("dev-state");
            fs::create_dir_all(&store)
                .map_err(|error| proof_error(format!("creating dev store {}: {error}", store.display())))?;
            fs::create_dir_all(&state)
                .map_err(|error| proof_error(format!("creating dev state {}: {error}", state.display())))?;
            (store, state)
        }
        // Promoted/cold runs use a fresh empty store per attempt.
        None => {
            let store = staging_dir.join(NATIVE_STORE_DIR);
            let state = staging_dir.join(NATIVE_STATE_DIR);
            fs::create_dir(&store)
                .map_err(|error| proof_error(format!("creating native store {}: {error}", store.display())))?;
            fs::create_dir(&state)
                .map_err(|error| proof_error(format!("creating native state {}: {error}", state.display())))?;
            (store, state)
        }
    };
    let transcripts_dir = staging_dir.join(TRANSCRIPTS_DIR);
    fs::create_dir(&transcripts_dir)
        .map_err(|error| proof_error(format!("creating transcripts {}: {error}", transcripts_dir.display())))?;
    crate::source_bundle::import_source_bundle(&profile, &native_state_dir, true)?;
    if options.dev_provider_cache.is_some() {
        seed_dev_store_snapshot(options, &plan, &native_store_dir, &native_state_dir)?;
    }
    Ok(PreparedAttempt {
        final_dir: options.output_dir.to_path_buf(),
        staging_dir,
        source_root,
        stagex_seed,
        stagex_lineage,
        stagex_source_bundle,
        native_source_manifest,
        rust_source_archive_dir,
        native_store_dir,
        native_state_dir,
        transcripts_dir,
        disk_available_bytes_before,
        started_at,
        plan,
    })
}

fn validate_profile_records(
    records: &SourceBuiltFixedPointProfileRecords<'_>,
    options: &SourceBuiltFixedPointOptions<'_>,
) -> Result<(), RunError> {
    if records.native_source_records.len() < SOURCE_RECORD_COUNT_MIN {
        return Err(proof_error("native source record set is empty".to_string()));
    }
    let native_count = u32::try_from(records.native_source_records.len())
        .map_err(|_| proof_error("native source record count exceeds u32".to_string()))?;
    if native_count > options.source_records_max {
        return Err(proof_error(format!(
            "native source record count {native_count} exceeds configured bound {}",
            options.source_records_max
        )));
    }
    if records.stagex_seed.files.is_empty() || records.rust_source_archive_set.files.is_empty() {
        return Err(proof_error("source profile has an empty required source authority".to_string()));
    }
    Ok(())
}

fn validate_materialized_source_records(
    native_manifest: &crate::source_bundle::SourceBundleManifest,
    stagex_manifest: &crate::source_bundle::SourceBundleManifest,
    profile_records: &[&SourceRecord],
) -> Result<(), RunError> {
    for (label, manifest) in [("native", native_manifest), ("StageX", stagex_manifest)] {
        if manifest.store_prefix != LOGICAL_STORE_PREFIX {
            return Err(proof_error(format!(
                "{label} source manifest store prefix must be {LOGICAL_STORE_PREFIX}, got {}",
                manifest.store_prefix
            )));
        }
    }
    let mut expected_by_identity = BTreeMap::new();
    for record in native_manifest
        .records
        .iter()
        .chain(&stagex_manifest.records)
        .filter(|record| crate::source_bundle::source_record_is_fetcher_input(record))
    {
        if let Some(existing) = expected_by_identity.get(record.identity.as_str()) {
            if *existing != record {
                return Err(proof_error(format!(
                    "native and StageX manifests conflict for source identity {}",
                    record.identity
                )));
            }
            continue;
        }
        expected_by_identity.insert(record.identity.as_str(), record);
    }
    let mut profile_by_identity = BTreeMap::new();
    for record in profile_records {
        if !crate::source_bundle::source_record_is_fetcher_input(record) {
            return Err(proof_error(format!(
                "materialized source profile contains non-fetch source authority {}",
                record.identity
            )));
        }
        if profile_by_identity.insert(record.identity.as_str(), *record).is_some() {
            return Err(proof_error(format!(
                "materialized source profile repeats source identity {}",
                record.identity
            )));
        }
    }
    let expected_record_count = expected_by_identity.len();
    for (identity, expected_record) in expected_by_identity {
        let Some(profile_record) = profile_by_identity.get(identity) else {
            return Err(proof_error(format!("materialized source profile omits bound source identity {identity}")));
        };
        if expected_record != *profile_record {
            return Err(proof_error(format!(
                "source identity {identity} differs from its independently bound manifest"
            )));
        }
    }
    assert!(!profile_records.is_empty());
    assert!(profile_by_identity.len() >= expected_record_count);
    debug_assert_eq!(native_manifest.store_prefix, stagex_manifest.store_prefix);
    Ok(())
}

fn prepare_plan(
    options: &SourceBuiltFixedPointOptions<'_>,
    records: &SourceBuiltFixedPointProfileRecords<'_>,
    source_digests: &MaterializedSourceDigests,
    native_source_manifest_path: &Path,
) -> Result<SourceBuiltFixedPointPlan, RunError> {
    let native_manifest_size = file_size(native_source_manifest_path)?;
    let source_inputs = vec![
        source_input_from_record(
            "stagex-seed",
            SourceAuthorityRole::StagexSeed,
            records.stagex_seed,
            &source_digests.stagex_seed,
        ),
        SourceAuthorityInput {
            id: "stagex-lineage".to_string(),
            role: SourceAuthorityRole::StagexLineage,
            kind: SourceContentKind::RegularFile,
            digest_blake3: options.expected_stagex_lineage_blake3.to_string(),
            size_bytes: records.stagex_lineage.payload_bytes,
        },
        source_input_from_record(
            "stagex-source-bundle",
            SourceAuthorityRole::StagexSourceBundle,
            records.stagex_source_bundle,
            &source_digests.stagex_source_bundle,
        ),
        SourceAuthorityInput {
            id: "native-source-bundle".to_string(),
            role: SourceAuthorityRole::NativeSourceBundle,
            kind: SourceContentKind::RegularFile,
            digest_blake3: source_digests.native_source_manifest.clone(),
            size_bytes: native_manifest_size,
        },
        source_input_from_record(
            "rust-source-archive-set",
            SourceAuthorityRole::RustSourceArchiveSet,
            records.rust_source_archive_set,
            &source_digests.rust_source_archive_set,
        ),
        SourceAuthorityInput {
            id: "provider-recipe-projection".to_string(),
            role: SourceAuthorityRole::ProviderRecipeProjection,
            kind: SourceContentKind::Directory,
            digest_blake3: source_digests.provider_recipe_projection.clone(),
            size_bytes: source_digests.provider_recipe_projection_bytes,
        },
        source_input_from_record(
            "mantle-source",
            SourceAuthorityRole::MantleSource,
            records.mantle_source,
            &source_digests.mantle_source,
        ),
        source_input_from_record(
            "vendor-inputs",
            SourceAuthorityRole::VendorInputs,
            records.vendor_inputs,
            &source_digests.vendor_inputs,
        ),
    ];
    let plan_input = SourceBuiltFixedPointPlanInput {
        proof_id: format!(
            "source-built-fixed-point-{}",
            &options.expected_source_profile_blake3[..BLAKE3_HEX_LENGTH / 4]
        ),
        logical_store_prefix: LOGICAL_STORE_PREFIX.to_string(),
        source_inputs,
        initial_output_authority: InitialOutputAuthorityState {
            stagex_transition_entries: 0,
            native_provider_entries: 0,
            rust_provider_entries: 0,
            mantle_output_entries: 0,
        },
        policies: SourceBuiltFixedPointPolicies {
            expected_native_provider_digest_blake3: options.expected_native_provider_blake3.to_string(),
            closure_policy_digest_blake3: policy_digest(CLOSURE_POLICY_TEXT),
            hermeticity_policy_digest_blake3: policy_digest(HERMETICITY_POLICY_TEXT),
            protected_execution_policy_digest_blake3: options.expected_stagex_lineage_blake3.to_string(),
            effect_policy_digest_blake3: policy_digest(EFFECT_POLICY_TEXT),
            normalization_policy_digest_blake3: policy_digest(NORMALIZATION_POLICY_TEXT),
            hermeticity_mode: ProofHermeticityMode::Strict,
            live_fetch_allowed: false,
            cargo_invocation_allowed: false,
            ambient_discovery_allowed: false,
            fallback_allowed: false,
            provider_cache_completion_allowed: false,
        },
        resource_bounds: SourceBuiltFixedPointResourceBounds {
            elapsed_seconds_max: options.elapsed_seconds_max,
            disk_bytes_max: options.disk_bytes_max,
            open_file_descriptors_max: SOURCE_BUILT_FIXED_POINT_OPEN_FILE_DESCRIPTORS_MAX,
            protected_exec_events_max: options.protected_exec_events_max,
            source_records_max: options.source_records_max,
        },
    };
    let plan = plan_source_built_fixed_point(plan_input)
        .map_err(|error| proof_error(format!("preparing source-built fixed-point plan: {error}")))?;
    assert_eq!(plan.stages.len(), EXPECTED_STAGE_COUNT);
    debug_assert_eq!(plan.logical_store_prefix, LOGICAL_STORE_PREFIX);
    Ok(plan)
}

fn run_attempt(options: &SourceBuiltFixedPointOptions<'_>, prepared: &PreparedAttempt) -> Result<bool, RunError> {
    enforce_open_file_descriptor_limit(prepared.plan.resource_bounds.open_file_descriptors_max)?;
    validate_runtime_bounds(options, prepared)?;
    let dev_resume::PreparedDevResume {
        plan: resume_plan,
        provider_resume,
        fixed_point_resume,
    } = dev_resume::prepare_dev_resume(options, prepared)?;
    let restored_checkpoint = if provider_resume.is_none() {
        restore_constructed_provider_checkpoint(options, prepared)?
    } else {
        None
    };
    let restored_native_prefix = if restored_checkpoint.is_none() && provider_resume.is_none() {
        options
            .proof_native_checkpoint_attempt
            .map(|attempt| restore_native_provider_prefix_attempt(options, prepared, attempt))
            .transpose()?
    } else {
        None
    };
    let adopt = if restored_checkpoint.is_none()
        && restored_native_prefix.is_none()
        && provider_resume.is_none()
        && !options.dev_resume
    {
        dev_cache_adoption(options, &prepared.plan)?
    } else {
        false
    };
    let is_dev = options.dev_provider_cache.is_some();
    if adopt {
        record_dev_cache_hit(prepared)?;
    }
    let providers = select_or_construct_providers(
        options,
        prepared,
        is_dev,
        provider_resume,
        restored_checkpoint,
        restored_native_prefix,
        adopt,
    )?;
    validate_runtime_bounds(options, prepared)?;
    if options.proof_checkpoint_store.is_some() && providers.provider_checkpoint.is_none() {
        publish_constructed_provider_checkpoint(options, prepared, &providers)?;
        validate_runtime_bounds(options, prepared)?;
    }
    run_cargo_free_fixed_point(options, prepared, &providers, &fixed_point_resume, &resume_plan)?;
    validate_runtime_bounds(options, prepared)?;
    if is_dev {
        finish_dev_attempt(options, prepared, &providers, &resume_plan, adopt)?;
        Ok(true)
    } else {
        finish_promoted_attempt(options, prepared, &providers)?;
        Ok(false)
    }
}

fn select_or_construct_providers(
    options: &SourceBuiltFixedPointOptions<'_>,
    prepared: &PreparedAttempt,
    is_dev: bool,
    provider_resume: Option<checkpoint_integration::DevProviderResume>,
    restored_checkpoint: Option<ConstructedProviders>,
    restored_native_prefix: Option<NativeProviderPrefix>,
    adopt: bool,
) -> Result<ConstructedProviders, RunError> {
    match provider_resume {
        Some(checkpoint_integration::DevProviderResume::Complete(providers)) => {
            admit_stagex_transition_handoff(prepared, is_dev, &providers.stagex_transition_execution_dir)?;
            Ok(*providers)
        }
        Some(checkpoint_integration::DevProviderResume::Native(prefix)) => {
            admit_stagex_transition_handoff(prepared, is_dev, &prefix.stagex_transition_execution_dir)?;
            construct_rust_provider_from_native_prefix(options, prepared, *prefix)
        }
        Some(checkpoint_integration::DevProviderResume::Stagex {
            execution_dir,
            provider_report,
        }) => {
            admit_stagex_transition_handoff(prepared, is_dev, &execution_dir)?;
            admit_restored_stagex_prefix(prepared, &execution_dir, &provider_report)?;
            construct_full_source_providers(options, prepared, execution_dir, provider_report, None)
        }
        Some(checkpoint_integration::DevProviderResume::Transition { execution_dir }) => {
            construct_providers_after_transition(options, prepared, is_dev, Some(execution_dir))
        }
        None => {
            if let Some(providers) = restored_checkpoint {
                return Ok(providers);
            }
            if let Some(prefix) = restored_native_prefix {
                return construct_rust_provider_from_native_prefix(options, prepared, prefix);
            }
            if adopt {
                let adopted = adopt_cached_provider_subtrees(options, prepared, &prepared.plan)?;
                return construct_full_source_providers(
                    options,
                    prepared,
                    prepared.staging_dir.join(STAGEX_TRANSITION_EXECUTION_DIR),
                    synthesized_adopted_stagex_report(prepared),
                    Some(adopted),
                );
            }
            construct_providers_after_transition(options, prepared, is_dev, None)
        }
    }
}

fn construct_providers_after_transition(
    options: &SourceBuiltFixedPointOptions<'_>,
    prepared: &PreparedAttempt,
    is_dev: bool,
    restored_transition: Option<PathBuf>,
) -> Result<ConstructedProviders, RunError> {
    let transition = prepare_stagex_transition(options, prepared, restored_transition)?;
    admit_stagex_transition_handoff(prepared, is_dev, &transition)?;
    let stagex_provider = build_stagex_provider(options, prepared, is_dev, &transition)?;
    construct_full_source_providers(options, prepared, transition, stagex_provider, None)
}

fn prepare_stagex_transition(
    options: &SourceBuiltFixedPointOptions<'_>,
    prepared: &PreparedAttempt,
    restored: Option<PathBuf>,
) -> Result<PathBuf, RunError> {
    if let Some(restored) = restored {
        if !restored.is_dir() || restored != prepared.staging_dir.join(STAGEX_TRANSITION_EXECUTION_DIR) {
            return Err(proof_error("restored StageX transition path is invalid".to_string()));
        }
        return Ok(restored);
    }
    let execution_dir = prepared.staging_dir.join(STAGEX_TRANSITION_EXECUTION_DIR);
    let same_attempt_resume = options.dev_resume
        && execution_dir.is_dir()
        && transition_marker_is_trusted(prepared, &transition_input_replay_digest(prepared)?)?;
    if same_attempt_resume {
        return Ok(execution_dir);
    }
    let result = run_in_isolated_exec_thread("StageX transition", || {
        crate::stagex_transition::materialize_protected_transition(StagexTransitionRequest {
            seed_path: &prepared.stagex_seed,
            hex0_source_path: &prepared.source_root.join("bootstrap/seeds/AMD64/hex0_AMD64.hex0"),
            kaem_source_path: &prepared.source_root.join("bootstrap/seeds/AMD64/kaem-minimal.hex0"),
            lineage_manifest_path: &prepared.stagex_lineage,
            source_bundle_path: Some(&prepared.stagex_source_bundle),
            stage0_answers_path: Some(&prepared.source_root.join("bootstrap/stage0-amd64.answers")),
            scratch_dir: &execution_dir,
        })
    })?;
    let report = result.map_err(|error| proof_error(format!("StageX transition failed: {error}")))?;
    if report.status != PROOF_STATUS_COMPLETE {
        return Err(proof_error(format!("StageX transition status must be complete, got {}", report.status)));
    }
    let event_count = u32::try_from(report.protected_exec_events.len())
        .map_err(|_| proof_error("StageX protected-exec event count exceeds u32".to_string()))?;
    if event_count > options.protected_exec_events_max {
        return Err(proof_error("StageX protected-exec event count exceeds the configured bound".to_string()));
    }
    if options.dev_provider_cache.is_some() || options.dev_resume {
        let replay_digest = transition_input_replay_digest(prepared)?;
        write_stage_marker(prepared, "stagex-transition", &replay_digest)?;
    }
    checkpoint_integration::publish_dev_provider_boundary(
        options,
        prepared,
        checkpoint_integration::ProviderPrefix::Transition(&execution_dir),
    )?;
    debug_assert!(execution_dir.is_dir());
    debug_assert!(event_count <= options.protected_exec_events_max);
    Ok(execution_dir)
}

fn admit_stagex_transition_handoff(
    prepared: &PreparedAttempt,
    is_dev: bool,
    execution_dir: &Path,
) -> Result<(), RunError> {
    let transition_root = prepared.native_store_dir.join(STAGEX_TRANSITION_STORE_BASENAME);
    let replay_root = prepared.staging_dir.join(STAGEX_TRANSITION_HANDOFF_REPLAY_DIR);
    materialize_or_validate_stagex_transition_handoff(is_dev, execution_dir, &transition_root, &replay_root)?;
    let logical = crate::full_source_provider::adopt_verified_local_provider_path_strict(
        &transition_root,
        &prepared.native_store_dir,
        &prepared.native_state_dir,
        LOGICAL_STORE_PREFIX,
    )?;
    if logical != STAGEX_TRANSITION_LOGICAL_PATH {
        return Err(proof_error("restored StageX transition logical path mismatch".to_string()));
    }
    crate::source_bundle::import_constructed_store_path_source(
        &logical,
        &transition_root,
        &prepared.native_state_dir,
        LOGICAL_STORE_PREFIX,
    )?;
    debug_assert!(transition_root.is_dir());
    Ok(())
}

fn build_stagex_provider(
    options: &SourceBuiltFixedPointOptions<'_>,
    prepared: &PreparedAttempt,
    is_dev: bool,
    transition: &Path,
) -> Result<crate::stagex_provider::StagexProviderPublicationReport, RunError> {
    let provider_root = prepared.native_store_dir.join(STAGEX_PROVIDER_STORE_BASENAME);
    let is_reuse = is_dev && provider_root.exists();
    let replay_root = prepared.staging_dir.join(STAGEX_PROVIDER_REPLAY_DIR);
    let output_root = if is_reuse { &replay_root } else { &provider_root };
    let result = run_in_isolated_exec_thread("StageX provider publication", || {
        crate::stagex_provider::materialize_stagex_provider(StagexProviderRequest {
            lineage_manifest_path: &prepared.stagex_lineage,
            transition_root: transition,
            output_path: output_root,
        })
    })?;
    let report = result.map_err(|error| proof_error(format!("StageX provider publication failed: {error}")))?;
    validate_stagex_provider_normalized_identity(&report.normalized_provider_digest_blake3)?;
    if is_reuse {
        let persistent = crate::stagex_provider::observe_normalized_provider_payload_digest(&provider_root)
            .map_err(|error| proof_error(format!("observing persistent StageX provider payload: {error}")))?;
        validate_reusable_stagex_provider_identity(&report.normalized_provider_digest_blake3, &persistent)?;
    }
    validate_runtime_bounds(options, prepared)?;
    admit_restored_stagex_prefix(prepared, transition, &report)?;
    checkpoint_integration::publish_dev_provider_boundary(
        options,
        prepared,
        checkpoint_integration::ProviderPrefix::Stagex(transition, &report),
    )?;
    debug_assert!(provider_root.is_dir());
    Ok(report)
}

fn admit_restored_stagex_prefix(
    prepared: &PreparedAttempt,
    transition: &Path,
    report: &crate::stagex_provider::StagexProviderPublicationReport,
) -> Result<(), RunError> {
    if !transition.is_dir() || !report.output_path.is_dir() {
        return Err(proof_error("restored StageX prefix is incomplete".to_string()));
    }
    let provider_root = prepared.native_store_dir.join(STAGEX_PROVIDER_STORE_BASENAME);
    // When the StageX provider is replayed into the attempt staging tree its
    // basename ("stagex-provider-replay") is not a valid store path name.
    // Admission must target the persistent store copy under the pinned store
    // basename; the replay's normalized identity check has already bound the
    // fresh run to that store payload.
    let admitted_root = if report.output_path != provider_root && provider_root.is_dir() {
        &provider_root
    } else {
        &report.output_path
    };
    let logical = crate::full_source_provider::adopt_verified_local_provider_path_strict(
        admitted_root,
        &prepared.native_store_dir,
        &prepared.native_state_dir,
        LOGICAL_STORE_PREFIX,
    )?;
    if logical != STAGEX_PROVIDER_LOGICAL_PATH {
        return Err(proof_error("restored StageX provider logical path mismatch".to_string()));
    }
    crate::source_bundle::import_constructed_store_path_source(
        &logical,
        admitted_root,
        &prepared.native_state_dir,
        LOGICAL_STORE_PREFIX,
    )?;
    debug_assert!(transition.is_dir());
    debug_assert!(admitted_root.is_dir());
    Ok(())
}

fn finish_dev_attempt(
    options: &SourceBuiltFixedPointOptions<'_>,
    prepared: &PreparedAttempt,
    providers: &ConstructedProviders,
    resume_plan: &crunch_dev_resume_core::ResumePlan,
    adopted: bool,
) -> Result<(), RunError> {
    let published_bundle_identities = dev_resume::published_bundle_identities(prepared)?;
    if !adopted && providers.provider_checkpoint.is_none() {
        write_dev_provider_cache(options, prepared, providers)?;
    }
    let disposition = if resume_plan.disposition == crunch_dev_resume_core::ResumeDisposition::Restore {
        "resume-restored"
    } else if adopted {
        "provider-cache-adopted"
    } else {
        "cold-executed"
    };
    let adoption_plan = adopted.then(|| provider_adoption_report_plan(resume_plan));
    let report_plan = adoption_plan.as_ref().unwrap_or(resume_plan);
    dev_resume::write_dev_resume_report(prepared, report_plan, &published_bundle_identities, disposition)?;
    write_dev_adopted_marker(prepared)
}

fn provider_adoption_report_plan(cold_plan: &crunch_dev_resume_core::ResumePlan) -> crunch_dev_resume_core::ResumePlan {
    let executed_stages = vec![
        crunch_dev_resume_core::ResumeStage::FullSourceRustProvider,
        crunch_dev_resume_core::ResumeStage::MantleStage1,
        crunch_dev_resume_core::ResumeStage::MantleStage2,
    ];
    let mut plan = cold_plan.clone();
    plan.restored_stages.clear();
    plan.executed_stages = executed_stages;
    plan.first_incomplete_stage = plan.executed_stages.first().copied();
    plan.selected_bundle_identity_blake3 = None;
    plan.completed_stage = None;
    debug_assert!(plan.restored_stages.is_empty());
    debug_assert_eq!(plan.first_incomplete_stage, Some(crunch_dev_resume_core::ResumeStage::FullSourceRustProvider));
    plan
}

fn finish_promoted_attempt(
    options: &SourceBuiltFixedPointOptions<'_>,
    prepared: &PreparedAttempt,
    providers: &ConstructedProviders,
) -> Result<(), RunError> {
    let root_action_trust = crate::source_built_root_action_trust::write_root_action_trust(
        &prepared.staging_dir,
        &prepared.staging_dir.join(FIXED_POINT_DIR),
        &prepared.plan,
        providers,
    )?;
    debug_assert!(root_action_trust.plan_path.is_file());
    debug_assert!(root_action_trust.reconciliation_path.is_file());
    crate::source_built_fixed_point_receipt::write_source_built_fixed_point_receipt(
        &prepared.staging_dir,
        &prepared.plan,
        providers,
    )?;
    validate_runtime_bounds(options, prepared)
}

fn write_dev_adopted_marker(prepared: &PreparedAttempt) -> Result<(), RunError> {
    let report = serde_json::json!({
        "schema": PROOF_STATUS_SCHEMA,
        "status": "dev-cache-adopted",
        "plan_digest_blake3": prepared.plan.plan_digest_blake3,
        "output": prepared.final_dir,
        "notice": "dev-only: cached providers adopted; no promoted receipt, no release alias",
    });
    let bytes = serde_json::to_vec_pretty(&report)
        .map_err(|error| proof_error(format!("serializing dev-adopted report: {error}")))?;
    fs::write(prepared.staging_dir.join("dev-adopted-report.json"), bytes)
        .map_err(|error| proof_error(format!("writing dev-adopted report: {error}")))?;
    debug_assert!(prepared.staging_dir.join("dev-adopted-report.json").is_file());
    Ok(())
}

/// After a successful cold dev run, publish the StageX and native provider store
/// subtrees plus a receipt-validated entry into the provider cache so a later dev
/// run can adopt them. Only ever called on the cold (non-adopted) path.
fn write_dev_provider_cache(
    options: &SourceBuiltFixedPointOptions<'_>,
    prepared: &PreparedAttempt,
    providers: &ConstructedProviders,
) -> Result<(), RunError> {
    let cache = options
        .dev_provider_cache
        .ok_or_else(|| proof_error("dev provider-cache flag is unset while publishing cache".to_string()))?;
    let plan = &prepared.plan;
    let policies = DevCachePolicies::from_plan(plan);
    let cache_key = dev_provider_cache_key(&plan.source_authority_digest_blake3, &policies);
    let entry_root = cache.join(DEV_CACHE_PROVIDERS_SUBDIR).join(&plan.plan_digest_blake3).join(&cache_key);
    let stagex_basename = STAGEX_PROVIDER_STORE_BASENAME;
    let stagex_src = prepared.native_store_dir.join(stagex_basename);
    if !stagex_src.is_dir() {
        return Err(proof_error(format!("cold StageX provider store missing: {}", stagex_src.display())));
    }
    let native_basename = providers
        .native_provider
        .output
        .path
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| proof_error("native provider output has no UTF-8 basename".to_string()))?;
    let native_src = &providers.native_provider.output.path;
    if !native_src.is_dir() {
        return Err(proof_error(format!("cold native provider store missing: {}", native_src.display())));
    }
    let entry = crate::source_built_fixed_point_dev_cache::DevProviderCacheEntry {
        schema: crate::source_built_fixed_point_dev_cache::DEV_CACHE_SCHEMA.to_string(),
        plan_digest_blake3: plan.plan_digest_blake3.clone(),
        cache_key: cache_key.clone(),
        source_authority_digest_blake3: plan.source_authority_digest_blake3.clone(),
        closure_policy_digest_blake3: policies.closure_policy_digest_blake3,
        hermeticity_policy_digest_blake3: policies.hermeticity_policy_digest_blake3,
        protected_execution_policy_digest_blake3: policies.protected_execution_policy_digest_blake3,
        effect_policy_digest_blake3: policies.effect_policy_digest_blake3,
        normalization_policy_digest_blake3: policies.normalization_policy_digest_blake3,
        stagex_provider: crate::source_built_fixed_point_dev_cache::DevProviderReference {
            logical_path: STAGEX_PROVIDER_LOGICAL_PATH.to_string(),
            store_basename: stagex_basename.to_string(),
            digest_blake3: STAGEX_PROVIDER_EXPECTED_NORMALIZED_DIGEST.to_string(),
        },
        native_provider: crate::source_built_fixed_point_dev_cache::DevProviderReference {
            logical_path: providers.native_provider.output.artifact_attestation.logical_path.clone(),
            store_basename: native_basename.to_string(),
            digest_blake3: providers.native_admission.output_digest_blake3.clone(),
        },
    };
    if entry_root.exists() {
        return validate_existing_dev_provider_cache(&entry_root, &entry);
    }
    fs::create_dir_all(&entry_root).map_err(|error| {
        proof_error(format!("creating dev provider-cache entry root {}: {error}", entry_root.display()))
    })?;
    crate::stagex_mes_lib::copy_tree_bounded(&stagex_src, &entry_root.join(stagex_basename))
        .map_err(|error| proof_error(format!("publishing cached StageX provider: {error}")))?;
    crate::stagex_mes_lib::copy_tree_bounded(native_src, &entry_root.join(native_basename))
        .map_err(|error| proof_error(format!("publishing cached native provider: {error}")))?;
    write_json_create_new(&entry_root.join(DEV_CACHE_ENTRY_FILE), &entry)?;
    debug_assert!(entry_root.join(DEV_CACHE_ENTRY_FILE).is_file());
    debug_assert!(entry_root.join(stagex_basename).is_dir());
    debug_assert!(entry_root.join(native_basename).is_dir());
    Ok(())
}

fn validate_existing_dev_provider_cache(entry_root: &Path, expected: &DevProviderCacheEntry) -> Result<(), RunError> {
    let path = entry_root.join(DEV_CACHE_ENTRY_FILE);
    let bytes = fs::read(&path).map_err(|error| {
        proof_error(format!("reading existing dev provider-cache entry {}: {error}", path.display()))
    })?;
    let observed: DevProviderCacheEntry = serde_json::from_slice(&bytes).map_err(|error| {
        proof_error(format!("parsing existing dev provider-cache entry {}: {error}", path.display()))
    })?;
    if observed != *expected {
        return Err(proof_error("existing dev provider-cache entry conflicts with current providers".to_string()));
    }
    if !entry_root.join(&observed.stagex_provider.store_basename).is_dir()
        || !entry_root.join(&observed.native_provider.store_basename).is_dir()
    {
        return Err(proof_error("existing dev provider-cache payload is incomplete".to_string()));
    }
    debug_assert_eq!(observed.cache_key, expected.cache_key);
    debug_assert_eq!(observed.plan_digest_blake3, expected.plan_digest_blake3);
    Ok(())
}

fn last_published_source_digest(options: &SourceBuiltFixedPointOptions<'_>) -> Result<Option<String>, RunError> {
    let parent = options.output_dir.parent().ok_or_else(|| proof_error("proof output has no parent".to_string()))?;
    let latest = parent.join(LATEST_SOURCE_BUILT_ALIAS);
    if !latest.is_symlink() {
        return Ok(None);
    }
    let target = fs::read_link(&latest)
        .map_err(|error| proof_error(format!("reading latest fixed-point alias {}: {error}", latest.display())))?;
    let receipt_path = parent.join(target).join(FINAL_RECEIPT_FILE);
    if !receipt_path.is_file() {
        return Ok(None);
    }
    let bytes = fs::read(&receipt_path).map_err(|error| {
        proof_error(format!("reading latest fixed-point receipt {}: {error}", receipt_path.display()))
    })?;
    let value: serde_json::Value = serde_json::from_slice(&bytes).map_err(|error| {
        proof_error(format!("parsing latest fixed-point receipt {}: {error}", receipt_path.display()))
    })?;
    Ok(value
        .get("source_built_fixed_point")
        .and_then(|extension| extension.get("source_authority_digest_blake3"))
        .and_then(serde_json::Value::as_str)
        .map(str::to_string))
}

fn dev_fast_fail_check(
    options: &SourceBuiltFixedPointOptions<'_>,
    prepared: &PreparedAttempt,
) -> Result<FastFailDecision, RunError> {
    let last = last_published_source_digest(options)?;
    Ok(evaluate_fast_fail(&prepared.plan.source_authority_digest_blake3, last.as_deref()))
}

fn emit_fast_fail_notice(
    options: &SourceBuiltFixedPointOptions<'_>,
    prepared: &PreparedAttempt,
    prior_digest: Option<&str>,
) {
    let digest = prior_digest.unwrap_or("unknown-prior-digest");
    if options.json {
        let value = serde_json::json!({
            "schema": FAST_FAIL_SCHEMA,
            "status": "fast-fail",
            "matched_source_authority_digest_blake3": digest,
            "plan_digest_blake3": prepared.plan.plan_digest_blake3,
        });
        println!("{}", serde_json::to_string_pretty(&value).unwrap_or_else(|_| "{}".to_string()));
    } else {
        eprintln!("{FAST_FAIL_NOTICE}");
        eprintln!("  matched_source_authority_digest_blake3: {digest}");
    }
}

fn seed_dev_store_snapshot(
    options: &SourceBuiltFixedPointOptions<'_>,
    plan: &SourceBuiltFixedPointPlan,
    store_dir: &Path,
    state_dir: &Path,
) -> Result<bool, RunError> {
    let Some(cache) = options.dev_provider_cache else {
        return Ok(false);
    };
    let policies = DevCachePolicies::from_plan(plan);
    let entry = match read_provider_cache_entry(options, plan, &policies)? {
        Some(entry) => entry,
        None => return Ok(false),
    };
    if evaluate_provider_cache_lookup(Some(&entry), plan, &policies) != DevProviderCacheLookup::Hit {
        return Ok(false);
    }
    let entry_root = cache.join(DEV_CACHE_PROVIDERS_SUBDIR).join(&plan.plan_digest_blake3).join(&entry.cache_key);
    let stagex_dst = store_dir.join(&entry.stagex_provider.store_basename);
    if !stagex_dst.exists() {
        crate::stagex_mes_lib::copy_tree_bounded(&entry_root.join(&entry.stagex_provider.store_basename), &stagex_dst)
            .map_err(|error| proof_error(format!("seeding cached StageX provider: {error}")))?;
    }
    register_adopted_provider(&stagex_dst, store_dir, state_dir)?;
    let native_dst = store_dir.join(&entry.native_provider.store_basename);
    if !native_dst.exists() {
        crate::stagex_mes_lib::copy_tree_bounded(&entry_root.join(&entry.native_provider.store_basename), &native_dst)
            .map_err(|error| proof_error(format!("seeding cached native provider: {error}")))?;
    }
    register_adopted_provider(&native_dst, store_dir, state_dir)?;
    debug_assert!(stagex_dst.is_dir());
    debug_assert!(native_dst.is_dir());
    Ok(true)
}

fn stage_markers_dir(staging_dir: &Path) -> PathBuf {
    staging_dir.join(STAGE_MARKERS_SUBDIR)
}

fn write_stage_marker(prepared: &PreparedAttempt, stage_id: &str, digest_blake3: &str) -> Result<(), RunError> {
    let markers_dir = stage_markers_dir(&prepared.staging_dir);
    fs::create_dir_all(&markers_dir)
        .map_err(|error| proof_error(format!("creating stage markers {}: {error}", markers_dir.display())))?;
    let marker = StageCompletionMarker {
        schema: crate::source_built_fixed_point_dev_cache::STAGE_MARKER_SCHEMA.to_string(),
        plan_digest_blake3: prepared.plan.plan_digest_blake3.clone(),
        stage_id: stage_id.to_string(),
        digest_blake3: digest_blake3.to_string(),
    };
    write_json_create_new(&markers_dir.join(stage_id), &marker)
}

fn read_stage_marker(prepared: &PreparedAttempt, stage_id: &str) -> Result<Option<StageCompletionMarker>, RunError> {
    let path = stage_markers_dir(&prepared.staging_dir).join(stage_id);
    if !path.is_file() {
        return Ok(None);
    }
    let bytes =
        fs::read(&path).map_err(|error| proof_error(format!("reading stage marker {}: {error}", path.display())))?;
    let marker: StageCompletionMarker = serde_json::from_slice(&bytes)
        .map_err(|error| proof_error(format!("parsing stage marker {}: {error}", path.display())))?;
    Ok(Some(marker))
}

fn transition_marker_is_trusted(prepared: &PreparedAttempt, fresh_digest_blake3: &str) -> Result<bool, RunError> {
    let marker = read_stage_marker(prepared, "stagex-transition")?;
    Ok(validate_stage_marker(
        marker.as_ref(),
        &prepared.plan.plan_digest_blake3,
        "stagex-transition",
        fresh_digest_blake3,
    ) == StageMarkerValidation::Trusted)
}

/// Fresh replay digest for the StageX transition's primary materialized source inputs.
/// Used to re-validate a saved resume marker against the current sources.
fn transition_input_replay_digest(prepared: &PreparedAttempt) -> Result<String, RunError> {
    let mut hasher = blake3::Hasher::new_derive_key("mantle-stagex-transition-replay-v1");
    for path in [
        &prepared.stagex_seed,
        &prepared.stagex_lineage,
        &prepared.stagex_source_bundle,
    ] {
        let bytes = fs::read(path)
            .map_err(|error| proof_error(format!("hashing transition input {}: {error}", path.display())))?;
        let digest = blake3::hash(&bytes).to_hex().to_string();
        hasher.update(digest.as_bytes());
        hasher.update(b"\n");
    }
    let digest = hasher.finalize().to_hex().to_string();
    assert_eq!(digest.len(), BLAKE3_HEX_LENGTH);
    Ok(digest)
}

fn dev_provider_cache_entry_path(cache: &Path, plan_digest_blake3: &str, cache_key: &str) -> PathBuf {
    cache
        .join(DEV_CACHE_PROVIDERS_SUBDIR)
        .join(plan_digest_blake3)
        .join(cache_key)
        .join(DEV_CACHE_ENTRY_FILE)
}

fn read_provider_cache_entry(
    options: &SourceBuiltFixedPointOptions<'_>,
    plan: &SourceBuiltFixedPointPlan,
    policies: &DevCachePolicies,
) -> Result<Option<DevProviderCacheEntry>, RunError> {
    let Some(cache) = options.dev_provider_cache else {
        return Ok(None);
    };
    let cache_key = dev_provider_cache_key(&plan.source_authority_digest_blake3, policies);
    let entry_path = dev_provider_cache_entry_path(cache, &plan.plan_digest_blake3, &cache_key);
    if !entry_path.is_file() {
        return Ok(None);
    }
    let bytes = fs::read(&entry_path)
        .map_err(|error| proof_error(format!("reading dev provider-cache entry {}: {error}", entry_path.display())))?;
    let entry: DevProviderCacheEntry = serde_json::from_slice(&bytes)
        .map_err(|error| proof_error(format!("parsing dev provider-cache entry {}: {error}", entry_path.display())))?;
    Ok(Some(entry))
}

/// Decide, from the current plan, whether this dev attempt may adopt receipt-validated
/// cached providers instead of reconstructing the StageX and native providers.
pub(crate) fn dev_cache_adoption(
    options: &SourceBuiltFixedPointOptions<'_>,
    plan: &SourceBuiltFixedPointPlan,
) -> Result<bool, RunError> {
    let Some(cache) = options.dev_provider_cache else {
        return Ok(false);
    };
    let policies = DevCachePolicies::from_plan(plan);
    let entry = read_provider_cache_entry(options, plan, &policies)?;
    let hit = evaluate_provider_cache_lookup(entry.as_ref(), plan, &policies) == DevProviderCacheLookup::Hit;
    if !hit {
        return Ok(false);
    }
    let entry = entry.expect("hit implies a present entry");
    let stagex_root = cache
        .join(DEV_CACHE_PROVIDERS_SUBDIR)
        .join(&plan.plan_digest_blake3)
        .join(&entry.cache_key)
        .join(&entry.stagex_provider.store_basename);
    let native_root = cache
        .join(DEV_CACHE_PROVIDERS_SUBDIR)
        .join(&plan.plan_digest_blake3)
        .join(&entry.cache_key)
        .join(&entry.native_provider.store_basename);
    if !stagex_root.is_dir() || !native_root.is_dir() {
        return Ok(false);
    }
    Ok(true)
}

fn record_dev_cache_hit(prepared: &PreparedAttempt) -> Result<(), RunError> {
    let transcript = prepared.transcripts_dir.join("dev-provider-cache.txt");
    let line =
        format!("dev-cache-hit plan_digest={} stagex=adopted native=adopted\n", prepared.plan.plan_digest_blake3);
    write_bytes_create_new(&transcript, line.as_bytes())
}

/// Copy the cached StageX and native provider store subtrees into the fresh
/// staging native store and return an adopted native-provider observation.
/// Only called after a validated receipt hit, so the cached bytes are bound to
/// the current source-authority and policy digests.
fn adopt_cached_provider_subtrees(
    options: &SourceBuiltFixedPointOptions<'_>,
    prepared: &PreparedAttempt,
    plan: &SourceBuiltFixedPointPlan,
) -> Result<BuildObservation, RunError> {
    let cache = options
        .dev_provider_cache
        .ok_or_else(|| proof_error("dev provider-cache flag is unset during adoption".to_string()))?;
    let policies = DevCachePolicies::from_plan(plan);
    let entry = read_provider_cache_entry(options, plan, &policies)?
        .ok_or_else(|| proof_error("dev provider-cache entry missing during adoption".to_string()))?;
    if evaluate_provider_cache_lookup(Some(&entry), plan, &policies) != DevProviderCacheLookup::Hit {
        return Err(proof_error("dev provider-cache entry did not re-validate during adoption".to_string()));
    }
    let entry_root = cache.join(DEV_CACHE_PROVIDERS_SUBDIR).join(&plan.plan_digest_blake3).join(&entry.cache_key);
    let stagex_dst = prepared.native_store_dir.join(&entry.stagex_provider.store_basename);
    if !stagex_dst.exists() {
        crate::stagex_mes_lib::copy_tree_bounded(&entry_root.join(&entry.stagex_provider.store_basename), &stagex_dst)
            .map_err(|error| proof_error(format!("adopting cached StageX provider: {error}")))?;
    }
    let native_dst = prepared.native_store_dir.join(&entry.native_provider.store_basename);
    if !native_dst.exists() {
        crate::stagex_mes_lib::copy_tree_bounded(&entry_root.join(&entry.native_provider.store_basename), &native_dst)
            .map_err(|error| proof_error(format!("adopting cached native provider: {error}")))?;
    }
    let tx = prepared.transcripts_dir.join("native-provider.adopted.txt");
    let tx_bytes = format!(
        "dev-cache-adopted native provider logical={} physical={} store_registered=true\n",
        entry.native_provider.logical_path,
        native_dst.display()
    );
    write_bytes_create_new(&tx, tx_bytes.as_bytes())?;
    let tx_digest = crate::protected_exec::blake3_file_hex(&tx)
        .map_err(|error| proof_error(format!("hashing adopted native transcript {}: {error}", tx.display())))?;
    debug_assert!(stagex_dst.is_dir());
    debug_assert!(native_dst.is_dir());
    Ok(BuildObservation {
        output: BuildJsonOutput {
            name: BUILD_OUTPUT_NAME.to_string(),
            path: native_dst.clone(),
            artifact_attestation: BuildJsonAttestationReference {
                logical_path: entry.native_provider.logical_path.clone(),
                path: tx.clone(),
            },
        },
        transcript_path: tx,
        transcript_digest_blake3: tx_digest,
        action_trust: None,
    })
}

/// Register an on-disk provider subtree with the content-addressed store service
/// (through `SourceStore` admission), matching how the cold path adopts
/// freshly constructed transition/provider paths. Returns the logical store path.
fn register_adopted_provider(
    physical_path: &Path,
    native_store_dir: &Path,
    native_state_dir: &Path,
) -> Result<String, RunError> {
    let logical = crate::full_source_provider::adopt_verified_local_provider_path_strict(
        physical_path,
        native_store_dir,
        native_state_dir,
        LOGICAL_STORE_PREFIX,
    )?;
    crate::source_bundle::import_constructed_store_path_source(
        &logical,
        physical_path,
        native_state_dir,
        LOGICAL_STORE_PREFIX,
    )?;
    debug_assert!(logical.starts_with(LOGICAL_STORE_PREFIX));
    Ok(logical)
}

/// A placeholder StageX provider publication report for an adopted run. The
/// adopted score is carried only as a bounded reference; adopted runs never emit
/// a promoted receipt, so these path/digest fields are not hashed into evidence.
fn synthesized_adopted_stagex_report(
    prepared: &PreparedAttempt,
) -> crate::stagex_provider::StagexProviderPublicationReport {
    let stagex_root = prepared.native_store_dir.join(STAGEX_PROVIDER_STORE_BASENAME);
    crate::stagex_provider::StagexProviderPublicationReport {
        schema: "mantle-stagex-provider-publication-v1",
        provider_kind: "stagex-intermediate-provider",
        output_path: stagex_root.clone(),
        receipt_path: stagex_root.join(crate::stagex_provider::PROVIDER_RECEIPT_RELATIVE_PATH),
        normalized_provider_digest_blake3: STAGEX_PROVIDER_EXPECTED_NORMALIZED_DIGEST.to_string(),
        output_digest_blake3: STAGEX_PROVIDER_EXPECTED_NORMALIZED_DIGEST.to_string(),
        final_bundle_digest_blake3: STAGEX_PROVIDER_EXPECTED_NORMALIZED_DIGEST.to_string(),
        provider_validation_audit_digest_blake3: STAGEX_PROVIDER_EXPECTED_NORMALIZED_DIGEST.to_string(),
        provider_validation_report_digest_blake3: STAGEX_PROVIDER_EXPECTED_NORMALIZED_DIGEST.to_string(),
    }
}

fn run_in_isolated_exec_thread<T, F>(operation_name: &str, operation: F) -> Result<T, RunError>
where
    T: Send,
    F: FnOnce() -> T + Send,
{
    if operation_name.is_empty() {
        return Err(proof_error("isolated protected operation name is empty".to_string()));
    }
    let result = std::thread::scope(|scope| scope.spawn(operation).join());
    let value = result.map_err(|_| proof_error(format!("{operation_name} worker panicked")))?;
    assert!(!operation_name.is_empty());
    Ok(value)
}

fn materialize_or_validate_stagex_transition_handoff(
    is_dev: bool,
    execution_root: &Path,
    handoff_root: &Path,
    replay_root: &Path,
) -> Result<(), RunError> {
    if !is_dev || !handoff_root.exists() {
        return materialize_stagex_transition_handoff(execution_root, handoff_root);
    }
    if replay_root.exists() {
        return Err(proof_error(format!(
            "StageX transition replay destination already exists: {}",
            replay_root.display()
        )));
    }
    materialize_stagex_transition_handoff(execution_root, replay_root)?;
    validate_identical_directory_trees("StageX transition handoff", replay_root, handoff_root)?;
    fs::remove_dir_all(replay_root).map_err(|error| {
        proof_error(format!("removing validated StageX transition replay {}: {error}", replay_root.display()))
    })?;
    assert!(handoff_root.is_dir());
    assert!(!replay_root.exists());
    Ok(())
}

fn validate_identical_directory_trees(label: &str, replay_root: &Path, persistent_root: &Path) -> Result<(), RunError> {
    if label.is_empty() || !replay_root.is_dir() || !persistent_root.is_dir() {
        return Err(proof_error(format!(
            "{label} replay comparison requires two directories: replay={} persistent={}",
            replay_root.display(),
            persistent_root.display()
        )));
    }
    let replay = crate::release_tree_copy::hash_directory_tree(replay_root)
        .map_err(|error| proof_error(format!("hashing {label} replay {}: {error}", replay_root.display())))?;
    let persistent = crate::release_tree_copy::hash_directory_tree(persistent_root)
        .map_err(|error| proof_error(format!("hashing persistent {label} {}: {error}", persistent_root.display())))?;
    if replay != persistent {
        return Err(proof_error(format!(
            "{label} replay tree mismatch: replay_bytes={} replay_blake3={} persistent_bytes={} persistent_blake3={}",
            replay.0, replay.1, persistent.0, persistent.1
        )));
    }
    assert_eq!(replay.0, persistent.0);
    assert_eq!(replay.1, persistent.1);
    Ok(())
}

fn materialize_stagex_transition_handoff(execution_root: &Path, handoff_root: &Path) -> Result<(), RunError> {
    validate_stagex_transition_handoff_layout()?;
    validate_stagex_transition_handoff_sources(execution_root)?;
    if handoff_root.exists() {
        return Err(proof_error(format!(
            "StageX transition handoff destination already exists: {}",
            handoff_root.display()
        )));
    }
    fs::create_dir(handoff_root).map_err(|error| {
        proof_error(format!("creating StageX transition handoff {}: {error}", handoff_root.display()))
    })?;
    for relative in STAGEX_TRANSITION_HANDOFF_DIRECTORIES {
        crate::stagex_mes_lib::copy_tree_bounded(&execution_root.join(relative), &handoff_root.join(relative))
            .map_err(|error| proof_error(format!("copying StageX transition handoff {relative}: {error}")))?;
    }
    let report = StagexTransitionHandoffReport {
        format: STAGEX_TRANSITION_HANDOFF_REPORT_FORMAT,
        copied_directories: STAGEX_TRANSITION_HANDOFF_DIRECTORIES.to_vec(),
        non_claim: STAGEX_TRANSITION_HANDOFF_NON_CLAIM,
    };
    let report_bytes = serde_json::to_vec_pretty(&report)
        .map_err(|error| proof_error(format!("serializing StageX transition handoff report: {error}")))?;
    crate::stagex_mes_lib::write_create_new(&handoff_root.join(STAGEX_TRANSITION_HANDOFF_REPORT_FILE), &report_bytes)
        .map_err(|error| proof_error(format!("writing StageX transition handoff report: {error}")))?;
    validate_stagex_transition_handoff_outputs(handoff_root)?;
    assert!(execution_root.join(STAGEX_TRANSITION_REPORT_FILE).is_file());
    assert!(handoff_root.join(STAGEX_TRANSITION_HANDOFF_REPORT_FILE).is_file());
    Ok(())
}

fn validate_stagex_transition_handoff_layout() -> Result<(), RunError> {
    let directories = STAGEX_TRANSITION_HANDOFF_DIRECTORIES.iter().copied().map(Path::new).collect::<Vec<_>>();
    let unique_directories = directories.iter().copied().collect::<std::collections::BTreeSet<_>>();
    let files_are_covered = STAGEX_TRANSITION_HANDOFF_REQUIRED_FILES
        .iter()
        .all(|file| directories.iter().any(|directory| Path::new(file).starts_with(directory)));
    let paths_are_relative = directories
        .iter()
        .copied()
        .chain(STAGEX_TRANSITION_HANDOFF_REQUIRED_FILES.iter().copied().map(Path::new))
        .all(|path| {
            path.is_relative()
                && !path.components().any(|component| matches!(component, std::path::Component::ParentDir))
        });
    if unique_directories.len() != directories.len() || !files_are_covered || !paths_are_relative {
        return Err(proof_error("StageX transition handoff layout is invalid".to_string()));
    }
    assert_eq!(unique_directories.len(), STAGEX_TRANSITION_HANDOFF_DIRECTORY_COUNT);
    assert!(files_are_covered);
    Ok(())
}

fn validate_stagex_transition_handoff_sources(execution_root: &Path) -> Result<(), RunError> {
    if !execution_root.is_absolute() || !execution_root.is_dir() {
        return Err(proof_error(format!(
            "StageX transition execution root is not an absolute directory: {}",
            execution_root.display()
        )));
    }
    for relative in STAGEX_TRANSITION_HANDOFF_DIRECTORIES {
        if !execution_root.join(relative).is_dir() {
            return Err(proof_error(format!("StageX transition handoff directory is missing: {relative}")));
        }
    }
    for relative in STAGEX_TRANSITION_HANDOFF_REQUIRED_FILES {
        if !execution_root.join(relative).is_file() {
            return Err(proof_error(format!("StageX transition handoff file is missing: {relative}")));
        }
    }
    for relative in [
        STAGEX_TRANSITION_PLAN_FILE,
        STAGEX_TRANSITION_REPORT_FILE,
        STAGEX_TRANSITION_AUDIT_FILE,
    ] {
        if !execution_root.join(relative).is_file() {
            return Err(proof_error(format!("StageX transition evidence file is missing: {relative}")));
        }
    }
    assert!(execution_root.is_absolute());
    assert!(execution_root.join(STAGEX_TRANSITION_REPORT_FILE).is_file());
    Ok(())
}

fn validate_stagex_transition_handoff_outputs(handoff_root: &Path) -> Result<(), RunError> {
    for relative in STAGEX_TRANSITION_HANDOFF_DIRECTORIES {
        if !handoff_root.join(relative).is_dir() {
            return Err(proof_error(format!("copied StageX transition handoff directory is missing: {relative}")));
        }
    }
    for relative in STAGEX_TRANSITION_HANDOFF_REQUIRED_FILES {
        if !handoff_root.join(relative).is_file() {
            return Err(proof_error(format!("copied StageX transition handoff file is missing: {relative}")));
        }
    }
    assert!(handoff_root.is_absolute());
    assert!(handoff_root.join(STAGEX_TRANSITION_HANDOFF_REPORT_FILE).is_file());
    Ok(())
}

fn validate_runtime_bounds(
    options: &SourceBuiltFixedPointOptions<'_>,
    prepared: &PreparedAttempt,
) -> Result<(), RunError> {
    let elapsed_max = Duration::from_secs(options.elapsed_seconds_max);
    let elapsed = prepared.started_at.elapsed();
    if elapsed > elapsed_max {
        return Err(proof_error(format!(
            "proof elapsed time {:?} exceeds configured bound {:?}",
            elapsed, elapsed_max
        )));
    }
    let available_bytes = fs2::available_space(&prepared.staging_dir)
        .map_err(|error| proof_error(format!("probing proof disk use {}: {error}", prepared.staging_dir.display())))?;
    let consumed_bytes = prepared.disk_available_bytes_before.saturating_sub(available_bytes);
    if consumed_bytes > options.disk_bytes_max {
        return Err(proof_error(format!(
            "proof observed disk use {consumed_bytes} bytes exceeds configured bound {}",
            options.disk_bytes_max
        )));
    }
    Ok(())
}

fn validate_stagex_provider_normalized_identity(observed_digest_blake3: &str) -> Result<(), RunError> {
    if observed_digest_blake3 != STAGEX_PROVIDER_EXPECTED_NORMALIZED_DIGEST {
        return Err(proof_error(format!(
            "StageX normalized provider digest mismatch: expected {STAGEX_PROVIDER_EXPECTED_NORMALIZED_DIGEST}, observed {observed_digest_blake3}"
        )));
    }
    assert_eq!(observed_digest_blake3.len(), BLAKE3_HEX_LENGTH);
    debug_assert!(observed_digest_blake3.bytes().all(|byte| byte.is_ascii_hexdigit()));
    Ok(())
}

fn validate_reusable_stagex_provider_identity(
    replay_digest_blake3: &str,
    persistent_digest_blake3: &str,
) -> Result<(), RunError> {
    if replay_digest_blake3 != persistent_digest_blake3 {
        return Err(proof_error(format!(
            "StageX provider replay payload mismatch: replay_blake3={replay_digest_blake3} persistent_blake3={persistent_digest_blake3}"
        )));
    }
    assert_eq!(replay_digest_blake3.len(), BLAKE3_HEX_LENGTH);
    assert_eq!(persistent_digest_blake3.len(), BLAKE3_HEX_LENGTH);
    Ok(())
}

fn construct_full_source_providers(
    options: &SourceBuiltFixedPointOptions<'_>,
    prepared: &PreparedAttempt,
    stagex_transition_execution_dir: PathBuf,
    stagex_provider_report: crate::stagex_provider::StagexProviderPublicationReport,
    adopted_native: Option<BuildObservation>,
) -> Result<ConstructedProviders, RunError> {
    let native_prefix = construct_native_provider_prefix(
        options,
        prepared,
        stagex_transition_execution_dir,
        stagex_provider_report,
        adopted_native,
    )?;
    construct_rust_provider_from_native_prefix(options, prepared, native_prefix)
}

fn construct_native_provider_prefix(
    options: &SourceBuiltFixedPointOptions<'_>,
    prepared: &PreparedAttempt,
    stagex_transition_execution_dir: PathBuf,
    stagex_provider_report: crate::stagex_provider::StagexProviderPublicationReport,
    adopted_native: Option<BuildObservation>,
) -> Result<NativeProviderPrefix, RunError> {
    let native_provider = match adopted_native {
        Some(observation) => observation,
        None => run_native_build(options, prepared, NATIVE_PROVIDER_ID, NATIVE_PROVIDER_NCL)?,
    };
    let native_admission_report_path = prepared.staging_dir.join(NATIVE_ADMISSION_REPORT_FILE);
    let native_admission = crate::full_source_provider::cmd_admit_full_source_provider(
        &native_provider.output.path,
        options.expected_native_provider_blake3,
        &prepared.native_source_manifest,
        &crate::source_bundle::read_source_bundle(&prepared.native_source_manifest)?.manifest_blake3,
        &native_admission_report_path,
        false,
    )?;
    if native_admission.status != "admitted" {
        return Err(proof_error(format!(
            "native provider admission status must be admitted, got {}",
            native_admission.status
        )));
    }
    let host_tools = build_full_source_host_tools(options, prepared)?;
    let native_action_trust = aggregate_native_build_action_trust(options, prepared, &native_provider, &host_tools)?;
    let host_tool_manifest_dir = prepared.staging_dir.join(HOST_TOOLS_EVIDENCE_DIR);
    let host_tool_manifest = crate::full_source_rust_binding_shell::materialize_full_source_rust_host_tools(
        FullSourceRustHostToolMaterializationRequest {
            admission_report_path: &native_admission_report_path,
            make_root: &host_tools["make"].output.path,
            make_attestation_path: &host_tools["make"].output.artifact_attestation.path,
            cmake_root: &host_tools["cmake"].output.path,
            cmake_attestation_path: &host_tools["cmake"].output.artifact_attestation.path,
            python_root: &host_tools["python"].output.path,
            python_attestation_path: &host_tools["python"].output.artifact_attestation.path,
            perl_root: &host_tools["perl"].output.path,
            perl_attestation_path: &host_tools["perl"].output.artifact_attestation.path,
            busybox_root: &host_tools["busybox"].output.path,
            busybox_attestation_path: &host_tools["busybox"].output.artifact_attestation.path,
            linux_headers_root: &host_tools["linux-headers"].output.path,
            linux_headers_attestation_path: &host_tools["linux-headers"].output.artifact_attestation.path,
            output_dir: &host_tool_manifest_dir,
        },
    )
    .map_err(|error| proof_error(format!("constructing full-source Rust host-tool evidence: {error}")))?;
    let prefix = NativeProviderPrefix {
        stagex_transition_execution_dir,
        stagex_provider_report,
        native_provider,
        native_action_trust,
        native_admission,
        native_admission_report_path,
        rust_host_tools: host_tools,
        rust_host_tool_evidence_dir: host_tool_manifest_dir,
        rust_host_tool_manifest_path: host_tool_manifest,
    };
    checkpoint_integration::publish_dev_provider_boundary(
        options,
        prepared,
        checkpoint_integration::ProviderPrefix::Native(&prefix),
    )?;
    Ok(prefix)
}

fn construct_rust_provider_from_native_prefix(
    options: &SourceBuiltFixedPointOptions<'_>,
    prepared: &PreparedAttempt,
    prefix: NativeProviderPrefix,
) -> Result<ConstructedProviders, RunError> {
    let NativeProviderPrefix {
        stagex_transition_execution_dir,
        stagex_provider_report,
        native_provider,
        native_action_trust,
        native_admission,
        native_admission_report_path,
        rust_host_tools: host_tools,
        rust_host_tool_evidence_dir: host_tool_manifest_dir,
        rust_host_tool_manifest_path: host_tool_manifest,
    } = prefix;
    let rust_provider_root = prepared.staging_dir.join(RUST_PROVIDER_DIR);
    let rust_scratch = prepared.staging_dir.join(RUST_PROVIDER_SCRATCH_DIR);
    let rust_provider_open_file_descriptors_max =
        u32::try_from(crate::source_built_fixed_point::SOURCE_BUILT_FIXED_POINT_OPEN_FILE_DESCRIPTORS_MAX)
            .map_err(|_| proof_error("Rust provider action open-file limit exceeds u32".to_string()))?;
    let rust_provider_action_limits = crate::source_built_rust_provider_action::RustProviderActionLimits {
        parallel_jobs_max: crate::rust_source_provider::RUST_BOOTSTRAP_JOB_COUNT,
        open_file_descriptors_max: rust_provider_open_file_descriptors_max,
        storage_bytes_max: options.disk_bytes_max,
        exec_events_per_stage_max: options.protected_exec_events_max,
    };
    fs::create_dir(&rust_scratch)
        .map_err(|error| proof_error(format!("creating Rust provider scratch {}: {error}", rust_scratch.display())))?;
    let rust_provider = crate::rust_source_provider::materialize_full_source_bound_rust_provider_with_route_plan(
        &prepared.source_root.join(RUST_RECIPE_NCL),
        Some(&prepared.source_root.join(RUST_ROUTE_PLAN_NCL)),
        &native_admission_report_path,
        &host_tool_manifest,
        &prepared.rust_source_archive_dir,
        Some(rust_provider_action_limits),
        &rust_provider_root,
        &rust_scratch,
        options.verbose,
    )
    .map_err(|error| proof_error(format!("constructing full-source Rust provider: {error}")))?;
    if rust_provider.action_trust.is_none() {
        return Err(proof_error(
            "promoted full-source Rust provider lacks action plan, audit, and reconciliation".to_string(),
        ));
    }
    let toolchain_closure_path = prepared.staging_dir.join(TOOLCHAIN_CLOSURE_FILE);
    crate::native_toolchain_closure::cmd_materialize_native_toolchain_closure(NativeToolchainClosureOptions {
        rust_source_provider: &rust_provider.output_path,
        host_root: &native_provider.output.path,
        target_root: &native_provider.output.path,
        output: &toolchain_closure_path,
    })?;
    let providers = ConstructedProviders {
        stagex_transition_execution_dir,
        stagex_provider_report,
        native_provider,
        native_action_trust,
        native_admission,
        native_admission_report_path,
        rust_provider,
        rust_host_tools: host_tools,
        rust_host_tool_evidence_dir: host_tool_manifest_dir,
        toolchain_closure_path,
        provider_checkpoint: None,
    };
    checkpoint_integration::publish_dev_provider_boundary(
        options,
        prepared,
        checkpoint_integration::ProviderPrefix::Complete(&providers),
    )?;
    Ok(providers)
}

fn aggregate_native_build_action_trust(
    options: &SourceBuiltFixedPointOptions<'_>,
    prepared: &PreparedAttempt,
    native_provider: &BuildObservation,
    host_tools: &BTreeMap<String, BuildObservation>,
) -> Result<Option<NativeBuildActionTrustEvidence>, RunError> {
    let mut evidence = Vec::with_capacity(host_tools.len().saturating_add(1));
    evidence.push(native_provider.action_trust.as_ref());
    evidence.extend(host_tools.values().map(|observation| observation.action_trust.as_ref()));
    if evidence.iter().any(|item| item.is_none()) {
        if options.dev_provider_cache.is_some() {
            return Ok(None);
        }
        return Err(proof_error(
            "promoted native provider construction lacks an eager action plan or reconciliation".to_string(),
        ));
    }
    let evidence = evidence.into_iter().flatten().collect::<Vec<_>>();
    let plans = evidence.iter().map(|item| item.plan.clone()).collect::<Vec<_>>();
    let plan =
        crate::source_built_derivation_action_plan::compose_eager_derivation_action_plans(NATIVE_PROVIDER_ID, &plans)
            .map_err(|error| proof_error(format!("composing native provider action plans: {error}")))?;
    let observed = evidence
        .iter()
        .flat_map(|item| item.reconciliation.observed_event_ids_blake3.iter().cloned())
        .collect::<Vec<_>>();
    let reconciliation =
        crate::source_built_derivation_action_plan::reconcile_eager_derivation_actions(&plan, &observed)
            .map_err(|error| proof_error(format!("reconciling native provider action plan: {error}")))?;
    let plan_path = prepared.staging_dir.join(NATIVE_ACTION_PLAN_FILE);
    let reconciliation_path = prepared.staging_dir.join(NATIVE_ACTION_RECONCILIATION_FILE);
    write_json_create_new(&plan_path, &plan)?;
    write_json_create_new(&reconciliation_path, &reconciliation)?;
    if options.dev_provider_cache.is_none() {
        crate::source_built_derivation_action_plan::require_complete_eager_derivation_reconciliation(&reconciliation)
            .map_err(|error| proof_error(format!("native provider action reconciliation: {error}")))?;
    } else {
        // Dev stages can observe cached derivations. Retain their exact action
        // evidence without promoting missing events to fresh execution.
        validate_native_build_reconciliation("dev native prefix", &reconciliation)?;
    }
    assert!(reconciliation.matched_action_count <= plan.action_count);
    debug_assert_eq!(reconciliation.action_plan_digest_blake3, plan.plan_digest_blake3);
    Ok(Some(NativeBuildActionTrustEvidence {
        plan_path,
        reconciliation_path,
        plan,
        reconciliation,
    }))
}

fn build_full_source_host_tools(
    options: &SourceBuiltFixedPointOptions<'_>,
    prepared: &PreparedAttempt,
) -> Result<BTreeMap<String, BuildObservation>, RunError> {
    let mut outputs = BTreeMap::new();
    for (label, relative_path) in [
        ("make", HOST_MAKE_NCL),
        ("linux-headers", HOST_LINUX_HEADERS_NCL),
        ("busybox", HOST_BUSYBOX_NCL),
        ("cmake", HOST_CMAKE_NCL),
        ("python", HOST_PYTHON_NCL),
        ("perl", HOST_PERL_NCL),
    ] {
        let observed = run_native_build(options, prepared, label, relative_path)?;
        if outputs.insert(label.to_string(), observed).is_some() {
            return Err(proof_error(format!("duplicate host-tool build label {label}")));
        }
    }
    assert_eq!(outputs.len(), 6);
    debug_assert!(outputs.values().all(|output| output.output.path.is_absolute()));
    Ok(outputs)
}

fn run_native_build(
    options: &SourceBuiltFixedPointOptions<'_>,
    prepared: &PreparedAttempt,
    label: &str,
    relative_ncl: &str,
) -> Result<BuildObservation, RunError> {
    let ncl_path = prepared.source_root.join(relative_ncl);
    let action_plan = plan_native_build_actions(options, prepared, label, &ncl_path)?;
    let plan_path = prepared.transcripts_dir.join(format!("{label}.action-plan.json"));
    write_json_create_new(&plan_path, &action_plan)?;
    let mut command = native_build_command(options, prepared, &ncl_path)?;
    let output = command.output().map_err(|error| proof_error(format!("launching native build {label}: {error}")))?;
    validate_runtime_bounds(options, prepared)?;
    let stdout_path = prepared.transcripts_dir.join(format!("{label}.json"));
    let stderr_path = prepared.transcripts_dir.join(format!("{label}.stderr.txt"));
    write_bytes_create_new(&stdout_path, &output.stdout)?;
    write_bytes_create_new(&stderr_path, &output.stderr)?;
    let allow_cached = options.dev_provider_cache.is_some();
    let report = parse_build_report(label, &output, allow_cached)?;
    let observed_goal_ids = report
        .scheduler_priority_decisions
        .iter()
        .map(|decision| decision.selected_goal_key_blake3.clone())
        .collect::<Vec<_>>();
    let reconciliation = crate::source_built_derivation_action_plan::reconcile_eager_derivation_actions(
        &action_plan,
        &observed_goal_ids,
    )
    .map_err(|error| proof_error(format!("reconciling native build {label}: {error}")))?;
    let reconciliation_path = prepared.transcripts_dir.join(format!("{label}.action-reconciliation.json"));
    write_json_create_new(&reconciliation_path, &reconciliation)?;
    validate_native_build_reconciliation(label, &reconciliation)?;
    let action_trust = NativeBuildActionTrustEvidence {
        plan_path,
        reconciliation_path,
        plan: action_plan,
        reconciliation,
    };
    let build_output = require_single_build_output(label, report, allow_cached)?;
    let transcript_digest_blake3 = crate::protected_exec::blake3_file_hex(&stdout_path)
        .map_err(|error| proof_error(format!("hashing build transcript {}: {error}", stdout_path.display())))?;
    Ok(BuildObservation {
        output: build_output,
        transcript_path: stdout_path,
        transcript_digest_blake3,
        action_trust: Some(action_trust),
    })
}

fn plan_native_build_actions(
    options: &SourceBuiltFixedPointOptions<'_>,
    prepared: &PreparedAttempt,
    label: &str,
    ncl_path: &Path,
) -> Result<crate::source_built_derivation_action_plan::EagerDerivationActionPlan, RunError> {
    let sandbox_shell_digest = crate::protected_exec::blake3_file_hex(options.sandbox_shell)
        .map_err(|error| proof_error(format!("hashing proof sandbox shell: {error}")))?;
    let import_paths = [
        prepared.source_root.join("lib").into_os_string(),
        prepared.source_root.join("bootstrap").into_os_string(),
    ];
    let stage_id = format!("full-source-native-provider:{label}");
    crate::build_plan::capture_eager_derivation_action_plan(
        ncl_path,
        &import_paths,
        LOGICAL_STORE_PREFIX,
        &stage_id,
        &sandbox_shell_digest,
    )
}

fn validate_native_build_reconciliation(
    label: &str,
    reconciliation: &crate::source_built_derivation_action_plan::EagerDerivationReconciliation,
) -> Result<(), RunError> {
    if !reconciliation.unknown_event_ids_blake3.is_empty()
        || !reconciliation.overbound_event_ids_blake3.is_empty()
        || !reconciliation.local_only
        || reconciliation.cache_only_completion_count != 0
    {
        return Err(proof_error(format!(
            "native build {label} emitted unknown, overbound, remote, or cache-only action evidence"
        )));
    }
    assert!(reconciliation.matched_event_count <= reconciliation.observed_event_count);
    debug_assert!(reconciliation.blockers.iter().all(|blocker| blocker == "missing-derivation-actions"));
    Ok(())
}

fn native_build_command(
    options: &SourceBuiltFixedPointOptions<'_>,
    prepared: &PreparedAttempt,
    ncl_path: &Path,
) -> Result<Command, RunError> {
    let current_executable = std::env::current_exe()
        .map_err(|error| proof_error(format!("resolving current Mantle executable: {error}")))?;
    let mut command = Command::new(current_executable);
    let bwrap_parent = options
        .bwrap
        .parent()
        .ok_or_else(|| proof_error(format!("proof bwrap path has no parent: {}", options.bwrap.display())))?;
    let home = prepared.staging_dir.join(HOME_DIR);
    let temp = prepared.staging_dir.join(TMP_DIR);
    fs::create_dir_all(&home)
        .map_err(|error| proof_error(format!("creating proof home {}: {error}", home.display())))?;
    fs::create_dir_all(&temp)
        .map_err(|error| proof_error(format!("creating proof temporary directory {}: {error}", temp.display())))?;
    command
        .env_clear()
        .env("PATH", bwrap_parent)
        .env("HOME", &home)
        .env("USER", "mantle-proof")
        .env("LOGNAME", "mantle-proof")
        .env("TZ", "UTC")
        .env("LANG", "C")
        .env("LC_ALL", "C")
        .env("TMPDIR", &temp)
        .env("CRUNCH_NO_FUSE", "1")
        .env("SNIX_BUILD_SANDBOX_SHELL", options.sandbox_shell)
        .arg("--json")
        .arg("--store")
        .arg(&prepared.native_store_dir)
        .arg("--store-prefix")
        .arg(LOGICAL_STORE_PREFIX)
        .arg("--state-dir")
        .arg(&prepared.native_state_dir)
        .arg("build")
        .arg(ncl_path)
        .arg("--import-path")
        .arg(prepared.source_root.join("lib"))
        .arg("--import-path")
        .arg(prepared.source_root.join("bootstrap"))
        .arg("--offline-source-preflight")
        .arg("--no-substitute")
        .arg("--strict-hermetic")
        .arg("--jobs")
        .arg(options.jobs.to_string());
    Ok(command)
}

fn parse_build_report(label: &str, output: &Output, allow_cached: bool) -> Result<BuildJsonReport, RunError> {
    let stdout = std::str::from_utf8(&output.stdout)
        .map_err(|error| proof_error(format!("native build {label} stdout is not UTF-8: {error}")))?;
    if !output.status.success() {
        return Err(proof_error(format!(
            "native build {label} failed with status {:?}: {}",
            output.status.code(),
            native_build_failure_summary(stdout)
        )));
    }
    let report: BuildJsonReport = serde_json::from_str(stdout)
        .map_err(|error| proof_error(format!("parsing native build {label} report: {error}")))?;
    if report.schema != BUILD_REPORT_SCHEMA {
        return Err(proof_error(format!("native build {label} report schema is {}", report.schema)));
    }
    if report.hermeticity_mode != "strict" || !report.hermeticity_audit_events.is_empty() {
        return Err(proof_error(format!("native build {label} did not preserve strict zero-audit hermeticity")));
    }
    if !report.failed.is_empty() {
        return Err(proof_error(format!(
            "native build {label} report contains failures: {}",
            build_failures_summary(&report.failed)
        )));
    }
    if !allow_cached && report.outcomes.iter().any(|outcome| outcome.cached) {
        return Err(proof_error(format!("native build {label} report contains a cache-hit outcome")));
    }
    debug_assert!(report.failed.is_empty());
    Ok(report)
}

fn require_single_build_output(
    label: &str,
    report: BuildJsonReport,
    allow_cached: bool,
) -> Result<BuildJsonOutput, RunError> {
    if report.outcomes.len() != EXPECTED_SINGLE_OUTPUT_COUNT {
        return Err(proof_error(format!(
            "native build {label} must have one root outcome, got {}",
            report.outcomes.len()
        )));
    }
    let mut outcomes = report.outcomes.into_iter();
    let outcome = outcomes.next().expect("validated one outcome");
    if outcome.cached && !allow_cached {
        return Err(proof_error(format!("native build {label} was satisfied from cache")));
    }
    if outcome.outputs.len() != EXPECTED_SINGLE_OUTPUT_COUNT {
        return Err(proof_error(format!("native build {label} must have one output, got {}", outcome.outputs.len())));
    }
    let output = outcome.outputs.into_iter().next().expect("validated one output");
    if output.name != BUILD_OUTPUT_NAME || !output.path.is_absolute() || !output.artifact_attestation.path.is_absolute()
    {
        return Err(proof_error(format!("native build {label} returned malformed output authority")));
    }
    if !output.path.exists() || !output.artifact_attestation.path.is_file() {
        return Err(proof_error(format!("native build {label} output or attestation is missing")));
    }
    if outcome.label.is_empty() || !output.artifact_attestation.logical_path.starts_with(LOGICAL_STORE_PREFIX) {
        return Err(proof_error(format!("native build {label} returned invalid report identity")));
    }
    Ok(output)
}

fn run_cargo_free_fixed_point(
    options: &SourceBuiltFixedPointOptions<'_>,
    prepared: &PreparedAttempt,
    providers: &ConstructedProviders,
    resume: &dev_resume::FixedPointResume,
    resume_plan: &crunch_dev_resume_core::ResumePlan,
) -> Result<(), RunError> {
    let target = TARGET_TRIPLE.to_string();
    let fixed_point_dir = prepared.staging_dir.join(FIXED_POINT_DIR);
    if matches!(resume, dev_resume::FixedPointResume::Complete) {
        return validate_completed_fixed_point(options, providers, &fixed_point_dir);
    }
    let open_file_descriptors_max =
        u32::try_from(crate::source_built_fixed_point::SOURCE_BUILT_FIXED_POINT_OPEN_FILE_DESCRIPTORS_MAX)
            .map_err(|_| proof_error("Rust action open-file limit exceeds u32".to_string()))?;
    let rust_action_resources = crate::source_built_rust_action_plan::RustActionResourceLimits {
        parallel_jobs_max: options.jobs,
        open_file_descriptors_max,
        storage_bytes_max: options.disk_bytes_max,
        exec_events_per_action_max: options.protected_exec_events_max,
    };
    let publication = dev_resume::FixedPointPublication {
        options,
        prepared,
        resume_plan,
    };
    let publisher: Option<&dyn crate::cargo_free_self_build::FixedPointStage1Publisher> =
        options.dev_resume.then_some(&publication);
    crate::cargo_free_self_build::cmd_cargo_free_fixed_point_self_build_with_publisher(
        CargoFreeSelfBuildOptions {
            root: &prepared.source_root,
            out_dir: &fixed_point_dir,
            rustc: &providers.rust_provider.output_path.join(RUSTC_RELATIVE_PATH),
            targets: &[target],
            toolchain_closure: Some(&providers.toolchain_closure_path),
            rust_source_provider: Some(&providers.rust_provider.output_path),
            rust_action_resources: Some(rust_action_resources),
            hermeticity_mode: crunch_pipeline::HermeticityMode::Strict,
            resume: match resume {
                dev_resume::FixedPointResume::None => crate::cargo_free_self_build::CargoFreeFixedPointResume::None,
                dev_resume::FixedPointResume::Stage1 => crate::cargo_free_self_build::CargoFreeFixedPointResume::Stage1,
                dev_resume::FixedPointResume::Complete => {
                    return Err(proof_error("complete fixed-point resume reached execution".to_string()));
                }
            },
            json: options.json,
        },
        publisher,
    )?;
    validate_completed_fixed_point(options, providers, &fixed_point_dir)?;
    dev_resume::publish_fixed_point_complete(&publication)
}

fn validate_completed_fixed_point(
    options: &SourceBuiltFixedPointOptions<'_>,
    providers: &ConstructedProviders,
    fixed_point_dir: &Path,
) -> Result<(), RunError> {
    let meta_path = fixed_point_dir.join(crate::cargo_free_self_build::META_FILE);
    let meta: serde_json::Value = serde_json::from_slice(
        &fs::read(&meta_path)
            .map_err(|error| proof_error(format!("reading fixed-point summary {}: {error}", meta_path.display())))?,
    )
    .map_err(|error| proof_error(format!("parsing fixed-point summary {}: {error}", meta_path.display())))?;
    if meta.pointer("/status").and_then(serde_json::Value::as_str) != Some("success")
        || meta.pointer("/fixed_point").and_then(serde_json::Value::as_bool) != Some(true)
    {
        return Err(proof_error(format!("Cargo-free fixed-point summary is not successful: {}", meta_path.display())));
    }
    let stage1_digest = meta.pointer("/stage1/binary_blake3").and_then(serde_json::Value::as_str);
    let stage2_digest = meta.pointer("/stage2/binary_blake3").and_then(serde_json::Value::as_str);
    if stage1_digest.is_none() || stage1_digest != stage2_digest {
        return Err(proof_error("stage1 and stage2 Mantle BLAKE3 identities do not match".to_string()));
    }
    if stage1_digest.is_none_or(|digest| digest.len() != BLAKE3_HEX_LENGTH) {
        return Err(proof_error("fixed-point binary digest has invalid length".to_string()));
    }
    validate_fixed_point_rust_action_evidence(fixed_point_dir, &meta, "stage1")?;
    validate_fixed_point_rust_action_evidence(fixed_point_dir, &meta, "stage2")?;
    debug_assert_eq!(providers.native_admission.output_digest_blake3, options.expected_native_provider_blake3);
    Ok(())
}

fn validate_fixed_point_rust_action_evidence(
    fixed_point_dir: &Path,
    meta: &serde_json::Value,
    stage: &str,
) -> Result<(), RunError> {
    let authority_path = fixed_point_action_artifact_path(fixed_point_dir, meta, stage, "rust_child_action_authority")?;
    let plan_path = fixed_point_action_artifact_path(fixed_point_dir, meta, stage, "rust_child_action_plan")?;
    let audit_path = fixed_point_action_artifact_path(fixed_point_dir, meta, stage, "rust_child_action_audit")?;
    let reconciliation_path =
        fixed_point_action_artifact_path(fixed_point_dir, meta, stage, "rust_child_action_reconciliation")?;
    let authority = serde_json::from_slice::<crate::source_built_rust_action_plan::RustChildActionAuthority>(
        &fs::read(&authority_path).map_err(|error| {
            proof_error(format!("reading Rust action authority {}: {error}", authority_path.display()))
        })?,
    )
    .map_err(|error| proof_error(format!("parsing Rust action authority {}: {error}", authority_path.display())))?;
    crate::source_built_rust_action_plan::validate_rust_child_action_authority(&authority)
        .map_err(|error| proof_error(format!("invalid Rust action authority for {stage}: {error}")))?;
    let plan = serde_json::from_slice::<crate::source_built_rust_action_plan::RustChildActionPlan>(
        &fs::read(&plan_path)
            .map_err(|error| proof_error(format!("reading Rust action plan {}: {error}", plan_path.display())))?,
    )
    .map_err(|error| proof_error(format!("parsing Rust action plan {}: {error}", plan_path.display())))?;
    crate::source_built_rust_action_plan::validate_rust_child_action_plan(&plan)
        .map_err(|error| proof_error(format!("invalid Rust action plan for {stage}: {error}")))?;
    let reconciliation = serde_json::from_slice::<crate::source_built_rust_action_plan::RustChildActionReconciliation>(
        &fs::read(&reconciliation_path).map_err(|error| {
            proof_error(format!("reading Rust action reconciliation {}: {error}", reconciliation_path.display()))
        })?,
    )
    .map_err(|error| {
        proof_error(format!("parsing Rust action reconciliation {}: {error}", reconciliation_path.display()))
    })?;
    crate::source_built_rust_action_plan::validate_rust_child_action_reconciliation(&plan, &reconciliation)
        .map_err(|error| proof_error(format!("invalid Rust action reconciliation for {stage}: {error}")))?;
    if !reconciliation.is_complete() {
        return Err(proof_error(format!("Rust action reconciliation for {stage} is incomplete")));
    }
    let audit = serde_json::from_slice::<crate::source_built_rust_action_plan::RustChildActionAudit>(
        &fs::read(&audit_path)
            .map_err(|error| proof_error(format!("reading Rust action audit {}: {error}", audit_path.display())))?,
    )
    .map_err(|error| proof_error(format!("parsing Rust action audit {}: {error}", audit_path.display())))?;
    crate::source_built_rust_action_plan::validate_rust_child_action_audit(&plan, &audit)
        .map_err(|error| proof_error(format!("invalid Rust action audit for {stage}: {error}")))?;
    assert!(authority_path.is_file());
    assert!(reconciliation.is_complete());
    Ok(())
}

fn fixed_point_action_artifact_path(
    fixed_point_dir: &Path,
    meta: &serde_json::Value,
    stage: &str,
    field: &str,
) -> Result<PathBuf, RunError> {
    let pointer = format!("/{stage}/{field}");
    let relative = meta
        .pointer(&pointer)
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| proof_error(format!("fixed-point summary is missing {pointer}")))?;
    let relative = Path::new(relative);
    if relative.is_absolute()
        || relative.components().any(|component| !matches!(component, std::path::Component::Normal(_)))
    {
        return Err(proof_error(format!("fixed-point Rust action path is not bundle-relative: {relative:?}")));
    }
    let path = fixed_point_dir.join(relative);
    if !path.is_file() {
        return Err(proof_error(format!("fixed-point Rust action artifact is missing: {}", path.display())));
    }
    assert!(path.starts_with(fixed_point_dir));
    assert!(path.is_file());
    Ok(path)
}

fn publish_attempt(prepared: &PreparedAttempt) -> Result<(), RunError> {
    crate::linux_rename::rename_path_no_replace(&prepared.staging_dir, &prepared.final_dir).map_err(|error| {
        proof_error(format!(
            "publishing proof bundle {} -> {}: {error}",
            prepared.staging_dir.display(),
            prepared.final_dir.display()
        ))
    })?;
    let parent = prepared.final_dir.parent().ok_or_else(|| proof_error("proof output has no parent".to_string()))?;
    let target_name = prepared
        .final_dir
        .file_name()
        .ok_or_else(|| proof_error("proof output has no basename".to_string()))?;
    let aliases = [LATEST_ALIAS, LATEST_SOURCE_BUILT_ALIAS];
    if let Err(alias_error) = update_success_aliases(parent, &aliases, target_name) {
        let rollback_result = crate::linux_rename::rename_path_no_replace(&prepared.final_dir, &prepared.staging_dir);
        return match rollback_result {
            Ok(()) => Err(alias_error),
            Err(rollback_error) => Err(proof_error(format!(
                "{alias_error}; proof publication rollback {} -> {} also failed: {rollback_error}",
                prepared.final_dir.display(),
                prepared.staging_dir.display()
            ))),
        };
    }
    assert_eq!(aliases.len(), SUCCESS_ALIAS_COUNT);
    debug_assert!(prepared.final_dir.is_dir());
    Ok(())
}

fn update_success_aliases(parent: &Path, alias_names: &[&str], target_name: &std::ffi::OsStr) -> Result<(), RunError> {
    if alias_names.len() != SUCCESS_ALIAS_COUNT {
        return Err(proof_error(format!(
            "success alias transaction requires {SUCCESS_ALIAS_COUNT} aliases, got {}",
            alias_names.len()
        )));
    }
    let transaction = alias_names
        .iter()
        .map(|alias_name| {
            (
                parent.join(alias_name),
                parent.join(format!(".{alias_name}.{}.tmp", std::process::id())),
                parent.join(format!(".{alias_name}.{}.backup", std::process::id())),
            )
        })
        .collect::<Vec<_>>();
    for (alias, temporary, backup) in &transaction {
        if alias.exists() && !alias.is_symlink() {
            return Err(proof_error(format!("success alias path is not a symlink: {}", alias.display())));
        }
        remove_stale_alias_transaction_path(temporary)?;
        remove_stale_alias_transaction_path(backup)?;
    }
    for (_, temporary, _) in &transaction {
        if let Err(error) = std::os::unix::fs::symlink(target_name, temporary) {
            rollback_success_aliases(&transaction, 0);
            return Err(proof_error(format!("creating success alias staging {}: {error}", temporary.display())));
        }
    }
    let mut committed = 0usize;
    for (index, (alias, temporary, backup)) in transaction.iter().enumerate() {
        if let Err(error) = backup_existing_alias(alias, backup) {
            rollback_success_aliases(&transaction, committed);
            return Err(error);
        }
        if let Err(error) = fs::rename(temporary, alias) {
            if backup.is_symlink() {
                let _ = fs::rename(backup, alias);
            }
            rollback_success_aliases(&transaction, committed);
            return Err(proof_error(format!("publishing success alias {}: {error}", alias.display())));
        }
        committed = index.saturating_add(1);
    }
    if committed != alias_names.len() {
        rollback_success_aliases(&transaction, committed);
        return Err(proof_error(format!(
            "success alias transaction committed {committed} of {} aliases",
            alias_names.len()
        )));
    }
    for (alias, _, _) in &transaction {
        let observed_target = match fs::read_link(alias) {
            Ok(target) => target,
            Err(error) => {
                rollback_success_aliases(&transaction, committed);
                return Err(proof_error(format!("reading published success alias {}: {error}", alias.display())));
            }
        };
        if observed_target != Path::new(target_name) {
            rollback_success_aliases(&transaction, committed);
            return Err(proof_error(format!(
                "published success alias {} has target {}, expected {}",
                alias.display(),
                observed_target.display(),
                Path::new(target_name).display()
            )));
        }
    }
    for (_, _, backup) in &transaction {
        if backup.is_symlink() {
            let _ = fs::remove_file(backup);
        }
    }
    Ok(())
}

fn backup_existing_alias(alias: &Path, backup: &Path) -> Result<(), RunError> {
    if !alias.is_symlink() {
        return Ok(());
    }
    fs::rename(alias, backup)
        .map_err(|error| proof_error(format!("backing up success alias {}: {error}", alias.display())))
}

fn rollback_success_aliases(transaction: &[(PathBuf, PathBuf, PathBuf)], committed: usize) {
    for (index, (alias, temporary, backup)) in transaction.iter().enumerate().rev() {
        if index < committed && alias.is_symlink() {
            let _ = fs::remove_file(alias);
        }
        if backup.is_symlink() {
            let _ = fs::rename(backup, alias);
        }
        if temporary.is_symlink() {
            let _ = fs::remove_file(temporary);
        }
    }
}

fn remove_stale_alias_transaction_path(path: &Path) -> Result<(), RunError> {
    if !path.exists() && !path.is_symlink() {
        return Ok(());
    }
    if !path.is_symlink() {
        return Err(proof_error(format!("alias transaction path is not a symlink: {}", path.display())));
    }
    fs::remove_file(path)
        .map_err(|error| proof_error(format!("removing stale alias transaction path {}: {error}", path.display())))
}

fn print_dev_adopted_completion(
    options: &SourceBuiltFixedPointOptions<'_>,
    prepared: &PreparedAttempt,
) -> Result<(), RunError> {
    if options.json {
        let value = serde_json::json!({
            "schema": PROOF_STATUS_SCHEMA,
            "status": "dev-cache-adopted",
            "output": prepared.staging_dir,
            "plan_digest_blake3": prepared.plan.plan_digest_blake3,
            "notice": "dev-only: receipt-validated providers adopted; no promoted receipt, no release alias",
        });
        println!("{}", serde_json::to_string_pretty(&value).map_err(|error| proof_error(error.to_string()))?);
    } else {
        eprintln!("Completed dev adoption run at {}", prepared.staging_dir.display());
        eprintln!("  plan_digest_blake3: {}", prepared.plan.plan_digest_blake3);
        eprintln!("  dev-only: cached providers adopted; no promoted receipt, no release alias");
    }
    Ok(())
}

fn print_completion(options: &SourceBuiltFixedPointOptions<'_>, prepared: &PreparedAttempt) -> Result<(), RunError> {
    if options.json {
        let value = serde_json::json!({
            "schema": PROOF_STATUS_SCHEMA,
            "status": PROOF_STATUS_COMPLETE,
            "output": prepared.final_dir,
            "plan_digest_blake3": prepared.plan.plan_digest_blake3,
            "receipt": prepared.final_dir.join(FINAL_RECEIPT_FILE),
        });
        println!("{}", serde_json::to_string_pretty(&value).map_err(|error| proof_error(error.to_string()))?);
    } else {
        eprintln!("Completed source-built Mantle fixed-point proof {}", prepared.final_dir.display());
        eprintln!("  plan_digest_blake3: {}", prepared.plan.plan_digest_blake3);
        eprintln!("  receipt: {}", prepared.final_dir.join(FINAL_RECEIPT_FILE).display());
    }
    Ok(())
}

fn source_input_from_record(
    id: &str,
    role: SourceAuthorityRole,
    record: &SourceRecord,
    materialized_digest_blake3: &str,
) -> SourceAuthorityInput {
    assert!(!record.files.is_empty());
    assert!(record.payload_bytes > 0);
    assert_eq!(materialized_digest_blake3.len(), BLAKE3_HEX_LENGTH);
    SourceAuthorityInput {
        id: id.to_string(),
        role,
        kind: match role {
            SourceAuthorityRole::StagexSeed
            | SourceAuthorityRole::StagexLineage
            | SourceAuthorityRole::StagexSourceBundle
            | SourceAuthorityRole::NativeSourceBundle => SourceContentKind::RegularFile,
            _ => SourceContentKind::Directory,
        },
        digest_blake3: materialized_digest_blake3.to_string(),
        size_bytes: record.payload_bytes,
    }
}

fn hash_materialized_source(path: &Path) -> Result<String, RunError> {
    if path.is_file() {
        return crate::protected_exec::blake3_file_hex(path)
            .map_err(|error| proof_error(format!("hashing source file {}: {error}", path.display())));
    }
    if !path.is_dir() {
        return Err(proof_error(format!("materialized source path is not a file or directory: {}", path.display())));
    }
    let source_limits = crunch_release_core::TreeCopyLimits {
        entries_count_max: MATERIALIZED_SOURCE_TREE_ENTRY_COUNT_MAX,
        ..crunch_release_core::TreeCopyLimits::RELEASE_BUNDLE
    };
    let (_, digest) = crate::release_tree_copy::hash_directory_tree_with_limits(path, source_limits)
        .map_err(|error| proof_error(format!("hashing source tree {}: {error}", path.display())))?;
    assert_eq!(digest.len(), BLAKE3_HEX_LENGTH);
    debug_assert!(path.is_dir());
    Ok(digest)
}

fn sole_materialized_file(record: &SourceRecord, root: &Path) -> Result<PathBuf, RunError> {
    let mut paths = record.files.iter().map(|file| file.path.as_str()).collect::<Vec<_>>();
    paths.sort_unstable();
    paths.dedup();
    if paths.len() != EXPECTED_SINGLE_OUTPUT_COUNT {
        return Err(proof_error(format!(
            "source record {} must materialize one file path, got {}",
            record.identity,
            paths.len()
        )));
    }
    let path = root.join(paths[0]);
    if !path.is_file() {
        return Err(proof_error(format!("materialized source file is missing: {}", path.display())));
    }
    assert!(path.starts_with(root));
    debug_assert!(path.is_absolute());
    Ok(path)
}

fn staging_path(output: &Path) -> Result<PathBuf, RunError> {
    let parent = output.parent().ok_or_else(|| proof_error("proof output has no parent".to_string()))?;
    let name = output
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| proof_error("proof output basename is not UTF-8".to_string()))?;
    Ok(parent.join(format!(".{name}{STAGING_SUFFIX}-{}", std::process::id())))
}

fn create_private_attempt_dir(path: &Path) -> Result<(), RunError> {
    if path.exists() {
        return Err(proof_error(format!("proof staging path already exists: {}", path.display())));
    }
    fs::create_dir_all(path.parent().ok_or_else(|| proof_error("proof staging path has no parent".to_string()))?)
        .map_err(|error| proof_error(format!("creating proof parent for {}: {error}", path.display())))?;
    fs::create_dir(path).map_err(|error| proof_error(format!("creating proof staging {}: {error}", path.display())))?;
    fs::set_permissions(path, fs::Permissions::from_mode(ATTEMPT_DIRECTORY_MODE))
        .map_err(|error| proof_error(format!("setting proof staging permissions {}: {error}", path.display())))?;
    assert!(path.is_dir());
    debug_assert_eq!(
        fs::metadata(path).ok().map(|metadata| metadata.permissions().mode() & 0o777),
        Some(ATTEMPT_DIRECTORY_MODE)
    );
    Ok(())
}

fn write_attempt_status(
    staging_dir: &Path,
    plan_digest_blake3: Option<&str>,
    status: &'static str,
    blocker: Option<&str>,
) -> Result<(), RunError> {
    let path = staging_dir.join(ATTEMPT_STATUS_FILE);
    let value = AttemptStatus {
        schema: PROOF_STATUS_SCHEMA,
        status,
        plan_digest_blake3: plan_digest_blake3.map(ToString::to_string),
        blocker: blocker.map(ToString::to_string),
    };
    let bytes = serde_json::to_vec_pretty(&value)
        .map_err(|error| proof_error(format!("serializing attempt status: {error}")))?;
    fs::write(&path, bytes)
        .map_err(|error| proof_error(format!("writing attempt status {}: {error}", path.display())))?;
    assert!(matches!(status, PROOF_STATUS_RUNNING | PROOF_STATUS_FAILED | PROOF_STATUS_COMPLETE));
    debug_assert!(path.is_file());
    Ok(())
}

fn write_json_create_new<T: Serialize>(path: &Path, value: &T) -> Result<(), RunError> {
    let mut bytes = serde_json::to_vec_pretty(value)
        .map_err(|error| proof_error(format!("serializing {}: {error}", path.display())))?;
    bytes.push(b'\n');
    write_bytes_create_new(path, &bytes)
}

fn write_bytes_create_new(path: &Path, bytes: &[u8]) -> Result<(), RunError> {
    let mut file = fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(path)
        .map_err(|error| proof_error(format!("creating {}: {error}", path.display())))?;
    file.write_all(bytes).map_err(|error| proof_error(format!("writing {}: {error}", path.display())))?;
    file.sync_all().map_err(|error| proof_error(format!("syncing {}: {error}", path.display())))?;
    assert!(path.is_file());
    debug_assert_eq!(fs::metadata(path).ok().map(|metadata| metadata.len()), Some(bytes.len() as u64));
    Ok(())
}

fn validate_expected_digest(label: &str, digest: &str) -> Result<(), RunError> {
    let valid = digest.len() == BLAKE3_HEX_LENGTH
        && digest.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte));
    if !valid {
        return Err(proof_error(format!(
            "expected {label} BLAKE3 must be {BLAKE3_HEX_LENGTH} lowercase hexadecimal characters"
        )));
    }
    assert!(!label.is_empty());
    debug_assert!(valid);
    Ok(())
}

fn validate_executable(label: &str, path: &Path) -> Result<(), RunError> {
    let metadata =
        fs::metadata(path).map_err(|error| proof_error(format!("reading {label} {}: {error}", path.display())))?;
    if !metadata.is_file() || metadata.permissions().mode() & FILE_MODE_EXECUTABLE_MASK == 0 {
        return Err(proof_error(format!("{label} is not an executable regular file: {}", path.display())));
    }
    assert!(metadata.len() > 0);
    debug_assert!(path.is_absolute());
    Ok(())
}

fn validate_static_executable(label: &str, path: &Path) -> Result<(), RunError> {
    let bytes = fs::read(path).map_err(|error| proof_error(format!("reading {label} {}: {error}", path.display())))?;
    let risk = crate::protected_exec::classify_seed_closure_risk_bytes(&bytes);
    if risk != crate::protected_exec::SeedClosureRisk::StaticElfCandidate {
        return Err(proof_error(format!("{label} must be a static ELF candidate, got {risk}")));
    }
    assert!(!bytes.is_empty());
    debug_assert!(path.is_absolute());
    Ok(())
}

fn file_size(path: &Path) -> Result<u64, RunError> {
    let size = fs::metadata(path)
        .map_err(|error| proof_error(format!("reading file metadata {}: {error}", path.display())))?
        .len();
    if size == 0 {
        return Err(proof_error(format!("file is empty: {}", path.display())));
    }
    assert!(size > 0);
    debug_assert!(path.is_file());
    Ok(size)
}

fn policy_digest(text: &str) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(POLICY_DIGEST_DOMAIN);
    hasher.update(text.as_bytes());
    let digest = hasher.finalize().to_hex().to_string();
    assert_eq!(digest.len(), BLAKE3_HEX_LENGTH);
    debug_assert!(!text.is_empty());
    digest
}

fn native_build_failure_summary(stdout: &str) -> String {
    if let Ok(report) = serde_json::from_str::<BuildJsonReport>(stdout) {
        return build_failures_summary(&report.failed);
    }
    let Ok(report) = serde_json::from_str::<serde_json::Value>(stdout) else {
        return "stdout is not a supported JSON report".to_string();
    };
    let format = report.get("format").and_then(serde_json::Value::as_str).unwrap_or("unknown");
    let readiness = report
        .get("ready_class")
        .or_else(|| report.get("readiness"))
        .and_then(serde_json::Value::as_str)
        .unwrap_or("unknown");
    let blocker_identities = [
        "missing_records",
        "network_required_records",
        "stale_records",
        "untrusted_records",
    ]
    .iter()
    .filter_map(|field| report.get(field).and_then(serde_json::Value::as_array))
    .flatten()
    .filter_map(serde_json::Value::as_str)
    .take(NATIVE_FAILURE_IDENTITY_COUNT_MAX)
    .collect::<Vec<_>>()
    .join(",");
    if format == OFFLINE_PREFLIGHT_REPORT_FORMAT {
        return format!("format={format} readiness={readiness} blockers={blocker_identities}");
    }
    format!("format={format} readiness={readiness}")
}

fn build_failures_summary(failures: &[BuildJsonFailure]) -> String {
    failures
        .iter()
        .map(|failure| format!("{}:{}:{}:{}", failure.root, failure.phase, failure.error_class, failure.message))
        .collect::<Vec<_>>()
        .join("; ")
}

fn proof_error(message: String) -> RunError {
    RunError::Build(format!("source-built fixed-point blocked: {message}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    const DIGEST: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    const OTHER_DIGEST: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
    const RETAINED_TRANSITION_EXECUTION_ROOT_ENV: &str = "MANTLE_STAGE_X_TRANSITION_EXECUTION_ROOT";
    const RETAINED_TRANSITION_HANDOFF_ROOT_ENV: &str = "MANTLE_STAGE_X_TRANSITION_HANDOFF_ROOT";
    const RETAINED_TRANSITION_SOURCE_STATE_ENV: &str = "MANTLE_STAGE_X_TRANSITION_SOURCE_STATE";
    const OPEN_FILE_LIMIT_CHILD_ENV: &str = "MANTLE_TEST_OPEN_FILE_LIMIT_CHILD";
    const OPEN_FILE_LIMIT_TEST_MAX: u64 = 256;
    const VENDOR_FIXTURE_MANIFEST: &[u8] = b"[package]\nname = \"vendor-fixture\"\nversion = \"0.1.0\"\nedition = \"2024\"\n\n[lib]\npath = \"src/lib.rs\"\n";
    const VENDOR_FIXTURE_LIBRARY: &[u8] = b"pub fn fixture() {}\n";
    const VENDOR_FIXTURE_REVISION: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

    fn fixture_cargo_sha256(bytes: &[u8]) -> String {
        use sha2::Digest as _;

        data_encoding::HEXLOWER.encode(&sha2::Sha256::digest(bytes))
    }

    fn write_materialized_vendor_fixture(source_root: &Path) {
        let cargo_dir = source_root.join(".cargo");
        let package_dir = source_root.join(VENDOR_RELATIVE_PATH).join("vendor-fixture");
        fs::create_dir_all(package_dir.join("src")).unwrap();
        fs::create_dir_all(&cargo_dir).unwrap();
        fs::write(
            cargo_dir.join("vendor-config.toml"),
            format!(
                "[source.\"git+https://example.invalid/vendor-fixture?rev={VENDOR_FIXTURE_REVISION}\"]\nreplace-with = \"vendored-sources\"\n\n[source.vendored-sources]\ndirectory = \"vendor-deps\"\n"
            ),
        )
        .unwrap();
        fs::write(
            source_root.join("Cargo.lock"),
            format!(
                "version = 4\n\n[[package]]\nname = \"vendor-fixture\"\nversion = \"0.1.0\"\nsource = \"git+https://example.invalid/vendor-fixture?rev={VENDOR_FIXTURE_REVISION}#{VENDOR_FIXTURE_REVISION}\"\n"
            ),
        )
        .unwrap();
        fs::write(package_dir.join("Cargo.toml"), VENDOR_FIXTURE_MANIFEST).unwrap();
        fs::write(package_dir.join("src/lib.rs"), VENDOR_FIXTURE_LIBRARY).unwrap();
        let files = BTreeMap::from([
            ("Cargo.toml", fixture_cargo_sha256(VENDOR_FIXTURE_MANIFEST)),
            ("src/lib.rs", fixture_cargo_sha256(VENDOR_FIXTURE_LIBRARY)),
        ]);
        fs::write(
            package_dir.join(".cargo-checksum.json"),
            serde_json::to_vec(&serde_json::json!({ "files": files, "package": null })).unwrap(),
        )
        .unwrap();
        assert!(source_root.join("Cargo.lock").is_file());
        assert!(package_dir.join(".cargo-checksum.json").is_file());
    }

    fn write_stagex_transition_handoff_fixture(execution_root: &Path) {
        for relative in STAGEX_TRANSITION_HANDOFF_DIRECTORIES {
            fs::create_dir_all(execution_root.join(relative)).unwrap();
        }
        for relative in STAGEX_TRANSITION_HANDOFF_REQUIRED_FILES {
            let path = execution_root.join(relative);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, b"runtime-output").unwrap();
        }
        fs::write(execution_root.join(STAGEX_TRANSITION_PLAN_FILE), b"{}").unwrap();
        fs::write(execution_root.join(STAGEX_TRANSITION_REPORT_FILE), b"{}").unwrap();
        fs::write(execution_root.join(STAGEX_TRANSITION_AUDIT_FILE), b"[]").unwrap();
        assert!(execution_root.join(STAGEX_TRANSITION_REPORT_FILE).is_file());
        assert!(STAGEX_TRANSITION_HANDOFF_REQUIRED_FILES.iter().all(|path| execution_root.join(path).is_file()));
    }

    #[test]
    fn materialized_vendor_inputs_are_checked_before_construction() {
        let temp = tempfile::tempdir().unwrap();
        write_materialized_vendor_fixture(temp.path());

        validate_materialized_vendor_inputs(temp.path()).unwrap();
        fs::remove_dir_all(temp.path().join(VENDOR_RELATIVE_PATH).join("vendor-fixture")).unwrap();
        let error = validate_materialized_vendor_inputs(temp.path()).unwrap_err();

        assert!(error.to_string().contains("validating materialized vendored Cargo inputs"));
        assert!(error.to_string().contains("missing from vendor-deps"));
    }

    #[test]
    fn open_file_descriptor_limit_planning_is_bounded_and_fail_closed() {
        let lower = plan_open_file_descriptor_limit(128, 8_192, OPEN_FILE_LIMIT_TEST_MAX).unwrap();
        let exact = plan_open_file_descriptor_limit(OPEN_FILE_LIMIT_TEST_MAX, 8_192, OPEN_FILE_LIMIT_TEST_MAX).unwrap();
        let zero = plan_open_file_descriptor_limit(128, 8_192, 0).unwrap_err();
        let insufficient = plan_open_file_descriptor_limit(128, 128, OPEN_FILE_LIMIT_TEST_MAX).unwrap_err();
        let inverted =
            plan_open_file_descriptor_limit(512, OPEN_FILE_LIMIT_TEST_MAX, OPEN_FILE_LIMIT_TEST_MAX).unwrap_err();

        assert!(lower.update_required);
        assert!(!exact.update_required);
        assert_eq!(lower.soft_limit, OPEN_FILE_LIMIT_TEST_MAX);
        assert!(zero.contains("nonzero"));
        assert!(insufficient.contains("hard limit"));
        assert!(inverted.contains("soft limit"));
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn open_file_descriptor_limit_is_enforced_in_a_subprocess() {
        let executable = std::env::current_exe().unwrap();
        let output = Command::new(executable)
            .args([
                "--exact",
                "source_built_fixed_point_shell::tests::open_file_descriptor_limit_child",
                "--nocapture",
            ])
            .env(OPEN_FILE_LIMIT_CHILD_ENV, "1")
            .output()
            .unwrap();

        assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
        assert!(String::from_utf8_lossy(&output.stdout).contains("open-file-limit-child-ok"));
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn open_file_descriptor_limit_child() {
        if std::env::var_os(OPEN_FILE_LIMIT_CHILD_ENV).is_none() {
            return;
        }
        let (_, hard_limit) = read_open_file_descriptor_limits().unwrap();
        assert!(hard_limit >= OPEN_FILE_LIMIT_TEST_MAX);
        enforce_open_file_descriptor_limit(OPEN_FILE_LIMIT_TEST_MAX).unwrap();
        let (soft_limit, observed_hard_limit) = read_open_file_descriptor_limits().unwrap();

        assert_eq!(soft_limit, OPEN_FILE_LIMIT_TEST_MAX);
        assert_eq!(observed_hard_limit, hard_limit);
        println!("open-file-limit-child-ok");
    }

    #[test]
    fn provider_adoption_report_lists_only_stages_executed_in_this_attempt() {
        let cold = crunch_dev_resume_core::plan_resume(&crunch_dev_resume_core::ResumePlanInput {
            mode: crunch_dev_resume_core::ResumeRunMode::Dev,
            candidates: Vec::new(),
            observed_rejections: Vec::new(),
        });
        let report = provider_adoption_report_plan(&cold);

        assert!(report.restored_stages.is_empty());
        assert_eq!(report.executed_stages, vec![
            crunch_dev_resume_core::ResumeStage::FullSourceRustProvider,
            crunch_dev_resume_core::ResumeStage::MantleStage1,
            crunch_dev_resume_core::ResumeStage::MantleStage2,
        ]);
        assert_eq!(report.first_incomplete_stage, Some(crunch_dev_resume_core::ResumeStage::FullSourceRustProvider));
    }

    #[test]
    fn options_reject_existing_output_and_malformed_digests() {
        let temp = tempfile::tempdir().unwrap();
        let executable = temp.path().join("bwrap");
        fs::write(&executable, b"\x7fELFstatic-tool").unwrap();
        fs::set_permissions(&executable, fs::Permissions::from_mode(0o755)).unwrap();
        let output = temp.path().join("existing");
        fs::create_dir(&output).unwrap();
        let options = SourceBuiltFixedPointOptions {
            source_profile: &temp.path().join("profile.json"),
            expected_source_profile_blake3: DIGEST,
            expected_stagex_lineage_blake3: DIGEST,
            expected_native_provider_blake3: DIGEST,
            output_dir: &output,
            bwrap: &executable,
            sandbox_shell: &executable,
            jobs: MAX_JOBS_MIN,
            elapsed_seconds_max: 1,
            disk_bytes_max: 1,
            protected_exec_events_max: 1,
            source_records_max: 1,
            proof_checkpoint_store: None,
            proof_checkpoint_import_attempt: None,
            proof_native_checkpoint_attempt: None,
            dev_provider_cache: None,
            dev_resume: false,
            dev_fast_fail: false,
            verbose: false,
            json: false,
        };
        let existing = validate_options(&options).unwrap_err();
        let absent = temp.path().join("absent");
        let mut malformed = options.clone();
        malformed.output_dir = &absent;
        malformed.expected_native_provider_blake3 = "bad";
        let digest = validate_options(&malformed).unwrap_err();
        let native_attempt = temp.path().join("native-attempt");
        fs::create_dir(&native_attempt).unwrap();
        let mut unbound_native_reuse = options.clone();
        unbound_native_reuse.output_dir = &absent;
        unbound_native_reuse.proof_native_checkpoint_attempt = Some(&native_attempt);
        let native_reuse = validate_options(&unbound_native_reuse).unwrap_err();
        let mut unbound_dev_resume = options.clone();
        unbound_dev_resume.output_dir = &absent;
        unbound_dev_resume.dev_resume = true;
        let dev_resume = validate_options(&unbound_dev_resume).unwrap_err();

        assert!(existing.to_string().contains("must be absent"));
        assert!(digest.to_string().contains("expected native provider BLAKE3"));
        assert!(native_reuse.to_string().contains("requires a checkpoint store"));
        assert!(dev_resume.to_string().contains("requires --dev-provider-cache"));
    }

    #[test]
    fn native_build_command_is_strict_offline_and_substitution_free() {
        let temp = tempfile::tempdir().unwrap();
        let bwrap = temp.path().join("bwrap");
        let shell = temp.path().join("busybox");
        fs::write(&bwrap, b"\x7fELFbwrap").unwrap();
        fs::write(&shell, b"\x7fELFstatic-busybox").unwrap();
        let output = temp.path().join("proof");
        fs::create_dir(&output).unwrap();
        let options = SourceBuiltFixedPointOptions {
            source_profile: &temp.path().join("profile.json"),
            expected_source_profile_blake3: DIGEST,
            expected_stagex_lineage_blake3: DIGEST,
            expected_native_provider_blake3: DIGEST,
            output_dir: &temp.path().join("final"),
            bwrap: &bwrap,
            sandbox_shell: &shell,
            jobs: MAX_JOBS_MIN,
            elapsed_seconds_max: 1,
            disk_bytes_max: 1,
            protected_exec_events_max: 1,
            source_records_max: 1,
            proof_checkpoint_store: None,
            proof_checkpoint_import_attempt: None,
            proof_native_checkpoint_attempt: None,
            dev_provider_cache: None,
            dev_resume: false,
            dev_fast_fail: false,
            verbose: false,
            json: false,
        };
        let prepared = PreparedAttempt {
            final_dir: temp.path().join("final"),
            staging_dir: output.clone(),
            source_root: output.join("source"),
            stagex_seed: output.join("seed"),
            stagex_lineage: output.join("lineage"),
            stagex_source_bundle: output.join("stagex-source.json"),
            native_source_manifest: output.join("native.json"),
            rust_source_archive_dir: output.join("rust"),
            native_store_dir: output.join("store"),
            native_state_dir: output.join("state"),
            transcripts_dir: output.join("transcripts"),
            disk_available_bytes_before: options.disk_bytes_max,
            started_at: Instant::now(),
            plan: test_plan(),
        };
        let command =
            native_build_command(&options, &prepared, Path::new("/source/bootstrap/seed-full-toolchain.ncl")).unwrap();
        let args = command.get_args().map(OsString::from).collect::<Vec<_>>();

        assert!(args.iter().any(|argument| argument == "--offline-source-preflight"));
        assert!(args.iter().any(|argument| argument == "--no-substitute"));
        assert!(args.iter().any(|argument| argument == "--strict-hermetic"));
        assert!(!args.iter().any(|argument| argument == "--impure"));
    }

    #[test]
    fn isolated_exec_worker_returns_values_and_reports_invalid_completion() {
        let orchestrator_thread = std::thread::current().id();
        let (worker_thread, value) = run_in_isolated_exec_thread("positive worker", || {
            (std::thread::current().id(), STAGEX_TRANSITION_HANDOFF_DIRECTORY_COUNT)
        })
        .unwrap();
        let empty_name = run_in_isolated_exec_thread("", || ()).unwrap_err();
        let panic = run_in_isolated_exec_thread("panicking worker", || panic!("controlled worker panic")).unwrap_err();

        assert_ne!(worker_thread, orchestrator_thread);
        assert_eq!(value, STAGEX_TRANSITION_HANDOFF_DIRECTORY_COUNT);
        assert!(empty_name.to_string().contains("operation name is empty"));
        assert!(panic.to_string().contains("worker panicked"));
    }

    #[test]
    fn native_build_failure_summary_preserves_offline_blockers_and_rejects_malformed_output() {
        use std::os::unix::process::ExitStatusExt as _;

        const NATIVE_BUILD_FAILURE_EXIT_CODE: i32 = 1;
        let preflight = serde_json::json!({
            "format": OFFLINE_PREFLIGHT_REPORT_FORMAT,
            "ready_class": "network-required",
            "missing_records": [],
            "network_required_records": ["fixed-url-missing"],
            "stale_records": [],
            "untrusted_records": []
        })
        .to_string();
        let summary = native_build_failure_summary(&preflight);
        let malformed = native_build_failure_summary("not-json");
        let failed_output = Output {
            status: std::process::ExitStatus::from_raw(NATIVE_BUILD_FAILURE_EXIT_CODE << 8),
            stdout: preflight.into_bytes(),
            stderr: Vec::new(),
        };
        let parsed_failure = parse_build_report("native-provider", &failed_output, false).unwrap_err();

        assert_eq!(
            summary,
            "format=mantle-source-offline-preflight-v1 readiness=network-required blockers=fixed-url-missing"
        );
        assert_eq!(malformed, "stdout is not a supported JSON report");
        assert!(parsed_failure.to_string().contains("readiness=network-required"));
        assert!(parsed_failure.to_string().contains("fixed-url-missing"));
        assert_ne!(summary, malformed);
    }

    #[test]
    fn stagex_transition_handoff_projects_only_declared_runtime_outputs() {
        let temp = tempfile::tempdir().unwrap();
        let execution_root = temp.path().join("execution");
        let handoff_root = temp.path().join(STAGEX_TRANSITION_STORE_BASENAME);
        fs::create_dir(&execution_root).unwrap();
        write_stagex_transition_handoff_fixture(&execution_root);
        let excluded_dir = execution_root.join("binutils-stage/runtime/tools");
        fs::create_dir_all(&excluded_dir).unwrap();
        let excluded_link = excluded_dir.join("file");
        std::os::unix::fs::symlink("/proof-owned/stagex-configure-utility", &excluded_link).unwrap();
        let original_target = fs::read_link(&excluded_link).unwrap();

        materialize_stagex_transition_handoff(&execution_root, &handoff_root).unwrap();

        let report: serde_json::Value =
            serde_json::from_slice(&fs::read(handoff_root.join(STAGEX_TRANSITION_HANDOFF_REPORT_FILE)).unwrap())
                .unwrap();
        let observed_roots = fs::read_dir(&handoff_root)
            .unwrap()
            .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
            .collect::<std::collections::BTreeSet<_>>();
        let expected_roots = STAGEX_TRANSITION_HANDOFF_DIRECTORIES
            .iter()
            .map(|relative| relative.split('/').next().unwrap().to_string())
            .chain([STAGEX_TRANSITION_HANDOFF_REPORT_FILE.to_string()])
            .collect::<std::collections::BTreeSet<_>>();
        assert_eq!(report["copied_directories"], serde_json::json!(STAGEX_TRANSITION_HANDOFF_DIRECTORIES));
        assert_eq!(observed_roots, expected_roots);
        assert_eq!(fs::read_link(&excluded_link).unwrap(), original_target);
        assert!(!handoff_root.join("binutils-stage").exists());
    }

    #[test]
    fn dev_handoff_reuses_only_an_identical_persistent_tree() {
        let temp = tempfile::tempdir().unwrap();
        let execution_root = temp.path().join("execution");
        let handoff_root = temp.path().join(STAGEX_TRANSITION_STORE_BASENAME);
        let replay_root = temp.path().join(STAGEX_TRANSITION_HANDOFF_REPLAY_DIR);
        fs::create_dir(&execution_root).unwrap();
        write_stagex_transition_handoff_fixture(&execution_root);
        materialize_stagex_transition_handoff(&execution_root, &handoff_root).unwrap();

        materialize_or_validate_stagex_transition_handoff(true, &execution_root, &handoff_root, &replay_root).unwrap();
        assert!(handoff_root.is_dir());
        assert!(!replay_root.exists());

        fs::write(handoff_root.join(STAGEX_TRANSITION_HANDOFF_REQUIRED_FILES[0]), b"substituted-runtime-output")
            .unwrap();
        let error =
            materialize_or_validate_stagex_transition_handoff(true, &execution_root, &handoff_root, &replay_root)
                .unwrap_err();
        assert!(error.to_string().contains("StageX transition handoff replay tree mismatch"));
        assert!(replay_root.is_dir());
    }

    #[test]
    #[ignore = "requires retained completed StageX transition"]
    fn projects_and_imports_retained_stagex_transition_handoff() {
        let execution_root = PathBuf::from(std::env::var(RETAINED_TRANSITION_EXECUTION_ROOT_ENV).unwrap());
        let handoff_root = PathBuf::from(std::env::var(RETAINED_TRANSITION_HANDOFF_ROOT_ENV).unwrap());
        let state_dir = PathBuf::from(std::env::var(RETAINED_TRANSITION_SOURCE_STATE_ENV).unwrap());

        materialize_stagex_transition_handoff(&execution_root, &handoff_root).unwrap();
        let import = crate::source_bundle::import_constructed_store_path_source(
            STAGEX_TRANSITION_LOGICAL_PATH,
            &handoff_root,
            &state_dir,
            LOGICAL_STORE_PREFIX,
        )
        .unwrap();

        assert_eq!(import.imported_count, 1);
        assert!(import.pinned);
        assert!(!handoff_root.join("binutils-stage").exists());
        assert!(execution_root.join("binutils-stage").is_dir());
    }

    #[test]
    fn missing_stagex_transition_handoff_input_leaves_destination_absent() {
        let temp = tempfile::tempdir().unwrap();
        let execution_root = temp.path().join("execution");
        let handoff_root = temp.path().join(STAGEX_TRANSITION_STORE_BASENAME);
        fs::create_dir(&execution_root).unwrap();
        write_stagex_transition_handoff_fixture(&execution_root);
        fs::remove_dir_all(execution_root.join("m4-stage")).unwrap();

        let error = materialize_stagex_transition_handoff(&execution_root, &handoff_root).unwrap_err();

        assert!(error.to_string().contains("handoff directory is missing: m4-stage/runtime/output"));
        assert!(!handoff_root.exists());
    }

    #[test]
    fn stagex_provider_handoff_accepts_only_expected_normalized_identity() {
        validate_stagex_provider_normalized_identity(STAGEX_PROVIDER_EXPECTED_NORMALIZED_DIGEST).unwrap();
        let substituted = validate_stagex_provider_normalized_identity(DIGEST).unwrap_err();

        assert!(substituted.to_string().contains("normalized provider digest mismatch"));
        assert_ne!(STAGEX_PROVIDER_EXPECTED_NORMALIZED_DIGEST, DIGEST);
    }

    #[test]
    fn stagex_provider_cache_reuses_only_the_same_normalized_payload() {
        validate_reusable_stagex_provider_identity(
            STAGEX_PROVIDER_EXPECTED_NORMALIZED_DIGEST,
            STAGEX_PROVIDER_EXPECTED_NORMALIZED_DIGEST,
        )
        .unwrap();
        let mismatch =
            validate_reusable_stagex_provider_identity(STAGEX_PROVIDER_EXPECTED_NORMALIZED_DIGEST, DIGEST).unwrap_err();

        assert!(mismatch.to_string().contains("StageX provider replay payload mismatch"));
        assert_ne!(STAGEX_PROVIDER_EXPECTED_NORMALIZED_DIGEST, DIGEST);
    }

    #[test]
    fn stagex_native_handoff_uses_only_the_fresh_reserved_provider_path() {
        const INPUT: &str = include_str!("../bootstrap/stagex-provider-proof-input.ncl");
        const HANDOFF: &str = include_str!("../bootstrap/tcc-musl-native.ncl");

        assert!(INPUT.contains(STAGEX_PROVIDER_LOGICAL_PATH));
        assert!(HANDOFF.contains("stagex-provider-proof-input.ncl"));
        assert!(HANDOFF.contains("inputs = [stagex_provider]"));
        assert!(!HANDOFF.contains("seed-full-admitted.ncl"));
        assert!(!HANDOFF.contains("tinycc-mes.ncl"));
        assert!(!HANDOFF.contains("tcc-musl-selfhost.ncl"));
    }

    #[test]
    fn conventional_native_chain_is_cut_over_to_fresh_stagex_authority() {
        const TRANSITION_INPUT: &str = include_str!("../bootstrap/stagex-transition-proof-input.ncl");
        const GCC40: &str = include_str!("../bootstrap/gcc-4.0-native.ncl");
        const STAGE0: &str = include_str!("../bootstrap/stage0-posix.ncl");
        const MAKE: &str = include_str!("../bootstrap/make-tcc.ncl");
        const BASH: &str = include_str!("../bootstrap/bash-2.05b-tcc.ncl");
        const TCC_SELFHOST: &str = include_str!("../bootstrap/tcc-musl-selfhost.ncl");
        const TCC_V2: &str = include_str!("../bootstrap/tcc-musl-v2.ncl");
        const MUSL: &str = include_str!("../bootstrap/musl-1.1.24-native.ncl");
        const BINUTILS: &str = include_str!("../bootstrap/binutils-tcc.ncl");

        assert!(TRANSITION_INPUT.contains(STAGEX_TRANSITION_LOGICAL_PATH));
        assert!(GCC40.contains("stagex-provider-proof-input.ncl"));
        assert!(GCC40.contains("stagex-transition-proof-input.ncl"));
        assert!(!GCC40.contains("derivationFile \"tcc-musl-selfhost.ncl\""));
        assert!(!GCC40.contains("derivationFile \"tcc-musl-v2.ncl\""));
        assert!(!GCC40.contains("derivationFile \"musl-1.1.24-native.ncl\""));
        assert!(!GCC40.contains("derivationFile \"binutils-tcc.ncl\""));
        for adapter in [STAGE0, MAKE] {
            assert!(adapter.contains("stagex-transition-proof-input.ncl"));
            assert!(!adapter.contains("seed-legacy.ncl"));
        }
        assert!(BASH.contains("Build bash 2.05b using tinycc 0.9.27"));
        assert!(TCC_SELFHOST.contains("tcc-0.9.27-musl-selfhost"));
        assert!(TCC_V2.contains("Final tcc 0.9.27 rebuilt"));
        assert!(MUSL.contains("musl-1.1.24-native-candidate"));
        assert!(BINUTILS.contains("source-built TCC-era binutils matrix passed"));
        for transition_recipe in [BASH, TCC_SELFHOST, TCC_V2, MUSL, BINUTILS] {
            assert!(!transition_recipe.contains("stagex-provider-proof-input.ncl"));
        }
    }

    #[test]
    fn disk_capacity_accepts_sufficient_space_and_rejects_shortfall() {
        const AVAILABLE_BYTES: u64 = 2;
        const REQUIRED_BYTES: u64 = 1;
        validate_disk_capacity(AVAILABLE_BYTES, REQUIRED_BYTES).unwrap();
        let error = validate_disk_capacity(REQUIRED_BYTES, AVAILABLE_BYTES).unwrap_err();

        assert!(error.to_string().contains("disk preflight"));
        assert!(error.to_string().contains("only 1 bytes are available"));
    }

    #[test]
    fn runtime_bounds_reject_elapsed_timeout_before_disk_probe() {
        const ELAPSED_SECONDS_OVER_BOUND: u64 = 2;
        let temp = tempfile::tempdir().unwrap();
        let executable = write_executable(&temp.path().join("bwrap"));
        let source_profile = temp.path().join("profile.json");
        let output = temp.path().join("proof");
        let options = options_fixture(&source_profile, &output, &executable, None, false, false);
        let mut prepared = prepared_fixture(&temp, temp.path().join("proof.staging"));
        prepared.started_at = Instant::now().checked_sub(Duration::from_secs(ELAPSED_SECONDS_OVER_BOUND)).unwrap();

        let error = validate_runtime_bounds(&options, &prepared).unwrap_err();
        let message = error.to_string();

        assert!(message.contains("source-built fixed-point blocked"));
        assert!(message.contains("proof elapsed time"));
        assert!(message.contains("exceeds configured bound"));
    }

    #[test]
    fn preparation_failure_status_preserves_blocker_without_inventing_plan_identity() {
        let temp = tempfile::tempdir().unwrap();
        write_attempt_status(temp.path(), None, PROOF_STATUS_FAILED, Some("materialization failed")).unwrap();
        let value: serde_json::Value =
            serde_json::from_slice(&fs::read(temp.path().join(ATTEMPT_STATUS_FILE)).unwrap()).unwrap();

        assert_eq!(value["status"], PROOF_STATUS_FAILED);
        assert_eq!(value["blocker"], "materialization failed");
        assert!(value.get("plan_digest_blake3").is_none());
    }

    #[test]
    fn publication_rolls_bundle_back_when_alias_transaction_fails() {
        const TEST_DISK_AVAILABLE_BYTES: u64 = 1;
        let temp = tempfile::tempdir().unwrap();
        let staging = temp.path().join("proof.staging");
        let final_dir = temp.path().join("proof");
        fs::create_dir(&staging).unwrap();
        fs::create_dir(temp.path().join(LATEST_SOURCE_BUILT_ALIAS)).unwrap();
        let prepared = PreparedAttempt {
            final_dir: final_dir.clone(),
            staging_dir: staging.clone(),
            source_root: staging.join("source"),
            stagex_seed: staging.join("seed"),
            stagex_lineage: staging.join("lineage"),
            stagex_source_bundle: staging.join("stagex-source.json"),
            native_source_manifest: staging.join("native.json"),
            rust_source_archive_dir: staging.join("rust"),
            native_store_dir: staging.join("store"),
            native_state_dir: staging.join("state"),
            transcripts_dir: staging.join("transcripts"),
            disk_available_bytes_before: TEST_DISK_AVAILABLE_BYTES,
            started_at: Instant::now(),
            plan: test_plan(),
        };

        let error = publish_attempt(&prepared).unwrap_err();

        assert!(error.to_string().contains("not a symlink"));
        assert!(staging.is_dir());
        assert!(!final_dir.exists());
        assert!(!temp.path().join(LATEST_ALIAS).exists());
    }

    #[test]
    fn success_alias_transaction_updates_both_or_neither() {
        let temp = tempfile::tempdir().unwrap();
        let parent = temp.path();
        std::os::unix::fs::symlink("old-proof", parent.join(LATEST_ALIAS)).unwrap();
        fs::create_dir(parent.join(LATEST_SOURCE_BUILT_ALIAS)).unwrap();

        let error = update_success_aliases(
            parent,
            &[LATEST_ALIAS, LATEST_SOURCE_BUILT_ALIAS],
            std::ffi::OsStr::new("new-proof"),
        )
        .unwrap_err();

        assert!(error.to_string().contains("not a symlink"));
        assert_eq!(fs::read_link(parent.join(LATEST_ALIAS)).unwrap(), Path::new("old-proof"));
        assert!(parent.join(LATEST_SOURCE_BUILT_ALIAS).is_dir());
        fs::remove_dir(parent.join(LATEST_SOURCE_BUILT_ALIAS)).unwrap();
        update_success_aliases(parent, &[LATEST_ALIAS, LATEST_SOURCE_BUILT_ALIAS], std::ffi::OsStr::new("new-proof"))
            .unwrap();
        assert_eq!(fs::read_link(parent.join(LATEST_ALIAS)).unwrap(), Path::new("new-proof"));
        assert_eq!(fs::read_link(parent.join(LATEST_SOURCE_BUILT_ALIAS)).unwrap(), Path::new("new-proof"));
    }

    #[test]
    fn materialized_source_records_preserve_bound_union_and_allow_profile_bound_fetches() {
        let temp = tempfile::tempdir().unwrap();
        let native_payload = temp.path().join("native");
        let stagex_payload = temp.path().join("stagex");
        let extra_payload = temp.path().join("extra");
        let mismatched_native_payload = temp.path().join("mismatched-native");
        fs::write(&native_payload, b"native source").unwrap();
        fs::write(&stagex_payload, b"stagex source").unwrap();
        fs::write(&extra_payload, b"extra source").unwrap();
        fs::write(&mismatched_native_payload, b"mismatched native source").unwrap();
        let native = crate::source_bundle::plan_source_bundle(
            &[crate::source_bundle::SourceSpec {
                kind: crate::source_bundle::SourceRecordKind::FixedUrl,
                identity: "native".to_string(),
                path: native_payload,
                adapter: None,
            }],
            LOGICAL_STORE_PREFIX,
        )
        .unwrap();
        let stagex = crate::source_bundle::plan_source_bundle(
            &[crate::source_bundle::SourceSpec {
                kind: crate::source_bundle::SourceRecordKind::FixedUrl,
                identity: "stagex".to_string(),
                path: stagex_payload,
                adapter: None,
            }],
            LOGICAL_STORE_PREFIX,
        )
        .unwrap();
        let extra = crate::source_bundle::plan_source_bundle(
            &[crate::source_bundle::SourceSpec {
                kind: crate::source_bundle::SourceRecordKind::FixedUrl,
                identity: "extra".to_string(),
                path: extra_payload,
                adapter: None,
            }],
            LOGICAL_STORE_PREFIX,
        )
        .unwrap();
        let mismatched_native = crate::source_bundle::plan_source_bundle(
            &[crate::source_bundle::SourceSpec {
                kind: crate::source_bundle::SourceRecordKind::FixedUrl,
                identity: "native".to_string(),
                path: mismatched_native_payload,
                adapter: None,
            }],
            LOGICAL_STORE_PREFIX,
        )
        .unwrap();
        let exact = vec![&native.records[0], &stagex.records[0]];
        let mut native_with_constructed_authority = native.clone();
        let mut constructed_authority = native.records[0].clone();
        constructed_authority.kind = crate::source_bundle::SourceRecordKind::ToolchainSourceRoot;
        constructed_authority.identity = "fresh-stagex-provider".to_string();
        constructed_authority.files.clear();
        constructed_authority.payload_bytes = 0;
        native_with_constructed_authority.records.push(constructed_authority);

        validate_materialized_source_records(&native, &stagex, &exact).unwrap();
        validate_materialized_source_records(&native_with_constructed_authority, &stagex, &exact).unwrap();
        let native_raw_digest = hash_materialized_source(&temp.path().join("native")).unwrap();
        let native_input = source_input_from_record(
            "native",
            SourceAuthorityRole::NativeSourceBundle,
            &native.records[0],
            &native_raw_digest,
        );
        let missing_error = validate_materialized_source_records(&native, &stagex, &[&native.records[0]]).unwrap_err();
        validate_materialized_source_records(&native, &stagex, &[
            &native.records[0],
            &stagex.records[0],
            &extra.records[0],
        ])
        .unwrap();
        let constructed_error = validate_materialized_source_records(&native, &stagex, &[
            &native.records[0],
            &stagex.records[0],
            &native_with_constructed_authority.records[1],
        ])
        .unwrap_err();
        let mismatch_error = validate_materialized_source_records(&native, &stagex, &[
            &mismatched_native.records[0],
            &stagex.records[0],
        ])
        .unwrap_err();

        assert_eq!(native_input.digest_blake3, native_raw_digest);
        assert_ne!(native_input.digest_blake3, native.records[0].content_blake3);
        assert!(missing_error.to_string().contains("omits bound source identity"));
        assert!(constructed_error.to_string().contains("non-fetch source authority"));
        assert!(mismatch_error.to_string().contains("differs from its independently bound manifest"));
    }

    #[test]
    fn policy_digests_are_domain_separated_and_stable() {
        let closure = policy_digest(CLOSURE_POLICY_TEXT);
        let hermeticity = policy_digest(HERMETICITY_POLICY_TEXT);

        assert_eq!(closure, policy_digest(CLOSURE_POLICY_TEXT));
        assert_ne!(closure, hermeticity);
        assert_eq!(closure.len(), BLAKE3_HEX_LENGTH);
    }

    #[test]
    fn store_service_registration_adopts_real_provider_and_is_observable() {
        const STORE_PATH_HASH: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
        let temp = tempfile::tempdir().unwrap();
        let store_dir = temp.path().join("out");
        let state_dir = temp.path().join("state");
        fs::create_dir_all(&store_dir).unwrap();
        let basename = format!("{STORE_PATH_HASH}-native-provider-test");
        let provider = store_dir.join(&basename);
        fs::create_dir_all(&provider).unwrap();
        fs::write(provider.join("provider.txt"), b"verified-provider").unwrap();

        let logical = register_adopted_provider(&provider, &store_dir, &state_dir).unwrap();

        assert_eq!(logical, format!("/mantle/store/{basename}"));
        assert!(state_dir.join("pathinfo.redb").is_file());
        assert!(state_dir.join(NATIVE_STATE_DIR).is_dir() || state_dir.join("blobs").is_dir());
    }

    #[test]
    fn store_seed_is_disabled_without_cache_flag() {
        let temp = tempfile::tempdir().unwrap();
        let executable = write_executable(&temp.path().join("bwrap"));
        let output = temp.path().join("final-proof");
        let source_profile = temp.path().join("profile.json");
        let options = options_fixture(&source_profile, &output, &executable, None, false, false);
        let seeded =
            seed_dev_store_snapshot(&options, &test_plan(), &temp.path().join("store"), &temp.path().join("state"))
                .unwrap();

        assert!(!seeded);
    }

    #[test]
    fn stage_marker_write_read_and_reject_mutation() {
        let temp = tempfile::tempdir().unwrap();
        let prepared = prepared_fixture(&temp, temp.path().join("staged"));
        write_stage_marker(&prepared, "stagex-transition", DIGEST).unwrap();

        let marker = read_stage_marker(&prepared, "stagex-transition").unwrap().expect("marker written");
        let trusted =
            validate_stage_marker(Some(&marker), &prepared.plan.plan_digest_blake3, "stagex-transition", DIGEST);
        let mutated =
            validate_stage_marker(Some(&marker), &prepared.plan.plan_digest_blake3, "stagex-transition", OTHER_DIGEST);

        assert_eq!(trusted, StageMarkerValidation::Trusted);
        assert_eq!(mutated, StageMarkerValidation::Reject);
    }

    #[test]
    fn dev_cache_adoption_requires_validated_entry_and_present_subtrees() {
        let temp = tempfile::tempdir().unwrap();
        let cache = temp.path().join("cache");
        let plan = test_plan();
        let policies = DevCachePolicies::from_plan(&plan);
        let cache_key = dev_provider_cache_key(&plan.source_authority_digest_blake3, &policies);
        let entry_root = cache.join(DEV_CACHE_PROVIDERS_SUBDIR).join(&plan.plan_digest_blake3).join(&cache_key);
        let stagex = entry_root.join("aaa-stagex");
        let native = entry_root.join("aaa-native");
        fs::create_dir_all(&stagex).unwrap();
        fs::create_dir_all(&native).unwrap();
        let entry = crate::source_built_fixed_point_dev_cache::DevProviderCacheEntry {
            schema: crate::source_built_fixed_point_dev_cache::DEV_CACHE_SCHEMA.to_string(),
            plan_digest_blake3: plan.plan_digest_blake3.clone(),
            cache_key: cache_key.clone(),
            source_authority_digest_blake3: plan.source_authority_digest_blake3.clone(),
            closure_policy_digest_blake3: plan.policies.closure_policy_digest_blake3.clone(),
            hermeticity_policy_digest_blake3: plan.policies.hermeticity_policy_digest_blake3.clone(),
            protected_execution_policy_digest_blake3: plan.policies.protected_execution_policy_digest_blake3.clone(),
            effect_policy_digest_blake3: plan.policies.effect_policy_digest_blake3.clone(),
            normalization_policy_digest_blake3: plan.policies.normalization_policy_digest_blake3.clone(),
            stagex_provider: crate::source_built_fixed_point_dev_cache::DevProviderReference {
                logical_path: "/mantle/store/aaa-stagex".to_string(),
                store_basename: "aaa-stagex".to_string(),
                digest_blake3: DIGEST.to_string(),
            },
            native_provider: crate::source_built_fixed_point_dev_cache::DevProviderReference {
                logical_path: "/mantle/store/aaa-native".to_string(),
                store_basename: "aaa-native".to_string(),
                digest_blake3: DIGEST.to_string(),
            },
        };
        let entry_path = entry_root.join(DEV_CACHE_ENTRY_FILE);
        fs::write(&entry_path, serde_json::to_vec_pretty(&entry).unwrap()).unwrap();

        let executable = write_executable(&temp.path().join("bwrap"));
        let output = temp.path().join("final-proof");
        let source_profile = temp.path().join("profile.json");
        let options = options_fixture(&source_profile, &output, &executable, Some(&cache), false, false);
        let hit = dev_cache_adoption(&options, &plan).unwrap();
        let cold_cache = temp.path().join("other");
        let cold_options = options_fixture(&source_profile, &output, &executable, Some(&cold_cache), false, false);
        let cold = dev_cache_adoption(&cold_options, &test_plan()).unwrap();

        assert!(hit);
        assert!(!cold);
    }

    #[test]
    fn adopt_cached_provider_subtrees_copies_and_returns_observation() {
        use crate::source_built_fixed_point_dev_cache::DevProviderCacheEntry;
        use crate::source_built_fixed_point_dev_cache::DevProviderReference;

        let temp = tempfile::tempdir().unwrap();
        let cache = temp.path().join("cache");
        let plan = test_plan();
        let policies = DevCachePolicies::from_plan(&plan);
        let cache_key = dev_provider_cache_key(&plan.source_authority_digest_blake3, &policies);
        let entry_root = cache.join(DEV_CACHE_PROVIDERS_SUBDIR).join(&plan.plan_digest_blake3).join(&cache_key);
        let stagex = entry_root.join("aaa-stagex");
        let native = entry_root.join("aaa-native");
        fs::create_dir_all(&stagex).unwrap();
        fs::create_dir_all(&native).unwrap();
        fs::write(native.join("bin"), b"native-provider-data").unwrap();
        let entry = DevProviderCacheEntry {
            schema: crate::source_built_fixed_point_dev_cache::DEV_CACHE_SCHEMA.to_string(),
            plan_digest_blake3: plan.plan_digest_blake3.clone(),
            cache_key: cache_key.clone(),
            source_authority_digest_blake3: plan.source_authority_digest_blake3.clone(),
            closure_policy_digest_blake3: plan.policies.closure_policy_digest_blake3.clone(),
            hermeticity_policy_digest_blake3: plan.policies.hermeticity_policy_digest_blake3.clone(),
            protected_execution_policy_digest_blake3: plan.policies.protected_execution_policy_digest_blake3.clone(),
            effect_policy_digest_blake3: plan.policies.effect_policy_digest_blake3.clone(),
            normalization_policy_digest_blake3: plan.policies.normalization_policy_digest_blake3.clone(),
            stagex_provider: DevProviderReference {
                logical_path: STAGEX_PROVIDER_LOGICAL_PATH.to_string(),
                store_basename: "aaa-stagex".to_string(),
                digest_blake3: STAGEX_PROVIDER_EXPECTED_NORMALIZED_DIGEST.to_string(),
            },
            native_provider: DevProviderReference {
                logical_path: "/mantle/store/aaa-native".to_string(),
                store_basename: "aaa-native".to_string(),
                digest_blake3: DIGEST.to_string(),
            },
        };
        fs::write(entry_root.join(DEV_CACHE_ENTRY_FILE), serde_json::to_vec_pretty(&entry).unwrap()).unwrap();
        let executable = write_executable(&temp.path().join("bwrap"));
        let output = temp.path().join("final-proof");
        let source_profile = temp.path().join("profile.json");
        let options = options_fixture(&source_profile, &output, &executable, Some(&cache), false, false);
        let prepared = prepared_fixture(&temp, temp.path().join("staged"));

        let adopted = adopt_cached_provider_subtrees(&options, &prepared, &test_plan()).unwrap();

        assert!(prepared.native_store_dir.join("aaa-stagex").is_dir());
        assert!(prepared.native_store_dir.join("aaa-native").join("bin").is_file());
        assert!(adopted.output.path.ends_with("aaa-native"));
        assert!(adopted.transcript_digest_blake3.len() == BLAKE3_HEX_LENGTH);
    }

    #[test]
    fn dev_resume_report_separates_restored_executed_and_published_stages() {
        const PUBLISHED_IDENTITY: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
        let temp = tempfile::tempdir().unwrap();
        let prepared = prepared_fixture(&temp, temp.path().join("report-staging"));
        let plan = crunch_dev_resume_core::plan_resume(&crunch_dev_resume_core::ResumePlanInput {
            mode: crunch_dev_resume_core::ResumeRunMode::Dev,
            candidates: Vec::new(),
            observed_rejections: Vec::new(),
        });
        let path =
            dev_resume::write_dev_resume_report(&prepared, &plan, &[PUBLISHED_IDENTITY.to_string()], "cold-executed")
                .unwrap();
        let report: crunch_dev_resume_core::DevResumeReport = serde_json::from_slice(&fs::read(path).unwrap()).unwrap();

        assert!(report.restored_stages.is_empty());
        assert_eq!(report.executed_stages, crunch_dev_resume_core::ResumeStage::ALL);
        assert_eq!(report.published_bundle_identities_blake3, [PUBLISHED_IDENTITY]);
        assert!(!report.promoted_receipt_written);
        assert!(!report.release_alias_updated);
    }

    pub(super) fn prepared_fixture(temp: &tempfile::TempDir, staging: PathBuf) -> PreparedAttempt {
        fs::create_dir_all(&staging).unwrap();
        fs::create_dir_all(staging.join(NATIVE_STORE_DIR)).unwrap();
        fs::create_dir_all(staging.join(NATIVE_STATE_DIR)).unwrap();
        fs::create_dir_all(staging.join(TRANSCRIPTS_DIR)).unwrap();
        PreparedAttempt {
            final_dir: temp.path().join("final"),
            staging_dir: staging.clone(),
            source_root: staging.join("source"),
            stagex_seed: staging.join("seed"),
            stagex_lineage: staging.join("lineage"),
            stagex_source_bundle: staging.join("stagex-source.json"),
            native_source_manifest: staging.join("native.json"),
            rust_source_archive_dir: staging.join("rust"),
            native_store_dir: staging.join(NATIVE_STORE_DIR),
            native_state_dir: staging.join(NATIVE_STATE_DIR),
            transcripts_dir: staging.join(TRANSCRIPTS_DIR),
            disk_available_bytes_before: 1,
            started_at: Instant::now(),
            plan: test_plan(),
        }
    }

    pub(super) fn options_fixture<'a>(
        source_profile: &'a Path,
        output_dir: &'a Path,
        executable: &'a Path,
        cache: Option<&'a Path>,
        resume: bool,
        fast_fail: bool,
    ) -> SourceBuiltFixedPointOptions<'a> {
        SourceBuiltFixedPointOptions {
            source_profile,
            expected_source_profile_blake3: DIGEST,
            expected_stagex_lineage_blake3: DIGEST,
            expected_native_provider_blake3: DIGEST,
            output_dir,
            bwrap: executable,
            sandbox_shell: executable,
            jobs: MAX_JOBS_MIN,
            elapsed_seconds_max: 1,
            disk_bytes_max: 1,
            protected_exec_events_max: 1,
            source_records_max: 1,
            proof_checkpoint_store: None,
            proof_checkpoint_import_attempt: None,
            proof_native_checkpoint_attempt: None,
            dev_provider_cache: cache,
            dev_resume: resume,
            dev_fast_fail: fast_fail,
            verbose: false,
            json: false,
        }
    }

    #[test]
    fn last_published_source_digest_reads_latest_receipt_binding() {
        let temp = tempfile::tempdir().unwrap();
        let target = temp.path().join("proof-bundle");
        fs::create_dir(&target).unwrap();
        fs::write(
            target.join(FINAL_RECEIPT_FILE),
            serde_json::json!({
                "source_built_fixed_point": {
                    "source_authority_digest_blake3": DIGEST
                }
            })
            .to_string(),
        )
        .unwrap();
        std::os::unix::fs::symlink("proof-bundle", temp.path().join(LATEST_SOURCE_BUILT_ALIAS)).unwrap();
        let executable = write_executable(&temp.path().join("bwrap"));
        let out = temp.path().join("out");
        let source_profile = temp.path().join("profile.json");
        let no_alias = options_fixture(&source_profile, &out, &executable, None, false, false);

        let matched = last_published_source_digest(&no_alias).unwrap();
        assert_eq!(matched.as_deref(), Some(DIGEST));
        std::fs::remove_file(temp.path().join(LATEST_SOURCE_BUILT_ALIAS)).unwrap();
        let absent = last_published_source_digest(&no_alias).unwrap();
        assert_eq!(absent, None);
    }

    // r[verify bootstrap_inventory.source_built_mantle_checkpoint_reuse]
    #[test]
    fn imported_provider_authority_accepts_legacy_plan_only_with_matching_recipe_projection() {
        let temp = tempfile::tempdir().unwrap();
        let source_root = temp.path().join(INPUTS_DIR).join(SOURCE_ROOT_DIR);
        for root in PROVIDER_RECIPE_PROJECTION_ROOTS {
            fs::create_dir_all(source_root.join(root)).unwrap();
            fs::write(source_root.join(root).join("input"), root.as_bytes()).unwrap();
        }
        let (projection_bytes, projection_digest) = hash_provider_recipe_projection(&source_root).unwrap();
        let mut current = test_plan();
        let projection = current
            .source_inputs
            .iter_mut()
            .find(|input| input.role == SourceAuthorityRole::ProviderRecipeProjection)
            .unwrap();
        projection.size_bytes = projection_bytes;
        projection.digest_blake3 = projection_digest;
        let mut legacy = current.clone();
        legacy.schema = LEGACY_PROVIDER_CHECKPOINT_IMPORT_PLAN_SCHEMA.to_string();
        legacy.source_inputs.retain(|input| input.role != SourceAuthorityRole::ProviderRecipeProjection);
        for stage in legacy
            .stages
            .iter_mut()
            .take(crate::source_built_fixed_point_checkpoint::PROVIDER_CHECKPOINT_STAGE_COUNT)
        {
            stage.inputs.retain(|input| {
                !matches!(input, crate::source_built_fixed_point::StageAuthorityInput::Source {
                    role: SourceAuthorityRole::ProviderRecipeProjection
                })
            });
        }

        validate_imported_provider_authority(&current, &legacy, temp.path()).unwrap();
        legacy
            .source_inputs
            .iter_mut()
            .find(|input| input.role == SourceAuthorityRole::RustSourceArchiveSet)
            .unwrap()
            .digest_blake3 = OTHER_DIGEST.to_string();
        let mismatch = validate_imported_provider_authority(&current, &legacy, temp.path()).unwrap_err();

        assert!(mismatch.to_string().contains("RustSourceArchiveSet"));
    }

    // r[verify bootstrap_inventory.source_built_mantle_checkpoint_reuse]
    #[test]
    fn checkpoint_import_rejects_running_attempt_status() {
        let temp = tempfile::tempdir().unwrap();
        let plan = test_plan();
        fs::write(temp.path().join(PLAN_FILE), serde_json::to_vec_pretty(&plan).unwrap()).unwrap();
        fs::write(
            temp.path().join(ATTEMPT_STATUS_FILE),
            serde_json::to_vec_pretty(&serde_json::json!({
                "schema": PROOF_STATUS_SCHEMA,
                "status": PROOF_STATUS_RUNNING,
                "plan_digest_blake3": plan.plan_digest_blake3,
                "blocker": null
            }))
            .unwrap(),
        )
        .unwrap();

        let error = validate_imported_attempt_status(temp.path(), &plan).unwrap_err();

        assert!(error.to_string().contains("running proof attempt"));
        assert!(!error.to_string().contains("unsupported status"));
    }

    fn write_executable(path: &Path) -> PathBuf {
        fs::write(path, b"\x7fELFstatic-tool").unwrap();
        fs::set_permissions(path, fs::Permissions::from_mode(0o755)).unwrap();
        path.to_path_buf()
    }

    fn test_plan() -> SourceBuiltFixedPointPlan {
        test_plan_with_disk_bytes(1)
    }

    pub(super) fn test_plan_with_disk_bytes(disk_bytes_max: u64) -> SourceBuiltFixedPointPlan {
        let source_inputs = [
            ("seed", SourceAuthorityRole::StagexSeed, SourceContentKind::RegularFile, 'a'),
            ("lineage", SourceAuthorityRole::StagexLineage, SourceContentKind::RegularFile, 'b'),
            ("stagex-source", SourceAuthorityRole::StagexSourceBundle, SourceContentKind::RegularFile, 'c'),
            ("native", SourceAuthorityRole::NativeSourceBundle, SourceContentKind::RegularFile, 'd'),
            ("rust", SourceAuthorityRole::RustSourceArchiveSet, SourceContentKind::Directory, 'e'),
            ("provider-recipes", SourceAuthorityRole::ProviderRecipeProjection, SourceContentKind::Directory, '8'),
            ("source", SourceAuthorityRole::MantleSource, SourceContentKind::Directory, 'f'),
            ("vendor", SourceAuthorityRole::VendorInputs, SourceContentKind::Directory, '7'),
        ]
        .into_iter()
        .map(|(id, role, kind, character)| SourceAuthorityInput {
            id: id.to_string(),
            role,
            kind,
            digest_blake3: character.to_string().repeat(BLAKE3_HEX_LENGTH),
            size_bytes: 1,
        })
        .collect();
        plan_source_built_fixed_point(SourceBuiltFixedPointPlanInput {
            proof_id: "shell-test".to_string(),
            logical_store_prefix: LOGICAL_STORE_PREFIX.to_string(),
            source_inputs,
            initial_output_authority: InitialOutputAuthorityState {
                stagex_transition_entries: 0,
                native_provider_entries: 0,
                rust_provider_entries: 0,
                mantle_output_entries: 0,
            },
            policies: SourceBuiltFixedPointPolicies {
                expected_native_provider_digest_blake3: DIGEST.to_string(),
                closure_policy_digest_blake3: DIGEST.to_string(),
                hermeticity_policy_digest_blake3: DIGEST.to_string(),
                protected_execution_policy_digest_blake3: DIGEST.to_string(),
                effect_policy_digest_blake3: DIGEST.to_string(),
                normalization_policy_digest_blake3: DIGEST.to_string(),
                hermeticity_mode: ProofHermeticityMode::Strict,
                live_fetch_allowed: false,
                cargo_invocation_allowed: false,
                ambient_discovery_allowed: false,
                fallback_allowed: false,
                provider_cache_completion_allowed: false,
            },
            resource_bounds: SourceBuiltFixedPointResourceBounds {
                elapsed_seconds_max: 1,
                disk_bytes_max,
                open_file_descriptors_max: OPEN_FILE_LIMIT_TEST_MAX,
                protected_exec_events_max: 1,
                source_records_max: 1,
            },
        })
        .unwrap()
    }
}
