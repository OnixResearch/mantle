use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::path::PathBuf;

#[cfg(test)]
pub(crate) use crunch_release_core::BLAKE3_HEX_LENGTH_CHARS as BLAKE3_HEX_LEN;
pub(crate) use crunch_release_core::BundledArtifact;
pub(crate) use crunch_release_core::BundledArtifactKind;
pub(crate) use crunch_release_core::CLAIM_SCOPE_PACKAGED_INTEGRITY;
pub(crate) use crunch_release_core::DEFAULT_PROOF_WORKFLOW_COMMAND;
pub(crate) use crunch_release_core::DEFAULT_PROOF_WORKFLOW_VERSION;
use crunch_release_core::ExternalEvidence;
#[cfg(test)]
pub(crate) use crunch_release_core::FULL_SELF_HOSTING_PROOF_SCHEMA;
use crunch_release_core::KANI_EVIDENCE_CLAIM_SCOPE;
use crunch_release_core::KANI_NON_CLAIM_RELEASE_ELIGIBILITY;
use crunch_release_core::KANI_NON_CLAIM_SEMANTICS;
use crunch_release_core::KANI_NON_CLAIM_VERIFIER_SOUNDNESS;
use crunch_release_core::KANI_NON_CLAIM_WHOLE_PROGRAM;
use crunch_release_core::KANI_RECEIPT_EVIDENCE_ROLE;
use crunch_release_core::KANI_SOLVER_KIND_CBMC_DEFAULT;
use crunch_release_core::KANI_TOOLCHAIN_EVIDENCE_SCHEMA;
use crunch_release_core::KANI_VALENCE_SEMANTIC_ROLE;
use crunch_release_core::KaniSolverIdentity;
use crunch_release_core::KaniToolchainEvidence;
use crunch_release_core::PROVIDER_FIXED_POINT_PROOF_EVIDENCE_ROLE;
use crunch_release_core::ProviderFixedPointProofArtifact;
pub(crate) use crunch_release_core::RELEASE_EVIDENCE_SCHEMA;
#[cfg(test)]
pub(crate) use crunch_release_core::RELEASE_SOURCE_ARCHIVE_PROFILE;
#[cfg(test)]
pub(crate) use crunch_release_core::RELEASE_SOURCE_ARCHIVE_VERSION;
use crunch_release_core::ReleaseEvidenceError;
pub(crate) use crunch_release_core::ReleaseEvidenceManifest;
pub(crate) use crunch_release_core::ReleaseProofLinkage;
use crunch_release_core::ReleaseReproducibilityReport;
use crunch_release_core::ReleaseReproducibilityReportLinkage;
pub(crate) use crunch_release_core::ReleaseWorkflowIdentity;
use crunch_release_core::RoleBoundedReleaseArtifact;
pub(crate) use crunch_release_core::SOURCE_ACQUISITION_KIND_EXTERNAL_ARCHIVE;
pub(crate) use crunch_release_core::SOURCE_ACQUISITION_KIND_GIT;
use crunch_release_core::STACK_PROVENANCE_CLAIM_SCOPE;
use crunch_release_core::STACK_PROVENANCE_EVIDENCE_ROLE;
use crunch_release_core::STACK_PROVENANCE_GRAPH_REPORT_SCHEMA;
use crunch_release_core::STACK_PROVENANCE_OPAQUE_BOUNDARY;
use crunch_release_core::STACK_PROVENANCE_SIDECAR_SCHEMA;
pub(crate) use crunch_release_core::SourceAcquisition;
use crunch_release_core::StackProvenanceReleaseEvidence;
use crunch_release_core::VALENCE_STACK_PROVENANCE_RECEIPT_ROLE;
use crunch_release_core::canonical_release_evidence_manifest;
use crunch_release_core::extract_full_self_hosting_proof_identity_fields;
use crunch_release_core::release_reproducibility_report_canonical_bytes;
use crunch_release_core::validate_bundled_artifact_record;
use crunch_release_core::validate_provider_fixed_point_release_artifact_binding;
use crunch_release_core::validate_release_reproducibility_report_artifact_names;
use crunch_release_core::validate_release_reproducibility_report_linkage;

use crate::errors::RunError;

