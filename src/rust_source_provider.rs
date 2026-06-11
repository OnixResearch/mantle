//! Shell/orchestration for the source-built Rust compiler/sysroot provider.
//!
//! This module performs filesystem work around the pure provider metadata
//! validator in `source_toolchain_closure`. It deliberately fails closed for
//! materialization until Mantle has a real Rust-from-source bootstrap path; the
//! validation entry points are ready for a future materializer output and refuse
//! prebuilt/wrapper metadata in the pure core.

use std::ffi::OsString;
use std::fs;
use std::fs::File;
use std::io::Read;
use std::path::Component;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

use crate::source_toolchain_closure::RUST_SOURCE_PROVIDER_ID;
use crate::source_toolchain_closure::RUST_SOURCE_PROVIDER_METADATA_PATH;
use crate::source_toolchain_closure::RUST_SOURCE_PROVIDER_RECEIPT_SCHEMA;
use crate::source_toolchain_closure::RustProviderArtifact;
use crate::source_toolchain_closure::RustProviderBuildReceiptIdentity;
use crate::source_toolchain_closure::RustProviderObservedArtifact;
use crate::source_toolchain_closure::RustProviderObservedBuildReceipt;
use crate::source_toolchain_closure::RustProviderReceiptArtifact;
use crate::source_toolchain_closure::RustProviderReceiptStep;
use crate::source_toolchain_closure::RustProviderRole;
use crate::source_toolchain_closure::RustProviderSourceIdentity;
use crate::source_toolchain_closure::RustSourceProviderBootstrapOutput;
use crate::source_toolchain_closure::RustSourceProviderBootstrapPlan;
use crate::source_toolchain_closure::RustSourceProviderBootstrapPlanValidation;
use crate::source_toolchain_closure::RustSourceProviderBootstrapSource;
use crate::source_toolchain_closure::RustSourceProviderBootstrapStage;
use crate::source_toolchain_closure::RustSourceProviderBootstrapStageKind;
use crate::source_toolchain_closure::RustSourceProviderBuildReceipt;
use crate::source_toolchain_closure::RustSourceProviderMetadata;
use crate::source_toolchain_closure::RustSourceProviderProvenance;
use crate::source_toolchain_closure::RustSourceProviderValidation;
use crate::source_toolchain_closure::ToolchainBuildReceiptKind;
use crate::source_toolchain_closure::ToolchainSourceKind;

pub(crate) const RUST_SOURCE_PROVIDER_BLOCKED_REASON: &str = "source-built Rust provider materialization is not implemented: Mantle has no receipt-bound Rust-from-source bootstrap that can build rustc/cargo/rustlib without prebuilt Rust";

const DIRECTORY_DIGEST_MAX_ENTRIES: usize = 200_000;
const RUST_SOURCE_PROVIDER_PLAN_FILE: &str = "rust-source-plan.ncl";
const FIRST_STAGE_PLAN_SCHEMA: &str = "mantle-rust-source-provider-first-stage-plan-v1";
const FIRST_STAGE_PLAN_FILE: &str = "mrustc-first-stage-plan.json";
const FIRST_STAGE_SOURCES_MANIFEST_SCHEMA: &str = "mantle-rust-source-provider-first-stage-sources-v1";
const FIRST_STAGE_SOURCES_MANIFEST_FILE: &str = "mrustc-first-stage-sources.json";
const FIRST_STAGE_BUILD_MANIFEST_SCHEMA: &str = "mantle-rust-source-provider-first-stage-build-v1";
const FIRST_STAGE_BUILD_MANIFEST_FILE: &str = "mrustc-first-stage-build.json";
const FIRST_STAGE_BUILD_LOG_FILE: &str = "mrustc-first-stage-build.log";
const FIRST_STAGE_PROVIDER_CANDIDATE_SCHEMA: &str = "mantle-rust-source-provider-first-stage-candidate-v1";
const FIRST_STAGE_PROVIDER_CANDIDATE_DIR: &str = "mrustc-first-stage-provider-candidate";
const FIRST_STAGE_PROVIDER_CANDIDATE_MANIFEST_FILE: &str = "mrustc-first-stage-provider-candidate.json";
const FIRST_STAGE_PROVIDER_CANDIDATE_SMOKE_DIR: &str = "mrustc-first-stage-provider-candidate-smoke";
const FIRST_STAGE_PROVIDER_CANDIDATE_SMOKE_WORK_DIR: &str = "work";
const FIRST_STAGE_PROVIDER_RECEIPT_RELATIVE_PATH: &str = "share/mantle-rust-provider/receipts/build.json";
const FIRST_STAGE_PROVIDER_RECEIPT_ID: &str = "mrustc-first-stage-build";
const FIRST_STAGE_PROVIDER_RECEIPT_NAME: &str = "mrustc-first-stage-build-receipt";
const FIRST_STAGE_PROVIDER_BUILD_RECIPE: &str = "mrustc-first-stage-source-route";
const RUSTC_STAGE1_PLAN_SCHEMA: &str = "mantle-rust-source-provider-rustc-stage1-plan-v1";
const RUSTC_STAGE1_PLAN_FILE: &str = "rustc-stage1-plan.json";
const RUSTC_STAGE1_SOURCES_MANIFEST_SCHEMA: &str = "mantle-rust-source-provider-rustc-stage1-sources-v1";
const RUSTC_STAGE1_SOURCES_MANIFEST_FILE: &str = "rustc-stage1-sources.json";
const RUSTC_STAGE1_ARCHIVE_DIR: &str = "rustc-stage1-archives";
const RUSTC_STAGE1_SOURCE_DIR: &str = "rustc-stage1-sources";
const RUSTC_STAGE1_BUILD_DIR: &str = "rustc-stage1-build";
const RUSTC_STAGE1_OUTPUT_DIR: &str = "rustc-stage1-output";
const RUSTC_STAGE1_BUILD_MANIFEST_SCHEMA: &str = "mantle-rust-source-provider-rustc-stage1-build-v1";
const RUSTC_STAGE1_BUILD_MANIFEST_FILE: &str = "rustc-stage1-build.json";
const RUSTC_STAGE1_BUILD_LOG_FILE: &str = "rustc-stage1-build.log";
const RUSTC_STAGE1_SCRIPT_FILE: &str = "run-rustc-stage1.sh";
const RUSTC_STAGE1_CHAIN_DIR: &str = "rustc-stage1-chain";
const RUSTC_STAGE1_PROVIDER_CANDIDATE_SCHEMA: &str = "mantle-rust-source-provider-rustc-stage1-candidate-v1";
const RUSTC_STAGE1_PROVIDER_CANDIDATE_DIR: &str = "rustc-stage1-provider-candidate";
const RUSTC_STAGE1_PROVIDER_CANDIDATE_MANIFEST_FILE: &str = "rustc-stage1-provider-candidate.json";
const RUSTC_STAGE1_PROVIDER_CANDIDATE_SMOKE_DIR: &str = "rustc-stage1-provider-candidate-smoke";
const RUSTC_STAGE1_PROVIDER_CANDIDATE_SMOKE_WORK_DIR: &str = "work";
const RUSTC_STAGE1_PROVIDER_RECEIPT_RELATIVE_PATH: &str = "share/mantle-rust-provider/receipts/rustc-stage1-build.json";
const RUSTC_STAGE1_PROVIDER_RECEIPT_ID: &str = "rustc-stage1-build";
const RUSTC_STAGE1_PROVIDER_RECEIPT_NAME: &str = "rustc-stage1-build-receipt";
const RUSTC_STAGE1_PROVIDER_BUILD_RECIPE: &str = "rustc-stage1-source-route";
const RUSTC_STAGE1_SOURCE_BUILD_SCRIPT: &str = "mantle-rustc-stage1-build.sh";
const RUSTC_STAGE1_SHELL_PROGRAM: &str = "sh";
const FIRST_STAGE_SCRIPT_FILE: &str = "run-mrustc-first-stage.sh";
const FIRST_STAGE_ARCHIVE_DIR: &str = "archives";
const FIRST_STAGE_SOURCE_DIR: &str = "sources";
const FIRST_STAGE_BUILD_DIR: &str = "build";
const FIRST_STAGE_SOURCE_ARCHIVE_EXTENSION: &str = "tar.gz";
const FIRST_STAGE_TAR_GZ_EXTENSION: &str = ".tar.gz";
const FIRST_STAGE_TGZ_EXTENSION: &str = ".tgz";
const FIRST_STAGE_MISSING_SOURCE_EXIT_CODE: i32 = 2;
const FIRST_STAGE_BUILD_FAILED_EXIT_CODE: i32 = 3;
const RUSTC_STAGE1_BUILD_FAILED_EXIT_CODE: i32 = 4;
const FIRST_STAGE_FETCH_TIMEOUT_SECS: u64 = 600;
const FIRST_STAGE_FETCH_MAX_RETRIES: u32 = 3;
const FIRST_STAGE_FETCH_RETRY_BASE_DELAY_MS: u64 = 2_000;
const FIRST_STAGE_FETCH_MAX_MIB: u64 = 768;
const BYTES_PER_KIB: u64 = 1_024;
const BYTES_PER_MIB: u64 = BYTES_PER_KIB * BYTES_PER_KIB;
const FIRST_STAGE_FETCH_MAX_BYTES: u64 = FIRST_STAGE_FETCH_MAX_MIB * BYTES_PER_MIB;
const FIRST_STAGE_ARCHIVE_MAX_ENTRIES: u32 = 500_000;
const FIRST_STAGE_MRUSTC_SOURCE_PREFIX: &str = "mrustc-";
const FIRST_STAGE_RUST_SOURCE_PREFIX: &str = "rust-";
const FIRST_STAGE_MRUSTC_BINARY: &str = "bin/mrustc";
const FIRST_STAGE_MINICARGO_BINARY: &str = "bin/minicargo";
const FIRST_STAGE_TRANSLATED_RUSTC_BINARY: &str = "output/rustc";
const FIRST_STAGE_TRANSLATED_CARGO_BINARY: &str = "output/cargo";
const FIRST_STAGE_RUN_RUSTC_DIR: &str = "run_rustc";
const FIRST_STAGE_PREFIX_DIR: &str = "run_rustc/output/prefix";
const FIRST_STAGE_PREFIX_RUSTC_BINARY: &str = "run_rustc/output/prefix/bin/rustc";
const FIRST_STAGE_PREFIX_CARGO_BINARY: &str = "run_rustc/output/prefix/bin/cargo";
const FIRST_STAGE_MINICARGO_MAKEFILE: &str = "minicargo.mk";
const FIRST_STAGE_MAKEFILE: &str = "Makefile";
const FIRST_STAGE_MAKE_PROGRAM: &str = "make";
const FIRST_STAGE_COPY_PROGRAM: &str = "cp";
const FIRST_STAGE_CXXFLAGS: &str = "-g0 -O2 -D_LIBCPP_HARDENING_MODE=_LIBCPP_HARDENING_MODE_NONE";
const FIRST_STAGE_MAKE_FALLBACK_GLOB: &str = "/nix/store/*-gnumake-*/bin/make /nix/store/*-gnumake-static-*/bin/make";
const PROVIDER_RUSTC_RELATIVE_PATH: &str = "bin/rustc";
const PROVIDER_CARGO_RELATIVE_PATH: &str = "bin/cargo";
const PROVIDER_RUSTLIB_PREFIX: &str = "lib/rustlib";
const REQUIRED_FIRST_STAGE_OUTPUT_ROLES: [RustProviderRole; 4] = [
    RustProviderRole::Rustc,
    RustProviderRole::Cargo,
    RustProviderRole::HostRustlib,
    RustProviderRole::TargetRustlib,
];
const REQUIRED_RUSTC_STAGE1_OUTPUT_ROLES: [RustProviderRole; 4] = [
    RustProviderRole::Rustc,
    RustProviderRole::Cargo,
    RustProviderRole::HostRustlib,
    RustProviderRole::TargetRustlib,
];
const FIRST_STAGE_PROVIDER_RECEIPT_ARTIFACT_COUNT: usize = 1;
const RUSTC_STAGE1_PROVIDER_RECEIPT_ARTIFACT_COUNT: usize = 1;
const SMOKE_OUTPUT_CAPTURE_BYTES: usize = 16_384;
const SMOKE_SOURCE_FILE: &str = "mantle-rust-provider-smoke.rs";
const SMOKE_OUTPUT_FILE: &str = "libmantle_rust_provider_smoke.rlib";
const SMOKE_CRATE_NAME: &str = "mantle_rust_provider_smoke";
const SMOKE_RETURN_CODE: u32 = 42;
const SMOKE_EVIDENCE_SCHEMA: &str = "mantle-rust-source-provider-smoke-evidence-v1";
const SMOKE_EVIDENCE_SUMMARY_FILE: &str = "smoke.json";
const SMOKE_EVIDENCE_STDOUT_FILE: &str = "smoke-stdout.txt";
const SMOKE_EVIDENCE_STDERR_FILE: &str = "smoke-stderr.txt";
const SMOKE_EVIDENCE_SOURCE_FILE: &str = "smoke-source.rs";
const SMOKE_EVIDENCE_OUTPUT_FILE: &str = "smoke-output.rlib";
const SMOKE_EVIDENCE_METADATA_FILE: &str = "provider-metadata.json";
const RUST_PROVIDER_ARTIFACT_TEXT_PROBE_BYTES: usize = 4_096;
const RUST_PROVIDER_ARTIFACT_MARKER_SCAN_BYTES: usize = 65_536;

#[derive(Debug, Clone)]
pub(crate) struct RustSourceProviderMaterialization {
    pub(crate) output_path: PathBuf,
    pub(crate) recipe_digest_blake3: String,
    pub(crate) metadata_path: PathBuf,
    pub(crate) metadata_digest_blake3: String,
}

#[derive(Debug, Clone)]
pub(crate) struct RustSourceProviderDirectoryValidation {
    pub(crate) metadata_path: PathBuf,
    pub(crate) metadata_digest_blake3: String,
    pub(crate) metadata: RustSourceProviderMetadata,
    pub(crate) validation: RustSourceProviderValidation,
}

#[derive(Debug, Clone)]
pub(crate) struct RustSourceProviderImport {
    pub(crate) input_path: PathBuf,
    pub(crate) output_path: PathBuf,
    pub(crate) validation: RustSourceProviderDirectoryValidation,
}

#[derive(Debug, Clone)]
pub(crate) struct RustSourceProviderSmoke {
    pub(crate) provider_path: PathBuf,
    pub(crate) scratch_dir: PathBuf,
    pub(crate) rustc_path: PathBuf,
    pub(crate) source_path: PathBuf,
    pub(crate) output_path: PathBuf,
    pub(crate) output_digest_blake3: String,
    pub(crate) target_triple: String,
    pub(crate) stdout: String,
    pub(crate) stderr: String,
}

