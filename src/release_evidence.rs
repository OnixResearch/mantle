// machine-artifact-public: release.evidence-reports
use std::path::Path;
use std::path::PathBuf;

#[cfg(test)]
pub(crate) use crunch_release_core::BLAKE3_HEX_LENGTH_CHARS as BLAKE3_HEX_LEN;
pub(crate) use crunch_release_core::BundledArtifact;
pub(crate) use crunch_release_core::BundledArtifactKind;
pub(crate) use crunch_release_core::CLAIM_SCOPE_PACKAGED_INTEGRITY;
use crunch_release_core::CONTENT_BOUND_EVIDENCE_CLAIM_SCOPE;
use crunch_release_core::CONTENT_BOUND_EVIDENCE_EXTERNAL_SCHEMA_V1;
use crunch_release_core::CairnReleaseEvidenceHandoff;
use crunch_release_core::ContentBoundReleaseBuildInputV1;
use crunch_release_core::ContentBoundReleaseEvidenceV1;
pub(crate) use crunch_release_core::DEFAULT_PROOF_WORKFLOW_COMMAND;
pub(crate) use crunch_release_core::DEFAULT_PROOF_WORKFLOW_VERSION;
use crunch_release_core::ExternalEvidence;
#[cfg(test)]
pub(crate) use crunch_release_core::FULL_SELF_HOSTING_PROOF_SCHEMA;
use crunch_release_core::KANI_EVIDENCE_CLAIM_SCOPE;
use crunch_release_core::KANI_TOOLCHAIN_EVIDENCE_SCHEMA;
use crunch_release_core::KANI_VALENCE_SEMANTIC_ROLE;
use crunch_release_core::KaniSolverIdentity;
use crunch_release_core::KaniToolchainEvidence;
use crunch_release_core::PROVIDER_FIXED_POINT_PROOF_EVIDENCE_ROLE;
use crunch_release_core::ProviderFixedPointProofArtifact;
use crunch_release_core::PublicationArtifactInput;
use crunch_release_core::PublicationEvent;
use crunch_release_core::PublicationFailurePhase;
use crunch_release_core::PublicationPlan;
use crunch_release_core::PublicationPlanRequest;
use crunch_release_core::PublicationPolicyFact;
use crunch_release_core::PublicationState;
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
pub(crate) use crunch_release_core::SourceObservationBinding;
use crunch_release_core::StackProvenanceReleaseEvidence;
use crunch_release_core::VALENCE_STACK_PROVENANCE_RECEIPT_ROLE;
use crunch_release_core::build_content_bound_release_evidence;
use crunch_release_core::cairn_release_evidence_validation_receipt;
use crunch_release_core::canonical_release_evidence_manifest;
use crunch_release_core::extract_full_self_hosting_proof_identity_fields;
use crunch_release_core::initial_publication_state;
use crunch_release_core::plan_release_publication;
use crunch_release_core::publication_commit_eligible;
use crunch_release_core::release_evidence_cairn_bundle_binding;
use crunch_release_core::release_reproducibility_report_canonical_bytes;
use crunch_release_core::transition_publication_state;
use crunch_release_core::validate_bundled_artifact_record;
use crunch_release_core::validate_cairn_release_evidence_validation_receipt;
use crunch_release_core::validate_provider_fixed_point_release_artifact_binding;
use crunch_release_core::validate_release_reproducibility_report_artifact_names;
use crunch_release_core::validate_release_reproducibility_report_linkage;
use crunch_release_core::validate_staged_publication_artifacts;

use crate::content_bound_requirement_evidence::ContentBoundRequirementCreateRequest;
use crate::errors::RunError;
use crate::release_capability::ReleaseCapabilityRoot;
use crate::release_capability::ValidatedReleasePath;
use crate::release_publication::PublicationCommitError;
use crate::release_publication::ReleasePublicationStage;
use crate::release_publication::create_release_publication_stage;
use crate::release_publication::observe_publication_destination;
use crate::release_tree_copy::PreparedTreeCopy;

const PROOF_INVENTORY_RELATIVE_PATH: &str = "stage0-prerequisites/inventory.md";
const MAX_BINARY_ARTIFACTS: u32 = 16;
const PUBLICATION_POLICY_INDEX_WIDTH: usize = 3;
const STACK_PROVENANCE_EXTERNAL_ARTIFACT_COUNT: usize = 2;

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
    pub source_observation: Option<SourceObservationBinding>,
    pub git_source: Option<GitSourceCreateRequest>,
    pub external_evidence: Vec<ExternalEvidenceCreateRequest>,
    pub kani_toolchain_evidence: Vec<KaniToolchainEvidenceCreateRequest>,
    pub stack_provenance: Option<StackProvenanceCreateRequest>,
    pub content_bound_requirement: Option<ContentBoundRequirementCreateRequest>,
    pub cairn_handoff_descriptor_path: Option<PathBuf>,
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
            source_observation: None,
            git_source: None,
            external_evidence: vec![],
            kani_toolchain_evidence: vec![],
            stack_provenance: None,
            content_bound_requirement: None,
            cairn_handoff_descriptor_path: None,
        }
    }
}

fn core_error_to_run_error(err: ReleaseEvidenceError) -> RunError {
    RunError::Internal(err.to_string())
}

const PUBLICATION_DIAGNOSTIC_BLOCKERS_MAX: usize = 16;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PublicationShellPhase {
    StageCreated,
    InputsCopied,
    ArtifactsHashed,
    ManifestSerialized,
    ManifestWritten,
    StagedVerified,
    PreCommit,
    Cleanup,
}

pub(crate) struct PublicationShellContext<'a> {
    pub stage_path: &'a Path,
    pub final_path: &'a Path,
    pub plan_identity_blake3: &'a str,
}

pub(crate) trait PublicationShellAdapter {
    fn after_phase(
        &mut self,
        phase: PublicationShellPhase,
        context: &PublicationShellContext<'_>,
    ) -> Result<(), RunError>;
}

struct ProductionPublicationShell;

impl PublicationShellAdapter for ProductionPublicationShell {
    fn after_phase(
        &mut self,
        _phase: PublicationShellPhase,
        _context: &PublicationShellContext<'_>,
    ) -> Result<(), RunError> {
        Ok(())
    }
}

struct PreparedReleasePublication {
    plan: PublicationPlan,
    proof_identity: FullSelfHostingProofIdentity,
    prepared_proof_bundle: PreparedTreeCopy,
    prepared_provider_fixed_point: Option<PreparedTreeCopy>,
}

struct PublicationAttemptFailure {
    state: PublicationState,
    error: RunError,
}

pub(crate) fn create_release_evidence_bundle(
    request: &ReleaseBundleCreateRequest,
) -> Result<ReleaseEvidenceManifest, RunError> {
    let mut shell = ProductionPublicationShell;
    create_release_evidence_bundle_with_adapter(request, &mut shell)
}

// r[impl mantle.release_provenance.bundle_publication.atomic_commit]
// r[impl mantle.release_provenance.bundle_publication.failure_isolation]
pub(crate) fn create_release_evidence_bundle_with_adapter(
    request: &ReleaseBundleCreateRequest,
    shell: &mut dyn PublicationShellAdapter,
) -> Result<ReleaseEvidenceManifest, RunError> {
    let prepared = prepare_release_publication(request)?;
    let initial_state = initial_publication_state(&prepared.plan);
    let stage = create_release_publication_stage(&request.bundle_dir, &prepared.plan)
        .map_err(|error| publication_phase_error(PublicationFailurePhase::StageCreation, error))?;
    let result = publish_prepared_release(request, prepared, &stage, initial_state, shell);
    match result {
        Ok(manifest) => Ok(manifest),
        Err(failure) => cleanup_failed_publication(&stage, failure, shell),
    }
}

fn prepare_release_publication(request: &ReleaseBundleCreateRequest) -> Result<PreparedReleasePublication, RunError> {
    validate_create_request(request)?;
    validate_optional_provider_fixed_point_proof(request)?;
    let proof_identity = load_full_self_hosting_proof_identity(&request.proof_bundle_dir)?;
    let prepared_proof_bundle = crate::release_tree_copy::prepare_tree_copy(&request.proof_bundle_dir)?;
    let prepared_provider_fixed_point = request
        .provider_fixed_point_proof_dir
        .as_deref()
        .map(crate::release_tree_copy::prepare_tree_copy)
        .transpose()?;
    let expected =
        expected_publication_artifacts(request, &prepared_proof_bundle, prepared_provider_fixed_point.as_ref())?;
    validate_pre_mutation_dependencies(request, &expected)?;
    let destination = observe_publication_destination(&request.bundle_dir)?;
    let plan = plan_release_publication(PublicationPlanRequest {
        release_id: request.release_id.clone(),
        destination,
        artifacts: expected.iter().map(publication_artifact_input).collect(),
        policy: publication_policy_facts(request, &expected)?,
    })
    .map_err(publication_plan_error)?;
    assert!(!plan.artifacts.is_empty(), "release publication plan must include artifacts");
    assert_eq!(plan.release_id, request.release_id);
    Ok(PreparedReleasePublication {
        plan,
        proof_identity,
        prepared_proof_bundle,
        prepared_provider_fixed_point,
    })
}

fn expected_publication_artifacts(
    request: &ReleaseBundleCreateRequest,
    prepared_proof_bundle: &PreparedTreeCopy,
    prepared_provider_fixed_point: Option<&PreparedTreeCopy>,
) -> Result<Vec<BundledArtifact>, RunError> {
    let source_relative = bundle_file_destination("source", &request.source_archive_path, 0)?;
    let source_archive =
        build_artifact_record(&request.source_archive_path, &source_relative, BundledArtifactKind::File)?;
    let binaries = build_input_binary_artifacts(&request.binary_paths)?;
    let proof_bundle = prepared_directory_artifact(prepared_proof_bundle, Path::new("proof/self-hosting"))?;
    let inventory_path = request.proof_bundle_dir.join(PROOF_INVENTORY_RELATIVE_PATH);
    let inventory = build_artifact_record(&inventory_path, Path::new("proof/inventory.md"), BundledArtifactKind::File)?;
    let mut artifacts = vec![source_archive.clone()];
    artifacts.extend(binaries.clone());
    artifacts.push(proof_bundle.clone());
    artifacts.push(inventory);
    append_optional_expected_artifacts(
        OptionalExpectedArtifacts {
            request,
            prepared_provider: prepared_provider_fixed_point,
            source_archive: &source_archive,
            binaries: &binaries,
            proof_bundle: &proof_bundle,
        },
        &mut artifacts,
    )?;
    append_expected_external_artifacts(request, &mut artifacts)?;
    append_expected_content_bound_requirement_artifacts(request, &mut artifacts)?;
    append_expected_cairn_handoff_artifacts(request, &mut artifacts)?;
    debug_assert!(!artifacts.is_empty());
    debug_assert!(artifacts.capacity() >= artifacts.len());
    Ok(artifacts)
}

struct OptionalExpectedArtifacts<'a> {
    request: &'a ReleaseBundleCreateRequest,
    prepared_provider: Option<&'a PreparedTreeCopy>,
    source_archive: &'a BundledArtifact,
    binaries: &'a [BundledArtifact],
    proof_bundle: &'a BundledArtifact,
}

