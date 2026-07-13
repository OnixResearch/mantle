use std::collections::BTreeMap;
use std::path::Path;

use crunch_wasm_component_core::Blake3Identity;
use crunch_wasm_component_core::GeneratedInputPlan;
use crunch_wasm_component_core::LockFacts;
use crunch_wasm_component_core::LockValidationRequest;
use crunch_wasm_component_core::SourceAcquisitionPlan;
use crunch_wasm_component_core::ToolCohort;
use crunch_wasm_component_core::ToolIdentity;
use crunch_wasm_component_core::WkgLock;
use crunch_wasm_component_core::finalize_generated_inputs;
use crunch_wasm_component_core::plan_source_acquisition;
use crunch_wasm_component_core::registry_config_identity;
use crunch_wasm_component_core::validate_manifest;

use crate::ComponentPipelineRequest;
use crate::Error;
use crate::PIPELINE_REQUEST_SCHEMA;
use crate::VerifiedToolchain;
use crate::files::package_file_measurement;
use crate::files::read_source_file_bounded;
use crate::files::validate_relative_path;
use crate::verify_toolchain_manifest;

const DECLARED_COMPONENT_TOOL_COUNT: usize = 9;
const MAX_WKG_LOCK_BYTES: u64 = 1024 * 1024;

pub(crate) struct PreparedPipeline {
    pub request: ComponentPipelineRequest,
    pub request_blake3: Blake3Identity,
    pub toolchain: VerifiedToolchain,
    pub generated_plan: GeneratedInputPlan,
    pub source_plan: SourceAcquisitionPlan,
    pub lock_bytes: Vec<u8>,
}

pub(crate) fn prepare_pipeline(request: ComponentPipelineRequest) -> Result<PreparedPipeline, Error> {
    validate_request_shape(&request)?;
    let request_bytes = serde_json::to_vec(&request)
        .map_err(|error| Error::Invalid(format!("serializing component request identity input: {error}")))?;
    let request_blake3 = Blake3Identity::from_slice(&request_bytes);
    let toolchain = verify_toolchain_manifest(Path::new(&request.toolchain_manifest))?;
    validate_declared_cohort(&request.manifest.cohort, &toolchain)?;
    let manifest_validation = validate_manifest(request.manifest.clone());
    if !manifest_validation.blockers.is_empty() {
        return Err(core_blockers("manifest", &manifest_validation.blockers));
    }
    let generated = finalize_generated_inputs(request.generated_inputs.clone());
    let generated_plan = generated.plan.ok_or_else(|| core_blockers("generated-inputs", &generated.blockers))?;
    let lock_path = Path::new(&request.source_root).join(&request.manifest.package_resolution.lock_path);
    let lock_bytes = read_source_file_bounded(&lock_path, MAX_WKG_LOCK_BYTES, "checked wkg.lock")?;
    let lock = parse_lock(&lock_path, &lock_bytes)?;
    validate_materialization_files(&request)?;
    let registry_config_blake3 = registry_config_identity(request.manifest.package_resolution.clone())
        .map_err(|error| Error::Invalid(format!("identifying wkg registry configuration: {error}")))?;
    let facts = LockFacts {
        lock,
        lock_bytes_blake3: Blake3Identity::from_slice(&lock_bytes),
        registry_config_blake3,
        materializations: request.package_materializations.clone(),
    };
    let source_result = plan_source_acquisition(LockValidationRequest {
        manifest: request.manifest.clone(),
        facts,
        lock_bytes: lock_bytes.clone(),
    });
    let source_plan = source_result.plan.ok_or_else(|| core_blockers("wkg.lock", &source_result.blockers))?;
    debug_assert!(!source_plan.requests.is_empty());
    debug_assert!(!generated_plan.inputs.is_empty());
    Ok(PreparedPipeline {
        request,
        request_blake3,
        toolchain,
        generated_plan,
        source_plan,
        lock_bytes,
    })
}

