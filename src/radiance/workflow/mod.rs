mod assembly;
pub(super) mod helpers;
pub(super) mod view;

pub(super) const SEED_RELATIVE_PATH: &str = "seed/radiance.rv64";

#[derive(Debug, Clone)]
pub(super) struct ProofRequest<'a> {
    pub source_bundle: &'a std::path::Path,
    pub expected_source_bundle_blake3: &'a str,
    pub cohort: &'a std::path::Path,
    pub expected_cohort_blake3: &'a str,
    pub cc: &'a std::path::Path,
    pub cc_driver: &'a std::path::Path,
    pub linker: &'a std::path::Path,
    pub crt_dir: &'a std::path::Path,
    pub libgcc_dir: &'a std::path::Path,
    pub output: &'a std::path::Path,
}

#[cfg(target_os = "linux")]
struct AdmittedInputs {
    profile: crate::radiance::profile::Definition,
    manifest: crate::source_bundle::SourceBundleManifest,
    cohort: crunch_radiance_reference_core::RadianceSourceCohortWire,
}

#[cfg(target_os = "linux")]
pub(super) fn run_proof(request: ProofRequest<'_>) -> Result<view::ProofReport, crate::errors::RunError> {
    validate_request(&request)?;
    let AdmittedInputs {
        profile,
        manifest,
        cohort,
    } = admit_inputs(&request)?;
    helpers::create_root(request.output)?;
    let scratch = request.output.join("scratch");
    std::fs::create_dir(&scratch).map_err(|error| {
        crate::errors::RunError::Internal(format!("creating Radiance proof scratch {}: {error}", scratch.display()))
    })?;
    let source_state = scratch.join("source-state");
    let source_ingest_result = crate::source_bundle::import_source_bundle(&manifest, &source_state, false)?;
    let materialized = helpers::materialize(&manifest, &scratch.join("sources"))?;
    let execution = assembly::execute_graph(assembly::Input {
        operator: &request,
        profile: &profile,
        manifest: &manifest,
        cohort,
        materialized: &materialized,
        scratch: &scratch,
    })?;
    let receipt_bytes =
        crate::radiance::source::canonical_pretty_json(&execution.receipt, "Radiance reference receipt")?;
    crate::radiance::publication::publish_receipt_bytes(request.output, &receipt_bytes)?;
    let disposition = view::disposition(&execution.receipt, &profile);
    let proof_result = view::build(view::BuildInput {
        receipt: &execution.receipt,
        source_bundle_blake3: &manifest.manifest_blake3,
        audit_blake3: &execution.protected_audit_blake3,
        output: request.output,
        disposition,
    })?;
    helpers::write_import_summary(request.output, &source_ingest_result)?;
    std::fs::remove_dir_all(&scratch).map_err(|error| {
        crate::errors::RunError::Internal(format!("removing Radiance proof scratch {}: {error}", scratch.display()))
    })?;
    debug_assert_eq!(proof_result.proof_time_network_requests, 0);
    debug_assert_eq!(proof_result.proof_success, proof_result.disposition == view::ProofDisposition::Match);
    Ok(proof_result)
}

#[cfg(target_os = "linux")]
fn admit_inputs(request: &ProofRequest<'_>) -> Result<AdmittedInputs, crate::errors::RunError> {
    let profile = crate::radiance::profile::load()?;
    let manifest = crate::source_bundle::read_source_bundle(request.source_bundle)?;
    if manifest.manifest_blake3 != request.expected_source_bundle_blake3 {
        return Err(crate::errors::RunError::Internal(
            "Radiance source bundle identity does not match the explicit expected digest".to_string(),
        ));
    }
    let cohort = helpers::read_cohort(request.cohort)?;
    crunch_radiance_reference_core::validate_source_cohort(cohort.clone())
        .map_err(|error| crate::errors::RunError::Internal(format!("validating Radiance source cohort: {error:?}")))?;
    if cohort.cohort_blake3 != request.expected_cohort_blake3 {
        return Err(crate::errors::RunError::Internal(
            "Radiance source cohort identity does not match the explicit expected digest".to_string(),
        ));
    }
    helpers::validate_manifest_binding(&manifest, &cohort)?;
    debug_assert_eq!(manifest.manifest_blake3, request.expected_source_bundle_blake3);
    debug_assert_eq!(cohort.cohort_blake3, request.expected_cohort_blake3);
    Ok(AdmittedInputs {
        profile,
        manifest,
        cohort,
    })
}