fn append_optional_expected_artifacts(
    input: OptionalExpectedArtifacts<'_>,
    artifacts: &mut Vec<BundledArtifact>,
) -> Result<(), RunError> {
    debug_assert!(!artifacts.is_empty());
    debug_assert!(artifacts.capacity() >= artifacts.len());
    if let Some(prepared) = input.prepared_provider {
        artifacts.push(prepared_directory_artifact(prepared, Path::new("proof/provider-fixed-point"))?);
    }
    if let Some(report_path) = &input.request.reproducibility_report_path {
        validate_reproducibility_report_for_bundle(
            input.request,
            report_path,
            input.source_archive,
            input.binaries,
            input.proof_bundle,
        )?;
        artifacts.push(build_artifact_record(
            report_path,
            Path::new("reproducibility/reproducibility-report.json"),
            BundledArtifactKind::File,
        )?);
    }
    Ok(())
}

fn append_expected_external_artifacts(
    request: &ReleaseBundleCreateRequest,
    artifacts: &mut Vec<BundledArtifact>,
) -> Result<(), RunError> {
    let mut external_count = 0_u32;
    let artifacts_before_count = artifacts.len();
    for evidence in &request.external_evidence {
        external_count = external_count
            .checked_add(1)
            .ok_or_else(|| RunError::Internal("release external evidence count overflowed u32".to_string()))?;
        let relative = bundle_file_destination("external-evidence", &evidence.path, external_count)?;
        artifacts.push(build_artifact_record(&evidence.path, &relative, BundledArtifactKind::File)?);
    }
    if let Some(stack) = &request.stack_provenance {
        for path in [&stack.sidecar_path, &stack.valence_receipt_path] {
            external_count = external_count
                .checked_add(1)
                .ok_or_else(|| RunError::Internal("release stack evidence count overflowed u32".to_string()))?;
            let relative = bundle_file_destination("external-evidence", path, external_count)?;
            artifacts.push(build_artifact_record(path, &relative, BundledArtifactKind::File)?);
        }
    }
    debug_assert!(artifacts.len() >= artifacts_before_count);
    debug_assert!(artifacts.capacity() >= artifacts.len());
    Ok(())
}

fn append_expected_content_bound_requirement_artifacts(
    request: &ReleaseBundleCreateRequest,
    artifacts: &mut Vec<BundledArtifact>,
) -> Result<(), RunError> {
    let Some(content_bound) = &request.content_bound_requirement else {
        return Ok(());
    };
    for file in &content_bound.files {
        let artifact = BundledArtifact {
            kind: BundledArtifactKind::File,
            relative_path: file.row.bundle_relative_path.clone(),
            size_bytes: file.row.size_bytes,
            digest_blake3: file.row.content_identity.blake3.clone(),
        };
        artifacts.push(
            validate_bundled_artifact_record(artifact, "content_bound_requirement_evidence".to_string())
                .map_err(core_error_to_run_error)?,
        );
    }
    Ok(())
}

fn append_expected_cairn_handoff_artifacts(
    request: &ReleaseBundleCreateRequest,
    artifacts: &mut Vec<BundledArtifact>,
) -> Result<(), RunError> {
    let Some(descriptor_path) = request.cairn_handoff_descriptor_path.as_deref() else {
        return Ok(());
    };
    let measured = crate::cairn_release_handoff::plan_cairn_handoff_artifacts(descriptor_path, &request.bundle_dir)?;
    for artifact in measured {
        artifacts.push(BundledArtifact {
            kind: BundledArtifactKind::File,
            relative_path: artifact.relative_path,
            size_bytes: artifact.size_bytes,
            digest_blake3: artifact.measured_digest_blake3,
        });
    }
    debug_assert!(!artifacts.is_empty());
    debug_assert!(request.cairn_handoff_descriptor_path.is_some());
    Ok(())
}

fn prepared_directory_artifact(prepared: &PreparedTreeCopy, relative_path: &Path) -> Result<BundledArtifact, RunError> {
    let (size_bytes, digest_blake3) = prepared.artifact_identity()?;
    let artifact = BundledArtifact {
        kind: BundledArtifactKind::Directory,
        relative_path: path_to_forward_slash_string(relative_path)?,
        size_bytes,
        digest_blake3,
    };
    validate_bundled_artifact_record(artifact, "publication-plan-artifact".to_string()).map_err(core_error_to_run_error)
}

fn validate_pre_mutation_dependencies(
    request: &ReleaseBundleCreateRequest,
    expected: &[BundledArtifact],
) -> Result<(), RunError> {
    let binaries = expected
        .iter()
        .filter(|artifact| artifact.relative_path.starts_with("binaries/"))
        .cloned()
        .collect::<Vec<_>>();
    if request.stack_provenance.is_some() {
        select_stack_provenance_binary_artifact(request, &binaries)?;
    }
    let mut available_roles =
        request.external_evidence.iter().map(|evidence| evidence.role.as_str()).collect::<Vec<_>>();
    if request.stack_provenance.is_some() {
        available_roles.push(STACK_PROVENANCE_EVIDENCE_ROLE);
        available_roles.push(VALENCE_STACK_PROVENANCE_RECEIPT_ROLE);
    }
    for evidence in &request.kani_toolchain_evidence {
        if !available_roles.contains(&evidence.receipt_role.as_str()) {
            return Err(RunError::Internal(format!(
                "release evidence Kani receipt role required but missing from external evidence: {}",
                evidence.receipt_role
            )));
        }
    }
    assert_eq!(binaries.len(), request.binary_paths.len());
    assert!(!binaries.is_empty(), "validated release publication must include binaries");
    Ok(())
}

fn publication_artifact_input(artifact: &BundledArtifact) -> PublicationArtifactInput {
    PublicationArtifactInput {
        relative_path: artifact.relative_path.clone(),
        kind: artifact.kind,
        size_bytes: artifact.size_bytes,
        digest_blake3: artifact.digest_blake3.clone(),
    }
}

fn publication_policy_facts(
    request: &ReleaseBundleCreateRequest,
    expected: &[BundledArtifact],
) -> Result<Vec<PublicationPolicyFact>, RunError> {
    let mut facts = vec![
        policy_fact("manifest-schema", RELEASE_EVIDENCE_SCHEMA),
        policy_fact("claim-scope", CLAIM_SCOPE_PACKAGED_INTEGRITY),
        policy_fact("workflow-command", &request.workflow_command),
        policy_fact("workflow-version", &request.workflow_version),
        policy_fact("provider-fixed-point-present", boolean_text(request.provider_fixed_point_proof_dir.is_some())),
        policy_fact("reproducibility-report-present", boolean_text(request.reproducibility_report_path.is_some())),
        policy_fact("stack-provenance-present", boolean_text(request.stack_provenance.is_some())),
        policy_fact(
            "content-bound-requirement-evidence-present",
            boolean_text(request.content_bound_requirement.is_some()),
        ),
        policy_fact("cairn-handoff-present", boolean_text(request.cairn_handoff_descriptor_path.is_some())),
        policy_fact("artifact-count", &expected.len().to_string()),
    ];
    append_source_policy_facts(request, &mut facts);
    append_external_policy_facts(request, &mut facts)?;
    assert!(!facts.is_empty(), "release publication policy facts must not be empty");
    assert!(facts.iter().all(|fact| !fact.name.is_empty()));
    Ok(facts)
}

fn append_source_policy_facts(request: &ReleaseBundleCreateRequest, facts: &mut Vec<PublicationPolicyFact>) {
    facts.push(policy_fact("source-acquisition-url", request.source_acquisition_url.as_deref().unwrap_or("absent")));
    if let Some(git) = &request.git_source {
        facts.push(policy_fact("git-source-url", &git.remote_url));
        facts.push(policy_fact("git-source-commit", &git.commit));
        facts.push(policy_fact("git-source-ref", git.reference.as_deref().unwrap_or("absent")));
        facts.push(policy_fact("git-source-tag", git.tag.as_deref().unwrap_or("absent")));
    }
}

fn append_external_policy_facts(
    request: &ReleaseBundleCreateRequest,
    facts: &mut Vec<PublicationPolicyFact>,
) -> Result<(), RunError> {
    let facts_before_count = facts.len();
    for (index, evidence) in request.external_evidence.iter().enumerate() {
        let index_u32 = u32::try_from(index)
            .map_err(|_| RunError::Internal("release external policy index overflowed u32".to_string()))?;
        let value =
            serde_json::to_string(&(&evidence.role, &evidence.schema, &evidence.claim_scope, &evidence.non_claims))
                .map_err(|error| RunError::Internal(format!("serializing release external policy fact: {error}")))?;
        facts.push(policy_fact(
            &format!("external-evidence-{index_u32:0width$}", width = PUBLICATION_POLICY_INDEX_WIDTH),
            &value,
        ));
    }
    for (index, evidence) in request.kani_toolchain_evidence.iter().enumerate() {
        let index_u32 = u32::try_from(index)
            .map_err(|_| RunError::Internal("release Kani policy index overflowed u32".to_string()))?;
        let value = serde_json::to_string(&(
            &evidence.receipt_role,
            &evidence.kani_version,
            &evidence.rust_toolchain,
            &evidence.cbmc_version,
            &evidence.invocation_wrapper,
            &evidence.closure_identity_blake3,
            &evidence.expected_closure_identity_blake3,
            &evidence.non_claims,
        ))
        .map_err(|error| RunError::Internal(format!("serializing release Kani policy fact: {error}")))?;
        facts.push(policy_fact(
            &format!("kani-evidence-{index_u32:0width$}", width = PUBLICATION_POLICY_INDEX_WIDTH),
            &value,
        ));
    }
    debug_assert!(facts.len() >= facts_before_count);
    debug_assert!(facts.capacity() >= facts.len());
    Ok(())
}

// Policy facts are always constructed at labeled call sites as name/value
// pairs; preserving this tiny constructor keeps the policy ordering obvious.
#[allow(
    tigerstyle::ambiguous_params,
    reason = "two-field policy fact constructor is reviewed as an ordered name/value pair"
)]
fn policy_fact(name: &str, value: &str) -> PublicationPolicyFact {
    PublicationPolicyFact {
        name: name.to_string(),
        value: value.to_string(),
    }
}

const fn boolean_text(value: bool) -> &'static str {
    if value { "true" } else { "false" }
}

fn publication_plan_error(blockers: Vec<crunch_release_core::PublicationPlanBlocker>) -> RunError {
    let total_count = blockers.len();
    let rendered = blockers
        .iter()
        .take(PUBLICATION_DIAGNOSTIC_BLOCKERS_MAX)
        .map(|blocker| blocker.message.as_str())
        .collect::<Vec<_>>()
        .join("; ");
    RunError::Internal(format!("release publication plan rejected with {total_count} blocker(s): {rendered}"))
}

fn publish_prepared_release(
    request: &ReleaseBundleCreateRequest,
    prepared: PreparedReleasePublication,
    stage: &ReleasePublicationStage,
    state: PublicationState,
    shell: &mut dyn PublicationShellAdapter,
) -> Result<ReleaseEvidenceManifest, PublicationAttemptFailure> {
    debug_assert!(!prepared.plan.artifacts.is_empty());
    debug_assert_ne!(stage.stage_path(), stage.final_path());
    let state = advance_publication(state, PublicationEvent::StageCreated)?;
    let state = observe_publication_phase(shell, stage, &prepared.plan, state, PublicationShellPhase::StageCreated)?;
    let mut staged_request = request.clone();
    staged_request.bundle_dir = stage.stage_path().to_path_buf();
    let manifest = assemble_release_manifest(
        &staged_request,
        stage.root(),
        prepared.proof_identity,
        prepared.prepared_proof_bundle,
        prepared.prepared_provider_fixed_point,
    )
    .map_err(|error| fail_publication(state.clone(), PublicationFailurePhase::InputCopy, error))?;
    let state = advance_publication(state, PublicationEvent::InputsCopied)?;
    let state = observe_publication_phase(shell, stage, &prepared.plan, state, PublicationShellPhase::InputsCopied)?;
    let staged_artifacts = manifest_publication_artifacts(&manifest, stage.stage_path())
        .map_err(|error| fail_publication(state.clone(), PublicationFailurePhase::ArtifactHashing, error))?;
    validate_staged_publication_artifacts(&prepared.plan, staged_artifacts)
        .map_err(publication_plan_error)
        .map_err(|error| fail_publication(state.clone(), PublicationFailurePhase::ArtifactHashing, error))?;
    let state = advance_publication(state, PublicationEvent::ArtifactsHashed)?;
    let state = observe_publication_phase(shell, stage, &prepared.plan, state, PublicationShellPhase::ArtifactsHashed)?;
    serialize_verify_and_commit(manifest, stage, &prepared.plan, state, shell)
}