const PROOF_INVENTORY_RELATIVE_PATH: &str = "stage0-prerequisites/inventory.md";
const MAX_BINARY_ARTIFACTS: u32 = 16;
const MAX_BUNDLE_TREE_ENTRIES: u32 = 4096;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FullSelfHostingProofIdentity {
    pub schema: String,
    pub proof_mode: String,
    pub selected_provider_kind: String,
    pub staged_source: String,
    pub stage2_binary_digest_blake3: String,
    pub prerequisite_inventory_digest_blake3: String,
    pub proof_manifest_digest_blake3: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct GitSourceCreateRequest {
    pub remote_url: String,
    pub commit: String,
    pub reference: Option<String>,
    pub tag: Option<String>,
}

#[derive(Debug, Clone)]
pub(crate) struct ExternalEvidenceCreateRequest {
    pub path: PathBuf,
    pub role: String,
    pub schema: String,
    pub claim_scope: String,
    pub non_claims: Vec<String>,
}

#[derive(Debug, Clone)]
pub(crate) struct KaniToolchainEvidenceCreateRequest {
    pub receipt_role: String,
    pub kani_version: String,
    pub rust_toolchain: String,
    pub cbmc_version: String,
    pub solver: KaniSolverIdentity,
    pub invocation_wrapper: String,
    pub closure_identity_blake3: String,
    pub expected_closure_identity_blake3: String,
    pub non_claims: Vec<String>,
}

// r[impl mantle.release_provenance.valence_receipt_binding]
// r[impl mantle.release_provenance.opaque_boundary]
#[derive(Debug, Clone)]
pub(crate) struct StackProvenanceCreateRequest {
    pub sidecar_path: PathBuf,
    pub valence_receipt_path: PathBuf,
    pub binary_path: Option<PathBuf>,
}

#[derive(Debug, Clone)]
pub(crate) struct ReleaseBundleCreateRequest {
    pub release_id: String,
    pub bundle_dir: PathBuf,
    pub source_archive_path: PathBuf,
    pub binary_paths: Vec<PathBuf>,
    pub proof_bundle_dir: PathBuf,
    pub workflow_command: String,
    pub workflow_version: String,
    pub reproducibility_report_path: Option<PathBuf>,
    pub provider_fixed_point_proof_dir: Option<PathBuf>,
    pub source_acquisition_url: Option<String>,
    pub git_source: Option<GitSourceCreateRequest>,
    pub external_evidence: Vec<ExternalEvidenceCreateRequest>,
    pub kani_toolchain_evidence: Vec<KaniToolchainEvidenceCreateRequest>,
    pub stack_provenance: Option<StackProvenanceCreateRequest>,
}

impl ReleaseBundleCreateRequest {
    #[cfg(test)]
    pub(crate) fn with_defaults(
        release_id: String,
        bundle_dir: PathBuf,
        source_archive_path: PathBuf,
        binary_paths: Vec<PathBuf>,
        proof_bundle_dir: PathBuf,
    ) -> Self {
        Self {
            release_id,
            bundle_dir,
            source_archive_path,
            binary_paths,
            proof_bundle_dir,
            workflow_command: DEFAULT_PROOF_WORKFLOW_COMMAND.to_string(),
            workflow_version: DEFAULT_PROOF_WORKFLOW_VERSION.to_string(),
            reproducibility_report_path: None,
            provider_fixed_point_proof_dir: None,
            source_acquisition_url: None,
            git_source: None,
            external_evidence: vec![],
            kani_toolchain_evidence: vec![],
            stack_provenance: None,
        }
    }
}

fn core_error_to_run_error(err: ReleaseEvidenceError) -> RunError {
    RunError::Internal(err.to_string())
}

pub(crate) fn create_release_evidence_bundle(
    request: &ReleaseBundleCreateRequest,
) -> Result<ReleaseEvidenceManifest, RunError> {
    validate_create_request(request)?;
    validate_optional_provider_fixed_point_proof(request)?;
    prepare_output_bundle_dir(&request.bundle_dir)?;

    let proof_identity = load_full_self_hosting_proof_identity(&request.proof_bundle_dir)?;
    let source_archive = copy_file_into_bundle(
        &request.source_archive_path,
        &request.bundle_dir,
        &bundle_file_destination("source", &request.source_archive_path, 0)?,
    )?;
    let binaries = copy_binary_set_into_bundle(&request.binary_paths, &request.bundle_dir)?;
    let proof_bundle =
        copy_directory_into_bundle(&request.proof_bundle_dir, &request.bundle_dir, Path::new("proof/self-hosting"))?;
    let inventory_path = request.proof_bundle_dir.join(PROOF_INVENTORY_RELATIVE_PATH);
    let prerequisite_inventory =
        copy_file_into_bundle(&inventory_path, &request.bundle_dir, Path::new("proof/inventory.md"))?;
    let provider_fixed_point_proof = copy_optional_provider_fixed_point_proof(request)?;
    let reproducibility_report =
        copy_optional_reproducibility_report(request, &source_archive, &binaries, &proof_bundle)?;
    let mut external_evidence = copy_external_evidence(request)?;
    let stack_provenance = copy_optional_stack_provenance_evidence(request, &mut external_evidence, &binaries)?;
    let kani_toolchain_evidence = build_kani_toolchain_evidence(request, &external_evidence)?;
    let source_archive_digest_blake3 = source_archive.digest_blake3.clone();
    let source_acquisition = build_source_acquisition(request, &source_archive_digest_blake3);

    let manifest = ReleaseEvidenceManifest {
        schema: RELEASE_EVIDENCE_SCHEMA.to_string(),
        release_id: request.release_id.clone(),
        claim_scope: CLAIM_SCOPE_PACKAGED_INTEGRITY.to_string(),
        workflow: ReleaseWorkflowIdentity {
            command: request.workflow_command.clone(),
            version: request.workflow_version.clone(),
        },
        source_archive,
        source_acquisition,
        binaries,
        proof_bundle,
        prerequisite_inventory: prerequisite_inventory.clone(),
        provider_fixed_point_proof,
        reproducibility_report,
        deterministic_build_proof: None,
        deterministic_sandbox_isolation_evidence: None,
        independent_agreement_report: None,
        external_evidence,
        kani_toolchain_evidence,
        stack_provenance,
        function_address_evidence: None,
        proof_linkage: ReleaseProofLinkage {
            release_id: request.release_id.clone(),
            source_archive_digest_blake3,
            proof_bundle_schema: proof_identity.schema,
            proof_mode: proof_identity.proof_mode,
            selected_provider_kind: proof_identity.selected_provider_kind,
            staged_source: proof_identity.staged_source,
            stage2_binary_digest_blake3: proof_identity.stage2_binary_digest_blake3,
            prerequisite_inventory_digest_blake3: proof_identity.prerequisite_inventory_digest_blake3,
            proof_manifest_digest_blake3: proof_identity.proof_manifest_digest_blake3,
        },
        provenance_coverage: None,
    };
    write_manifest_file(&request.bundle_dir, &manifest)?;
    Ok(manifest)
}

pub(crate) fn verify_release_evidence_bundle(bundle_dir: &Path) -> Result<ReleaseEvidenceManifest, RunError> {
    let manifest_path = bundle_dir.join("manifest.json");
    let manifest_bytes = std::fs::read(&manifest_path)
        .map_err(|err| RunError::Internal(format!("reading {}: {err}", manifest_path.display())))?;
    let manifest: ReleaseEvidenceManifest = serde_json::from_slice(&manifest_bytes)
        .map_err(|err| RunError::Internal(format!("parsing {}: {err}", manifest_path.display())))?;
    let expected_canonical = canonical_release_evidence_manifest(manifest.clone()).map_err(core_error_to_run_error)?;
    if manifest_bytes != expected_canonical {
        return Err(RunError::Internal("release evidence manifest.json is not canonical compact JSON".to_string()));
    }

    verify_manifest_artifacts(&manifest, bundle_dir)?;
    verify_manifest_proof_linkage(&manifest, bundle_dir)?;
    Ok(manifest)
}

pub(crate) fn load_full_self_hosting_proof_identity(
    bundle_dir: &Path,
) -> Result<FullSelfHostingProofIdentity, RunError> {
    if !bundle_dir.exists() {
        return Err(RunError::Internal(format!("proof bundle directory does not exist: {}", bundle_dir.display())));
    }
    if !bundle_dir.is_dir() {
        return Err(RunError::Internal(format!("proof bundle path is not a directory: {}", bundle_dir.display())));
    }

    let manifest_path = bundle_dir.join("manifest.json");
    let manifest_bytes = std::fs::read(&manifest_path)
        .map_err(|err| RunError::Internal(format!("reading {}: {err}", manifest_path.display())))?;
    let proof_manifest_digest_blake3 = blake3::hash(&manifest_bytes).to_hex().to_string();
    let manifest = extract_full_self_hosting_proof_identity_fields(manifest_bytes).map_err(core_error_to_run_error)?;
    Ok(FullSelfHostingProofIdentity {
        schema: manifest.schema,
        proof_mode: manifest.proof_mode,
        selected_provider_kind: manifest.selected_provider_kind,
        staged_source: manifest.staged_source,
        stage2_binary_digest_blake3: manifest.stage2_binary_digest_blake3,
        prerequisite_inventory_digest_blake3: manifest.prerequisite_inventory_digest_blake3,
        proof_manifest_digest_blake3,
    })
}

fn build_source_acquisition(
    request: &ReleaseBundleCreateRequest,
    source_archive_digest_blake3: &str,
) -> Option<SourceAcquisition> {
    if let Some(git_source) = &request.git_source {
        return Some(SourceAcquisition::git(
            git_source.remote_url.clone(),
            git_source.commit.clone(),
            git_source.reference.clone(),
            git_source.tag.clone(),
            source_archive_digest_blake3.to_string(),
        ));
    }
    request
        .source_acquisition_url
        .as_ref()
        .map(|url| SourceAcquisition::external_archive(url.clone(), source_archive_digest_blake3.to_string()))
}

fn validate_create_request(request: &ReleaseBundleCreateRequest) -> Result<(), RunError> {
    if request.release_id.trim().is_empty() {
        return Err(RunError::Internal("release evidence release_id must not be empty".to_string()));
    }
    if request.workflow_command.trim().is_empty() {
        return Err(RunError::Internal("release evidence workflow command must not be empty".to_string()));
    }
    if request.workflow_version.trim().is_empty() {
        return Err(RunError::Internal("release evidence workflow version must not be empty".to_string()));
    }
    if !request.source_archive_path.is_file() {
        return Err(RunError::Internal(format!(
            "release evidence source archive is missing: {}",
            request.source_archive_path.display()
        )));
    }
    validate_source_acquisition_request(request)?;
    let binary_count: u32 = request
        .binary_paths
        .len()
        .try_into()
        .map_err(|_| RunError::Internal("release evidence binary count overflowed u32".to_string()))?;
    if binary_count == 0 {
        return Err(RunError::Internal("release evidence requires at least one --binary input".to_string()));
    }
    if binary_count > MAX_BINARY_ARTIFACTS {
        return Err(RunError::Internal(format!(
            "release evidence received {binary_count} binaries, limit is {MAX_BINARY_ARTIFACTS}"
        )));
    }
    for binary_path in &request.binary_paths {
        if !binary_path.is_file() {
            return Err(RunError::Internal(format!("release evidence binary is missing: {}", binary_path.display())));
        }
    }
    if let Some(report_path) = &request.reproducibility_report_path {
        if !report_path.is_file() {
            return Err(RunError::Internal(format!(
                "release evidence reproducibility report is missing: {}",
                report_path.display()
            )));
        }
    }
    for evidence in &request.external_evidence {
        validate_external_evidence_request(evidence)?;
    }
    for evidence in &request.kani_toolchain_evidence {
        validate_kani_toolchain_evidence_request(evidence)?;
    }
    if let Some(stack_provenance) = &request.stack_provenance {
        validate_stack_provenance_create_request(stack_provenance)?;
    }
    Ok(())
}

fn validate_stack_provenance_create_request(request: &StackProvenanceCreateRequest) -> Result<(), RunError> {
    if !request.sidecar_path.is_file() {
        return Err(RunError::Internal(format!(
            "release evidence stack provenance sidecar is missing: {}",
            request.sidecar_path.display()
        )));
    }
    if !request.valence_receipt_path.is_file() {
        return Err(RunError::Internal(format!(
            "release evidence Valence stack provenance receipt is missing: {}",
            request.valence_receipt_path.display()
        )));
    }
    if let Some(binary_path) = &request.binary_path {
        if !binary_path.is_file() {
            return Err(RunError::Internal(format!(
                "release evidence stack provenance binary is missing: {}",
                binary_path.display()
            )));
        }
    }
    Ok(())
}

fn validate_external_evidence_request(evidence: &ExternalEvidenceCreateRequest) -> Result<(), RunError> {
    if !evidence.path.is_file() {
        return Err(RunError::Internal(format!(
            "release evidence external evidence file is missing: {}",
            evidence.path.display()
        )));
    }
    if evidence.role.trim().is_empty() {
        return Err(RunError::Internal("release evidence external evidence role must not be empty".to_string()));
    }
    if evidence.schema.trim().is_empty() {
        return Err(RunError::Internal("release evidence external evidence schema must not be empty".to_string()));
    }
    if evidence.claim_scope.trim().is_empty() {
        return Err(RunError::Internal("release evidence external evidence claim scope must not be empty".to_string()));
    }
    if evidence.non_claims.is_empty() {
        return Err(RunError::Internal("release evidence external evidence non-claims must not be empty".to_string()));
    }
    for non_claim in &evidence.non_claims {
        if non_claim.trim().is_empty() {
            return Err(RunError::Internal(
                "release evidence external evidence non-claims must not contain empty entries".to_string(),
            ));
        }
    }
    Ok(())
}

fn validate_kani_toolchain_evidence_request(evidence: &KaniToolchainEvidenceCreateRequest) -> Result<(), RunError> {
    if evidence.receipt_role.trim().is_empty() {
        return Err(RunError::Internal("release evidence Kani receipt role must not be empty".to_string()));
    }
    if evidence.kani_version.trim().is_empty() {
        return Err(RunError::Internal("release evidence Kani version must not be empty".to_string()));
    }
    if evidence.rust_toolchain.trim().is_empty() {
        return Err(RunError::Internal("release evidence Kani Rust toolchain must not be empty".to_string()));
    }
    if evidence.cbmc_version.trim().is_empty() {
        return Err(RunError::Internal("release evidence Kani CBMC version must not be empty".to_string()));
    }
    if evidence.solver.kind.trim().is_empty() || evidence.solver.version.trim().is_empty() {
        return Err(RunError::Internal("release evidence Kani solver identity must not be empty".to_string()));
    }
    if evidence.invocation_wrapper.trim().is_empty() {
        return Err(RunError::Internal("release evidence Kani invocation wrapper must not be empty".to_string()));
    }
    if evidence.non_claims.is_empty() {
        return Err(RunError::Internal("release evidence Kani non-claims must not be empty".to_string()));
    }
    Ok(())
}

fn validate_source_acquisition_request(request: &ReleaseBundleCreateRequest) -> Result<(), RunError> {
    if request.source_acquisition_url.is_some() && request.git_source.is_some() {
        return Err(RunError::Internal(
            "release evidence Git source metadata conflicts with external source acquisition URL".to_string(),
        ));
    }
    if let Some(url) = &request.source_acquisition_url {
        if url.trim().is_empty() {
            return Err(RunError::Internal("release evidence source acquisition URL must not be empty".to_string()));
        }
    }
    if let Some(git_source) = &request.git_source {
        validate_git_source_create_request(git_source)?;
    }
    Ok(())
}

fn validate_git_source_create_request(git_source: &GitSourceCreateRequest) -> Result<(), RunError> {
    if git_source.remote_url.trim().is_empty() {
        return Err(RunError::Internal("release evidence Git source URL must not be empty".to_string()));
    }
    if git_source.commit.trim().is_empty() {
        return Err(RunError::Internal("release evidence Git source commit must not be empty".to_string()));
    }
    if git_source.reference.as_deref().is_some_and(|reference| reference.trim().is_empty()) {
        return Err(RunError::Internal("release evidence Git source ref must not be empty".to_string()));
    }
    if git_source.tag.as_deref().is_some_and(|tag| tag.trim().is_empty()) {
        return Err(RunError::Internal("release evidence Git source tag must not be empty".to_string()));
    }
    Ok(())
}

fn prepare_output_bundle_dir(bundle_dir: &Path) -> Result<(), RunError> {
    if bundle_dir.exists() {
        if !bundle_dir.is_dir() {
            return Err(RunError::Internal(format!(
                "release evidence bundle path is not a directory: {}",
                bundle_dir.display()
            )));
        }
        let mut existing_entries = std::fs::read_dir(bundle_dir)
            .map_err(|err| RunError::Internal(format!("reading {}: {err}", bundle_dir.display())))?;
        if existing_entries.next().is_some() {
            return Err(RunError::Internal(format!(
                "release evidence bundle directory must be empty: {}",
                bundle_dir.display()
            )));
        }
        return Ok(());
    }
    std::fs::create_dir_all(bundle_dir)
        .map_err(|err| RunError::Internal(format!("creating {}: {err}", bundle_dir.display())))
}

fn bundle_file_destination(prefix: &str, source_path: &Path, index_u32: u32) -> Result<PathBuf, RunError> {
    let file_name = source_path.file_name().ok_or_else(|| {
        RunError::Internal(format!("release evidence input has no file name: {}", source_path.display()))
    })?;
    let mut relative = PathBuf::from(prefix);
    if index_u32 == 0 {
        relative.push(file_name);
    } else {
        relative.push(format!("{index_u32:02}-{}", file_name.to_string_lossy()));
    }
    Ok(relative)
}

fn copy_binary_set_into_bundle(binary_paths: &[PathBuf], bundle_dir: &Path) -> Result<Vec<BundledArtifact>, RunError> {
    let mut bundled = Vec::with_capacity(binary_paths.len());
    for (index_usize, binary_path) in binary_paths.iter().enumerate() {
        let index_u32: u32 = index_usize
            .try_into()
            .map_err(|_| RunError::Internal("release evidence binary index overflowed u32".to_string()))?;
        let relative = bundle_file_destination("binaries", binary_path, index_u32.saturating_add(1))?;
        bundled.push(copy_file_into_bundle(binary_path, bundle_dir, &relative)?);
    }
    assert!(!bundled.is_empty(), "binary bundle copy must emit at least one artifact");
    Ok(bundled)
}

fn validate_optional_provider_fixed_point_proof(request: &ReleaseBundleCreateRequest) -> Result<(), RunError> {
    let Some(proof_dir) = &request.provider_fixed_point_proof_dir else {
        return Ok(());
    };
    let result = crate::cargo_free_self_build::verify_provider_fixed_point_proof_bundle(proof_dir);
    if !result.valid {
        return Err(RunError::Internal(format!(
            "release evidence provider fixed-point proof is invalid: {}",
            result.blockers.join("; ")
        )));
    }
    validate_provider_fixed_point_matches_input_binaries(request, &result)
}

fn validate_provider_fixed_point_matches_input_binaries(
    request: &ReleaseBundleCreateRequest,
    result: &crate::cargo_free_self_build::ProviderFixedPointProofVerification,
) -> Result<(), RunError> {
    let Some(stage_digest) = result.stage_binary_digest_blake3.as_deref() else {
        return Err(RunError::Internal(
            "release evidence provider fixed-point proof stage binary digest is missing".to_string(),
        ));
    };
    let binaries = build_input_binary_artifacts(&request.binary_paths)?;
    validate_provider_fixed_point_release_artifact_binding(&binaries, stage_digest).map_err(core_error_to_run_error)?;
    Ok(())
}

fn build_input_binary_artifacts(binary_paths: &[PathBuf]) -> Result<Vec<BundledArtifact>, RunError> {
    let mut artifacts = Vec::with_capacity(binary_paths.len());
    for (index_usize, binary_path) in binary_paths.iter().enumerate() {
        let index_u32: u32 = index_usize
            .try_into()
            .map_err(|_| RunError::Internal("release evidence binary index overflowed u32".to_string()))?;
        let relative = bundle_file_destination("binaries", binary_path, index_u32.saturating_add(1))?;
        artifacts.push(build_artifact_record(binary_path, &relative, BundledArtifactKind::File)?);
    }
    Ok(artifacts)
}

fn copy_optional_provider_fixed_point_proof(
    request: &ReleaseBundleCreateRequest,
) -> Result<Option<ProviderFixedPointProofArtifact>, RunError> {
    let Some(proof_dir) = &request.provider_fixed_point_proof_dir else {
        return Ok(None);
    };
    let artifact = copy_directory_into_bundle(proof_dir, &request.bundle_dir, Path::new("proof/provider-fixed-point"))?;
    Ok(Some(ProviderFixedPointProofArtifact {
        kind: artifact.kind,
        relative_path: artifact.relative_path,
        size_bytes: artifact.size_bytes,
        digest_blake3: artifact.digest_blake3,
        evidence_role: PROVIDER_FIXED_POINT_PROOF_EVIDENCE_ROLE.to_string(),
    }))
}

fn copy_optional_reproducibility_report(
    request: &ReleaseBundleCreateRequest,
    source_archive: &BundledArtifact,
    binaries: &[BundledArtifact],
    proof_bundle: &BundledArtifact,
) -> Result<Option<BundledArtifact>, RunError> {
    let Some(report_path) = &request.reproducibility_report_path else {
        return Ok(None);
    };
    validate_reproducibility_report_for_bundle(request, report_path, source_archive, binaries, proof_bundle)?;
    let relative = Path::new("reproducibility").join("reproducibility-report.json");
    copy_file_into_bundle(report_path, &request.bundle_dir, &relative).map(Some)
}

fn copy_external_evidence(request: &ReleaseBundleCreateRequest) -> Result<Vec<ExternalEvidence>, RunError> {
    let mut records = Vec::with_capacity(request.external_evidence.len());
    for evidence in &request.external_evidence {
        let next_index = next_external_evidence_index(records.len())?;
        records.push(copy_external_evidence_record(request, evidence, next_index)?);
    }
    Ok(records)
}

fn next_external_evidence_index(existing_count: usize) -> Result<u32, RunError> {
    let index: u32 = existing_count
        .try_into()
        .map_err(|_| RunError::Internal("release evidence external evidence index overflowed u32".to_string()))?;
    Ok(index.saturating_add(1))
}

fn copy_external_evidence_record(
    request: &ReleaseBundleCreateRequest,
    evidence: &ExternalEvidenceCreateRequest,
    index: u32,
) -> Result<ExternalEvidence, RunError> {
    let relative = bundle_file_destination("external-evidence", &evidence.path, index)?;
    let artifact = copy_file_into_bundle(&evidence.path, &request.bundle_dir, &relative)?;
    Ok(ExternalEvidence {
        role: evidence.role.clone(),
        schema: evidence.schema.clone(),
        relative_path: artifact.relative_path,
        digest_blake3: artifact.digest_blake3,
        claim_scope: evidence.claim_scope.clone(),
        non_claims: evidence.non_claims.clone(),
    })
}

// r[impl mantle.release_provenance.valence_receipt_binding.sidecar_digest]
// r[impl mantle.release_provenance.valence_receipt_binding.valence_receipt]
// r[impl mantle.release_provenance.valence_receipt_binding.binary_identity]
// r[impl mantle.release_provenance.valence_receipt_binding.stale]
fn copy_optional_stack_provenance_evidence(
    request: &ReleaseBundleCreateRequest,
    external_evidence: &mut Vec<ExternalEvidence>,
    binaries: &[BundledArtifact],
) -> Result<Option<StackProvenanceReleaseEvidence>, RunError> {
    let Some(stack_request) = &request.stack_provenance else {
        return Ok(None);
    };
    let sidecar = ExternalEvidenceCreateRequest {
        path: stack_request.sidecar_path.clone(),
        role: STACK_PROVENANCE_EVIDENCE_ROLE.to_string(),
        schema: STACK_PROVENANCE_SIDECAR_SCHEMA.to_string(),
        claim_scope: STACK_PROVENANCE_CLAIM_SCOPE.to_string(),
        non_claims: vec![STACK_PROVENANCE_OPAQUE_BOUNDARY.to_string()],
    };
    let valence_receipt = ExternalEvidenceCreateRequest {
        path: stack_request.valence_receipt_path.clone(),
        role: VALENCE_STACK_PROVENANCE_RECEIPT_ROLE.to_string(),
        schema: STACK_PROVENANCE_GRAPH_REPORT_SCHEMA.to_string(),
        claim_scope: STACK_PROVENANCE_CLAIM_SCOPE.to_string(),
        non_claims: vec![STACK_PROVENANCE_OPAQUE_BOUNDARY.to_string()],
    };
    let sidecar_artifact =
        copy_external_evidence_record(request, &sidecar, next_external_evidence_index(external_evidence.len())?)?;
    external_evidence.push(sidecar_artifact.clone());
    let receipt_artifact = copy_external_evidence_record(
        request,
        &valence_receipt,
        next_external_evidence_index(external_evidence.len())?,
    )?;
    external_evidence.push(receipt_artifact.clone());
    let binary_artifact = select_stack_provenance_binary_artifact(request, binaries)?;
    Ok(Some(StackProvenanceReleaseEvidence {
        sidecar_role: sidecar_artifact.role,
        sidecar_schema: sidecar_artifact.schema,
        sidecar_claim_scope: sidecar_artifact.claim_scope,
        sidecar_relative_path: sidecar_artifact.relative_path,
        sidecar_digest_blake3: sidecar_artifact.digest_blake3,
        valence_receipt_role: receipt_artifact.role,
        valence_receipt_schema: receipt_artifact.schema,
        valence_receipt_relative_path: receipt_artifact.relative_path,
        valence_receipt_digest_blake3: receipt_artifact.digest_blake3,
        release_binary_relative_path: binary_artifact.relative_path.clone(),
        release_binary_digest_blake3: binary_artifact.digest_blake3.clone(),
        non_claims: vec![STACK_PROVENANCE_OPAQUE_BOUNDARY.to_string()],
    }))
}

fn select_stack_provenance_binary_artifact<'a>(
    request: &ReleaseBundleCreateRequest,
    binaries: &'a [BundledArtifact],
) -> Result<&'a BundledArtifact, RunError> {
    if let Some(binary_path) = request.stack_provenance.as_ref().and_then(|request| request.binary_path.as_ref()) {
        let Some(binary_index) = request.binary_paths.iter().position(|path| path == binary_path) else {
            return Err(RunError::Internal(
                "release evidence stack provenance binary must match one --binary input".to_string(),
            ));
        };
        return binaries.get(binary_index).ok_or_else(|| {
            RunError::Internal(
                "release evidence stack provenance binary index did not match copied binaries".to_string(),
            )
        });
    }
    if binaries.len() == 1 {
        return Ok(&binaries[0]);
    }
    Err(RunError::Internal(
        "release evidence stack provenance binary must be selected when multiple binaries are bundled".to_string(),
    ))
}

