//! Shell/orchestration for the source-built Rust compiler/sysroot provider.
//!
//! This module performs filesystem work around the pure provider metadata
//! validator in `source_toolchain_closure`. It deliberately fails closed for
//! materialization until Mantle has a real Rust-from-source bootstrap path; the
//! validation entry points are ready for a future materializer output and refuse
//! prebuilt/wrapper metadata in the pure core.

use std::ffi::OsString;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

use crate::source_toolchain_closure::RUST_SOURCE_PROVIDER_METADATA_PATH;
use crate::source_toolchain_closure::RustProviderObservedArtifact;
use crate::source_toolchain_closure::RustProviderObservedBuildReceipt;
use crate::source_toolchain_closure::RustProviderRole;
use crate::source_toolchain_closure::RustSourceProviderBootstrapOutput;
use crate::source_toolchain_closure::RustSourceProviderBootstrapPlan;
use crate::source_toolchain_closure::RustSourceProviderBootstrapPlanValidation;
use crate::source_toolchain_closure::RustSourceProviderBootstrapSource;
use crate::source_toolchain_closure::RustSourceProviderBootstrapStage;
use crate::source_toolchain_closure::RustSourceProviderBootstrapStageKind;
use crate::source_toolchain_closure::RustSourceProviderBuildReceipt;
use crate::source_toolchain_closure::RustSourceProviderMetadata;
use crate::source_toolchain_closure::RustSourceProviderValidation;

pub(crate) const RUST_SOURCE_PROVIDER_BLOCKED_REASON: &str = "source-built Rust provider materialization is not implemented: Mantle has no receipt-bound Rust-from-source bootstrap that can build rustc/cargo/rustlib without prebuilt Rust";