fn serialize_verify_and_commit(
    manifest: ReleaseEvidenceManifest,
    stage: &ReleasePublicationStage,
    plan: &PublicationPlan,
    state: PublicationState,
    shell: &mut dyn PublicationShellAdapter,
) -> Result<ReleaseEvidenceManifest, PublicationAttemptFailure> {
    debug_assert!(!plan.artifacts.is_empty());
    debug_assert_ne!(stage.stage_path(), stage.final_path());
    let bytes = canonical_release_evidence_manifest(manifest.clone())
        .map_err(core_error_to_run_error)
        .map_err(|error| fail_publication(state.clone(), PublicationFailurePhase::ManifestSerialization, error))?;
    let state = advance_publication(state, PublicationEvent::ManifestSerialized)?;
    let state = observe_publication_phase(shell, stage, plan, state, PublicationShellPhase::ManifestSerialized)?;
    write_manifest_bytes(stage.root(), &bytes)
        .map_err(|error| fail_publication(state.clone(), PublicationFailurePhase::ManifestWrite, error))?;
    let state = advance_publication(state, PublicationEvent::ManifestWritten)?;
    let state = observe_publication_phase(shell, stage, plan, state, PublicationShellPhase::ManifestWritten)?;
    let verified = match verify_release_evidence_bundle(stage.stage_path()) {
        Ok(verified) => verified,
        Err(error) => {
            let failed = advance_publication(state, PublicationEvent::VerificationRejected)?;
            return Err(PublicationAttemptFailure {
                state: failed,
                error: publication_phase_error(PublicationFailurePhase::Verification, error),
            });
        }
    };
    if verified != manifest {
        return Err(fail_publication(
            state,
            PublicationFailurePhase::Verification,
            RunError::Internal("staged release verification returned a different manifest".to_string()),
        ));
    }
    let state = advance_publication(state, PublicationEvent::VerificationAccepted)?;
    let state = observe_publication_phase(shell, stage, plan, state, PublicationShellPhase::StagedVerified)?;
    commit_verified_stage(manifest, stage, plan, state, shell)
}

fn commit_verified_stage(
    manifest: ReleaseEvidenceManifest,
    stage: &ReleasePublicationStage,
    plan: &PublicationPlan,
    state: PublicationState,
    shell: &mut dyn PublicationShellAdapter,
) -> Result<ReleaseEvidenceManifest, PublicationAttemptFailure> {
    let state = observe_publication_phase(shell, stage, plan, state, PublicationShellPhase::PreCommit)?;
    let final_verified = verify_release_evidence_bundle(stage.stage_path())
        .map_err(|error| fail_publication(state.clone(), PublicationFailurePhase::PreCommit, error))?;
    if final_verified != manifest {
        return Err(fail_publication(
            state,
            PublicationFailurePhase::PreCommit,
            RunError::Internal("pre-commit release verification returned a different manifest".to_string()),
        ));
    }
    stage
        .remove_ownership_marker()
        .map_err(|error| fail_publication(state.clone(), PublicationFailurePhase::PreCommit, error))?;
    assert!(publication_commit_eligible(&state), "only verified release stages may commit");
    assert_ne!(stage.stage_path(), stage.final_path(), "release stage and destination must differ");
    let state = match stage.commit_no_replace() {
        Ok(()) => advance_publication(state, PublicationEvent::CommitSucceeded)?,
        Err(PublicationCommitError::DestinationExists(message)) => {
            let failed = advance_publication(state, PublicationEvent::CommitLostRace)?;
            return Err(PublicationAttemptFailure {
                state: failed,
                error: RunError::Internal(message),
            });
        }
        Err(PublicationCommitError::Other(message)) => {
            return Err(fail_publication(state, PublicationFailurePhase::Commit, RunError::Internal(message)));
        }
    };
    assert_eq!(state.phase, crunch_release_core::PublicationPhase::Committed);
    assert!(state.failures.is_empty(), "committed release publication cannot carry failures");
    Ok(manifest)
}

fn assemble_release_manifest(
    request: &ReleaseBundleCreateRequest,
    stage_root: &ReleaseCapabilityRoot,
    proof_identity: FullSelfHostingProofIdentity,
    prepared_proof_bundle: PreparedTreeCopy,
    prepared_provider_fixed_point: Option<PreparedTreeCopy>,
) -> Result<ReleaseEvidenceManifest, RunError> {
    let source_archive = copy_file_into_bundle(
        &request.source_archive_path,
        stage_root,
        &request.bundle_dir,
        &bundle_file_destination("source", &request.source_archive_path, 0)?,
    )?;
    let binaries = copy_binary_set_into_bundle(&request.binary_paths, stage_root, &request.bundle_dir)?;
    let proof_bundle = copy_prepared_directory_into_bundle(
        prepared_proof_bundle,
        stage_root,
        &request.bundle_dir,
        Path::new("proof/self-hosting"),
    )?;
    let inventory_path = request.proof_bundle_dir.join(PROOF_INVENTORY_RELATIVE_PATH);
    let prerequisite_inventory =
        copy_file_into_bundle(&inventory_path, stage_root, &request.bundle_dir, Path::new("proof/inventory.md"))?;
    debug_assert_eq!(source_archive.kind, BundledArtifactKind::File);
    debug_assert_eq!(proof_bundle.kind, BundledArtifactKind::Directory);
    build_release_manifest(request, stage_root, ReleaseManifestInputs {
        proof_identity,
        prepared_provider_fixed_point,
        source_archive,
        binaries,
        proof_bundle,
        prerequisite_inventory,
    })
}

struct ReleaseManifestInputs {
    proof_identity: FullSelfHostingProofIdentity,
    prepared_provider_fixed_point: Option<PreparedTreeCopy>,
    source_archive: BundledArtifact,
    binaries: Vec<BundledArtifact>,
    proof_bundle: BundledArtifact,
    prerequisite_inventory: BundledArtifact,
}

struct ReleaseManifestEvidence {
    provider_fixed_point_proof: Option<ProviderFixedPointProofArtifact>,
    reproducibility_report: Option<BundledArtifact>,
    external_evidence: Vec<ExternalEvidence>,
    kani_toolchain_evidence: Vec<KaniToolchainEvidence>,
    stack_provenance: Option<StackProvenanceReleaseEvidence>,
    content_bound_requirement: Option<ContentBoundReleaseEvidenceV1>,
    cairn_handoff: Option<CairnReleaseEvidenceHandoff>,
}

fn build_release_manifest(
    request: &ReleaseBundleCreateRequest,
    stage_root: &ReleaseCapabilityRoot,
    mut inputs: ReleaseManifestInputs,
) -> Result<ReleaseEvidenceManifest, RunError> {
    let mut evidence = collect_release_manifest_evidence(request, stage_root, &mut inputs)?;
    let source_archive_digest_blake3 = inputs.source_archive.digest_blake3.clone();
    let source_acquisition = build_source_acquisition(request, &source_archive_digest_blake3);
    let cairn_handoff = evidence.cairn_handoff.take();
    let mut manifest = compose_release_manifest(request, inputs, evidence, source_acquisition);
    if let Some(handoff) = cairn_handoff {
        let binding = release_evidence_cairn_bundle_binding(&manifest).map_err(core_error_to_run_error)?;
        manifest.cairn_handoff_validation =
            Some(cairn_release_evidence_validation_receipt(binding, handoff).map_err(core_error_to_run_error)?);
    }
    debug_assert_eq!(manifest.schema, RELEASE_EVIDENCE_SCHEMA);
    debug_assert!(!manifest.binaries.is_empty());
    Ok(manifest)
}

fn collect_release_manifest_evidence(
    request: &ReleaseBundleCreateRequest,
    stage_root: &ReleaseCapabilityRoot,
    inputs: &mut ReleaseManifestInputs,
) -> Result<ReleaseManifestEvidence, RunError> {
    let content_bound_requirement = build_content_bound_requirement_for_release(request, &inputs.binaries)?;
    let provider_fixed_point_proof =
        copy_optional_provider_fixed_point_proof(request, stage_root, inputs.prepared_provider_fixed_point.take())?;
    let reproducibility_artifact = copy_optional_reproducibility_report(
        request,
        stage_root,
        &inputs.source_archive,
        &inputs.binaries,
        &inputs.proof_bundle,
    )?;
    let mut external_evidence = copy_external_evidence(request, stage_root)?;
    copy_content_bound_requirement_files(request, stage_root, &mut external_evidence)?;
    let stack_provenance =
        copy_optional_stack_provenance_evidence(request, stage_root, &mut external_evidence, &inputs.binaries)?;
    let kani_toolchain_evidence = build_kani_toolchain_evidence(request, &external_evidence)?;
    let cairn_handoff = request
        .cairn_handoff_descriptor_path
        .as_deref()
        .map(|descriptor| {
            crate::cairn_release_handoff::prepare_cairn_handoff_for_bundle(descriptor, &request.bundle_dir)
        })
        .transpose()?;
    let stack_provenance_artifact_count =
        stack_provenance.as_ref().map_or(0, |_| STACK_PROVENANCE_EXTERNAL_ARTIFACT_COUNT);
    let content_bound_artifact_count =
        request.content_bound_requirement.as_ref().map_or(0, |content_bound| content_bound.files.len());
    let expected_external_evidence_count = request
        .external_evidence
        .len()
        .saturating_add(content_bound_artifact_count)
        .saturating_add(stack_provenance_artifact_count);
    debug_assert_eq!(external_evidence.len(), expected_external_evidence_count);
    debug_assert_eq!(kani_toolchain_evidence.len(), request.kani_toolchain_evidence.len());
    Ok(ReleaseManifestEvidence {
        provider_fixed_point_proof,
        reproducibility_report: reproducibility_artifact,
        external_evidence,
        kani_toolchain_evidence,
        stack_provenance,
        content_bound_requirement,
        cairn_handoff,
    })
}