fn build_kani_toolchain_evidence(
    request: &ReleaseBundleCreateRequest,
    external_evidence: &[ExternalEvidence],
) -> Result<Vec<KaniToolchainEvidence>, RunError> {
    let mut records = Vec::with_capacity(request.kani_toolchain_evidence.len());
    for evidence in &request.kani_toolchain_evidence {
        let linked_receipt =
            external_evidence.iter().find(|external| external.role == evidence.receipt_role).ok_or_else(|| {
                RunError::Internal(format!(
                    "release evidence Kani receipt role required but missing from external evidence: {}",
                    evidence.receipt_role
                ))
            })?;
        records.push(KaniToolchainEvidence {
            schema: KANI_TOOLCHAIN_EVIDENCE_SCHEMA.to_string(),
            receipt_role: linked_receipt.role.clone(),
            receipt_relative_path: linked_receipt.relative_path.clone(),
            receipt_digest_blake3: linked_receipt.digest_blake3.clone(),
            kani_version: evidence.kani_version.clone(),
            rust_toolchain: evidence.rust_toolchain.clone(),
            cbmc_version: evidence.cbmc_version.clone(),
            solver: evidence.solver.clone(),
            invocation_wrapper: evidence.invocation_wrapper.clone(),
            closure_identity_blake3: evidence.closure_identity_blake3.clone(),
            expected_closure_identity_blake3: evidence.expected_closure_identity_blake3.clone(),
            valence_semantic_role: KANI_VALENCE_SEMANTIC_ROLE.to_string(),
            claim_scope: KANI_EVIDENCE_CLAIM_SCOPE.to_string(),
            non_claims: evidence.non_claims.clone(),
        });
    }
    Ok(records)
}

fn validate_reproducibility_report_for_bundle(
    request: &ReleaseBundleCreateRequest,
    report_path: &Path,
    source_archive: &BundledArtifact,
    binaries: &[BundledArtifact],
    proof_bundle: &BundledArtifact,
) -> Result<(), RunError> {
    let report_bytes = std::fs::read(report_path)
        .map_err(|err| RunError::Internal(format!("reading {}: {err}", report_path.display())))?;
    let report: ReleaseReproducibilityReport = serde_json::from_slice(&report_bytes)
        .map_err(|err| RunError::Internal(format!("parsing {}: {err}", report_path.display())))?;
    let canonical_bytes =
        release_reproducibility_report_canonical_bytes(report.clone()).map_err(core_error_to_run_error)?;
    if report_bytes != canonical_bytes {
        return Err(RunError::Internal(
            "release evidence reproducibility report is not canonical compact JSON".to_string(),
        ));
    }
    let expected = ReleaseReproducibilityReportLinkage {
        release_id: request.release_id.clone(),
        source_archive_digest_blake3: source_archive.digest_blake3.clone(),
        proof_bundle_digest_blake3: proof_bundle.digest_blake3.clone(),
    };
    let report = validate_release_reproducibility_report_linkage(report, expected).map_err(core_error_to_run_error)?;
    let expected_names = binaries.iter().map(|artifact| artifact.relative_path.clone()).collect::<Vec<_>>();
    validate_release_reproducibility_report_artifact_names(report, expected_names).map_err(core_error_to_run_error)?;
    Ok(())
}

fn copy_file_into_bundle(
    source_path: &Path,
    bundle_dir: &Path,
    relative_path: &Path,
) -> Result<BundledArtifact, RunError> {
    if !source_path.is_file() {
        return Err(RunError::Internal(format!("bundle input file missing: {}", source_path.display())));
    }
    let dest_path = bundle_dir.join(relative_path);
    if let Some(parent) = dest_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|err| RunError::Internal(format!("creating {}: {err}", parent.display())))?;
    }
    std::fs::copy(source_path, &dest_path).map_err(|err| {
        RunError::Internal(format!("copying {} to {}: {err}", source_path.display(), dest_path.display()))
    })?;
    build_artifact_record(&dest_path, relative_path, BundledArtifactKind::File)
}

fn copy_directory_into_bundle(
    source_dir: &Path,
    bundle_dir: &Path,
    relative_path: &Path,
) -> Result<BundledArtifact, RunError> {
    if !source_dir.is_dir() {
        return Err(RunError::Internal(format!("bundle input directory missing: {}", source_dir.display())));
    }
    let dest_dir = bundle_dir.join(relative_path);
    std::fs::create_dir_all(&dest_dir)
        .map_err(|err| RunError::Internal(format!("creating {}: {err}", dest_dir.display())))?;
    copy_directory_tree(source_dir, &dest_dir)?;
    build_artifact_record(&dest_dir, relative_path, BundledArtifactKind::Directory)
}

pub(crate) fn copy_directory_tree(source_dir: &Path, dest_dir: &Path) -> Result<(), RunError> {
    let mut entries = Vec::new();
    collect_paths_sorted(source_dir, &mut entries)?;
    assert!(
        entries.len() <= usize::try_from(MAX_BUNDLE_TREE_ENTRIES).unwrap(),
        "bundle tree copy entry count exceeded limit"
    );
    for source_entry in &entries {
        let relative = source_entry.strip_prefix(source_dir).map_err(|err| {
            RunError::Internal(format!(
                "bundle tree strip_prefix {} from {}: {err}",
                source_entry.display(),
                source_dir.display()
            ))
        })?;
        let dest_entry = dest_dir.join(relative);
        copy_tree_entry(source_entry, &dest_entry)?;
    }
    Ok(())
}