fn validate_request_shape(request: &ComponentPipelineRequest) -> Result<(), Error> {
    if request.schema != PIPELINE_REQUEST_SCHEMA {
        return Err(Error::Invalid(format!("unsupported pipeline request schema `{}`", request.schema)));
    }
    if request.package_resolution_network {
        return Err(Error::Blocked(
            "OCI package resolution is not admitted by the current local-only production shell".to_string(),
        ));
    }
    let source_root = Path::new(&request.source_root);
    let toolchain_manifest = Path::new(&request.toolchain_manifest);
    if !source_root.is_absolute() || !source_root.is_dir() || !toolchain_manifest.is_absolute() {
        return Err(Error::Invalid("source root and toolchain manifest must be existing absolute inputs".to_string()));
    }
    for relative in [
        request.wit_relative_path.as_str(),
        request.cargo_component_relative_path.as_str(),
        request.manifest.package_resolution.lock_path.as_str(),
    ] {
        validate_relative_path(Path::new(relative))?;
    }
    if request.expected_runtime_stdout.len() > 1_048_576 {
        return Err(Error::Invalid("expected runtime stdout exceeds one MiB".to_string()));
    }
    debug_assert!(source_root.is_absolute());
    debug_assert!(!request.wit_relative_path.is_empty());
    Ok(())
}

fn parse_lock(path: &Path, bytes: &[u8]) -> Result<WkgLock, Error> {
    let text = std::str::from_utf8(bytes)
        .map_err(|error| Error::Invalid(format!("wkg.lock {} is not UTF-8: {error}", path.display())))?;
    let lock = toml::from_str(text)
        .map_err(|error| Error::Invalid(format!("parsing checked wkg.lock {}: {error}", path.display())))?;
    debug_assert!(!bytes.is_empty());
    debug_assert!(!text.is_empty());
    Ok(lock)
}

fn validate_materialization_files(request: &ComponentPipelineRequest) -> Result<(), Error> {
    for materialization in &request.package_materializations {
        let path = Path::new(&materialization.object.logical_path);
        let (blake3, sha256, size_bytes) = package_file_measurement(path)?;
        if size_bytes != materialization.object.size_bytes
            || blake3 != materialization.object.digest_blake3
            || sha256 != materialization.protocol_digest
        {
            return Err(Error::Invalid(format!(
                "package materialization bytes or digest drifted for `{}`",
                materialization.package
            )));
        }
    }
    debug_assert_eq!(
        request.package_materializations.len(),
        request
            .package_materializations
            .iter()
            .filter(|item| Path::new(&item.object.logical_path).is_file())
            .count()
    );
    debug_assert!(request.package_materializations.iter().all(|item| item.object.size_bytes > 0));
    Ok(())
}

fn validate_declared_cohort(cohort: &ToolCohort, toolchain: &VerifiedToolchain) -> Result<(), Error> {
    let declared = declared_tools(cohort);
    for (name, identity) in declared {
        let record = toolchain
            .manifest
            .tools
            .iter()
            .find(|record| record.name == name)
            .ok_or_else(|| Error::Invalid(format!("pinned cohort omits `{name}`")))?;
        if identity.version != record.version || identity.executable.digest_blake3 != record.binary_digest_blake3 {
            return Err(Error::Invalid(format!("declared component cohort drifts from pinned tool `{name}`")));
        }
    }
    if cohort.rust_target != toolchain.manifest.rust_target {
        return Err(Error::Invalid("declared Rust target drifts from pinned component cohort".to_string()));
    }
    debug_assert_eq!(declared_tools(cohort).len(), DECLARED_COMPONENT_TOOL_COUNT);
    debug_assert_eq!(cohort.rust_target, toolchain.manifest.rust_target);
    Ok(())
}

fn declared_tools(cohort: &ToolCohort) -> BTreeMap<&'static str, &ToolIdentity> {
    BTreeMap::from([
        ("rustc", &cohort.rust_toolchain),
        ("wkg", &cohort.wkg),
        ("wit-bindgen", &cohort.wit_bindgen),
        ("wasm-component-ld", &cohort.wasm_component_ld),
        ("wasm-tools", &cohort.wasm_tools),
        ("wac", &cohort.wac),
        ("wasi-virt", &cohort.wasi_virt),
        ("wizer", &cohort.wizer),
        ("wasmtime", &cohort.wasmtime),
    ])
}

pub(crate) fn core_blockers(subject: &str, blockers: &[crunch_wasm_component_core::ComponentBlocker]) -> Error {
    let summary = blockers
        .iter()
        .map(|blocker| format!("{}:{}:{}", blocker.code, blocker.subject, blocker.message))
        .collect::<Vec<_>>()
        .join("; ");
    Error::Invalid(format!("{subject} rejected: {summary}"))
}