#[derive(Debug, Clone)]
pub(crate) struct RustSourceProviderSmokeEvidence {
    pub(crate) evidence_dir: PathBuf,
    pub(crate) summary_path: PathBuf,
    pub(crate) stdout_path: PathBuf,
    pub(crate) stderr_path: PathBuf,
    pub(crate) source_path: PathBuf,
    pub(crate) output_path: PathBuf,
    pub(crate) metadata_path: PathBuf,
    pub(crate) output_digest_blake3: String,
    pub(crate) metadata_digest_blake3: String,
    pub(crate) policy_digest_blake3: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RustSourceProviderMaterializationPlan {
    recipe_path: PathBuf,
    output_dir: PathBuf,
    scratch_dir: PathBuf,
    recipe_digest_blake3: String,
    route_plan_path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct LoadedRustSourceProviderRoute {
    plan_path: PathBuf,
    plan_digest_blake3: String,
    plan: RustSourceProviderBootstrapPlan,
    validation: RustSourceProviderBootstrapPlanValidation,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RustSourceProviderFirstStageBoundary {
    route_plan_path: PathBuf,
    route_plan_digest_blake3: String,
    route_policy_digest_blake3: String,
    stage_id: String,
    stage_kind: RustSourceProviderBootstrapStageKind,
    rust_version: String,
    target_triple: String,
    mrustc_target_version: String,
    source_ids: Vec<String>,
    sources: Vec<RustSourceProviderBootstrapSource>,
    expected_outputs: Vec<RustSourceProviderBootstrapOutput>,
    archive_dir: PathBuf,
    source_dir: PathBuf,
    build_dir: PathBuf,
    output_dir: PathBuf,
    plan_path: PathBuf,
    sources_manifest_path: PathBuf,
    build_manifest_path: PathBuf,
    build_log_path: PathBuf,
    provider_candidate_dir: PathBuf,
    provider_candidate_manifest_path: PathBuf,
    provider_candidate_smoke_dir: PathBuf,
    provider_candidate_smoke_work_dir: PathBuf,
    script_path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RustSourceProviderRustcStage1Boundary {
    route_plan_path: PathBuf,
    route_plan_digest_blake3: String,
    route_policy_digest_blake3: String,
    stage_id: String,
    stage_kind: RustSourceProviderBootstrapStageKind,
    bootstrap_stage_id: String,
    rust_version: String,
    host_triple: String,
    target_triple: String,
    source_ids: Vec<String>,
    sources: Vec<RustSourceProviderBootstrapSource>,
    expected_outputs: Vec<RustSourceProviderBootstrapOutput>,
    bootstrap_provider_candidate_dir: PathBuf,
    bootstrap_provider_metadata_digest_blake3: String,
    bootstrap_provider_policy_digest_blake3: String,
    archive_dir: PathBuf,
    source_dir: PathBuf,
    build_dir: PathBuf,
    stage_output_dir: PathBuf,
    output_dir: PathBuf,
    plan_path: PathBuf,
    sources_manifest_path: PathBuf,
    build_manifest_path: PathBuf,
    build_log_path: PathBuf,
    script_path: PathBuf,
    provider_candidate_dir: PathBuf,
    provider_candidate_manifest_path: PathBuf,
    provider_candidate_smoke_dir: PathBuf,
    provider_candidate_smoke_work_dir: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
struct RustSourceProviderFirstStageSourceAcquisition {
    id: String,
    url: String,
    archive_path: PathBuf,
    archive_sha256_hex: String,
    extracted_path: PathBuf,
    extracted_digest_blake3: String,
    archive_entry_count: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
struct RustSourceProviderFirstStageBuild {
    stage_id: String,
    script_path: PathBuf,
    log_path: PathBuf,
    mrustc_path: PathBuf,
    mrustc_digest_blake3: String,
    minicargo_path: PathBuf,
    minicargo_digest_blake3: String,
    translated_rustc_path: PathBuf,
    translated_rustc_digest_blake3: String,
    translated_cargo_path: PathBuf,
    translated_cargo_digest_blake3: String,
    prefix_path: PathBuf,
    prefix_digest_blake3: String,
    prefix_rustlib_path: PathBuf,
    prefix_rustlib_digest_blake3: String,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
struct RustSourceProviderFirstStageProviderCandidate {
    stage_id: String,
    candidate_dir: PathBuf,
    metadata_path: PathBuf,
    metadata_digest_blake3: String,
    policy_digest_blake3: String,
    artifact_count: usize,
    source_count: usize,
    receipt_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RustSourceProviderBootstrapProviderCandidate {
    stage_id: String,
    candidate_dir: PathBuf,
    metadata_digest_blake3: String,
    policy_digest_blake3: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RustcStage1ScratchLayout {
    TopLevel,
    Scoped,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
struct RustSourceProviderRustcStage1Build {
    stage_id: String,
    script_path: PathBuf,
    log_path: PathBuf,
    stage_output_dir: PathBuf,
    stage_output_digest_blake3: String,
    rustc_path: PathBuf,
    rustc_digest_blake3: String,
    cargo_path: PathBuf,
    cargo_digest_blake3: String,
    host_rustlib_path: PathBuf,
    host_rustlib_digest_blake3: String,
    target_rustlib_path: PathBuf,
    target_rustlib_digest_blake3: String,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
struct RustSourceProviderRustcStage1ProviderCandidate {
    stage_id: String,
    candidate_dir: PathBuf,
    metadata_path: PathBuf,
    metadata_digest_blake3: String,
    policy_digest_blake3: String,
    artifact_count: usize,
    source_count: usize,
    receipt_count: usize,
}

#[derive(Debug, Clone)]
struct RustSourceProviderRustcStage1ProviderCandidateRun {
    boundary: RustSourceProviderRustcStage1Boundary,
    candidate: RustSourceProviderRustcStage1ProviderCandidate,
    smoke: RustSourceProviderSmokeEvidence,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RustSourceProviderFirstStageBuildSources {
    mrustc_id: String,
    rust_id: String,
    mrustc_path: PathBuf,
    rust_path: PathBuf,
    mrustc_archive_path: PathBuf,
    rust_archive_path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RustSourceProviderSmokePlan {
    provider_dir: PathBuf,
    scratch_dir: PathBuf,
    rustc_path: PathBuf,
    source_path: PathBuf,
    output_path: PathBuf,
    target_triple: String,
}

#[derive(Debug)]
pub(crate) enum RustSourceProviderError {
    Read(String),
    Parse(String),
    Validate(String),
    MissingArtifact(String),
    Digest(String),
    Fetch(String),
    Extract(String),
    Build(String),
    Copy(String),
    Smoke(String),
    Blocked {
        recipe_path: PathBuf,
        recipe_digest_blake3: String,
        route_plan_path: Option<PathBuf>,
        route_plan_digest_blake3: Option<String>,
        first_stage_id: Option<String>,
        first_stage_script_path: Option<PathBuf>,
        reason: &'static str,
    },
}

impl std::fmt::Display for RustSourceProviderError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Read(message) => write!(formatter, "read: {message}"),
            Self::Parse(message) => write!(formatter, "parse: {message}"),
            Self::Validate(message) => write!(formatter, "validate: {message}"),
            Self::MissingArtifact(message) => write!(formatter, "missing artifact: {message}"),
            Self::Digest(message) => write!(formatter, "digest: {message}"),
            Self::Fetch(message) => write!(formatter, "fetch: {message}"),
            Self::Extract(message) => write!(formatter, "extract: {message}"),
            Self::Build(message) => write!(formatter, "build: {message}"),
            Self::Copy(message) => write!(formatter, "copy: {message}"),
            Self::Smoke(message) => write!(formatter, "smoke: {message}"),
            Self::Blocked {
                recipe_path,
                recipe_digest_blake3,
                route_plan_path,
                route_plan_digest_blake3,
                first_stage_id,
                first_stage_script_path,
                reason,
            } => {
                write!(
                    formatter,
                    "{reason}; recipe={} recipe_digest_blake3={recipe_digest_blake3}",
                    recipe_path.display()
                )?;
                if let Some(route_plan_path) = route_plan_path {
                    write!(formatter, " route_plan={}", route_plan_path.display())?;
                }
                if let Some(route_plan_digest_blake3) = route_plan_digest_blake3 {
                    write!(formatter, " route_plan_digest_blake3={route_plan_digest_blake3}")?;
                }
                if let Some(first_stage_id) = first_stage_id {
                    write!(formatter, " first_stage_id={first_stage_id}")?;
                }
                if let Some(first_stage_script_path) = first_stage_script_path {
                    write!(formatter, " first_stage_script={}", first_stage_script_path.display())?;
                }
                Ok(())
            }
        }
    }
}

impl std::error::Error for RustSourceProviderError {}

pub(crate) fn materialize_rust_source_provider(
    recipe_path: &Path,
    output_dir: &Path,
    scratch_dir: &Path,
    verbose: bool,
) -> Result<RustSourceProviderMaterialization, RustSourceProviderError> {
    let recipe_bytes = fs::read(recipe_path)
        .map_err(|err| RustSourceProviderError::Read(format!("recipe {}: {err}", recipe_path.display())))?;
    if recipe_bytes.is_empty() {
        return Err(RustSourceProviderError::Read(format!("recipe {} is empty", recipe_path.display())));
    }
    let recipe_digest_blake3 = blake3::hash(&recipe_bytes).to_hex().to_string();
    let plan = plan_materialization(recipe_path, output_dir, scratch_dir, &recipe_digest_blake3)?;
    let route = load_rust_source_provider_route(&plan.route_plan_path)?;
    let boundary = prepare_first_stage_boundary(&plan, &route)?;
    write_first_stage_boundary(&boundary)?;
    let first_stage_sources = acquire_first_stage_sources(&boundary, verbose)?;
    write_first_stage_sources_manifest(&boundary, &first_stage_sources)?;
    let first_stage_build = run_first_stage_build(&boundary, verbose)?;
    write_first_stage_build_manifest(&boundary, &first_stage_build)?;
    let first_stage_candidate =
        assemble_first_stage_provider_candidate(&boundary, &route, &first_stage_sources, &first_stage_build)?;
    let first_stage_candidate_smoke = smoke_first_stage_provider_candidate(&boundary, &first_stage_candidate)?;
    write_first_stage_provider_candidate_manifest(&boundary, &first_stage_candidate)?;
    let first_stage_bootstrap_candidate = bootstrap_candidate_from_first_stage(&first_stage_candidate);
    let rustc_stage1_run = run_rustc_stage1_provider_candidate(
        &plan,
        &route,
        &first_stage_bootstrap_candidate,
        RustcStage1ScratchLayout::TopLevel,
        verbose,
    )?;
    let rustc_stage1_bootstrap_candidate = bootstrap_candidate_from_rustc_stage1(&rustc_stage1_run.candidate);
    let rustc_stage1_next_run = run_rustc_stage1_provider_candidate(
        &plan,
        &route,
        &rustc_stage1_bootstrap_candidate,
        RustcStage1ScratchLayout::Scoped,
        verbose,
    )?;
    if verbose {
        eprintln!("Rust source provider recipe: {}", plan.recipe_path.display());
        eprintln!("  recipe_digest_blake3: {}", plan.recipe_digest_blake3);
        eprintln!("  route_plan: {}", boundary.route_plan_path.display());
        eprintln!("  route_plan_digest_blake3: {}", boundary.route_plan_digest_blake3);
        eprintln!("  route_policy_digest_blake3: {}", boundary.route_policy_digest_blake3);
        eprintln!("  first_stage_id: {}", boundary.stage_id);
        eprintln!("  first_stage_script: {}", boundary.script_path.display());
        eprintln!("  first_stage_plan: {}", boundary.plan_path.display());
        eprintln!("  first_stage_sources: {}", boundary.sources_manifest_path.display());
        eprintln!("  first_stage_build: {}", boundary.build_manifest_path.display());
        eprintln!("  first_stage_build_log: {}", boundary.build_log_path.display());
        eprintln!("  first_stage_provider_candidate: {}", first_stage_candidate.candidate_dir.display());
        eprintln!("  first_stage_provider_candidate_manifest: {}", boundary.provider_candidate_manifest_path.display());
        eprintln!("  first_stage_provider_candidate_smoke: {}", first_stage_candidate_smoke.summary_path.display());
        eprintln!("  rustc_stage1_id: {}", rustc_stage1_run.boundary.stage_id);
        eprintln!("  rustc_stage1_plan: {}", rustc_stage1_run.boundary.plan_path.display());
        eprintln!("  rustc_stage1_sources: {}", rustc_stage1_run.boundary.sources_manifest_path.display());
        eprintln!("  rustc_stage1_build: {}", rustc_stage1_run.boundary.build_manifest_path.display());
        eprintln!("  rustc_stage1_build_log: {}", rustc_stage1_run.boundary.build_log_path.display());
        eprintln!("  rustc_stage1_provider_candidate: {}", rustc_stage1_run.candidate.candidate_dir.display());
        eprintln!(
            "  rustc_stage1_provider_candidate_manifest: {}",
            rustc_stage1_run.boundary.provider_candidate_manifest_path.display()
        );
        eprintln!("  rustc_stage1_provider_candidate_smoke: {}", rustc_stage1_run.smoke.summary_path.display());
        eprintln!("  rustc_stage1_next_id: {}", rustc_stage1_next_run.boundary.stage_id);
        eprintln!("  rustc_stage1_next_plan: {}", rustc_stage1_next_run.boundary.plan_path.display());
        eprintln!("  rustc_stage1_next_sources: {}", rustc_stage1_next_run.boundary.sources_manifest_path.display());
        eprintln!("  rustc_stage1_next_build: {}", rustc_stage1_next_run.boundary.build_manifest_path.display());
        eprintln!("  rustc_stage1_next_build_log: {}", rustc_stage1_next_run.boundary.build_log_path.display());
        eprintln!(
            "  rustc_stage1_next_provider_candidate: {}",
            rustc_stage1_next_run.candidate.candidate_dir.display()
        );
        eprintln!(
            "  rustc_stage1_next_provider_candidate_manifest: {}",
            rustc_stage1_next_run.boundary.provider_candidate_manifest_path.display()
        );
        eprintln!(
            "  rustc_stage1_next_provider_candidate_smoke: {}",
            rustc_stage1_next_run.smoke.summary_path.display()
        );
        eprintln!("  planned_output: {}", plan.output_dir.display());
        eprintln!("  planned_scratch: {}", plan.scratch_dir.display());
    }
    Err(RustSourceProviderError::Blocked {
        recipe_path: plan.recipe_path,
        recipe_digest_blake3: plan.recipe_digest_blake3,
        route_plan_path: Some(boundary.route_plan_path),
        route_plan_digest_blake3: Some(boundary.route_plan_digest_blake3),
        first_stage_id: Some(boundary.stage_id),
        first_stage_script_path: Some(boundary.script_path),
        reason: RUST_SOURCE_PROVIDER_BLOCKED_REASON,
    })
}

pub(crate) fn import_rust_source_provider(
    import_dir: &Path,
    output_dir: &Path,
) -> Result<RustSourceProviderImport, RustSourceProviderError> {
    validate_import_request(import_dir, output_dir)?;
    validate_materialized_rust_source_provider(import_dir)?;
    copy_provider_tree(import_dir, output_dir)?;
    let validation = match validate_materialized_rust_source_provider(output_dir) {
        Ok(validation) => validation,
        Err(err) => {
            let _ = remove_import_output(output_dir);
            return Err(err);
        }
    };
    Ok(RustSourceProviderImport {
        input_path: import_dir.to_path_buf(),
        output_path: output_dir.to_path_buf(),
        validation,
    })
}

pub(crate) fn smoke_rust_source_provider(
    provider_dir: &Path,
    scratch_dir: &Path,
) -> Result<RustSourceProviderSmoke, RustSourceProviderError> {
    let validation = validate_materialized_rust_source_provider(provider_dir)?;
    let plan = plan_smoke(provider_dir, scratch_dir, &validation.metadata)?;
    fs::create_dir_all(&plan.scratch_dir).map_err(|err| {
        RustSourceProviderError::Smoke(format!("create smoke scratch {}: {err}", plan.scratch_dir.display()))
    })?;
    let source = smoke_source_text();
    fs::write(&plan.source_path, source).map_err(|err| {
        RustSourceProviderError::Smoke(format!("write smoke source {}: {err}", plan.source_path.display()))
    })?;
    let output = Command::new(&plan.rustc_path)
        .env_clear()
        .arg("--sysroot")
        .arg(&plan.provider_dir)
        .arg("--target")
        .arg(&plan.target_triple)
        .arg("--crate-name")
        .arg(SMOKE_CRATE_NAME)
        .arg("--crate-type")
        .arg("lib")
        .arg("--emit")
        .arg("link")
        .arg(&plan.source_path)
        .arg("-o")
        .arg(&plan.output_path)
        .output()
        .map_err(|err| RustSourceProviderError::Smoke(format!("launch {}: {err}", plan.rustc_path.display())))?;
    let stdout = bounded_output_text(&output.stdout);
    let stderr = bounded_output_text(&output.stderr);
    if !output.status.success() {
        return Err(RustSourceProviderError::Smoke(format!(
            "rustc smoke failed with status {}; stdout={stdout:?}; stderr={stderr:?}",
            output.status
        )));
    }
    if !plan.output_path.is_file() {
        return Err(RustSourceProviderError::Smoke(format!(
            "smoke output {} was not created",
            plan.output_path.display()
        )));
    }
    let output_digest_blake3 = file_digest_blake3(&plan.output_path)?;
    Ok(RustSourceProviderSmoke {
        provider_path: plan.provider_dir,
        scratch_dir: plan.scratch_dir,
        rustc_path: plan.rustc_path,
        source_path: plan.source_path,
        output_path: plan.output_path,
        output_digest_blake3,
        target_triple: plan.target_triple,
        stdout,
        stderr,
    })
}

pub(crate) fn persist_rust_source_provider_smoke_evidence(
    evidence_dir: &Path,
    smoke: &RustSourceProviderSmoke,
    validation: &RustSourceProviderDirectoryValidation,
) -> Result<RustSourceProviderSmokeEvidence, RustSourceProviderError> {
    if evidence_dir.as_os_str().is_empty() {
        return Err(RustSourceProviderError::Smoke("smoke evidence dir is empty".to_string()));
    }
    fs::create_dir_all(evidence_dir).map_err(|err| {
        RustSourceProviderError::Smoke(format!("create smoke evidence dir {}: {err}", evidence_dir.display()))
    })?;
    let evidence = smoke_evidence_paths(evidence_dir, smoke, validation);
    copy_evidence_file(&smoke.source_path, &evidence.source_path)?;
    copy_evidence_file(&smoke.output_path, &evidence.output_path)?;
    copy_evidence_file(&validation.metadata_path, &evidence.metadata_path)?;
    write_evidence_text(&evidence.stdout_path, &smoke.stdout)?;
    write_evidence_text(&evidence.stderr_path, &smoke.stderr)?;
    let copied_output_digest = file_digest_blake3(&evidence.output_path)?;
    if copied_output_digest != smoke.output_digest_blake3 {
        return Err(RustSourceProviderError::Smoke("copied smoke output digest mismatch".to_string()));
    }
    let copied_metadata_digest = file_digest_blake3(&evidence.metadata_path)?;
    if copied_metadata_digest != validation.metadata_digest_blake3 {
        return Err(RustSourceProviderError::Smoke("copied provider metadata digest mismatch".to_string()));
    }
    write_smoke_evidence_summary(&evidence, smoke, validation)?;
    Ok(evidence)
}

pub(crate) fn validate_materialized_rust_source_provider(
    provider_dir: &Path,
) -> Result<RustSourceProviderDirectoryValidation, RustSourceProviderError> {
    let metadata_path = provider_dir.join(RUST_SOURCE_PROVIDER_METADATA_PATH);
    let metadata_bytes = fs::read(&metadata_path).map_err(|err| {
        RustSourceProviderError::Read(format!("provider metadata {}: {err}", metadata_path.display()))
    })?;
    if metadata_bytes.is_empty() {
        return Err(RustSourceProviderError::Read(format!("provider metadata {} is empty", metadata_path.display())));
    }
    let metadata_digest_blake3 = blake3::hash(&metadata_bytes).to_hex().to_string();
    let metadata = serde_json::from_slice::<RustSourceProviderMetadata>(&metadata_bytes).map_err(|err| {
        RustSourceProviderError::Parse(format!("provider metadata {}: {err}", metadata_path.display()))
    })?;
    let observed = observed_provider_artifacts(provider_dir, &metadata)?;
    let observed_receipts = observed_provider_receipts(provider_dir, &metadata)?;
    crate::source_toolchain_closure::enforce_observed_rust_source_provider_artifacts(&metadata, &observed)
        .map_err(|err| RustSourceProviderError::Validate(err.message().to_string()))?;
    let validation =
        crate::source_toolchain_closure::enforce_observed_rust_source_provider_receipts(&metadata, &observed_receipts)
            .map_err(|err| RustSourceProviderError::Validate(err.message().to_string()))?;
    Ok(RustSourceProviderDirectoryValidation {
        metadata_path,
        metadata_digest_blake3,
        metadata,
        validation,
    })
}

fn smoke_evidence_paths(
    evidence_dir: &Path,
    smoke: &RustSourceProviderSmoke,
    validation: &RustSourceProviderDirectoryValidation,
) -> RustSourceProviderSmokeEvidence {
    RustSourceProviderSmokeEvidence {
        evidence_dir: evidence_dir.to_path_buf(),
        summary_path: evidence_dir.join(SMOKE_EVIDENCE_SUMMARY_FILE),
        stdout_path: evidence_dir.join(SMOKE_EVIDENCE_STDOUT_FILE),
        stderr_path: evidence_dir.join(SMOKE_EVIDENCE_STDERR_FILE),
        source_path: evidence_dir.join(SMOKE_EVIDENCE_SOURCE_FILE),
        output_path: evidence_dir.join(SMOKE_EVIDENCE_OUTPUT_FILE),
        metadata_path: evidence_dir.join(SMOKE_EVIDENCE_METADATA_FILE),
        output_digest_blake3: smoke.output_digest_blake3.clone(),
        metadata_digest_blake3: validation.metadata_digest_blake3.clone(),
        policy_digest_blake3: validation.validation.policy_digest_blake3.clone(),
    }
}

fn write_smoke_evidence_summary(
    evidence: &RustSourceProviderSmokeEvidence,
    smoke: &RustSourceProviderSmoke,
    validation: &RustSourceProviderDirectoryValidation,
) -> Result<(), RustSourceProviderError> {
    let summary = serde_json::json!({
        "schema": SMOKE_EVIDENCE_SCHEMA,
        "provider_path": smoke.provider_path,
        "rustc_path": smoke.rustc_path,
        "target_triple": smoke.target_triple,
        "stdout_path": evidence.stdout_path,
        "stderr_path": evidence.stderr_path,
        "source_path": evidence.source_path,
        "output_path": evidence.output_path,
        "output_digest_blake3": evidence.output_digest_blake3,
        "metadata": {
            "original_path": validation.metadata_path,
            "evidence_path": evidence.metadata_path,
            "metadata_digest_blake3": evidence.metadata_digest_blake3,
            "policy_digest_blake3": evidence.policy_digest_blake3,
            "host_triple": validation.metadata.host_triple,
            "target_triple": validation.metadata.target_triple,
            "artifact_count": validation.validation.artifact_count,
            "source_count": validation.validation.source_count,
            "receipt_count": validation.validation.receipt_count,
        }
    });
    let bytes = serde_json::to_vec_pretty(&summary)
        .map_err(|err| RustSourceProviderError::Smoke(format!("serialize smoke evidence summary: {err}")))?;
    fs::write(&evidence.summary_path, bytes).map_err(|err| {
        RustSourceProviderError::Smoke(format!(
            "write smoke evidence summary {}: {err}",
            evidence.summary_path.display()
        ))
    })
}

fn copy_evidence_file(source_path: &Path, dest_path: &Path) -> Result<(), RustSourceProviderError> {
    fs::copy(source_path, dest_path).map_err(|err| {
        RustSourceProviderError::Smoke(format!(
            "copy evidence {} -> {}: {err}",
            source_path.display(),
            dest_path.display()
        ))
    })?;
    Ok(())
}

fn write_evidence_text(path: &Path, text: &str) -> Result<(), RustSourceProviderError> {
    fs::write(path, text).map_err(|err| RustSourceProviderError::Smoke(format!("write {}: {err}", path.display())))
}

fn write_json_pretty<T: serde::Serialize>(path: &Path, value: &T, label: &str) -> Result<(), RustSourceProviderError> {
    let bytes = serde_json::to_vec_pretty(value)
        .map_err(|err| RustSourceProviderError::Parse(format!("serialize {label}: {err}")))?;
    let parent = path
        .parent()
        .ok_or_else(|| RustSourceProviderError::Read(format!("{} has no parent", path.display())))?;
    fs::create_dir_all(parent)
        .map_err(|err| RustSourceProviderError::Read(format!("create {}: {err}", parent.display())))?;
    fs::write(path, bytes)
        .map_err(|err| RustSourceProviderError::Read(format!("write {label} {}: {err}", path.display())))
}

fn validate_import_request(import_dir: &Path, output_dir: &Path) -> Result<(), RustSourceProviderError> {
    if import_dir.as_os_str().is_empty() {
        return Err(RustSourceProviderError::Read("import dir is empty".to_string()));
    }
    if output_dir.as_os_str().is_empty() {
        return Err(RustSourceProviderError::Read("output dir is empty".to_string()));
    }
    if !import_dir.is_dir() {
        return Err(RustSourceProviderError::Read(format!("import dir {} is not a directory", import_dir.display())));
    }
    if output_dir.exists() {
        return Err(RustSourceProviderError::Copy(format!("output dir {} already exists", output_dir.display())));
    }
    Ok(())
}

fn plan_materialization(
    recipe_path: &Path,
    output_dir: &Path,
    scratch_dir: &Path,
    recipe_digest_blake3: &str,
) -> Result<RustSourceProviderMaterializationPlan, RustSourceProviderError> {
    if recipe_path.as_os_str().is_empty() {
        return Err(RustSourceProviderError::Read("recipe path is empty".to_string()));
    }
    if output_dir.as_os_str().is_empty() {
        return Err(RustSourceProviderError::Read("output dir is empty".to_string()));
    }
    if scratch_dir.as_os_str().is_empty() {
        return Err(RustSourceProviderError::Read("scratch dir is empty".to_string()));
    }
    if recipe_digest_blake3.is_empty() {
        return Err(RustSourceProviderError::Digest("recipe digest is empty".to_string()));
    }
    let route_plan_path = route_plan_path_for_recipe(recipe_path)?;
    Ok(RustSourceProviderMaterializationPlan {
        recipe_path: recipe_path.to_path_buf(),
        output_dir: output_dir.to_path_buf(),
        scratch_dir: scratch_dir.to_path_buf(),
        recipe_digest_blake3: recipe_digest_blake3.to_string(),
        route_plan_path,
    })
}

fn route_plan_path_for_recipe(recipe_path: &Path) -> Result<PathBuf, RustSourceProviderError> {
    let parent = recipe_path
        .parent()
        .ok_or_else(|| RustSourceProviderError::Read(format!("recipe {} has no parent", recipe_path.display())))?;
    Ok(parent.join(RUST_SOURCE_PROVIDER_PLAN_FILE))
}

fn load_rust_source_provider_route(plan_path: &Path) -> Result<LoadedRustSourceProviderRoute, RustSourceProviderError> {
    let plan_bytes = fs::read(plan_path)
        .map_err(|err| RustSourceProviderError::Read(format!("route plan {}: {err}", plan_path.display())))?;
    if plan_bytes.is_empty() {
        return Err(RustSourceProviderError::Read(format!("route plan {} is empty", plan_path.display())));
    }
    let plan_digest_blake3 = blake3::hash(&plan_bytes).to_hex().to_string();
    let import_paths: Vec<OsString> = Vec::new();
    let plan: RustSourceProviderBootstrapPlan = crunch_eval::evaluate_and_deserialize(plan_path, &import_paths)
        .map_err(|err| RustSourceProviderError::Parse(format!("route plan {}: {err}", plan_path.display())))?;
    let validation = crate::source_toolchain_closure::validate_rust_source_provider_bootstrap_plan(&plan)
        .map_err(|err| RustSourceProviderError::Validate(err.message().to_string()))?;
    Ok(LoadedRustSourceProviderRoute {
        plan_path: plan_path.to_path_buf(),
        plan_digest_blake3,
        plan,
        validation,
    })
}

fn prepare_first_stage_boundary(
    materialization: &RustSourceProviderMaterializationPlan,
    route: &LoadedRustSourceProviderRoute,
) -> Result<RustSourceProviderFirstStageBoundary, RustSourceProviderError> {
    let stage = select_first_mrustc_stage(&route.plan)?;
    validate_first_stage_outputs(stage)?;
    let sources = sources_for_stage(&route.plan, stage)?;
    let expected_outputs = expected_outputs_for_stage(&route.plan, stage)?;
    Ok(RustSourceProviderFirstStageBoundary {
        route_plan_path: route.plan_path.clone(),
        route_plan_digest_blake3: route.plan_digest_blake3.clone(),
        route_policy_digest_blake3: route.validation.policy_digest_blake3.clone(),
        stage_id: stage.id.clone(),
        stage_kind: stage.kind,
        rust_version: stage.rust_version.clone(),
        target_triple: route.plan.target_triple.clone(),
        mrustc_target_version: mrustc_target_version(&stage.rust_version),
        source_ids: stage.source_ids.clone(),
        sources,
        expected_outputs,
        archive_dir: materialization.scratch_dir.join(FIRST_STAGE_ARCHIVE_DIR),
        source_dir: materialization.scratch_dir.join(FIRST_STAGE_SOURCE_DIR),
        build_dir: materialization.scratch_dir.join(FIRST_STAGE_BUILD_DIR),
        output_dir: materialization.output_dir.clone(),
        plan_path: materialization.scratch_dir.join(FIRST_STAGE_PLAN_FILE),
        sources_manifest_path: materialization.scratch_dir.join(FIRST_STAGE_SOURCES_MANIFEST_FILE),
        build_manifest_path: materialization.scratch_dir.join(FIRST_STAGE_BUILD_MANIFEST_FILE),
        build_log_path: materialization.scratch_dir.join(FIRST_STAGE_BUILD_LOG_FILE),
        provider_candidate_dir: materialization.scratch_dir.join(FIRST_STAGE_PROVIDER_CANDIDATE_DIR),
        provider_candidate_manifest_path: materialization
            .scratch_dir
            .join(FIRST_STAGE_PROVIDER_CANDIDATE_MANIFEST_FILE),
        provider_candidate_smoke_dir: materialization.scratch_dir.join(FIRST_STAGE_PROVIDER_CANDIDATE_SMOKE_DIR),
        provider_candidate_smoke_work_dir: materialization
            .scratch_dir
            .join(FIRST_STAGE_PROVIDER_CANDIDATE_SMOKE_DIR)
            .join(FIRST_STAGE_PROVIDER_CANDIDATE_SMOKE_WORK_DIR),
        script_path: materialization.scratch_dir.join(FIRST_STAGE_SCRIPT_FILE),
    })
}

fn select_first_mrustc_stage(
    plan: &RustSourceProviderBootstrapPlan,
) -> Result<&RustSourceProviderBootstrapStage, RustSourceProviderError> {
    let mut matches = plan.stages.iter().filter(|stage| stage.kind == RustSourceProviderBootstrapStageKind::MrustcSeed);
    let Some(stage) = matches.next() else {
        return Err(RustSourceProviderError::Validate("rust source route has no mrustc seed stage".to_string()));
    };
    if matches.next().is_none() {
        return Ok(stage);
    }
    Err(RustSourceProviderError::Validate("rust source route has multiple mrustc seed stages".to_string()))
}

fn validate_first_stage_outputs(stage: &RustSourceProviderBootstrapStage) -> Result<(), RustSourceProviderError> {
    for role in REQUIRED_FIRST_STAGE_OUTPUT_ROLES {
        if stage.outputs.contains(&role) {
            continue;
        }
        return Err(RustSourceProviderError::Validate(format!(
            "first mrustc stage '{}' lacks output role {role:?}",
            stage.id
        )));
    }
    Ok(())
}

fn mrustc_target_version(rust_version: &str) -> String {
    rust_version.strip_suffix(".0").unwrap_or(rust_version).to_string()
}

fn prepare_rustc_stage1_boundary(
    materialization: &RustSourceProviderMaterializationPlan,
    route: &LoadedRustSourceProviderRoute,
    bootstrap_candidate: &RustSourceProviderBootstrapProviderCandidate,
    scratch_layout: RustcStage1ScratchLayout,
) -> Result<RustSourceProviderRustcStage1Boundary, RustSourceProviderError> {
    let stage = select_next_rustc_stage1_stage(&route.plan, bootstrap_candidate)?;
    validate_rustc_stage1_outputs(stage)?;
    let sources = sources_for_stage(&route.plan, stage)?;
    let expected_outputs = expected_outputs_for_stage(&route.plan, stage)?;
    let stage_root = rustc_stage1_stage_root(materialization, &stage.id, scratch_layout)?;
    Ok(RustSourceProviderRustcStage1Boundary {
        route_plan_path: route.plan_path.clone(),
        route_plan_digest_blake3: route.plan_digest_blake3.clone(),
        route_policy_digest_blake3: route.validation.policy_digest_blake3.clone(),
        stage_id: stage.id.clone(),
        stage_kind: stage.kind,
        bootstrap_stage_id: stage.bootstrap_stage_id.clone(),
        rust_version: stage.rust_version.clone(),
        host_triple: route.plan.host_triple.clone(),
        target_triple: route.plan.target_triple.clone(),
        source_ids: stage.source_ids.clone(),
        sources,
        expected_outputs,
        bootstrap_provider_candidate_dir: bootstrap_candidate.candidate_dir.clone(),
        bootstrap_provider_metadata_digest_blake3: bootstrap_candidate.metadata_digest_blake3.clone(),
        bootstrap_provider_policy_digest_blake3: bootstrap_candidate.policy_digest_blake3.clone(),
        archive_dir: stage_root.join(RUSTC_STAGE1_ARCHIVE_DIR),
        source_dir: stage_root.join(RUSTC_STAGE1_SOURCE_DIR),
        build_dir: stage_root.join(RUSTC_STAGE1_BUILD_DIR),
        stage_output_dir: stage_root.join(RUSTC_STAGE1_OUTPUT_DIR),
        output_dir: materialization.output_dir.clone(),
        plan_path: stage_root.join(RUSTC_STAGE1_PLAN_FILE),
        sources_manifest_path: stage_root.join(RUSTC_STAGE1_SOURCES_MANIFEST_FILE),
        build_manifest_path: stage_root.join(RUSTC_STAGE1_BUILD_MANIFEST_FILE),
        build_log_path: stage_root.join(RUSTC_STAGE1_BUILD_LOG_FILE),
        script_path: stage_root.join(RUSTC_STAGE1_SCRIPT_FILE),
        provider_candidate_dir: stage_root.join(RUSTC_STAGE1_PROVIDER_CANDIDATE_DIR),
        provider_candidate_manifest_path: stage_root.join(RUSTC_STAGE1_PROVIDER_CANDIDATE_MANIFEST_FILE),
        provider_candidate_smoke_dir: stage_root.join(RUSTC_STAGE1_PROVIDER_CANDIDATE_SMOKE_DIR),
        provider_candidate_smoke_work_dir: stage_root
            .join(RUSTC_STAGE1_PROVIDER_CANDIDATE_SMOKE_DIR)
            .join(RUSTC_STAGE1_PROVIDER_CANDIDATE_SMOKE_WORK_DIR),
    })
}

fn bootstrap_candidate_from_first_stage(
    candidate: &RustSourceProviderFirstStageProviderCandidate,
) -> RustSourceProviderBootstrapProviderCandidate {
    RustSourceProviderBootstrapProviderCandidate {
        stage_id: candidate.stage_id.clone(),
        candidate_dir: candidate.candidate_dir.clone(),
        metadata_digest_blake3: candidate.metadata_digest_blake3.clone(),
        policy_digest_blake3: candidate.policy_digest_blake3.clone(),
    }
}

fn bootstrap_candidate_from_rustc_stage1(
    candidate: &RustSourceProviderRustcStage1ProviderCandidate,
) -> RustSourceProviderBootstrapProviderCandidate {
    RustSourceProviderBootstrapProviderCandidate {
        stage_id: candidate.stage_id.clone(),
        candidate_dir: candidate.candidate_dir.clone(),
        metadata_digest_blake3: candidate.metadata_digest_blake3.clone(),
        policy_digest_blake3: candidate.policy_digest_blake3.clone(),
    }
}

fn rustc_stage1_stage_root(
    materialization: &RustSourceProviderMaterializationPlan,
    stage_id: &str,
    scratch_layout: RustcStage1ScratchLayout,
) -> Result<PathBuf, RustSourceProviderError> {
    match scratch_layout {
        RustcStage1ScratchLayout::TopLevel => Ok(materialization.scratch_dir.clone()),
        RustcStage1ScratchLayout::Scoped => {
            Ok(materialization.scratch_dir.join(RUSTC_STAGE1_CHAIN_DIR).join(safe_source_component(stage_id)?))
        }
    }
}

fn select_next_rustc_stage1_stage<'a>(
    plan: &'a RustSourceProviderBootstrapPlan,
    bootstrap_candidate: &RustSourceProviderBootstrapProviderCandidate,
) -> Result<&'a RustSourceProviderBootstrapStage, RustSourceProviderError> {
    let mut matches = plan.stages.iter().filter(|stage| {
        stage.kind == RustSourceProviderBootstrapStageKind::RustcStage1
            && stage.bootstrap_stage_id == bootstrap_candidate.stage_id
    });
    let Some(stage) = matches.next() else {
        return Err(RustSourceProviderError::Validate(format!(
            "rust source route has no rustc stage1 after '{}'",
            bootstrap_candidate.stage_id
        )));
    };
    if matches.next().is_none() {
        return Ok(stage);
    }
    Err(RustSourceProviderError::Validate(format!(
        "rust source route has multiple rustc stage1 stages after '{}'",
        bootstrap_candidate.stage_id
    )))
}

fn run_rustc_stage1_provider_candidate(
    materialization: &RustSourceProviderMaterializationPlan,
    route: &LoadedRustSourceProviderRoute,
    bootstrap_candidate: &RustSourceProviderBootstrapProviderCandidate,
    scratch_layout: RustcStage1ScratchLayout,
    verbose: bool,
) -> Result<RustSourceProviderRustcStage1ProviderCandidateRun, RustSourceProviderError> {
    let boundary = prepare_rustc_stage1_boundary(materialization, route, bootstrap_candidate, scratch_layout)?;
    write_rustc_stage1_boundary(&boundary)?;
    let sources = acquire_rustc_stage1_sources(&boundary, verbose)?;
    write_rustc_stage1_sources_manifest(&boundary, &sources)?;
    let build = run_rustc_stage1_build(&boundary, verbose)?;
    write_rustc_stage1_build_manifest(&boundary, &build)?;
    let candidate = assemble_rustc_stage1_provider_candidate(&boundary, route, &sources, &build)?;
    let smoke = smoke_rustc_stage1_provider_candidate(&boundary, &candidate)?;
    write_rustc_stage1_provider_candidate_manifest(&boundary, &candidate)?;
    Ok(RustSourceProviderRustcStage1ProviderCandidateRun {
        boundary,
        candidate,
        smoke,
    })
}

fn validate_rustc_stage1_outputs(stage: &RustSourceProviderBootstrapStage) -> Result<(), RustSourceProviderError> {
    for role in REQUIRED_RUSTC_STAGE1_OUTPUT_ROLES {
        if stage.outputs.contains(&role) {
            continue;
        }
        return Err(RustSourceProviderError::Validate(format!(
            "rustc stage1 '{}' lacks output role {role:?}",
            stage.id
        )));
    }
    Ok(())
}

fn sources_for_stage(
    plan: &RustSourceProviderBootstrapPlan,
    stage: &RustSourceProviderBootstrapStage,
) -> Result<Vec<RustSourceProviderBootstrapSource>, RustSourceProviderError> {
    let mut sources = Vec::with_capacity(stage.source_ids.len());
    for source_id in &stage.source_ids {
        let Some(source) = plan.sources.iter().find(|source| &source.id == source_id) else {
            return Err(RustSourceProviderError::Validate(format!(
                "stage '{}' references unknown source_id '{source_id}'",
                stage.id
            )));
        };
        sources.push(source.clone());
    }
    Ok(sources)
}

fn expected_outputs_for_stage(
    plan: &RustSourceProviderBootstrapPlan,
    stage: &RustSourceProviderBootstrapStage,
) -> Result<Vec<RustSourceProviderBootstrapOutput>, RustSourceProviderError> {
    let mut outputs = Vec::with_capacity(stage.outputs.len());
    for role in &stage.outputs {
        let Some(output) = plan.final_outputs.iter().find(|output| output.role == *role) else {
            return Err(RustSourceProviderError::Validate(format!(
                "first mrustc stage '{}' has no output path for role {role:?}",
                stage.id
            )));
        };
        outputs.push(output.clone());
    }
    outputs.sort_by_key(|output| (output.role, output.path.clone()));
    Ok(outputs)
}

fn write_first_stage_boundary(boundary: &RustSourceProviderFirstStageBoundary) -> Result<(), RustSourceProviderError> {
    fs::create_dir_all(&boundary.archive_dir)
        .map_err(|err| RustSourceProviderError::Read(format!("create {}: {err}", boundary.archive_dir.display())))?;
    fs::create_dir_all(&boundary.source_dir)
        .map_err(|err| RustSourceProviderError::Read(format!("create {}: {err}", boundary.source_dir.display())))?;
    fs::create_dir_all(&boundary.build_dir)
        .map_err(|err| RustSourceProviderError::Read(format!("create {}: {err}", boundary.build_dir.display())))?;
    let plan_bytes = first_stage_boundary_manifest(boundary)?;
    fs::write(&boundary.plan_path, plan_bytes).map_err(|err| {
        RustSourceProviderError::Read(format!("write first stage plan {}: {err}", boundary.plan_path.display()))
    })?;
    let script = first_stage_boundary_script(boundary)?;
    fs::write(&boundary.script_path, script).map_err(|err| {
        RustSourceProviderError::Read(format!("write first stage script {}: {err}", boundary.script_path.display()))
    })?;
    make_executable(&boundary.script_path)
}

fn write_rustc_stage1_boundary(
    boundary: &RustSourceProviderRustcStage1Boundary,
) -> Result<(), RustSourceProviderError> {
    fs::create_dir_all(&boundary.archive_dir)
        .map_err(|err| RustSourceProviderError::Read(format!("create {}: {err}", boundary.archive_dir.display())))?;
    fs::create_dir_all(&boundary.source_dir)
        .map_err(|err| RustSourceProviderError::Read(format!("create {}: {err}", boundary.source_dir.display())))?;
    fs::create_dir_all(&boundary.build_dir)
        .map_err(|err| RustSourceProviderError::Read(format!("create {}: {err}", boundary.build_dir.display())))?;
    let manifest = rustc_stage1_boundary_manifest(boundary);
    write_json_pretty(&boundary.plan_path, &manifest, "rustc stage1 plan")?;
    let script = rustc_stage1_boundary_script(boundary)?;
    fs::write(&boundary.script_path, script).map_err(|err| {
        RustSourceProviderError::Read(format!("write rustc stage1 script {}: {err}", boundary.script_path.display()))
    })?;
    make_executable(&boundary.script_path)
}

fn rustc_stage1_boundary_manifest(boundary: &RustSourceProviderRustcStage1Boundary) -> serde_json::Value {
    serde_json::json!({
        "schema": RUSTC_STAGE1_PLAN_SCHEMA,
        "route_plan": {
            "path": boundary.route_plan_path.display().to_string(),
            "digest_blake3": &boundary.route_plan_digest_blake3,
            "policy_digest_blake3": &boundary.route_policy_digest_blake3,
        },
        "stage": {
            "id": &boundary.stage_id,
            "kind": boundary.stage_kind,
            "bootstrap_stage_id": &boundary.bootstrap_stage_id,
            "rust_version": &boundary.rust_version,
            "host_triple": &boundary.host_triple,
            "target_triple": &boundary.target_triple,
            "source_ids": &boundary.source_ids,
            "expected_outputs": &boundary.expected_outputs,
        },
        "bootstrap_provider_candidate": {
            "path": boundary.bootstrap_provider_candidate_dir.display().to_string(),
            "metadata_digest_blake3": &boundary.bootstrap_provider_metadata_digest_blake3,
            "policy_digest_blake3": &boundary.bootstrap_provider_policy_digest_blake3,
        },
        "sources": &boundary.sources,
        "paths": {
            "archive_dir": boundary.archive_dir.display().to_string(),
            "source_dir": boundary.source_dir.display().to_string(),
            "build_dir": boundary.build_dir.display().to_string(),
            "stage_output_dir": boundary.stage_output_dir.display().to_string(),
            "output_dir": boundary.output_dir.display().to_string(),
            "script": boundary.script_path.display().to_string(),
            "source_manifest": boundary.sources_manifest_path.display().to_string(),
            "build_manifest": boundary.build_manifest_path.display().to_string(),
            "build_log": boundary.build_log_path.display().to_string(),
        },
        "next_blocked_reason": RUST_SOURCE_PROVIDER_BLOCKED_REASON,
    })
}

fn rustc_stage1_boundary_script(
    boundary: &RustSourceProviderRustcStage1Boundary,
) -> Result<String, RustSourceProviderError> {
    let inputs = rustc_stage1_script_inputs(boundary)?;
    let mut script = String::new();
    push_rustc_stage1_script_header(&mut script, boundary, &inputs);
    push_rustc_stage1_script_input_checks(&mut script);
    push_rustc_stage1_script_build_commands(&mut script);
    Ok(script)
}

struct RustSourceProviderRustcStage1ScriptInputs {
    source_id: String,
    source_path: PathBuf,
    build_script_path: PathBuf,
}

fn rustc_stage1_script_inputs(
    boundary: &RustSourceProviderRustcStage1Boundary,
) -> Result<RustSourceProviderRustcStage1ScriptInputs, RustSourceProviderError> {
    let source = rustc_stage1_build_source(boundary)?;
    let source_component = safe_source_component(&source.id)?;
    let source_path = boundary.source_dir.join(&source_component);
    let build_script_path = source_path.join(RUSTC_STAGE1_SOURCE_BUILD_SCRIPT);
    Ok(RustSourceProviderRustcStage1ScriptInputs {
        source_id: source.id.clone(),
        source_path,
        build_script_path,
    })
}

fn push_rustc_stage1_script_header(
    script: &mut String,
    boundary: &RustSourceProviderRustcStage1Boundary,
    inputs: &RustSourceProviderRustcStage1ScriptInputs,
) {
    script.push_str("#!/bin/sh\n");
    script.push_str("set -eu\n");
    script.push_str(&format!("STAGE_ID={}\n", shell_quote(&boundary.stage_id)));
    script.push_str(&format!("RUSTC_VERSION={}\n", shell_quote(&boundary.rust_version)));
    script.push_str(&format!("HOST_TRIPLE={}\n", shell_quote(&boundary.host_triple)));
    script.push_str(&format!("TARGET_TRIPLE={}\n", shell_quote(&boundary.target_triple)));
    script
        .push_str(&format!("SOURCE_MANIFEST={}\n", shell_quote(&boundary.sources_manifest_path.display().to_string())));
    script.push_str(&format!("RUST_SOURCE_ID={}\n", shell_quote(&inputs.source_id)));
    script.push_str(&format!("RUST_SOURCE={}\n", shell_quote(&inputs.source_path.display().to_string())));
    script.push_str(&format!("BUILD_SCRIPT={}\n", shell_quote(&inputs.build_script_path.display().to_string())));
    script.push_str(&format!(
        "BOOTSTRAP_PROVIDER={}\n",
        shell_quote(&boundary.bootstrap_provider_candidate_dir.display().to_string())
    ));
    script.push_str(&format!("BUILD_DIR={}\n", shell_quote(&boundary.build_dir.display().to_string())));
    script.push_str(&format!("STAGE_OUTPUT={}\n", shell_quote(&boundary.stage_output_dir.display().to_string())));
    script.push_str(&format!("SHELL_PROGRAM={}\n", shell_quote(RUSTC_STAGE1_SHELL_PROGRAM)));
}

fn push_rustc_stage1_script_input_checks(script: &mut String) {
    script.push_str("printf '%s\\n' \"mantle rustc stage1: $STAGE_ID\"\n");
    push_required_shell_var_file_check(
        script,
        "SOURCE_MANIFEST",
        "missing rustc stage1 source manifest",
        FIRST_STAGE_MISSING_SOURCE_EXIT_CODE,
    );
    push_required_shell_var_dir_check(
        script,
        "RUST_SOURCE",
        "missing verified rustc stage1 source",
        FIRST_STAGE_MISSING_SOURCE_EXIT_CODE,
    );
    push_required_shell_var_file_check(
        script,
        "BUILD_SCRIPT",
        "rustc stage1 source has no build script",
        RUSTC_STAGE1_BUILD_FAILED_EXIT_CODE,
    );
    push_required_shell_path_file_check(
        script,
        "\"$BOOTSTRAP_PROVIDER/bin/rustc\"",
        "bootstrap provider candidate has no rustc",
        RUSTC_STAGE1_BUILD_FAILED_EXIT_CODE,
    );
    push_required_shell_path_file_check(
        script,
        "\"$BOOTSTRAP_PROVIDER/bin/cargo\"",
        "bootstrap provider candidate has no cargo",
        RUSTC_STAGE1_BUILD_FAILED_EXIT_CODE,
    );
    push_required_shell_path_dir_check(
        script,
        "\"$BOOTSTRAP_PROVIDER/lib/rustlib/$TARGET_TRIPLE/lib\"",
        "bootstrap provider candidate has no target rustlib",
        RUSTC_STAGE1_BUILD_FAILED_EXIT_CODE,
    );
    push_required_shell_command_check(script, "SHELL_PROGRAM", "sh is required for rustc stage1");
}

fn push_rustc_stage1_script_build_commands(script: &mut String) {
    script.push_str("mkdir -p \"$BUILD_DIR\" \"$STAGE_OUTPUT\"\n");
    script.push_str(
        "MANTLE_BOOTSTRAP_PROVIDER=\"$BOOTSTRAP_PROVIDER\" MANTLE_STAGE_OUTPUT=\"$STAGE_OUTPUT\" MANTLE_RUST_VERSION=\"$RUSTC_VERSION\" MANTLE_HOST_TRIPLE=\"$HOST_TRIPLE\" MANTLE_TARGET_TRIPLE=\"$TARGET_TRIPLE\" \"$SHELL_PROGRAM\" \"$BUILD_SCRIPT\"\n",
    );
    push_required_shell_path_file_check(
        script,
        "\"$STAGE_OUTPUT/bin/rustc\"",
        "rustc stage1 build did not produce bin/rustc",
        RUSTC_STAGE1_BUILD_FAILED_EXIT_CODE,
    );
    push_required_shell_path_file_check(
        script,
        "\"$STAGE_OUTPUT/bin/cargo\"",
        "rustc stage1 build did not produce bin/cargo",
        RUSTC_STAGE1_BUILD_FAILED_EXIT_CODE,
    );
    push_required_shell_path_dir_check(
        script,
        "\"$STAGE_OUTPUT/lib/rustlib/$HOST_TRIPLE/lib\"",
        "rustc stage1 build did not produce host rustlib",
        RUSTC_STAGE1_BUILD_FAILED_EXIT_CODE,
    );
    push_required_shell_path_dir_check(
        script,
        "\"$STAGE_OUTPUT/lib/rustlib/$TARGET_TRIPLE/lib\"",
        "rustc stage1 build did not produce target rustlib",
        RUSTC_STAGE1_BUILD_FAILED_EXIT_CODE,
    );
    script.push_str("printf '%s\\n' \"rustc stage1 products ready\"\n");
}

fn rustc_stage1_build_source(
    boundary: &RustSourceProviderRustcStage1Boundary,
) -> Result<&RustSourceProviderBootstrapSource, RustSourceProviderError> {
    let mut matches = boundary.sources.iter().filter(|source| source.id.starts_with(FIRST_STAGE_RUST_SOURCE_PREFIX));
    let Some(source) = matches.next() else {
        return Err(RustSourceProviderError::Build(format!("rustc stage1 '{}' has no Rust source", boundary.stage_id)));
    };
    if matches.next().is_none() {
        return Ok(source);
    }
    Err(RustSourceProviderError::Build(format!(
        "rustc stage1 '{}' has multiple Rust sources",
        boundary.stage_id
    )))
}

fn first_stage_boundary_manifest(
    boundary: &RustSourceProviderFirstStageBoundary,
) -> Result<Vec<u8>, RustSourceProviderError> {
    let manifest = serde_json::json!({
        "schema": FIRST_STAGE_PLAN_SCHEMA,
        "route_plan": {
            "path": boundary.route_plan_path.display().to_string(),
            "digest_blake3": &boundary.route_plan_digest_blake3,
            "policy_digest_blake3": &boundary.route_policy_digest_blake3,
        },
        "stage": {
            "id": &boundary.stage_id,
            "kind": boundary.stage_kind,
            "rust_version": &boundary.rust_version,
            "target_triple": &boundary.target_triple,
            "mrustc_target_version": &boundary.mrustc_target_version,
            "source_ids": &boundary.source_ids,
            "expected_outputs": &boundary.expected_outputs,
        },
        "sources": &boundary.sources,
        "paths": {
            "archive_dir": boundary.archive_dir.display().to_string(),
            "source_dir": boundary.source_dir.display().to_string(),
            "build_dir": boundary.build_dir.display().to_string(),
            "output_dir": boundary.output_dir.display().to_string(),
            "script": boundary.script_path.display().to_string(),
            "source_manifest": boundary.sources_manifest_path.display().to_string(),
            "build_manifest": boundary.build_manifest_path.display().to_string(),
            "build_log": boundary.build_log_path.display().to_string(),
            "provider_candidate_dir": boundary.provider_candidate_dir.display().to_string(),
            "provider_candidate_manifest": boundary.provider_candidate_manifest_path.display().to_string(),
            "provider_candidate_smoke_dir": boundary.provider_candidate_smoke_dir.display().to_string(),
            "provider_candidate_smoke_work_dir": boundary.provider_candidate_smoke_work_dir.display().to_string(),
        },
        "blocked_reason": RUST_SOURCE_PROVIDER_BLOCKED_REASON,
    });
    serde_json::to_vec_pretty(&manifest)
        .map_err(|err| RustSourceProviderError::Parse(format!("serialize first stage plan: {err}")))
}

fn acquire_first_stage_sources(
    boundary: &RustSourceProviderFirstStageBoundary,
    verbose: bool,
) -> Result<Vec<RustSourceProviderFirstStageSourceAcquisition>, RustSourceProviderError> {
    acquire_stage_sources(&boundary.stage_id, &boundary.archive_dir, &boundary.source_dir, &boundary.sources, verbose)
}

fn acquire_rustc_stage1_sources(
    boundary: &RustSourceProviderRustcStage1Boundary,
    verbose: bool,
) -> Result<Vec<RustSourceProviderFirstStageSourceAcquisition>, RustSourceProviderError> {
    acquire_stage_sources(&boundary.stage_id, &boundary.archive_dir, &boundary.source_dir, &boundary.sources, verbose)
}

fn acquire_stage_sources(
    stage_id: &str,
    archive_dir: &Path,
    source_dir: &Path,
    sources: &[RustSourceProviderBootstrapSource],
    verbose: bool,
) -> Result<Vec<RustSourceProviderFirstStageSourceAcquisition>, RustSourceProviderError> {
    assert!(!stage_id.trim().is_empty(), "stage id must not be empty");
    assert!(!sources.is_empty(), "stage sources must not be empty");
    assert!(!archive_dir.as_os_str().is_empty(), "archive dir must not be empty");
    fs::create_dir_all(archive_dir)
        .map_err(|err| RustSourceProviderError::Read(format!("create {}: {err}", archive_dir.display())))?;
    fs::create_dir_all(source_dir)
        .map_err(|err| RustSourceProviderError::Read(format!("create {}: {err}", source_dir.display())))?;
    let mut acquired = Vec::with_capacity(sources.len());
    for source in sources {
        acquired.push(acquire_stage_source(archive_dir, source_dir, source, verbose)?);
    }
    if acquired.len() != sources.len() {
        return Err(RustSourceProviderError::Fetch(format!("stage '{stage_id}' source acquisition count mismatch")));
    }
    Ok(acquired)
}

fn acquire_stage_source(
    archive_dir: &Path,
    source_dir: &Path,
    source: &RustSourceProviderBootstrapSource,
    verbose: bool,
) -> Result<RustSourceProviderFirstStageSourceAcquisition, RustSourceProviderError> {
    let source_component = safe_source_component(&source.id)?;
    ensure_supported_first_stage_archive(&source.url)?;
    let archive_path = archive_dir.join(format!("{source_component}.{FIRST_STAGE_SOURCE_ARCHIVE_EXTENSION}"));
    let extracted_path = source_dir.join(&source_component);
    let bytes = fetch_first_stage_source_bytes(source)?;
    let archive_sha256_hex = sha256_hex(&bytes);
    if archive_sha256_hex != source.sha256_hex {
        return Err(RustSourceProviderError::Digest(format!(
            "source '{}' digest mismatch: expected {}, got {archive_sha256_hex}",
            source.id, source.sha256_hex
        )));
    }
    fs::write(&archive_path, &bytes)
        .map_err(|err| RustSourceProviderError::Fetch(format!("write {}: {err}", archive_path.display())))?;
    prepare_empty_directory(&extracted_path)?;
    let archive_entry_count = extract_first_stage_tar_gz(&archive_path, &extracted_path)?;
    let extracted_digest_blake3 = content_digest_blake3(&extracted_path)?;
    if verbose {
        eprintln!(
            "  acquired source {} sha256={} extracted={} entries={}",
            source.id,
            archive_sha256_hex,
            extracted_path.display(),
            archive_entry_count
        );
    }
    Ok(RustSourceProviderFirstStageSourceAcquisition {
        id: source.id.clone(),
        url: source.url.clone(),
        archive_path,
        archive_sha256_hex,
        extracted_path,
        extracted_digest_blake3,
        archive_entry_count,
    })
}

fn write_first_stage_sources_manifest(
    boundary: &RustSourceProviderFirstStageBoundary,
    sources: &[RustSourceProviderFirstStageSourceAcquisition],
) -> Result<(), RustSourceProviderError> {
    if sources.len() != boundary.sources.len() {
        return Err(RustSourceProviderError::Fetch("first stage sources manifest count mismatch".to_string()));
    }
    let manifest = serde_json::json!({
        "schema": FIRST_STAGE_SOURCES_MANIFEST_SCHEMA,
        "stage_id": boundary.stage_id,
        "route_plan_digest_blake3": boundary.route_plan_digest_blake3,
        "route_policy_digest_blake3": boundary.route_policy_digest_blake3,
        "source_count": sources.len(),
        "sources": sources,
        "blocked_reason": RUST_SOURCE_PROVIDER_BLOCKED_REASON,
    });
    write_json_pretty(&boundary.sources_manifest_path, &manifest, "first stage sources manifest")
}

fn write_rustc_stage1_sources_manifest(
    boundary: &RustSourceProviderRustcStage1Boundary,
    sources: &[RustSourceProviderFirstStageSourceAcquisition],
) -> Result<(), RustSourceProviderError> {
    if sources.len() != boundary.sources.len() {
        return Err(RustSourceProviderError::Fetch("rustc stage1 sources manifest count mismatch".to_string()));
    }
    let manifest = serde_json::json!({
        "schema": RUSTC_STAGE1_SOURCES_MANIFEST_SCHEMA,
        "stage_id": boundary.stage_id,
        "bootstrap_stage_id": boundary.bootstrap_stage_id,
        "route_plan_digest_blake3": boundary.route_plan_digest_blake3,
        "route_policy_digest_blake3": boundary.route_policy_digest_blake3,
        "source_count": sources.len(),
        "sources": sources,
        "bootstrap_provider_candidate": {
            "path": boundary.bootstrap_provider_candidate_dir.display().to_string(),
            "metadata_digest_blake3": boundary.bootstrap_provider_metadata_digest_blake3,
            "policy_digest_blake3": boundary.bootstrap_provider_policy_digest_blake3,
        },
        "next_blocked_reason": RUST_SOURCE_PROVIDER_BLOCKED_REASON,
    });
    write_json_pretty(&boundary.sources_manifest_path, &manifest, "rustc stage1 sources manifest")
}

fn run_rustc_stage1_build(
    boundary: &RustSourceProviderRustcStage1Boundary,
    verbose: bool,
) -> Result<RustSourceProviderRustcStage1Build, RustSourceProviderError> {
    validate_rustc_stage1_build_inputs(boundary)?;
    prepare_empty_provider_candidate_dir(&boundary.stage_output_dir)?;
    if verbose {
        eprintln!("  rustc_stage1_build_script: {}", boundary.script_path.display());
        eprintln!("  rustc_stage1_build_log: {}", boundary.build_log_path.display());
    }
    let log = File::create(&boundary.build_log_path).map_err(|err| {
        RustSourceProviderError::Build(format!("create {}: {err}", boundary.build_log_path.display()))
    })?;
    let status = Command::new(&boundary.script_path)
        .stdout(log.try_clone().map_err(|err| {
            RustSourceProviderError::Build(format!("clone {}: {err}", boundary.build_log_path.display()))
        })?)
        .stderr(log)
        .status()
        .map_err(|err| RustSourceProviderError::Build(format!("launch {}: {err}", boundary.script_path.display())))?;
    if !status.success() {
        let log_tail = fs::read(&boundary.build_log_path)
            .map(|bytes| bounded_output_text(&bytes))
            .unwrap_or_else(|err| format!("<failed to read build log: {err}>"));
        return Err(RustSourceProviderError::Build(format!(
            "rustc stage1 build failed with status {status}; log={}; tail={log_tail:?}",
            boundary.build_log_path.display()
        )));
    }
    rustc_stage1_build_from_outputs(boundary)
}

fn validate_rustc_stage1_build_inputs(
    boundary: &RustSourceProviderRustcStage1Boundary,
) -> Result<(), RustSourceProviderError> {
    if !boundary.script_path.is_file() {
        return Err(RustSourceProviderError::Build(format!(
            "rustc stage1 script {} is missing",
            boundary.script_path.display()
        )));
    }
    if !boundary.sources_manifest_path.is_file() {
        return Err(RustSourceProviderError::Build(format!(
            "rustc stage1 source manifest {} is missing",
            boundary.sources_manifest_path.display()
        )));
    }
    if !boundary.bootstrap_provider_candidate_dir.join(RUST_SOURCE_PROVIDER_METADATA_PATH).is_file() {
        return Err(RustSourceProviderError::Build(format!(
            "bootstrap provider candidate metadata is missing under {}",
            boundary.bootstrap_provider_candidate_dir.display()
        )));
    }
    Ok(())
}

fn rustc_stage1_build_from_outputs(
    boundary: &RustSourceProviderRustcStage1Boundary,
) -> Result<RustSourceProviderRustcStage1Build, RustSourceProviderError> {
    let rustc_path = boundary.stage_output_dir.join(PROVIDER_RUSTC_RELATIVE_PATH);
    let cargo_path = boundary.stage_output_dir.join(PROVIDER_CARGO_RELATIVE_PATH);
    let host_rustlib_path = boundary.stage_output_dir.join(provider_rustlib_relative_path(&boundary.host_triple));
    let target_rustlib_path = boundary.stage_output_dir.join(provider_rustlib_relative_path(&boundary.target_triple));
    validate_first_stage_build_product(&rustc_path, PROVIDER_RUSTC_RELATIVE_PATH)?;
    validate_first_stage_build_product(&cargo_path, PROVIDER_CARGO_RELATIVE_PATH)?;
    validate_first_stage_build_product(&host_rustlib_path, "rustc stage1 host rustlib")?;
    validate_first_stage_build_product(&target_rustlib_path, "rustc stage1 target rustlib")?;
    Ok(RustSourceProviderRustcStage1Build {
        stage_id: boundary.stage_id.clone(),
        script_path: boundary.script_path.clone(),
        log_path: boundary.build_log_path.clone(),
        stage_output_digest_blake3: content_digest_blake3(&boundary.stage_output_dir)?,
        stage_output_dir: boundary.stage_output_dir.clone(),
        rustc_digest_blake3: file_digest_blake3(&rustc_path)?,
        cargo_digest_blake3: file_digest_blake3(&cargo_path)?,
        host_rustlib_digest_blake3: content_digest_blake3(&host_rustlib_path)?,
        target_rustlib_digest_blake3: content_digest_blake3(&target_rustlib_path)?,
        rustc_path,
        cargo_path,
        host_rustlib_path,
        target_rustlib_path,
    })
}

fn write_rustc_stage1_build_manifest(
    boundary: &RustSourceProviderRustcStage1Boundary,
    build: &RustSourceProviderRustcStage1Build,
) -> Result<(), RustSourceProviderError> {
    let manifest = serde_json::json!({
        "schema": RUSTC_STAGE1_BUILD_MANIFEST_SCHEMA,
        "stage_id": boundary.stage_id,
        "bootstrap_stage_id": boundary.bootstrap_stage_id,
        "route_plan_digest_blake3": boundary.route_plan_digest_blake3,
        "route_policy_digest_blake3": boundary.route_policy_digest_blake3,
        "bootstrap_provider_candidate": {
            "path": boundary.bootstrap_provider_candidate_dir.display().to_string(),
            "metadata_digest_blake3": boundary.bootstrap_provider_metadata_digest_blake3,
            "policy_digest_blake3": boundary.bootstrap_provider_policy_digest_blake3,
        },
        "build": build,
        "next_blocked_reason": RUST_SOURCE_PROVIDER_BLOCKED_REASON,
    });
    write_json_pretty(&boundary.build_manifest_path, &manifest, "rustc stage1 build manifest")
}

fn assemble_rustc_stage1_provider_candidate(
    boundary: &RustSourceProviderRustcStage1Boundary,
    route: &LoadedRustSourceProviderRoute,
    sources: &[RustSourceProviderFirstStageSourceAcquisition],
    build: &RustSourceProviderRustcStage1Build,
) -> Result<RustSourceProviderRustcStage1ProviderCandidate, RustSourceProviderError> {
    validate_rustc_stage1_candidate_request(boundary, sources, build)?;
    prepare_empty_provider_candidate_dir(&boundary.provider_candidate_dir)?;
    copy_provider_prefix(&build.stage_output_dir, &boundary.provider_candidate_dir)?;
    let source_identities = rustc_stage1_provider_source_identities(boundary, sources)?;
    let artifact_source_id = rustc_stage1_artifact_source_id(boundary)?;
    let receipt_artifacts = rustc_stage1_provider_receipt_artifacts(&boundary.provider_candidate_dir, route)?;
    let receipt = rustc_stage1_provider_receipt(boundary, route, &source_identities, &receipt_artifacts);
    let receipt_path = boundary.provider_candidate_dir.join(RUSTC_STAGE1_PROVIDER_RECEIPT_RELATIVE_PATH);
    write_json_pretty(&receipt_path, &receipt, "rustc stage1 provider receipt")?;
    let receipt_digest_blake3 = file_digest_blake3(&receipt_path)?;
    let metadata = rustc_stage1_provider_metadata(
        route,
        &source_identities,
        &receipt_artifacts,
        &receipt_digest_blake3,
        &artifact_source_id,
    );
    let metadata_path = boundary.provider_candidate_dir.join(RUST_SOURCE_PROVIDER_METADATA_PATH);
    write_json_pretty(&metadata_path, &metadata, "rustc stage1 provider metadata")?;
    let validation = validate_materialized_rust_source_provider(&boundary.provider_candidate_dir)?;
    Ok(RustSourceProviderRustcStage1ProviderCandidate {
        stage_id: boundary.stage_id.clone(),
        candidate_dir: boundary.provider_candidate_dir.clone(),
        metadata_path,
        metadata_digest_blake3: validation.metadata_digest_blake3,
        policy_digest_blake3: validation.validation.policy_digest_blake3,
        artifact_count: validation.validation.artifact_count,
        source_count: validation.validation.source_count,
        receipt_count: validation.validation.receipt_count,
    })
}

fn smoke_rustc_stage1_provider_candidate(
    boundary: &RustSourceProviderRustcStage1Boundary,
    candidate: &RustSourceProviderRustcStage1ProviderCandidate,
) -> Result<RustSourceProviderSmokeEvidence, RustSourceProviderError> {
    let validation = validate_materialized_rust_source_provider(&candidate.candidate_dir)?;
    let smoke = smoke_rust_source_provider(&candidate.candidate_dir, &boundary.provider_candidate_smoke_work_dir)?;
    persist_rust_source_provider_smoke_evidence(&boundary.provider_candidate_smoke_dir, &smoke, &validation)
}

fn validate_rustc_stage1_candidate_request(
    boundary: &RustSourceProviderRustcStage1Boundary,
    sources: &[RustSourceProviderFirstStageSourceAcquisition],
    build: &RustSourceProviderRustcStage1Build,
) -> Result<(), RustSourceProviderError> {
    if sources.len() != boundary.sources.len() {
        return Err(RustSourceProviderError::Validate("rustc stage1 candidate source count mismatch".to_string()));
    }
    if build.stage_id != boundary.stage_id {
        return Err(RustSourceProviderError::Validate(format!(
            "rustc stage1 candidate build stage expected {}, got {}",
            boundary.stage_id, build.stage_id
        )));
    }
    if boundary.provider_candidate_dir == boundary.output_dir {
        return Err(RustSourceProviderError::Validate(
            "rustc stage1 provider candidate must not be the final output dir".to_string(),
        ));
    }
    validate_first_stage_build_product(&build.stage_output_dir, RUSTC_STAGE1_OUTPUT_DIR)
}

fn rustc_stage1_provider_source_identities(
    boundary: &RustSourceProviderRustcStage1Boundary,
    sources: &[RustSourceProviderFirstStageSourceAcquisition],
) -> Result<Vec<RustProviderSourceIdentity>, RustSourceProviderError> {
    let mut identities = Vec::with_capacity(boundary.sources.len() + 1);
    for source in &boundary.sources {
        let acquired = sources.iter().find(|acquired| acquired.id == source.id).ok_or_else(|| {
            RustSourceProviderError::Validate(format!(
                "rustc stage1 candidate lacks source acquisition '{}'",
                source.id
            ))
        })?;
        identities.push(RustProviderSourceIdentity {
            id: source.id.clone(),
            kind: source.kind,
            name: source.name.clone(),
            digest_blake3: acquired.extracted_digest_blake3.clone(),
        });
    }
    identities.push(RustProviderSourceIdentity {
        id: format!("{}-bootstrap-provider", boundary.stage_id),
        kind: ToolchainSourceKind::Generated,
        name: "bootstrap-provider-candidate".to_string(),
        digest_blake3: boundary.bootstrap_provider_metadata_digest_blake3.clone(),
    });
    Ok(identities)
}

fn rustc_stage1_artifact_source_id(
    boundary: &RustSourceProviderRustcStage1Boundary,
) -> Result<String, RustSourceProviderError> {
    Ok(rustc_stage1_build_source(boundary)?.id.clone())
}

fn rustc_stage1_provider_receipt_artifacts(
    candidate_dir: &Path,
    route: &LoadedRustSourceProviderRoute,
) -> Result<Vec<RustProviderReceiptArtifact>, RustSourceProviderError> {
    Ok(vec![
        provider_receipt_artifact(candidate_dir, RustProviderRole::Rustc, "rustc", PROVIDER_RUSTC_RELATIVE_PATH)?,
        provider_receipt_artifact(candidate_dir, RustProviderRole::Cargo, "cargo", PROVIDER_CARGO_RELATIVE_PATH)?,
        provider_receipt_artifact(
            candidate_dir,
            RustProviderRole::HostRustlib,
            "host-rustlib",
            &provider_rustlib_relative_path(&route.plan.host_triple),
        )?,
        provider_receipt_artifact(
            candidate_dir,
            RustProviderRole::TargetRustlib,
            "target-rustlib",
            &provider_rustlib_relative_path(&route.plan.target_triple),
        )?,
    ])
}

fn rustc_stage1_provider_receipt(
    boundary: &RustSourceProviderRustcStage1Boundary,
    route: &LoadedRustSourceProviderRoute,
    sources: &[RustProviderSourceIdentity],
    artifacts: &[RustProviderReceiptArtifact],
) -> RustSourceProviderBuildReceipt {
    RustSourceProviderBuildReceipt {
        schema: RUST_SOURCE_PROVIDER_RECEIPT_SCHEMA.to_string(),
        receipt_id: RUSTC_STAGE1_PROVIDER_RECEIPT_ID.to_string(),
        provider_id: RUST_SOURCE_PROVIDER_ID.to_string(),
        host_triple: route.plan.host_triple.clone(),
        target_triple: route.plan.target_triple.clone(),
        source_ids: sources.iter().map(|source| source.id.clone()).collect(),
        output_artifacts: artifacts.to_vec(),
        build_steps: rustc_stage1_provider_receipt_steps(boundary),
    }
}

fn rustc_stage1_provider_receipt_steps(
    boundary: &RustSourceProviderRustcStage1Boundary,
) -> Vec<RustProviderReceiptStep> {
    vec![
        RustProviderReceiptStep {
            name: "acquire-rustc-stage1-source".to_string(),
            program: "mantle-rust-source-fetch".to_string(),
            arguments: boundary.source_ids.clone(),
        },
        RustProviderReceiptStep {
            name: "validate-bootstrap-provider".to_string(),
            program: "mantle-provider-validator".to_string(),
            arguments: vec![
                format!("metadata-digest={}", boundary.bootstrap_provider_metadata_digest_blake3),
                format!("policy-digest={}", boundary.bootstrap_provider_policy_digest_blake3),
            ],
        },
        RustProviderReceiptStep {
            name: "run-rustc-stage1".to_string(),
            program: RUSTC_STAGE1_SOURCE_BUILD_SCRIPT.to_string(),
            arguments: vec![
                format!("version={}", boundary.rust_version),
                format!("host={}", boundary.host_triple),
                format!("target={}", boundary.target_triple),
            ],
        },
        RustProviderReceiptStep {
            name: "record-rustc-stage1-products".to_string(),
            program: "mantle-provider-digest".to_string(),
            arguments: vec![
                "rustc".to_string(),
                "cargo".to_string(),
                "host-rustlib".to_string(),
                "target-rustlib".to_string(),
            ],
        },
    ]
}

fn rustc_stage1_provider_metadata(
    route: &LoadedRustSourceProviderRoute,
    sources: &[RustProviderSourceIdentity],
    receipt_artifacts: &[RustProviderReceiptArtifact],
    receipt_digest_blake3: &str,
    artifact_source_id: &str,
) -> RustSourceProviderMetadata {
    RustSourceProviderMetadata {
        schema: crate::source_toolchain_closure::RUST_SOURCE_PROVIDER_SCHEMA.to_string(),
        provider_id: RUST_SOURCE_PROVIDER_ID.to_string(),
        host_triple: route.plan.host_triple.clone(),
        target_triple: route.plan.target_triple.clone(),
        provenance: RustSourceProviderProvenance {
            source_built: true,
            uses_prebuilt_rust: false,
            build_recipe: RUSTC_STAGE1_PROVIDER_BUILD_RECIPE.to_string(),
        },
        sources: sources.to_vec(),
        build_receipts: vec![rustc_stage1_provider_receipt_identity(receipt_digest_blake3)],
        artifacts: rustc_stage1_provider_artifacts(receipt_artifacts, receipt_digest_blake3, artifact_source_id),
    }
}

fn rustc_stage1_provider_receipt_identity(receipt_digest_blake3: &str) -> RustProviderBuildReceiptIdentity {
    RustProviderBuildReceiptIdentity {
        id: RUSTC_STAGE1_PROVIDER_RECEIPT_ID.to_string(),
        kind: ToolchainBuildReceiptKind::ExternalAttestedBuild,
        name: RUSTC_STAGE1_PROVIDER_RECEIPT_NAME.to_string(),
        path: RUSTC_STAGE1_PROVIDER_RECEIPT_RELATIVE_PATH.to_string(),
        digest_blake3: receipt_digest_blake3.to_string(),
    }
}

fn rustc_stage1_provider_artifacts(
    receipt_artifacts: &[RustProviderReceiptArtifact],
    receipt_digest_blake3: &str,
    artifact_source_id: &str,
) -> Vec<RustProviderArtifact> {
    let mut artifacts: Vec<RustProviderArtifact> = receipt_artifacts
        .iter()
        .map(|artifact| RustProviderArtifact {
            role: artifact.role,
            name: artifact.name.clone(),
            path: artifact.path.clone(),
            content_digest_blake3: artifact.content_digest_blake3.clone(),
            source_id: artifact_source_id.to_string(),
            build_receipt_id: RUSTC_STAGE1_PROVIDER_RECEIPT_ID.to_string(),
        })
        .collect();
    artifacts.push(RustProviderArtifact {
        role: RustProviderRole::ProviderReceipt,
        name: RUSTC_STAGE1_PROVIDER_RECEIPT_NAME.to_string(),
        path: RUSTC_STAGE1_PROVIDER_RECEIPT_RELATIVE_PATH.to_string(),
        content_digest_blake3: receipt_digest_blake3.to_string(),
        source_id: artifact_source_id.to_string(),
        build_receipt_id: RUSTC_STAGE1_PROVIDER_RECEIPT_ID.to_string(),
    });
    artifacts
}

fn write_rustc_stage1_provider_candidate_manifest(
    boundary: &RustSourceProviderRustcStage1Boundary,
    candidate: &RustSourceProviderRustcStage1ProviderCandidate,
) -> Result<(), RustSourceProviderError> {
    let manifest = serde_json::json!({
        "schema": RUSTC_STAGE1_PROVIDER_CANDIDATE_SCHEMA,
        "stage_id": boundary.stage_id,
        "route_plan_digest_blake3": boundary.route_plan_digest_blake3,
        "route_policy_digest_blake3": boundary.route_policy_digest_blake3,
        "bootstrap_stage_id": boundary.bootstrap_stage_id,
        "bootstrap_provider_candidate": {
            "path": boundary.bootstrap_provider_candidate_dir.display().to_string(),
            "metadata_digest_blake3": boundary.bootstrap_provider_metadata_digest_blake3,
            "policy_digest_blake3": boundary.bootstrap_provider_policy_digest_blake3,
        },
        "candidate": candidate,
        "candidate_only": true,
        "final_output_written": false,
        "next_blocked_reason": RUST_SOURCE_PROVIDER_BLOCKED_REASON,
    });
    write_json_pretty(&boundary.provider_candidate_manifest_path, &manifest, "rustc stage1 provider candidate manifest")
}

fn run_first_stage_build(
    boundary: &RustSourceProviderFirstStageBoundary,
    verbose: bool,
) -> Result<RustSourceProviderFirstStageBuild, RustSourceProviderError> {
    let build_sources = first_stage_build_sources(boundary)?;
    validate_first_stage_build_inputs(&build_sources)?;
    if verbose {
        eprintln!("  first_stage_build_script: {}", boundary.script_path.display());
        eprintln!("  first_stage_build_log: {}", boundary.build_log_path.display());
    }
    let log = File::create(&boundary.build_log_path).map_err(|err| {
        RustSourceProviderError::Build(format!("create {}: {err}", boundary.build_log_path.display()))
    })?;
    let status = Command::new(&boundary.script_path)
        .stdout(log.try_clone().map_err(|err| {
            RustSourceProviderError::Build(format!("clone {}: {err}", boundary.build_log_path.display()))
        })?)
        .stderr(log)
        .status()
        .map_err(|err| RustSourceProviderError::Build(format!("launch {}: {err}", boundary.script_path.display())))?;
    if !status.success() {
        let log_tail = fs::read(&boundary.build_log_path)
            .map(|bytes| bounded_output_text(&bytes))
            .unwrap_or_else(|err| format!("<failed to read build log: {err}>"));
        return Err(RustSourceProviderError::Build(format!(
            "first-stage mrustc/minicargo build failed with status {status}; log={}; tail={log_tail:?}",
            boundary.build_log_path.display()
        )));
    }
    let mrustc_path = build_sources.mrustc_path.join(FIRST_STAGE_MRUSTC_BINARY);
    let minicargo_path = build_sources.mrustc_path.join(FIRST_STAGE_MINICARGO_BINARY);
    let translated_rustc_path = build_sources.mrustc_path.join(FIRST_STAGE_TRANSLATED_RUSTC_BINARY);
    let translated_cargo_path = build_sources.mrustc_path.join(FIRST_STAGE_TRANSLATED_CARGO_BINARY);
    let prefix_path = build_sources.mrustc_path.join(FIRST_STAGE_PREFIX_DIR);
    let prefix_rustlib_path = prefix_path.join("lib/rustlib").join(&boundary.target_triple).join("lib");
    validate_first_stage_build_product(&mrustc_path, FIRST_STAGE_MRUSTC_BINARY)?;
    validate_first_stage_build_product(&minicargo_path, FIRST_STAGE_MINICARGO_BINARY)?;
    validate_first_stage_build_product(&translated_rustc_path, FIRST_STAGE_TRANSLATED_RUSTC_BINARY)?;
    validate_first_stage_build_product(&translated_cargo_path, FIRST_STAGE_TRANSLATED_CARGO_BINARY)?;
    validate_first_stage_build_product(&prefix_path, FIRST_STAGE_PREFIX_DIR)?;
    validate_first_stage_build_product(&prefix_rustlib_path, "prefix rustlib")?;
    Ok(RustSourceProviderFirstStageBuild {
        stage_id: boundary.stage_id.clone(),
        script_path: boundary.script_path.clone(),
        log_path: boundary.build_log_path.clone(),
        mrustc_digest_blake3: file_digest_blake3(&mrustc_path)?,
        minicargo_digest_blake3: file_digest_blake3(&minicargo_path)?,
        translated_rustc_digest_blake3: file_digest_blake3(&translated_rustc_path)?,
        translated_cargo_digest_blake3: file_digest_blake3(&translated_cargo_path)?,
        prefix_digest_blake3: content_digest_blake3(&prefix_path)?,
        prefix_rustlib_digest_blake3: content_digest_blake3(&prefix_rustlib_path)?,
        mrustc_path,
        minicargo_path,
        translated_rustc_path,
        translated_cargo_path,
        prefix_path,
        prefix_rustlib_path,
    })
}

fn write_first_stage_build_manifest(
    boundary: &RustSourceProviderFirstStageBoundary,
    build: &RustSourceProviderFirstStageBuild,
) -> Result<(), RustSourceProviderError> {
    let manifest = serde_json::json!({
        "schema": FIRST_STAGE_BUILD_MANIFEST_SCHEMA,
        "stage_id": boundary.stage_id,
        "route_plan_digest_blake3": boundary.route_plan_digest_blake3,
        "route_policy_digest_blake3": boundary.route_policy_digest_blake3,
        "build": build,
        "next_blocked_reason": RUST_SOURCE_PROVIDER_BLOCKED_REASON,
    });
    let bytes = serde_json::to_vec_pretty(&manifest)
        .map_err(|err| RustSourceProviderError::Parse(format!("serialize first stage build manifest: {err}")))?;
    fs::write(&boundary.build_manifest_path, bytes).map_err(|err| {
        RustSourceProviderError::Read(format!(
            "write first stage build manifest {}: {err}",
            boundary.build_manifest_path.display()
        ))
    })
}

fn assemble_first_stage_provider_candidate(
    boundary: &RustSourceProviderFirstStageBoundary,
    route: &LoadedRustSourceProviderRoute,
    sources: &[RustSourceProviderFirstStageSourceAcquisition],
    build: &RustSourceProviderFirstStageBuild,
) -> Result<RustSourceProviderFirstStageProviderCandidate, RustSourceProviderError> {
    validate_first_stage_candidate_request(boundary, sources, build)?;
    prepare_empty_provider_candidate_dir(&boundary.provider_candidate_dir)?;
    copy_provider_prefix(&build.prefix_path, &boundary.provider_candidate_dir)?;
    let source_identities = first_stage_provider_source_identities(boundary, sources)?;
    let build_sources = first_stage_build_sources(boundary)?;
    let artifact_source_id = build_sources.rust_id;
    let receipt_artifacts = first_stage_provider_receipt_artifacts(&boundary.provider_candidate_dir, route)?;
    let receipt = first_stage_provider_receipt(boundary, route, &source_identities, &receipt_artifacts);
    let receipt_path = boundary.provider_candidate_dir.join(FIRST_STAGE_PROVIDER_RECEIPT_RELATIVE_PATH);
    write_json_pretty(&receipt_path, &receipt, "first-stage provider receipt")?;
    let receipt_digest_blake3 = file_digest_blake3(&receipt_path)?;
    let metadata = first_stage_provider_metadata(
        route,
        &source_identities,
        &receipt_artifacts,
        &receipt_digest_blake3,
        &artifact_source_id,
    );
    let metadata_path = boundary.provider_candidate_dir.join(RUST_SOURCE_PROVIDER_METADATA_PATH);
    write_json_pretty(&metadata_path, &metadata, "first-stage provider metadata")?;
    let validation = validate_materialized_rust_source_provider(&boundary.provider_candidate_dir)?;
    Ok(RustSourceProviderFirstStageProviderCandidate {
        stage_id: boundary.stage_id.clone(),
        candidate_dir: boundary.provider_candidate_dir.clone(),
        metadata_path,
        metadata_digest_blake3: validation.metadata_digest_blake3,
        policy_digest_blake3: validation.validation.policy_digest_blake3,
        artifact_count: validation.validation.artifact_count,
        source_count: validation.validation.source_count,
        receipt_count: validation.validation.receipt_count,
    })
}

fn smoke_first_stage_provider_candidate(
    boundary: &RustSourceProviderFirstStageBoundary,
    candidate: &RustSourceProviderFirstStageProviderCandidate,
) -> Result<RustSourceProviderSmokeEvidence, RustSourceProviderError> {
    let validation = validate_materialized_rust_source_provider(&candidate.candidate_dir)?;
    let smoke = smoke_rust_source_provider(&candidate.candidate_dir, &boundary.provider_candidate_smoke_work_dir)?;
    persist_rust_source_provider_smoke_evidence(&boundary.provider_candidate_smoke_dir, &smoke, &validation)
}

fn validate_first_stage_candidate_request(
    boundary: &RustSourceProviderFirstStageBoundary,
    sources: &[RustSourceProviderFirstStageSourceAcquisition],
    build: &RustSourceProviderFirstStageBuild,
) -> Result<(), RustSourceProviderError> {
    if sources.len() != boundary.sources.len() {
        return Err(RustSourceProviderError::Validate("first-stage candidate source count mismatch".to_string()));
    }
    if build.stage_id != boundary.stage_id {
        return Err(RustSourceProviderError::Validate(format!(
            "first-stage candidate build stage expected {}, got {}",
            boundary.stage_id, build.stage_id
        )));
    }
    if boundary.provider_candidate_dir == boundary.output_dir {
        return Err(RustSourceProviderError::Validate(
            "provider candidate must not be the final output dir".to_string(),
        ));
    }
    validate_first_stage_build_product(&build.prefix_path, FIRST_STAGE_PREFIX_DIR)
}

fn prepare_empty_provider_candidate_dir(path: &Path) -> Result<(), RustSourceProviderError> {
    if path.exists() && path.is_dir() {
        fs::remove_dir_all(path)
            .map_err(|err| RustSourceProviderError::Copy(format!("remove {}: {err}", path.display())))?;
    } else if path.exists() {
        fs::remove_file(path)
            .map_err(|err| RustSourceProviderError::Copy(format!("remove {}: {err}", path.display())))?;
    }
    fs::create_dir_all(path).map_err(|err| RustSourceProviderError::Copy(format!("create {}: {err}", path.display())))
}

fn copy_provider_prefix(src: &Path, dst: &Path) -> Result<(), RustSourceProviderError> {
    let mut copied_entries = 0usize;
    copy_directory_contents(src, dst, &mut copied_entries)?;
    if copied_entries == 0 {
        return Err(RustSourceProviderError::Copy(format!("provider prefix {} was empty", src.display())));
    }
    Ok(())
}

fn first_stage_provider_source_identities(
    boundary: &RustSourceProviderFirstStageBoundary,
    sources: &[RustSourceProviderFirstStageSourceAcquisition],
) -> Result<Vec<RustProviderSourceIdentity>, RustSourceProviderError> {
    let mut identities = Vec::with_capacity(boundary.sources.len());
    for source in &boundary.sources {
        let acquired = sources.iter().find(|acquired| acquired.id == source.id).ok_or_else(|| {
            RustSourceProviderError::Validate(format!("first-stage candidate lacks source acquisition '{}'", source.id))
        })?;
        identities.push(RustProviderSourceIdentity {
            id: source.id.clone(),
            kind: source.kind,
            name: source.name.clone(),
            digest_blake3: acquired.extracted_digest_blake3.clone(),
        });
    }
    Ok(identities)
}

fn first_stage_provider_receipt_artifacts(
    candidate_dir: &Path,
    route: &LoadedRustSourceProviderRoute,
) -> Result<Vec<RustProviderReceiptArtifact>, RustSourceProviderError> {
    Ok(vec![
        provider_receipt_artifact(candidate_dir, RustProviderRole::Rustc, "rustc", PROVIDER_RUSTC_RELATIVE_PATH)?,
        provider_receipt_artifact(candidate_dir, RustProviderRole::Cargo, "cargo", PROVIDER_CARGO_RELATIVE_PATH)?,
        provider_receipt_artifact(
            candidate_dir,
            RustProviderRole::HostRustlib,
            "host-rustlib",
            &provider_rustlib_relative_path(&route.plan.host_triple),
        )?,
        provider_receipt_artifact(
            candidate_dir,
            RustProviderRole::TargetRustlib,
            "target-rustlib",
            &provider_rustlib_relative_path(&route.plan.target_triple),
        )?,
    ])
}

fn provider_receipt_artifact(
    provider_dir: &Path,
    role: RustProviderRole,
    name: &str,
    relative_path: &str,
) -> Result<RustProviderReceiptArtifact, RustSourceProviderError> {
    Ok(RustProviderReceiptArtifact {
        role,
        name: name.to_string(),
        path: relative_path.to_string(),
        content_digest_blake3: content_digest_blake3(&provider_dir.join(relative_path))?,
    })
}

fn first_stage_provider_receipt(
    boundary: &RustSourceProviderFirstStageBoundary,
    route: &LoadedRustSourceProviderRoute,
    sources: &[RustProviderSourceIdentity],
    artifacts: &[RustProviderReceiptArtifact],
) -> RustSourceProviderBuildReceipt {
    RustSourceProviderBuildReceipt {
        schema: RUST_SOURCE_PROVIDER_RECEIPT_SCHEMA.to_string(),
        receipt_id: FIRST_STAGE_PROVIDER_RECEIPT_ID.to_string(),
        provider_id: RUST_SOURCE_PROVIDER_ID.to_string(),
        host_triple: route.plan.host_triple.clone(),
        target_triple: route.plan.target_triple.clone(),
        source_ids: sources.iter().map(|source| source.id.clone()).collect(),
        output_artifacts: artifacts.to_vec(),
        build_steps: first_stage_provider_receipt_steps(boundary),
    }
}

fn first_stage_provider_receipt_steps(boundary: &RustSourceProviderFirstStageBoundary) -> Vec<RustProviderReceiptStep> {
    vec![
        RustProviderReceiptStep {
            name: "acquire-first-stage-sources".to_string(),
            program: "mantle-rust-source-fetch".to_string(),
            arguments: boundary.source_ids.clone(),
        },
        RustProviderReceiptStep {
            name: "build-mrustc".to_string(),
            program: FIRST_STAGE_MAKE_PROGRAM.to_string(),
            arguments: vec![format!("CXXFLAGS={FIRST_STAGE_CXXFLAGS}")],
        },
        RustProviderReceiptStep {
            name: "build-minicargo".to_string(),
            program: FIRST_STAGE_MAKE_PROGRAM.to_string(),
            arguments: vec![
                "-f".to_string(),
                FIRST_STAGE_MINICARGO_MAKEFILE.to_string(),
                FIRST_STAGE_MINICARGO_BINARY.to_string(),
            ],
        },
        RustProviderReceiptStep {
            name: "translate-rustc".to_string(),
            program: FIRST_STAGE_MAKE_PROGRAM.to_string(),
            arguments: vec![
                "-f".to_string(),
                FIRST_STAGE_MINICARGO_MAKEFILE.to_string(),
                FIRST_STAGE_TRANSLATED_RUSTC_BINARY.to_string(),
            ],
        },
        RustProviderReceiptStep {
            name: "translate-cargo".to_string(),
            program: FIRST_STAGE_MAKE_PROGRAM.to_string(),
            arguments: vec![
                "-f".to_string(),
                FIRST_STAGE_MINICARGO_MAKEFILE.to_string(),
                FIRST_STAGE_TRANSLATED_CARGO_BINARY.to_string(),
            ],
        },
        RustProviderReceiptStep {
            name: "build-rust-prefix".to_string(),
            program: FIRST_STAGE_MAKE_PROGRAM.to_string(),
            arguments: vec!["-C".to_string(), FIRST_STAGE_RUN_RUSTC_DIR.to_string()],
        },
    ]
}

fn first_stage_provider_metadata(
    route: &LoadedRustSourceProviderRoute,
    sources: &[RustProviderSourceIdentity],
    receipt_artifacts: &[RustProviderReceiptArtifact],
    receipt_digest_blake3: &str,
    artifact_source_id: &str,
) -> RustSourceProviderMetadata {
    RustSourceProviderMetadata {
        schema: crate::source_toolchain_closure::RUST_SOURCE_PROVIDER_SCHEMA.to_string(),
        provider_id: RUST_SOURCE_PROVIDER_ID.to_string(),
        host_triple: route.plan.host_triple.clone(),
        target_triple: route.plan.target_triple.clone(),
        provenance: RustSourceProviderProvenance {
            source_built: true,
            uses_prebuilt_rust: false,
            build_recipe: FIRST_STAGE_PROVIDER_BUILD_RECIPE.to_string(),
        },
        sources: sources.to_vec(),
        build_receipts: vec![first_stage_provider_receipt_identity(receipt_digest_blake3)],
        artifacts: first_stage_provider_artifacts(receipt_artifacts, receipt_digest_blake3, artifact_source_id),
    }
}

fn first_stage_provider_receipt_identity(receipt_digest_blake3: &str) -> RustProviderBuildReceiptIdentity {
    RustProviderBuildReceiptIdentity {
        id: FIRST_STAGE_PROVIDER_RECEIPT_ID.to_string(),
        kind: ToolchainBuildReceiptKind::ExternalAttestedBuild,
        name: FIRST_STAGE_PROVIDER_RECEIPT_NAME.to_string(),
        path: FIRST_STAGE_PROVIDER_RECEIPT_RELATIVE_PATH.to_string(),
        digest_blake3: receipt_digest_blake3.to_string(),
    }
}

fn first_stage_provider_artifacts(
    receipt_artifacts: &[RustProviderReceiptArtifact],
    receipt_digest_blake3: &str,
    artifact_source_id: &str,
) -> Vec<RustProviderArtifact> {
    let mut artifacts: Vec<RustProviderArtifact> = receipt_artifacts
        .iter()
        .map(|artifact| RustProviderArtifact {
            role: artifact.role,
            name: artifact.name.clone(),
            path: artifact.path.clone(),
            content_digest_blake3: artifact.content_digest_blake3.clone(),
            source_id: artifact_source_id.to_string(),
            build_receipt_id: FIRST_STAGE_PROVIDER_RECEIPT_ID.to_string(),
        })
        .collect();
    artifacts.push(RustProviderArtifact {
        role: RustProviderRole::ProviderReceipt,
        name: FIRST_STAGE_PROVIDER_RECEIPT_NAME.to_string(),
        path: FIRST_STAGE_PROVIDER_RECEIPT_RELATIVE_PATH.to_string(),
        content_digest_blake3: receipt_digest_blake3.to_string(),
        source_id: artifact_source_id.to_string(),
        build_receipt_id: FIRST_STAGE_PROVIDER_RECEIPT_ID.to_string(),
    });
    artifacts
}

fn provider_rustlib_relative_path(triple: &str) -> String {
    format!("{PROVIDER_RUSTLIB_PREFIX}/{triple}/lib")
}

fn write_first_stage_provider_candidate_manifest(
    boundary: &RustSourceProviderFirstStageBoundary,
    candidate: &RustSourceProviderFirstStageProviderCandidate,
) -> Result<(), RustSourceProviderError> {
    let manifest = serde_json::json!({
        "schema": FIRST_STAGE_PROVIDER_CANDIDATE_SCHEMA,
        "stage_id": boundary.stage_id,
        "route_plan_digest_blake3": boundary.route_plan_digest_blake3,
        "route_policy_digest_blake3": boundary.route_policy_digest_blake3,
        "candidate": candidate,
        "candidate_only": true,
        "next_blocked_reason": RUST_SOURCE_PROVIDER_BLOCKED_REASON,
    });
    write_json_pretty(&boundary.provider_candidate_manifest_path, &manifest, "first-stage provider candidate manifest")
}

fn first_stage_build_sources(
    boundary: &RustSourceProviderFirstStageBoundary,
) -> Result<RustSourceProviderFirstStageBuildSources, RustSourceProviderError> {
    let mrustc = select_first_stage_source(boundary, FIRST_STAGE_MRUSTC_SOURCE_PREFIX)?;
    let rust = select_first_stage_source(boundary, FIRST_STAGE_RUST_SOURCE_PREFIX)?;
    let mrustc_id = safe_source_component(&mrustc.id)?;
    let rust_id = safe_source_component(&rust.id)?;
    Ok(RustSourceProviderFirstStageBuildSources {
        mrustc_path: boundary.source_dir.join(&mrustc_id),
        rust_path: boundary.source_dir.join(&rust_id),
        mrustc_archive_path: boundary.archive_dir.join(format!("{mrustc_id}.{FIRST_STAGE_SOURCE_ARCHIVE_EXTENSION}")),
        rust_archive_path: boundary.archive_dir.join(format!("{rust_id}.{FIRST_STAGE_SOURCE_ARCHIVE_EXTENSION}")),
        mrustc_id,
        rust_id,
    })
}

fn select_first_stage_source<'a>(
    boundary: &'a RustSourceProviderFirstStageBoundary,
    id_prefix: &str,
) -> Result<&'a RustSourceProviderBootstrapSource, RustSourceProviderError> {
    let mut matches = boundary.sources.iter().filter(|source| source.id.starts_with(id_prefix));
    let Some(source) = matches.next() else {
        return Err(RustSourceProviderError::Build(format!(
            "first stage '{}' has no source with id prefix '{id_prefix}'",
            boundary.stage_id
        )));
    };
    if matches.next().is_none() {
        return Ok(source);
    }
    Err(RustSourceProviderError::Build(format!(
        "first stage '{}' has multiple sources with id prefix '{id_prefix}'",
        boundary.stage_id
    )))
}

fn validate_first_stage_build_inputs(
    sources: &RustSourceProviderFirstStageBuildSources,
) -> Result<(), RustSourceProviderError> {
    require_first_stage_path(&sources.mrustc_path, "mrustc source dir", true)?;
    require_first_stage_path(&sources.rust_path, "Rust source dir", true)?;
    require_first_stage_path(&sources.mrustc_archive_path, "mrustc source archive", false)?;
    require_first_stage_path(&sources.rust_archive_path, "Rust source archive", false)?;
    require_first_stage_path(&sources.mrustc_path.join(FIRST_STAGE_MAKEFILE), "mrustc Makefile", false)?;
    require_first_stage_path(
        &sources.mrustc_path.join(FIRST_STAGE_MINICARGO_MAKEFILE),
        "mrustc minicargo makefile",
        false,
    )
}

fn require_first_stage_path(path: &Path, label: &str, directory: bool) -> Result<(), RustSourceProviderError> {
    if directory && path.is_dir() {
        return Ok(());
    }
    if !directory && path.is_file() {
        return Ok(());
    }
    Err(RustSourceProviderError::Build(format!("missing {label}: {}", path.display())))
}

fn validate_first_stage_build_product(path: &Path, label: &str) -> Result<(), RustSourceProviderError> {
    if path.is_file() || path.is_dir() {
        return Ok(());
    }
    Err(RustSourceProviderError::Build(format!("missing first-stage product {label}: {}", path.display())))
}

fn fetch_first_stage_source_bytes(
    source: &RustSourceProviderBootstrapSource,
) -> Result<Vec<u8>, RustSourceProviderError> {
    let parsed = url::Url::parse(&source.url)
        .map_err(|err| RustSourceProviderError::Fetch(format!("source '{}' url parse: {err}", source.id)))?;
    match parsed.scheme() {
        "file" => read_file_url_limited(&source.id, &parsed),
        "http" | "https" => fetch_http_source_bytes(&source.id, &source.url),
        scheme => Err(RustSourceProviderError::Fetch(format!(
            "source '{}' uses unsupported url scheme '{scheme}'",
            source.id
        ))),
    }
}

fn read_file_url_limited(source_id: &str, url: &url::Url) -> Result<Vec<u8>, RustSourceProviderError> {
    let path = url
        .to_file_path()
        .map_err(|_| RustSourceProviderError::Fetch(format!("source '{source_id}' file url is not a local path")))?;
    let mut file = File::open(&path).map_err(|err| {
        RustSourceProviderError::Fetch(format!("source '{source_id}' open {}: {err}", path.display()))
    })?;
    read_limited_bytes(&mut file, &format!("source '{source_id}' {}", path.display()))
}

fn fetch_http_source_bytes(source_id: &str, url: &str) -> Result<Vec<u8>, RustSourceProviderError> {
    let agent = ureq::Agent::config_builder()
        .timeout_global(Some(std::time::Duration::from_secs(FIRST_STAGE_FETCH_TIMEOUT_SECS)))
        .build()
        .into();
    let mut last_error = String::new();
    for attempt in 0..=FIRST_STAGE_FETCH_MAX_RETRIES {
        if attempt != 0 {
            sleep_before_first_stage_fetch_retry(url, attempt);
        }
        match fetch_http_source_once(&agent, source_id, url) {
            Ok(bytes) => return Ok(bytes),
            Err(err) => {
                last_error = err.to_string();
                if !is_transient_first_stage_fetch_error(&last_error) {
                    return Err(err);
                }
            }
        }
    }
    Err(RustSourceProviderError::Fetch(last_error))
}

fn fetch_http_source_once(agent: &ureq::Agent, source_id: &str, url: &str) -> Result<Vec<u8>, RustSourceProviderError> {
    let response = agent
        .get(url)
        .call()
        .map_err(|err| RustSourceProviderError::Fetch(format!("source '{source_id}' {url}: {err}")))?;
    let mut body = response.into_body();
    let mut reader = body.with_config().limit(FIRST_STAGE_FETCH_MAX_BYTES).reader();
    read_limited_bytes(&mut reader, &format!("source '{source_id}' {url}"))
}

fn sleep_before_first_stage_fetch_retry(url: &str, attempt: u32) {
    let exponent = attempt.saturating_sub(1).min(FIRST_STAGE_FETCH_MAX_RETRIES);
    let delay_ms = FIRST_STAGE_FETCH_RETRY_BASE_DELAY_MS.saturating_mul(1_u64 << exponent);
    eprintln!("  [rust-source-provider] retry {attempt}/{FIRST_STAGE_FETCH_MAX_RETRIES} for {url} after {delay_ms}ms");
    std::thread::sleep(std::time::Duration::from_millis(delay_ms));
}

fn read_limited_bytes(reader: &mut dyn Read, label: &str) -> Result<Vec<u8>, RustSourceProviderError> {
    let mut bytes = Vec::new();
    reader
        .take(FIRST_STAGE_FETCH_MAX_BYTES)
        .read_to_end(&mut bytes)
        .map_err(|err| RustSourceProviderError::Fetch(format!("{label}: reading body: {err}")))?;
    if bytes.is_empty() {
        return Err(RustSourceProviderError::Fetch(format!("{label}: empty source body")));
    }
    Ok(bytes)
}

fn is_transient_first_stage_fetch_error(message: &str) -> bool {
    let lower = message.to_ascii_lowercase();
    lower.contains("network is unreachable")
        || lower.contains("connection reset")
        || lower.contains("timed out")
        || lower.contains("try again")
        || lower.contains("502")
        || lower.contains("503")
        || lower.contains("504")
}

fn ensure_supported_first_stage_archive(url: &str) -> Result<(), RustSourceProviderError> {
    let lower = url.to_ascii_lowercase();
    if lower.ends_with(FIRST_STAGE_TAR_GZ_EXTENSION) || lower.ends_with(FIRST_STAGE_TGZ_EXTENSION) {
        return Ok(());
    }
    Err(RustSourceProviderError::Extract(format!("first stage source archive must be tar.gz/tgz: {url}")))
}

fn prepare_empty_directory(path: &Path) -> Result<(), RustSourceProviderError> {
    if path.exists() && path.is_dir() {
        fs::remove_dir_all(path)
            .map_err(|err| RustSourceProviderError::Extract(format!("remove {}: {err}", path.display())))?;
    } else if path.exists() {
        fs::remove_file(path)
            .map_err(|err| RustSourceProviderError::Extract(format!("remove {}: {err}", path.display())))?;
    }
    fs::create_dir_all(path)
        .map_err(|err| RustSourceProviderError::Extract(format!("create {}: {err}", path.display())))
}

fn extract_first_stage_tar_gz(archive_path: &Path, dest: &Path) -> Result<u32, RustSourceProviderError> {
    let file = File::open(archive_path)
        .map_err(|err| RustSourceProviderError::Extract(format!("open {}: {err}", archive_path.display())))?;
    let decoder = flate2::read::GzDecoder::new(file);
    let mut archive = tar::Archive::new(decoder);
    let mut entry_count = 0_u32;
    for entry in archive
        .entries()
        .map_err(|err| RustSourceProviderError::Extract(format!("read {}: {err}", archive_path.display())))?
    {
        entry_count = entry_count.checked_add(1).ok_or_else(|| {
            RustSourceProviderError::Extract("first stage archive entry count overflowed".to_string())
        })?;
        if entry_count > FIRST_STAGE_ARCHIVE_MAX_ENTRIES {
            return Err(RustSourceProviderError::Extract(format!(
                "first stage archive exceeds {FIRST_STAGE_ARCHIVE_MAX_ENTRIES} entries"
            )));
        }
        let entry = entry.map_err(|err| RustSourceProviderError::Extract(format!("read archive entry: {err}")))?;
        unpack_first_stage_entry(entry, dest)?;
    }
    if entry_count == 0 {
        return Err(RustSourceProviderError::Extract(format!("{} is empty", archive_path.display())));
    }
    Ok(entry_count)
}

fn unpack_first_stage_entry<R: Read>(mut entry: tar::Entry<'_, R>, dest: &Path) -> Result<(), RustSourceProviderError> {
    let raw_path = entry
        .path()
        .map_err(|err| RustSourceProviderError::Extract(format!("read archive path: {err}")))?
        .into_owned();
    let Some(relative_path) = strip_first_archive_component(&raw_path)? else {
        return Ok(());
    };
    let output_path = dest.join(relative_path);
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|err| RustSourceProviderError::Extract(format!("create {}: {err}", parent.display())))?;
    }
    entry
        .unpack(&output_path)
        .map_err(|err| RustSourceProviderError::Extract(format!("unpack {}: {err}", output_path.display())))?;
    Ok(())
}

fn strip_first_archive_component(path: &Path) -> Result<Option<PathBuf>, RustSourceProviderError> {
    let mut components = path.components();
    let Some(first) = components.next() else {
        return Ok(None);
    };
    if !matches!(first, Component::Normal(_)) {
        return Err(RustSourceProviderError::Extract(format!("unsafe archive path {}", path.display())));
    }
    let mut stripped = PathBuf::new();
    for component in components {
        match component {
            Component::Normal(part) => stripped.push(part),
            Component::CurDir => {}
            _ => return Err(RustSourceProviderError::Extract(format!("unsafe archive path {}", path.display()))),
        }
    }
    if stripped.as_os_str().is_empty() {
        return Ok(None);
    }
    Ok(Some(stripped))
}

fn safe_source_component(source_id: &str) -> Result<String, RustSourceProviderError> {
    let mut components = Path::new(source_id).components();
    let Some(Component::Normal(first)) = components.next() else {
        return Err(RustSourceProviderError::Validate(format!("source id '{source_id}' is not a path component")));
    };
    if components.next().is_some() {
        return Err(RustSourceProviderError::Validate(format!(
            "source id '{source_id}' is not a single path component"
        )));
    }
    Ok(first.to_string_lossy().to_string())
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = <sha2::Sha256 as sha2::Digest>::new();
    <sha2::Sha256 as sha2::Digest>::update(&mut hasher, bytes);
    data_encoding::HEXLOWER.encode(&<sha2::Sha256 as sha2::Digest>::finalize(hasher))
}

fn first_stage_boundary_script(
    boundary: &RustSourceProviderFirstStageBoundary,
) -> Result<String, RustSourceProviderError> {
    let build_sources = first_stage_build_sources(boundary)?;
    let mut script = String::new();
    push_first_stage_script_header(&mut script, boundary, &build_sources);
    push_first_stage_source_checks(&mut script, boundary);
    push_first_stage_tool_checks(&mut script);
    push_first_stage_build_commands(&mut script);
    Ok(script)
}

fn push_first_stage_script_header(
    script: &mut String,
    boundary: &RustSourceProviderFirstStageBoundary,
    build_sources: &RustSourceProviderFirstStageBuildSources,
) {
    script.push_str("#!/bin/sh\n");
    script.push_str("set -eu\n");
    script.push_str(&format!("STAGE_ID={}\n", shell_quote(&boundary.stage_id)));
    script.push_str(&format!("RUSTC_VERSION={}\n", shell_quote(&boundary.rust_version)));
    script.push_str(&format!("RUSTC_TARGET={}\n", shell_quote(&boundary.target_triple)));
    script.push_str(&format!("MRUSTC_TARGET_VER={}\n", shell_quote(&boundary.mrustc_target_version)));
    script.push_str(&format!("SOURCE_DIR={}\n", shell_quote(&boundary.source_dir.display().to_string())));
    script.push_str(&format!("BUILD_DIR={}\n", shell_quote(&boundary.build_dir.display().to_string())));
    script.push_str(&format!("OUTPUT_DIR={}\n", shell_quote(&boundary.output_dir.display().to_string())));
    script.push_str(&format!("ARCHIVE_DIR={}\n", shell_quote(&boundary.archive_dir.display().to_string())));
    script.push_str(&format!("MRUSTC_SOURCE_ID={}\n", shell_quote(&build_sources.mrustc_id)));
    script.push_str(&format!("RUST_SOURCE_ID={}\n", shell_quote(&build_sources.rust_id)));
    script.push_str(&format!("MRUSTC_SOURCE={}\n", shell_quote(&build_sources.mrustc_path.display().to_string())));
    script.push_str(&format!("RUST_SOURCE={}\n", shell_quote(&build_sources.rust_path.display().to_string())));
    script.push_str(&format!(
        "MRUSTC_ARCHIVE={}\n",
        shell_quote(&build_sources.mrustc_archive_path.display().to_string())
    ));
    script.push_str(&format!("RUST_ARCHIVE={}\n", shell_quote(&build_sources.rust_archive_path.display().to_string())));
    script
        .push_str(&format!("SOURCE_MANIFEST={}\n", shell_quote(&boundary.sources_manifest_path.display().to_string())));
    script.push_str(&format!("MAKE_PROGRAM={}\n", shell_quote(FIRST_STAGE_MAKE_PROGRAM)));
    script.push_str(&format!("MAKE_FALLBACK_GLOB={}\n", shell_quote(FIRST_STAGE_MAKE_FALLBACK_GLOB)));
    script.push_str(&format!("COPY_PROGRAM={}\n", shell_quote(FIRST_STAGE_COPY_PROGRAM)));
    script.push_str(&format!("MRUSTC_CXXFLAGS={}\n", shell_quote(FIRST_STAGE_CXXFLAGS)));
}

fn push_first_stage_source_checks(script: &mut String, boundary: &RustSourceProviderFirstStageBoundary) {
    script.push_str("printf '%s\\n' \"mantle rust source first stage: $STAGE_ID\"\n");
    script.push_str(&format!(
        "if [ ! -f \"$SOURCE_MANIFEST\" ]; then printf '%s\\n' {} >&2; exit {}; fi\n",
        shell_quote("missing verified source manifest"),
        FIRST_STAGE_MISSING_SOURCE_EXIT_CODE
    ));
    for source in &boundary.sources {
        script.push_str(&format!(
            "if [ ! -d \"$SOURCE_DIR\"/{id} ]; then printf '%s\\n' {message} >&2; exit {missing}; fi\n",
            id = shell_quote(&source.id),
            message = shell_quote(&format!(
                "missing verified source '{}' from {} with sha256 {}",
                source.id, source.url, source.sha256_hex
            )),
            missing = FIRST_STAGE_MISSING_SOURCE_EXIT_CODE,
        ));
    }
    push_required_shell_var_file_check(
        script,
        "MRUSTC_ARCHIVE",
        "missing verified mrustc archive",
        FIRST_STAGE_MISSING_SOURCE_EXIT_CODE,
    );
    push_required_shell_var_file_check(
        script,
        "RUST_ARCHIVE",
        "missing verified Rust archive",
        FIRST_STAGE_MISSING_SOURCE_EXIT_CODE,
    );
    push_mrustc_source_file_check(script, FIRST_STAGE_MAKEFILE, "mrustc source has no Makefile");
    push_mrustc_source_file_check(script, FIRST_STAGE_MINICARGO_MAKEFILE, "mrustc source has no minicargo.mk");
}

fn push_first_stage_tool_checks(script: &mut String) {
    script.push_str(
        "if ! command -v \"$MAKE_PROGRAM\" >/dev/null 2>&1; then for candidate in $MAKE_FALLBACK_GLOB; do if [ -x \"$candidate\" ]; then MAKE_PROGRAM=\"$candidate\"; break; fi; done; fi\n",
    );
    push_required_shell_command_check(script, "MAKE_PROGRAM", "make is required for mrustc first stage");
    push_required_shell_command_check(script, "COPY_PROGRAM", "cp is required for mrustc first stage");
}

fn push_first_stage_build_commands(script: &mut String) {
    script.push_str(
        "printf '%s\\n' \"verified sources manifest: $SOURCE_MANIFEST\"\nprintf '%s\\n' \"build dir: $BUILD_DIR\"\nprintf '%s\\n' \"output dir: $OUTPUT_DIR\"\n",
    );
    script.push_str("mkdir -p \"$BUILD_DIR\"\n");
    script.push_str("$COPY_PROGRAM \"$RUST_ARCHIVE\" \"$MRUSTC_SOURCE/rustc-${RUSTC_VERSION}-src.tar.gz\"\n");
    script.push_str("cd \"$MRUSTC_SOURCE\"\n");
    script.push_str("export RUSTC_TARGET MRUSTC_TARGET_VER RUSTC_VERSION\n");
    script.push_str("export RUSTC_INSTALL_BINDIR=bin\n");
    script.push_str("export OUTDIR_SUF=\n");
    script.push_str("export CARGO_CFG_RUSTIX_NO_LINUX_RAW=1\n");
    script.push_str("$MAKE_PROGRAM CXXFLAGS=\"$MRUSTC_CXXFLAGS\"\n");
    script.push_str(&format!("$MAKE_PROGRAM -f {FIRST_STAGE_MINICARGO_MAKEFILE} {FIRST_STAGE_MINICARGO_BINARY}\n"));
    script.push_str(&format!(
        "$MAKE_PROGRAM -f {FIRST_STAGE_MINICARGO_MAKEFILE} {FIRST_STAGE_TRANSLATED_RUSTC_BINARY}\n"
    ));
    script.push_str(&format!(
        "$MAKE_PROGRAM -f {FIRST_STAGE_MINICARGO_MAKEFILE} {FIRST_STAGE_TRANSLATED_CARGO_BINARY}\n"
    ));
    script.push_str(&format!("$MAKE_PROGRAM -C {FIRST_STAGE_RUN_RUSTC_DIR}\n"));
    push_required_relative_file_check(script, FIRST_STAGE_MRUSTC_BINARY, "mrustc build did not produce bin/mrustc");
    push_required_relative_file_check(
        script,
        FIRST_STAGE_MINICARGO_BINARY,
        "mrustc build did not produce bin/minicargo",
    );
    push_required_relative_file_check(
        script,
        FIRST_STAGE_TRANSLATED_RUSTC_BINARY,
        "mrustc build did not produce output/rustc",
    );
    push_required_relative_file_check(
        script,
        FIRST_STAGE_TRANSLATED_CARGO_BINARY,
        "mrustc build did not produce output/cargo",
    );
    push_required_relative_file_check(
        script,
        FIRST_STAGE_PREFIX_RUSTC_BINARY,
        "mrustc run_rustc did not produce prefix rustc",
    );
    push_required_relative_file_check(
        script,
        FIRST_STAGE_PREFIX_CARGO_BINARY,
        "mrustc run_rustc did not produce prefix cargo",
    );
    script.push_str("printf '%s\\n' \"Rust 1.90 first-stage products ready\"\n");
}

fn push_mrustc_source_file_check(script: &mut String, file_name: &str, message: &str) {
    script.push_str(&format!(
        "if [ ! -f \"$MRUSTC_SOURCE/{file_name}\" ]; then printf '%s\\n' {} >&2; exit {}; fi\n",
        shell_quote(message),
        FIRST_STAGE_BUILD_FAILED_EXIT_CODE
    ));
}

fn push_required_shell_var_file_check(script: &mut String, shell_var: &str, message: &str, exit_code: i32) {
    script.push_str(&format!(
        "if [ ! -f \"${shell_var}\" ]; then printf '%s\\n' {} >&2; exit {exit_code}; fi\n",
        shell_quote(message)
    ));
}

fn push_required_shell_var_dir_check(script: &mut String, shell_var: &str, message: &str, exit_code: i32) {
    script.push_str(&format!(
        "if [ ! -d \"${shell_var}\" ]; then printf '%s\\n' {} >&2; exit {exit_code}; fi\n",
        shell_quote(message)
    ));
}

fn push_required_shell_path_file_check(script: &mut String, shell_expr: &str, message: &str, exit_code: i32) {
    script.push_str(&format!(
        "if [ ! -f {shell_expr} ]; then printf '%s\\n' {} >&2; exit {exit_code}; fi\n",
        shell_quote(message)
    ));
}

fn push_required_shell_path_dir_check(script: &mut String, shell_expr: &str, message: &str, exit_code: i32) {
    script.push_str(&format!(
        "if [ ! -d {shell_expr} ]; then printf '%s\\n' {} >&2; exit {exit_code}; fi\n",
        shell_quote(message)
    ));
}

fn push_required_relative_file_check(script: &mut String, relative_path: &str, message: &str) {
    script.push_str(&format!(
        "if [ ! -f {relative_path} ]; then printf '%s\\n' {} >&2; exit {}; fi\n",
        shell_quote(message),
        FIRST_STAGE_BUILD_FAILED_EXIT_CODE
    ));
}

fn push_required_shell_command_check(script: &mut String, shell_var: &str, message: &str) {
    script.push_str(&format!(
        "if ! command -v \"${shell_var}\" >/dev/null 2>&1; then printf '%s\\n' {} >&2; exit {}; fi\n",
        shell_quote(message),
        FIRST_STAGE_BUILD_FAILED_EXIT_CODE
    ));
}

fn shell_quote(value: &str) -> String {
    let escaped = value.replace('\'', "'\\''");
    format!("'{escaped}'")
}

#[cfg(unix)]
fn make_executable(path: &Path) -> Result<(), RustSourceProviderError> {
    use std::os::unix::fs::PermissionsExt;

    const EXECUTABLE_FILE_MODE: u32 = 0o755;
    let mut permissions = fs::metadata(path)
        .map_err(|err| RustSourceProviderError::Read(format!("metadata {}: {err}", path.display())))?
        .permissions();
    permissions.set_mode(EXECUTABLE_FILE_MODE);
    fs::set_permissions(path, permissions)
        .map_err(|err| RustSourceProviderError::Read(format!("chmod {}: {err}", path.display())))
}

#[cfg(not(unix))]
fn make_executable(path: &Path) -> Result<(), RustSourceProviderError> {
    if path.is_file() {
        return Ok(());
    }
    Err(RustSourceProviderError::Read(format!("first stage script {} is missing", path.display())))
}

fn plan_smoke(
    provider_dir: &Path,
    scratch_dir: &Path,
    metadata: &RustSourceProviderMetadata,
) -> Result<RustSourceProviderSmokePlan, RustSourceProviderError> {
    if provider_dir.as_os_str().is_empty() {
        return Err(RustSourceProviderError::Smoke("provider dir is empty".to_string()));
    }
    if scratch_dir.as_os_str().is_empty() {
        return Err(RustSourceProviderError::Smoke("smoke scratch dir is empty".to_string()));
    }
    if metadata.host_triple.trim().is_empty() {
        return Err(RustSourceProviderError::Smoke("provider host triple is empty".to_string()));
    }
    let rustc_path = provider_dir.join("bin/rustc");
    Ok(RustSourceProviderSmokePlan {
        provider_dir: provider_dir.to_path_buf(),
        scratch_dir: scratch_dir.to_path_buf(),
        rustc_path,
        source_path: scratch_dir.join(SMOKE_SOURCE_FILE),
        output_path: scratch_dir.join(SMOKE_OUTPUT_FILE),
        target_triple: metadata.host_triple.clone(),
    })
}

fn smoke_source_text() -> String {
    format!(
        "#![no_std]\n#[no_mangle]\npub extern \"C\" fn mantle_rust_provider_smoke() -> u32 {{ {SMOKE_RETURN_CODE} }}\n"
    )
}

fn bounded_output_text(bytes: &[u8]) -> String {
    if bytes.len() <= SMOKE_OUTPUT_CAPTURE_BYTES {
        return String::from_utf8_lossy(bytes).to_string();
    }
    let mut text = String::from_utf8_lossy(&bytes[..SMOKE_OUTPUT_CAPTURE_BYTES]).to_string();
    text.push_str("\n<truncated>");
    text
}

fn copy_provider_tree(import_dir: &Path, output_dir: &Path) -> Result<(), RustSourceProviderError> {
    let parent = output_dir
        .parent()
        .ok_or_else(|| RustSourceProviderError::Copy(format!("{} has no parent", output_dir.display())))?;
    fs::create_dir_all(parent)
        .map_err(|err| RustSourceProviderError::Copy(format!("create {}: {err}", parent.display())))?;
    fs::create_dir(output_dir)
        .map_err(|err| RustSourceProviderError::Copy(format!("create {}: {err}", output_dir.display())))?;
    let mut copied_entries = 0usize;
    if let Err(err) = copy_directory_contents(import_dir, output_dir, &mut copied_entries) {
        let _ = remove_import_output(output_dir);
        return Err(err);
    }
    Ok(())
}

fn copy_directory_contents(src: &Path, dst: &Path, copied_entries: &mut usize) -> Result<(), RustSourceProviderError> {
    for entry in
        fs::read_dir(src).map_err(|err| RustSourceProviderError::Copy(format!("read dir {}: {err}", src.display())))?
    {
        *copied_entries = copied_entries
            .checked_add(1)
            .ok_or_else(|| RustSourceProviderError::Copy("provider import entry count overflowed".to_string()))?;
        if *copied_entries > DIRECTORY_DIGEST_MAX_ENTRIES {
            return Err(RustSourceProviderError::Copy(format!(
                "provider import has more than {DIRECTORY_DIGEST_MAX_ENTRIES} entries"
            )));
        }
        let entry = entry.map_err(|err| RustSourceProviderError::Copy(format!("read dir entry: {err}")))?;
        let source_path = entry.path();
        let dest_path = dst.join(entry.file_name());
        let file_type = entry
            .file_type()
            .map_err(|err| RustSourceProviderError::Copy(format!("file type {}: {err}", source_path.display())))?;
        if file_type.is_dir() {
            fs::create_dir(&dest_path)
                .map_err(|err| RustSourceProviderError::Copy(format!("create {}: {err}", dest_path.display())))?;
            copy_directory_contents(&source_path, &dest_path, copied_entries)?;
        } else if file_type.is_file() {
            fs::copy(&source_path, &dest_path).map_err(|err| {
                RustSourceProviderError::Copy(format!(
                    "copy {} -> {}: {err}",
                    source_path.display(),
                    dest_path.display()
                ))
            })?;
        } else if file_type.is_symlink() {
            copy_symlink(&source_path, &dest_path)?;
        } else {
            return Err(RustSourceProviderError::Copy(format!("unsupported file type at {}", source_path.display())));
        }
    }
    Ok(())
}

#[cfg(unix)]
fn copy_symlink(source_path: &Path, dest_path: &Path) -> Result<(), RustSourceProviderError> {
    let target = fs::read_link(source_path)
        .map_err(|err| RustSourceProviderError::Copy(format!("readlink {}: {err}", source_path.display())))?;
    std::os::unix::fs::symlink(&target, dest_path).map_err(|err| {
        RustSourceProviderError::Copy(format!("symlink {} -> {}: {err}", dest_path.display(), target.display()))
    })
}

#[cfg(not(unix))]
fn copy_symlink(source_path: &Path, _dest_path: &Path) -> Result<(), RustSourceProviderError> {
    Err(RustSourceProviderError::Copy(format!(
        "symlink import is unsupported on this platform at {}",
        source_path.display()
    )))
}

fn remove_import_output(output_dir: &Path) -> Result<(), RustSourceProviderError> {
    if !output_dir.exists() {
        return Ok(());
    }
    fs::remove_dir_all(output_dir)
        .map_err(|err| RustSourceProviderError::Copy(format!("remove {}: {err}", output_dir.display())))
}

fn observed_provider_artifacts(
    provider_dir: &Path,
    metadata: &RustSourceProviderMetadata,
) -> Result<Vec<RustProviderObservedArtifact>, RustSourceProviderError> {
    let mut observed = Vec::with_capacity(metadata.artifacts.len());
    for artifact in &metadata.artifacts {
        let path = provider_dir.join(&artifact.path);
        if !path.exists() {
            return Err(RustSourceProviderError::MissingArtifact(format!("{} at {}", artifact.path, path.display())));
        }
        observed.push(RustProviderObservedArtifact {
            role: artifact.role,
            path: artifact.path.clone(),
            content_digest_blake3: observed_provider_artifact_digest(artifact.role, &path)?,
        });
    }
    Ok(observed)
}

fn observed_provider_receipts(
    provider_dir: &Path,
    metadata: &RustSourceProviderMetadata,
) -> Result<Vec<RustProviderObservedBuildReceipt>, RustSourceProviderError> {
    let mut observed = Vec::with_capacity(metadata.build_receipts.len());
    for receipt in &metadata.build_receipts {
        let path = provider_dir.join(&receipt.path);
        let bytes = fs::read(&path)
            .map_err(|err| RustSourceProviderError::Read(format!("provider receipt {}: {err}", path.display())))?;
        if bytes.is_empty() {
            return Err(RustSourceProviderError::Read(format!("provider receipt {} is empty", path.display())));
        }
        let parsed = serde_json::from_slice::<RustSourceProviderBuildReceipt>(&bytes)
            .map_err(|err| RustSourceProviderError::Parse(format!("provider receipt {}: {err}", path.display())))?;
        observed.push(RustProviderObservedBuildReceipt {
            path: receipt.path.clone(),
            content_digest_blake3: blake3::hash(&bytes).to_hex().to_string(),
            receipt: parsed,
        });
    }
    Ok(observed)
}

fn observed_provider_artifact_digest(role: RustProviderRole, path: &Path) -> Result<String, RustSourceProviderError> {
    if path.is_file() {
        let bytes =
            fs::read(path).map_err(|err| RustSourceProviderError::Digest(format!("read {}: {err}", path.display())))?;
        validate_observed_provider_artifact_bytes(role, path, &bytes)?;
        return Ok(blake3::hash(&bytes).to_hex().to_string());
    }
    if path.is_dir() {
        return directory_digest_blake3(path);
    }
    Err(RustSourceProviderError::MissingArtifact(format!(
        "{} is neither file nor directory",
        path.display()
    )))
}

fn validate_observed_provider_artifact_bytes(
    role: RustProviderRole,
    path: &Path,
    bytes: &[u8],
) -> Result<(), RustSourceProviderError> {
    if !matches!(role, RustProviderRole::Rustc | RustProviderRole::Cargo) {
        return Ok(());
    }
    if !looks_like_text_artifact(bytes) {
        return Ok(());
    }
    let scan_len = bytes.len().min(RUST_PROVIDER_ARTIFACT_MARKER_SCAN_BYTES);
    let text = String::from_utf8_lossy(&bytes[..scan_len]);
    if let Some(marker) = crate::source_toolchain_closure::disallowed_rust_provider_marker(&text) {
        return Err(RustSourceProviderError::Validate(format!(
            "provider artifact {} contains disallowed Rust provider marker '{marker}'",
            path.display()
        )));
    }
    Ok(())
}

fn looks_like_text_artifact(bytes: &[u8]) -> bool {
    if bytes.starts_with(b"#!") {
        return true;
    }
    let probe_len = bytes.len().min(RUST_PROVIDER_ARTIFACT_TEXT_PROBE_BYTES);
    let probe = &bytes[..probe_len];
    !probe.contains(&0) && std::str::from_utf8(probe).is_ok()
}

fn content_digest_blake3(path: &Path) -> Result<String, RustSourceProviderError> {
    if path.is_file() {
        return file_digest_blake3(path);
    }
    if path.is_dir() {
        return directory_digest_blake3(path);
    }
    Err(RustSourceProviderError::MissingArtifact(format!(
        "{} is neither file nor directory",
        path.display()
    )))
}

fn file_digest_blake3(path: &Path) -> Result<String, RustSourceProviderError> {
    let bytes =
        fs::read(path).map_err(|err| RustSourceProviderError::Digest(format!("read {}: {err}", path.display())))?;
    Ok(blake3::hash(&bytes).to_hex().to_string())
}

fn directory_digest_blake3(dir: &Path) -> Result<String, RustSourceProviderError> {
    let mut entries = Vec::new();
    collect_relative_paths(dir, dir, &mut entries)?;
    if entries.len() > DIRECTORY_DIGEST_MAX_ENTRIES {
        return Err(RustSourceProviderError::Digest(format!(
            "directory {} has more than {DIRECTORY_DIGEST_MAX_ENTRIES} entries",
            dir.display()
        )));
    }
    entries.sort();
    let mut hasher = blake3::Hasher::new();
    for relative in &entries {
        let full = dir.join(relative);
        hasher.update(relative.as_bytes());
        hasher.update(b"\0");
        if full.is_dir() {
            hasher.update(b"dir");
        } else if full.is_file() {
            hasher.update(b"file");
            let bytes = fs::read(&full)
                .map_err(|err| RustSourceProviderError::Digest(format!("read {}: {err}", full.display())))?;
            hasher.update(&bytes);
        } else if full.is_symlink() {
            hasher.update(b"symlink");
            let target = fs::read_link(&full)
                .map_err(|err| RustSourceProviderError::Digest(format!("readlink {}: {err}", full.display())))?;
            hasher.update(target.to_string_lossy().as_bytes());
        }
    }
    Ok(hasher.finalize().to_hex().to_string())
}

fn collect_relative_paths(root: &Path, current: &Path, out: &mut Vec<String>) -> Result<(), RustSourceProviderError> {
    for entry in fs::read_dir(current)
        .map_err(|err| RustSourceProviderError::Digest(format!("read dir {}: {err}", current.display())))?
    {
        let entry = entry.map_err(|err| RustSourceProviderError::Digest(format!("read dir entry: {err}")))?;
        let path = entry.path();
        let relative = path
            .strip_prefix(root)
            .map_err(|err| RustSourceProviderError::Digest(format!("strip prefix: {err}")))?
            .to_string_lossy()
            .to_string();
        out.push(relative);
        if path.is_dir() && !path.is_symlink() {
            collect_relative_paths(root, &path, out)?;
        }
    }
    Ok(())
}

#[cfg(test)]
fn write_json<T: serde::Serialize>(path: &Path, value: &T) -> Result<(), RustSourceProviderError> {
    let bytes = serde_json::to_vec_pretty(value)
        .map_err(|err| RustSourceProviderError::Parse(format!("serialize test metadata: {err}")))?;
    let parent = path
        .parent()
        .ok_or_else(|| RustSourceProviderError::Read(format!("{} has no parent", path.display())))?;
    fs::create_dir_all(parent)
        .map_err(|err| RustSourceProviderError::Read(format!("create {}: {err}", parent.display())))?;
    fs::write(path, bytes).map_err(|err| RustSourceProviderError::Read(format!("write {}: {err}", path.display())))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::source_toolchain_closure::RUST_SOURCE_PROVIDER_ID;
    use crate::source_toolchain_closure::RUST_SOURCE_PROVIDER_RECEIPT_SCHEMA;
    use crate::source_toolchain_closure::RUST_SOURCE_PROVIDER_SCHEMA;
    use crate::source_toolchain_closure::RustProviderArtifact;
    use crate::source_toolchain_closure::RustProviderBuildReceiptIdentity;
    use crate::source_toolchain_closure::RustProviderReceiptArtifact;
    use crate::source_toolchain_closure::RustProviderReceiptStep;
    use crate::source_toolchain_closure::RustProviderRole;
    use crate::source_toolchain_closure::RustProviderSourceIdentity;
    use crate::source_toolchain_closure::RustSourceProviderBuildReceipt;
    use crate::source_toolchain_closure::RustSourceProviderProvenance;
    use crate::source_toolchain_closure::ToolchainBuildReceiptKind;
    use crate::source_toolchain_closure::ToolchainSourceKind;

    const HOST_TRIPLE: &str = "x86_64-unknown-linux-gnu";
    const TARGET_TRIPLE: &str = "x86_64-unknown-linux-musl";
    const SYNTHETIC_RLIB_BYTES: &[u8] = b"synthetic rlib\n";
    const FIRST_STAGE_TEST_SOURCE_COUNT: usize = 2;
    const RUSTC_STAGE1_TEST_SOURCE_COUNT: usize = 1;
    const RUSTC_STAGE1_BOOTSTRAP_PROVIDER_SOURCE_COUNT: usize = 1;
    const RUSTC_STAGE1_NEXT_ID: &str = "rust-1.92.0-stage1";
    const SOURCE_ARCHIVE_FILE_MODE: u32 = 0o644;
    const SHA256_HEX_CHAR_COUNT: usize = 64;
    #[cfg(unix)]
    const EXECUTABLE_MODE: u32 = 0o755;

    #[test]
    fn materializer_prepares_first_stage_boundary_then_fails_closed() {
        let dir = tempfile::tempdir().unwrap();
        let recipe = dir.path().join("rust-source.ncl");
        let output = dir.path().join("out");
        let scratch = dir.path().join("scratch");
        fs::write(&recipe, "blocked recipe\n").unwrap();
        write_test_route_plan(dir.path());

        let err = materialize_rust_source_provider(&recipe, &output, &scratch, false).unwrap_err();

        let text = err.to_string();
        assert!(text.contains(RUST_SOURCE_PROVIDER_BLOCKED_REASON), "unexpected materializer error: {text}");
        assert!(text.contains("recipe_digest_blake3="));
        assert!(text.contains("route_plan_digest_blake3="));
        assert!(text.contains("first_stage_id=mrustc-to-rust-1.90.0"));
        assert!(!text.contains("source-built claim"));
        assert!(!output.exists());
        assert!(scratch.join(FIRST_STAGE_SCRIPT_FILE).is_file());
        assert!(scratch.join(FIRST_STAGE_PLAN_FILE).is_file());
        assert!(scratch.join(FIRST_STAGE_SOURCES_MANIFEST_FILE).is_file());
        assert!(scratch.join(FIRST_STAGE_BUILD_MANIFEST_FILE).is_file());
        assert!(scratch.join(FIRST_STAGE_BUILD_LOG_FILE).is_file());
        assert!(scratch.join(FIRST_STAGE_PROVIDER_CANDIDATE_MANIFEST_FILE).is_file());
        assert!(scratch.join(FIRST_STAGE_PROVIDER_CANDIDATE_DIR).is_dir());
        assert!(scratch.join(FIRST_STAGE_PROVIDER_CANDIDATE_DIR).join(RUST_SOURCE_PROVIDER_METADATA_PATH).is_file());
        assert!(
            scratch
                .join(FIRST_STAGE_PROVIDER_CANDIDATE_DIR)
                .join(FIRST_STAGE_PROVIDER_RECEIPT_RELATIVE_PATH)
                .is_file()
        );
        assert!(scratch.join(FIRST_STAGE_PROVIDER_CANDIDATE_SMOKE_DIR).join(SMOKE_EVIDENCE_SUMMARY_FILE).is_file());
        assert!(scratch.join(FIRST_STAGE_PROVIDER_CANDIDATE_SMOKE_DIR).join(SMOKE_EVIDENCE_OUTPUT_FILE).is_file());
        assert!(scratch.join(RUSTC_STAGE1_PLAN_FILE).is_file());
        assert!(scratch.join(RUSTC_STAGE1_SOURCES_MANIFEST_FILE).is_file());
        assert!(scratch.join(RUSTC_STAGE1_BUILD_MANIFEST_FILE).is_file());
        assert!(scratch.join(RUSTC_STAGE1_BUILD_LOG_FILE).is_file());
        assert!(scratch.join(RUSTC_STAGE1_SCRIPT_FILE).is_file());
        assert!(scratch.join(RUSTC_STAGE1_ARCHIVE_DIR).is_dir());
        assert!(scratch.join(RUSTC_STAGE1_SOURCE_DIR).is_dir());
        assert!(scratch.join(RUSTC_STAGE1_BUILD_DIR).is_dir());
        assert!(scratch.join(RUSTC_STAGE1_OUTPUT_DIR).join(PROVIDER_RUSTC_RELATIVE_PATH).is_file());
        assert!(scratch.join(RUSTC_STAGE1_OUTPUT_DIR).join(PROVIDER_CARGO_RELATIVE_PATH).is_file());
        assert!(scratch.join(RUSTC_STAGE1_PROVIDER_CANDIDATE_MANIFEST_FILE).is_file());
        assert!(scratch.join(RUSTC_STAGE1_PROVIDER_CANDIDATE_DIR).is_dir());
        assert!(scratch.join(RUSTC_STAGE1_PROVIDER_CANDIDATE_DIR).join(RUST_SOURCE_PROVIDER_METADATA_PATH).is_file());
        assert!(
            scratch
                .join(RUSTC_STAGE1_PROVIDER_CANDIDATE_DIR)
                .join(RUSTC_STAGE1_PROVIDER_RECEIPT_RELATIVE_PATH)
                .is_file()
        );
        assert!(scratch.join(RUSTC_STAGE1_PROVIDER_CANDIDATE_SMOKE_DIR).join(SMOKE_EVIDENCE_SUMMARY_FILE).is_file());
        assert!(scratch.join(RUSTC_STAGE1_PROVIDER_CANDIDATE_SMOKE_DIR).join(SMOKE_EVIDENCE_OUTPUT_FILE).is_file());
        assert!(scratch.join(RUSTC_STAGE1_SOURCE_DIR).join("rust-1.91.1/README.txt").is_file());
        assert!(scratch.join(FIRST_STAGE_ARCHIVE_DIR).is_dir());
        assert!(scratch.join(FIRST_STAGE_SOURCE_DIR).is_dir());
        assert!(scratch.join(FIRST_STAGE_BUILD_DIR).is_dir());
        assert!(scratch.join(FIRST_STAGE_SOURCE_DIR).join("mrustc-0.12.0/README.txt").is_file());
        assert!(scratch.join(FIRST_STAGE_SOURCE_DIR).join("mrustc-0.12.0/bin/mrustc").is_file());
        assert!(scratch.join(FIRST_STAGE_SOURCE_DIR).join("mrustc-0.12.0/bin/minicargo").is_file());
        assert!(scratch.join(FIRST_STAGE_SOURCE_DIR).join("mrustc-0.12.0/output/rustc").is_file());
        assert!(scratch.join(FIRST_STAGE_SOURCE_DIR).join("mrustc-0.12.0/output/cargo").is_file());
        assert!(
            scratch
                .join(FIRST_STAGE_SOURCE_DIR)
                .join("mrustc-0.12.0/run_rustc/output/prefix/bin/rustc")
                .is_file()
        );
        assert!(scratch.join(FIRST_STAGE_SOURCE_DIR).join("rust-1.90.0/README.txt").is_file());
        #[cfg(unix)]
        assert_eq!(script_mode(&scratch.join(FIRST_STAGE_SCRIPT_FILE)), EXECUTABLE_MODE);
        let manifest: serde_json::Value =
            serde_json::from_slice(&fs::read(scratch.join(FIRST_STAGE_PLAN_FILE)).unwrap()).unwrap();
        assert_eq!(manifest["schema"], FIRST_STAGE_PLAN_SCHEMA);
        assert_eq!(manifest["stage"]["id"], "mrustc-to-rust-1.90.0");
        assert_eq!(
            manifest["stage"]["expected_outputs"].as_array().unwrap().len(),
            REQUIRED_FIRST_STAGE_OUTPUT_ROLES.len()
        );
        let sources_manifest: serde_json::Value =
            serde_json::from_slice(&fs::read(scratch.join(FIRST_STAGE_SOURCES_MANIFEST_FILE)).unwrap()).unwrap();
        assert_eq!(sources_manifest["schema"], FIRST_STAGE_SOURCES_MANIFEST_SCHEMA);
        assert_eq!(sources_manifest["source_count"], FIRST_STAGE_TEST_SOURCE_COUNT);
        let build_manifest: serde_json::Value =
            serde_json::from_slice(&fs::read(scratch.join(FIRST_STAGE_BUILD_MANIFEST_FILE)).unwrap()).unwrap();
        assert_eq!(build_manifest["schema"], FIRST_STAGE_BUILD_MANIFEST_SCHEMA);
        assert_eq!(build_manifest["build"]["mrustc_digest_blake3"].as_str().unwrap().len(), SHA256_HEX_CHAR_COUNT);
        assert_eq!(
            build_manifest["build"]["translated_rustc_digest_blake3"].as_str().unwrap().len(),
            SHA256_HEX_CHAR_COUNT
        );
        assert_eq!(
            build_manifest["build"]["prefix_rustlib_digest_blake3"].as_str().unwrap().len(),
            SHA256_HEX_CHAR_COUNT
        );
        let candidate_manifest: serde_json::Value =
            serde_json::from_slice(&fs::read(scratch.join(FIRST_STAGE_PROVIDER_CANDIDATE_MANIFEST_FILE)).unwrap())
                .unwrap();
        assert_eq!(candidate_manifest["schema"], FIRST_STAGE_PROVIDER_CANDIDATE_SCHEMA);
        assert_eq!(candidate_manifest["candidate_only"], true);
        let expected_candidate_artifacts =
            REQUIRED_FIRST_STAGE_OUTPUT_ROLES.len() + FIRST_STAGE_PROVIDER_RECEIPT_ARTIFACT_COUNT;
        assert_eq!(candidate_manifest["candidate"]["artifact_count"], expected_candidate_artifacts);
        assert_eq!(candidate_manifest["candidate"]["source_count"], FIRST_STAGE_TEST_SOURCE_COUNT);
        let candidate_validation =
            validate_materialized_rust_source_provider(&scratch.join(FIRST_STAGE_PROVIDER_CANDIDATE_DIR)).unwrap();
        assert_eq!(candidate_validation.validation.artifact_count, expected_candidate_artifacts);
        assert_eq!(candidate_validation.validation.receipt_count, 1);
        assert_eq!(candidate_validation.validation.source_count, FIRST_STAGE_TEST_SOURCE_COUNT);
        let smoke_summary: serde_json::Value = serde_json::from_slice(
            &fs::read(scratch.join(FIRST_STAGE_PROVIDER_CANDIDATE_SMOKE_DIR).join(SMOKE_EVIDENCE_SUMMARY_FILE))
                .unwrap(),
        )
        .unwrap();
        assert_eq!(smoke_summary["schema"], SMOKE_EVIDENCE_SCHEMA);
        assert_eq!(smoke_summary["output_digest_blake3"], blake3::hash(SYNTHETIC_RLIB_BYTES).to_hex().to_string());
        let rustc_stage1_plan: serde_json::Value =
            serde_json::from_slice(&fs::read(scratch.join(RUSTC_STAGE1_PLAN_FILE)).unwrap()).unwrap();
        assert_eq!(rustc_stage1_plan["schema"], RUSTC_STAGE1_PLAN_SCHEMA);
        assert_eq!(rustc_stage1_plan["stage"]["id"], "rust-1.91.1-stage1");
        assert_eq!(rustc_stage1_plan["stage"]["bootstrap_stage_id"], "mrustc-to-rust-1.90.0");
        assert_eq!(
            rustc_stage1_plan["bootstrap_provider_candidate"]["metadata_digest_blake3"],
            candidate_validation.metadata_digest_blake3
        );
        let rustc_stage1_sources: serde_json::Value =
            serde_json::from_slice(&fs::read(scratch.join(RUSTC_STAGE1_SOURCES_MANIFEST_FILE)).unwrap()).unwrap();
        assert_eq!(rustc_stage1_sources["schema"], RUSTC_STAGE1_SOURCES_MANIFEST_SCHEMA);
        assert_eq!(rustc_stage1_sources["source_count"], RUSTC_STAGE1_TEST_SOURCE_COUNT);
        assert_eq!(rustc_stage1_sources["sources"][0]["id"], "rust-1.91.1");
        let rustc_stage1_build: serde_json::Value =
            serde_json::from_slice(&fs::read(scratch.join(RUSTC_STAGE1_BUILD_MANIFEST_FILE)).unwrap()).unwrap();
        assert_eq!(rustc_stage1_build["schema"], RUSTC_STAGE1_BUILD_MANIFEST_SCHEMA);
        assert_eq!(rustc_stage1_build["build"]["stage_id"], "rust-1.91.1-stage1");
        assert_eq!(rustc_stage1_build["build"]["rustc_digest_blake3"].as_str().unwrap().len(), SHA256_HEX_CHAR_COUNT);
        assert_eq!(
            rustc_stage1_build["build"]["target_rustlib_digest_blake3"].as_str().unwrap().len(),
            SHA256_HEX_CHAR_COUNT
        );
        let rustc_stage1_candidate_manifest: serde_json::Value =
            serde_json::from_slice(&fs::read(scratch.join(RUSTC_STAGE1_PROVIDER_CANDIDATE_MANIFEST_FILE)).unwrap())
                .unwrap();
        assert_eq!(rustc_stage1_candidate_manifest["schema"], RUSTC_STAGE1_PROVIDER_CANDIDATE_SCHEMA);
        assert_eq!(rustc_stage1_candidate_manifest["candidate_only"], true);
        assert_eq!(rustc_stage1_candidate_manifest["final_output_written"], false);
        assert_eq!(rustc_stage1_candidate_manifest["candidate"]["stage_id"], "rust-1.91.1-stage1");
        let expected_rustc_stage1_candidate_artifacts =
            REQUIRED_RUSTC_STAGE1_OUTPUT_ROLES.len() + RUSTC_STAGE1_PROVIDER_RECEIPT_ARTIFACT_COUNT;
        let expected_rustc_stage1_candidate_sources =
            RUSTC_STAGE1_TEST_SOURCE_COUNT + RUSTC_STAGE1_BOOTSTRAP_PROVIDER_SOURCE_COUNT;
        assert_eq!(
            rustc_stage1_candidate_manifest["candidate"]["artifact_count"],
            expected_rustc_stage1_candidate_artifacts
        );
        assert_eq!(
            rustc_stage1_candidate_manifest["candidate"]["source_count"],
            expected_rustc_stage1_candidate_sources
        );
        let rustc_stage1_candidate_validation =
            validate_materialized_rust_source_provider(&scratch.join(RUSTC_STAGE1_PROVIDER_CANDIDATE_DIR)).unwrap();
        assert_eq!(
            rustc_stage1_candidate_validation.validation.artifact_count,
            expected_rustc_stage1_candidate_artifacts
        );
        assert_eq!(rustc_stage1_candidate_validation.validation.receipt_count, 1);
        assert_eq!(rustc_stage1_candidate_validation.validation.source_count, expected_rustc_stage1_candidate_sources);
        assert!(
            rustc_stage1_candidate_validation
                .metadata
                .sources
                .iter()
                .any(|source| source.id == "rust-1.91.1-stage1-bootstrap-provider")
        );
        let rustc_stage1_smoke_summary: serde_json::Value = serde_json::from_slice(
            &fs::read(scratch.join(RUSTC_STAGE1_PROVIDER_CANDIDATE_SMOKE_DIR).join(SMOKE_EVIDENCE_SUMMARY_FILE))
                .unwrap(),
        )
        .unwrap();
        assert_eq!(rustc_stage1_smoke_summary["schema"], SMOKE_EVIDENCE_SCHEMA);
        assert_eq!(
            rustc_stage1_smoke_summary["output_digest_blake3"],
            blake3::hash(SYNTHETIC_RLIB_BYTES).to_hex().to_string()
        );
        let rustc_stage1_next_root = scratch.join(RUSTC_STAGE1_CHAIN_DIR).join(RUSTC_STAGE1_NEXT_ID);
        assert!(rustc_stage1_next_root.join(RUSTC_STAGE1_PLAN_FILE).is_file());
        assert!(rustc_stage1_next_root.join(RUSTC_STAGE1_SOURCES_MANIFEST_FILE).is_file());
        assert!(rustc_stage1_next_root.join(RUSTC_STAGE1_BUILD_MANIFEST_FILE).is_file());
        assert!(rustc_stage1_next_root.join(RUSTC_STAGE1_BUILD_LOG_FILE).is_file());
        assert!(rustc_stage1_next_root.join(RUSTC_STAGE1_SCRIPT_FILE).is_file());
        assert!(rustc_stage1_next_root.join(RUSTC_STAGE1_SOURCE_DIR).join("rust-1.92.0/README.txt").is_file());
        assert!(rustc_stage1_next_root.join(RUSTC_STAGE1_OUTPUT_DIR).join(PROVIDER_RUSTC_RELATIVE_PATH).is_file());
        assert!(rustc_stage1_next_root.join(RUSTC_STAGE1_OUTPUT_DIR).join(PROVIDER_CARGO_RELATIVE_PATH).is_file());
        assert!(rustc_stage1_next_root.join(RUSTC_STAGE1_PROVIDER_CANDIDATE_MANIFEST_FILE).is_file());
        assert!(
            rustc_stage1_next_root
                .join(RUSTC_STAGE1_PROVIDER_CANDIDATE_DIR)
                .join(RUST_SOURCE_PROVIDER_METADATA_PATH)
                .is_file()
        );
        assert!(
            rustc_stage1_next_root
                .join(RUSTC_STAGE1_PROVIDER_CANDIDATE_DIR)
                .join(RUSTC_STAGE1_PROVIDER_RECEIPT_RELATIVE_PATH)
                .is_file()
        );
        assert!(
            rustc_stage1_next_root
                .join(RUSTC_STAGE1_PROVIDER_CANDIDATE_SMOKE_DIR)
                .join(SMOKE_EVIDENCE_SUMMARY_FILE)
                .is_file()
        );
        let rustc_stage1_next_plan: serde_json::Value =
            serde_json::from_slice(&fs::read(rustc_stage1_next_root.join(RUSTC_STAGE1_PLAN_FILE)).unwrap()).unwrap();
        assert_eq!(rustc_stage1_next_plan["schema"], RUSTC_STAGE1_PLAN_SCHEMA);
        assert_eq!(rustc_stage1_next_plan["stage"]["id"], RUSTC_STAGE1_NEXT_ID);
        assert_eq!(rustc_stage1_next_plan["stage"]["bootstrap_stage_id"], "rust-1.91.1-stage1");
        assert_eq!(
            rustc_stage1_next_plan["bootstrap_provider_candidate"]["metadata_digest_blake3"],
            rustc_stage1_candidate_validation.metadata_digest_blake3
        );
        let rustc_stage1_next_sources: serde_json::Value =
            serde_json::from_slice(&fs::read(rustc_stage1_next_root.join(RUSTC_STAGE1_SOURCES_MANIFEST_FILE)).unwrap())
                .unwrap();
        assert_eq!(rustc_stage1_next_sources["schema"], RUSTC_STAGE1_SOURCES_MANIFEST_SCHEMA);
        assert_eq!(rustc_stage1_next_sources["source_count"], RUSTC_STAGE1_TEST_SOURCE_COUNT);
        assert_eq!(rustc_stage1_next_sources["sources"][0]["id"], "rust-1.92.0");
        let rustc_stage1_next_build: serde_json::Value =
            serde_json::from_slice(&fs::read(rustc_stage1_next_root.join(RUSTC_STAGE1_BUILD_MANIFEST_FILE)).unwrap())
                .unwrap();
        assert_eq!(rustc_stage1_next_build["schema"], RUSTC_STAGE1_BUILD_MANIFEST_SCHEMA);
        assert_eq!(rustc_stage1_next_build["build"]["stage_id"], RUSTC_STAGE1_NEXT_ID);
        assert_eq!(
            rustc_stage1_next_build["build"]["rustc_digest_blake3"].as_str().unwrap().len(),
            SHA256_HEX_CHAR_COUNT
        );
        let rustc_stage1_next_candidate_manifest: serde_json::Value = serde_json::from_slice(
            &fs::read(rustc_stage1_next_root.join(RUSTC_STAGE1_PROVIDER_CANDIDATE_MANIFEST_FILE)).unwrap(),
        )
        .unwrap();
        assert_eq!(rustc_stage1_next_candidate_manifest["schema"], RUSTC_STAGE1_PROVIDER_CANDIDATE_SCHEMA);
        assert_eq!(rustc_stage1_next_candidate_manifest["candidate_only"], true);
        assert_eq!(rustc_stage1_next_candidate_manifest["final_output_written"], false);
        assert_eq!(
            rustc_stage1_next_candidate_manifest["candidate"]["artifact_count"],
            expected_rustc_stage1_candidate_artifacts
        );
        assert_eq!(
            rustc_stage1_next_candidate_manifest["candidate"]["source_count"],
            expected_rustc_stage1_candidate_sources
        );
        let rustc_stage1_next_candidate_validation = validate_materialized_rust_source_provider(
            &rustc_stage1_next_root.join(RUSTC_STAGE1_PROVIDER_CANDIDATE_DIR),
        )
        .unwrap();
        assert_eq!(
            rustc_stage1_next_candidate_validation.validation.artifact_count,
            expected_rustc_stage1_candidate_artifacts
        );
        assert_eq!(rustc_stage1_next_candidate_validation.validation.receipt_count, 1);
        assert_eq!(
            rustc_stage1_next_candidate_validation.validation.source_count,
            expected_rustc_stage1_candidate_sources
        );
        assert!(
            rustc_stage1_next_candidate_validation
                .metadata
                .sources
                .iter()
                .any(|source| source.id == "rust-1.92.0-stage1-bootstrap-provider")
        );
        let rustc_stage1_next_smoke_summary: serde_json::Value = serde_json::from_slice(
            &fs::read(
                rustc_stage1_next_root
                    .join(RUSTC_STAGE1_PROVIDER_CANDIDATE_SMOKE_DIR)
                    .join(SMOKE_EVIDENCE_SUMMARY_FILE),
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(rustc_stage1_next_smoke_summary["schema"], SMOKE_EVIDENCE_SCHEMA);
        assert_eq!(
            rustc_stage1_next_smoke_summary["output_digest_blake3"],
            blake3::hash(SYNTHETIC_RLIB_BYTES).to_hex().to_string()
        );
        let rustc_stage1_next_log =
            fs::read_to_string(rustc_stage1_next_root.join(RUSTC_STAGE1_BUILD_LOG_FILE)).unwrap();
        assert!(rustc_stage1_next_log.contains(RUSTC_STAGE1_NEXT_ID));
        assert!(rustc_stage1_next_log.contains("rustc stage1 products ready"));
        let rustc_stage1_log = fs::read_to_string(scratch.join(RUSTC_STAGE1_BUILD_LOG_FILE)).unwrap();
        assert!(rustc_stage1_log.contains("mantle rustc stage1"));
        assert!(rustc_stage1_log.contains("rustc stage1 products ready"));
        let script = fs::read_to_string(scratch.join(FIRST_STAGE_SCRIPT_FILE)).unwrap();
        assert!(script.contains("missing verified source"));
        assert!(script.contains("verified sources manifest"));
        assert!(script.contains("mrustc-0.12.0"));
        assert!(script.contains(FIRST_STAGE_MINICARGO_BINARY));
    }

    #[test]
    fn first_stage_provider_candidate_rejects_tampered_artifact() {
        let dir = tempfile::tempdir().unwrap();
        let recipe = dir.path().join("rust-source.ncl");
        let output = dir.path().join("out");
        let scratch = dir.path().join("scratch");
        fs::write(&recipe, "blocked recipe\n").unwrap();
        write_test_route_plan(dir.path());
        let _ = materialize_rust_source_provider(&recipe, &output, &scratch, false).unwrap_err();
        let candidate = scratch.join(FIRST_STAGE_PROVIDER_CANDIDATE_DIR);

        fs::write(candidate.join(PROVIDER_RUSTC_RELATIVE_PATH), b"changed-rustc").unwrap();
        let err = validate_materialized_rust_source_provider(&candidate).unwrap_err();

        assert!(err.to_string().contains("digest mismatch"));
        assert!(!output.exists());
    }

    #[test]
    fn rustc_stage1_provider_candidate_rejects_tampered_artifact() {
        let dir = tempfile::tempdir().unwrap();
        let recipe = dir.path().join("rust-source.ncl");
        let output = dir.path().join("out");
        let scratch = dir.path().join("scratch");
        fs::write(&recipe, "blocked recipe\n").unwrap();
        write_test_route_plan(dir.path());
        let _ = materialize_rust_source_provider(&recipe, &output, &scratch, false).unwrap_err();
        let candidate = scratch.join(RUSTC_STAGE1_PROVIDER_CANDIDATE_DIR);

        fs::write(candidate.join(PROVIDER_CARGO_RELATIVE_PATH), b"changed-cargo").unwrap();
        let err = validate_materialized_rust_source_provider(&candidate).unwrap_err();

        assert!(err.to_string().contains("digest mismatch"));
        assert!(!output.exists());
    }

    #[test]
    fn chained_rustc_stage1_provider_candidate_rejects_tampered_artifact() {
        let dir = tempfile::tempdir().unwrap();
        let recipe = dir.path().join("rust-source.ncl");
        let output = dir.path().join("out");
        let scratch = dir.path().join("scratch");
        fs::write(&recipe, "blocked recipe\n").unwrap();
        write_test_route_plan(dir.path());
        let _ = materialize_rust_source_provider(&recipe, &output, &scratch, false).unwrap_err();
        let candidate = scratch
            .join(RUSTC_STAGE1_CHAIN_DIR)
            .join(RUSTC_STAGE1_NEXT_ID)
            .join(RUSTC_STAGE1_PROVIDER_CANDIDATE_DIR);

        fs::write(candidate.join(PROVIDER_RUSTC_RELATIVE_PATH), b"changed-chained-rustc").unwrap();
        let err = validate_materialized_rust_source_provider(&candidate).unwrap_err();

        assert!(err.to_string().contains("digest mismatch"));
        assert!(!output.exists());
    }

    #[test]
    fn materializer_rejects_missing_route_plan_without_output() {
        let dir = tempfile::tempdir().unwrap();
        let recipe = dir.path().join("rust-source.ncl");
        let output = dir.path().join("out");
        let scratch = dir.path().join("scratch");
        fs::write(&recipe, "blocked recipe\n").unwrap();

        let err = materialize_rust_source_provider(&recipe, &output, &scratch, false).unwrap_err();

        assert!(err.to_string().contains(RUST_SOURCE_PROVIDER_PLAN_FILE));
        assert!(!output.exists());
        assert!(!scratch.join(FIRST_STAGE_SCRIPT_FILE).exists());
    }

    #[test]
    fn materializer_rejects_first_stage_source_digest_mismatch_without_output() {
        let dir = tempfile::tempdir().unwrap();
        let recipe = dir.path().join("rust-source.ncl");
        let output = dir.path().join("out");
        let scratch = dir.path().join("scratch");
        fs::write(&recipe, "blocked recipe\n").unwrap();
        write_test_route_plan_with_mrustc_sha(dir.path(), &sample_sha256_hex('d'));

        let err = materialize_rust_source_provider(&recipe, &output, &scratch, false).unwrap_err();

        let message = err.to_string();
        assert!(message.contains("digest mismatch"));
        assert!(message.contains("mrustc-0.12.0"));
        assert!(!output.exists());
        assert!(!scratch.join(FIRST_STAGE_SOURCES_MANIFEST_FILE).exists());
    }

    #[test]
    fn materializer_rejects_rustc_stage1_source_digest_mismatch_without_output() {
        let dir = tempfile::tempdir().unwrap();
        let recipe = dir.path().join("rust-source.ncl");
        let output = dir.path().join("out");
        let scratch = dir.path().join("scratch");
        fs::write(&recipe, "blocked recipe\n").unwrap();
        write_test_route_plan_with_rust191_sha(dir.path(), &sample_sha256_hex('e'));

        let err = materialize_rust_source_provider(&recipe, &output, &scratch, false).unwrap_err();

        let message = err.to_string();
        assert!(message.contains("digest mismatch"));
        assert!(message.contains("rust-1.91.1"));
        assert!(!output.exists());
        assert!(scratch.join(FIRST_STAGE_PROVIDER_CANDIDATE_MANIFEST_FILE).is_file());
        assert!(scratch.join(RUSTC_STAGE1_PLAN_FILE).is_file());
        assert!(!scratch.join(RUSTC_STAGE1_SOURCES_MANIFEST_FILE).exists());
        assert!(!scratch.join(RUSTC_STAGE1_PROVIDER_CANDIDATE_MANIFEST_FILE).exists());
    }

    #[test]
    fn materializer_rejects_chained_rustc_stage1_source_digest_mismatch_without_output() {
        let dir = tempfile::tempdir().unwrap();
        let recipe = dir.path().join("rust-source.ncl");
        let output = dir.path().join("out");
        let scratch = dir.path().join("scratch");
        fs::write(&recipe, "blocked recipe\n").unwrap();
        write_test_route_plan_with_rust192_sha(dir.path(), &sample_sha256_hex('f'));

        let err = materialize_rust_source_provider(&recipe, &output, &scratch, false).unwrap_err();

        let message = err.to_string();
        assert!(message.contains("digest mismatch"));
        assert!(message.contains("rust-1.92.0"));
        assert!(!output.exists());
        assert!(scratch.join(RUSTC_STAGE1_PROVIDER_CANDIDATE_MANIFEST_FILE).is_file());
        let rustc_stage1_next_root = scratch.join(RUSTC_STAGE1_CHAIN_DIR).join(RUSTC_STAGE1_NEXT_ID);
        assert!(rustc_stage1_next_root.join(RUSTC_STAGE1_PLAN_FILE).is_file());
        assert!(!rustc_stage1_next_root.join(RUSTC_STAGE1_SOURCES_MANIFEST_FILE).exists());
        assert!(!rustc_stage1_next_root.join(RUSTC_STAGE1_PROVIDER_CANDIDATE_MANIFEST_FILE).exists());
    }

    #[cfg(unix)]
    #[test]
    fn first_stage_script_fails_before_provider_output_when_sources_missing() {
        let dir = tempfile::tempdir().unwrap();
        let recipe = dir.path().join("rust-source.ncl");
        let output = dir.path().join("out");
        let scratch = dir.path().join("scratch");
        fs::write(&recipe, "blocked recipe\n").unwrap();
        write_test_route_plan(dir.path());
        let _ = materialize_rust_source_provider(&recipe, &output, &scratch, false).unwrap_err();
        fs::remove_dir_all(scratch.join(FIRST_STAGE_SOURCE_DIR).join("mrustc-0.12.0")).unwrap();

        let status = Command::new(scratch.join(FIRST_STAGE_SCRIPT_FILE)).status().unwrap();

        assert_eq!(status.code(), Some(FIRST_STAGE_MISSING_SOURCE_EXIT_CODE));
        assert!(!output.exists());
    }

    #[test]
    fn directory_validator_accepts_complete_fake_provider() {
        let dir = tempfile::tempdir().unwrap();
        let metadata = write_fake_provider(dir.path());

        let validation = validate_materialized_rust_source_provider(dir.path()).unwrap();

        assert_eq!(validation.metadata_path, dir.path().join(RUST_SOURCE_PROVIDER_METADATA_PATH));
        assert_eq!(validation.validation.artifact_count, metadata.artifacts.len());
        assert_eq!(validation.validation.source_count, metadata.sources.len());
        assert_eq!(validation.validation.receipt_count, metadata.build_receipts.len());
        assert!(!validation.metadata_digest_blake3.is_empty());
    }

    #[test]
    fn directory_validator_rejects_artifact_digest_mismatch() {
        let dir = tempfile::tempdir().unwrap();
        write_fake_provider(dir.path());
        fs::write(dir.path().join("bin/rustc"), b"changed").unwrap();

        let err = validate_materialized_rust_source_provider(dir.path()).unwrap_err();

        assert!(err.to_string().contains("digest mismatch"));
    }

    #[test]
    fn directory_validator_rejects_nix_rustc_wrapper_even_with_matching_digest() {
        let dir = tempfile::tempdir().unwrap();
        let wrapper = b"#!/bin/sh\nexec /nix/store/example-rust-default/bin/rustc \"$@\"\n";
        write_fake_provider(dir.path());
        write_bytes(dir.path(), "bin/rustc", wrapper);
        refresh_fake_receipt_and_metadata(dir.path());

        let err = validate_materialized_rust_source_provider(dir.path()).unwrap_err();
        let message = err.to_string();

        assert!(message.contains("disallowed Rust provider marker"));
        assert!(message.contains("'nix'"));
    }

    #[test]
    fn import_provider_copies_validated_provider() {
        let dir = tempfile::tempdir().unwrap();
        let import_dir = dir.path().join("import");
        let output_dir = dir.path().join("output");
        fs::create_dir(&import_dir).unwrap();
        let metadata = write_fake_provider(&import_dir);

        let imported = import_rust_source_provider(&import_dir, &output_dir).unwrap();

        assert_eq!(imported.input_path, import_dir);
        assert_eq!(imported.output_path, output_dir);
        assert_eq!(imported.validation.validation.artifact_count, metadata.artifacts.len());
        assert!(imported.output_path.join("bin/rustc").is_file());
        assert!(imported.output_path.join(RUST_SOURCE_PROVIDER_METADATA_PATH).is_file());
    }

    #[test]
    fn import_provider_rejects_prebuilt_metadata_without_output() {
        let dir = tempfile::tempdir().unwrap();
        let import_dir = dir.path().join("import");
        let output_dir = dir.path().join("output");
        fs::create_dir(&import_dir).unwrap();
        let mut metadata = write_fake_provider(&import_dir);
        metadata.provenance.uses_prebuilt_rust = true;
        write_json(&import_dir.join(RUST_SOURCE_PROVIDER_METADATA_PATH), &metadata).unwrap();

        let err = import_rust_source_provider(&import_dir, &output_dir).unwrap_err();

        assert!(err.to_string().contains("uses prebuilt Rust"));
        assert!(!output_dir.exists());
    }

    #[test]
    fn import_provider_rejects_existing_output_without_overwrite() {
        let dir = tempfile::tempdir().unwrap();
        let import_dir = dir.path().join("import");
        let output_dir = dir.path().join("output");
        fs::create_dir(&import_dir).unwrap();
        fs::create_dir(&output_dir).unwrap();
        write_fake_provider(&import_dir);

        let err = import_rust_source_provider(&import_dir, &output_dir).unwrap_err();

        assert!(err.to_string().contains("already exists"));
        assert!(output_dir.exists());
    }

    #[test]
    fn import_provider_rejects_malformed_receipt_without_output() {
        let dir = tempfile::tempdir().unwrap();
        let import_dir = dir.path().join("import");
        let output_dir = dir.path().join("output");
        fs::create_dir(&import_dir).unwrap();
        write_fake_provider(&import_dir);
        fs::write(import_dir.join("share/mantle-rust-provider/receipts/build.json"), b"not-json").unwrap();

        let err = import_rust_source_provider(&import_dir, &output_dir).unwrap_err();

        assert!(err.to_string().contains("provider receipt"));
        assert!(!output_dir.exists());
    }

    #[cfg(unix)]
    #[test]
    fn smoke_provider_runs_synthetic_rustc_after_validation() {
        let dir = tempfile::tempdir().unwrap();
        let provider_dir = dir.path().join("provider");
        let scratch_dir = dir.path().join("scratch");
        fs::create_dir(&provider_dir).unwrap();
        write_synthetic_smoke_provider(&provider_dir, None);

        let smoke = smoke_rust_source_provider(&provider_dir, &scratch_dir).unwrap();

        assert_eq!(smoke.provider_path, provider_dir);
        assert_eq!(smoke.scratch_dir, scratch_dir);
        assert_eq!(smoke.target_triple, HOST_TRIPLE);
        assert!(smoke.rustc_path.ends_with("bin/rustc"));
        assert!(smoke.source_path.ends_with(SMOKE_SOURCE_FILE));
        assert!(smoke.output_path.ends_with(SMOKE_OUTPUT_FILE));
        assert_eq!(smoke.output_digest_blake3, blake3::hash(SYNTHETIC_RLIB_BYTES).to_hex().to_string());
        assert!(smoke.stdout.contains("synthetic rustc smoke"));
        assert!(smoke.stderr.is_empty());
    }

    #[cfg(unix)]
    #[test]
    fn smoke_evidence_persists_summary_output_and_metadata() {
        let dir = tempfile::tempdir().unwrap();
        let provider_dir = dir.path().join("provider");
        let scratch_dir = dir.path().join("scratch");
        let evidence_dir = dir.path().join("evidence");
        fs::create_dir(&provider_dir).unwrap();
        write_synthetic_smoke_provider(&provider_dir, None);
        let validation = validate_materialized_rust_source_provider(&provider_dir).unwrap();
        let smoke = smoke_rust_source_provider(&provider_dir, &scratch_dir).unwrap();

        let evidence = persist_rust_source_provider_smoke_evidence(&evidence_dir, &smoke, &validation).unwrap();
        let summary: serde_json::Value = serde_json::from_slice(&fs::read(&evidence.summary_path).unwrap()).unwrap();

        assert_eq!(evidence.evidence_dir, evidence_dir);
        assert_eq!(evidence.output_digest_blake3, smoke.output_digest_blake3);
        assert_eq!(evidence.metadata_digest_blake3, validation.metadata_digest_blake3);
        assert_eq!(summary["schema"], SMOKE_EVIDENCE_SCHEMA);
        assert_eq!(summary["output_digest_blake3"], smoke.output_digest_blake3);
        assert_eq!(fs::read_to_string(&evidence.stdout_path).unwrap(), smoke.stdout);
        assert_eq!(fs::read_to_string(&evidence.stderr_path).unwrap(), smoke.stderr);
        assert_eq!(fs::read(&evidence.output_path).unwrap(), SYNTHETIC_RLIB_BYTES);
        assert_eq!(content_digest_blake3(&evidence.metadata_path).unwrap(), validation.metadata_digest_blake3);
    }

    #[cfg(unix)]
    #[test]
    fn smoke_evidence_rejects_tampered_smoke_output() {
        let dir = tempfile::tempdir().unwrap();
        let provider_dir = dir.path().join("provider");
        let scratch_dir = dir.path().join("scratch");
        let evidence_dir = dir.path().join("evidence");
        fs::create_dir(&provider_dir).unwrap();
        write_synthetic_smoke_provider(&provider_dir, None);
        let validation = validate_materialized_rust_source_provider(&provider_dir).unwrap();
        let smoke = smoke_rust_source_provider(&provider_dir, &scratch_dir).unwrap();
        fs::write(&smoke.output_path, b"tampered").unwrap();

        let err = persist_rust_source_provider_smoke_evidence(&evidence_dir, &smoke, &validation).unwrap_err();

        assert!(err.to_string().contains("copied smoke output digest mismatch"));
        assert!(!evidence_dir.join(SMOKE_EVIDENCE_SUMMARY_FILE).exists());
    }

    #[cfg(unix)]
    #[test]
    fn smoke_provider_rejects_invalid_metadata_before_launch() {
        let dir = tempfile::tempdir().unwrap();
        let provider_dir = dir.path().join("provider");
        let scratch_dir = dir.path().join("scratch");
        let sentinel_path = dir.path().join("launched");
        fs::create_dir(&provider_dir).unwrap();
        let mut metadata = write_synthetic_smoke_provider(&provider_dir, Some(&sentinel_path));
        metadata.provenance.uses_prebuilt_rust = true;
        write_json(&provider_dir.join(RUST_SOURCE_PROVIDER_METADATA_PATH), &metadata).unwrap();

        let err = smoke_rust_source_provider(&provider_dir, &scratch_dir).unwrap_err();

        assert!(err.to_string().contains("uses prebuilt Rust"));
        assert!(!sentinel_path.exists());
        assert!(!scratch_dir.join(SMOKE_OUTPUT_FILE).exists());
    }

    fn write_test_route_plan(root: &Path) {
        let sources = write_test_source_archives(root);
        write_test_route_plan_with_sources(root, &sources, &sources.mrustc_sha256_hex);
    }

    fn write_test_route_plan_with_mrustc_sha(root: &Path, mrustc_sha256_hex: &str) {
        let sources = write_test_source_archives(root);
        write_test_route_plan_with_source_shas(
            root,
            &sources,
            mrustc_sha256_hex,
            &sources.rust191_sha256_hex,
            &sources.rust192_sha256_hex,
        );
    }

    fn write_test_route_plan_with_rust191_sha(root: &Path, rust191_sha256_hex: &str) {
        let sources = write_test_source_archives(root);
        write_test_route_plan_with_source_shas(
            root,
            &sources,
            &sources.mrustc_sha256_hex,
            rust191_sha256_hex,
            &sources.rust192_sha256_hex,
        );
    }

    fn write_test_route_plan_with_rust192_sha(root: &Path, rust192_sha256_hex: &str) {
        let sources = write_test_source_archives(root);
        write_test_route_plan_with_source_shas(
            root,
            &sources,
            &sources.mrustc_sha256_hex,
            &sources.rust191_sha256_hex,
            rust192_sha256_hex,
        );
    }

    fn write_test_route_plan_with_sources(root: &Path, sources: &TestSourceArchives, mrustc_sha256_hex: &str) {
        write_test_route_plan_with_source_shas(
            root,
            sources,
            mrustc_sha256_hex,
            &sources.rust191_sha256_hex,
            &sources.rust192_sha256_hex,
        )
    }

    fn write_test_route_plan_with_source_shas(
        root: &Path,
        sources: &TestSourceArchives,
        mrustc_sha256_hex: &str,
        rust191_sha256_hex: &str,
        rust192_sha256_hex: &str,
    ) {
        let text = r#"
let Source = {
  id | String,
  kind | String,
  name | String,
  version | String,
  url | String,
  sha256_hex | String,
} in
let Stage = {
  id | String,
  kind | String,
  source_ids | Array String,
  bootstrap_stage_id | String,
  rust_version | String,
  outputs | Array String,
  notes | Array String,
} in
let Output = {
  role | String,
  path | String,
} in
let Policy = {
  source_built | Bool,
  uses_prebuilt_rust | Bool,
  forbids_prebuilt_rust | Bool,
  reference | String,
} in
let Plan = {
  schema | String,
  provider_id | String,
  route | String,
  host_triple | String,
  target_triple | String,
  final_version | String,
  policy | Policy,
  sources | Array Source,
  stages | Array Stage,
  final_outputs | Array Output,
} in
{
  schema = "mantle-rust-source-provider-bootstrap-plan-v1",
  provider_id = "mantle-rust-source-provider",
  route = "mrustc-source-route",
  host_triple = "x86_64-unknown-linux-musl",
  target_triple = "x86_64-unknown-linux-musl",
  final_version = "1.94.0",
  policy = {
    source_built = true,
    uses_prebuilt_rust = false,
    forbids_prebuilt_rust = true,
    reference = "stagex route",
  },
  sources = [
    {
      id = "mrustc-0.12.0",
      kind = "tarball",
      name = "mrustc-source",
      version = "0.12.0",
      url = "__MRUSTC_URL__",
      sha256_hex = "__MRUSTC_SHA256__",
    },
    {
      id = "rust-1.90.0",
      kind = "tarball",
      name = "rust-compiler-source",
      version = "1.90.0",
      url = "__RUST190_URL__",
      sha256_hex = "__RUST190_SHA256__",
    },
    {
      id = "rust-1.91.1",
      kind = "tarball",
      name = "rust-compiler-source",
      version = "1.91.1",
      url = "__RUST191_URL__",
      sha256_hex = "__RUST191_SHA256__",
    },
    {
      id = "rust-1.92.0",
      kind = "tarball",
      name = "rust-compiler-source",
      version = "1.92.0",
      url = "__RUST192_URL__",
      sha256_hex = "__RUST192_SHA256__",
    },
    {
      id = "rust-1.94.0",
      kind = "tarball",
      name = "rust-compiler-source",
      version = "1.94.0",
      url = "__RUST194_URL__",
      sha256_hex = "__RUST194_SHA256__",
    },
  ],
  stages = [
    {
      id = "mrustc-to-rust-1.90.0",
      kind = "mrustc-seed",
      source_ids = ["mrustc-0.12.0", "rust-1.90.0"],
      bootstrap_stage_id = "",
      rust_version = "1.90.0",
      outputs = ["rustc", "cargo", "host-rustlib", "target-rustlib"],
      notes = ["compile source stage"],
    },
    {
      id = "rust-1.91.1-stage1",
      kind = "rustc-stage1",
      source_ids = ["rust-1.91.1"],
      bootstrap_stage_id = "mrustc-to-rust-1.90.0",
      rust_version = "1.91.1",
      outputs = ["rustc", "cargo", "host-rustlib", "target-rustlib"],
      notes = ["advance one Rust source release after mrustc"],
    },
    {
      id = "rust-1.92.0-stage1",
      kind = "rustc-stage1",
      source_ids = ["rust-1.92.0"],
      bootstrap_stage_id = "rust-1.91.1-stage1",
      rust_version = "1.92.0",
      outputs = ["rustc", "cargo", "host-rustlib", "target-rustlib"],
      notes = ["advance one chained Rust source release"],
    },
    {
      id = "rust-1.94.0-final",
      kind = "rustc-final",
      source_ids = ["rust-1.94.0"],
      bootstrap_stage_id = "rust-1.92.0-stage1",
      rust_version = "1.94.0",
      outputs = ["rustc", "cargo", "rustdoc", "host-rustlib", "target-rustlib", "provider-receipt"],
      notes = ["install provider metadata and receipts"],
    },
  ],
  final_outputs = [
    { role = "rustc", path = "bin/rustc" },
    { role = "cargo", path = "bin/cargo" },
    { role = "rustdoc", path = "bin/rustdoc" },
    { role = "host-rustlib", path = "lib/rustlib/x86_64-unknown-linux-musl/lib" },
    { role = "target-rustlib", path = "lib/rustlib/x86_64-unknown-linux-musl/lib" },
    { role = "provider-receipt", path = "share/mantle-rust-provider/receipts/build.json" },
  ],
} | Plan
"#;
        let text = text
            .replace("__MRUSTC_URL__", &sources.mrustc_url)
            .replace("__MRUSTC_SHA256__", mrustc_sha256_hex)
            .replace("__RUST190_URL__", &sources.rust190_url)
            .replace("__RUST190_SHA256__", &sources.rust190_sha256_hex)
            .replace("__RUST191_URL__", &sources.rust191_url)
            .replace("__RUST191_SHA256__", rust191_sha256_hex)
            .replace("__RUST192_URL__", &sources.rust192_url)
            .replace("__RUST192_SHA256__", rust192_sha256_hex)
            .replace("__RUST194_URL__", &sources.rust194_url)
            .replace("__RUST194_SHA256__", &sources.rust194_sha256_hex);
        fs::write(root.join(RUST_SOURCE_PROVIDER_PLAN_FILE), text).unwrap();
    }

    struct TestSourceArchives {
        mrustc_url: String,
        mrustc_sha256_hex: String,
        rust190_url: String,
        rust190_sha256_hex: String,
        rust191_url: String,
        rust191_sha256_hex: String,
        rust192_url: String,
        rust192_sha256_hex: String,
        rust194_url: String,
        rust194_sha256_hex: String,
    }

    fn write_test_source_archives(root: &Path) -> TestSourceArchives {
        let archive_dir = root.join("test-source-archives");
        fs::create_dir_all(&archive_dir).unwrap();
        let mrustc_path = archive_dir.join("mrustc-0.12.0.tar.gz");
        let rust190_path = archive_dir.join("rust-1.90.0.tar.gz");
        let rust191_path = archive_dir.join("rust-1.91.1.tar.gz");
        let rust192_path = archive_dir.join("rust-1.92.0.tar.gz");
        let rust194_path = archive_dir.join("rust-1.94.0.tar.gz");
        write_test_source_archive(&mrustc_path, "mrustc-0.12.0", b"mrustc seed source\n");
        write_test_source_archive(&rust190_path, "rust-1.90.0", b"rust 1.90 source\n");
        write_test_source_archive(&rust191_path, "rust-1.91.1", b"rust 1.91.1 source\n");
        write_test_source_archive(&rust192_path, "rust-1.92.0", b"rust 1.92.0 source\n");
        write_test_source_archive(&rust194_path, "rust-1.94.0", b"rust 1.94 source\n");
        TestSourceArchives {
            mrustc_url: url::Url::from_file_path(&mrustc_path).unwrap().to_string(),
            mrustc_sha256_hex: sha256_file_hex(&mrustc_path),
            rust190_url: url::Url::from_file_path(&rust190_path).unwrap().to_string(),
            rust190_sha256_hex: sha256_file_hex(&rust190_path),
            rust191_url: url::Url::from_file_path(&rust191_path).unwrap().to_string(),
            rust191_sha256_hex: sha256_file_hex(&rust191_path),
            rust192_url: url::Url::from_file_path(&rust192_path).unwrap().to_string(),
            rust192_sha256_hex: sha256_file_hex(&rust192_path),
            rust194_url: url::Url::from_file_path(&rust194_path).unwrap().to_string(),
            rust194_sha256_hex: sha256_file_hex(&rust194_path),
        }
    }

    fn write_test_source_archive(path: &Path, top_dir: &str, readme: &[u8]) {
        let file = File::create(path).unwrap();
        let encoder = flate2::write::GzEncoder::new(file, flate2::Compression::default());
        let mut builder = tar::Builder::new(encoder);
        append_test_tar_file(&mut builder, &format!("{top_dir}/README.txt"), readme);
        if top_dir.starts_with(FIRST_STAGE_MRUSTC_SOURCE_PREFIX) {
            append_test_tar_file(&mut builder, &format!("{top_dir}/{FIRST_STAGE_MAKEFILE}"), test_mrustc_makefile());
            append_test_tar_file(
                &mut builder,
                &format!("{top_dir}/{FIRST_STAGE_MINICARGO_MAKEFILE}"),
                test_mrustc_minicargo_makefile(),
            );
            append_test_tar_file(
                &mut builder,
                &format!("{top_dir}/{FIRST_STAGE_RUN_RUSTC_DIR}/{FIRST_STAGE_MAKEFILE}"),
                test_mrustc_run_rustc_makefile(),
            );
        }
        if matches!(top_dir, "rust-1.91.1" | "rust-1.92.0") {
            append_test_tar_file(
                &mut builder,
                &format!("{top_dir}/{RUSTC_STAGE1_SOURCE_BUILD_SCRIPT}"),
                test_rustc_stage1_build_script(),
            );
        }
        builder.finish().unwrap();
        let encoder = builder.into_inner().unwrap();
        encoder.finish().unwrap();
    }

    fn append_test_tar_file(builder: &mut tar::Builder<flate2::write::GzEncoder<File>>, path: &str, bytes: &[u8]) {
        let mut header = tar::Header::new_gnu();
        header.set_size(u64::try_from(bytes.len()).unwrap());
        header.set_mode(SOURCE_ARCHIVE_FILE_MODE);
        header.set_cksum();
        builder.append_data(&mut header, path, bytes).unwrap();
    }

    fn test_mrustc_makefile() -> &'static [u8] {
        b"all:\n\tmkdir -p bin\n\tprintf 'synthetic mrustc\\n' > bin/mrustc\n\tchmod +x bin/mrustc\n"
    }

    fn test_mrustc_minicargo_makefile() -> &'static [u8] {
        b"bin/minicargo:\n\tmkdir -p bin\n\tprintf 'synthetic minicargo\\n' > bin/minicargo\n\tchmod +x bin/minicargo\noutput/rustc:\n\tmkdir -p output\n\tprintf 'synthetic translated rustc\\n' > output/rustc\n\tchmod +x output/rustc\noutput/cargo:\n\tmkdir -p output\n\tprintf 'synthetic translated cargo\\n' > output/cargo\n\tchmod +x output/cargo\n"
    }

    fn test_mrustc_run_rustc_makefile() -> &'static [u8] {
        b"all:\n\tmkdir -p output/prefix/bin output/prefix/lib/rustlib/x86_64-unknown-linux-musl/lib\n\tprintf '%s\\n' '#!/bin/sh' 'out=' 'prev=' 'for arg in \"$$@\"; do' '  if [ \"$$prev\" = \"-o\" ]; then out=\"$$arg\"; prev=\"\"; continue; fi' '  if [ \"$$arg\" = \"-o\" ]; then prev=\"-o\"; continue; fi' 'done' 'if [ -z \"$$out\" ]; then echo missing-output >&2; exit 2; fi' 'printf \"synthetic rlib\\\\n\" > \"$$out\"' 'echo synthetic rustc smoke' > output/prefix/bin/rustc\n\tprintf 'synthetic prefix cargo\\n' > output/prefix/bin/cargo\n\tprintf 'synthetic prefix std\\n' > output/prefix/lib/rustlib/x86_64-unknown-linux-musl/lib/libstd.rlib\n\tchmod +x output/prefix/bin/rustc output/prefix/bin/cargo\n"
    }

    fn test_rustc_stage1_build_script() -> &'static [u8] {
        b"set -eu\ntest -f \"$MANTLE_BOOTSTRAP_PROVIDER/bin/rustc\"\ntest -f \"$MANTLE_BOOTSTRAP_PROVIDER/bin/cargo\"\nmkdir -p \"$MANTLE_STAGE_OUTPUT/bin\" \"$MANTLE_STAGE_OUTPUT/lib/rustlib/$MANTLE_HOST_TRIPLE/lib\" \"$MANTLE_STAGE_OUTPUT/lib/rustlib/$MANTLE_TARGET_TRIPLE/lib\"\nprintf '%s\\n' '#!/bin/sh' 'out=' 'prev=' 'for arg in \"$@\"; do' '  if [ \"$prev\" = \"-o\" ]; then out=\"$arg\"; prev=\"\"; continue; fi' '  if [ \"$arg\" = \"-o\" ]; then prev=\"-o\"; continue; fi' 'done' 'if [ -z \"$out\" ]; then echo missing-output >&2; exit 2; fi' 'printf \"synthetic rlib\\\\n\" > \"$out\"' 'echo synthetic stage1 rustc smoke' > \"$MANTLE_STAGE_OUTPUT/bin/rustc\"\nprintf 'synthetic stage1 cargo for %s\\n' \"$MANTLE_RUST_VERSION\" > \"$MANTLE_STAGE_OUTPUT/bin/cargo\"\nprintf 'synthetic stage1 host std\\n' > \"$MANTLE_STAGE_OUTPUT/lib/rustlib/$MANTLE_HOST_TRIPLE/lib/libstd.rlib\"\nprintf 'synthetic stage1 target std\\n' > \"$MANTLE_STAGE_OUTPUT/lib/rustlib/$MANTLE_TARGET_TRIPLE/lib/libstd.rlib\"\nchmod +x \"$MANTLE_STAGE_OUTPUT/bin/rustc\" \"$MANTLE_STAGE_OUTPUT/bin/cargo\"\n"
    }

    fn sha256_file_hex(path: &Path) -> String {
        sha256_hex(&fs::read(path).unwrap())
    }

    fn sample_sha256_hex(ch: char) -> String {
        assert!(ch.is_ascii_hexdigit(), "sample digest char must be hex");
        assert!(!ch.is_ascii_uppercase(), "sample digest char must be lowercase");
        std::iter::repeat_n(ch, SHA256_HEX_CHAR_COUNT).collect()
    }

    #[cfg(unix)]
    fn script_mode(path: &Path) -> u32 {
        use std::os::unix::fs::PermissionsExt;

        fs::metadata(path).unwrap().permissions().mode() & EXECUTABLE_MODE
    }

    fn write_fake_provider(root: &Path) -> RustSourceProviderMetadata {
        write_bytes(root, "bin/rustc", b"rustc");
        write_bytes(root, "bin/cargo", b"cargo");
        write_bytes(root, "lib/rustlib/x86_64-unknown-linux-gnu/lib/libstd.rlib", b"host-std");
        write_bytes(root, "lib/rustlib/x86_64-unknown-linux-musl/lib/libstd.rlib", b"target-std");
        refresh_fake_receipt_and_metadata(root)
    }

    #[cfg(unix)]
    fn write_synthetic_smoke_provider(root: &Path, sentinel_path: Option<&Path>) -> RustSourceProviderMetadata {
        write_bytes(root, "bin/rustc", synthetic_rustc_script(sentinel_path).as_bytes());
        make_executable(&root.join("bin/rustc"));
        write_bytes(root, "bin/cargo", b"cargo");
        write_bytes(root, "lib/rustlib/x86_64-unknown-linux-gnu/lib/libstd.rlib", b"host-std");
        write_bytes(root, "lib/rustlib/x86_64-unknown-linux-musl/lib/libstd.rlib", b"target-std");
        refresh_fake_receipt_and_metadata(root)
    }

    fn refresh_fake_receipt_and_metadata(root: &Path) -> RustSourceProviderMetadata {
        let receipt = fake_receipt(root);
        write_json(&root.join("share/mantle-rust-provider/receipts/build.json"), &receipt).unwrap();
        let metadata = fake_metadata(root);
        write_json(&root.join(RUST_SOURCE_PROVIDER_METADATA_PATH), &metadata).unwrap();
        metadata
    }

    #[cfg(unix)]
    fn synthetic_rustc_script(sentinel_path: Option<&Path>) -> String {
        let sentinel_write =
            sentinel_path.map(|path| format!("printf launched > '{}'\n", path.display())).unwrap_or_default();
        format!(
            "#!/bin/sh\n{sentinel_write}out=''\nprev=''\nfor arg in \"$@\"; do\n  if [ \"$prev\" = '-o' ]; then out=\"$arg\"; prev=''; continue; fi\n  if [ \"$arg\" = '-o' ]; then prev='-o'; continue; fi\ndone\nif [ -z \"$out\" ]; then echo missing-output >&2; exit 2; fi\nprintf 'synthetic rlib\\n' > \"$out\"\necho synthetic rustc smoke\n"
        )
    }

    #[cfg(unix)]
    fn make_executable(path: &Path) {
        use std::os::unix::fs::PermissionsExt;

        let mut permissions = fs::metadata(path).unwrap().permissions();
        permissions.set_mode(EXECUTABLE_MODE);
        fs::set_permissions(path, permissions).unwrap();
    }

    fn fake_receipt(root: &Path) -> RustSourceProviderBuildReceipt {
        RustSourceProviderBuildReceipt {
            schema: RUST_SOURCE_PROVIDER_RECEIPT_SCHEMA.to_string(),
            receipt_id: "build-receipt".to_string(),
            provider_id: RUST_SOURCE_PROVIDER_ID.to_string(),
            host_triple: HOST_TRIPLE.to_string(),
            target_triple: TARGET_TRIPLE.to_string(),
            source_ids: vec!["rust-src".to_string()],
            output_artifacts: vec![
                receipt_artifact(root, RustProviderRole::Rustc, "rustc", "bin/rustc"),
                receipt_artifact(root, RustProviderRole::Cargo, "cargo", "bin/cargo"),
                receipt_artifact(
                    root,
                    RustProviderRole::HostRustlib,
                    "host-rustlib",
                    "lib/rustlib/x86_64-unknown-linux-gnu/lib",
                ),
                receipt_artifact(
                    root,
                    RustProviderRole::TargetRustlib,
                    "target-rustlib",
                    "lib/rustlib/x86_64-unknown-linux-musl/lib",
                ),
            ],
            build_steps: vec![RustProviderReceiptStep {
                name: "compile-rust-from-source".to_string(),
                program: "mantle-rust-source-stage".to_string(),
                arguments: vec!["bootstrap/rust-source.ncl".to_string()],
            }],
        }
    }

    fn fake_metadata(root: &Path) -> RustSourceProviderMetadata {
        RustSourceProviderMetadata {
            schema: RUST_SOURCE_PROVIDER_SCHEMA.to_string(),
            provider_id: RUST_SOURCE_PROVIDER_ID.to_string(),
            host_triple: HOST_TRIPLE.to_string(),
            target_triple: TARGET_TRIPLE.to_string(),
            provenance: RustSourceProviderProvenance {
                source_built: true,
                uses_prebuilt_rust: false,
                build_recipe: "bootstrap/rust-source.ncl".to_string(),
            },
            sources: vec![RustProviderSourceIdentity {
                id: "rust-src".to_string(),
                kind: ToolchainSourceKind::Tarball,
                name: "rust-compiler-source".to_string(),
                digest_blake3: blake3::hash(b"source").to_hex().to_string(),
            }],
            build_receipts: vec![RustProviderBuildReceiptIdentity {
                id: "build-receipt".to_string(),
                kind: ToolchainBuildReceiptKind::MantleDerivation,
                name: "rust-source-build-receipt".to_string(),
                path: "share/mantle-rust-provider/receipts/build.json".to_string(),
                digest_blake3: digest_path(root, "share/mantle-rust-provider/receipts/build.json"),
            }],
            artifacts: vec![
                artifact(root, RustProviderRole::Rustc, "rustc", "bin/rustc"),
                artifact(root, RustProviderRole::Cargo, "cargo", "bin/cargo"),
                artifact(
                    root,
                    RustProviderRole::HostRustlib,
                    "host-rustlib",
                    "lib/rustlib/x86_64-unknown-linux-gnu/lib",
                ),
                artifact(
                    root,
                    RustProviderRole::TargetRustlib,
                    "target-rustlib",
                    "lib/rustlib/x86_64-unknown-linux-musl/lib",
                ),
                artifact(
                    root,
                    RustProviderRole::ProviderReceipt,
                    "build-receipt",
                    "share/mantle-rust-provider/receipts/build.json",
                ),
            ],
        }
    }

    fn artifact(root: &Path, role: RustProviderRole, name: &str, path: &str) -> RustProviderArtifact {
        RustProviderArtifact {
            role,
            name: name.to_string(),
            path: path.to_string(),
            content_digest_blake3: digest_path(root, path),
            source_id: "rust-src".to_string(),
            build_receipt_id: "build-receipt".to_string(),
        }
    }

    fn receipt_artifact(root: &Path, role: RustProviderRole, name: &str, path: &str) -> RustProviderReceiptArtifact {
        RustProviderReceiptArtifact {
            role,
            name: name.to_string(),
            path: path.to_string(),
            content_digest_blake3: digest_path(root, path),
        }
    }

    fn write_bytes(root: &Path, relative: &str, bytes: &[u8]) {
        let path = root.join(relative);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, bytes).unwrap();
    }

    fn digest_path(root: &Path, relative: &str) -> String {
        content_digest_blake3(&root.join(relative)).unwrap()
    }
}
