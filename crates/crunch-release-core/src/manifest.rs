use alloc::collections::BTreeSet;
use alloc::format;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;

use serde::Deserialize;
use serde::Serialize;

use crate::ReleaseEvidenceError;

pub const RELEASE_EVIDENCE_SCHEMA: &str = "mantle-release-evidence-v1";
pub const FULL_SELF_HOSTING_PROOF_SCHEMA: &str = "mantle-self-hosting-proof-v2";
pub const CLAIM_SCOPE_PACKAGED_INTEGRITY: &str = "packaged-integrity-evidence";
pub const DEFAULT_PROOF_WORKFLOW_COMMAND: &str = "./scripts/prove-self-hosting.sh";
pub const DEFAULT_PROOF_WORKFLOW_VERSION: &str = "mantle-self-hosting-proof-v2";
pub const PROVIDER_FIXED_POINT_PROOF_EVIDENCE_ROLE: &str = "cargo-free-source-built-handoff-evidence";
pub const DETERMINISTIC_BUILD_PROOF_EVIDENCE_ROLE: &str = "deterministic-build-proof-receipt";
pub const DETERMINISTIC_SANDBOX_ISOLATION_EVIDENCE_ROLE: &str = "deterministic-sandbox-isolation-evidence";
pub const SOURCE_ACQUISITION_KIND_EXTERNAL_ARCHIVE: &str = "external-archive";
pub const BLAKE3_HEX_LENGTH_CHARS: usize = 64;

