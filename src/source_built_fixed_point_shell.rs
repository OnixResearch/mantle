use std::collections::BTreeMap;
#[cfg(test)]
use std::ffi::OsString;
use std::fs;
use std::io::Write as _;
use std::os::unix::fs::OpenOptionsExt as _;
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
use crate::source_built_fixed_point::ProofHermeticityMode;
use crate::source_built_fixed_point::ProofStageKind;
use crate::source_built_fixed_point::SOURCE_BUILT_FIXED_POINT_OPEN_FILE_DESCRIPTORS_MAX;
use crate::source_built_fixed_point::SourceAuthorityInput;
use crate::source_built_fixed_point::SourceAuthorityRole;
use crate::source_built_fixed_point::SourceBuiltFixedPointPlan;
use crate::source_built_fixed_point::SourceBuiltFixedPointPolicies;
use crate::source_built_fixed_point::SourceBuiltFixedPointResourceBounds;
use crate::source_built_fixed_point::SourceContentKind;
use crate::source_built_fixed_point::StageAuthorityInput;
use crate::source_built_fixed_point_action_trust::ActionExecutionEvent;
use crate::source_built_fixed_point_action_trust::ActionReconciliation;
use crate::source_built_fixed_point_action_trust::ActionTrustPlan;
use crate::source_built_fixed_point_action_trust::reconcile_actions;
use crate::source_built_fixed_point_action_trust::validate_action_plan;
use crate::source_built_fixed_point_dev_cache::DEV_CACHE_ENTRY_FILE;
use crate::source_built_fixed_point_dev_cache::DevCachePolicies;
use crate::source_built_fixed_point_dev_cache::DevProviderCacheEntry;
use crate::source_built_fixed_point_dev_cache::DevProviderCacheLookup;
use crate::source_built_fixed_point_dev_cache::FAST_FAIL_SCHEMA;
use crate::source_built_fixed_point_dev_cache::FastFailDecision;
use crate::source_built_fixed_point_dev_cache::StageCompletionMarker;
use crate::source_built_fixed_point_dev_cache::StageMarkerValidation;
use crate::source_built_fixed_point_dev_cache::dev_provider_cache_key;
use crate::source_built_fixed_point_dev_cache::evaluate_fast_fail;
use crate::source_built_fixed_point_dev_cache::evaluate_provider_cache_lookup;
use crate::source_built_fixed_point_dev_cache::validate_stage_marker;
use crate::source_built_fixed_point_planning::InitialOutputAuthorityState;
use crate::source_built_fixed_point_planning::SourceBuiltFixedPointPlanInput;
use crate::source_built_fixed_point_planning::plan_source_built_fixed_point;
use crate::source_built_fixed_point_resume::ResumeOutputReference;
use crate::source_built_fixed_point_resume::ResumePlan;
use crate::source_built_fixed_point_resume::plan_stage_resume;
use crate::source_built_fixed_point_resume_bundle::BLAKE3_DIGEST_HEX_LEN;
use crate::source_built_fixed_point_resume_bundle::RESUME_BUNDLES_SUBDIR;
use crate::source_built_fixed_point_resume_bundle::publish_stage_bundle;
use crate::source_built_fixed_point_resume_bundle::read_stage_bundles;
use crate::source_built_fixed_point_resume_bundle::restore_stage_bundle;
use crate::source_built_fixed_point_trust_report::ACTION_EVENTS_FILE;
use crate::source_built_fixed_point_trust_report::ACTION_JOURNAL_FILE;
use crate::source_built_fixed_point_trust_report::ACTION_PLAN_FILE;
use crate::source_built_fixed_point_trust_report::ACTION_RECONCILIATION_FILE;
use crate::source_built_fixed_point_trust_report::FINAL_RECEIPT_FILE;
use crate::source_bundle::SourceBuiltFixedPointProfileRecords;
use crate::source_bundle::SourceRecord;
use crate::source_bundle::assemble_source_bundle;
use crate::source_bundle::materialize_source_record_payload;
use crate::source_bundle::source_built_fixed_point_profile_records;
use crate::stagex_provider::StagexProviderRequest;
use crate::stagex_transition::StagexTransitionRequest;

#[path = "source_built_fixed_point_cold.rs"]
mod cold;
pub(crate) use cold::cmd_source_built_replay;

#[path = "source_built_fixed_point_authoring.rs"]
mod authoring;
pub(crate) use authoring::cmd_prepare_source_programs;

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
const STAGEX_TRANSITION_HANDOFF_REPLAY_DIR: &str = "stagex-transition-handoff-replay";
const STAGEX_PROVIDER_REPLAY_DIR: &str = "stagex-provider-replay";
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
pub(crate) const PROVIDER_KIND_LINKAGE_FILE: &str = "provider-kind-linkage.json";
pub(crate) const RELEASE_INPUTS_FILE: &str = "release-evidence-inputs.json";
const LATEST_ALIAS: &str = "latest";
const LATEST_SOURCE_BUILT_ALIAS: &str = "latest-source-built-fixed-point";
const SUCCESS_PUBLICATION_POINTER: &str = ".source-built-fixed-point-current";
const SUCCESS_PREVIOUS_POINTER: &str = ".source-built-fixed-point-previous";
const SUCCESS_PUBLICATION_LOCK: &str = ".source-built-fixed-point-publication.lock";
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
const FAST_FAIL_NOTICE: &str = "dev fast-fail: source profile unchanged from the last published fixed-point receipt; reporting the prior success without a fresh rebuild";
const EXPECTED_SINGLE_OUTPUT_COUNT: usize = 1;
const EXPECTED_STAGE_COUNT: usize = 6;
const BLAKE3_HEX_LENGTH: usize = 64;
const MAX_JOBS_MIN: u32 = 1;
const MAX_JOBS_MAX: u32 = 16;
const MATERIALIZED_SOURCE_TREE_ENTRY_COUNT_MAX: u32 = 1_000_000;
const FILE_MODE_EXECUTABLE_MASK: u32 = 0o111;
const ATTEMPT_DIRECTORY_MODE: u32 = 0o700;
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
    pub(crate) dev_provider_cache: Option<&'a Path>,
    pub(crate) dev_resume: bool,
    pub(crate) dev_fast_fail: bool,
    pub(crate) verbose: bool,
    pub(crate) backend: crunch_store::StoreBackend,
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
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct StagexTransitionHandoffReport {
    format: &'static str,
    copied_directories: Vec<&'static str>,
    non_claim: &'static str,
}

#[derive(Debug)]
pub(crate) struct ConstructedProviders {
    pub(crate) native_provider: BuildObservation,
    pub(crate) native_admission: crate::full_source_provider::FullSourceProviderAdmissionReport,
    pub(crate) rust_provider: crate::rust_source_provider::RustSourceProviderMaterialization,
    pub(crate) toolchain_closure_path: PathBuf,
}

/// Opt-in proof capture. A journal preserves every actual record before any
/// admission decision; only complete independently reconciled evidence is sealed.
pub(crate) struct ActionObservationRecorder<'plan> {
    root: PathBuf,
    plan: &'plan ActionTrustPlan,
    proof: &'plan SourceBuiltFixedPointPlan,
    admission: crate::source_built_fixed_point_action_trust::ActionObservationState,
    journal: std::io::BufWriter<fs::File>,
    events: Vec<ActionExecutionEvent>,
    event_limit: u32,
    failed: bool,
}

