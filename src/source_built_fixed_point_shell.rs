use std::collections::BTreeMap;
#[cfg(test)]
use std::ffi::OsString;
use std::fs;
use std::io::Write as _;
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
use crate::source_built_fixed_point::InitialOutputAuthorityState;
use crate::source_built_fixed_point::ProofHermeticityMode;
use crate::source_built_fixed_point::SourceAuthorityInput;
use crate::source_built_fixed_point::SourceAuthorityRole;
use crate::source_built_fixed_point::SourceBuiltFixedPointPlan;
use crate::source_built_fixed_point::SourceBuiltFixedPointPlanInput;
use crate::source_built_fixed_point::SourceBuiltFixedPointPolicies;
use crate::source_built_fixed_point::SourceBuiltFixedPointResourceBounds;
use crate::source_built_fixed_point::SourceContentKind;
use crate::source_built_fixed_point::plan_source_built_fixed_point;
use crate::source_bundle::SourceBuiltFixedPointProfileRecords;
use crate::source_bundle::SourceRecord;
use crate::source_bundle::assemble_source_bundle;
use crate::source_bundle::materialize_source_record_payload;
use crate::source_bundle::source_built_fixed_point_profile_records;
use crate::stagex_provider::StagexProviderRequest;
use crate::stagex_transition::StagexTransitionRequest;

const LOGICAL_STORE_PREFIX: &str = "/mantle/store";
const PROFILE_SCHEMA: &str = "mantle-source-bundle-v1";
const BUILD_REPORT_SCHEMA: &str = "crunch-build-report-v1";
const OFFLINE_PREFLIGHT_REPORT_FORMAT: &str = "mantle-source-offline-preflight-v1";
const NATIVE_FAILURE_IDENTITY_COUNT_MAX: usize = 8;
const _: () = assert!(NATIVE_FAILURE_IDENTITY_COUNT_MAX > 0);
const PROOF_STATUS_SCHEMA: &str = "mantle-source-built-fixed-point-attempt-v1";
const PROOF_STATUS_RUNNING: &str = "running";
const PROOF_STATUS_FAILED: &str = "failed";
const PROOF_STATUS_COMPLETE: &str = "complete";
const STAGEX_TRANSITION_EXECUTION_DIR: &str = "stagex-transition-execution";
pub(crate) const STAGEX_TRANSITION_REPORT_FILE: &str = "transition-report.json";
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
const EXPECTED_SINGLE_OUTPUT_COUNT: usize = 1;
const EXPECTED_STAGE_COUNT: usize = 6;
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
    pub(crate) verbose: bool,
    pub(crate) json: bool,
}