const MAX_BINARY_ARTIFACTS_COUNT: u32 = 16;
const MAX_RELATIVE_PATH_BYTES_COUNT: u32 = 4096;
const MAX_SOURCE_ACQUISITION_URL_BYTES_COUNT: u32 = 8192;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum BundledArtifactKind {
    File,
    Directory,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BundledArtifact {
    pub kind: BundledArtifactKind,
    pub relative_path: String,
    pub size_bytes: u64,
    pub digest_blake3: String,
}

pub fn validate_bundled_artifact_record(
    artifact: BundledArtifact,
    field_name: String,
) -> Result<BundledArtifact, ReleaseEvidenceError> {
    validate_bundled_artifact(&artifact, &field_name)?;
    Ok(artifact)
}

fn validate_bundled_artifact(artifact: &BundledArtifact, field_name: &str) -> Result<(), ReleaseEvidenceError> {
    validate_relative_member_path(&artifact.relative_path, field_name)?;
    if artifact.size_bytes == 0 {
        return Err(validation_error(format!("release evidence {field_name}.size_bytes must be non-zero")));
    }
    validate_blake3_hex(&artifact.digest_blake3, &format!("{field_name}.digest_blake3"))?;
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReleaseWorkflowIdentity {
    pub command: String,
    pub version: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderFixedPointProofArtifact {
    pub kind: BundledArtifactKind,
    pub relative_path: String,
    pub size_bytes: u64,
    pub digest_blake3: String,
    pub evidence_role: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderFixedPointReleaseArtifactBinding {
    pub relative_path: String,
    pub digest_blake3: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoleBoundedReleaseArtifact {
    pub kind: BundledArtifactKind,
    pub relative_path: String,
    pub size_bytes: u64,
    pub digest_blake3: String,
    pub evidence_role: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceAcquisition {
    pub kind: String,
    pub url: String,
    pub digest_blake3: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReleaseProofLinkage {
    pub release_id: String,
    pub source_archive_digest_blake3: String,
    pub proof_bundle_schema: String,
    pub proof_mode: String,
    pub selected_provider_kind: String,
    pub staged_source: String,
    pub stage2_binary_digest_blake3: String,
    pub prerequisite_inventory_digest_blake3: String,
    pub proof_manifest_digest_blake3: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReleaseEvidenceManifest {
    pub schema: String,
    pub release_id: String,
    pub claim_scope: String,
    pub workflow: ReleaseWorkflowIdentity,
    pub source_archive: BundledArtifact,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_acquisition: Option<SourceAcquisition>,
    pub binaries: Vec<BundledArtifact>,
    pub proof_bundle: BundledArtifact,
    pub prerequisite_inventory: BundledArtifact,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider_fixed_point_proof: Option<ProviderFixedPointProofArtifact>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reproducibility_report: Option<BundledArtifact>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub deterministic_build_proof: Option<RoleBoundedReleaseArtifact>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub deterministic_sandbox_isolation_evidence: Option<RoleBoundedReleaseArtifact>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub independent_agreement_report: Option<BundledArtifact>,
    pub proof_linkage: ReleaseProofLinkage,
}

pub fn canonical_release_evidence_manifest(manifest: ReleaseEvidenceManifest) -> Result<Vec<u8>, ReleaseEvidenceError> {
    validate_release_evidence_manifest(&manifest)?;
    serde_json::to_vec(&manifest).map_err(|err| parse_error(format!("serializing release evidence manifest: {err}")))
}

pub fn validate_provider_fixed_point_release_artifact_binding(
    binaries: &[BundledArtifact],
    provider_stage_binary_digest_blake3: &str,
) -> Result<ProviderFixedPointReleaseArtifactBinding, ReleaseEvidenceError> {
    validate_blake3_hex(provider_stage_binary_digest_blake3, "provider_fixed_point.stage_binary_digest_blake3")?;
    validate_provider_binding_binary_count(binaries)?;
    for (index_usize, artifact) in binaries.iter().enumerate() {
        let index_u32 = u32_count(index_usize, "release evidence binary index overflowed u32")?;
        validate_bundled_artifact(artifact, &format!("binaries[{index_u32}]"))?;
        if artifact.digest_blake3 == provider_stage_binary_digest_blake3 {
            return Ok(ProviderFixedPointReleaseArtifactBinding {
                relative_path: artifact.relative_path.clone(),
                digest_blake3: artifact.digest_blake3.clone(),
            });
        }
    }
    Err(validation_error(
        "provider fixed-point proof stage binary digest does not match any bundled release binary artifact".to_string(),
    ))
}

fn validate_provider_binding_binary_count(binaries: &[BundledArtifact]) -> Result<(), ReleaseEvidenceError> {
    let binary_count = u32_count(binaries.len(), "release evidence binary artifact count overflowed u32")?;
    if binary_count == 0 {
        return Err(validation_error(
            "provider fixed-point release artifact binding requires at least one binary artifact".to_string(),
        ));
    }
    if binary_count > MAX_BINARY_ARTIFACTS_COUNT {
        return Err(validation_error(format!(
            "release evidence records {binary_count} binary artifacts, limit is {MAX_BINARY_ARTIFACTS_COUNT}"
        )));
    }
    Ok(())
}

fn validate_release_evidence_manifest(manifest: &ReleaseEvidenceManifest) -> Result<(), ReleaseEvidenceError> {
    validate_manifest_header(manifest)?;
    validate_manifest_artifacts(manifest)?;
    validate_source_acquisition(manifest)?;
    validate_manifest_linkage(manifest)?;
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FullSelfHostingProofIdentityFields {
    pub schema: String,
    pub proof_mode: String,
    pub selected_provider_kind: String,
    pub staged_source: String,
    pub stage2_binary_digest_blake3: String,
    pub prerequisite_inventory_digest_blake3: String,
}

pub fn extract_full_self_hosting_proof_identity_fields(
    manifest_bytes: Vec<u8>,
) -> Result<FullSelfHostingProofIdentityFields, ReleaseEvidenceError> {
    let manifest: SelfHostingProofManifestView = serde_json::from_slice(&manifest_bytes).map_err(|err| {
        parse_error(format!("full proof artifact required: parsing full proof manifest failed: {err}"))
    })?;
    validate_full_proof_manifest(&manifest)?;
    Ok(FullSelfHostingProofIdentityFields {
        schema: manifest.schema,
        proof_mode: manifest.prerequisites.mode,
        selected_provider_kind: manifest.prerequisites.provider_kind,
        staged_source: manifest.staged_source,
        stage2_binary_digest_blake3: manifest.binaries.stage2.digest_blake3,
        prerequisite_inventory_digest_blake3: manifest.prerequisites.inventory_doc.digest_blake3,
    })
}

#[derive(Debug, Deserialize)]
struct SelfHostingProofManifestView {
    schema: String,
    staged_source: String,
    prerequisites: SelfHostingProofPrerequisitesView,
    binaries: SelfHostingProofBinariesView,
    tools: SelfHostingProofToolsView,
    fixed_point: SelfHostingProofFixedPointView,
    stage0: SelfHostingProofStageView,
    stage2: SelfHostingProofStageView,
}

#[derive(Debug, Deserialize)]
struct SelfHostingProofPrerequisitesView {
    mode: String,
    provider_kind: String,
    inventory_doc: SelfHostingProofHashedPath,
}

#[derive(Debug, Deserialize)]
struct SelfHostingProofBinariesView {
    stage1: SelfHostingProofHashedPath,
    stage2: SelfHostingProofHashedPath,
}

#[derive(Debug, Deserialize)]
struct SelfHostingProofToolsView {
    stage0_bwrap: SelfHostingProofHashedPath,
    stage0_busybox: SelfHostingProofHashedPath,
    stage2_bwrap: SelfHostingProofHashedPath,
    stage2_busybox: SelfHostingProofHashedPath,
}

#[derive(Debug, Deserialize)]
struct SelfHostingProofFixedPointView {
    stage1_equals_stage2: bool,
    stage0_bwrap_equals_stage2_bwrap: bool,
    stage0_busybox_equals_stage2_busybox: bool,
}

#[derive(Debug, Deserialize)]
struct SelfHostingProofStageView {
    report: SelfHostingProofReportView,
}

#[derive(Debug, Deserialize)]
struct SelfHostingProofReportView {
    staged_source: String,
    output_binary: String,
    busybox_path: Option<String>,
}

#[derive(Debug, Deserialize)]
struct SelfHostingProofHashedPath {
    path: String,
    size_bytes: u64,
    digest_blake3: String,
}

fn validate_manifest_header(manifest: &ReleaseEvidenceManifest) -> Result<(), ReleaseEvidenceError> {
    if manifest.schema != RELEASE_EVIDENCE_SCHEMA {
        return Err(validation_error(format!(
            "release evidence schema must be {RELEASE_EVIDENCE_SCHEMA}, got {}",
            manifest.schema
        )));
    }
    if manifest.release_id.trim().is_empty() {
        return Err(validation_error("release evidence release_id must not be empty".to_string()));
    }
    if manifest.claim_scope != CLAIM_SCOPE_PACKAGED_INTEGRITY {
        return Err(validation_error(format!(
            "release evidence claim_scope must be {CLAIM_SCOPE_PACKAGED_INTEGRITY}, got {}",
            manifest.claim_scope
        )));
    }
    if manifest.workflow.command.trim().is_empty() {
        return Err(validation_error("release evidence workflow.command must not be empty".to_string()));
    }
    if manifest.workflow.version.trim().is_empty() {
        return Err(validation_error("release evidence workflow.version must not be empty".to_string()));
    }
    Ok(())
}

fn validate_manifest_artifacts(manifest: &ReleaseEvidenceManifest) -> Result<(), ReleaseEvidenceError> {
    let binary_count = u32_count(manifest.binaries.len(), "release evidence binary artifact count overflowed u32")?;
    if binary_count == 0 {
        return Err(validation_error("release evidence must record at least one binary artifact".to_string()));
    }
    if binary_count > MAX_BINARY_ARTIFACTS_COUNT {
        return Err(validation_error(format!(
            "release evidence records {binary_count} binary artifacts, limit is {MAX_BINARY_ARTIFACTS_COUNT}"
        )));
    }

    let mut seen_paths = BTreeSet::new();
    validate_and_record_path(&manifest.source_archive, "source_archive", &mut seen_paths)?;
    validate_and_record_path(&manifest.proof_bundle, "proof_bundle", &mut seen_paths)?;
    validate_and_record_path(&manifest.prerequisite_inventory, "prerequisite_inventory", &mut seen_paths)?;
    if let Some(proof) = &manifest.provider_fixed_point_proof {
        validate_provider_fixed_point_proof_artifact(proof, &mut seen_paths)?;
    }
    if let Some(report) = &manifest.reproducibility_report {
        validate_and_record_path(report, "reproducibility_report", &mut seen_paths)?;
        if report.kind != BundledArtifactKind::File {
            return Err(validation_error(
                "release evidence reproducibility_report must be recorded as a file artifact".to_string(),
            ));
        }
    }
    if let Some(proof) = &manifest.deterministic_build_proof {
        validate_role_bounded_release_artifact(
            proof,
            "deterministic_build_proof",
            DETERMINISTIC_BUILD_PROOF_EVIDENCE_ROLE,
            "deterministic-release/deterministic-build-proof.json",
            &mut seen_paths,
        )?;
    }
    if let Some(evidence) = &manifest.deterministic_sandbox_isolation_evidence {
        validate_role_bounded_release_artifact(
            evidence,
            "deterministic_sandbox_isolation_evidence",
            DETERMINISTIC_SANDBOX_ISOLATION_EVIDENCE_ROLE,
            "deterministic-release/deterministic-sandbox-isolation-evidence.json",
            &mut seen_paths,
        )?;
    }
    if let Some(report) = &manifest.independent_agreement_report {
        validate_and_record_path(report, "independent_agreement_report", &mut seen_paths)?;
        if report.kind != BundledArtifactKind::File {
            return Err(validation_error(
                "release evidence independent_agreement_report must be recorded as a file artifact".to_string(),
            ));
        }
        if report.relative_path != "independent-agreement/agreement-report.json" {
            return Err(validation_error(
                "release evidence independent_agreement_report must be independent-agreement/agreement-report.json"
                    .to_string(),
            ));
        }
    }
    for (index_usize, artifact) in manifest.binaries.iter().enumerate() {
        let index_u32 = u32_count(index_usize, "release evidence binary index overflowed u32")?;
        validate_and_record_path(artifact, &format!("binaries[{index_u32}]"), &mut seen_paths)?;
    }
    Ok(())
}

fn validate_and_record_path(
    artifact: &BundledArtifact,
    field_name: &str,
    seen_paths: &mut BTreeSet<String>,
) -> Result<(), ReleaseEvidenceError> {
    validate_bundled_artifact(artifact, field_name)?;
    record_unique_artifact_path(&artifact.relative_path, seen_paths)
}

fn validate_provider_fixed_point_proof_artifact(
    artifact: &ProviderFixedPointProofArtifact,
    seen_paths: &mut BTreeSet<String>,
) -> Result<(), ReleaseEvidenceError> {
    validate_bundled_artifact(&provider_fixed_point_bundled_artifact(artifact), "provider_fixed_point_proof")?;
    if artifact.kind != BundledArtifactKind::Directory {
        return Err(validation_error(
            "release evidence provider_fixed_point_proof must be recorded as a directory artifact".to_string(),
        ));
    }
    if artifact.evidence_role != PROVIDER_FIXED_POINT_PROOF_EVIDENCE_ROLE {
        return Err(validation_error(format!(
            "release evidence provider_fixed_point_proof.evidence_role must be {PROVIDER_FIXED_POINT_PROOF_EVIDENCE_ROLE}, got {}",
            artifact.evidence_role
        )));
    }
    record_unique_artifact_path(&artifact.relative_path, seen_paths)
}

fn provider_fixed_point_bundled_artifact(artifact: &ProviderFixedPointProofArtifact) -> BundledArtifact {
    BundledArtifact {
        kind: artifact.kind,
        relative_path: artifact.relative_path.clone(),
        size_bytes: artifact.size_bytes,
        digest_blake3: artifact.digest_blake3.clone(),
    }
}

fn validate_role_bounded_release_artifact(
    artifact: &RoleBoundedReleaseArtifact,
    field_name: &str,
    expected_role: &str,
    expected_relative_path: &str,
    seen_paths: &mut BTreeSet<String>,
) -> Result<(), ReleaseEvidenceError> {
    validate_bundled_artifact(&role_bounded_bundled_artifact(artifact), field_name)?;
    if artifact.kind != BundledArtifactKind::File {
        return Err(validation_error(format!("release evidence {field_name} must be recorded as a file artifact")));
    }
    if artifact.evidence_role != expected_role {
        return Err(validation_error(format!(
            "release evidence {field_name}.evidence_role must be {expected_role}, got {}",
            artifact.evidence_role
        )));
    }
    if artifact.relative_path != expected_relative_path {
        return Err(validation_error(format!("release evidence {field_name} must be {expected_relative_path}")));
    }
    record_unique_artifact_path(&artifact.relative_path, seen_paths)
}

fn role_bounded_bundled_artifact(artifact: &RoleBoundedReleaseArtifact) -> BundledArtifact {
    BundledArtifact {
        kind: artifact.kind,
        relative_path: artifact.relative_path.clone(),
        size_bytes: artifact.size_bytes,
        digest_blake3: artifact.digest_blake3.clone(),
    }
}

fn record_unique_artifact_path(
    relative_path: &str,
    seen_paths: &mut BTreeSet<String>,
) -> Result<(), ReleaseEvidenceError> {
    if !seen_paths.insert(relative_path.to_string()) {
        return Err(validation_error(format!(
            "release evidence contains duplicate bundle member path {relative_path}"
        )));
    }
    Ok(())
}

fn validate_source_acquisition(manifest: &ReleaseEvidenceManifest) -> Result<(), ReleaseEvidenceError> {
    let Some(source_acquisition) = &manifest.source_acquisition else {
        return Ok(());
    };
    if source_acquisition.kind != SOURCE_ACQUISITION_KIND_EXTERNAL_ARCHIVE {
        return Err(validation_error(format!(
            "release evidence source_acquisition.kind must be {SOURCE_ACQUISITION_KIND_EXTERNAL_ARCHIVE}, got {}",
            source_acquisition.kind
        )));
    }
    validate_source_acquisition_url(&source_acquisition.url)?;
    validate_blake3_hex(&source_acquisition.digest_blake3, "source_acquisition.digest_blake3")?;
    if source_acquisition.digest_blake3 != manifest.source_archive.digest_blake3 {
        return Err(validation_error(
            "release evidence source_acquisition.digest_blake3 must match source_archive.digest_blake3".to_string(),
        ));
    }
    Ok(())
}

fn validate_source_acquisition_url(url: &str) -> Result<(), ReleaseEvidenceError> {
    if url.trim().is_empty() {
        return Err(validation_error("release evidence source_acquisition.url must not be empty".to_string()));
    }
    let byte_count = u32_count(url.as_bytes().len(), "release evidence source_acquisition.url length overflowed u32")?;
    if byte_count > MAX_SOURCE_ACQUISITION_URL_BYTES_COUNT {
        return Err(validation_error(format!(
            "release evidence source_acquisition.url is {byte_count} bytes, limit is {MAX_SOURCE_ACQUISITION_URL_BYTES_COUNT}"
        )));
    }
    if !(url.starts_with("file://") || url.starts_with("http://") || url.starts_with("https://")) {
        return Err(validation_error(
            "release evidence source_acquisition.url must start with file://, http://, or https://".to_string(),
        ));
    }
    Ok(())
}

fn validate_manifest_linkage(manifest: &ReleaseEvidenceManifest) -> Result<(), ReleaseEvidenceError> {
    if manifest.proof_linkage.release_id != manifest.release_id {
        return Err(validation_error("release evidence proof linkage release_id must match release_id".to_string()));
    }
    validate_blake3_hex(
        &manifest.proof_linkage.source_archive_digest_blake3,
        "proof_linkage.source_archive_digest_blake3",
    )?;
    validate_blake3_hex(
        &manifest.proof_linkage.stage2_binary_digest_blake3,
        "proof_linkage.stage2_binary_digest_blake3",
    )?;
    validate_blake3_hex(
        &manifest.proof_linkage.prerequisite_inventory_digest_blake3,
        "proof_linkage.prerequisite_inventory_digest_blake3",
    )?;
    validate_blake3_hex(
        &manifest.proof_linkage.proof_manifest_digest_blake3,
        "proof_linkage.proof_manifest_digest_blake3",
    )?;
    if manifest.proof_linkage.proof_bundle_schema != FULL_SELF_HOSTING_PROOF_SCHEMA {
        return Err(validation_error(format!(
            "release evidence proof_bundle_schema must be {FULL_SELF_HOSTING_PROOF_SCHEMA}, got {}",
            manifest.proof_linkage.proof_bundle_schema
        )));
    }
    if manifest.proof_linkage.proof_mode.trim().is_empty() {
        return Err(validation_error("release evidence proof_mode must not be empty".to_string()));
    }
    validate_provider_kind(&manifest.proof_linkage.selected_provider_kind, "proof_linkage.selected_provider_kind")?;
    if manifest.proof_linkage.staged_source.trim().is_empty() {
        return Err(validation_error("release evidence staged_source must not be empty".to_string()));
    }
    if manifest.proof_bundle.kind != BundledArtifactKind::Directory {
        return Err(validation_error(
            "release evidence proof_bundle must be recorded as a directory artifact".to_string(),
        ));
    }
    if manifest.prerequisite_inventory.kind != BundledArtifactKind::File {
        return Err(validation_error(
            "release evidence prerequisite_inventory must be recorded as a file artifact".to_string(),
        ));
    }
    if manifest.proof_linkage.source_archive_digest_blake3 != manifest.source_archive.digest_blake3 {
        return Err(validation_error(
            "release evidence proof linkage source archive digest does not match bundled source archive".to_string(),
        ));
    }
    if manifest.proof_linkage.prerequisite_inventory_digest_blake3 != manifest.prerequisite_inventory.digest_blake3 {
        return Err(validation_error(
            "release evidence proof linkage prerequisite inventory digest does not match bundled prerequisite inventory"
                .to_string(),
        ));
    }
    if !manifest
        .binaries
        .iter()
        .any(|artifact| artifact.digest_blake3 == manifest.proof_linkage.stage2_binary_digest_blake3)
    {
        return Err(validation_error(
            "release evidence proof linkage stage2 digest does not match any bundled binary artifact".to_string(),
        ));
    }
    Ok(())
}

fn validate_full_proof_manifest(manifest: &SelfHostingProofManifestView) -> Result<(), ReleaseEvidenceError> {
    if manifest.schema != FULL_SELF_HOSTING_PROOF_SCHEMA {
        return Err(validation_error(format!(
            "full proof artifact required: expected schema {FULL_SELF_HOSTING_PROOF_SCHEMA}, got {}",
            manifest.schema
        )));
    }
    if manifest.staged_source.trim().is_empty() {
        return Err(validation_error("full proof artifact required: staged_source is missing".to_string()));
    }
    if manifest.prerequisites.mode.trim().is_empty() {
        return Err(validation_error("full proof artifact required: proof mode is missing".to_string()));
    }
    validate_provider_kind(&manifest.prerequisites.provider_kind, "prerequisites.provider_kind")?;

    validate_hashed_path(&manifest.prerequisites.inventory_doc, "prerequisites.inventory_doc")?;
    validate_hashed_path(&manifest.binaries.stage1, "binaries.stage1")?;
    validate_hashed_path(&manifest.binaries.stage2, "binaries.stage2")?;
    validate_hashed_path(&manifest.tools.stage0_bwrap, "tools.stage0_bwrap")?;
    validate_hashed_path(&manifest.tools.stage0_busybox, "tools.stage0_busybox")?;
    validate_hashed_path(&manifest.tools.stage2_bwrap, "tools.stage2_bwrap")?;
    validate_hashed_path(&manifest.tools.stage2_busybox, "tools.stage2_busybox")?;

    if !manifest.fixed_point.stage1_equals_stage2 {
        return Err(validation_error("full proof artifact required: stage1_equals_stage2 must be true".to_string()));
    }
    if !manifest.fixed_point.stage0_bwrap_equals_stage2_bwrap {
        return Err(validation_error(
            "full proof artifact required: stage0_bwrap_equals_stage2_bwrap must be true".to_string(),
        ));
    }
    if !manifest.fixed_point.stage0_busybox_equals_stage2_busybox {
        return Err(validation_error(
            "full proof artifact required: stage0_busybox_equals_stage2_busybox must be true".to_string(),
        ));
    }

    validate_stage_report(&manifest.stage0.report, &manifest.staged_source, "stage0.report")?;
    validate_stage_report(&manifest.stage2.report, &manifest.staged_source, "stage2.report")?;
    if manifest.stage2.report.output_binary != manifest.binaries.stage2.path {
        return Err(validation_error(
            "full proof artifact required: stage2 report output_binary must match binaries.stage2.path".to_string(),
        ));
    }
    Ok(())
}

fn validate_provider_kind(provider_kind: &str, field_name: &str) -> Result<(), ReleaseEvidenceError> {
    match provider_kind {
        "legacy-fetch" | "source-root" | "stagex-lineage" => Ok(()),
        "" => Err(validation_error(format!("{field_name} must not be empty"))),
        other => Err(validation_error(format!(
            "{field_name} must be one of legacy-fetch, source-root, stagex-lineage; got {other}"
        ))),
    }
}

fn validate_hashed_path(hashed: &SelfHostingProofHashedPath, field_name: &str) -> Result<(), ReleaseEvidenceError> {
    if hashed.path.trim().is_empty() {
        return Err(validation_error(format!("full proof artifact required: {field_name}.path is missing")));
    }
    if hashed.size_bytes == 0 {
        return Err(validation_error(format!(
            "full proof artifact required: {field_name}.size_bytes must be non-zero"
        )));
    }
    validate_blake3_hex(&hashed.digest_blake3, &format!("{field_name}.digest_blake3"))
}

fn validate_stage_report(
    report: &SelfHostingProofReportView,
    expected_staged_source: &str,
    field_name: &str,
) -> Result<(), ReleaseEvidenceError> {
    if report.staged_source != expected_staged_source {
        return Err(validation_error(format!(
            "full proof artifact required: {field_name}.staged_source does not match top-level staged_source"
        )));
    }
    if report.output_binary.trim().is_empty() {
        return Err(validation_error(format!("full proof artifact required: {field_name}.output_binary is missing")));
    }
    if report.busybox_path.is_none() {
        return Err(validation_error(format!("full proof artifact required: {field_name}.busybox_path is missing")));
    }
    Ok(())
}

fn validate_relative_member_path(path: &str, field_name: &str) -> Result<(), ReleaseEvidenceError> {
    if path.trim().is_empty() {
        return Err(validation_error(format!("release evidence {field_name} must not be empty")));
    }
    if path.starts_with('/') {
        return Err(validation_error(format!(
            "release evidence {field_name} must be relative, got absolute path {path}"
        )));
    }
    if path.split('/').any(|component| component == "..") {
        return Err(validation_error(format!("release evidence {field_name} must not escape the bundle root: {path}")));
    }
    let path_len_bytes = u32_count(path.len(), &format!("release evidence {field_name} length overflowed u32"))?;
    if path_len_bytes > MAX_RELATIVE_PATH_BYTES_COUNT {
        return Err(validation_error(format!(
            "release evidence {field_name} exceeds {MAX_RELATIVE_PATH_BYTES_COUNT} bytes"
        )));
    }
    Ok(())
}

pub(crate) fn validate_blake3_hex(digest_hex: &str, field_name: &str) -> Result<(), ReleaseEvidenceError> {
    if digest_hex.len() != BLAKE3_HEX_LENGTH_CHARS {
        return Err(validation_error(format!(
            "{field_name} must be {BLAKE3_HEX_LENGTH_CHARS} lowercase hex chars, got {}",
            digest_hex.len()
        )));
    }
    if !digest_hex.bytes().all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase()) {
        return Err(validation_error(format!("{field_name} must contain lowercase hex only")));
    }
    Ok(())
}

fn parse_error(message: String) -> ReleaseEvidenceError {
    ReleaseEvidenceError::Parse(message)
}

pub(crate) fn validation_error(message: String) -> ReleaseEvidenceError {
    ReleaseEvidenceError::Validation(message)
}

pub(crate) fn u32_count(count: usize, overflow_message: &str) -> Result<u32, ReleaseEvidenceError> {
    u32::try_from(count).map_err(|_| validation_error(overflow_message.to_string()))
}

#[cfg(test)]
mod tests {
    use alloc::vec;

    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::*;

    fn sample_digest(seed: u8) -> String {
        let byte = format!("{:x}", seed % 16);
        byte.repeat(BLAKE3_HEX_LENGTH_CHARS)
    }

    fn sample_artifact(kind: BundledArtifactKind, relative_path: &str, seed: u8) -> BundledArtifact {
        BundledArtifact {
            kind,
            relative_path: relative_path.to_string(),
            size_bytes: 123,
            digest_blake3: sample_digest(seed),
        }
    }

    fn sample_provider_fixed_point_artifact(seed: u8) -> ProviderFixedPointProofArtifact {
        ProviderFixedPointProofArtifact {
            kind: BundledArtifactKind::Directory,
            relative_path: "proof/provider-fixed-point".to_string(),
            size_bytes: 123,
            digest_blake3: sample_digest(seed),
            evidence_role: PROVIDER_FIXED_POINT_PROOF_EVIDENCE_ROLE.to_string(),
        }
    }

    fn sample_role_bounded_artifact(relative_path: &str, role: &str, seed: u8) -> RoleBoundedReleaseArtifact {
        RoleBoundedReleaseArtifact {
            kind: BundledArtifactKind::File,
            relative_path: relative_path.to_string(),
            size_bytes: 123,
            digest_blake3: sample_digest(seed),
            evidence_role: role.to_string(),
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
        }
    }

    fn sample_full_proof_manifest(inventory_digest: &str, stage2_digest: &str) -> Vec<u8> {
        serde_json::to_vec(&json!({
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
        }))
        .unwrap()
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
    fn validate_accepts_matching_external_source_acquisition() {
        let mut manifest = sample_manifest();
        manifest.source_acquisition = Some(SourceAcquisition {
            kind: SOURCE_ACQUISITION_KIND_EXTERNAL_ARCHIVE.to_string(),
            url: "https://example.invalid/mantle-src.tar".to_string(),
            digest_blake3: manifest.source_archive.digest_blake3.clone(),
        });

        let bytes = canonical_release_evidence_manifest(manifest).unwrap();
        let text = String::from_utf8(bytes).unwrap();

        assert!(text.contains("source_acquisition"));
        assert!(text.contains(SOURCE_ACQUISITION_KIND_EXTERNAL_ARCHIVE));
    }

    #[test]
    fn validate_rejects_source_acquisition_digest_mismatch() {
        let mut manifest = sample_manifest();
        manifest.source_acquisition = Some(SourceAcquisition {
            kind: SOURCE_ACQUISITION_KIND_EXTERNAL_ARCHIVE.to_string(),
            url: "https://example.invalid/mantle-src.tar".to_string(),
            digest_blake3: sample_digest(2),
        });

        let err = canonical_release_evidence_manifest(manifest).unwrap_err();

        assert!(err.to_string().contains("source_acquisition.digest_blake3 must match"));
    }

    #[test]
    fn validate_rejects_unsupported_source_acquisition_url() {
        let mut manifest = sample_manifest();
        manifest.source_acquisition = Some(SourceAcquisition {
            kind: SOURCE_ACQUISITION_KIND_EXTERNAL_ARCHIVE.to_string(),
            url: "git@example.invalid:mantle.git".to_string(),
            digest_blake3: manifest.source_archive.digest_blake3.clone(),
        });

        let err = canonical_release_evidence_manifest(manifest).unwrap_err();

        assert!(err.to_string().contains("must start with file://, http://, or https://"));
    }

    #[test]
    fn validate_accepts_provider_fixed_point_proof_artifact() {
        let mut manifest = sample_manifest();
        manifest.provider_fixed_point_proof = Some(sample_provider_fixed_point_artifact(8));

        let bytes = canonical_release_evidence_manifest(manifest).unwrap();
        let text = String::from_utf8(bytes).unwrap();

        assert!(text.contains("provider_fixed_point_proof"));
        assert!(text.contains(PROVIDER_FIXED_POINT_PROOF_EVIDENCE_ROLE));
    }

    #[test]
    fn validate_rejects_provider_fixed_point_proof_with_wrong_role() {
        let mut manifest = sample_manifest();
        let mut proof = sample_provider_fixed_point_artifact(8);
        proof.evidence_role = "release-reproducibility".to_string();
        manifest.provider_fixed_point_proof = Some(proof);

        let err = canonical_release_evidence_manifest(manifest).unwrap_err();

        assert!(err.to_string().contains("provider_fixed_point_proof.evidence_role"));
    }

    #[test]
    fn validate_rejects_provider_fixed_point_proof_file_kind() {
        let mut manifest = sample_manifest();
        let mut proof = sample_provider_fixed_point_artifact(8);
        proof.kind = BundledArtifactKind::File;
        manifest.provider_fixed_point_proof = Some(proof);

        let err = canonical_release_evidence_manifest(manifest).unwrap_err();

        assert!(err.to_string().contains("must be recorded as a directory artifact"));
    }

    #[test]
    fn provider_fixed_point_release_artifact_binding_accepts_matching_binary() {
        let manifest = sample_manifest();
        let stage_digest = manifest.binaries[0].digest_blake3.clone();

        let binding =
            validate_provider_fixed_point_release_artifact_binding(&manifest.binaries, &stage_digest).unwrap();

        assert_eq!(binding.relative_path, "binaries/01-mantle");
        assert_eq!(binding.digest_blake3, stage_digest);
    }

    #[test]
    fn provider_fixed_point_release_artifact_binding_rejects_mismatch() {
        let manifest = sample_manifest();
        let err =
            validate_provider_fixed_point_release_artifact_binding(&manifest.binaries, &sample_digest(12)).unwrap_err();

        assert!(err.to_string().contains("does not match any bundled release binary artifact"));
    }

    #[test]
    fn provider_fixed_point_release_artifact_binding_rejects_missing_binaries() {
        let err = validate_provider_fixed_point_release_artifact_binding(&[], &sample_digest(12)).unwrap_err();

        assert!(err.to_string().contains("at least one binary artifact"));
    }

    #[test]
    fn provider_fixed_point_release_artifact_binding_rejects_malformed_digest() {
        let manifest = sample_manifest();
        let err =
            validate_provider_fixed_point_release_artifact_binding(&manifest.binaries, "not-a-blake3").unwrap_err();

        assert!(err.to_string().contains("provider_fixed_point.stage_binary_digest_blake3"));
    }

    #[test]
    fn validate_accepts_deterministic_proof_artifacts() {
        let mut manifest = sample_manifest();
        manifest.deterministic_build_proof = Some(sample_role_bounded_artifact(
            "deterministic-release/deterministic-build-proof.json",
            DETERMINISTIC_BUILD_PROOF_EVIDENCE_ROLE,
            10,
        ));
        manifest.deterministic_sandbox_isolation_evidence = Some(sample_role_bounded_artifact(
            "deterministic-release/deterministic-sandbox-isolation-evidence.json",
            DETERMINISTIC_SANDBOX_ISOLATION_EVIDENCE_ROLE,
            11,
        ));

        let bytes = canonical_release_evidence_manifest(manifest).unwrap();
        let text = String::from_utf8(bytes).unwrap();

        assert!(text.contains("deterministic_build_proof"));
        assert!(text.contains(DETERMINISTIC_BUILD_PROOF_EVIDENCE_ROLE));
        assert!(text.contains("deterministic_sandbox_isolation_evidence"));
        assert!(text.contains(DETERMINISTIC_SANDBOX_ISOLATION_EVIDENCE_ROLE));
    }

    #[test]
    fn validate_rejects_deterministic_proof_with_wrong_role() {
        let mut manifest = sample_manifest();
        manifest.deterministic_build_proof = Some(sample_role_bounded_artifact(
            "deterministic-release/deterministic-build-proof.json",
            "wrong-role",
            10,
        ));

        let err = canonical_release_evidence_manifest(manifest).unwrap_err();

        assert!(err.to_string().contains("deterministic_build_proof.evidence_role"));
    }

    #[test]
    fn validate_rejects_deterministic_proof_with_wrong_path() {
        let mut manifest = sample_manifest();
        manifest.deterministic_sandbox_isolation_evidence = Some(sample_role_bounded_artifact(
            "deterministic-release/wrong.json",
            DETERMINISTIC_SANDBOX_ISOLATION_EVIDENCE_ROLE,
            11,
        ));

        let err = canonical_release_evidence_manifest(manifest).unwrap_err();

        assert!(err.to_string().contains("deterministic_sandbox_isolation_evidence"));
        assert!(err.to_string().contains("deterministic-release/deterministic-sandbox-isolation-evidence.json"));
    }

    #[test]
    fn validate_rejects_deterministic_proof_duplicate_path() {
        let mut manifest = sample_manifest();
        manifest.source_archive.relative_path = "deterministic-release/deterministic-build-proof.json".to_string();
        manifest.deterministic_build_proof = Some(sample_role_bounded_artifact(
            "deterministic-release/deterministic-build-proof.json",
            DETERMINISTIC_BUILD_PROOF_EVIDENCE_ROLE,
            10,
        ));

        let err = canonical_release_evidence_manifest(manifest).unwrap_err();

        assert!(err.to_string().contains("duplicate bundle member path"));
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
    fn extract_full_proof_identity_fields_accepts_valid_manifest() {
        let manifest_bytes = sample_full_proof_manifest(&sample_digest(9), &sample_digest(11));
        let identity = extract_full_self_hosting_proof_identity_fields(manifest_bytes).unwrap();
        assert_eq!(identity.schema, FULL_SELF_HOSTING_PROOF_SCHEMA);
        assert_eq!(identity.proof_mode, "fixed-point");
        assert_eq!(identity.selected_provider_kind, "source-root");
        assert_eq!(identity.staged_source, "/tmp/proof-store/abcd-mantle-src");
        assert_eq!(identity.stage2_binary_digest_blake3, sample_digest(11));
        assert_eq!(identity.prerequisite_inventory_digest_blake3, sample_digest(9));
    }

    #[test]
    fn extract_full_proof_identity_fields_rejects_missing_provider_kind() {
        let mut manifest: serde_json::Value =
            serde_json::from_slice(&sample_full_proof_manifest(&sample_digest(9), &sample_digest(11))).unwrap();
        manifest["prerequisites"].as_object_mut().unwrap().remove("provider_kind");
        let err = extract_full_self_hosting_proof_identity_fields(serde_json::to_vec(&manifest).unwrap()).unwrap_err();
        assert!(err.to_string().contains("provider_kind"));
    }

    #[test]
    fn release_manifest_rejects_unknown_selected_provider_kind() {
        let mut manifest = sample_manifest();
        manifest.proof_linkage.selected_provider_kind = "mystery".to_string();
        let err = canonical_release_evidence_manifest(manifest).unwrap_err();
        assert!(err.to_string().contains("selected_provider_kind"));
    }

    #[test]
    fn extract_full_proof_identity_fields_rejects_wrong_schema() {
        let manifest_bytes = br#"{"schema":"fake-proof"}"#.to_vec();
        let err = extract_full_self_hosting_proof_identity_fields(manifest_bytes).unwrap_err();
        assert!(err.to_string().contains("full proof artifact required"));
    }
}