fn copy_tree_entry(source_path: &Path, dest_path: &Path) -> Result<(), RunError> {
    let metadata = std::fs::symlink_metadata(source_path)
        .map_err(|err| RunError::Internal(format!("symlink_metadata {}: {err}", source_path.display())))?;
    if metadata.is_dir() {
        std::fs::create_dir_all(dest_path)
            .map_err(|err| RunError::Internal(format!("creating {}: {err}", dest_path.display())))?;
        return Ok(());
    }
    if metadata.is_file() {
        if let Some(parent) = dest_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|err| RunError::Internal(format!("creating {}: {err}", parent.display())))?;
        }
        std::fs::copy(source_path, dest_path).map_err(|err| {
            RunError::Internal(format!("copying {} to {}: {err}", source_path.display(), dest_path.display()))
        })?;
        return Ok(());
    }
    if metadata.file_type().is_symlink() {
        return copy_symlink_entry(source_path, dest_path);
    }
    Err(RunError::Internal(format!("unsupported bundle tree entry type: {}", source_path.display())))
}

#[cfg(unix)]
fn copy_symlink_entry(source_path: &Path, dest_path: &Path) -> Result<(), RunError> {
    use std::os::unix::fs::symlink;

    let target = std::fs::read_link(source_path)
        .map_err(|err| RunError::Internal(format!("read_link {}: {err}", source_path.display())))?;
    if let Some(parent) = dest_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|err| RunError::Internal(format!("creating {}: {err}", parent.display())))?;
    }
    symlink(&target, dest_path).map_err(|err| {
        RunError::Internal(format!("creating symlink {} -> {}: {err}", dest_path.display(), target.display()))
    })
}

#[cfg(not(unix))]
fn copy_symlink_entry(source_path: &Path, _dest_path: &Path) -> Result<(), RunError> {
    Err(RunError::Internal(format!(
        "symlink bundle copy is only supported on Unix: {}",
        source_path.display()
    )))
}

fn build_artifact_record(
    path: &Path,
    relative_path: &Path,
    kind: BundledArtifactKind,
) -> Result<BundledArtifact, RunError> {
    let relative_path_string = path_to_forward_slash_string(relative_path)?;
    let (size_bytes, digest_blake3) = match kind {
        BundledArtifactKind::File => hash_file(path)?,
        BundledArtifactKind::Directory => hash_directory(path)?,
    };
    let artifact = BundledArtifact {
        kind,
        relative_path: relative_path_string,
        size_bytes,
        digest_blake3,
    };
    validate_bundled_artifact_record(artifact, "artifact".to_string()).map_err(core_error_to_run_error)
}

pub(crate) fn compute_path_blake3_digest(path: &Path) -> Result<String, RunError> {
    if path.is_file() {
        return hash_file(path).map(|(_size_bytes, digest_blake3)| digest_blake3);
    }
    if path.is_dir() {
        return hash_directory(path).map(|(_size_bytes, digest_blake3)| digest_blake3);
    }
    Err(RunError::Internal(format!("expected file or directory artifact: {}", path.display())))
}

fn hash_file(path: &Path) -> Result<(u64, String), RunError> {
    let metadata =
        std::fs::metadata(path).map_err(|err| RunError::Internal(format!("metadata {}: {err}", path.display())))?;
    if !metadata.is_file() {
        return Err(RunError::Internal(format!("expected file artifact: {}", path.display())));
    }
    let mut file = File::open(path).map_err(|err| RunError::Internal(format!("open {}: {err}", path.display())))?;
    let mut hasher = blake3::Hasher::new();
    let mut buffer = [0_u8; 8192];
    loop {
        let bytes_read = file
            .read(&mut buffer)
            .map_err(|err| RunError::Internal(format!("read {}: {err}", path.display())))?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }
    Ok((metadata.len(), hasher.finalize().to_hex().to_string()))
}

fn hash_directory(path: &Path) -> Result<(u64, String), RunError> {
    if !path.is_dir() {
        return Err(RunError::Internal(format!("expected directory artifact: {}", path.display())));
    }
    let mut entries = Vec::new();
    collect_paths_sorted(path, &mut entries)?;
    let mut hasher = blake3::Hasher::new();
    let mut total_file_bytes: u64 = 0;
    for entry in &entries {
        total_file_bytes = total_file_bytes.saturating_add(hash_tree_entry(path, entry, &mut hasher)?);
    }
    if total_file_bytes == 0 {
        total_file_bytes = 1;
    }
    Ok((total_file_bytes, hasher.finalize().to_hex().to_string()))
}

fn collect_paths_sorted(root: &Path, entries: &mut Vec<PathBuf>) -> Result<(), RunError> {
    let mut children = Vec::new();
    for child_result in
        std::fs::read_dir(root).map_err(|err| RunError::Internal(format!("read_dir {}: {err}", root.display())))?
    {
        let child =
            child_result.map_err(|err| RunError::Internal(format!("read_dir entry {}: {err}", root.display())))?;
        children.push(child.path());
    }
    children.sort();
    for child in children {
        let entry_count_u32: u32 = entries
            .len()
            .try_into()
            .map_err(|_| RunError::Internal("bundle tree entry count overflowed u32".to_string()))?;
        if entry_count_u32 >= MAX_BUNDLE_TREE_ENTRIES {
            return Err(RunError::Internal(format!("bundle tree exceeds {MAX_BUNDLE_TREE_ENTRIES} entries")));
        }
        entries.push(child.clone());
        if child.is_dir() {
            collect_paths_sorted(&child, entries)?;
        }
    }
    Ok(())
}

fn hash_tree_entry(root: &Path, entry: &Path, hasher: &mut blake3::Hasher) -> Result<u64, RunError> {
    let relative = entry.strip_prefix(root).map_err(|err| {
        RunError::Internal(format!("tree hash strip_prefix {} from {}: {err}", entry.display(), root.display()))
    })?;
    let metadata = std::fs::symlink_metadata(entry)
        .map_err(|err| RunError::Internal(format!("symlink_metadata {}: {err}", entry.display())))?;
    let relative_bytes = relative.as_os_str().as_encoded_bytes();
    hasher.update(&(relative_bytes.len() as u64).to_le_bytes());
    hasher.update(relative_bytes);
    hasher.update(&entry_mode_bits(&metadata).to_le_bytes());

    if metadata.file_type().is_symlink() {
        let target = std::fs::read_link(entry)
            .map_err(|err| RunError::Internal(format!("read_link {}: {err}", entry.display())))?;
        let target_bytes = target.as_os_str().as_encoded_bytes();
        hasher.update(b"symlink\0");
        hasher.update(&(target_bytes.len() as u64).to_le_bytes());
        hasher.update(target_bytes);
        return Ok(0);
    }
    if metadata.is_dir() {
        hasher.update(b"dir\0");
        return Ok(0);
    }
    if metadata.is_file() {
        hasher.update(b"file\0");
        hasher.update(&metadata.len().to_le_bytes());
        let mut file =
            File::open(entry).map_err(|err| RunError::Internal(format!("open {}: {err}", entry.display())))?;
        let mut buffer = [0_u8; 8192];
        loop {
            let bytes_read = file
                .read(&mut buffer)
                .map_err(|err| RunError::Internal(format!("read {}: {err}", entry.display())))?;
            if bytes_read == 0 {
                break;
            }
            hasher.update(&buffer[..bytes_read]);
        }
        return Ok(metadata.len());
    }
    Err(RunError::Internal(format!("unsupported tree hash entry type: {}", entry.display())))
}

#[cfg(unix)]
fn entry_mode_bits(metadata: &std::fs::Metadata) -> u32 {
    use std::os::unix::fs::MetadataExt;
    metadata.mode()
}

#[cfg(not(unix))]
fn entry_mode_bits(_metadata: &std::fs::Metadata) -> u32 {
    0
}

fn write_manifest_file(bundle_dir: &Path, manifest: &ReleaseEvidenceManifest) -> Result<(), RunError> {
    let manifest_bytes = canonical_release_evidence_manifest(manifest.clone()).map_err(core_error_to_run_error)?;
    let manifest_path = bundle_dir.join("manifest.json");
    std::fs::write(&manifest_path, manifest_bytes)
        .map_err(|err| RunError::Internal(format!("writing {}: {err}", manifest_path.display())))
}

fn verify_manifest_artifacts(manifest: &ReleaseEvidenceManifest, bundle_dir: &Path) -> Result<(), RunError> {
    verify_artifact_matches_bundle(&manifest.source_archive, bundle_dir, "source_archive")?;
    verify_artifact_matches_bundle(&manifest.proof_bundle, bundle_dir, "proof_bundle")?;
    verify_artifact_matches_bundle(&manifest.prerequisite_inventory, bundle_dir, "prerequisite_inventory")?;
    if let Some(artifact) = &manifest.provider_fixed_point_proof {
        verify_provider_fixed_point_artifact_matches_bundle(artifact, bundle_dir)?;
    }
    for (index_usize, artifact) in manifest.binaries.iter().enumerate() {
        let index_u32: u32 = index_usize
            .try_into()
            .map_err(|_| RunError::Internal("release evidence verify binary index overflowed u32".to_string()))?;
        verify_artifact_matches_bundle(artifact, bundle_dir, &format!("binaries[{index_u32}]"))?;
    }
    if let Some(report) = &manifest.reproducibility_report {
        verify_artifact_matches_bundle(report, bundle_dir, "reproducibility_report")?;
    }
    if let Some(proof) = &manifest.deterministic_build_proof {
        verify_role_bounded_artifact_matches_bundle(proof, bundle_dir, "deterministic_build_proof")?;
    }
    if let Some(evidence) = &manifest.deterministic_sandbox_isolation_evidence {
        verify_role_bounded_artifact_matches_bundle(evidence, bundle_dir, "deterministic_sandbox_isolation_evidence")?;
    }
    if let Some(report) = &manifest.independent_agreement_report {
        verify_artifact_matches_bundle(report, bundle_dir, "independent_agreement_report")?;
    }
    for (index_usize, evidence) in manifest.external_evidence.iter().enumerate() {
        let index_u32: u32 = index_usize.try_into().map_err(|_| {
            RunError::Internal("release evidence verify external evidence index overflowed u32".to_string())
        })?;
        verify_external_evidence_matches_bundle(evidence, bundle_dir, &format!("external_evidence[{index_u32}]"))?;
    }
    Ok(())
}

fn verify_external_evidence_matches_bundle(
    evidence: &ExternalEvidence,
    bundle_dir: &Path,
    field_name: &str,
) -> Result<(), RunError> {
    let bundled = BundledArtifact {
        kind: BundledArtifactKind::File,
        relative_path: evidence.relative_path.clone(),
        size_bytes: std::fs::metadata(bundle_dir.join(&evidence.relative_path))
            .map_err(|err| RunError::Internal(format!("verifying {field_name}: {err}")))?
            .len(),
        digest_blake3: evidence.digest_blake3.clone(),
    };
    verify_artifact_matches_bundle(&bundled, bundle_dir, field_name)
}

fn verify_provider_fixed_point_artifact_matches_bundle(
    artifact: &ProviderFixedPointProofArtifact,
    bundle_dir: &Path,
) -> Result<(), RunError> {
    let bundled = BundledArtifact {
        kind: artifact.kind,
        relative_path: artifact.relative_path.clone(),
        size_bytes: artifact.size_bytes,
        digest_blake3: artifact.digest_blake3.clone(),
    };
    verify_artifact_matches_bundle(&bundled, bundle_dir, "provider_fixed_point_proof")
}