fn compose_release_manifest(
    request: &ReleaseBundleCreateRequest,
    inputs: ReleaseManifestInputs,
    evidence: ReleaseManifestEvidence,
    source_acquisition: Option<SourceAcquisition>,
) -> ReleaseEvidenceManifest {
    ReleaseEvidenceManifest {
        schema: RELEASE_EVIDENCE_SCHEMA.to_string(),
        release_id: request.release_id.clone(),
        claim_scope: CLAIM_SCOPE_PACKAGED_INTEGRITY.to_string(),
        workflow: ReleaseWorkflowIdentity {
            command: request.workflow_command.clone(),
            version: request.workflow_version.clone(),
        },
        source_archive: inputs.source_archive.clone(),
        source_acquisition,
        binaries: inputs.binaries,
        proof_bundle: inputs.proof_bundle,
        prerequisite_inventory: inputs.prerequisite_inventory,
        provider_fixed_point_proof: evidence.provider_fixed_point_proof,
        reproducibility_report: evidence.reproducibility_report,
        deterministic_build_proof: None,
        deterministic_sandbox_isolation_evidence: None,
        independent_agreement_report: None,
        external_evidence: evidence.external_evidence,
        kani_toolchain_evidence: evidence.kani_toolchain_evidence,
        stack_provenance: evidence.stack_provenance,
        opaque_evidence_sidecar_bindings: vec![],
        cairn_handoff_validation: None,
        function_address_evidence: None,
        proof_linkage: ReleaseProofLinkage {
            release_id: request.release_id.clone(),
            source_archive_digest_blake3: inputs.source_archive.digest_blake3,
            proof_bundle_schema: inputs.proof_identity.schema,
            proof_mode: inputs.proof_identity.proof_mode,
            selected_provider_kind: inputs.proof_identity.selected_provider_kind,
            staged_source: inputs.proof_identity.staged_source,
            stage2_binary_digest_blake3: inputs.proof_identity.stage2_binary_digest_blake3,
            prerequisite_inventory_digest_blake3: inputs.proof_identity.prerequisite_inventory_digest_blake3,
            proof_manifest_digest_blake3: inputs.proof_identity.proof_manifest_digest_blake3,
        },
        provenance_coverage: None,
        content_bound_requirement_evidence: evidence.content_bound_requirement,
    }
}

fn manifest_publication_artifacts(
    manifest: &ReleaseEvidenceManifest,
    bundle_dir: &Path,
) -> Result<Vec<PublicationArtifactInput>, RunError> {
    let mut artifacts = vec![publication_artifact_input(&manifest.source_archive)];
    artifacts.extend(manifest.binaries.iter().map(publication_artifact_input));
    artifacts.push(publication_artifact_input(&manifest.proof_bundle));
    artifacts.push(publication_artifact_input(&manifest.prerequisite_inventory));
    if let Some(provider) = &manifest.provider_fixed_point_proof {
        artifacts.push(PublicationArtifactInput {
            relative_path: provider.relative_path.clone(),
            kind: provider.kind,
            size_bytes: provider.size_bytes,
            digest_blake3: provider.digest_blake3.clone(),
        });
    }
    if let Some(report) = &manifest.reproducibility_report {
        artifacts.push(publication_artifact_input(report));
    }
    if let Some(receipt) = &manifest.cairn_handoff_validation {
        artifacts.push(publication_input_from_cairn_artifact(&receipt.handoff.authentication.archive_receipt));
        for row in &receipt.handoff.rows {
            artifacts.push(publication_input_from_cairn_artifact(&row.artifact));
            artifacts.push(publication_input_from_cairn_artifact(&row.cairn_policy));
        }
    }
    for evidence in &manifest.external_evidence {
        let path = bundle_dir.join(&evidence.relative_path);
        let measured = build_artifact_record(&path, Path::new(&evidence.relative_path), BundledArtifactKind::File)?;
        artifacts.push(publication_artifact_input(&measured));
    }
    debug_assert!(!artifacts.is_empty());
    debug_assert!(artifacts.capacity() >= artifacts.len());
    Ok(artifacts)
}

fn publication_input_from_cairn_artifact(
    artifact: &crunch_release_core::CairnMeasuredArtifact,
) -> PublicationArtifactInput {
    PublicationArtifactInput {
        relative_path: artifact.relative_path.clone(),
        kind: BundledArtifactKind::File,
        size_bytes: artifact.size_bytes,
        digest_blake3: artifact.measured_digest_blake3.clone(),
    }
}

fn write_manifest_bytes(stage_root: &ReleaseCapabilityRoot, bytes: &[u8]) -> Result<(), RunError> {
    let path = ValidatedReleasePath::new("manifest.json")
        .map_err(|error| RunError::Internal(format!("invalid release manifest path: {error:?}")))?;
    stage_root
        .write_new_file_nofollow(&path, bytes)
        .map_err(|error| RunError::Internal(format!("writing capability-scoped release manifest: {error}")))?;
    assert!(!bytes.is_empty(), "canonical release manifest must not be empty");
    assert!(stage_root.kind() == crate::release_capability::ReleaseRootKind::ReleaseEvidence);
    Ok(())
}

fn observe_publication_phase(
    shell: &mut dyn PublicationShellAdapter,
    stage: &ReleasePublicationStage,
    plan: &PublicationPlan,
    state: PublicationState,
    phase: PublicationShellPhase,
) -> Result<PublicationState, PublicationAttemptFailure> {
    let context = PublicationShellContext {
        stage_path: stage.stage_path(),
        final_path: stage.final_path(),
        plan_identity_blake3: &plan.plan_identity_blake3,
    };
    let failure_phase = shell_failure_phase(phase);
    shell
        .after_phase(phase, &context)
        .map_err(|error| fail_publication(state.clone(), failure_phase, error))?;
    Ok(state)
}

fn shell_failure_phase(phase: PublicationShellPhase) -> PublicationFailurePhase {
    match phase {
        PublicationShellPhase::StageCreated => PublicationFailurePhase::StageCreation,
        PublicationShellPhase::InputsCopied => PublicationFailurePhase::InputCopy,
        PublicationShellPhase::ArtifactsHashed => PublicationFailurePhase::ArtifactHashing,
        PublicationShellPhase::ManifestSerialized => PublicationFailurePhase::ManifestSerialization,
        PublicationShellPhase::ManifestWritten => PublicationFailurePhase::ManifestWrite,
        PublicationShellPhase::StagedVerified => PublicationFailurePhase::Verification,
        PublicationShellPhase::PreCommit => PublicationFailurePhase::PreCommit,
        PublicationShellPhase::Cleanup => PublicationFailurePhase::Cleanup,
    }
}

fn advance_publication(
    state: PublicationState,
    event: PublicationEvent,
) -> Result<PublicationState, PublicationAttemptFailure> {
    transition_publication_state(state.clone(), event).map_err(|error| PublicationAttemptFailure {
        state,
        error: RunError::Internal(format!(
            "release publication state transition rejected at {:?} for {:?}",
            error.phase, error.event
        )),
    })
}

fn fail_publication(
    state: PublicationState,
    phase: PublicationFailurePhase,
    error: RunError,
) -> PublicationAttemptFailure {
    let original_state = state.clone();
    let failed_state = match transition_publication_state(state, PublicationEvent::Failed(phase)) {
        Ok(failed_state) => failed_state,
        Err(transition_error) => {
            return PublicationAttemptFailure {
                state: original_state,
                error: RunError::Internal(format!(
                    "release publication failure transition rejected at {:?} for {:?}: {error}",
                    transition_error.phase, transition_error.event
                )),
            };
        }
    };
    PublicationAttemptFailure {
        state: failed_state,
        error: publication_phase_error(phase, error),
    }
}

fn publication_phase_error(phase: PublicationFailurePhase, error: RunError) -> RunError {
    RunError::Internal(format!("release publication phase {phase:?} failed: {error}"))
}

fn cleanup_failed_publication(
    stage: &ReleasePublicationStage,
    failure: PublicationAttemptFailure,
    shell: &mut dyn PublicationShellAdapter,
) -> Result<ReleaseEvidenceManifest, RunError> {
    let context = PublicationShellContext {
        stage_path: stage.stage_path(),
        final_path: stage.final_path(),
        plan_identity_blake3: &failure.state.plan_identity_blake3,
    };
    if let Err(cleanup_error) = shell.after_phase(PublicationShellPhase::Cleanup, &context) {
        let cleanup_state = transition_publication_state(
            failure.state.clone(),
            PublicationEvent::Failed(PublicationFailurePhase::Cleanup),
        )
        .map_err(|transition_error| {
            RunError::Internal(format!(
                "{}; cleanup failure state transition rejected at {:?} for {:?}",
                failure.error, transition_error.phase, transition_error.event
            ))
        })?;
        assert_eq!(cleanup_state.phase, crunch_release_core::PublicationPhase::Failed);
        assert_eq!(cleanup_state.failures.last(), Some(&PublicationFailurePhase::Cleanup));
        let combined = format!("{}; release publication cleanup failed: {cleanup_error}", failure.error);
        return Err(RunError::Internal(combined));
    }
    if let Err(cleanup_error) = stage.cleanup_current_stage() {
        let cleanup_state = transition_publication_state(
            failure.state.clone(),
            PublicationEvent::Failed(PublicationFailurePhase::Cleanup),
        )
        .map_err(|transition_error| {
            RunError::Internal(format!(
                "{}; filesystem cleanup failure state transition rejected at {:?} for {:?}",
                failure.error, transition_error.phase, transition_error.event
            ))
        })?;
        assert_eq!(cleanup_state.phase, crunch_release_core::PublicationPhase::Failed);
        assert_eq!(cleanup_state.failures.last(), Some(&PublicationFailurePhase::Cleanup));
        let combined = format!("{}; release publication cleanup failed: {cleanup_error}", failure.error);
        return Err(RunError::Internal(combined));
    }
    Err(failure.error)
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
    verify_cairn_handoff_bundle_bytes(&manifest, bundle_dir)?;
    Ok(manifest)
}