#[cfg(not(target_os = "linux"))]
pub(super) fn run_proof(_request: ProofRequest<'_>) -> Result<view::ProofReport, crate::errors::RunError> {
    Err(crate::errors::RunError::Internal(
        "Radiance reference proof requires Linux ptrace and seccomp enforcement".to_string(),
    ))
}

pub(super) fn verify_proof(output: &std::path::Path) -> Result<view::ProofReport, crate::errors::RunError> {
    if !output.is_absolute() || !output.is_dir() {
        return Err(crate::errors::RunError::Internal(
            "Radiance proof verification requires an absolute output directory".to_string(),
        ));
    }
    let receipt_path = output.join(helpers::publication_path("receipt.json"));
    let bytes = std::fs::read(&receipt_path).map_err(|error| {
        crate::errors::RunError::Internal(format!("reading Radiance receipt {}: {error}", receipt_path.display()))
    })?;
    let receipt: crunch_radiance_reference_core::RadianceReferenceReceipt = serde_json::from_slice(&bytes)
        .map_err(|error| crate::errors::RunError::Internal(format!("parsing Radiance receipt: {error}")))?;
    crunch_radiance_reference_core::validate_radiance_reference_receipt(&receipt)
        .map_err(|error| crate::errors::RunError::Internal(format!("validating Radiance receipt: {error:?}")))?;
    helpers::verify_published(output, &receipt)?;
    let audit_path = output.join(helpers::publication_path(crate::radiance::publication::PROTECTED_AUDIT_PATH));
    let audit = std::fs::read(&audit_path).map_err(|error| {
        crate::errors::RunError::Internal(format!("reading Radiance audit {}: {error}", audit_path.display()))
    })?;
    let audit_blake3 = blake3::hash(&audit).to_hex().to_string();
    if audit_blake3 != receipt.protected_execution_audit_blake3 {
        return Err(crate::errors::RunError::Internal(
            "Radiance protected audit identity does not match its receipt".to_string(),
        ));
    }
    crate::radiance::publication::validate_protected_audit_bytes(&audit, &receipt)?;
    let profile = crate::radiance::profile::load()?;
    let disposition = view::disposition(&receipt, &profile);
    let proof_result = view::build(view::BuildInput {
        receipt: &receipt,
        source_bundle_blake3: &receipt.source_bundle_blake3,
        audit_blake3: &audit_blake3,
        output,
        disposition,
    })?;
    debug_assert!(!proof_result.receipt_blake3.is_empty());
    debug_assert_eq!(proof_result.proof_time_network_requests, 0);
    Ok(proof_result)
}

pub(super) fn validate_request(request: &ProofRequest<'_>) -> Result<(), crate::errors::RunError> {
    if !request.output.is_absolute() || request.output.exists() {
        return Err(crate::errors::RunError::Internal(
            "Radiance proof output must be an absent absolute path".to_string(),
        ));
    }
    crate::radiance::source::require_absolute_executable(request.cc, "Radiance C compiler launcher")?;
    crate::radiance::source::require_absolute_executable(request.cc_driver, "Radiance C compiler driver")?;
    crate::radiance::source::require_absolute_executable(request.linker, "Radiance linker")?;
    crate::radiance::runtime::native::require_link_inputs(request.crt_dir, request.libgcc_dir)?;
    if !helpers::valid_blake3_hex(request.expected_source_bundle_blake3)
        || !helpers::valid_blake3_hex(request.expected_cohort_blake3)
    {
        return Err(crate::errors::RunError::Internal(
            "Radiance expected source identities must be lowercase BLAKE3 digests".to_string(),
        ));
    }
    debug_assert!(request.output.is_absolute());
    debug_assert!(!request.output.exists());
    Ok(())
}