fn verify_role_bounded_artifact_matches_bundle(
    artifact: &RoleBoundedReleaseArtifact,
    bundle_dir: &Path,
    field_name: &str,
) -> Result<(), RunError> {
    let bundled = BundledArtifact {
        kind: artifact.kind,
        relative_path: artifact.relative_path.clone(),
        size_bytes: artifact.size_bytes,
        digest_blake3: artifact.digest_blake3.clone(),
    };
    verify_artifact_matches_bundle(&bundled, bundle_dir, field_name)
}

fn verify_artifact_matches_bundle(
    artifact: &BundledArtifact,
    bundle_dir: &Path,
    field_name: &str,
) -> Result<(), RunError> {
    let artifact_path = bundle_dir.join(&artifact.relative_path);
    let actual = build_artifact_record(&artifact_path, Path::new(&artifact.relative_path), artifact.kind)
        .map_err(|err| RunError::Internal(format!("verifying {field_name}: {err}")))?;
    if artifact != &actual {
        return Err(RunError::Internal(format!(
            "release evidence {field_name} does not match manifest: expected {} {} got {} {}",
            artifact.size_bytes, artifact.digest_blake3, actual.size_bytes, actual.digest_blake3,
        )));
    }
    Ok(())
}

fn verify_manifest_proof_linkage(manifest: &ReleaseEvidenceManifest, bundle_dir: &Path) -> Result<(), RunError> {
    let proof_bundle_dir = bundle_dir.join(&manifest.proof_bundle.relative_path);
    let proof_identity = load_full_self_hosting_proof_identity(&proof_bundle_dir)?;
    if manifest.proof_linkage.release_id != manifest.release_id {
        return Err(RunError::Internal("release evidence proof linkage release_id mismatch".to_string()));
    }
    if manifest.proof_linkage.source_archive_digest_blake3 != manifest.source_archive.digest_blake3 {
        return Err(RunError::Internal("release evidence proof linkage source archive digest mismatch".to_string()));
    }
    if proof_identity.schema != manifest.proof_linkage.proof_bundle_schema {
        return Err(RunError::Internal("release evidence proof linkage schema mismatch".to_string()));
    }
    if proof_identity.proof_mode != manifest.proof_linkage.proof_mode {
        return Err(RunError::Internal("release evidence proof linkage proof_mode mismatch".to_string()));
    }
    if proof_identity.selected_provider_kind != manifest.proof_linkage.selected_provider_kind {
        return Err(RunError::Internal("release evidence proof linkage selected_provider_kind mismatch".to_string()));
    }
    if proof_identity.staged_source != manifest.proof_linkage.staged_source {
        return Err(RunError::Internal("release evidence proof linkage staged_source mismatch".to_string()));
    }
    if proof_identity.stage2_binary_digest_blake3 != manifest.proof_linkage.stage2_binary_digest_blake3 {
        return Err(RunError::Internal("release evidence proof linkage stage2 digest mismatch".to_string()));
    }
    if proof_identity.prerequisite_inventory_digest_blake3
        != manifest.proof_linkage.prerequisite_inventory_digest_blake3
    {
        return Err(RunError::Internal(
            "release evidence proof linkage prerequisite inventory digest mismatch".to_string(),
        ));
    }
    if proof_identity.proof_manifest_digest_blake3 != manifest.proof_linkage.proof_manifest_digest_blake3 {
        return Err(RunError::Internal("release evidence proof linkage proof manifest digest mismatch".to_string()));
    }
    Ok(())
}