struct MaterializedSourceDigests {
    stagex_seed: String,
    stagex_source_bundle: String,
    native_source_manifest: String,
    rust_source_archive_set: String,
    mantle_source: String,
    vendor_inputs: String,
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
struct BuildJsonReport {
    schema: String,
    hermeticity_mode: String,
    hermeticity_audit_events: Vec<serde_json::Value>,
    outcomes: Vec<BuildJsonOutcome>,
    failed: Vec<BuildJsonFailure>,
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
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct StagexTransitionHandoffReport {
    format: &'static str,
    copied_directories: Vec<&'static str>,
    non_claim: &'static str,
}

#[derive(Debug)]
pub(crate) struct ConstructedProviders {
    pub(crate) stagex_transition_execution_dir: PathBuf,
    pub(crate) stagex_provider_report: crate::stagex_provider::StagexProviderPublicationReport,
    pub(crate) native_provider: BuildObservation,
    pub(crate) native_admission: crate::full_source_provider::FullSourceProviderAdmissionReport,
    pub(crate) native_admission_report_path: PathBuf,
    pub(crate) rust_provider: crate::rust_source_provider::RustSourceProviderMaterialization,
    pub(crate) toolchain_closure_path: PathBuf,
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
    let result = run_attempt(&options, &prepared);
    if let Err(error) = result {
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
    write_attempt_status(&prepared.staging_dir, Some(&prepared.plan.plan_digest_blake3), PROOF_STATUS_COMPLETE, None)?;
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
    if !(MAX_JOBS_MIN..=MAX_JOBS_MAX).contains(&options.jobs) {
        return Err(proof_error(format!("jobs must be within {MAX_JOBS_MIN}..={MAX_JOBS_MAX}, got {}", options.jobs)));
    }
    assert!(options.elapsed_seconds_max > 0);
    assert!(options.disk_bytes_max > 0);
    validate_disk_preflight(options.output_dir, options.disk_bytes_max)
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
    let vendor_root = source_root.join(VENDOR_RELATIVE_PATH);
    materialize_source_record_payload(records.vendor_inputs, &vendor_root)?;
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
        mantle_source: mantle_source_digest,
        vendor_inputs: vendor_inputs_digest,
    };
    let plan = prepare_plan(options, &records, &source_digests, &native_source_authority_manifest)?;
    let plan_path = staging_dir.join(PLAN_FILE);
    write_json_create_new(&plan_path, &plan)?;
    let native_store_dir = staging_dir.join(NATIVE_STORE_DIR);
    let native_state_dir = staging_dir.join(NATIVE_STATE_DIR);
    let transcripts_dir = staging_dir.join(TRANSCRIPTS_DIR);
    fs::create_dir(&native_store_dir)
        .map_err(|error| proof_error(format!("creating native store {}: {error}", native_store_dir.display())))?;
    fs::create_dir(&native_state_dir)
        .map_err(|error| proof_error(format!("creating native state {}: {error}", native_state_dir.display())))?;
    fs::create_dir(&transcripts_dir)
        .map_err(|error| proof_error(format!("creating transcripts {}: {error}", transcripts_dir.display())))?;
    crate::source_bundle::import_source_bundle(&profile, &native_state_dir, true)?;
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
        if profile_by_identity.insert(record.identity.as_str(), *record).is_some() {
            return Err(proof_error(format!(
                "materialized source profile repeats source identity {}",
                record.identity
            )));
        }
    }
    if profile_by_identity.len() != expected_by_identity.len() {
        return Err(proof_error(format!(
            "materialized source record set differs from the exact native and StageX union: expected={}, profile={}",
            expected_by_identity.len(),
            profile_by_identity.len()
        )));
    }
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

fn run_attempt(options: &SourceBuiltFixedPointOptions<'_>, prepared: &PreparedAttempt) -> Result<(), RunError> {
    validate_runtime_bounds(options, prepared)?;
    let stagex_transition_execution_dir = prepared.staging_dir.join(STAGEX_TRANSITION_EXECUTION_DIR);
    let transition_result = run_in_isolated_exec_thread("StageX transition", || {
        crate::stagex_transition::materialize_protected_transition(StagexTransitionRequest {
            seed_path: &prepared.stagex_seed,
            hex0_source_path: &prepared.source_root.join("bootstrap/seeds/AMD64/hex0_AMD64.hex0"),
            kaem_source_path: &prepared.source_root.join("bootstrap/seeds/AMD64/kaem-minimal.hex0"),
            lineage_manifest_path: &prepared.stagex_lineage,
            source_bundle_path: Some(&prepared.stagex_source_bundle),
            stage0_answers_path: Some(&prepared.source_root.join("bootstrap/stage0-amd64.answers")),
            scratch_dir: &stagex_transition_execution_dir,
        })
    })?;
    let transition_report =
        transition_result.map_err(|error| proof_error(format!("StageX transition failed: {error}")))?;
    if transition_report.status != PROOF_STATUS_COMPLETE {
        return Err(proof_error(format!(
            "StageX transition status must be complete, got {}",
            transition_report.status
        )));
    }
    let transition_event_count = u32::try_from(transition_report.protected_exec_events.len())
        .map_err(|_| proof_error("StageX protected-exec event count exceeds u32".to_string()))?;
    if transition_event_count > options.protected_exec_events_max {
        return Err(proof_error(format!(
            "StageX protected-exec event count {transition_event_count} exceeds configured bound {}",
            options.protected_exec_events_max
        )));
    }
    drop(transition_report);
    validate_runtime_bounds(options, prepared)?;
    let transition_report_path = stagex_transition_execution_dir.join(STAGEX_TRANSITION_REPORT_FILE);
    crate::protected_exec::blake3_file_hex(&transition_report_path)
        .map_err(|error| proof_error(format!("hashing fresh StageX transition report: {error}")))?;

    let stagex_transition_root = prepared.native_store_dir.join(STAGEX_TRANSITION_STORE_BASENAME);
    materialize_stagex_transition_handoff(&stagex_transition_execution_dir, &stagex_transition_root)?;
    let transition_logical_path = crate::full_source_provider::adopt_verified_local_provider_path_strict(
        &stagex_transition_root,
        &prepared.native_store_dir,
        &prepared.native_state_dir,
        LOGICAL_STORE_PREFIX,
    )?;
    if transition_logical_path != STAGEX_TRANSITION_LOGICAL_PATH {
        return Err(proof_error(format!(
            "fresh StageX transition logical path mismatch: expected {STAGEX_TRANSITION_LOGICAL_PATH}, observed {transition_logical_path}"
        )));
    }
    crate::source_bundle::import_constructed_store_path_source(
        &transition_logical_path,
        &stagex_transition_root,
        &prepared.native_state_dir,
        LOGICAL_STORE_PREFIX,
    )?;

    let stagex_provider_root = prepared.native_store_dir.join(STAGEX_PROVIDER_STORE_BASENAME);
    let stagex_provider_result = run_in_isolated_exec_thread("StageX provider publication", || {
        crate::stagex_provider::materialize_stagex_provider(StagexProviderRequest {
            lineage_manifest_path: &prepared.stagex_lineage,
            transition_root: &stagex_transition_execution_dir,
            output_path: &stagex_provider_root,
        })
    })?;
    let stagex_provider_report =
        stagex_provider_result.map_err(|error| proof_error(format!("StageX provider publication failed: {error}")))?;
    validate_stagex_provider_normalized_identity(&stagex_provider_report.normalized_provider_digest_blake3)?;
    validate_runtime_bounds(options, prepared)?;
    let stagex_logical_path = crate::full_source_provider::adopt_verified_local_provider_path_strict(
        &stagex_provider_root,
        &prepared.native_store_dir,
        &prepared.native_state_dir,
        LOGICAL_STORE_PREFIX,
    )?;
    if stagex_logical_path != STAGEX_PROVIDER_LOGICAL_PATH {
        return Err(proof_error(format!(
            "fresh StageX provider logical path mismatch: expected {STAGEX_PROVIDER_LOGICAL_PATH}, observed {stagex_logical_path}"
        )));
    }
    crate::source_bundle::import_constructed_store_path_source(
        &stagex_logical_path,
        &stagex_provider_root,
        &prepared.native_state_dir,
        LOGICAL_STORE_PREFIX,
    )?;
    let providers =
        construct_full_source_providers(options, prepared, stagex_transition_execution_dir, stagex_provider_report)?;
    validate_runtime_bounds(options, prepared)?;
    run_cargo_free_fixed_point(options, prepared, &providers)?;
    validate_runtime_bounds(options, prepared)?;
    crate::source_built_fixed_point_receipt::write_source_built_fixed_point_receipt(
        &prepared.staging_dir,
        &prepared.plan,
        &providers,
    )?;
    validate_runtime_bounds(options, prepared)
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
    for relative in [STAGEX_TRANSITION_REPORT_FILE, STAGEX_TRANSITION_AUDIT_FILE] {
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

fn construct_full_source_providers(
    options: &SourceBuiltFixedPointOptions<'_>,
    prepared: &PreparedAttempt,
    stagex_transition_execution_dir: PathBuf,
    stagex_provider_report: crate::stagex_provider::StagexProviderPublicationReport,
) -> Result<ConstructedProviders, RunError> {
    let native_provider = run_native_build(options, prepared, NATIVE_PROVIDER_ID, NATIVE_PROVIDER_NCL)?;
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
    let rust_provider_root = prepared.staging_dir.join(RUST_PROVIDER_DIR);
    let rust_scratch = prepared.staging_dir.join(RUST_PROVIDER_SCRATCH_DIR);
    fs::create_dir(&rust_scratch)
        .map_err(|error| proof_error(format!("creating Rust provider scratch {}: {error}", rust_scratch.display())))?;
    let rust_provider = crate::rust_source_provider::materialize_full_source_bound_rust_provider_with_route_plan(
        &prepared.source_root.join(RUST_RECIPE_NCL),
        Some(&prepared.source_root.join(RUST_ROUTE_PLAN_NCL)),
        &native_admission_report_path,
        &host_tool_manifest,
        &prepared.rust_source_archive_dir,
        &rust_provider_root,
        &rust_scratch,
        options.verbose,
    )
    .map_err(|error| proof_error(format!("constructing full-source Rust provider: {error}")))?;
    let toolchain_closure_path = prepared.staging_dir.join(TOOLCHAIN_CLOSURE_FILE);
    crate::native_toolchain_closure::cmd_materialize_native_toolchain_closure(NativeToolchainClosureOptions {
        rust_source_provider: &rust_provider.output_path,
        host_root: &native_provider.output.path,
        target_root: &native_provider.output.path,
        output: &toolchain_closure_path,
    })?;
    Ok(ConstructedProviders {
        stagex_transition_execution_dir,
        stagex_provider_report,
        native_provider,
        native_admission,
        native_admission_report_path,
        rust_provider,
        toolchain_closure_path,
    })
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
    let mut command = native_build_command(options, prepared, &ncl_path)?;
    let output = command.output().map_err(|error| proof_error(format!("launching native build {label}: {error}")))?;
    validate_runtime_bounds(options, prepared)?;
    let stdout_path = prepared.transcripts_dir.join(format!("{label}.json"));
    let stderr_path = prepared.transcripts_dir.join(format!("{label}.stderr.txt"));
    write_bytes_create_new(&stdout_path, &output.stdout)?;
    write_bytes_create_new(&stderr_path, &output.stderr)?;
    let report = parse_build_report(label, &output)?;
    let build_output = require_single_build_output(label, report)?;
    let transcript_digest_blake3 = crate::protected_exec::blake3_file_hex(&stdout_path)
        .map_err(|error| proof_error(format!("hashing build transcript {}: {error}", stdout_path.display())))?;
    Ok(BuildObservation {
        output: build_output,
        transcript_path: stdout_path,
        transcript_digest_blake3,
    })
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

fn parse_build_report(label: &str, output: &Output) -> Result<BuildJsonReport, RunError> {
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
    if report.outcomes.iter().any(|outcome| outcome.cached) {
        return Err(proof_error(format!("native build {label} report contains a cache-hit outcome")));
    }
    debug_assert!(report.failed.is_empty());
    Ok(report)
}

fn require_single_build_output(label: &str, report: BuildJsonReport) -> Result<BuildJsonOutput, RunError> {
    if report.outcomes.len() != EXPECTED_SINGLE_OUTPUT_COUNT {
        return Err(proof_error(format!(
            "native build {label} must have one root outcome, got {}",
            report.outcomes.len()
        )));
    }
    let mut outcomes = report.outcomes.into_iter();
    let outcome = outcomes.next().expect("validated one outcome");
    if outcome.cached {
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
) -> Result<(), RunError> {
    let target = TARGET_TRIPLE.to_string();
    let fixed_point_dir = prepared.staging_dir.join(FIXED_POINT_DIR);
    crate::cargo_free_self_build::cmd_cargo_free_fixed_point_self_build(CargoFreeSelfBuildOptions {
        root: &prepared.source_root,
        out_dir: &fixed_point_dir,
        rustc: &providers.rust_provider.output_path.join(RUSTC_RELATIVE_PATH),
        targets: &[target],
        toolchain_closure: Some(&providers.toolchain_closure_path),
        rust_source_provider: Some(&providers.rust_provider.output_path),
        hermeticity_mode: crunch_pipeline::HermeticityMode::Strict,
        json: options.json,
    })?;
    let meta_path = fixed_point_dir.join("meta.json");
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
    debug_assert_eq!(providers.native_admission.output_digest_blake3, options.expected_native_provider_blake3);
    Ok(())
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
    const RETAINED_TRANSITION_EXECUTION_ROOT_ENV: &str = "MANTLE_STAGE_X_TRANSITION_EXECUTION_ROOT";
    const RETAINED_TRANSITION_HANDOFF_ROOT_ENV: &str = "MANTLE_STAGE_X_TRANSITION_HANDOFF_ROOT";
    const RETAINED_TRANSITION_SOURCE_STATE_ENV: &str = "MANTLE_STAGE_X_TRANSITION_SOURCE_STATE";

    fn write_stagex_transition_handoff_fixture(execution_root: &Path) {
        for relative in STAGEX_TRANSITION_HANDOFF_DIRECTORIES {
            fs::create_dir_all(execution_root.join(relative)).unwrap();
        }
        for relative in STAGEX_TRANSITION_HANDOFF_REQUIRED_FILES {
            let path = execution_root.join(relative);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, b"runtime-output").unwrap();
        }
        fs::write(execution_root.join(STAGEX_TRANSITION_REPORT_FILE), b"{}").unwrap();
        fs::write(execution_root.join(STAGEX_TRANSITION_AUDIT_FILE), b"[]").unwrap();
        assert!(execution_root.join(STAGEX_TRANSITION_REPORT_FILE).is_file());
        assert!(STAGEX_TRANSITION_HANDOFF_REQUIRED_FILES.iter().all(|path| execution_root.join(path).is_file()));
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
            verbose: false,
            json: false,
        };
        let existing = validate_options(&options).unwrap_err();
        let absent = temp.path().join("absent");
        let mut malformed = options.clone();
        malformed.output_dir = &absent;
        malformed.expected_native_provider_blake3 = "bad";
        let digest = validate_options(&malformed).unwrap_err();

        assert!(existing.to_string().contains("must be absent"));
        assert!(digest.to_string().contains("expected native provider BLAKE3"));
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
        let parsed_failure = parse_build_report("native-provider", &failed_output).unwrap_err();

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
    fn materialized_source_records_match_exact_native_and_stagex_union() {
        let temp = tempfile::tempdir().unwrap();
        let native_payload = temp.path().join("native");
        let stagex_payload = temp.path().join("stagex");
        let extra_payload = temp.path().join("extra");
        fs::write(&native_payload, b"native source").unwrap();
        fs::write(&stagex_payload, b"stagex source").unwrap();
        fs::write(&extra_payload, b"extra source").unwrap();
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
        let extra_error = validate_materialized_source_records(&native, &stagex, &[
            &native.records[0],
            &stagex.records[0],
            &extra.records[0],
        ])
        .unwrap_err();

        assert_eq!(native_input.digest_blake3, native_raw_digest);
        assert_ne!(native_input.digest_blake3, native.records[0].content_blake3);
        assert!(missing_error.to_string().contains("exact native and StageX union"));
        assert!(extra_error.to_string().contains("exact native and StageX union"));
    }

    #[test]
    fn policy_digests_are_domain_separated_and_stable() {
        let closure = policy_digest(CLOSURE_POLICY_TEXT);
        let hermeticity = policy_digest(HERMETICITY_POLICY_TEXT);

        assert_eq!(closure, policy_digest(CLOSURE_POLICY_TEXT));
        assert_ne!(closure, hermeticity);
        assert_eq!(closure.len(), BLAKE3_HEX_LENGTH);
    }

    fn test_plan() -> SourceBuiltFixedPointPlan {
        let source_inputs = [
            ("seed", SourceAuthorityRole::StagexSeed, SourceContentKind::RegularFile, 'a'),
            ("lineage", SourceAuthorityRole::StagexLineage, SourceContentKind::RegularFile, 'b'),
            ("stagex-source", SourceAuthorityRole::StagexSourceBundle, SourceContentKind::RegularFile, 'c'),
            ("native", SourceAuthorityRole::NativeSourceBundle, SourceContentKind::RegularFile, 'd'),
            ("rust", SourceAuthorityRole::RustSourceArchiveSet, SourceContentKind::Directory, 'e'),
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
                disk_bytes_max: 1,
                protected_exec_events_max: 1,
                source_records_max: 1,
            },
        })
        .unwrap()
    }
}