const DIRECTORY_DIGEST_MAX_ENTRIES: usize = 200_000;
const RUST_SOURCE_PROVIDER_PLAN_FILE: &str = "rust-source-plan.ncl";
const FIRST_STAGE_PLAN_SCHEMA: &str = "mantle-rust-source-provider-first-stage-plan-v1";
const FIRST_STAGE_PLAN_FILE: &str = "mrustc-first-stage-plan.json";
const FIRST_STAGE_SCRIPT_FILE: &str = "run-mrustc-first-stage.sh";
const FIRST_STAGE_SOURCE_DIR: &str = "sources";
const FIRST_STAGE_BUILD_DIR: &str = "build";
const FIRST_STAGE_MISSING_SOURCE_EXIT_CODE: i32 = 2;
const FIRST_STAGE_NOT_IMPLEMENTED_EXIT_CODE: i32 = 1;
const REQUIRED_FIRST_STAGE_OUTPUT_ROLES: [RustProviderRole; 4] = [
    RustProviderRole::Rustc,
    RustProviderRole::Cargo,
    RustProviderRole::HostRustlib,
    RustProviderRole::TargetRustlib,
];
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
    source_ids: Vec<String>,
    sources: Vec<RustSourceProviderBootstrapSource>,
    expected_outputs: Vec<RustSourceProviderBootstrapOutput>,
    source_dir: PathBuf,
    build_dir: PathBuf,
    output_dir: PathBuf,
    plan_path: PathBuf,
    script_path: PathBuf,
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
    if verbose {
        eprintln!("Rust source provider recipe: {}", plan.recipe_path.display());
        eprintln!("  recipe_digest_blake3: {}", plan.recipe_digest_blake3);
        eprintln!("  route_plan: {}", boundary.route_plan_path.display());
        eprintln!("  route_plan_digest_blake3: {}", boundary.route_plan_digest_blake3);
        eprintln!("  route_policy_digest_blake3: {}", boundary.route_policy_digest_blake3);
        eprintln!("  first_stage_id: {}", boundary.stage_id);
        eprintln!("  first_stage_script: {}", boundary.script_path.display());
        eprintln!("  first_stage_plan: {}", boundary.plan_path.display());
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
        source_ids: stage.source_ids.clone(),
        sources,
        expected_outputs,
        source_dir: materialization.scratch_dir.join(FIRST_STAGE_SOURCE_DIR),
        build_dir: materialization.scratch_dir.join(FIRST_STAGE_BUILD_DIR),
        output_dir: materialization.output_dir.clone(),
        plan_path: materialization.scratch_dir.join(FIRST_STAGE_PLAN_FILE),
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

fn sources_for_stage(
    plan: &RustSourceProviderBootstrapPlan,
    stage: &RustSourceProviderBootstrapStage,
) -> Result<Vec<RustSourceProviderBootstrapSource>, RustSourceProviderError> {
    let mut sources = Vec::with_capacity(stage.source_ids.len());
    for source_id in &stage.source_ids {
        let Some(source) = plan.sources.iter().find(|source| &source.id == source_id) else {
            return Err(RustSourceProviderError::Validate(format!(
                "first mrustc stage '{}' references unknown source_id '{source_id}'",
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
    fs::create_dir_all(&boundary.source_dir)
        .map_err(|err| RustSourceProviderError::Read(format!("create {}: {err}", boundary.source_dir.display())))?;
    fs::create_dir_all(&boundary.build_dir)
        .map_err(|err| RustSourceProviderError::Read(format!("create {}: {err}", boundary.build_dir.display())))?;
    let plan_bytes = first_stage_boundary_manifest(boundary)?;
    fs::write(&boundary.plan_path, plan_bytes).map_err(|err| {
        RustSourceProviderError::Read(format!("write first stage plan {}: {err}", boundary.plan_path.display()))
    })?;
    let script = first_stage_boundary_script(boundary);
    fs::write(&boundary.script_path, script).map_err(|err| {
        RustSourceProviderError::Read(format!("write first stage script {}: {err}", boundary.script_path.display()))
    })?;
    make_executable(&boundary.script_path)
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
            "source_ids": &boundary.source_ids,
            "expected_outputs": &boundary.expected_outputs,
        },
        "sources": &boundary.sources,
        "paths": {
            "source_dir": boundary.source_dir.display().to_string(),
            "build_dir": boundary.build_dir.display().to_string(),
            "output_dir": boundary.output_dir.display().to_string(),
            "script": boundary.script_path.display().to_string(),
        },
        "blocked_reason": RUST_SOURCE_PROVIDER_BLOCKED_REASON,
    });
    serde_json::to_vec_pretty(&manifest)
        .map_err(|err| RustSourceProviderError::Parse(format!("serialize first stage plan: {err}")))
}

fn first_stage_boundary_script(boundary: &RustSourceProviderFirstStageBoundary) -> String {
    let mut script = String::new();
    script.push_str("#!/bin/sh\n");
    script.push_str("set -eu\n");
    script.push_str(&format!("STAGE_ID={}\n", shell_quote(&boundary.stage_id)));
    script.push_str(&format!("SOURCE_DIR={}\n", shell_quote(&boundary.source_dir.display().to_string())));
    script.push_str(&format!("BUILD_DIR={}\n", shell_quote(&boundary.build_dir.display().to_string())));
    script.push_str(&format!("OUTPUT_DIR={}\n", shell_quote(&boundary.output_dir.display().to_string())));
    script.push_str("printf '%s\\n' \"mantle rust source first stage: $STAGE_ID\"\n");
    for source in &boundary.sources {
        script.push_str(&format!(
            "if [ ! -e \"$SOURCE_DIR\"/{id} ]; then printf '%s\\n' {message} >&2; exit {missing}; fi\n",
            id = shell_quote(&source.id),
            message = shell_quote(&format!(
                "missing source '{}' from {} with sha256 {}",
                source.id, source.url, source.sha256_hex
            )),
            missing = FIRST_STAGE_MISSING_SOURCE_EXIT_CODE,
        ));
    }
    script.push_str(
        "printf '%s\\n' \"planned build dir: $BUILD_DIR\"\nprintf '%s\\n' \"planned output dir: $OUTPUT_DIR\"\n",
    );
    script.push_str(&format!(
        "printf '%s\\n' {} >&2\nexit {}\n",
        shell_quote(RUST_SOURCE_PROVIDER_BLOCKED_REASON),
        FIRST_STAGE_NOT_IMPLEMENTED_EXIT_CODE
    ));
    script
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
        assert!(text.contains(RUST_SOURCE_PROVIDER_BLOCKED_REASON));
        assert!(text.contains("recipe_digest_blake3="));
        assert!(text.contains("route_plan_digest_blake3="));
        assert!(text.contains("first_stage_id=mrustc-to-rust-1.90.0"));
        assert!(!text.contains("source-built claim"));
        assert!(!output.exists());
        assert!(scratch.join(FIRST_STAGE_SCRIPT_FILE).is_file());
        assert!(scratch.join(FIRST_STAGE_PLAN_FILE).is_file());
        assert!(scratch.join(FIRST_STAGE_SOURCE_DIR).is_dir());
        assert!(scratch.join(FIRST_STAGE_BUILD_DIR).is_dir());
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
        let script = fs::read_to_string(scratch.join(FIRST_STAGE_SCRIPT_FILE)).unwrap();
        assert!(script.contains("missing source"));
        assert!(script.contains("mrustc-0.12.0"));
        assert!(script.contains(RUST_SOURCE_PROVIDER_BLOCKED_REASON));
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
      url = "https://example.invalid/mrustc-0.12.0.tar.gz",
      sha256_hex = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
    },
    {
      id = "rust-1.90.0",
      kind = "tarball",
      name = "rust-compiler-source",
      version = "1.90.0",
      url = "https://example.invalid/rustc-1.90.0-src.tar.gz",
      sha256_hex = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
    },
    {
      id = "rust-1.94.0",
      kind = "tarball",
      name = "rust-compiler-source",
      version = "1.94.0",
      url = "https://example.invalid/rustc-1.94.0-src.tar.gz",
      sha256_hex = "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc",
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
      id = "rust-1.94.0-final",
      kind = "rustc-final",
      source_ids = ["rust-1.94.0"],
      bootstrap_stage_id = "mrustc-to-rust-1.90.0",
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
        fs::write(root.join(RUST_SOURCE_PROVIDER_PLAN_FILE), text).unwrap();
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