fn verify_cairn_handoff_bundle_bytes(manifest: &ReleaseEvidenceManifest, bundle_dir: &Path) -> Result<(), RunError> {
    let Some(receipt) = &manifest.cairn_handoff_validation else {
        return Ok(());
    };
    let measured = crate::cairn_release_handoff::remeasure_cairn_handoff_from_bundle(bundle_dir, receipt)?;
    let binding = release_evidence_cairn_bundle_binding(manifest).map_err(core_error_to_run_error)?;
    validate_cairn_release_evidence_validation_receipt(receipt, &binding, &measured).map_err(core_error_to_run_error)
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
    debug_assert!(!manifest.schema.is_empty());
    debug_assert!(!proof_manifest_digest_blake3.is_empty());
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

/// Attach the optional versioned source-observation binding.
fn attach_source_observation(
    acquisition: SourceAcquisition,
    request: &ReleaseBundleCreateRequest,
) -> SourceAcquisition {
    match &request.source_observation {
        Some(binding) => {
            debug_assert!(!binding.observation_blake3.is_empty());
            acquisition.with_source_observation(binding.clone())
        }
        None => acquisition,
    }
}

fn build_source_acquisition(
    request: &ReleaseBundleCreateRequest,
    source_archive_digest_blake3: &str,
) -> Option<SourceAcquisition> {
    if let Some(git_source) = &request.git_source {
        let acquisition = SourceAcquisition::git(
            git_source.remote_url.clone(),
            git_source.commit.clone(),
            git_source.reference.clone(),
            git_source.tag.clone(),
            source_archive_digest_blake3.to_string(),
        );
        return Some(attach_source_observation(acquisition, request));
    }
    request.source_acquisition_url.as_ref().map(|url| {
        let acquisition = SourceAcquisition::external_archive(url.clone(), source_archive_digest_blake3.to_string());
        attach_source_observation(acquisition, request)
    })
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
    if let Some(report_path) = &request.reproducibility_report_path
        && !report_path.is_file()
    {
        return Err(RunError::Internal(format!(
            "release evidence reproducibility report is missing: {}",
            report_path.display()
        )));
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
    if let Some(descriptor) = &request.cairn_handoff_descriptor_path
        && !descriptor.is_file()
    {
        return Err(RunError::Internal(format!(
            "release evidence Cairn handoff descriptor is missing: {}",
            descriptor.display()
        )));
    }
    debug_assert!(!request.release_id.trim().is_empty());
    debug_assert!(binary_count > 0);
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
    if let Some(binary_path) = &request.binary_path
        && !binary_path.is_file()
    {
        return Err(RunError::Internal(format!(
            "release evidence stack provenance binary is missing: {}",
            binary_path.display()
        )));
    }
    debug_assert!(request.sidecar_path.is_file());
    debug_assert!(request.valence_receipt_path.is_file());
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
    debug_assert!(!evidence.role.trim().is_empty());
    debug_assert!(!evidence.non_claims.is_empty());
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
    debug_assert!(!evidence.receipt_role.trim().is_empty());
    debug_assert!(!evidence.non_claims.is_empty());
    Ok(())
}

fn validate_source_acquisition_request(request: &ReleaseBundleCreateRequest) -> Result<(), RunError> {
    if request.source_acquisition_url.is_some() && request.git_source.is_some() {
        return Err(RunError::Internal(
            "release evidence Git source metadata conflicts with external source acquisition URL".to_string(),
        ));
    }
    if let Some(url) = &request.source_acquisition_url
        && url.trim().is_empty()
    {
        return Err(RunError::Internal("release evidence source acquisition URL must not be empty".to_string()));
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

fn copy_binary_set_into_bundle(
    binary_paths: &[PathBuf],
    bundle_root: &ReleaseCapabilityRoot,
    bundle_dir: &Path,
) -> Result<Vec<BundledArtifact>, RunError> {
    let mut bundled = Vec::with_capacity(binary_paths.len());
    for (index_usize, binary_path) in binary_paths.iter().enumerate() {
        let index_u32: u32 = index_usize
            .try_into()
            .map_err(|_| RunError::Internal("release evidence binary index overflowed u32".to_string()))?;
        let artifact_index = index_u32
            .checked_add(1)
            .ok_or_else(|| RunError::Internal("release evidence binary artifact index overflowed u32".to_string()))?;
        let relative = bundle_file_destination("binaries", binary_path, artifact_index)?;
        bundled.push(copy_file_into_bundle(binary_path, bundle_root, bundle_dir, &relative)?);
    }
    assert!(!bundled.is_empty(), "binary bundle copy must emit at least one artifact");
    assert_eq!(bundled.len(), binary_paths.len());
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
        let artifact_index = index_u32
            .checked_add(1)
            .ok_or_else(|| RunError::Internal("release evidence binary artifact index overflowed u32".to_string()))?;
        let relative = bundle_file_destination("binaries", binary_path, artifact_index)?;
        artifacts.push(build_artifact_record(binary_path, &relative, BundledArtifactKind::File)?);
    }
    Ok(artifacts)
}

fn copy_optional_provider_fixed_point_proof(
    request: &ReleaseBundleCreateRequest,
    bundle_root: &ReleaseCapabilityRoot,
    prepared: Option<PreparedTreeCopy>,
) -> Result<Option<ProviderFixedPointProofArtifact>, RunError> {
    if request.provider_fixed_point_proof_dir.is_none() {
        assert!(prepared.is_none(), "absent provider proof must not have a prepared copy plan");
        return Ok(None);
    }
    let prepared = prepared.ok_or_else(|| {
        RunError::Internal("provider fixed-point proof copy plan was not prepared before mutation".to_string())
    })?;
    let artifact = copy_prepared_directory_into_bundle(
        prepared,
        bundle_root,
        &request.bundle_dir,
        Path::new("proof/provider-fixed-point"),
    )?;
    debug_assert_eq!(artifact.kind, BundledArtifactKind::Directory);
    debug_assert_eq!(artifact.relative_path, "proof/provider-fixed-point");
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
    bundle_root: &ReleaseCapabilityRoot,
    source_archive: &BundledArtifact,
    binaries: &[BundledArtifact],
    proof_bundle: &BundledArtifact,
) -> Result<Option<BundledArtifact>, RunError> {
    let Some(report_path) = &request.reproducibility_report_path else {
        return Ok(None);
    };
    validate_reproducibility_report_for_bundle(request, report_path, source_archive, binaries, proof_bundle)?;
    let relative = Path::new("reproducibility").join("reproducibility-report.json");
    copy_file_into_bundle(report_path, bundle_root, &request.bundle_dir, &relative).map(Some)
}

fn build_content_bound_requirement_for_release(
    request: &ReleaseBundleCreateRequest,
    binaries: &[BundledArtifact],
) -> Result<Option<ContentBoundReleaseEvidenceV1>, RunError> {
    let Some(content_bound) = &request.content_bound_requirement else {
        return Ok(None);
    };
    let build = ContentBoundReleaseBuildInputV1 {
        input: content_bound.input.clone(),
        evidence_rows: content_bound.files.iter().map(|file| file.row.clone()).collect(),
        release_id: request.release_id.clone(),
        binary_digests_blake3: binaries.iter().map(|binary| binary.digest_blake3.clone()).collect(),
    };
    build_content_bound_release_evidence(build).map(Some).map_err(|issues| {
        let first = issues
            .first()
            .map(|issue| format!("{:?} at {}", issue.code, issue.field_path))
            .unwrap_or_else(|| "unknown content-bound requirement issue".to_string());
        RunError::Internal(format!("building content-bound release requirement evidence: {first}"))
    })
}

fn copy_content_bound_requirement_files(
    request: &ReleaseBundleCreateRequest,
    bundle_root: &ReleaseCapabilityRoot,
    external_evidence: &mut Vec<ExternalEvidence>,
) -> Result<(), RunError> {
    let Some(content_bound) = &request.content_bound_requirement else {
        return Ok(());
    };
    for file in &content_bound.files {
        let relative = ValidatedReleasePath::new(&file.row.bundle_relative_path)
            .map_err(|error| RunError::Internal(format!("content-bound bundle path is invalid: {error:?}")))?;
        bundle_root.write_new_file_nofollow(&relative, &file.bytes).map_err(|error| {
            RunError::Internal(format!("writing content-bound evidence file into release stage: {error}"))
        })?;
        let measured_digest = blake3::hash(&file.bytes).to_hex().to_string();
        assert_eq!(measured_digest, file.row.content_identity.blake3);
        external_evidence.push(ExternalEvidence {
            role: file.row.role.external_role().to_string(),
            schema: CONTENT_BOUND_EVIDENCE_EXTERNAL_SCHEMA_V1.to_string(),
            relative_path: file.row.bundle_relative_path.clone(),
            digest_blake3: file.row.content_identity.blake3.clone(),
            claim_scope: CONTENT_BOUND_EVIDENCE_CLAIM_SCOPE.to_string(),
            non_claims: file.row.non_claims.clone(),
        });
    }
    Ok(())
}

fn copy_external_evidence(
    request: &ReleaseBundleCreateRequest,
    bundle_root: &ReleaseCapabilityRoot,
) -> Result<Vec<ExternalEvidence>, RunError> {
    let mut records = Vec::with_capacity(request.external_evidence.len());
    for evidence in &request.external_evidence {
        let next_index = next_external_evidence_index(records.len())?;
        records.push(copy_external_evidence_record(request, bundle_root, evidence, next_index)?);
    }
    Ok(records)
}

fn next_external_evidence_index(existing_count: usize) -> Result<u32, RunError> {
    let index: u32 = existing_count
        .try_into()
        .map_err(|_| RunError::Internal("release evidence external evidence index overflowed u32".to_string()))?;
    index
        .checked_add(1)
        .ok_or_else(|| RunError::Internal("release evidence external artifact index overflowed u32".to_string()))
}

fn copy_external_evidence_record(
    request: &ReleaseBundleCreateRequest,
    bundle_root: &ReleaseCapabilityRoot,
    evidence: &ExternalEvidenceCreateRequest,
    index: u32,
) -> Result<ExternalEvidence, RunError> {
    let relative = bundle_file_destination("external-evidence", &evidence.path, index)?;
    let artifact = copy_file_into_bundle(&evidence.path, bundle_root, &request.bundle_dir, &relative)?;
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
    bundle_root: &ReleaseCapabilityRoot,
    external_evidence: &mut Vec<ExternalEvidence>,
    binaries: &[BundledArtifact],
) -> Result<Option<StackProvenanceReleaseEvidence>, RunError> {
    let Some(stack_request) = &request.stack_provenance else {
        return Ok(None);
    };
    let evidence_before_count = external_evidence.len();
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
    let sidecar_artifact = copy_external_evidence_record(
        request,
        bundle_root,
        &sidecar,
        next_external_evidence_index(external_evidence.len())?,
    )?;
    external_evidence.push(sidecar_artifact.clone());
    let receipt_artifact = copy_external_evidence_record(
        request,
        bundle_root,
        &valence_receipt,
        next_external_evidence_index(external_evidence.len())?,
    )?;
    external_evidence.push(receipt_artifact.clone());
    let expected_evidence_count = evidence_before_count.saturating_add(STACK_PROVENANCE_EXTERNAL_ARTIFACT_COUNT);
    debug_assert_eq!(external_evidence.len(), expected_evidence_count);
    debug_assert!(external_evidence.capacity() >= external_evidence.len());
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
        let binary = binaries.get(binary_index).ok_or_else(|| {
            RunError::Internal(
                "release evidence stack provenance binary index did not match copied binaries".to_string(),
            )
        })?;
        debug_assert!(binary_index < binaries.len());
        debug_assert!(!binary.relative_path.is_empty());
        return Ok(binary);
    }
    if binaries.len() == 1 {
        debug_assert_eq!(binaries.len(), 1);
        debug_assert!(!binaries[0].relative_path.is_empty());
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
    debug_assert_eq!(records.len(), request.kani_toolchain_evidence.len());
    debug_assert!(records.capacity() >= records.len());
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
    let reproducibility_evidence: ReleaseReproducibilityReport = serde_json::from_slice(&report_bytes)
        .map_err(|err| RunError::Internal(format!("parsing {}: {err}", report_path.display())))?;
    let canonical_bytes = release_reproducibility_report_canonical_bytes(reproducibility_evidence.clone())
        .map_err(core_error_to_run_error)?;
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
    let linked_evidence = validate_release_reproducibility_report_linkage(reproducibility_evidence, expected)
        .map_err(core_error_to_run_error)?;
    let expected_names = binaries.iter().map(|artifact| artifact.relative_path.clone()).collect::<Vec<_>>();
    let expected_name_count = expected_names.len();
    validate_release_reproducibility_report_artifact_names(linked_evidence, expected_names)
        .map_err(core_error_to_run_error)?;
    debug_assert_eq!(report_bytes, canonical_bytes);
    debug_assert_eq!(expected_name_count, binaries.len());
    Ok(())
}

pub(crate) fn copy_file_into_bundle(
    source_path: &Path,
    bundle_root: &ReleaseCapabilityRoot,
    bundle_dir: &Path,
    relative_path: &Path,
) -> Result<BundledArtifact, RunError> {
    crate::release_tree_copy::copy_file_into_capability(source_path, bundle_root, relative_path)?;
    let dest_path = bundle_dir.join(relative_path);
    let artifact = build_artifact_record(&dest_path, relative_path, BundledArtifactKind::File)?;
    assert_eq!(bundle_root.kind(), crate::release_capability::ReleaseRootKind::ReleaseEvidence);
    assert_eq!(artifact.relative_path, path_to_forward_slash_string(relative_path)?);
    Ok(artifact)
}

fn copy_prepared_directory_into_bundle(
    prepared: PreparedTreeCopy,
    bundle_root: &ReleaseCapabilityRoot,
    bundle_dir: &Path,
    relative_path: &Path,
) -> Result<BundledArtifact, RunError> {
    let dest_dir = bundle_dir.join(relative_path);
    crate::release_tree_copy::copy_prepared_tree_into_capability(prepared, bundle_root, relative_path)?;
    let artifact = build_artifact_record(&dest_dir, relative_path, BundledArtifactKind::Directory)?;
    assert_eq!(bundle_root.kind(), crate::release_capability::ReleaseRootKind::ReleaseEvidence);
    assert_eq!(artifact.kind, BundledArtifactKind::Directory);
    Ok(artifact)
}

pub(crate) fn copy_directory_tree(source_dir: &Path, dest_dir: &Path) -> Result<(), RunError> {
    crate::release_tree_copy::copy_directory_tree(source_dir, dest_dir)
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
    let metadata = std::fs::symlink_metadata(path).map_err(|error| {
        RunError::Internal(format!("reading no-follow artifact metadata {}: {error}", path.display()))
    })?;
    if metadata.file_type().is_symlink() {
        return Err(RunError::Internal(format!("top-level artifact must not be a symlink: {}", path.display())));
    }
    if metadata.is_file() {
        return hash_file(path).map(|(_size_bytes, digest_blake3)| digest_blake3);
    }
    if metadata.is_dir() {
        return hash_directory(path).map(|(_size_bytes, digest_blake3)| digest_blake3);
    }
    Err(RunError::Internal(format!("expected file or directory artifact: {}", path.display())))
}

fn hash_file(path: &Path) -> Result<(u64, String), RunError> {
    crate::release_tree_copy::hash_file_nofollow(path)
}

fn hash_directory(path: &Path) -> Result<(u64, String), RunError> {
    crate::release_tree_copy::hash_directory_tree(path)
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
    debug_assert!(!manifest.binaries.is_empty());
    debug_assert_eq!(manifest.source_archive.kind, BundledArtifactKind::File);
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
    debug_assert_eq!(manifest.proof_linkage.release_id, manifest.release_id);
    debug_assert_eq!(proof_identity.proof_mode, manifest.proof_linkage.proof_mode);
    Ok(())
}

fn path_to_forward_slash_string(path: &Path) -> Result<String, RunError> {
    let raw = path
        .to_str()
        .ok_or_else(|| RunError::Internal(format!("non-utf8 bundle path: {}", path.display())))?;
    Ok(raw.replace('\\', "/"))
}

#[cfg(test)]
pub(crate) mod tests {
    use crunch_release_core::KANI_NON_CLAIM_RELEASE_ELIGIBILITY;
    use crunch_release_core::KANI_NON_CLAIM_SEMANTICS;
    use crunch_release_core::KANI_NON_CLAIM_VERIFIER_SOUNDNESS;
    use crunch_release_core::KANI_NON_CLAIM_WHOLE_PROGRAM;
    use crunch_release_core::KANI_RECEIPT_EVIDENCE_ROLE;
    use crunch_release_core::KANI_SOLVER_KIND_CBMC_DEFAULT;
    use serde_json::json;

    use super::*;
    use crate::content_bound_requirement_evidence::CONTENT_BOUND_CREATE_FILE_SCHEMA_V1;
    use crate::content_bound_requirement_evidence::load_content_bound_requirement_create_request;

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
            opaque_evidence_sidecar_bindings: vec![],
            cairn_handoff_validation: None,
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
            content_bound_requirement_evidence: None,
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

    const TEST_BUNDLED_BINARY_RELATIVE_PATH: &str = "binaries/01-mantle";
    const TEST_QUARANTINE_PREFIX: &str = ".mantle-release-quarantine-";
    const TEST_UNRECOGNIZED_STAGE_MARKER_BYTES: &[u8] = b"not-a-valid-ownership-marker";
    const TEST_FAILPOINTS_COUNT: usize = 5;

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct PublicationPhaseObservation {
        phase: PublicationShellPhase,
        stage_path: PathBuf,
        final_exists: bool,
        stage_manifest_exists: bool,
        plan_identity_blake3: String,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum ConcurrentWinnerKind {
        File,
        Directory,
        #[cfg(unix)]
        Symlink,
    }

    #[derive(Default)]
    struct TestPublicationShell {
        fail_phases: Vec<PublicationShellPhase>,
        corrupt_before_verification: bool,
        corrupt_before_commit: bool,
        concurrent_winner: Option<ConcurrentWinnerKind>,
        mutate_source_after_plan: Option<PathBuf>,
        observations: Vec<PublicationPhaseObservation>,
    }

    impl PublicationShellAdapter for TestPublicationShell {
        fn after_phase(
            &mut self,
            phase: PublicationShellPhase,
            context: &PublicationShellContext<'_>,
        ) -> Result<(), RunError> {
            if phase == PublicationShellPhase::StageCreated
                && let Some(source) = &self.mutate_source_after_plan
            {
                std::fs::write(source, b"source-changed-after-plan").unwrap();
            }
            if phase == PublicationShellPhase::ManifestWritten && self.corrupt_before_verification {
                std::fs::write(context.stage_path.join(TEST_BUNDLED_BINARY_RELATIVE_PATH), b"tampered-stage").unwrap();
            }
            if phase == PublicationShellPhase::PreCommit {
                if self.corrupt_before_commit {
                    std::fs::write(context.stage_path.join(TEST_BUNDLED_BINARY_RELATIVE_PATH), b"tampered-pre-commit")
                        .unwrap();
                }
                if let Some(winner) = self.concurrent_winner {
                    install_concurrent_winner(context.final_path, winner);
                }
            }
            self.observations.push(PublicationPhaseObservation {
                phase,
                stage_path: context.stage_path.to_path_buf(),
                final_exists: context.final_path.exists(),
                stage_manifest_exists: context.stage_path.join("manifest.json").is_file(),
                plan_identity_blake3: context.plan_identity_blake3.to_string(),
            });
            if self.fail_phases.contains(&phase) {
                return Err(RunError::Internal(format!("injected release publication failure after {phase:?}")));
            }
            Ok(())
        }
    }

    fn install_concurrent_winner(path: &Path, winner: ConcurrentWinnerKind) {
        match winner {
            ConcurrentWinnerKind::File => std::fs::write(path, b"concurrent-winner-file").unwrap(),
            ConcurrentWinnerKind::Directory => std::fs::create_dir(path).unwrap(),
            #[cfg(unix)]
            ConcurrentWinnerKind::Symlink => std::os::unix::fs::symlink("concurrent-winner-target", path).unwrap(),
        }
    }

    pub(crate) fn publication_fixture(temp: &Path, release_id: &str, bundle_name: &str) -> ReleaseBundleCreateRequest {
        let input_root = temp.join(format!("inputs-{bundle_name}"));
        let source_archive = input_root.join("mantle-src.tar");
        let binary_path = input_root.join("mantle");
        let proof_bundle_dir = input_root.join("proof-input");
        write_file(&source_archive, b"source-archive");
        write_file(&binary_path, b"mantle-binary");
        let inventory_digest = blake3::hash(b"inventory").to_hex().to_string();
        let stage2_digest = blake3::hash(b"mantle-binary").to_hex().to_string();
        write_full_proof_manifest(&proof_bundle_dir, &inventory_digest, &stage2_digest);
        ReleaseBundleCreateRequest::with_defaults(
            release_id.to_string(),
            temp.join(bundle_name),
            source_archive,
            vec![binary_path],
            proof_bundle_dir,
        )
    }

    fn content_bound_create_request(
        temp: &Path,
    ) -> crate::content_bound_requirement_evidence::ContentBoundRequirementCreateRequest {
        let root = temp.join("content-bound-input");
        std::fs::create_dir_all(&root).expect("create content-bound input root");
        write_file(&root.join("source.rs"), b"content-bound source evidence");
        write_file(&root.join("test.rs"), b"content-bound test evidence");
        write_file(&root.join("receipt.json"), b"content-bound producer receipt");
        let registry: serde_json::Value =
            serde_json::from_str(include_str!("../fixtures/content-bound-requirements/mantle-registry.json"))
                .expect("parse frozen Mantle registry");
        let reference: serde_json::Value =
            serde_json::from_str(include_str!("../fixtures/content-bound-requirements/mantle-requirement-ref.json"))
                .expect("parse frozen Mantle requirement reference");
        let reference_identity = reference["reference_identity"]["blake3"].as_str().expect("reference identity");
        let release_non_claims = crunch_release_core::CONTENT_BOUND_RELEASE_NON_CLAIMS
            .iter()
            .map(|claim| (*claim).to_string())
            .collect::<Vec<_>>();
        let row_non_claims = crunch_release_core::CONTENT_BOUND_EVIDENCE_ROW_NON_CLAIMS
            .iter()
            .map(|claim| (*claim).to_string())
            .collect::<Vec<_>>();
        let input = json!({
            "schema": CONTENT_BOUND_CREATE_FILE_SCHEMA_V1,
            "release_input": {
                "schema": crunch_release_core::CONTENT_BOUND_RELEASE_INPUT_SCHEMA_V1,
                "registries": [registry],
                "requirement_refs": [reference],
                "coverage": [
                    {
                        "requirement_reference_blake3": reference_identity,
                        "evidence_role": "source",
                        "evidence_repository_paths": ["src/release_evidence.rs"]
                    },
                    {
                        "requirement_reference_blake3": reference_identity,
                        "evidence_role": "test",
                        "evidence_repository_paths": ["tests/release_cli.rs"]
                    }
                ],
                "non_claims": release_non_claims
            },
            "evidence_files": [
                {
                    "locator": "receipt.json",
                    "repository_id": "OnixResearch/mantle",
                    "repository_relative_path": "evidence/content-bound/producer-receipt.json",
                    "bundle_relative_path": "requirement-evidence/003-producer-receipt.json",
                    "role": "receipt",
                    "non_claims": row_non_claims
                },
                {
                    "locator": "source.rs",
                    "repository_id": "OnixResearch/mantle",
                    "repository_relative_path": "src/release_evidence.rs",
                    "bundle_relative_path": "requirement-evidence/001-release-evidence.rs",
                    "role": "source",
                    "symbol": "create_release_evidence_bundle",
                    "producer_receipt_repository_path": "evidence/content-bound/producer-receipt.json",
                    "non_claims": row_non_claims
                },
                {
                    "locator": "test.rs",
                    "repository_id": "OnixResearch/mantle",
                    "repository_relative_path": "tests/release_cli.rs",
                    "bundle_relative_path": "requirement-evidence/002-release-cli.rs",
                    "role": "test",
                    "producer_receipt_repository_path": "evidence/content-bound/producer-receipt.json",
                    "non_claims": row_non_claims
                }
            ]
        });
        std::fs::write(
            root.join("input.json"),
            serde_json::to_vec_pretty(&input).expect("serialize content-bound input"),
        )
        .expect("write content-bound input");
        load_content_bound_requirement_create_request(&root, "input.json").expect("load content-bound create request")
    }

    fn parent_entry_names(path: &Path) -> Vec<String> {
        let mut names = std::fs::read_dir(path)
            .unwrap()
            .map(|entry| entry.unwrap().file_name().into_string().unwrap())
            .collect::<Vec<_>>();
        names.sort();
        names
    }

    #[test]
    fn content_bound_requirement_files_create_verify_and_fail_closed() {
        // r[verify mantle.release_provenance.content_bound_evidence_manifest]
        let temp = tempfile::tempdir().expect("temp root");
        let mut request = publication_fixture(temp.path(), "mantle-content-bound-release", "content-bound-bundle");
        request.content_bound_requirement = Some(content_bound_create_request(temp.path()));
        let manifest = create_release_evidence_bundle(&request).expect("create content-bound release bundle");
        let verification = crunch_release_core::evaluate_content_bound_requirement_release_evidence(
            &manifest,
            crunch_release_core::CONTENT_BOUND_REQUIREMENT_MODE_REQUIRED,
        );
        assert!(verification.valid, "issues: {:?}", verification.issues);
        let verified =
            verify_release_evidence_bundle(&request.bundle_dir).expect("verify content-bound release bundle");
        assert!(verified.content_bound_requirement_evidence.is_some());

        std::fs::write(
            request.bundle_dir.join("requirement-evidence/001-release-evidence.rs"),
            b"stale content-bound evidence bytes",
        )
        .expect("tamper bundled evidence");
        let tampered =
            verify_release_evidence_bundle(&request.bundle_dir).expect_err("stale content-bound bytes must fail");
        let tampered_message = tampered.to_string();
        assert!(tampered_message.contains("does not match"), "unexpected tamper diagnostic: {tampered_message}");

        let mut invalid_request =
            publication_fixture(temp.path(), "mantle-content-bound-invalid", "content-bound-invalid-bundle");
        let mut invalid_content = content_bound_create_request(temp.path());
        invalid_content.input.registries[0].registry_blake3 =
            blake3::hash(b"stale registry identity").to_hex().to_string();
        invalid_request.content_bound_requirement = Some(invalid_content);
        let invalid =
            create_release_evidence_bundle(&invalid_request).expect_err("stale registry must fail before publication");
        assert!(invalid.to_string().contains("content-bound"));
        assert!(!invalid_request.bundle_dir.exists(), "failed strict release must not publish a partial bundle");
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

    // r[verify mantle.release_provenance.bundle_publication.fixtures.positive]
    // r[verify mantle.release_provenance.bundle_publication.validation.visibility]
    #[test]
    fn publication_production_path_is_manifest_last_and_atomically_visible() {
        let temp = tempfile::tempdir().unwrap();
        let request = publication_fixture(temp.path(), "mantle-atomic-positive", "release-bundle");
        let mut shell = TestPublicationShell::default();

        let created = create_release_evidence_bundle_with_adapter(&request, &mut shell).unwrap();
        let verified = verify_release_evidence_bundle(&request.bundle_dir).unwrap();
        let identity_count = shell
            .observations
            .iter()
            .map(|observation| &observation.plan_identity_blake3)
            .collect::<std::collections::BTreeSet<_>>()
            .len();

        assert_eq!(created, verified);
        assert_eq!(identity_count, 1);
        assert!(shell.observations.iter().all(|row| !row.final_exists));
        assert_eq!(shell.observations.last().unwrap().phase, PublicationShellPhase::PreCommit);
        assert!(request.bundle_dir.is_dir());
        assert!(
            !shell
                .observations
                .iter()
                .find(|row| row.phase == PublicationShellPhase::ManifestSerialized)
                .unwrap()
                .stage_manifest_exists
        );
        assert!(
            shell
                .observations
                .iter()
                .find(|row| row.phase == PublicationShellPhase::ManifestWritten)
                .unwrap()
                .stage_manifest_exists
        );
        assert!(!request.bundle_dir.join(crate::release_publication::RELEASE_STAGE_MARKER_FILENAME).exists());
    }

    // r[verify mantle.release_provenance.bundle_publication.failure_isolation]
    // r[verify mantle.release_provenance.bundle_publication.retry]
    #[test]
    fn named_phase_failpoints_leave_final_absent_and_fresh_retry_succeeds() {
        let phases: [PublicationShellPhase; TEST_FAILPOINTS_COUNT] = [
            PublicationShellPhase::InputsCopied,
            PublicationShellPhase::ArtifactsHashed,
            PublicationShellPhase::ManifestSerialized,
            PublicationShellPhase::StagedVerified,
            PublicationShellPhase::PreCommit,
        ];

        for (index, phase) in phases.into_iter().enumerate() {
            let temp = tempfile::tempdir().unwrap();
            let request = publication_fixture(temp.path(), &format!("mantle-failpoint-{index}"), "release-bundle");
            let mut failed_shell = TestPublicationShell {
                fail_phases: vec![phase],
                ..TestPublicationShell::default()
            };
            let error = create_release_evidence_bundle_with_adapter(&request, &mut failed_shell).unwrap_err();
            let failed_stage = failed_shell.observations[0].stage_path.clone();
            let failed_identity = failed_shell.observations[0].plan_identity_blake3.clone();

            assert!(error.to_string().contains("injected release publication failure"), "{phase:?}: {error}");
            assert!(!request.bundle_dir.exists(), "{phase:?}");
            assert!(!failed_stage.exists(), "{phase:?}");

            let mut retry_shell = TestPublicationShell::default();
            let retried = create_release_evidence_bundle_with_adapter(&request, &mut retry_shell).unwrap();
            let retry_stage = retry_shell.observations[0].stage_path.clone();
            let retry_identity = retry_shell.observations[0].plan_identity_blake3.clone();

            assert_eq!(retried.release_id, format!("mantle-failpoint-{index}"));
            assert_ne!(failed_stage, retry_stage, "retry must use a fresh random stage");
            assert_eq!(failed_identity, retry_identity, "random stage names must not affect plan identity");
            assert!(verify_release_evidence_bundle(&request.bundle_dir).is_ok());
        }
    }

    // r[verify mantle.release_provenance.bundle_publication.fixtures.negative.verification]
    #[test]
    fn staged_verification_failure_never_publishes_tampered_artifacts() {
        let temp = tempfile::tempdir().unwrap();
        let request = publication_fixture(temp.path(), "mantle-verification-failure", "release-bundle");
        let mut shell = TestPublicationShell {
            corrupt_before_verification: true,
            ..TestPublicationShell::default()
        };

        let error = create_release_evidence_bundle_with_adapter(&request, &mut shell).unwrap_err();

        assert!(error.to_string().contains("Verification"), "{error}");
        assert!(error.to_string().contains("does not match manifest"), "{error}");
        assert!(!request.bundle_dir.exists());
        assert!(shell.observations.iter().all(|row| !row.final_exists));
    }

    #[test]
    fn precommit_replacement_fails_before_publication() {
        let temp = tempfile::tempdir().unwrap();
        let request = publication_fixture(temp.path(), "mantle-precommit-replacement", "release-bundle");
        let mut shell = TestPublicationShell {
            corrupt_before_commit: true,
            ..TestPublicationShell::default()
        };

        let error = create_release_evidence_bundle_with_adapter(&request, &mut shell).unwrap_err();

        assert!(error.to_string().contains("PreCommit"));
        assert!(!request.bundle_dir.exists());
        assert!(shell.observations.iter().all(|row| !row.final_exists));
    }

    #[test]
    fn source_identity_drift_after_planning_fails_before_manifest_publication() {
        let temp = tempfile::tempdir().unwrap();
        let request = publication_fixture(temp.path(), "mantle-source-drift", "release-bundle");
        let mut shell = TestPublicationShell {
            mutate_source_after_plan: Some(request.source_archive_path.clone()),
            ..TestPublicationShell::default()
        };

        let error = create_release_evidence_bundle_with_adapter(&request, &mut shell).unwrap_err();

        assert!(error.to_string().contains("ArtifactHashing"), "{error}");
        assert!(error.to_string().contains("do not match the pre-mutation publication plan"), "{error}");
        assert!(!request.bundle_dir.exists());
    }

    // r[verify mantle.release_provenance.bundle_publication.fixtures.negative.race]
    #[test]
    fn preexisting_files_and_directories_are_unchanged_and_block_staging() {
        for destination_kind in ["file", "empty-directory", "nonempty-directory"] {
            let temp = tempfile::tempdir().unwrap();
            let request = publication_fixture(temp.path(), "mantle-preexisting", "release-bundle");
            match destination_kind {
                "file" => write_file(&request.bundle_dir, b"preexisting-file"),
                "empty-directory" => std::fs::create_dir(&request.bundle_dir).unwrap(),
                "nonempty-directory" => write_file(&request.bundle_dir.join("sentinel"), b"preexisting-directory"),
                _ => unreachable!(),
            }

            let error = create_release_evidence_bundle(&request).unwrap_err();

            assert!(error.to_string().contains("destination must be absent"), "{destination_kind}: {error}");
            match destination_kind {
                "file" => assert_eq!(std::fs::read(&request.bundle_dir).unwrap(), b"preexisting-file"),
                "empty-directory" => assert_eq!(std::fs::read_dir(&request.bundle_dir).unwrap().count(), 0),
                "nonempty-directory" => {
                    assert_eq!(std::fs::read(request.bundle_dir.join("sentinel")).unwrap(), b"preexisting-directory")
                }
                _ => unreachable!(),
            }
        }
    }

    #[test]
    #[cfg(unix)]
    fn preexisting_destination_symlink_is_unchanged_and_never_followed() {
        let temp = tempfile::tempdir().unwrap();
        let request = publication_fixture(temp.path(), "mantle-preexisting-symlink", "release-bundle");
        let target = temp.path().join("outside-target");
        write_file(&target.join("sentinel"), b"outside-sentinel");
        std::os::unix::fs::symlink(&target, &request.bundle_dir).unwrap();

        let error = create_release_evidence_bundle(&request).unwrap_err();

        assert!(error.to_string().contains("destination must be absent"), "{error}");
        assert_eq!(std::fs::read_link(&request.bundle_dir).unwrap(), target);
        assert_eq!(std::fs::read(target.join("sentinel")).unwrap(), b"outside-sentinel");
    }

    #[test]
    fn concurrent_file_and_directory_winners_are_never_clobbered() {
        for winner in [ConcurrentWinnerKind::File, ConcurrentWinnerKind::Directory] {
            let temp = tempfile::tempdir().unwrap();
            let request = publication_fixture(temp.path(), "mantle-commit-race", "release-bundle");
            let mut shell = TestPublicationShell {
                concurrent_winner: Some(winner),
                ..TestPublicationShell::default()
            };

            let error = create_release_evidence_bundle_with_adapter(&request, &mut shell).unwrap_err();

            assert!(error.to_string().contains("destination appeared before commit"), "{winner:?}: {error}");
            match winner {
                ConcurrentWinnerKind::File => {
                    assert_eq!(std::fs::read(&request.bundle_dir).unwrap(), b"concurrent-winner-file")
                }
                ConcurrentWinnerKind::Directory => {
                    assert_eq!(std::fs::read_dir(&request.bundle_dir).unwrap().count(), 0)
                }
                #[cfg(unix)]
                ConcurrentWinnerKind::Symlink => unreachable!(),
            }
            assert!(!request.bundle_dir.join("manifest.json").exists());
        }
    }

    #[test]
    #[cfg(unix)]
    fn concurrent_symlink_winner_is_never_replaced_or_followed() {
        let temp = tempfile::tempdir().unwrap();
        let request = publication_fixture(temp.path(), "mantle-symlink-race", "release-bundle");
        let mut shell = TestPublicationShell {
            concurrent_winner: Some(ConcurrentWinnerKind::Symlink),
            ..TestPublicationShell::default()
        };

        let error = create_release_evidence_bundle_with_adapter(&request, &mut shell).unwrap_err();

        assert!(error.to_string().contains("destination appeared before commit"), "{error}");
        assert_eq!(std::fs::read_link(&request.bundle_dir).unwrap(), Path::new("concurrent-winner-target"));
        assert!(std::fs::symlink_metadata(&request.bundle_dir).unwrap().file_type().is_symlink());
    }

    // r[verify mantle.release_provenance.bundle_publication.stale_stage]
    // r[verify mantle.release_provenance.bundle_publication.retry]
    #[test]
    fn cleanup_failure_is_retried_via_owned_stage_quarantine_only() {
        let temp = tempfile::tempdir().unwrap();
        let request = publication_fixture(temp.path(), "mantle-stale-stage", "release-bundle");
        let mut failed_shell = TestPublicationShell {
            fail_phases: vec![PublicationShellPhase::ArtifactsHashed, PublicationShellPhase::Cleanup],
            ..TestPublicationShell::default()
        };
        let error = create_release_evidence_bundle_with_adapter(&request, &mut failed_shell).unwrap_err();
        let stale_stage = failed_shell.observations[0].stage_path.clone();
        let stale_name = stale_stage.file_name().unwrap().to_string_lossy().to_string();
        let unrecognized = stale_stage.with_file_name(format!("{stale_name}-unrecognized"));
        std::fs::create_dir(&unrecognized).unwrap();
        write_file(
            &unrecognized.join(crate::release_publication::RELEASE_STAGE_MARKER_FILENAME),
            TEST_UNRECOGNIZED_STAGE_MARKER_BYTES,
        );

        assert!(error.to_string().contains("cleanup failed"), "{error}");
        assert!(stale_stage.is_dir());
        assert!(!request.bundle_dir.exists());

        let retried = create_release_evidence_bundle(&request).unwrap();
        let names = parent_entry_names(temp.path());

        assert_eq!(retried.release_id, "mantle-stale-stage");
        assert!(!stale_stage.exists(), "recognized stale stage should move to quarantine");
        assert!(unrecognized.is_dir(), "unrecognized stale sibling must stay untouched");
        assert!(names.iter().any(|name| name.starts_with(TEST_QUARANTINE_PREFIX)));
        assert!(verify_release_evidence_bundle(&request.bundle_dir).is_ok());
    }

    // r[verify mantle.release_provenance.bundle_tree_copy.fixtures.positive]
    // r[verify mantle.release_provenance.bundle_tree_copy.validation.production]
    #[test]
    #[cfg(unix)]
    fn create_release_bundle_preserves_internal_symlink_and_tree_digest() {
        use std::os::unix::fs::symlink;

        const INTERNAL_PROOF_BYTES: &[u8] = b"internal-proof";
        const INTERNAL_LINK_TARGET: &str = "nested/proof.txt";

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
        write_file(&proof_bundle_dir.join("nested/proof.txt"), INTERNAL_PROOF_BYTES);
        symlink(INTERNAL_LINK_TARGET, proof_bundle_dir.join("proof-link")).unwrap();
        let expected_digest = hash_directory(&proof_bundle_dir).unwrap();

        let request = ReleaseBundleCreateRequest::with_defaults(
            "mantle-symlink-positive".to_string(),
            output_bundle_dir.clone(),
            source_archive,
            vec![binary_path],
            proof_bundle_dir,
        );
        let created = create_release_evidence_bundle(&request).unwrap();
        let verified = verify_release_evidence_bundle(&output_bundle_dir).unwrap();

        assert_eq!((created.proof_bundle.size_bytes, created.proof_bundle.digest_blake3.clone()), expected_digest);
        assert_eq!(created, verified);
        assert_eq!(
            std::fs::read_link(output_bundle_dir.join("proof/self-hosting/proof-link")).unwrap(),
            Path::new(INTERNAL_LINK_TARGET)
        );
    }

    // r[verify mantle.release_provenance.bundle_tree_copy.fixtures.negative.symlink_escape]
    // r[verify mantle.release_provenance.bundle_tree_copy.validation.production]
    #[test]
    #[cfg(unix)]
    fn create_release_bundle_rejects_directory_symlink_escape_without_external_writes() {
        use std::os::unix::fs::symlink;

        const EXTERNAL_SENTINEL_BYTES: &[u8] = b"external-sentinel";
        const ATTACKER_REPLACEMENT_BYTES: &[u8] = b"attacker-replacement";
        const ATTACKER_CREATED_BYTES: &[u8] = b"attacker-created";
        const ESCAPING_TARGET: &str = "../../../outside";

        let source_temp = tempfile::tempdir().unwrap();
        let destination_temp = tempfile::tempdir().unwrap();
        let source_archive = source_temp.path().join("mantle-src.tar");
        let binary_path = source_temp.path().join("mantle");
        let proof_bundle_dir = source_temp.path().join("a/b/proof-input");
        let source_outside_dir = source_temp.path().join("outside");
        let output_bundle_dir = destination_temp.path().join("release-bundle");
        let external_dir = destination_temp.path().join("outside");
        let external_sentinel_path = external_dir.join("sentinel.txt");
        let external_created_path = external_dir.join("created.txt");

        write_file(&source_archive, b"source-archive");
        write_file(&binary_path, b"mantle-binary");
        let inventory_digest = blake3::hash(b"inventory").to_hex().to_string();
        let stage2_digest = blake3::hash(b"mantle-binary").to_hex().to_string();
        write_full_proof_manifest(&proof_bundle_dir, &inventory_digest, &stage2_digest);
        write_file(&source_outside_dir.join("sentinel.txt"), ATTACKER_REPLACEMENT_BYTES);
        write_file(&source_outside_dir.join("created.txt"), ATTACKER_CREATED_BYTES);
        symlink(ESCAPING_TARGET, proof_bundle_dir.join("escape")).unwrap();
        write_file(&external_sentinel_path, EXTERNAL_SENTINEL_BYTES);

        let request = ReleaseBundleCreateRequest::with_defaults(
            "mantle-security-regression".to_string(),
            output_bundle_dir.clone(),
            source_archive,
            vec![binary_path],
            proof_bundle_dir,
        );
        let result = create_release_evidence_bundle(&request);

        assert_eq!(std::fs::read(&external_sentinel_path).unwrap(), EXTERNAL_SENTINEL_BYTES);
        assert!(!external_created_path.exists(), "tree copy must not create an external path");
        assert!(result.is_err(), "escaping proof-bundle symlink must fail closed");
        assert!(!output_bundle_dir.exists(), "invalid copy plans must block bundle mutation");
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

    // r[verify mantle.release_provenance.cairn_evidence_handoff.production_wiring]
    // r[verify mantle.release_provenance.cairn_evidence_handoff.bypass_protection]
    #[test]
    fn create_and_verify_bundle_with_measured_cairn_handoff() {
        let (temp, output_bundle_dir, created) = create_cairn_handoff_bundle();
        let verified = verify_release_evidence_bundle(&output_bundle_dir).unwrap();
        let receipt = created.cairn_handoff_validation.as_ref().expect("Cairn receipt");

        assert_eq!(created, verified);
        assert_eq!(receipt.validation_status, crunch_release_core::CAIRN_HANDOFF_VALIDATION_STATUS);
        assert_eq!(receipt.authentication_status, crunch_release_core::CAIRN_HANDOFF_AUTHENTICATION_STATUS);
        assert!(temp.path().exists());
    }

    // r[verify mantle.release_provenance.cairn_evidence_handoff.measured_inputs]
    #[test]
    fn verify_rejects_tampered_bundle_local_cairn_bytes() {
        let (_temp, output_bundle_dir, created) = create_cairn_handoff_bundle();
        let receipt = created.cairn_handoff_validation.as_ref().expect("Cairn receipt");
        let artifact_path = output_bundle_dir.join(&receipt.handoff.rows[0].artifact.relative_path);
        write_file(&artifact_path, b"tampered-cairn-receipt");

        let error = verify_release_evidence_bundle(&output_bundle_dir).unwrap_err();
        assert!(error.to_string().contains("declared digest"));
        assert!(!error.to_string().contains("release correctness proven"));
    }

    // r[verify mantle.release_provenance.cairn_evidence_handoff.cross_repo_dependency]
    #[test]
    fn verify_rejects_tampered_cairn_handoff_authentication_dependency() {
        let (_temp, output_bundle_dir, created) = create_cairn_handoff_bundle();
        let receipt = created.cairn_handoff_validation.as_ref().expect("Cairn receipt");
        let authentication_path = output_bundle_dir.join(&receipt.handoff.authentication.archive_receipt.relative_path);
        write_file(&authentication_path, b"tampered-cairn-authentication-receipt");

        let error = verify_release_evidence_bundle(&output_bundle_dir).unwrap_err();
        assert!(error.to_string().contains("declared digest"));
        assert!(!error.to_string().contains("release correctness proven"));
    }

    // r[verify mantle.release_provenance.cairn_evidence_handoff.measured_inputs]
    #[cfg(unix)]
    #[test]
    fn verify_rejects_symlinked_bundle_local_cairn_bytes() {
        use std::os::unix::fs::symlink;

        let (temp, output_bundle_dir, created) = create_cairn_handoff_bundle();
        let receipt = created.cairn_handoff_validation.as_ref().expect("Cairn receipt");
        let artifact_path = output_bundle_dir.join(&receipt.handoff.rows[0].artifact.relative_path);
        let replacement_target = temp.path().join("replacement-cairn-receipt.json");
        let original_bytes = std::fs::read(&artifact_path).unwrap();
        write_file(&replacement_target, &original_bytes);
        std::fs::remove_file(&artifact_path).unwrap();
        symlink(&replacement_target, &artifact_path).unwrap();

        let error = verify_release_evidence_bundle(&output_bundle_dir).unwrap_err();
        assert!(error.to_string().contains("no-follow"));
        assert!(!error.to_string().contains("release correctness proven"));
    }

    // r[verify mantle.release_provenance.cairn_evidence_handoff.measured_inputs]
    #[test]
    fn verify_rejects_stale_bundle_local_cairn_policy_bytes() {
        let (_temp, output_bundle_dir, created) = create_cairn_handoff_bundle();
        let receipt = created.cairn_handoff_validation.as_ref().expect("Cairn receipt");
        let policy_path = output_bundle_dir.join(&receipt.handoff.rows[0].cairn_policy.relative_path);
        write_file(&policy_path, b"stale-cairn-policy");

        let error = verify_release_evidence_bundle(&output_bundle_dir).unwrap_err();
        assert!(error.to_string().contains("declared digest"));
        assert!(!error.to_string().contains("source correctness proven"));
    }

    // r[verify mantle.release_provenance.cairn_evidence_handoff.bypass_protection]
    #[test]
    fn verify_rejects_cairn_receipt_reused_after_manifest_projection_changes() {
        let (_temp, output_bundle_dir, mut created) = create_cairn_handoff_bundle();
        created.workflow.version = "changed-after-handoff-validation".to_string();
        let bytes = serde_json::to_vec(&created).unwrap();
        write_file(&output_bundle_dir.join("manifest.json"), &bytes);

        let error = verify_release_evidence_bundle(&output_bundle_dir).unwrap_err();
        assert!(error.to_string().contains("bound to another release bundle"));
        assert!(!error.to_string().contains("release correctness proven"));
    }

    // r[verify mantle.release_provenance.cairn_evidence_handoff.bypass_protection]
    #[test]
    fn verify_rejects_cairn_receipt_reused_for_another_release_identity() {
        let (_temp, output_bundle_dir, mut created) = create_cairn_handoff_bundle();
        created.release_id = "another-release".to_string();
        created.proof_linkage.release_id = created.release_id.clone();
        let bytes = serde_json::to_vec(&created).unwrap();
        write_file(&output_bundle_dir.join("manifest.json"), &bytes);

        let error = verify_release_evidence_bundle(&output_bundle_dir).unwrap_err();
        assert!(error.to_string().contains("bound to another release bundle"));
        assert!(!error.to_string().contains("release correctness proven"));
    }

    fn create_cairn_handoff_bundle() -> (tempfile::TempDir, PathBuf, ReleaseEvidenceManifest) {
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
            "mantle-cairn-fixture".to_string(),
            output_bundle_dir.clone(),
            source_archive,
            vec![binary_path],
            proof_bundle_dir,
        );
        request.cairn_handoff_descriptor_path =
            Some(Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/cairn-handoff/positive/handoff.json"));
        let created = create_release_evidence_bundle(&request).unwrap();
        (temp, output_bundle_dir, created)
    }
}