impl<'plan> ActionObservationRecorder<'plan> {
    pub(crate) fn create(
        root: &Path,
        proof: &'plan SourceBuiltFixedPointPlan,
        plan: &'plan ActionTrustPlan,
    ) -> Result<Self, RunError> {
        validate_action_plan(proof, plan).map_err(proof_error)?;
        write_json_create_new(&root.join(ACTION_PLAN_FILE), plan)?;
        let journal_path = root.join(ACTION_JOURNAL_FILE);
        let journal = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&journal_path)
            .map_err(|error| proof_error(format!("creating action observation journal: {error}")))?;
        fs::File::open(root)
            .and_then(|directory| directory.sync_all())
            .map_err(|error| proof_error(format!("syncing action observation root: {error}")))?;
        Ok(Self {
            root: root.to_path_buf(),
            plan,
            proof,
            admission: crate::source_built_fixed_point_action_trust::ActionObservationState::default(),
            journal: std::io::BufWriter::with_capacity(64 * 1_024, journal),
            events: Vec::new(),
            event_limit: proof.resource_bounds.protected_exec_events_max,
            failed: false,
        })
    }

    pub(crate) fn record(&mut self, event: ActionExecutionEvent) -> Result<(), RunError> {
        let previously_failed = self.failed;
        self.failed = true;
        serde_json::to_writer(&mut self.journal, &event)
            .map_err(|error| proof_error(format!("writing action observation: {error}")))?;
        self.journal
            .write_all(b"\n")
            .and_then(|()| self.journal.flush())
            .and_then(|()| self.journal.get_ref().sync_data())
            .map_err(|error| proof_error(format!("syncing action observation: {error}")))?;
        if previously_failed {
            return Err(proof_error("action observation capture already failed".into()));
        }
        // Preserve the first violating record too, without admitting it or
        // allocating beyond the proof's bounded in-memory event set.
        if self.events.len() >= self.event_limit as usize {
            return Err(proof_error("action observation event count exceeds proof bound".into()));
        }
        self.admission.record_planned(self.proof, self.plan, &event).map_err(proof_error)?;
        self.events.push(event);
        self.failed = false;
        Ok(())
    }

    pub(crate) fn finish(mut self, proof: &SourceBuiltFixedPointPlan) -> Result<ActionReconciliation, RunError> {
        self.journal
            .flush()
            .and_then(|()| self.journal.get_ref().sync_all())
            .map_err(|error| proof_error(format!("syncing final action observations: {error}")))?;
        if self.failed {
            return Err(proof_error("cannot seal failed action observation capture".into()));
        }
        write_json_create_new(&self.root.join(ACTION_EVENTS_FILE), &self.events)?;
        let reconciliation = reconcile_actions(proof, self.plan, &self.events).map_err(proof_error)?;
        write_json_create_new(&self.root.join(ACTION_RECONCILIATION_FILE), &reconciliation)?;
        fs::File::open(&self.root)
            .and_then(|directory| directory.sync_all())
            .map_err(|error| proof_error(format!("syncing action reconciliation root: {error}")))?;
        Ok(reconciliation)
    }
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
    let development_only = match run_attempt(&options, &prepared) {
        Ok(development_only) => development_only,
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
    if development_only {
        // Any development mode, including stage resume, is ineligible
        // for promoted cold receipts and both successful-proof aliases.
        print_dev_completion(&options, &prepared)?;
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

pub(crate) fn validate_cold_compiler_environment<I, S>(names: I) -> Result<(), RunError>
where
    I: IntoIterator<Item = S>,
    S: AsRef<std::ffi::OsStr>,
{
    for name in names {
        let name = name.as_ref();
        if name.as_encoded_bytes().starts_with(b"MANTLE_CC_CACHE_")
            || name.as_encoded_bytes().starts_with(b"MANTLE_COMPILE_")
        {
            return Err(proof_error(format!(
                "inherited compiler-cache/scheduling variable {} cannot authorize a promoted cold proof",
                name.to_string_lossy()
            )));
        }
    }
    Ok(())
}

fn validate_options(options: &SourceBuiltFixedPointOptions<'_>) -> Result<(), RunError> {
    if !is_development_run(options) {
        validate_cold_compiler_environment(std::env::vars_os().map(|(name, _)| name))?;
    }
    if options.dev_resume && options.dev_provider_cache.is_none() {
        return Err(proof_error("dev resume requires an explicit dev provider-cache directory".into()));
    }
    if let Some(cache) = options.dev_provider_cache.filter(|cache| !cache.is_absolute()) {
        return Err(proof_error(format!("dev provider-cache path must be absolute: {}", cache.display())));
    }
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
    if options.output_dir.file_name().and_then(|name| name.to_str()).is_some_and(|name| {
        matches!(
            name,
            LATEST_ALIAS
                | LATEST_SOURCE_BUILT_ALIAS
                | SUCCESS_PUBLICATION_POINTER
                | SUCCESS_PREVIOUS_POINTER
                | SUCCESS_PUBLICATION_LOCK
        )
    }) {
        return Err(proof_error("proof output basename is reserved for success publication".into()));
    }
    validate_expected_digest("source profile", options.expected_source_profile_blake3)?;
    validate_expected_digest("StageX lineage", options.expected_stagex_lineage_blake3)?;
    validate_expected_digest("native provider", options.expected_native_provider_blake3)?;
    crate::source_built_fixed_point::validate_resource_bounds(&requested_resource_bounds(options))
        .map_err(|error| proof_error(error.to_string()))?;
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
    validate_disk_preflight(options.output_dir, options.disk_bytes_max)
}

fn requested_resource_bounds(options: &SourceBuiltFixedPointOptions<'_>) -> SourceBuiltFixedPointResourceBounds {
    SourceBuiltFixedPointResourceBounds {
        elapsed_seconds_max: options.elapsed_seconds_max,
        disk_bytes_max: options.disk_bytes_max,
        open_file_descriptors_max: SOURCE_BUILT_FIXED_POINT_OPEN_FILE_DESCRIPTORS_MAX,
        protected_exec_events_max: options.protected_exec_events_max,
        source_records_max: options.source_records_max,
    }
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

fn prepare_attempt(
    options: &SourceBuiltFixedPointOptions<'_>,
    started_at: Instant,
) -> Result<PreparedAttempt, RunError> {
    let profile = crate::source_bundle::read_source_bundle(options.source_profile)?;
    prepare_profile_attempt(options, started_at, &profile, false)
}

fn prepare_profile_attempt(
    options: &SourceBuiltFixedPointOptions<'_>,
    started_at: Instant,
    profile: &crate::source_bundle::SourceBundleManifest,
    preparing_programs: bool,
) -> Result<PreparedAttempt, RunError> {
    if profile.format != PROFILE_SCHEMA {
        return Err(proof_error(format!("source profile schema must be {PROFILE_SCHEMA}")));
    }
    if profile.manifest_blake3 != options.expected_source_profile_blake3 {
        return Err(proof_error(format!(
            "source profile digest mismatch: expected {}, observed {}",
            options.expected_source_profile_blake3, profile.manifest_blake3
        )));
    }
    let records = source_built_fixed_point_profile_records(profile)?;
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
    // Cold authority roots must remain disjoint: inserting vendors into the
    // already-hashed Mantle source would invalidate its admitted tree identity.
    // The legacy development builder still needs its unclaimable local overlay.
    let vendor_root = if is_development_run(options) {
        source_root.join(VENDOR_RELATIVE_PATH)
    } else {
        inputs_dir.join(VENDOR_RELATIVE_PATH)
    };
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
    let plan_path = staging_dir.join(if preparing_programs {
        "source-program-preparation-input-plan.json"
    } else {
        PLAN_FILE
    });
    write_json_create_new(&plan_path, &plan)?;
    if is_development_run(options) {
        write_development_authority(&staging_dir, options, &plan)?;
    }
    let (native_store_dir, native_state_dir) = if preparing_programs {
        (staging_dir.join(NATIVE_STORE_DIR), staging_dir.join(NATIVE_STATE_DIR))
    } else {
        match options.dev_provider_cache {
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
        }
    };
    let transcripts_dir = staging_dir.join(TRANSCRIPTS_DIR);
    if !preparing_programs {
        fs::create_dir(&transcripts_dir)
            .map_err(|error| proof_error(format!("creating transcripts {}: {error}", transcripts_dir.display())))?;
        crate::source_bundle::import_source_bundle(profile, &native_state_dir, true)?;
    }
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
        resource_bounds: requested_resource_bounds(options),
    };
    let plan = plan_source_built_fixed_point(plan_input)
        .map_err(|error| proof_error(format!("preparing source-built fixed-point plan: {error}")))?;
    assert_eq!(plan.stages.len(), EXPECTED_STAGE_COUNT);
    debug_assert_eq!(plan.logical_store_prefix, LOGICAL_STORE_PREFIX);
    Ok(plan)
}

fn is_development_run(options: &SourceBuiltFixedPointOptions<'_>) -> bool {
    options.dev_provider_cache.is_some() || options.dev_resume || options.dev_fast_fail
}

/// A coordination-only view of the proof plan's actual output authority edges.
/// Neither this declaration nor a stage command's exit verifies an action plan.
struct StageReadinessBinding<'a> {
    id: &'a str,
    dependencies: Vec<&'a str>,
    final_receipt_required: bool,
}

fn stage_readiness_bindings(plan: &SourceBuiltFixedPointPlan) -> Result<Vec<StageReadinessBinding<'_>>, String> {
    let mut bindings = Vec::with_capacity(plan.stages.len());
    for stage in &plan.stages {
        let mut dependencies = Vec::new();
        for input in &stage.inputs {
            let StageAuthorityInput::Output { role } = input else {
                continue;
            };
            let producer = plan
                .stages
                .iter()
                .find(|candidate| candidate.output == *role)
                .ok_or_else(|| format!("readiness stage {} has no output producer for {role:?}", stage.stage_id))?;
            if producer.stage_id == stage.stage_id || dependencies.contains(&producer.stage_id.as_str()) {
                return Err(format!("readiness stage {} has a self or duplicate predecessor", stage.stage_id));
            }
            dependencies.push(&producer.stage_id);
        }
        bindings.push(StageReadinessBinding {
            id: &stage.stage_id,
            dependencies,
            final_receipt_required: stage.kind == ProofStageKind::MantleStage2,
        });
    }
    Ok(bindings)
}
fn run_attempt(options: &SourceBuiltFixedPointOptions<'_>, prepared: &PreparedAttempt) -> Result<bool, RunError> {
    enforce_open_file_descriptor_limit(prepared.plan.resource_bounds.open_file_descriptors_max)?;
    validate_runtime_bounds(options, prepared)?;
    if !is_development_run(options) {
        return cold::run_cold_attempt(options, prepared);
    }
    let adopt = dev_cache_adoption(options, &prepared.plan)?;
    let is_dev = is_development_run(options);
    if adopt {
        record_dev_cache_hit(prepared)?;
    }
    let resume_plan = if options.dev_resume && !adopt {
        let plan = plan_dev_resume(prepared, options)?;
        write_json_create_new(&prepared.staging_dir.join("dev-resume-authority.json"), &plan)?;
        emit_resume_plan_notice(options, prepared, &plan);
        Some(plan)
    } else {
        None
    };
    // A verbose coordination projection is outside the proof tree and cannot
    // become a receipt. Without observed action reconciliation no stage
    // completion is asserted, even if the proof shell keeps executing.
    let stage_bindings = if options.verbose {
        stage_readiness_bindings(&prepared.plan)
            .map_err(|reason| proof_error(format!("deriving stage readiness dependencies: {reason}")))?
    } else {
        Vec::new()
    };
    let stage_declarations = stage_bindings
        .iter()
        .map(|binding| crunch_service_readiness_core::Component {
            id: binding.id,
            kind: crunch_service_readiness_core::ComponentKind::ProofStage,
            depends_on: &binding.dependencies,
            user_states: &[],
            restart_policy: None,
            stage_requirements: Some(crunch_service_readiness_core::StageRequirements {
                require_action_reconciliation: true,
                require_v2_receipt: binding.final_receipt_required,
            }),
        })
        .collect::<Vec<_>>();
    let render_stage_readiness =
        |observations: &[crunch_service_readiness_core::Observation<'_>]| -> Result<String, RunError> {
            let initial = crunch_service_readiness_core::evaluate(&crunch_service_readiness_core::Snapshot {
                schema: crunch_service_readiness_core::READINESS_SCHEMA,
                components: &stage_declarations,
                observations: &[],
            })
            .map_err(|error| proof_error(format!("admitting stage readiness graph: {error:?}")))?;
            let report = if observations.is_empty() {
                initial
            } else {
                crunch_service_readiness_core::advance(&initial, &crunch_service_readiness_core::Snapshot {
                    schema: crunch_service_readiness_core::READINESS_SCHEMA,
                    components: &stage_declarations,
                    observations,
                })
                .map_err(|error| proof_error(format!("admitting observed stage state: {error:?}")))?
            };
            let components = report
                .components
                .iter()
                .map(|row| {
                    serde_json::json!({
                        "id": row.id,
                        "generation": row.generation,
                        "states": row.states,
                        "restart_policy": null,
                        "restart_action": null,
                        "ready": row.ready,
                        "blocked_by": row.blocked_by,
                    })
                })
                .collect::<Vec<_>>();
            Ok(serde_json::json!({
                "schema": report.schema,
                "classification": report.classification,
                "evidence_eligible": report.evidence_eligible,
                "components": components,
            })
            .to_string())
        };
    if options.verbose {
        eprintln!("{}", render_stage_readiness(&[])?);
    }
    let mut transition_restored = false;
    let providers = if adopt {
        let adopted = adopt_cached_provider_subtrees(options, prepared, &prepared.plan)?;
        read_adopted_stagex_report(prepared)?;
        construct_full_source_providers(options, prepared, Some(adopted))?
    } else {
        let stagex_transition_execution_dir = prepared.staging_dir.join(STAGEX_TRANSITION_EXECUTION_DIR);
        let transition_replay_digest = transition_input_replay_digest(prepared)?;
        if options.dev_resume && !stagex_transition_execution_dir.exists() {
            let restored = resume_plan
                .as_ref()
                .and_then(|plan| plan.restored_stages.iter().find(|stage| stage.stage_id == "stagex-transition"))
                .and_then(|stage| stage.restored_bundle_digest_blake3.clone());
            if let Some(bundle_digest_blake3) = restored {
                restore_dev_stage_bundle(options, prepared, &bundle_digest_blake3, &stagex_transition_execution_dir)?;
                write_stage_marker(prepared, "stagex-transition", &transition_replay_digest)?;
                transition_restored = true;
            }
        }
        let transition_resume = options.dev_resume
            && stagex_transition_execution_dir.is_dir()
            && transition_marker_is_trusted(prepared, &transition_replay_digest)?;
        if !transition_resume {
            let stagex_started = if options.verbose {
                let observation = [crunch_service_readiness_core::Observation {
                    id: "stagex-transition",
                    generation: 1,
                    states: &["started"],
                    exit: None,
                    request_acknowledged: false,
                    proof_completion: None,
                }];
                Some(render_stage_readiness(&observation)?)
            } else {
                None
            };
            let transition_result = run_in_isolated_exec_thread("StageX transition", || {
                if let Some(report) = &stagex_started {
                    eprintln!("{report}");
                }
                crate::stagex_transition::materialize_protected_transition(StagexTransitionRequest {
                    seed_path: &prepared.stagex_seed,
                    hex0_source_path: &prepared.source_root.join("bootstrap/seeds/AMD64/hex0_AMD64.hex0"),
                    kaem_source_path: &prepared.source_root.join("bootstrap/seeds/AMD64/kaem-minimal.hex0"),
                    lineage_manifest_path: &prepared.stagex_lineage,
                    source_bundle_path: Some(&prepared.stagex_source_bundle),
                    stage0_answers_path: Some(&prepared.source_root.join("bootstrap/stage0-amd64.answers")),
                    scratch_dir: &stagex_transition_execution_dir,
                })
            });
            if options.verbose {
                let failed = !matches!(&transition_result, Ok(Ok(report)) if report.status == PROOF_STATUS_COMPLETE);
                if failed {
                    let observation = [crunch_service_readiness_core::Observation {
                        id: "stagex-transition",
                        generation: 1,
                        states: &["failed"],
                        exit: None,
                        request_acknowledged: false,
                        proof_completion: None,
                    }];
                    eprintln!("{}", render_stage_readiness(&observation)?);
                } else {
                    eprintln!("{}", render_stage_readiness(&[])?);
                }
            }
            let transition_result = transition_result?;
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
            let transition_report_path = stagex_transition_execution_dir.join(STAGEX_TRANSITION_REPORT_FILE);
            let transition_report_digest = crate::protected_exec::blake3_file_hex(&transition_report_path)
                .map_err(|error| proof_error(format!("hashing fresh StageX transition report: {error}")))?;
            if options.dev_provider_cache.is_some() || options.dev_resume {
                // Resume trust re-derives the transition from its primary inputs, so the
                // marker binds the replay identity rather than the report digest.
                write_stage_marker(prepared, "stagex-transition", &transition_replay_digest)?;
            }
            if options.dev_resume {
                let required_outputs = [ResumeOutputReference {
                    role: String::from("transition-execution-tree"),
                    digest_blake3: transition_report_digest.clone(),
                }];
                publish_dev_stage_bundle(
                    options,
                    prepared,
                    "stagex-transition",
                    &stagex_transition_execution_dir,
                    &required_outputs,
                )?;
            }
        }

        let stagex_transition_root = prepared.native_store_dir.join(STAGEX_TRANSITION_STORE_BASENAME);
        let stagex_transition_replay_root = prepared.staging_dir.join(STAGEX_TRANSITION_HANDOFF_REPLAY_DIR);
        materialize_or_validate_stagex_transition_handoff(
            is_dev,
            &stagex_transition_execution_dir,
            &stagex_transition_root,
            &stagex_transition_replay_root,
        )?;
        let transition_logical_path = crate::full_source_provider::adopt_verified_local_provider_path_strict(
            &stagex_transition_root,
            &prepared.native_store_dir,
            &prepared.native_state_dir,
            LOGICAL_STORE_PREFIX,
            options.backend,
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
        let reuse_stagex_provider = is_dev && stagex_provider_root.exists();
        let stagex_provider_replay_root = prepared.staging_dir.join(STAGEX_PROVIDER_REPLAY_DIR);
        let stagex_provider_output_root = if reuse_stagex_provider {
            &stagex_provider_replay_root
        } else {
            &stagex_provider_root
        };
        let stagex_provider_result = run_in_isolated_exec_thread("StageX provider publication", || {
            crate::stagex_provider::materialize_stagex_provider(StagexProviderRequest {
                lineage_manifest_path: &prepared.stagex_lineage,
                transition_root: &stagex_transition_execution_dir,
                output_path: stagex_provider_output_root,
            })
        })?;
        let stagex_provider_report = stagex_provider_result
            .map_err(|error| proof_error(format!("StageX provider publication failed: {error}")))?;
        validate_stagex_provider_normalized_identity(&stagex_provider_report.normalized_provider_digest_blake3)?;
        if reuse_stagex_provider {
            // Provider receipts bind current-run transition evidence. Keep the replay for receipts,
            // and compare the reusable runtime payload through its normalized identity.
            let persistent_digest =
                crate::stagex_provider::observe_normalized_provider_payload_digest(&stagex_provider_root)
                    .map_err(|error| proof_error(format!("observing persistent StageX provider payload: {error}")))?;
            validate_reusable_stagex_provider_identity(
                &stagex_provider_report.normalized_provider_digest_blake3,
                &persistent_digest,
            )?;
            assert!(stagex_provider_replay_root.is_dir());
            assert!(stagex_provider_root.is_dir());
        }
        validate_runtime_bounds(options, prepared)?;
        let stagex_logical_path = crate::full_source_provider::adopt_verified_local_provider_path_strict(
            &stagex_provider_root,
            &prepared.native_store_dir,
            &prepared.native_state_dir,
            LOGICAL_STORE_PREFIX,
            options.backend,
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
        construct_full_source_providers(options, prepared, None)?
    };
    validate_runtime_bounds(options, prepared)?;
    run_cargo_free_fixed_point(options, prepared, &providers)?;
    validate_runtime_bounds(options, prepared)?;
    // The retained legacy construction path is development-only. Its complete
    // non-proof behavior remains available, but it cannot publish a v2 receipt.
    if options.dev_provider_cache.is_some() && !adopt {
        write_dev_provider_cache(options, prepared, &providers)?;
    }
    write_dev_execution_report(prepared, adopt, transition_restored)?;
    Ok(true)
}

fn write_development_authority(
    root: &Path,
    options: &SourceBuiltFixedPointOptions<'_>,
    plan: &SourceBuiltFixedPointPlan,
) -> Result<(), RunError> {
    let report = serde_json::json!({
        "schema": PROOF_STATUS_SCHEMA,
        "status": "development-only",
        "plan_digest_blake3": plan.plan_digest_blake3,
        "output": root,
        "provider_cache_configured": options.dev_provider_cache.is_some(),
        "resume_requested": options.dev_resume,
        "fast_fail_requested": options.dev_fast_fail,
        "notice": "development execution cannot authorize a promoted cold receipt or release alias",
    });
    write_json_create_new(&root.join("dev-run-report.json"), &report)
}

fn write_dev_execution_report(
    prepared: &PreparedAttempt,
    providers_adopted: bool,
    transition_restored: bool,
) -> Result<(), RunError> {
    assert!(!(providers_adopted && transition_restored));
    let stages = prepared
        .plan
        .stages
        .iter()
        .map(|stage| {
            let disposition = match stage.kind {
                ProofStageKind::StagexTransition if providers_adopted => "skipped-provider-adoption",
                ProofStageKind::StagexTransition if transition_restored => "restored",
                ProofStageKind::StagexProviderPublication | ProofStageKind::FullSourceNativeProvider
                    if providers_adopted =>
                {
                    "adopted"
                }
                _ => "dispatched",
            };
            serde_json::json!({"stage_id": stage.stage_id, "disposition": disposition})
        })
        .collect::<Vec<_>>();
    debug_assert_eq!(stages.len(), EXPECTED_STAGE_COUNT);
    let report = serde_json::json!({
        "schema": "mantle-source-built-fixed-point-dev-execution-v1",
        "status": "development-only",
        "plan_digest_blake3": prepared.plan.plan_digest_blake3,
        "source_authority_digest_blake3": prepared.plan.source_authority_digest_blake3,
        "native_provider_digest_blake3": prepared.plan.policies.expected_native_provider_digest_blake3,
        "store": prepared.native_store_dir,
        "state": prepared.native_state_dir,
        "stages": stages,
        "non_claim": "stage dispatch is not fresh action execution; consult build transcripts for store hits; no promoted proof or release authority",
    });
    write_json_create_new(&prepared.staging_dir.join("dev-execution-report.json"), &report)
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
    if entry_root.exists() {
        return Err(proof_error(format!("dev provider-cache entry root exists: {}", entry_root.display())));
    }
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
    fs::create_dir_all(&entry_root).map_err(|error| {
        proof_error(format!("creating dev provider-cache entry root {}: {error}", entry_root.display()))
    })?;
    crate::stagex_mes_lib::copy_tree_bounded(&stagex_src, &entry_root.join(stagex_basename))
        .map_err(|error| proof_error(format!("publishing cached StageX provider: {error}")))?;
    crate::stagex_mes_lib::copy_tree_bounded(native_src, &entry_root.join(native_basename))
        .map_err(|error| proof_error(format!("publishing cached native provider: {error}")))?;
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
    write_json_create_new(&entry_root.join(DEV_CACHE_ENTRY_FILE), &entry)?;
    debug_assert!(entry_root.join(DEV_CACHE_ENTRY_FILE).is_file());
    debug_assert!(entry_root.join(stagex_basename).is_dir());
    debug_assert!(entry_root.join(native_basename).is_dir());
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
    register_adopted_provider(&stagex_dst, store_dir, state_dir, options.backend)?;
    let native_dst = store_dir.join(&entry.native_provider.store_basename);
    if !native_dst.exists() {
        crate::stagex_mes_lib::copy_tree_bounded(&entry_root.join(&entry.native_provider.store_basename), &native_dst)
            .map_err(|error| proof_error(format!("seeding cached native provider: {error}")))?;
    }
    register_adopted_provider(&native_dst, store_dir, state_dir, options.backend)?;
    debug_assert!(stagex_dst.is_dir());
    debug_assert!(native_dst.is_dir());
    Ok(true)
}

fn stage_markers_dir(staging_dir: &Path) -> PathBuf {
    staging_dir.join(STAGE_MARKERS_SUBDIR)
}

/// Persistent root for published cross-run stage bundles.
fn resume_bundles_root(options: &SourceBuiltFixedPointOptions<'_>) -> Result<PathBuf, RunError> {
    let cache = options
        .dev_provider_cache
        .ok_or_else(|| proof_error("dev resume requires an explicit dev provider-cache directory".into()))?;
    let root = cache.join(RESUME_BUNDLES_SUBDIR);
    debug_assert!(root.starts_with(cache));
    debug_assert!(!RESUME_BUNDLES_SUBDIR.is_empty());
    Ok(root)
}

/// Plan a dev resume from published bundles against the current plan.
fn plan_dev_resume(
    prepared: &PreparedAttempt,
    options: &SourceBuiltFixedPointOptions<'_>,
) -> Result<ResumePlan, RunError> {
    let bundles_root = resume_bundles_root(options)?;
    let bundles = read_stage_bundles(&bundles_root)?;
    let stage_ids: Vec<String> = prepared.plan.stages.iter().map(|stage| stage.stage_id.clone()).collect();
    let policies = DevCachePolicies::from_plan(&prepared.plan);
    let plan = plan_stage_resume(
        &prepared.plan.plan_digest_blake3,
        &prepared.plan.source_authority_digest_blake3,
        &stage_ids,
        &prepared.plan.policies.expected_native_provider_digest_blake3,
        &policies,
        &bundles,
    );
    debug_assert!(plan.restored_stages.len() + plan.executed_stages.len() == stage_ids.len());
    debug_assert!(!prepared.plan.source_authority_digest_blake3.is_empty());
    Ok(plan)
}

/// Restore one validated stage bundle into a fresh staging destination.
fn restore_dev_stage_bundle(
    options: &SourceBuiltFixedPointOptions<'_>,
    prepared: &PreparedAttempt,
    bundle_digest_blake3: &str,
    destination: &Path,
) -> Result<(), RunError> {
    let bundles_root = resume_bundles_root(options)?;
    let bundles = read_stage_bundles(&bundles_root)?;
    let reference = bundles
        .iter()
        .find(|reference| reference.bundle_digest_blake3 == bundle_digest_blake3)
        .ok_or_else(|| {
            proof_error(format!(
                "planned resume bundle {bundle_digest_blake3} is no longer present under {}",
                bundles_root.display()
            ))
        })?;
    restore_stage_bundle(&bundles_root, reference, destination)?;
    debug_assert!(destination.is_dir());
    debug_assert!(!prepared.plan.plan_digest_blake3.is_empty());
    Ok(())
}

/// Publish one completed stage as a content-addressed bundle.
fn publish_dev_stage_bundle(
    options: &SourceBuiltFixedPointOptions<'_>,
    prepared: &PreparedAttempt,
    stage_id: &str,
    payload_dir: &Path,
    required_outputs: &[ResumeOutputReference],
) -> Result<(), RunError> {
    let bundles_root = resume_bundles_root(options)?;
    std::fs::create_dir_all(&bundles_root).map_err(|error| {
        RunError::Internal(format!("creating resume bundles root {}: {error}", bundles_root.display()))
    })?;
    let policies = DevCachePolicies::from_plan(&prepared.plan);
    let reference = publish_stage_bundle(
        &bundles_root,
        &prepared.plan.plan_digest_blake3,
        stage_id,
        &prepared.plan.source_authority_digest_blake3,
        &prepared.plan.policies.expected_native_provider_digest_blake3,
        &policies,
        payload_dir,
        required_outputs,
    )?;
    debug_assert_eq!(reference.bundle_digest_blake3.len(), BLAKE3_DIGEST_HEX_LEN);
    debug_assert!(bundles_root.is_dir());
    Ok(())
}

/// Report the restored and executed stage split for a dev resume.
fn emit_resume_plan_notice(options: &SourceBuiltFixedPointOptions<'_>, prepared: &PreparedAttempt, plan: &ResumePlan) {
    debug_assert!(!plan.schema.is_empty());
    debug_assert!(plan.restored_stages.len() + plan.executed_stages.len() > 0);
    let restored: Vec<&str> = plan.restored_stages.iter().map(|stage| stage.stage_id.as_str()).collect();
    let executed: Vec<&str> = plan.executed_stages.iter().map(String::as_str).collect();
    debug_assert!(restored.len() + executed.len() > 0);
    if options.json {
        let value = serde_json::json!({
            "schema": plan.schema,
            "plan_digest_blake3": plan.plan_digest_blake3,
            "proof_id": prepared.plan.proof_id,
            "restored_stages": restored,
            "executed_stages": executed,
            "blocker_count": plan.blockers.len(),
        });
        println!("{value}");
        return;
    }
    eprintln!(
        "dev resume: restored={} executed={} blockers={}",
        restored.len(),
        executed.len(),
        plan.blockers.len()
    );
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
    debug_assert!(stagex_dst.is_dir());
    debug_assert!(native_dst.is_dir());
    Ok(BuildObservation {
        output: BuildJsonOutput {
            name: BUILD_OUTPUT_NAME.to_string(),
            path: native_dst.clone(),
            artifact_attestation: BuildJsonAttestationReference {
                logical_path: entry.native_provider.logical_path.clone(),
                path: tx,
            },
        },
    })
}

/// Register an on-disk provider subtree with the content-addressed store service
/// (`StoreHandle::adopt_verified_local_output`), matching how the cold path adopts
/// freshly constructed transition/provider paths. Returns the logical store path.
fn register_adopted_provider(
    physical_path: &Path,
    native_store_dir: &Path,
    native_state_dir: &Path,
    backend: crunch_store::StoreBackend,
) -> Result<String, RunError> {
    let logical = crate::full_source_provider::adopt_verified_local_provider_path_strict(
        physical_path,
        native_store_dir,
        native_state_dir,
        LOGICAL_STORE_PREFIX,
        backend,
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

/// Revalidate cached historical observations without claiming fresh execution.
fn read_adopted_stagex_report(
    prepared: &PreparedAttempt,
) -> Result<crate::stagex_provider::StagexProviderPublicationReport, RunError> {
    let stagex_root = prepared.native_store_dir.join(STAGEX_PROVIDER_STORE_BASENAME);
    crate::stagex_provider::read_adopted_provider_publication(&stagex_root, STAGEX_PROVIDER_EXPECTED_NORMALIZED_DIGEST)
        .map_err(|error| proof_error(format!("validating adopted StageX publication: {error}")))
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
    adopted_native: Option<BuildObservation>,
) -> Result<ConstructedProviders, RunError> {
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
        native_provider,
        native_admission,
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
    let allow_cached = options.dev_provider_cache.is_some();
    let report = parse_build_report(label, &output, allow_cached)?;
    let build_output = require_single_build_output(label, report, allow_cached)?;
    Ok(BuildObservation { output: build_output })
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
        .arg("--store-backend")
        .arg(options.backend.as_str())
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
    use mantle_application_contract::bootstrap_effects::BootstrapEffect::VerifyOutput;
    use mantle_application_contract::bootstrap_effects::BootstrapEffect::WriteOutput;

    let mut port = crate::bootstrap_effect_port::BootstrapPort::new(&[(WriteOutput, 2), (VerifyOutput, 1)])?;
    let parent = prepared.final_dir.parent().ok_or_else(|| proof_error("proof output has no parent".to_string()))?;
    let target_name = prepared
        .final_dir
        .file_name()
        .ok_or_else(|| proof_error("proof output has no basename".to_string()))?;
    let aliases = [LATEST_ALIAS, LATEST_SOURCE_BUILT_ALIAS];
    // Alias commit may have reached its single atomic pointer rename even if
    // a later directory sync fails. Never remove its complete target bundle;
    // the caller records durable failure at this preserved final root.
    let result = (|| {
        port.call(WriteOutput, || {
            crate::linux_rename::rename_path_no_replace(&prepared.staging_dir, &prepared.final_dir).map_err(|error| {
                proof_error(format!(
                    "publishing proof bundle {} -> {}: {error}",
                    prepared.staging_dir.display(),
                    prepared.final_dir.display()
                ))
            })
        })?;
        port.call(WriteOutput, || update_success_aliases(parent, &aliases, target_name))?;
        port.call(VerifyOutput, || verify_success_selection(parent, &aliases, target_name))
    })();
    port.finish(result)
}

fn verify_success_selection(parent: &Path, aliases: &[&str], target: &std::ffi::OsStr) -> Result<(), RunError> {
    use mantle_application_contract::ApplicationOutcome;
    use mantle_application_contract::CommandFamily;
    use mantle_application_contract::EffectId;
    use mantle_application_contract::EffectKind;
    use mantle_application_contract::EffectMeasure;
    use mantle_application_contract::EffectOutput;
    use mantle_application_contract::EffectSpec;
    use mantle_application_contract::ExpectedOutput;
    use mantle_application_contract::Observation;
    use mantle_application_contract::ObservationStatus;
    use mantle_application_contract::classify_observations;
    use mantle_application_contract::plan_effects;
    let expected = blake3::hash(target.as_encoded_bytes()).to_hex().to_string();
    let plan = plan_effects(CommandFamily::Bootstrap, &[EffectSpec {
        effect_id: "source-built-success-selection",
        kind: EffectKind::ReadFiles,
        limit: EffectMeasure::Bytes((target.as_encoded_bytes().len() as u64).max(1)),
        expected_output: ExpectedOutput::Identity(&expected),
    }])
    .map_err(|error| proof_error(format!("planning success selection readback: {}", error.code())))?;
    let selected = existing_success_target(parent, aliases, &parent.join(SUCCESS_PUBLICATION_POINTER))
        .and_then(|selected| selected.ok_or_else(|| proof_error("published success selection is absent".into())));
    let observed = Observation {
        effect_id: EffectId("source-built-success-selection".into()),
        kind: EffectKind::ReadFiles,
        status: if selected.is_ok() {
            ObservationStatus::Succeeded
        } else {
            ObservationStatus::Failed
        },
        output: selected.as_ref().map_or(EffectOutput::None, |path| {
            EffectOutput::Identity(blake3::hash(path.as_os_str().as_encoded_bytes()).to_hex().to_string())
        }),
        usage: EffectMeasure::Bytes(
            selected.as_ref().map_or(0, |path| path.as_os_str().as_encoded_bytes().len() as u64),
        ),
        diagnostics_code: selected.as_ref().err().map(|_| "source-built-success-selection-read-failed".into()),
    };
    match (classify_observations(&plan, std::slice::from_ref(&observed)), selected) {
        (ApplicationOutcome::Completed, Ok(_)) => Ok(()),
        (ApplicationOutcome::Failed { .. }, Err(error)) => Err(error),
        (outcome, _) => {
            Err(proof_error(format!("published source proof selection differs from planned target: {outcome:?}")))
        }
    }
}

fn update_success_aliases(parent: &Path, alias_names: &[&str], target_name: &std::ffi::OsStr) -> Result<(), RunError> {
    if alias_names != [LATEST_ALIAS, LATEST_SOURCE_BUILT_ALIAS].as_slice() {
        return Err(proof_error("success publication requires both canonical aliases exactly once".into()));
    }
    validate_local_alias_target(Path::new(target_name))?;
    let lock = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC)
        .open(parent.join(SUCCESS_PUBLICATION_LOCK))
        .map_err(|error| proof_error(format!("opening success publication lock: {error}")))?;
    if !lock
        .metadata()
        .map_err(|error| proof_error(format!("inspecting publication lock: {error}")))?
        .is_file()
    {
        return Err(proof_error("success publication lock must be a regular file".into()));
    }
    fs2::FileExt::try_lock_exclusive(&lock)
        .map_err(|error| proof_error(format!("another success publication owns the alias lock: {error}")))?;
    let pointer = parent.join(SUCCESS_PUBLICATION_POINTER);
    let previous = existing_success_target(parent, alias_names, &pointer)?;
    // Retain a root-local rollback target before touching either public name.
    // It intentionally remains after success as the previous generation link.
    let recovery = parent.join(SUCCESS_PREVIOUS_POINTER);
    remove_stale_alias_transaction_path(&recovery)?;
    if let Some(previous) = &previous {
        std::os::unix::fs::symlink(previous, &recovery)
            .map_err(|error| proof_error(format!("staging prior success publication: {error}")))?;
    }
    if read_success_link(&pointer)?.is_none()
        && let Some(previous) = &previous
    {
        std::os::unix::fs::symlink(previous, &pointer)
            .map_err(|error| proof_error(format!("initializing success publication pointer: {error}")))?;
    }
    prepare_stable_success_aliases(parent, alias_names)?;
    let temporary = parent.join(format!("{SUCCESS_PUBLICATION_POINTER}.{}.tmp", std::process::id()));
    remove_stale_alias_transaction_path(&temporary)?;
    std::os::unix::fs::symlink(target_name, &temporary)
        .map_err(|error| proof_error(format!("staging success publication pointer: {error}")))?;
    let directory = fs::File::open(parent)
        .map_err(|error| proof_error(format!("opening success publication directory: {error}")))?;
    directory
        .sync_all()
        .map_err(|error| proof_error(format!("syncing prepared success aliases: {error}")))?;
    // Both public names already refer to this one entry. This is the only
    // operation that can change either alias from the old proof to the new one.
    fs::rename(&temporary, &pointer)
        .map_err(|error| proof_error(format!("committing success publication pointer: {error}")))?;
    if let Err(commit_sync_error) = directory.sync_all() {
        let rollback = if previous.is_some() {
            fs::rename(&recovery, &pointer)
        } else {
            fs::remove_file(&pointer)
        };
        rollback.map_err(|error| {
            proof_error(format!(
                "success publication sync failed ({commit_sync_error}); restoring prior selection also failed: {error}"
            ))
        })?;
        directory.sync_all().map_err(|error| proof_error(format!(
            "success publication sync failed ({commit_sync_error}); prior selection restored but rollback sync failed: {error}"
        )))?;
        return Err(proof_error(format!(
            "success publication sync failed; prior selection restored: {commit_sync_error}"
        )));
    }
    Ok(())
}

fn existing_success_target(parent: &Path, aliases: &[&str], pointer: &Path) -> Result<Option<PathBuf>, RunError> {
    let pointer_target = read_success_link(pointer)?;
    if let Some(target) = &pointer_target {
        validate_local_alias_target(target)?;
    }
    let mut selected: Option<Option<PathBuf>> = None;
    for alias in aliases {
        let target = match read_success_link(&parent.join(alias))? {
            Some(target) if target == Path::new(SUCCESS_PUBLICATION_POINTER) => pointer_target.clone(),
            Some(target) => {
                validate_local_alias_target(&target)?;
                Some(target)
            }
            None => None,
        };
        if selected.as_ref().is_some_and(|previous| previous != &target) {
            return Err(proof_error(
                "existing success aliases disagree on the published proof, including absent aliases".into(),
            ));
        }
        selected = Some(target);
    }
    let selected = selected.flatten();
    if pointer_target.as_ref().is_some_and(|previous| Some(previous) != selected.as_ref()) {
        return Err(proof_error("success publication pointer disagrees with the public alias selection".into()));
    }
    Ok(selected)
}

fn read_success_link(path: &Path) -> Result<Option<PathBuf>, RunError> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_symlink() => fs::read_link(path)
            .map(Some)
            .map_err(|error| proof_error(format!("reading success alias {}: {error}", path.display()))),
        Ok(_) => Err(proof_error(format!("success alias path is not a symlink: {}", path.display()))),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(proof_error(format!("inspecting success alias {}: {error}", path.display()))),
    }
}

fn validate_local_alias_target(target: &Path) -> Result<(), RunError> {
    let mut components = target.components();
    if !matches!(components.next(), Some(std::path::Component::Normal(_))) || components.next().is_some() {
        return Err(proof_error(format!("success alias target must be one root-local basename: {}", target.display())));
    }
    if target.to_str().is_some_and(|name| {
        matches!(
            name,
            LATEST_ALIAS
                | LATEST_SOURCE_BUILT_ALIAS
                | SUCCESS_PUBLICATION_POINTER
                | SUCCESS_PREVIOUS_POINTER
                | SUCCESS_PUBLICATION_LOCK
        )
    }) {
        return Err(proof_error("success alias target cannot select publication metadata or another alias".into()));
    }
    Ok(())
}

fn prepare_stable_success_aliases(parent: &Path, aliases: &[&str]) -> Result<(), RunError> {
    for alias in aliases {
        let path = parent.join(alias);
        if read_success_link(&path)?.as_deref() == Some(Path::new(SUCCESS_PUBLICATION_POINTER)) {
            continue;
        }
        let temporary = parent.join(format!(".{alias}.{}.tmp", std::process::id()));
        remove_stale_alias_transaction_path(&temporary)?;
        std::os::unix::fs::symlink(SUCCESS_PUBLICATION_POINTER, &temporary)
            .map_err(|error| proof_error(format!("preparing shared success alias {alias}: {error}")))?;
        fs::rename(&temporary, &path)
            .map_err(|error| proof_error(format!("installing shared success alias {alias}: {error}")))?;
    }
    Ok(())
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

fn print_dev_completion(
    options: &SourceBuiltFixedPointOptions<'_>,
    prepared: &PreparedAttempt,
) -> Result<(), RunError> {
    if options.json {
        let value = serde_json::json!({
            "schema": PROOF_STATUS_SCHEMA,
            "status": "development-only",
            "output": prepared.staging_dir,
            "plan_digest_blake3": prepared.plan.plan_digest_blake3,
            "notice": "development execution cannot authorize a promoted cold receipt or release alias",
        });
        println!("{}", serde_json::to_string_pretty(&value).map_err(|error| proof_error(error.to_string()))?);
    } else {
        eprintln!("Completed development run at {}", prepared.staging_dir.display());
        eprintln!("  plan_digest_blake3: {}", prepared.plan.plan_digest_blake3);
        eprintln!("  development-only: no promoted cold receipt, no release alias");
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
    let temporary = staging_dir.join(format!(".{ATTEMPT_STATUS_FILE}.{}.tmp", std::process::id()));
    write_bytes_create_new(&temporary, &bytes)?;
    fs::rename(&temporary, &path)
        .map_err(|error| proof_error(format!("publishing attempt status {}: {error}", path.display())))?;
    fs::File::open(staging_dir)
        .and_then(|directory| directory.sync_all())
        .map_err(|error| proof_error(format!("syncing attempt status directory {}: {error}", staging_dir.display())))?;
    assert!(matches!(status, PROOF_STATUS_RUNNING | PROOF_STATUS_FAILED | PROOF_STATUS_COMPLETE));
    debug_assert!(path.is_file());
    Ok(())
}

pub(crate) fn write_json_create_new<T: Serialize>(path: &Path, value: &T) -> Result<(), RunError> {
    let file = fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(path)
        .map_err(|error| proof_error(format!("creating {}: {error}", path.display())))?;
    let mut writer = std::io::BufWriter::with_capacity(64 * 1_024, file);
    serde_json::to_writer_pretty(&mut writer, value)
        .map_err(|error| proof_error(format!("serializing {}: {error}", path.display())))?;
    writer
        .write_all(b"\n")
        .and_then(|()| writer.flush())
        .map_err(|error| proof_error(format!("writing {}: {error}", path.display())))?;
    writer
        .get_ref()
        .sync_all()
        .map_err(|error| proof_error(format!("syncing {}: {error}", path.display())))?;
    assert!(path.is_file());
    Ok(())
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
    use clap::Parser as _;

    use super::*;

    #[test]
    fn ignored_unknown_observation_error_cannot_publish_filtered_success() {
        let root = tempfile::tempdir().unwrap();
        let (proof, adapters, mut events) = crate::source_built_fixed_point_action_trust::tests::fixture();
        let plan = crate::source_built_fixed_point_planning::plan_actions(&proof, adapters).unwrap();
        let mut recorder = ActionObservationRecorder::create(root.path(), &proof, &plan).unwrap();
        let mut unknown = events.remove(0);
        unknown.action_id = "unplanned/child".into();
        assert!(recorder.record(unknown).is_err());
        assert!(recorder.finish(&proof).is_err());
        let journal = fs::read_to_string(root.path().join(ACTION_JOURNAL_FILE)).unwrap();
        let preserved: ActionExecutionEvent = serde_json::from_str(journal.trim_end()).unwrap();
        assert_eq!(preserved.action_id, "unplanned/child");
        assert!(!root.path().join(ACTION_EVENTS_FILE).exists());
        assert!(!root.path().join(ACTION_RECONCILIATION_FILE).exists());
    }

    const DIGEST: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    const OTHER_DIGEST: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
    const RETAINED_TRANSITION_EXECUTION_ROOT_ENV: &str = "MANTLE_STAGE_X_TRANSITION_EXECUTION_ROOT";
    const RETAINED_TRANSITION_HANDOFF_ROOT_ENV: &str = "MANTLE_STAGE_X_TRANSITION_HANDOFF_ROOT";
    const RETAINED_TRANSITION_SOURCE_STATE_ENV: &str = "MANTLE_STAGE_X_TRANSITION_SOURCE_STATE";
    const OPEN_FILE_LIMIT_CHILD_ENV: &str = "MANTLE_TEST_OPEN_FILE_LIMIT_CHILD";
    const OPEN_FILE_LIMIT_TEST_MAX: u64 = 256;

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
    fn promoted_cold_environment_rejects_cache_and_scheduling_keys_without_reading_values() {
        for rejected in [
            "MANTLE_CC_CACHE_POLICY",
            "MANTLE_CC_CACHE_SOCKET",
            "MANTLE_CC_CACHE_FUTURE",
            "MANTLE_COMPILE_BUILD_ID",
            "MANTLE_COMPILE_SLOT_SOCKET",
            "MANTLE_COMPILE_FUTURE",
        ] {
            let error = validate_cold_compiler_environment(["PATH", rejected]).unwrap_err();
            assert!(error.to_string().contains(rejected));
        }
        validate_cold_compiler_environment([
            "PATH",
            "MANTLE_LOG",
            "MANTLE_CC_CACHED",
            "MANTLE_COMPILER",
            "OTHER_COMPILE_BUILD_ID",
        ])
        .unwrap();
    }

    #[test]
    fn non_utf8_compiler_cache_variable_name_remains_disqualifying() {
        use std::os::unix::ffi::OsStringExt as _;
        let name = std::ffi::OsString::from_vec(b"MANTLE_CC_CACHE_\xff".to_vec());
        assert!(
            validate_cold_compiler_environment([name])
                .unwrap_err()
                .to_string()
                .contains("compiler-cache/scheduling")
        );
    }

    #[test]
    fn fixed_point_sandbox_shell_rejects_dynamic_linker_markers() {
        let temp = tempfile::tempdir().unwrap();
        let shell = temp.path().join("sandbox-shell");
        fs::write(&shell, b"\x7fELF-static-candidate").unwrap();
        validate_static_executable("sandbox shell", &shell).unwrap();

        fs::write(&shell, b"\x7fELF-dynamic-/lib/ld-linux-x86-64.so.2").unwrap();
        let error = validate_static_executable("sandbox shell", &shell).unwrap_err();
        assert!(matches!(error, RunError::Build(_)));
        assert_eq!(fs::read(&shell).unwrap(), b"\x7fELF-dynamic-/lib/ld-linux-x86-64.so.2");
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
            dev_provider_cache: None,
            dev_resume: false,
            dev_fast_fail: false,
            verbose: false,
            backend: crunch_store::StoreBackend::Snix,
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
        let mut invalid_limits = options.clone();
        invalid_limits.elapsed_seconds_max = 0;
        let zero = validate_options(&invalid_limits).unwrap_err();
        assert!(zero.to_string().contains("resource bounds are zero or exceed"));
        invalid_limits.elapsed_seconds_max = 86_401;
        let excessive = validate_options(&invalid_limits).unwrap_err();
        assert!(excessive.to_string().contains("resource bounds are zero or exceed"));
        invalid_limits.elapsed_seconds_max = 1;
        invalid_limits.disk_bytes_max = 0;
        assert!(validate_options(&invalid_limits).unwrap_err().to_string().contains("resource bounds"));
    }

    #[test]
    fn native_build_child_is_strict_offline_and_rejects_backend_mismatch() {
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
            dev_provider_cache: None,
            dev_resume: false,
            dev_fast_fail: false,
            verbose: false,
            backend: crunch_store::StoreBackend::Casita,
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
        let runtime = tokio::runtime::Runtime::new().unwrap();
        runtime
            .block_on(crunch_store::StoreHandle::open(crunch_store::StoreConfig::new(
                crunch_store::StoreBackend::Snix,
                prepared.native_state_dir.clone(),
                prepared.native_store_dir.clone(),
                LOGICAL_STORE_PREFIX.to_string(),
            )))
            .unwrap();
        let identity_path = prepared.native_state_dir.join("store-identity.json");
        let identity_before = fs::read(&identity_path).unwrap();
        let state_count_before = fs::read_dir(&prepared.native_state_dir).unwrap().count();
        let command =
            native_build_command(&options, &prepared, Path::new("/source/bootstrap/seed-full-toolchain.ncl")).unwrap();
        let args = command.get_args().map(OsString::from).collect::<Vec<_>>();

        assert!(args.iter().any(|argument| argument == "--offline-source-preflight"));
        assert!(args.iter().any(|argument| argument == "--no-substitute"));
        assert!(args.iter().any(|argument| argument == "--strict-hermetic"));
        assert!(!args.iter().any(|argument| argument == "--impure"));
        let argv = std::iter::once(OsString::from("mantle")).chain(args).collect::<Vec<_>>();
        let parsed = std::thread::Builder::new()
            .stack_size(crate::command_input::test_support::CLI_PARSE_TEST_STACK_BYTES)
            .spawn(move || crate::Args::try_parse_from(argv))
            .unwrap()
            .join()
            .unwrap()
            .unwrap();
        let mismatch = crunch_store::StoreConfig::preflight_backend_identity_for(
            parsed.store_backend,
            &prepared.native_state_dir,
            LOGICAL_STORE_PREFIX,
            &[],
            None,
        )
        .unwrap_err();
        assert!(mismatch.to_string().contains("store-backend-mismatch: requested casita, state declares snix"));
        assert_eq!(fs::read(&identity_path).unwrap(), identity_before);
        assert_eq!(fs::read_dir(&prepared.native_state_dir).unwrap().count(), state_count_before);
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
    fn failed_status_update_preserves_last_complete_status() {
        let temp = tempfile::tempdir().unwrap();
        let plan_digest = "a".repeat(BLAKE3_HEX_LENGTH);
        write_attempt_status(temp.path(), Some(&plan_digest), PROOF_STATUS_RUNNING, None).unwrap();
        let blocked_temporary = temp.path().join(format!(".{ATTEMPT_STATUS_FILE}.{}.tmp", std::process::id()));
        fs::create_dir(&blocked_temporary).unwrap();
        assert!(
            write_attempt_status(temp.path(), Some(&plan_digest), PROOF_STATUS_FAILED, Some("execution denied"),)
                .is_err()
        );
        let previous: serde_json::Value =
            serde_json::from_slice(&fs::read(temp.path().join(ATTEMPT_STATUS_FILE)).unwrap()).unwrap();
        assert_eq!(previous["status"], PROOF_STATUS_RUNNING);
        assert_eq!(previous["plan_digest_blake3"], plan_digest);
        fs::remove_dir(blocked_temporary).unwrap();
        write_attempt_status(temp.path(), Some(&plan_digest), PROOF_STATUS_FAILED, Some("execution denied")).unwrap();
        let failed: serde_json::Value =
            serde_json::from_slice(&fs::read(temp.path().join(ATTEMPT_STATUS_FILE)).unwrap()).unwrap();
        assert_eq!(failed["status"], PROOF_STATUS_FAILED);
        assert_eq!(failed["blocker"], "execution denied");
    }

    #[test]
    fn published_selection_readback_rejects_a_repointed_generation() {
        let temp = tempfile::tempdir().unwrap();
        for generation in ["old-proof", "new-proof"] {
            fs::create_dir(temp.path().join(generation)).unwrap();
        }
        let aliases = [LATEST_ALIAS, LATEST_SOURCE_BUILT_ALIAS];
        update_success_aliases(temp.path(), &aliases, std::ffi::OsStr::new("new-proof")).unwrap();
        let pointer = temp.path().join(SUCCESS_PUBLICATION_POINTER);
        fs::remove_file(&pointer).unwrap();
        std::os::unix::fs::symlink("old-proof", &pointer).unwrap();

        assert!(matches!(
            verify_success_selection(temp.path(), &aliases, std::ffi::OsStr::new("new-proof")),
            Err(RunError::Build(_))
        ));
        for alias in aliases {
            assert_eq!(fs::canonicalize(temp.path().join(alias)).unwrap(), temp.path().join("old-proof"));
        }
    }

    #[test]
    fn alias_publication_failure_preserves_candidate_artifacts_and_prior_success() {
        let temp = tempfile::tempdir().unwrap();
        let staging = temp.path().join("proof.staging");
        let mut prepared = prepared_fixture(&temp, staging.clone());
        prepared.final_dir = temp.path().join("proof");
        fs::write(staging.join("retained-evidence.txt"), b"candidate artifacts").unwrap();
        let old = temp.path().join("old-proof");
        fs::create_dir(&old).unwrap();
        std::os::unix::fs::symlink("old-proof", temp.path().join(LATEST_ALIAS)).unwrap();
        fs::create_dir(temp.path().join(LATEST_SOURCE_BUILT_ALIAS)).unwrap();

        assert!(publish_attempt(&prepared).is_err());
        assert_eq!(fs::read(prepared.final_dir.join("retained-evidence.txt")).unwrap(), b"candidate artifacts");
        assert_eq!(fs::canonicalize(temp.path().join(LATEST_ALIAS)).unwrap(), old);
        assert!(temp.path().join(LATEST_SOURCE_BUILT_ALIAS).is_dir());
    }

    #[test]
    fn interrupted_alias_migration_never_selects_part_of_the_new_generation() {
        let temp = tempfile::tempdir().unwrap();
        let parent = temp.path();
        for (name, generation) in [("old-proof", "old"), ("new-proof", "new")] {
            fs::create_dir(parent.join(name)).unwrap();
            fs::write(parent.join(name).join("generation"), generation).unwrap();
        }
        for alias in [LATEST_ALIAS, LATEST_SOURCE_BUILT_ALIAS] {
            std::os::unix::fs::symlink("old-proof", parent.join(alias)).unwrap();
        }
        // Fail after both names can have migrated to the shared old pointer,
        // but before the sole operation that can expose the new generation.
        let temporary = parent.join(format!("{SUCCESS_PUBLICATION_POINTER}.{}.tmp", std::process::id()));
        fs::create_dir(&temporary).unwrap();
        assert!(
            update_success_aliases(
                parent,
                &[LATEST_ALIAS, LATEST_SOURCE_BUILT_ALIAS],
                std::ffi::OsStr::new("new-proof")
            )
            .is_err()
        );
        for alias in [LATEST_ALIAS, LATEST_SOURCE_BUILT_ALIAS] {
            assert_eq!(fs::read_to_string(parent.join(alias).join("generation")).unwrap(), "old");
        }
        fs::remove_dir(temporary).unwrap();
        update_success_aliases(parent, &[LATEST_ALIAS, LATEST_SOURCE_BUILT_ALIAS], std::ffi::OsStr::new("new-proof"))
            .unwrap();
        for alias in [LATEST_ALIAS, LATEST_SOURCE_BUILT_ALIAS] {
            assert_eq!(fs::read_to_string(parent.join(alias).join("generation")).unwrap(), "new");
        }
    }

    #[test]
    fn disagreeing_success_aliases_cannot_authorize_a_new_publication() {
        let temp = tempfile::tempdir().unwrap();
        std::os::unix::fs::symlink("old-a", temp.path().join(LATEST_ALIAS)).unwrap();
        std::os::unix::fs::symlink("old-b", temp.path().join(LATEST_SOURCE_BUILT_ALIAS)).unwrap();
        assert!(
            update_success_aliases(
                temp.path(),
                &[LATEST_ALIAS, LATEST_SOURCE_BUILT_ALIAS],
                std::ffi::OsStr::new("new-proof")
            )
            .is_err()
        );
        assert_eq!(fs::read_link(temp.path().join(LATEST_ALIAS)).unwrap(), Path::new("old-a"));
        assert_eq!(fs::read_link(temp.path().join(LATEST_SOURCE_BUILT_ALIAS)).unwrap(), Path::new("old-b"));
    }

    #[test]
    fn partially_present_prior_aliases_are_rejected_without_creating_a_selection() {
        let temp = tempfile::tempdir().unwrap();
        std::os::unix::fs::symlink("old-proof", temp.path().join(LATEST_ALIAS)).unwrap();
        assert!(
            update_success_aliases(
                temp.path(),
                &[LATEST_ALIAS, LATEST_SOURCE_BUILT_ALIAS],
                std::ffi::OsStr::new("new-proof")
            )
            .is_err()
        );
        assert_eq!(fs::read_link(temp.path().join(LATEST_ALIAS)).unwrap(), Path::new("old-proof"));
        assert!(fs::symlink_metadata(temp.path().join(LATEST_SOURCE_BUILT_ALIAS)).is_err());
        assert!(fs::symlink_metadata(temp.path().join(SUCCESS_PUBLICATION_POINTER)).is_err());
    }

    #[test]
    fn success_aliases_cannot_publish_a_cycle_or_escape_the_root() {
        let temp = tempfile::tempdir().unwrap();
        for rejected in [
            "../proof",
            "/tmp/proof",
            SUCCESS_PUBLICATION_POINTER,
            LATEST_ALIAS,
            LATEST_SOURCE_BUILT_ALIAS,
            SUCCESS_PREVIOUS_POINTER,
            SUCCESS_PUBLICATION_LOCK,
        ] {
            assert!(
                update_success_aliases(
                    temp.path(),
                    &[LATEST_ALIAS, LATEST_SOURCE_BUILT_ALIAS],
                    std::ffi::OsStr::new(rejected)
                )
                .is_err()
            );
            assert!(fs::symlink_metadata(temp.path().join(LATEST_ALIAS)).is_err());
            assert!(fs::symlink_metadata(temp.path().join(LATEST_SOURCE_BUILT_ALIAS)).is_err());
        }
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

        let logical =
            register_adopted_provider(&provider, &store_dir, &state_dir, crunch_store::StoreBackend::Snix).unwrap();

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
    }

    #[test]
    fn dev_resume_restores_into_a_different_fresh_attempt() {
        let temp = tempfile::tempdir().unwrap();
        let cache = temp.path().join("cache");
        let profile = temp.path().join("profile.json");
        let executable = temp.path().join("bwrap");
        let first_out = temp.path().join("first-proof");
        let next_out = temp.path().join("next-proof");
        let first = options_fixture(&profile, &first_out, &executable, Some(&cache), true, false);
        let next = options_fixture(&profile, &next_out, &executable, Some(&cache), true, false);
        let prepared = prepared_fixture(&temp, temp.path().join("first-staging"));
        let payload = prepared.staging_dir.join(STAGEX_TRANSITION_EXECUTION_DIR);
        fs::create_dir(&payload).unwrap();
        fs::write(payload.join("transition-output"), b"completed transition").unwrap();
        let outputs = [ResumeOutputReference {
            role: "transition-execution-tree".into(),
            digest_blake3: DIGEST.into(),
        }];
        publish_dev_stage_bundle(&first, &prepared, "stagex-transition", &payload, &outputs).unwrap();
        let fresh = prepared_fixture(&temp, temp.path().join("next-staging"));
        let plan = plan_dev_resume(&fresh, &next).unwrap();
        assert_eq!(plan.restored_stages[0].stage_id, "stagex-transition");
        let digest = plan.restored_stages[0].restored_bundle_digest_blake3.as_ref().unwrap();
        let destination = fresh.staging_dir.join(STAGEX_TRANSITION_EXECUTION_DIR);
        restore_dev_stage_bundle(&next, &fresh, digest, &destination).unwrap();
        assert_eq!(fs::read(destination.join("transition-output")).unwrap(), b"completed transition");
        assert!(!first_out.exists());
        assert!(!next_out.exists());
        fs::write(cache.join(RESUME_BUNDLES_SUBDIR).join(digest).join("payload/transition-output"), b"tampered")
            .unwrap();
        let rejected = plan_dev_resume(&fresh, &next).unwrap();
        assert!(rejected.restored_stages.is_empty());
        assert!(rejected.blockers.iter().any(|blocker| blocker.code == "resume-mutated-bundle"));
    }

    fn prepared_fixture(temp: &tempfile::TempDir, staging: PathBuf) -> PreparedAttempt {
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

    fn options_fixture<'a>(
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
            dev_provider_cache: cache,
            dev_resume: resume,
            dev_fast_fail: fast_fail,
            verbose: false,
            backend: crunch_store::StoreBackend::Snix,
            json: false,
        }
    }

    #[test]
    fn every_development_mode_is_disqualified_before_cache_or_resume_execution() {
        let temp = tempfile::tempdir().unwrap();
        let profile = temp.path().join("profile.json");
        let out = temp.path().join("proof");
        let executable = temp.path().join("bwrap");
        let cold = options_fixture(&profile, &out, &executable, None, false, false);
        assert!(!is_development_run(&cold));
        crate::source_built_fixed_point_trust_report::reject_development_authority(temp.path()).unwrap();
        for (index, (cache, resume, fast_fail)) in [
            (None, true, false),
            (None, false, true),
            (Some(temp.path()), false, false),
        ]
        .into_iter()
        .enumerate()
        {
            let root = temp.path().join(format!("dev-{index}"));
            fs::create_dir(&root).unwrap();
            let options = options_fixture(&profile, &out, &executable, cache, resume, fast_fail);
            assert!(is_development_run(&options));
            write_development_authority(&root, &options, &test_plan()).unwrap();
            assert!(crate::source_built_fixed_point_trust_report::reject_development_authority(&root).is_err());
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

    fn write_executable(path: &Path) -> PathBuf {
        fs::write(path, b"\x7fELFstatic-tool").unwrap();
        fs::set_permissions(path, fs::Permissions::from_mode(0o755)).unwrap();
        path.to_path_buf()
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
                open_file_descriptors_max: OPEN_FILE_LIMIT_TEST_MAX,
                protected_exec_events_max: 1,
                source_records_max: 1,
            },
        })
        .unwrap()
    }
}