fn path_to_forward_slash_string(path: &Path) -> Result<String, RunError> {
    let raw = path
        .to_str()
        .ok_or_else(|| RunError::Internal(format!("non-utf8 bundle path: {}", path.display())))?;
    Ok(raw.replace('\\', "/"))
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    const PROVIDER_FIXED_POINT_SCHEMA: &str = "mantle-cargo-free-fixed-point-proof-v1";
    const RUST_SOURCE_PROVIDER_BINDING_SCHEMA: &str = "mantle-cargo-free-rust-source-provider-binding-v1";
    const PROVIDER_BINARY_BYTES: &[u8] = b"provider-fixed-point-binary";
    const PROVIDER_STAGE_UNIT_COUNT: u32 = 2;

    fn sample_digest(seed: u8) -> String {
        let byte = format!("{:x}", seed % 16);
        byte.repeat(BLAKE3_HEX_LEN)
    }

    fn write_file(path: &Path, content: &[u8]) {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(path, content).unwrap();
    }

    fn sample_artifact(kind: BundledArtifactKind, relative_path: &str, seed: u8) -> BundledArtifact {
        BundledArtifact {
            kind,
            relative_path: relative_path.to_string(),
            size_bytes: 123,
            digest_blake3: sample_digest(seed),
        }
    }

    fn sample_kani_non_claims() -> Vec<String> {
        vec![
            KANI_NON_CLAIM_WHOLE_PROGRAM.to_string(),
            KANI_NON_CLAIM_VERIFIER_SOUNDNESS.to_string(),
            KANI_NON_CLAIM_SEMANTICS.to_string(),
            KANI_NON_CLAIM_RELEASE_ELIGIBILITY.to_string(),
        ]
    }

    fn sample_kani_toolchain_request(receipt_role: &str) -> KaniToolchainEvidenceCreateRequest {
        KaniToolchainEvidenceCreateRequest {
            receipt_role: receipt_role.to_string(),
            kani_version: "kani 0.63.0".to_string(),
            rust_toolchain: "rustc 1.91.0-nightly".to_string(),
            cbmc_version: "cbmc 6.4.0".to_string(),
            solver: KaniSolverIdentity {
                kind: KANI_SOLVER_KIND_CBMC_DEFAULT.to_string(),
                version: "cbmc-default".to_string(),
            },
            invocation_wrapper: "cargo kani --harness checked_add".to_string(),
            closure_identity_blake3: sample_digest(14),
            expected_closure_identity_blake3: sample_digest(14),
            non_claims: sample_kani_non_claims(),
        }
    }

    fn sample_manifest() -> ReleaseEvidenceManifest {
        let stage2_binary = sample_artifact(BundledArtifactKind::File, "binaries/01-mantle", 3);
        let inventory = sample_artifact(BundledArtifactKind::File, "proof/inventory.md", 5);
        ReleaseEvidenceManifest {
            schema: RELEASE_EVIDENCE_SCHEMA.to_string(),
            release_id: "mantle-0.1.0-rc1".to_string(),
            claim_scope: CLAIM_SCOPE_PACKAGED_INTEGRITY.to_string(),
            workflow: ReleaseWorkflowIdentity {
                command: DEFAULT_PROOF_WORKFLOW_COMMAND.to_string(),
                version: DEFAULT_PROOF_WORKFLOW_VERSION.to_string(),
            },
            source_archive: sample_artifact(BundledArtifactKind::File, "source/mantle-src.tar", 1),
            source_acquisition: None,
            binaries: vec![stage2_binary.clone()],
            proof_bundle: sample_artifact(BundledArtifactKind::Directory, "proof/self-hosting", 7),
            prerequisite_inventory: inventory.clone(),
            provider_fixed_point_proof: None,
            reproducibility_report: None,
            deterministic_build_proof: None,
            deterministic_sandbox_isolation_evidence: None,
            independent_agreement_report: None,
            external_evidence: vec![],
            kani_toolchain_evidence: vec![],
            stack_provenance: None,
            function_address_evidence: None,
            proof_linkage: ReleaseProofLinkage {
                release_id: "mantle-0.1.0-rc1".to_string(),
                source_archive_digest_blake3: sample_digest(1),
                proof_bundle_schema: FULL_SELF_HOSTING_PROOF_SCHEMA.to_string(),
                proof_mode: "fixed-point".to_string(),
                selected_provider_kind: "source-root".to_string(),
                staged_source: "/tmp/proof-store/abcd-mantle-src".to_string(),
                stage2_binary_digest_blake3: stage2_binary.digest_blake3,
                prerequisite_inventory_digest_blake3: inventory.digest_blake3,
                proof_manifest_digest_blake3: sample_digest(9),
            },
            provenance_coverage: None,
        }
    }

    fn write_full_proof_manifest(bundle_dir: &Path, inventory_digest: &str, stage2_digest: &str) {
        let manifest = json!({
            "schema": FULL_SELF_HOSTING_PROOF_SCHEMA,
            "staged_source": "/tmp/proof-store/abcd-mantle-src",
            "prerequisites": {
                "mode": "fixed-point",
                "provider_kind": "source-root",
                "inventory_doc": {
                    "path": "/tmp/proof-bundle/stage0-prerequisites/inventory.md",
                    "size_bytes": 9,
                    "digest_blake3": inventory_digest
                }
            },
            "binaries": {
                "stage1": {
                    "path": "/tmp/proof-store/stage1-mantle/bin/mantle",
                    "size_bytes": 20,
                    "digest_blake3": sample_digest(10)
                },
                "stage2": {
                    "path": "/tmp/proof-store/stage2-mantle/bin/mantle",
                    "size_bytes": 13,
                    "digest_blake3": stage2_digest
                }
            },
            "tools": {
                "stage0_bwrap": {
                    "path": "/tmp/proof-store/stage0-bwrap/bin/bwrap",
                    "size_bytes": 22,
                    "digest_blake3": sample_digest(12)
                },
                "stage0_busybox": {
                    "path": "/tmp/proof-store/stage0-busybox/bin/busybox",
                    "size_bytes": 23,
                    "digest_blake3": sample_digest(13)
                },
                "stage2_bwrap": {
                    "path": "/tmp/proof-store/stage2-bwrap/bin/bwrap",
                    "size_bytes": 24,
                    "digest_blake3": sample_digest(14)
                },
                "stage2_busybox": {
                    "path": "/tmp/proof-store/stage2-busybox/bin/busybox",
                    "size_bytes": 25,
                    "digest_blake3": sample_digest(15)
                }
            },
            "fixed_point": {
                "stage1_equals_stage2": true,
                "stage0_bwrap_equals_stage2_bwrap": true,
                "stage0_busybox_equals_stage2_busybox": true
            },
            "stage0": {
                "report": {
                    "staged_source": "/tmp/proof-store/abcd-mantle-src",
                    "output_binary": "/tmp/proof-store/stage1-mantle/bin/mantle",
                    "busybox_path": "/tmp/proof-store/stage0-busybox/bin/busybox"
                }
            },
            "stage2": {
                "report": {
                    "staged_source": "/tmp/proof-store/abcd-mantle-src",
                    "output_binary": "/tmp/proof-store/stage2-mantle/bin/mantle",
                    "busybox_path": "/tmp/proof-store/stage2-busybox/bin/busybox"
                }
            }
        });
        std::fs::create_dir_all(bundle_dir).unwrap();
        write_file(&bundle_dir.join("manifest.json"), &serde_json::to_vec(&manifest).unwrap());
        write_file(&bundle_dir.join(PROOF_INVENTORY_RELATIVE_PATH), b"inventory");
        write_file(&bundle_dir.join("summary.txt"), b"summary");
        write_file(&bundle_dir.join("stage0/stdout.txt"), b"stage0 stdout");
    }

    fn write_provider_fixed_point_proof_bundle(bundle_dir: &Path, binary_bytes: &[u8]) {
        let binary_digest = blake3::hash(binary_bytes).to_hex().to_string();
        let policy_digest = sample_digest(12);
        write_file(&bundle_dir.join("stage1/mantle"), binary_bytes);
        write_file(&bundle_dir.join("stage2/mantle"), binary_bytes);
        write_provider_stage_receipt(&bundle_dir.join("stage1/receipt.json"));
        write_provider_stage_receipt(&bundle_dir.join("stage2/receipt.json"));
        write_file(&bundle_dir.join("non-claims.txt"), b"This proof does not claim Mantle bootstrap or release reproducibility.\nThis proof does not claim full Cargo compatibility.\n");
        write_file(
            &bundle_dir.join("preflight.json"),
            &serde_json::to_vec(&provider_preflight_json(&policy_digest)).unwrap(),
        );
        write_file(
            &bundle_dir.join("meta.json"),
            &serde_json::to_vec(&provider_meta_json(bundle_dir, &binary_digest, &policy_digest)).unwrap(),
        );
    }

    fn write_provider_stage_receipt(path: &Path) {
        write_file(
            path,
            &serde_json::to_vec(&json!({
                "topology_execution": {
                    "execution_status": "success",
                    "unit_executions": [
                        { "unit_id": "unit-a", "execution_status": "success" },
                        { "unit_id": "unit-b", "execution_status": "success" }
                    ]
                }
            }))
            .unwrap(),
        );
    }

    fn provider_stage_json(stage_name: &str, binary_digest: &str, policy_digest: &str) -> serde_json::Value {
        let receipt = format!("{stage_name}/receipt.json");
        let binary = format!("{stage_name}/mantle");
        json!({
            "name": stage_name,
            "dir": stage_name,
            "execution_dir": "execution",
            "receipt": receipt,
            "stderr": format!("{stage_name}/stderr.txt"),
            "status": format!("{stage_name}/status.txt"),
            "status_code": 0,
            "execution_status": "success",
            "cargo_marker_absent": true,
            "success": true,
            "unit_count": PROVIDER_STAGE_UNIT_COUNT,
            "failed_unit_count": 0,
            "binary": binary,
            "binary_blake3": binary_digest,
            "smoke_status_code": 0,
            "source_built_toolchain_closure_policy_digest_blake3": policy_digest,
            "blocker": null
        })
    }

    fn provider_preflight_json(policy_digest: &str) -> serde_json::Value {
        json!({
            "schema": PROVIDER_FIXED_POINT_SCHEMA,
            "root": "/repo/mantle",
            "bundle_dir": "/tmp/provider-fixed-point-proof",
            "source_built_toolchain_closure": {
                "status": "enforced-source-built",
                "claim": true,
                "policy_digest_blake3": policy_digest
            }
        })
    }

    fn provider_meta_json(bundle_dir: &Path, binary_digest: &str, policy_digest: &str) -> serde_json::Value {
        json!({
            "schema": PROVIDER_FIXED_POINT_SCHEMA,
            "status": "success",
            "root": "/repo/mantle",
            "bundle_dir": bundle_dir,
            "fixed_point": true,
            "hermeticity_mode": crunch_release_core::STRICT_HERMETICITY_MODE,
            "stage1": provider_stage_json("stage1", binary_digest, policy_digest),
            "stage2": provider_stage_json("stage2", binary_digest, policy_digest),
            "source_built_toolchain_closure": {
                "schema": "mantle-source-built-toolchain-closure-v1",
                "status": "enforced-source-built",
                "claim": true,
                "non_claim": null,
                "manifest_path": "/tmp/toolchain-closure.json",
                "policy_digest_blake3": policy_digest,
                "member_count": PROVIDER_STAGE_UNIT_COUNT,
                "source_built_member_count": PROVIDER_STAGE_UNIT_COUNT,
                "seed_exception_count": 0
            },
            "rust_source_provider": {
                "schema": RUST_SOURCE_PROVIDER_BINDING_SCHEMA,
                "status": "validated",
                "provider_dir": "/provider",
                "metadata_path": "/provider/share/mantle-rust-provider/provider.json",
                "metadata_digest_blake3": sample_digest(13),
                "policy_digest_blake3": policy_digest,
                "host_triple": "x86_64-unknown-linux-musl",
                "target_triple": "x86_64-unknown-linux-musl",
                "artifact_count": 6,
                "source_count": 2,
                "receipt_count": 1,
                "rustc_path": "/provider/bin/rustc"
            },
            "blocker": null,
            "non_claims": [
                "not-crunch-bootstrap",
                "not-release-reproducibility",
                "not-full-cargo-compatibility"
            ]
        })
    }

    #[test]
    fn canonical_bytes_are_stable_and_compact() {
        let manifest = sample_manifest();
        let first = canonical_release_evidence_manifest(manifest.clone()).unwrap();
        let second = canonical_release_evidence_manifest(manifest.clone()).unwrap();
        assert_eq!(first, second);
        assert!(!first.contains(&b'\n'));
    }

    #[test]
    fn validate_rejects_absolute_member_path() {
        let mut manifest = sample_manifest();
        manifest.source_archive.relative_path = "/tmp/source.tar".to_string();
        let err = canonical_release_evidence_manifest(manifest).unwrap_err();
        assert!(err.to_string().contains("must be relative"));
    }

    #[test]
    fn validate_rejects_prerequisite_inventory_linkage_mismatch() {
        let mut manifest = sample_manifest();
        manifest.proof_linkage.prerequisite_inventory_digest_blake3 = sample_digest(8);
        let err = canonical_release_evidence_manifest(manifest).unwrap_err();
        assert!(err.to_string().contains("prerequisite inventory digest does not match"));
    }

    #[test]
    fn validate_rejects_stage2_digest_not_present_in_binaries() {
        let mut manifest = sample_manifest();
        manifest.proof_linkage.stage2_binary_digest_blake3 = sample_digest(4);
        let err = canonical_release_evidence_manifest(manifest).unwrap_err();
        assert!(err.to_string().contains("does not match any bundled binary artifact"));
    }

    #[test]
    fn load_full_self_hosting_proof_identity_accepts_full_proof_bundle() {
        let dir = tempfile::tempdir().unwrap();
        write_full_proof_manifest(dir.path(), &sample_digest(9), &sample_digest(11));
        let identity = load_full_self_hosting_proof_identity(dir.path()).unwrap();
        assert_eq!(identity.schema, FULL_SELF_HOSTING_PROOF_SCHEMA);
        assert_eq!(identity.proof_mode, "fixed-point");
        assert_eq!(identity.staged_source, "/tmp/proof-store/abcd-mantle-src");
        assert_eq!(identity.stage2_binary_digest_blake3, sample_digest(11));
        assert_eq!(identity.prerequisite_inventory_digest_blake3, sample_digest(9));
        assert_eq!(identity.proof_manifest_digest_blake3.len(), BLAKE3_HEX_LEN);
    }

    #[test]
    fn load_full_self_hosting_proof_identity_rejects_prerequisite_only_artifact() {
        let dir = tempfile::tempdir().unwrap();
        write_file(&dir.path().join("manifest.json"), br#"{"schema":"fake-proof"}"#);
        let err = load_full_self_hosting_proof_identity(dir.path()).unwrap_err();
        assert!(err.to_string().contains("full proof artifact required"));
    }

    #[test]
    fn create_and_verify_release_bundle_round_trip() {
        let temp = tempfile::tempdir().unwrap();
        let source_archive = temp.path().join("mantle-src.tar");
        let binary_path = temp.path().join("mantle");
        let proof_bundle_dir = temp.path().join("proof-input");
        let output_bundle_dir = temp.path().join("release-bundle");

        write_file(&source_archive, b"source-archive");
        write_file(&binary_path, b"mantle-binary");
        let inventory_digest = blake3::hash(b"inventory").to_hex().to_string();
        let stage2_digest = blake3::hash(b"mantle-binary").to_hex().to_string();
        write_full_proof_manifest(&proof_bundle_dir, &inventory_digest, &stage2_digest);

        let request = ReleaseBundleCreateRequest::with_defaults(
            "mantle-0.1.0-rc1".to_string(),
            output_bundle_dir.clone(),
            source_archive.clone(),
            vec![binary_path.clone()],
            proof_bundle_dir.clone(),
        );
        let created = create_release_evidence_bundle(&request).unwrap();
        let verified = verify_release_evidence_bundle(&output_bundle_dir).unwrap();

        assert_eq!(created.release_id, "mantle-0.1.0-rc1");
        assert_eq!(created.proof_linkage.selected_provider_kind, "source-root");
        assert_eq!(created, verified);
        assert!(output_bundle_dir.join("proof/self-hosting/manifest.json").exists());
        assert!(output_bundle_dir.join("proof/inventory.md").exists());
        assert!(output_bundle_dir.join("manifest.json").exists());
    }

    #[test]
    fn create_and_verify_release_bundle_records_external_evidence() {
        let temp = tempfile::tempdir().unwrap();
        let source_archive = temp.path().join("mantle-src.tar");
        let binary_path = temp.path().join("mantle");
        let proof_bundle_dir = temp.path().join("proof-input");
        let external_evidence_path = temp.path().join("stack-provenance.json");
        let output_bundle_dir = temp.path().join("release-bundle");

        write_file(&source_archive, b"source-archive");
        write_file(&binary_path, b"mantle-binary");
        write_file(&external_evidence_path, br#"{"schema":"valence.stack-provenance-adapter.v1"}"#);
        let inventory_digest = blake3::hash(b"inventory").to_hex().to_string();
        let stage2_digest = blake3::hash(b"mantle-binary").to_hex().to_string();
        write_full_proof_manifest(&proof_bundle_dir, &inventory_digest, &stage2_digest);

        let mut request = ReleaseBundleCreateRequest::with_defaults(
            "mantle-0.1.0-rc1".to_string(),
            output_bundle_dir.clone(),
            source_archive,
            vec![binary_path],
            proof_bundle_dir,
        );
        request.external_evidence = vec![ExternalEvidenceCreateRequest {
            path: external_evidence_path,
            role: "stack-provenance-trace".to_string(),
            schema: "valence.stack-provenance-adapter.v1".to_string(),
            claim_scope: "identity-linkage-sidecar".to_string(),
            non_claims: vec!["not semantic validation by Mantle".to_string()],
        }];
        let created = create_release_evidence_bundle(&request).unwrap();
        let verified = verify_release_evidence_bundle(&output_bundle_dir).unwrap();
        let evidence = created.external_evidence.first().expect("external evidence recorded");

        assert_eq!(created, verified);
        assert_eq!(evidence.role, "stack-provenance-trace");
        assert_eq!(evidence.schema, "valence.stack-provenance-adapter.v1");
        assert_eq!(evidence.claim_scope, "identity-linkage-sidecar");
        assert!(output_bundle_dir.join(&evidence.relative_path).is_file());
    }

    #[test]
    fn create_and_verify_release_bundle_records_stack_provenance_evidence() {
        let temp = tempfile::tempdir().unwrap();
        let source_archive = temp.path().join("mantle-src.tar");
        let binary_path = temp.path().join("mantle");
        let proof_bundle_dir = temp.path().join("proof-input");
        let stack_sidecar_path = temp.path().join("stack-provenance-sidecar.json");
        let valence_receipt_path = temp.path().join("valence-stack-provenance-graph-report.json");
        let output_bundle_dir = temp.path().join("release-bundle");

        write_file(&source_archive, b"source-archive");
        write_file(&binary_path, b"mantle-binary");
        write_file(&stack_sidecar_path, br#"{"schema":"valence.stack-provenance-sidecar.v1"}"#);
        write_file(&valence_receipt_path, br#"{"schema":"valence.stack-provenance-graph-report.v1","valid":true}"#);
        let inventory_digest = blake3::hash(b"inventory").to_hex().to_string();
        let stage2_digest = blake3::hash(b"mantle-binary").to_hex().to_string();
        write_full_proof_manifest(&proof_bundle_dir, &inventory_digest, &stage2_digest);

        let mut request = ReleaseBundleCreateRequest::with_defaults(
            "mantle-0.1.0-rc1".to_string(),
            output_bundle_dir.clone(),
            source_archive,
            vec![binary_path],
            proof_bundle_dir,
        );
        request.stack_provenance = Some(StackProvenanceCreateRequest {
            sidecar_path: stack_sidecar_path,
            valence_receipt_path,
            binary_path: None,
        });

        let created = create_release_evidence_bundle(&request).unwrap();
        let verified = verify_release_evidence_bundle(&output_bundle_dir).unwrap();
        let stack_provenance = created.stack_provenance.as_ref().expect("stack provenance recorded");
        let policy_result = crunch_release_core::evaluate_stack_provenance_release_evidence(
            &created,
            crunch_release_core::STACK_PROVENANCE_MODE_REQUIRED,
        );

        assert_eq!(created, verified);
        assert!(policy_result.valid);
        assert_eq!(stack_provenance.sidecar_role, STACK_PROVENANCE_EVIDENCE_ROLE);
        assert_eq!(stack_provenance.valence_receipt_role, VALENCE_STACK_PROVENANCE_RECEIPT_ROLE);
        assert_eq!(stack_provenance.sidecar_claim_scope, STACK_PROVENANCE_CLAIM_SCOPE);
        assert!(stack_provenance.non_claims.contains(&STACK_PROVENANCE_OPAQUE_BOUNDARY.to_string()));
        assert!(output_bundle_dir.join(&stack_provenance.sidecar_relative_path).is_file());
        assert!(output_bundle_dir.join(&stack_provenance.valence_receipt_relative_path).is_file());
    }

    #[test]
    fn create_release_bundle_rejects_stack_provenance_without_binary_selection_for_multiple_binaries() {
        let temp = tempfile::tempdir().unwrap();
        let source_archive = temp.path().join("mantle-src.tar");
        let binary_path = temp.path().join("mantle");
        let other_binary_path = temp.path().join("mantle-helper");
        let proof_bundle_dir = temp.path().join("proof-input");
        let stack_sidecar_path = temp.path().join("stack-provenance-sidecar.json");
        let valence_receipt_path = temp.path().join("valence-stack-provenance-graph-report.json");
        let output_bundle_dir = temp.path().join("release-bundle");

        write_file(&source_archive, b"source-archive");
        write_file(&binary_path, b"mantle-binary");
        write_file(&other_binary_path, b"mantle-helper-binary");
        write_file(&stack_sidecar_path, br#"{"schema":"valence.stack-provenance-sidecar.v1"}"#);
        write_file(&valence_receipt_path, br#"{"schema":"valence.stack-provenance-graph-report.v1","valid":true}"#);
        let inventory_digest = blake3::hash(b"inventory").to_hex().to_string();
        let stage2_digest = blake3::hash(b"mantle-binary").to_hex().to_string();
        write_full_proof_manifest(&proof_bundle_dir, &inventory_digest, &stage2_digest);

        let mut request = ReleaseBundleCreateRequest::with_defaults(
            "mantle-0.1.0-rc1".to_string(),
            output_bundle_dir,
            source_archive,
            vec![binary_path, other_binary_path],
            proof_bundle_dir,
        );
        request.stack_provenance = Some(StackProvenanceCreateRequest {
            sidecar_path: stack_sidecar_path,
            valence_receipt_path,
            binary_path: None,
        });

        let err = create_release_evidence_bundle(&request).unwrap_err();

        assert!(err.to_string().contains("must be selected when multiple binaries"));
    }

    #[test]
    fn create_and_verify_release_bundle_records_kani_toolchain_evidence() {
        let temp = tempfile::tempdir().unwrap();
        let source_archive = temp.path().join("mantle-src.tar");
        let binary_path = temp.path().join("mantle");
        let proof_bundle_dir = temp.path().join("proof-input");
        let kani_receipt_path = temp.path().join("kani-receipt.json");
        let output_bundle_dir = temp.path().join("release-bundle");

        write_file(&source_archive, b"source-archive");
        write_file(&binary_path, b"mantle-binary");
        write_file(&kani_receipt_path, br#"{"schema_version":"cairn.kani_receipt.v1"}"#);
        let inventory_digest = blake3::hash(b"inventory").to_hex().to_string();
        let stage2_digest = blake3::hash(b"mantle-binary").to_hex().to_string();
        write_full_proof_manifest(&proof_bundle_dir, &inventory_digest, &stage2_digest);

        let mut request = ReleaseBundleCreateRequest::with_defaults(
            "mantle-0.1.0-rc1".to_string(),
            output_bundle_dir.clone(),
            source_archive,
            vec![binary_path],
            proof_bundle_dir,
        );
        request.external_evidence = vec![ExternalEvidenceCreateRequest {
            path: kani_receipt_path,
            role: KANI_RECEIPT_EVIDENCE_ROLE.to_string(),
            schema: "cairn.kani_receipt.v1".to_string(),
            claim_scope: KANI_EVIDENCE_CLAIM_SCOPE.to_string(),
            non_claims: sample_kani_non_claims(),
        }];
        request.kani_toolchain_evidence = vec![sample_kani_toolchain_request(KANI_RECEIPT_EVIDENCE_ROLE)];

        let created = create_release_evidence_bundle(&request).unwrap();
        let verified = verify_release_evidence_bundle(&output_bundle_dir).unwrap();
        let evidence = created.kani_toolchain_evidence.first().expect("Kani toolchain evidence recorded");

        assert_eq!(created, verified);
        assert_eq!(evidence.receipt_role, KANI_RECEIPT_EVIDENCE_ROLE);
        assert_eq!(evidence.valence_semantic_role, KANI_VALENCE_SEMANTIC_ROLE);
        assert_eq!(evidence.claim_scope, KANI_EVIDENCE_CLAIM_SCOPE);
    }

    #[test]
    fn create_release_bundle_rejects_kani_metadata_without_receipt() {
        let temp = tempfile::tempdir().unwrap();
        let source_archive = temp.path().join("mantle-src.tar");
        let binary_path = temp.path().join("mantle");
        let proof_bundle_dir = temp.path().join("proof-input");
        let output_bundle_dir = temp.path().join("release-bundle");

        write_file(&source_archive, b"source-archive");
        write_file(&binary_path, b"mantle-binary");
        let inventory_digest = blake3::hash(b"inventory").to_hex().to_string();
        let stage2_digest = blake3::hash(b"mantle-binary").to_hex().to_string();
        write_full_proof_manifest(&proof_bundle_dir, &inventory_digest, &stage2_digest);

        let mut request = ReleaseBundleCreateRequest::with_defaults(
            "mantle-0.1.0-rc1".to_string(),
            output_bundle_dir,
            source_archive,
            vec![binary_path],
            proof_bundle_dir,
        );
        request.kani_toolchain_evidence = vec![sample_kani_toolchain_request(KANI_RECEIPT_EVIDENCE_ROLE)];

        let err = create_release_evidence_bundle(&request).unwrap_err();

        assert!(err.to_string().contains("Kani receipt role required but missing"));
    }

    #[test]
    fn verify_release_bundle_rejects_tampered_external_evidence() {
        let temp = tempfile::tempdir().unwrap();
        let source_archive = temp.path().join("mantle-src.tar");
        let binary_path = temp.path().join("mantle");
        let proof_bundle_dir = temp.path().join("proof-input");
        let external_evidence_path = temp.path().join("stack-provenance.json");
        let output_bundle_dir = temp.path().join("release-bundle");

        write_file(&source_archive, b"source-archive");
        write_file(&binary_path, b"mantle-binary");
        write_file(&external_evidence_path, br#"{"schema":"valence.stack-provenance-adapter.v1"}"#);
        let inventory_digest = blake3::hash(b"inventory").to_hex().to_string();
        let stage2_digest = blake3::hash(b"mantle-binary").to_hex().to_string();
        write_full_proof_manifest(&proof_bundle_dir, &inventory_digest, &stage2_digest);

        let mut request = ReleaseBundleCreateRequest::with_defaults(
            "mantle-0.1.0-rc1".to_string(),
            output_bundle_dir.clone(),
            source_archive,
            vec![binary_path],
            proof_bundle_dir,
        );
        request.external_evidence = vec![ExternalEvidenceCreateRequest {
            path: external_evidence_path,
            role: "stack-provenance-trace".to_string(),
            schema: "valence.stack-provenance-adapter.v1".to_string(),
            claim_scope: "identity-linkage-sidecar".to_string(),
            non_claims: vec!["not semantic validation by Mantle".to_string()],
        }];
        let created = create_release_evidence_bundle(&request).unwrap();
        let evidence = created.external_evidence.first().expect("external evidence recorded");
        write_file(&output_bundle_dir.join(&evidence.relative_path), b"tampered");

        let err = verify_release_evidence_bundle(&output_bundle_dir).unwrap_err();
        assert!(err.to_string().contains("external_evidence[0] does not match manifest"));
    }

    #[test]
    fn create_and_verify_release_bundle_records_source_acquisition_url() {
        let temp = tempfile::tempdir().unwrap();
        let source_archive = temp.path().join("mantle-src.tar");
        let binary_path = temp.path().join("mantle");
        let proof_bundle_dir = temp.path().join("proof-input");
        let output_bundle_dir = temp.path().join("release-bundle");

        write_file(&source_archive, b"source-archive");
        write_file(&binary_path, b"mantle-binary");
        let inventory_digest = blake3::hash(b"inventory").to_hex().to_string();
        let stage2_digest = blake3::hash(b"mantle-binary").to_hex().to_string();
        write_full_proof_manifest(&proof_bundle_dir, &inventory_digest, &stage2_digest);
        let source_url = "https://example.invalid/releases/mantle-src.tar".to_string();

        let mut request = ReleaseBundleCreateRequest::with_defaults(
            "mantle-0.1.0-rc1".to_string(),
            output_bundle_dir.clone(),
            source_archive,
            vec![binary_path],
            proof_bundle_dir,
        );
        request.source_acquisition_url = Some(source_url.clone());
        let created = create_release_evidence_bundle(&request).unwrap();
        let verified = verify_release_evidence_bundle(&output_bundle_dir).unwrap();
        let source_acquisition = created.source_acquisition.as_ref().unwrap();

        assert_eq!(created, verified);
        assert_eq!(source_acquisition.kind, SOURCE_ACQUISITION_KIND_EXTERNAL_ARCHIVE);
        assert_eq!(source_acquisition.url, source_url);
        assert_eq!(source_acquisition.digest_blake3, created.source_archive.digest_blake3);
    }

    #[test]
    fn create_and_verify_release_bundle_records_git_source_acquisition() {
        let temp = tempfile::tempdir().unwrap();
        let source_archive = temp.path().join("mantle-src.tar");
        let binary_path = temp.path().join("mantle");
        let proof_bundle_dir = temp.path().join("proof-input");
        let output_bundle_dir = temp.path().join("release-bundle");

        write_file(&source_archive, b"source-archive");
        write_file(&binary_path, b"mantle-binary");
        let inventory_digest = blake3::hash(b"inventory").to_hex().to_string();
        let stage2_digest = blake3::hash(b"mantle-binary").to_hex().to_string();
        write_full_proof_manifest(&proof_bundle_dir, &inventory_digest, &stage2_digest);
        let git_url = "file:///tmp/mantle-origin.git".to_string();
        let commit = "a".repeat(BLAKE3_HEX_LEN);

        let mut request = ReleaseBundleCreateRequest::with_defaults(
            "mantle-0.1.0-rc1".to_string(),
            output_bundle_dir.clone(),
            source_archive,
            vec![binary_path],
            proof_bundle_dir,
        );
        request.git_source = Some(GitSourceCreateRequest {
            remote_url: git_url.clone(),
            commit: commit.clone(),
            reference: Some("refs/heads/main".to_string()),
            tag: None,
        });
        let created = create_release_evidence_bundle(&request).unwrap();
        let verified = verify_release_evidence_bundle(&output_bundle_dir).unwrap();
        let source_acquisition = created.source_acquisition.as_ref().unwrap();

        assert_eq!(created, verified);
        assert_eq!(source_acquisition.kind, SOURCE_ACQUISITION_KIND_GIT);
        assert_eq!(source_acquisition.url, git_url);
        assert_eq!(source_acquisition.commit.as_deref(), Some(commit.as_str()));
        assert_eq!(source_acquisition.reference.as_deref(), Some("refs/heads/main"));
        assert_eq!(source_acquisition.archive_profile.as_deref(), Some(RELEASE_SOURCE_ARCHIVE_PROFILE));
        assert_eq!(source_acquisition.archive_version.as_deref(), Some(RELEASE_SOURCE_ARCHIVE_VERSION));
        assert_eq!(source_acquisition.digest_blake3, created.source_archive.digest_blake3);
    }

    #[test]
    fn create_rejects_conflicting_source_acquisition_modes_before_manifest() {
        let temp = tempfile::tempdir().unwrap();
        let source_archive = temp.path().join("mantle-src.tar");
        let binary_path = temp.path().join("mantle");
        let proof_bundle_dir = temp.path().join("proof-input");
        let output_bundle_dir = temp.path().join("release-bundle");

        write_file(&source_archive, b"source-archive");
        write_file(&binary_path, b"mantle-binary");
        let inventory_digest = blake3::hash(b"inventory").to_hex().to_string();
        let stage2_digest = blake3::hash(b"mantle-binary").to_hex().to_string();
        write_full_proof_manifest(&proof_bundle_dir, &inventory_digest, &stage2_digest);
        let mut request = ReleaseBundleCreateRequest::with_defaults(
            "mantle-0.1.0-rc1".to_string(),
            output_bundle_dir.clone(),
            source_archive,
            vec![binary_path],
            proof_bundle_dir,
        );
        request.source_acquisition_url = Some("https://example.invalid/source.tar".to_string());
        request.git_source = Some(GitSourceCreateRequest {
            remote_url: "file:///tmp/mantle-origin.git".to_string(),
            commit: "a".repeat(BLAKE3_HEX_LEN),
            reference: None,
            tag: None,
        });

        let err = create_release_evidence_bundle(&request).unwrap_err();

        assert!(err.to_string().contains("conflicts with external source acquisition URL"));
        assert!(!output_bundle_dir.join("manifest.json").exists());
    }

    #[test]
    fn create_rejects_empty_git_source_commit_before_manifest() {
        let temp = tempfile::tempdir().unwrap();
        let source_archive = temp.path().join("mantle-src.tar");
        let binary_path = temp.path().join("mantle");
        let proof_bundle_dir = temp.path().join("proof-input");
        let output_bundle_dir = temp.path().join("release-bundle");

        write_file(&source_archive, b"source-archive");
        write_file(&binary_path, b"mantle-binary");
        let inventory_digest = blake3::hash(b"inventory").to_hex().to_string();
        let stage2_digest = blake3::hash(b"mantle-binary").to_hex().to_string();
        write_full_proof_manifest(&proof_bundle_dir, &inventory_digest, &stage2_digest);
        let mut request = ReleaseBundleCreateRequest::with_defaults(
            "mantle-0.1.0-rc1".to_string(),
            output_bundle_dir.clone(),
            source_archive,
            vec![binary_path],
            proof_bundle_dir,
        );
        request.git_source = Some(GitSourceCreateRequest {
            remote_url: "file:///tmp/mantle-origin.git".to_string(),
            commit: " ".to_string(),
            reference: None,
            tag: None,
        });

        let err = create_release_evidence_bundle(&request).unwrap_err();

        assert!(err.to_string().contains("Git source commit must not be empty"));
        assert!(!output_bundle_dir.join("manifest.json").exists());
    }

    #[test]
    fn create_rejects_empty_source_acquisition_url_before_manifest() {
        let temp = tempfile::tempdir().unwrap();
        let source_archive = temp.path().join("mantle-src.tar");
        let binary_path = temp.path().join("mantle");
        let proof_bundle_dir = temp.path().join("proof-input");
        let output_bundle_dir = temp.path().join("release-bundle");

        write_file(&source_archive, b"source-archive");
        write_file(&binary_path, b"mantle-binary");
        let inventory_digest = blake3::hash(b"inventory").to_hex().to_string();
        let stage2_digest = blake3::hash(b"mantle-binary").to_hex().to_string();
        write_full_proof_manifest(&proof_bundle_dir, &inventory_digest, &stage2_digest);

        let mut request = ReleaseBundleCreateRequest::with_defaults(
            "mantle-0.1.0-rc1".to_string(),
            output_bundle_dir.clone(),
            source_archive,
            vec![binary_path],
            proof_bundle_dir,
        );
        request.source_acquisition_url = Some("   ".to_string());
        let err = create_release_evidence_bundle(&request).unwrap_err();

        assert!(err.to_string().contains("source acquisition URL must not be empty"));
        assert!(!output_bundle_dir.join("manifest.json").exists());
    }

    #[test]
    fn create_and_verify_release_bundle_with_provider_fixed_point_proof() {
        let temp = tempfile::tempdir().unwrap();
        let source_archive = temp.path().join("mantle-src.tar");
        let binary_path = temp.path().join("mantle");
        let proof_bundle_dir = temp.path().join("proof-input");
        let provider_proof_dir = temp.path().join("provider-fixed-point-input");
        let output_bundle_dir = temp.path().join("release-bundle");

        write_file(&source_archive, b"source-archive");
        write_file(&binary_path, PROVIDER_BINARY_BYTES);
        let inventory_digest = blake3::hash(b"inventory").to_hex().to_string();
        let stage2_digest = blake3::hash(PROVIDER_BINARY_BYTES).to_hex().to_string();
        write_full_proof_manifest(&proof_bundle_dir, &inventory_digest, &stage2_digest);
        write_provider_fixed_point_proof_bundle(&provider_proof_dir, PROVIDER_BINARY_BYTES);

        let mut request = ReleaseBundleCreateRequest::with_defaults(
            "mantle-0.1.0-rc1".to_string(),
            output_bundle_dir.clone(),
            source_archive,
            vec![binary_path],
            proof_bundle_dir,
        );
        request.provider_fixed_point_proof_dir = Some(provider_proof_dir);
        let created = create_release_evidence_bundle(&request).unwrap();
        let verified = verify_release_evidence_bundle(&output_bundle_dir).unwrap();
        let provider_artifact = created.provider_fixed_point_proof.as_ref().unwrap();

        assert_eq!(created, verified);
        assert_eq!(provider_artifact.relative_path, "proof/provider-fixed-point");
        assert_eq!(provider_artifact.evidence_role, PROVIDER_FIXED_POINT_PROOF_EVIDENCE_ROLE);
        assert!(output_bundle_dir.join("proof/provider-fixed-point/meta.json").exists());
    }

    #[test]
    fn create_rejects_provider_fixed_point_proof_for_different_binary() {
        let temp = tempfile::tempdir().unwrap();
        let source_archive = temp.path().join("mantle-src.tar");
        let binary_path = temp.path().join("mantle");
        let proof_bundle_dir = temp.path().join("proof-input");
        let provider_proof_dir = temp.path().join("provider-fixed-point-input");
        let output_bundle_dir = temp.path().join("release-bundle");

        write_file(&source_archive, b"source-archive");
        write_file(&binary_path, b"legacy-release-binary");
        let inventory_digest = blake3::hash(b"inventory").to_hex().to_string();
        let stage2_digest = blake3::hash(b"legacy-release-binary").to_hex().to_string();
        write_full_proof_manifest(&proof_bundle_dir, &inventory_digest, &stage2_digest);
        write_provider_fixed_point_proof_bundle(&provider_proof_dir, PROVIDER_BINARY_BYTES);

        let mut request = ReleaseBundleCreateRequest::with_defaults(
            "mantle-0.1.0-rc1".to_string(),
            output_bundle_dir.clone(),
            source_archive,
            vec![binary_path],
            proof_bundle_dir,
        );
        request.provider_fixed_point_proof_dir = Some(provider_proof_dir);
        let err = create_release_evidence_bundle(&request).unwrap_err();

        assert!(err.to_string().contains("does not match any bundled release binary artifact"));
        assert!(!output_bundle_dir.join("manifest.json").exists());
    }

    #[test]
    fn create_rejects_invalid_provider_fixed_point_proof_before_manifest() {
        let temp = tempfile::tempdir().unwrap();
        let source_archive = temp.path().join("mantle-src.tar");
        let binary_path = temp.path().join("mantle");
        let proof_bundle_dir = temp.path().join("proof-input");
        let provider_proof_dir = temp.path().join("invalid-provider-fixed-point");
        let output_bundle_dir = temp.path().join("release-bundle");

        write_file(&source_archive, b"source-archive");
        write_file(&binary_path, b"mantle-binary");
        let inventory_digest = blake3::hash(b"inventory").to_hex().to_string();
        let stage2_digest = blake3::hash(b"mantle-binary").to_hex().to_string();
        write_full_proof_manifest(&proof_bundle_dir, &inventory_digest, &stage2_digest);
        write_file(&provider_proof_dir.join("meta.json"), br#"{}"#);

        let mut request = ReleaseBundleCreateRequest::with_defaults(
            "mantle-0.1.0-rc1".to_string(),
            output_bundle_dir.clone(),
            source_archive,
            vec![binary_path],
            proof_bundle_dir,
        );
        request.provider_fixed_point_proof_dir = Some(provider_proof_dir);
        let err = create_release_evidence_bundle(&request).unwrap_err();

        assert!(err.to_string().contains("provider fixed-point proof is invalid"));
        assert!(!output_bundle_dir.join("manifest.json").exists());
    }

    #[test]
    fn verify_rejects_provider_kind_linkage_mismatch() {
        let temp = tempfile::tempdir().unwrap();
        let source_archive = temp.path().join("mantle-src.tar");
        let binary_path = temp.path().join("mantle");
        let proof_bundle_dir = temp.path().join("proof-input");
        let output_bundle_dir = temp.path().join("release-bundle");

        write_file(&source_archive, b"source");
        write_file(&binary_path, b"mantle-binary");
        let inventory_digest = blake3::hash(b"inventory").to_hex().to_string();
        let stage2_digest = blake3::hash(b"mantle-binary").to_hex().to_string();
        write_full_proof_manifest(&proof_bundle_dir, &inventory_digest, &stage2_digest);

        let request = ReleaseBundleCreateRequest::with_defaults(
            "mantle-0.1.0-rc1".to_string(),
            output_bundle_dir.clone(),
            source_archive,
            vec![binary_path],
            proof_bundle_dir,
        );
        let mut created = create_release_evidence_bundle(&request).unwrap();
        created.proof_linkage.selected_provider_kind = "stagex-lineage".to_string();
        write_manifest_file(&output_bundle_dir, &created).unwrap();

        let err = verify_release_evidence_bundle(&output_bundle_dir).unwrap_err();
        assert!(err.to_string().contains("selected_provider_kind mismatch"));
    }

    #[test]
    fn verify_rejects_non_canonical_manifest_json() {
        let temp = tempfile::tempdir().unwrap();
        let source_archive = temp.path().join("mantle-src.tar");
        let binary_path = temp.path().join("mantle");
        let proof_bundle_dir = temp.path().join("proof-input");
        let output_bundle_dir = temp.path().join("release-bundle");

        write_file(&source_archive, b"source-archive");
        write_file(&binary_path, b"mantle-binary");
        let inventory_digest = blake3::hash(b"inventory").to_hex().to_string();
        let stage2_digest = blake3::hash(b"mantle-binary").to_hex().to_string();
        write_full_proof_manifest(&proof_bundle_dir, &inventory_digest, &stage2_digest);

        let request = ReleaseBundleCreateRequest::with_defaults(
            "mantle-0.1.0-rc1".to_string(),
            output_bundle_dir.clone(),
            source_archive,
            vec![binary_path],
            proof_bundle_dir,
        );
        let created = create_release_evidence_bundle(&request).unwrap();
        let pretty_json = serde_json::to_string_pretty(&created).unwrap();
        write_file(&output_bundle_dir.join("manifest.json"), pretty_json.as_bytes());

        let err = verify_release_evidence_bundle(&output_bundle_dir).unwrap_err();
        assert!(err.to_string().contains("not canonical compact JSON"));
    }

    #[test]
    fn verify_rejects_tampered_binary_artifact() {
        let temp = tempfile::tempdir().unwrap();
        let source_archive = temp.path().join("mantle-src.tar");
        let binary_path = temp.path().join("mantle");
        let proof_bundle_dir = temp.path().join("proof-input");
        let output_bundle_dir = temp.path().join("release-bundle");

        write_file(&source_archive, b"source-archive");
        write_file(&binary_path, b"mantle-binary");
        let inventory_digest = blake3::hash(b"inventory").to_hex().to_string();
        let stage2_digest = blake3::hash(b"mantle-binary").to_hex().to_string();
        write_full_proof_manifest(&proof_bundle_dir, &inventory_digest, &stage2_digest);

        let request = ReleaseBundleCreateRequest::with_defaults(
            "mantle-0.1.0-rc1".to_string(),
            output_bundle_dir.clone(),
            source_archive,
            vec![binary_path],
            proof_bundle_dir,
        );
        create_release_evidence_bundle(&request).unwrap();
        write_file(&output_bundle_dir.join("binaries/01-mantle"), b"tampered-binary");

        let err = verify_release_evidence_bundle(&output_bundle_dir).unwrap_err();
        assert!(err.to_string().contains("binaries[0] does not match manifest"));
    }
}
