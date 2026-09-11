use std::fmt::Write as _;
use std::path::Path;
use std::path::PathBuf;

use crunch_release_core::SourceObservationBinding;

use crunch_release_core::AST_GREP_EXTERNAL_EVIDENCE_ROLE;
use crunch_release_core::AST_GREP_STRUCTURAL_CLAIM_SCOPE;
use crunch_release_core::AST_GREP_STRUCTURAL_EVIDENCE_SCHEMA;
use crunch_release_core::DeterministicBuildProofReceipt;
use crunch_release_core::DeterministicSandboxIsolationEvidence;
use crunch_release_core::KaniSolverIdentity;
use crunch_release_core::ReleaseVerificationDecision;
use crunch_release_core::ReleaseVerificationFact;
use crunch_release_core::ReleaseVerificationFacts;
use crunch_release_core::ReleaseVerificationRequirement;
use crunch_release_core::ReleaseVerificationRequirements;
use crunch_release_core::aggregate_release_verification;
use crunch_release_core::deterministic_build_proof_has_genuine_rebuild_authority;
use crunch_release_core::deterministic_build_proof_receipt_canonical_bytes;
use crunch_release_core::deterministic_release_claim_eligible;
use crunch_release_core::deterministic_sandbox_isolation_evidence_canonical_bytes;
use crunch_release_core::validate_provider_fixed_point_release_artifact_binding;

use crate::ast_grep_evidence::validate_ast_grep_release_attachment_file;
use crate::content_bound_requirement_evidence::ContentBoundRequirementCreateRequest;
use crate::content_bound_requirement_evidence::load_content_bound_requirement_create_request;
use crate::errors::RunError;
use crate::function_address_binding_cmd::FunctionAddressBindingCommand;
use crate::function_address_binding_cmd::cmd_function_address_binding;
use crate::global_reproducibility_cmd::cmd_global_reproducibility;
use crate::global_reproducibility_release::cmd_global_reproducibility_release_evidence;
use crate::release_attestation::create_release_attestation;
use crate::release_attestation::create_witness_attestation;
use crate::release_attestation::default_verification_dir;
use crate::release_evidence::DEFAULT_PROOF_WORKFLOW_COMMAND;
use crate::release_evidence::DEFAULT_PROOF_WORKFLOW_VERSION;
use crate::release_evidence::ExternalEvidenceCreateRequest;
use crate::release_evidence::GitSourceCreateRequest;
use crate::release_evidence::KaniToolchainEvidenceCreateRequest;
use crate::release_evidence::ReleaseBundleCreateRequest;
use crate::release_evidence::StackProvenanceCreateRequest;
use crate::release_evidence::create_release_evidence_bundle;
use crate::release_evidence::verify_release_evidence_bundle;
use crate::release_nix_witness::ReleaseNixWitnessRequest;
use crate::release_nix_witness::write_release_nix_cross_builder_witness;
use crate::release_reproducibility::DEFAULT_REPRODUCIBILITY_WORKFLOW_VERSION;
use crate::release_reproducibility::ReleaseReproduceRequest;
use crate::release_reproducibility::ReleaseReproduceSummary;
use crate::release_reproducibility::ReproducibilityStatus;
use crate::release_reproducibility::VerifiedReproducibilityReport;
use crate::release_reproducibility::load_bundle_reproducibility_report;
use crate::release_reproducibility::reproduce_release_artifacts;
use crate::release_source::write_tracked_source_archive;
use crate::verification_gauntlet_cmd::cmd_release_gauntlet;
use crate::witness_handoff::create_witness_request_directory;
use crate::witness_handoff::default_witness_request_dir;
use crate::witness_rebuild::WITNESS_SCRATCH_ENV;
use crate::witness_rebuild::WitnessRebuildSuccess;
use crate::witness_rebuild::audit_meta_path;
use crate::witness_rebuild::build_failure_audit_meta;
use crate::witness_rebuild::build_prelaunch_failure_audit_meta;
use crate::witness_rebuild::build_success_audit_meta;
use crate::witness_rebuild::default_witness_scratch_dir;
use crate::witness_rebuild::plan_witness_rebuild;
use crate::witness_rebuild::prepare_witness_rebuild_scratch;
use crate::witness_rebuild::run_witness_rebuild_workflow;
use crate::witness_rebuild::source_acquisition_mode_for_plan;
use crate::witness_rebuild::validate_successful_rebuild;
use crate::witness_rebuild::write_audit_meta;

const PROVIDER_FIXED_POINT_SOURCE_BUNDLED: &str = "bundled";
const PROVIDER_FIXED_POINT_SOURCE_EXTERNAL: &str = "external";
const RELEASE_VERIFY_JSON_KIND: &str = "mantle-release-verify-v2";
const RELEASE_VERIFICATION_REJECTED_EXIT_CODE: u8 = 1;
const GLOBAL_REPRODUCIBILITY_STATUS_NOT_EVALUATED: &str = "not-evaluated";
const GLOBAL_REPRODUCIBILITY_RELEASE_VERIFY_REASON: &str = "release verification is scoped to this bundle; run release global-reproducibility with a universe and policy for a global claim";

pub(crate) fn cmd_release(
    action: crate::ReleaseAction,
    current_dir: &Path,
    state_dir: &Path,
    json: bool,
) -> Result<(), RunError> {
    let context = ReleaseCommandContext {
        current_dir,
        state_dir,
        is_json_output: json,
    };
    debug_assert_eq!(context.current_dir, current_dir);
    debug_assert_eq!(context.state_dir, state_dir);
    match action {
        crate::ReleaseAction::Create { .. } => cmd_release_create(context, release_create_command(action)?),
        crate::ReleaseAction::Verify { .. } => {
            cmd_release_verify(release_verify_request(action, current_dir)?, context.is_json_output)
        }
        crate::ReleaseAction::Transport { action } => cmd_release_transport_action(action, context.is_json_output),
        crate::ReleaseAction::Reproduce { .. } => cmd_release_reproduce(context, release_reproduce_command(action)?),
        crate::ReleaseAction::FunctionAddressBind { .. } => cmd_release_function_address_action(action, context),
        crate::ReleaseAction::GlobalReproducibility { .. } => cmd_global_reproducibility_action(action, context),
        crate::ReleaseAction::GlobalReproducibilityEvidence { .. } => {
            cmd_global_reproducibility_evidence_action(action, context)
        }
        crate::ReleaseAction::Gauntlet { action } => {
            cmd_release_gauntlet(action, context.current_dir, context.is_json_output)
        }
        crate::ReleaseAction::NixWitness { .. } => cmd_release_nix_witness_action(action, context),
        crate::ReleaseAction::Attest { .. } => cmd_release_attest_action(action, context),
        crate::ReleaseAction::WitnessExport { .. } => cmd_release_witness_export_action(action, context),
        crate::ReleaseAction::WitnessRebuild { .. } => {
            cmd_release_witness_rebuild(context, release_witness_rebuild_command(action)?)
        }
    }
}

#[derive(Clone, Copy)]
struct ReleaseCommandContext<'a> {
    current_dir: &'a Path,
    state_dir: &'a Path,
    is_json_output: bool,
}

fn cmd_release_transport_action(action: crate::ReleaseTransportAction, is_json_output: bool) -> Result<(), RunError> {
    match action {
        crate::ReleaseTransportAction::Pack { bundle_dir, to } => {
            let outcome = crate::release_chapter_transport::pack_release_transport(&bundle_dir, &to)?;
            render_release_transport_output(&outcome, is_json_output, "packed")
        }
        crate::ReleaseTransportAction::Inspect { transport_dir } => {
            let inspection = crate::release_chapter_transport::inspect_release_transport(&transport_dir)?;
            render_release_transport_output(&inspection, is_json_output, "valid")
        }
        crate::ReleaseTransportAction::Unpack { transport_dir, to } => {
            let outcome = crate::release_chapter_transport::unpack_release_transport(&transport_dir, &to)?;
            render_release_transport_output(&outcome, is_json_output, "unpacked")
        }
    }
}

fn render_release_transport_output<T: serde::Serialize>(
    value: &T,
    is_json_output: bool,
    status: &str,
) -> Result<(), RunError> {
    if is_json_output {
        let rendered = serde_json::to_string(value)
            .map_err(|error| RunError::Internal(format!("serializing release transport output: {error}")))?;
        println!("{rendered}");
    } else {
        println!("release chapter transport: {status}");
    }
    debug_assert!(!status.is_empty());
    debug_assert!(is_json_output || !status.is_empty());
    Ok(())
}

fn cmd_release_function_address_action(
    action: crate::ReleaseAction,
    context: ReleaseCommandContext<'_>,
) -> Result<(), RunError> {
    let crate::ReleaseAction::FunctionAddressBind {
        bundle_dir,
        mode,
        from_preserves_binding,
        sidecar,
        valence_receipt,
        kamacite_receipt,
        release_binary,
        receipt_out,
    } = action
    else {
        return Err(RunError::Internal("expected function-address release action".to_string()));
    };
    debug_assert!(context.current_dir.components().next().is_some());
    debug_assert!(context.state_dir.components().next().is_some());
    cmd_function_address_binding(context.current_dir, context.is_json_output, FunctionAddressBindingCommand {
        bundle_dir,
        mode,
        from_preserves_binding,
        sidecar_relative_path: sidecar,
        valence_receipt_relative_path: valence_receipt,
        kamacite_receipt_relative_path: kamacite_receipt,
        release_binary_relative_path: release_binary,
        receipt_out,
    })
}

fn cmd_global_reproducibility_action(
    action: crate::ReleaseAction,
    context: ReleaseCommandContext<'_>,
) -> Result<(), RunError> {
    let crate::ReleaseAction::GlobalReproducibility {
        universe,
        policy,
        evidence,
        report_path,
    } = action
    else {
        return Err(RunError::Internal("expected global reproducibility action".to_string()));
    };
    cmd_global_reproducibility(context.current_dir, context.is_json_output, universe, policy, evidence, report_path)
}

fn cmd_global_reproducibility_evidence_action(
    action: crate::ReleaseAction,
    context: ReleaseCommandContext<'_>,
) -> Result<(), RunError> {
    let crate::ReleaseAction::GlobalReproducibilityEvidence {
        universe,
        policy,
        bundle_dir,
        verification_dir,
        release_verify_json,
        evidence_path,
    } = action
    else {
        return Err(RunError::Internal("expected global reproducibility evidence action".to_string()));
    };
    debug_assert!(context.current_dir.components().next().is_some());
    debug_assert!(context.state_dir.components().next().is_some());
    cmd_global_reproducibility_release_evidence(
        context.current_dir,
        context.is_json_output,
        universe,
        policy,
        bundle_dir,
        verification_dir,
        release_verify_json,
        evidence_path,
    )
}

fn cmd_release_nix_witness_action(
    action: crate::ReleaseAction,
    context: ReleaseCommandContext<'_>,
) -> Result<(), RunError> {
    cmd_release_nix_witness(context, release_nix_witness_command(action)?)
}

fn cmd_release_attest_action(action: crate::ReleaseAction, context: ReleaseCommandContext<'_>) -> Result<(), RunError> {
    let crate::ReleaseAction::Attest {
        bundle_dir,
        verification_dir,
        signing_key,
    } = action
    else {
        return Err(RunError::Internal("expected release attest action".to_string()));
    };
    cmd_release_attest(context, bundle_dir, verification_dir, signing_key)
}

fn cmd_release_witness_export_action(
    action: crate::ReleaseAction,
    context: ReleaseCommandContext<'_>,
) -> Result<(), RunError> {
    let crate::ReleaseAction::WitnessExport {
        bundle_dir,
        verification_dir,
        request_dir,
    } = action
    else {
        return Err(RunError::Internal("expected witness export action".to_string()));
    };
    debug_assert!(context.current_dir.components().next().is_some());
    debug_assert!(context.state_dir.components().next().is_some());
    cmd_release_witness_export(context.current_dir, context.is_json_output, bundle_dir, verification_dir, request_dir)
}

fn release_create_command(action: crate::ReleaseAction) -> Result<ReleaseCreateCommand, RunError> {
    let crate::ReleaseAction::Create {
        release_id,
        bundle_dir,
        binary,
        proof_bundle,
        provider_fixed_point_proof,
        reproducibility_report,
        source_acquisition_url,
        source_observation,
        git_source_url,
        git_source_commit,
        git_source_ref,
        git_source_tag,
        cairn_handoff,
        external_evidence,
        external_evidence_role,
        external_evidence_schema,
        external_evidence_claim_scope,
        external_evidence_non_claim,
        kani_toolchain_evidence,
        stack_provenance_sidecar,
        stack_provenance_valence_receipt,
        stack_provenance_binary,
        requirement_evidence_root,
        requirement_evidence_input,
        workflow_command,
        workflow_version,
    } = action
    else {
        return Err(RunError::Internal("expected release create action".to_string()));
    };
    debug_assert!(binary.capacity() >= binary.len());
    debug_assert!(external_evidence.capacity() >= external_evidence.len());
    Ok(ReleaseCreateCommand {
        release_id,
        bundle_dir,
        binary,
        proof_bundle,
        provider_fixed_point_proof,
        reproducibility_report,
        source_acquisition_url,
        source_observation,
        git_source_url,
        git_source_commit,
        git_source_ref,
        git_source_tag,
        cairn_handoff,
        external_evidence,
        external_evidence_role,
        external_evidence_schema,
        external_evidence_claim_scope,
        external_evidence_non_claim,
        kani_toolchain_evidence,
        stack_provenance_sidecar,
        stack_provenance_valence_receipt,
        stack_provenance_binary,
        requirement_evidence_root,
        requirement_evidence_input,
        workflow_command,
        workflow_version,
    })
}

struct ReleaseCreateCommand {
    release_id: String,
    bundle_dir: Option<PathBuf>,
    binary: Vec<PathBuf>,
    proof_bundle: PathBuf,
    provider_fixed_point_proof: Option<PathBuf>,
    reproducibility_report: Option<PathBuf>,
    source_acquisition_url: Option<String>,
    source_observation: Option<PathBuf>,
    git_source_url: Option<String>,
    git_source_commit: Option<String>,
    git_source_ref: Option<String>,
    git_source_tag: Option<String>,
    cairn_handoff: Option<PathBuf>,
    external_evidence: Vec<PathBuf>,
    external_evidence_role: Vec<String>,
    external_evidence_schema: Vec<String>,
    external_evidence_claim_scope: Vec<String>,
    external_evidence_non_claim: Vec<String>,
    kani_toolchain_evidence: Vec<PathBuf>,
    stack_provenance_sidecar: Option<PathBuf>,
    stack_provenance_valence_receipt: Option<PathBuf>,
    stack_provenance_binary: Option<PathBuf>,
    requirement_evidence_root: Option<PathBuf>,
    requirement_evidence_input: Option<String>,
    workflow_command: String,
    workflow_version: String,
}

fn cmd_release_create(context: ReleaseCommandContext<'_>, command: ReleaseCreateCommand) -> Result<(), RunError> {
    let prepared = prepare_release_create(context.current_dir, command)?;
    let manifest = create_release_evidence_bundle(&prepared.request)?;
    emit_release_create_result(&prepared.request, &manifest, context.is_json_output)
}

struct PreparedReleaseCreate {
    request: ReleaseBundleCreateRequest,
    _source_archive_file: tempfile::NamedTempFile,
}

fn prepare_release_create(
    current_dir: &Path,
    command: ReleaseCreateCommand,
) -> Result<PreparedReleaseCreate, RunError> {
    debug_assert!(command.binary.capacity() >= command.binary.len());
    debug_assert!(command.workflow_command.capacity() >= command.workflow_command.len());
    let resolved_bundle_dir = resolve_bundle_dir(current_dir, &command.release_id, command.bundle_dir);
    let workflow_command = normalize_workflow_value(&command.workflow_command, DEFAULT_PROOF_WORKFLOW_COMMAND);
    let workflow_version = normalize_workflow_value(&command.workflow_version, DEFAULT_PROOF_WORKFLOW_VERSION);
    let git_source = release_create_git_source(
        command.git_source_url,
        command.git_source_commit,
        command.git_source_ref,
        command.git_source_tag,
    )?;
    let external_evidence = release_create_external_evidence(ExternalEvidenceCommand {
        current_dir,
        paths: command.external_evidence,
        roles: command.external_evidence_role,
        schemas: command.external_evidence_schema,
        claim_scopes: command.external_evidence_claim_scope,
        non_claims: command.external_evidence_non_claim,
    })?;
    let kani_toolchain_evidence = release_create_kani_toolchain_evidence(current_dir, command.kani_toolchain_evidence)?;
    let stack_provenance = release_create_stack_provenance(
        current_dir,
        command.stack_provenance_sidecar,
        command.stack_provenance_valence_receipt,
        command.stack_provenance_binary,
    )?;
    let content_bound_requirement = release_create_content_bound_requirement(
        current_dir,
        command.requirement_evidence_root,
        command.requirement_evidence_input,
    )?;
    if command.source_acquisition_url.is_some() && git_source.is_some() {
        return Err(RunError::Internal(
            "release create Git source flags conflict with --source-acquisition-url".to_string(),
        ));
    }
    let source_archive_file = tempfile::NamedTempFile::new()
        .map_err(|err| RunError::Internal(format!("creating temp source archive file: {err}")))?;
    write_tracked_source_archive(current_dir, source_archive_file.path())?;
    let request = ReleaseBundleCreateRequest {
        release_id: command.release_id,
        bundle_dir: resolved_bundle_dir,
        source_archive_path: source_archive_file.path().to_path_buf(),
        binary_paths: resolve_input_paths(current_dir, command.binary),
        proof_bundle_dir: resolve_input_path(current_dir, command.proof_bundle),
        workflow_command,
        workflow_version,
        reproducibility_report_path: command.reproducibility_report.map(|path| resolve_input_path(current_dir, path)),
        provider_fixed_point_proof_dir: command
            .provider_fixed_point_proof
            .map(|path| resolve_input_path(current_dir, path)),
        source_acquisition_url: command.source_acquisition_url,
        source_observation: read_source_observation_binding(current_dir, command.source_observation)?,
        git_source,
        external_evidence,
        kani_toolchain_evidence,
        stack_provenance,
        content_bound_requirement,
        cairn_handoff_descriptor_path: command.cairn_handoff.map(|path| resolve_input_path(current_dir, path)),
    };
    Ok(PreparedReleaseCreate {
        request,
        _source_archive_file: source_archive_file,
    })
}

/// Read and parse the optional source-observation binding file.
fn read_source_observation_binding(
    current_dir: &Path,
    path: Option<PathBuf>,
) -> Result<Option<SourceObservationBinding>, RunError> {
    let Some(path) = path else {
        return Ok(None);
    };
    let resolved = resolve_input_path(current_dir, path);
    let text = std::fs::read_to_string(&resolved).map_err(|err| {
        RunError::Internal(format!(
            "reading source observation binding {}: {err}",
            resolved.display()
        ))
    })?;
    let binding: SourceObservationBinding = serde_json::from_str(&text).map_err(|err| {
        RunError::Internal(format!(
            "parsing source observation binding {}: {err}",
            resolved.display()
        ))
    })?;
    debug_assert!(!binding.schema.is_empty());
    debug_assert!(!binding.observation_blake3.is_empty());
    Ok(Some(binding))
}

fn emit_release_create_result(
    request: &ReleaseBundleCreateRequest,
    manifest: &crate::release_evidence::ReleaseEvidenceManifest,
    is_json_output: bool,
) -> Result<(), RunError> {
    debug_assert_eq!(request.release_id, manifest.release_id);
    debug_assert!(!manifest.binaries.is_empty());
    if is_json_output {
        let rendered = serde_json::to_string(manifest)
            .map_err(|err| RunError::Internal(format!("serializing release evidence manifest: {err}")))?;
        println!("{rendered}");
        return Ok(());
    }
    println!("release evidence bundle: {}", request.bundle_dir.display());
    println!("release id: {}", manifest.release_id);
    println!("source archive: {}", manifest.source_archive.relative_path);
    println!("proof bundle: {}", manifest.proof_bundle.relative_path);
    println!("binaries: {}", manifest.binaries.len());
    emit_optional_release_create_result(manifest);
    println!("manifest: manifest.json");
    Ok(())
}

fn emit_optional_release_create_result(manifest: &crate::release_evidence::ReleaseEvidenceManifest) {
    debug_assert!(!manifest.release_id.is_empty());
    debug_assert!(manifest.external_evidence.capacity() >= manifest.external_evidence.len());
    if let Some(proof) = &manifest.provider_fixed_point_proof {
        println!("provider fixed-point proof: {}", proof.relative_path);
    }
    if let Some(report) = &manifest.reproducibility_report {
        println!("reproducibility report: {}", report.relative_path);
    }
    if !manifest.external_evidence.is_empty() {
        println!("external evidence: {}", manifest.external_evidence.len());
    }
    if !manifest.kani_toolchain_evidence.is_empty() {
        println!("Kani toolchain evidence: {}", manifest.kani_toolchain_evidence.len());
    }
    if let Some(receipt) = &manifest.cairn_handoff_validation {
        println!("Cairn handoff validation: {}", receipt.validation_status);
        println!("Cairn handoff authentication: {}", receipt.authentication_status);
    }
    if let Some(source_acquisition) = &manifest.source_acquisition {
        println!("source acquisition: {} ({})", source_acquisition.url, source_acquisition.kind);
        if let Some(commit) = &source_acquisition.commit {
            println!("source acquisition commit: {commit}");
        }
    }
}

fn release_create_content_bound_requirement(
    current_dir: &Path,
    root: Option<PathBuf>,
    input: Option<String>,
) -> Result<Option<ContentBoundRequirementCreateRequest>, RunError> {
    match (root, input) {
        (None, None) => Ok(None),
        (Some(root), Some(input)) => {
            load_content_bound_requirement_create_request(&resolve_input_path(current_dir, root), &input).map(Some)
        }
        _ => Err(RunError::Internal(
            "release create content-bound requirement evidence requires both root and input".to_string(),
        )),
    }
}

fn release_create_git_source(
    git_source_url: Option<String>,
    git_source_commit: Option<String>,
    git_source_ref: Option<String>,
    git_source_tag: Option<String>,
) -> Result<Option<GitSourceCreateRequest>, RunError> {
    let Some(remote_url) = git_source_url else {
        if git_source_commit.is_some() || git_source_ref.is_some() || git_source_tag.is_some() {
            return Err(RunError::Internal(
                "release create Git source commit/ref/tag flags require --git-source-url".to_string(),
            ));
        }
        return Ok(None);
    };
    let Some(commit) = git_source_commit else {
        return Err(RunError::Internal("release create --git-source-url requires --git-source-commit".to_string()));
    };
    Ok(Some(GitSourceCreateRequest {
        remote_url,
        commit,
        reference: git_source_ref,
        tag: git_source_tag,
    }))
}

struct ExternalEvidenceCommand<'a> {
    current_dir: &'a Path,
    paths: Vec<PathBuf>,
    roles: Vec<String>,
    schemas: Vec<String>,
    claim_scopes: Vec<String>,
    non_claims: Vec<String>,
}

fn release_create_external_evidence(
    command: ExternalEvidenceCommand<'_>,
) -> Result<Vec<ExternalEvidenceCreateRequest>, RunError> {
    let ExternalEvidenceCommand {
        current_dir,
        paths,
        roles,
        schemas,
        claim_scopes,
        non_claims,
    } = command;
    if paths.is_empty() {
        if !(roles.is_empty() && schemas.is_empty() && claim_scopes.is_empty() && non_claims.is_empty()) {
            return Err(RunError::Internal(
                "release create external evidence metadata requires --external-evidence".to_string(),
            ));
        }
        return Ok(vec![]);
    }
    debug_assert!(!paths.is_empty());
    debug_assert!(current_dir.components().next().is_some());
    validate_external_evidence_metadata_count(EvidenceMetadataCount {
        expected_count: paths.len(),
        actual_count: roles.len(),
        flag_name: "--external-evidence-role",
    })?;
    validate_external_evidence_metadata_count(EvidenceMetadataCount {
        expected_count: paths.len(),
        actual_count: schemas.len(),
        flag_name: "--external-evidence-schema",
    })?;
    validate_external_evidence_metadata_count(EvidenceMetadataCount {
        expected_count: paths.len(),
        actual_count: claim_scopes.len(),
        flag_name: "--external-evidence-claim-scope",
    })?;
    if non_claims.is_empty() {
        return Err(RunError::Internal(
            "release create --external-evidence-non-claim is required when external evidence is bundled".to_string(),
        ));
    }

    let requests = paths
        .into_iter()
        .zip(roles)
        .zip(schemas)
        .zip(claim_scopes)
        .map(|(((path, role), schema), claim_scope)| ExternalEvidenceCreateRequest {
            path: resolve_input_path(current_dir, path),
            role,
            schema,
            claim_scope,
            non_claims: non_claims.clone(),
        })
        .collect::<Vec<_>>();
    for request in &requests {
        validate_ast_grep_external_evidence_request(request)?;
    }
    Ok(requests)
}

fn validate_ast_grep_external_evidence_request(request: &ExternalEvidenceCreateRequest) -> Result<(), RunError> {
    if !ast_grep_attachment_marker_present(request) {
        return Ok(());
    }
    validate_ast_grep_release_attachment_file(
        &request.path,
        request.role.clone(),
        request.schema.clone(),
        request.claim_scope.clone(),
        request.non_claims.clone(),
    )
    .map_err(RunError::Internal)
}

fn ast_grep_attachment_marker_present(request: &ExternalEvidenceCreateRequest) -> bool {
    if request.role == AST_GREP_EXTERNAL_EVIDENCE_ROLE {
        return true;
    }
    if request.schema == AST_GREP_STRUCTURAL_EVIDENCE_SCHEMA {
        return true;
    }
    request.claim_scope == AST_GREP_STRUCTURAL_CLAIM_SCOPE
}

#[derive(Debug, serde::Deserialize)]
struct KaniToolchainEvidenceFile {
    receipt_role: String,
    kani_version: String,
    rust_toolchain: String,
    cbmc_version: String,
    solver: KaniSolverIdentity,
    invocation_wrapper: String,
    closure_identity_blake3: String,
    expected_closure_identity_blake3: String,
    non_claims: Vec<String>,
}

fn release_create_stack_provenance(
    current_dir: &Path,
    sidecar_path: Option<PathBuf>,
    valence_receipt_path: Option<PathBuf>,
    binary_path: Option<PathBuf>,
) -> Result<Option<StackProvenanceCreateRequest>, RunError> {
    match (sidecar_path, valence_receipt_path) {
        (None, None) => Ok(None),
        (Some(sidecar_path), Some(valence_receipt_path)) => Ok(Some(StackProvenanceCreateRequest {
            sidecar_path: resolve_input_path(current_dir, sidecar_path),
            valence_receipt_path: resolve_input_path(current_dir, valence_receipt_path),
            binary_path: binary_path.map(|path| resolve_input_path(current_dir, path)),
        })),
        (Some(_), None) => Err(RunError::Internal(
            "release create --stack-provenance-sidecar requires --stack-provenance-valence-receipt".to_string(),
        )),
        (None, Some(_)) => Err(RunError::Internal(
            "release create --stack-provenance-valence-receipt requires --stack-provenance-sidecar".to_string(),
        )),
    }
}

fn release_create_kani_toolchain_evidence(
    current_dir: &Path,
    paths: Vec<PathBuf>,
) -> Result<Vec<KaniToolchainEvidenceCreateRequest>, RunError> {
    let evidence_path_count = paths.len();
    let mut records = Vec::with_capacity(evidence_path_count);
    for path in paths {
        let resolved = resolve_input_path(current_dir, path);
        let bytes = std::fs::read(&resolved)
            .map_err(|error| RunError::Internal(format!("reading {}: {error}", resolved.display())))?;
        let file: KaniToolchainEvidenceFile = serde_json::from_slice(&bytes)
            .map_err(|error| RunError::Internal(format!("parsing {}: {error}", resolved.display())))?;
        records.push(KaniToolchainEvidenceCreateRequest {
            receipt_role: file.receipt_role,
            kani_version: file.kani_version,
            rust_toolchain: file.rust_toolchain,
            cbmc_version: file.cbmc_version,
            solver: file.solver,
            invocation_wrapper: file.invocation_wrapper,
            closure_identity_blake3: file.closure_identity_blake3,
            expected_closure_identity_blake3: file.expected_closure_identity_blake3,
            non_claims: file.non_claims,
        });
    }
    debug_assert_eq!(records.len(), evidence_path_count);
    debug_assert!(records.capacity() >= records.len());
    Ok(records)
}

struct EvidenceMetadataCount<'a> {
    expected_count: usize,
    actual_count: usize,
    flag_name: &'a str,
}

fn validate_external_evidence_metadata_count(counts: EvidenceMetadataCount<'_>) -> Result<(), RunError> {
    if counts.actual_count != counts.expected_count {
        return Err(RunError::Internal(format!(
            "release create {} count {} must match --external-evidence count {}",
            counts.flag_name, counts.actual_count, counts.expected_count
        )));
    }
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ReleaseVerifyRequest {
    current_dir: PathBuf,
    bundle_dir: PathBuf,
    require_reproducible: bool,
    require_stagex_no_quorum: bool,
    deterministic_proof: Option<PathBuf>,
    deterministic_sandbox_isolation_evidence: Option<PathBuf>,
    require_deterministic_release: bool,
    provider_fixed_point_proof: Option<PathBuf>,
    require_external_evidence_role: Vec<String>,
    require_provider_fixed_point_proof: bool,
    release_profile: String,
    stack_provenance_mode: String,
    requirement_coverage_mode: String,
}

#[derive(Debug)]
struct ReleaseVerifyEvaluation {
    resolved_bundle_dir: PathBuf,
    manifest: crate::release_evidence::ReleaseEvidenceManifest,
    reproducibility: Option<VerifiedReproducibilityReport>,
    reproducibility_status: ReproducibilityStatus,
    deterministic_result: DeterministicReleaseVerifyResult,
    provider_fixed_point_result: crate::cargo_free_self_build::ProviderFixedPointProofVerification,
    release_profile: String,
    stack_provenance_result: crunch_release_core::StackProvenanceReleaseVerification,
    content_bound_requirement_result: crunch_release_core::ContentBoundRequirementVerificationV1,
    cairn_handoff_result: crunch_release_core::CairnHandoffReleaseVerification,
    stagex_result: Option<crunch_bootstrap_core::StagexNoQuorumResult>,
    decision: ReleaseVerificationDecision,
}

fn release_verify_request(action: crate::ReleaseAction, current_dir: &Path) -> Result<ReleaseVerifyRequest, RunError> {
    let crate::ReleaseAction::Verify {
        bundle_dir,
        require_reproducible,
        require_stagex_no_quorum,
        deterministic_proof,
        deterministic_sandbox_isolation_evidence,
        require_deterministic_release,
        provider_fixed_point_proof,
        require_external_evidence_role,
        require_provider_fixed_point_proof,
        release_profile,
        stack_provenance,
        requirement_coverage,
    } = action
    else {
        return Err(RunError::Internal("expected release verify action".to_string()));
    };
    debug_assert!(require_external_evidence_role.capacity() >= require_external_evidence_role.len());
    debug_assert!(release_profile.capacity() >= release_profile.len());
    Ok(ReleaseVerifyRequest {
        current_dir: current_dir.to_path_buf(),
        bundle_dir,
        require_reproducible,
        require_stagex_no_quorum,
        deterministic_proof,
        deterministic_sandbox_isolation_evidence,
        require_deterministic_release,
        provider_fixed_point_proof,
        require_external_evidence_role,
        require_provider_fixed_point_proof,
        release_profile,
        stack_provenance_mode: stack_provenance,
        requirement_coverage_mode: requirement_coverage,
    })
}

fn cmd_release_verify(request: ReleaseVerifyRequest, is_json_output: bool) -> Result<(), RunError> {
    let evaluation = evaluate_release_verification(request)?;
    emit_release_verification(&evaluation, is_json_output)
}

// r[impl mantle.release_provenance.verification_decision.complete]
fn evaluate_release_verification(request: ReleaseVerifyRequest) -> Result<ReleaseVerifyEvaluation, RunError> {
    let resolved_bundle_dir = resolve_input_path(&request.current_dir, request.bundle_dir.clone());
    let is_onix_release = request.release_profile == crunch_release_core::RELEASE_PROFILE_ONIX_STACK;
    let is_deterministic_required = request.require_deterministic_release || is_onix_release;
    let deterministic_request = DeterministicVerifyRequest::new(
        &request.current_dir,
        request.deterministic_proof.clone(),
        request.deterministic_sandbox_isolation_evidence.clone(),
        is_deterministic_required,
    );
    let manifest = verify_release_evidence_bundle(&resolved_bundle_dir)?;
    debug_assert!(!manifest.release_id.is_empty());
    debug_assert!(resolved_bundle_dir.components().next().is_some());
    let provider_fixed_point_result = evaluate_provider_fixed_point_proof(
        &request.current_dir,
        &resolved_bundle_dir,
        &manifest,
        request.provider_fixed_point_proof.clone(),
        request.require_provider_fixed_point_proof,
    );
    let reproducibility = load_bundle_reproducibility_report(&resolved_bundle_dir, &manifest)?;
    let reproducibility_status = reproducibility_status(reproducibility.as_ref());
    let deterministic_result =
        evaluate_deterministic_release_claim(&manifest, &resolved_bundle_dir, deterministic_request)?;
    let effective_stack_mode = crunch_release_core::stack_provenance_mode_for_release_profile(
        &request.release_profile,
        &request.stack_provenance_mode,
    )
    .map_err(|err| RunError::Internal(err.to_string()))?;
    let stack_provenance_result =
        crunch_release_core::evaluate_stack_provenance_release_evidence(&manifest, effective_stack_mode);
    let content_bound_requirement_result = crunch_release_core::evaluate_content_bound_requirement_release_evidence(
        &manifest,
        &request.requirement_coverage_mode,
    );
    let cairn_binding = crunch_release_core::release_evidence_cairn_bundle_binding(&manifest)
        .map_err(|error| RunError::Internal(error.to_string()))?;
    let cairn_handoff_result = crunch_release_core::evaluate_cairn_handoff_release_evidence(
        manifest.cairn_handoff_validation.as_ref(),
        &cairn_binding,
        is_onix_release,
    );
    let stagex_result = selected_stagex_result(
        request.require_stagex_no_quorum,
        &manifest,
        &resolved_bundle_dir,
        reproducibility.as_ref(),
    );
    let decision = aggregate_release_verify_decision(ReleaseVerifyDecisionInput {
        request: &request,
        manifest: &manifest,
        reproducibility_status,
        deterministic_result: &deterministic_result,
        provider_fixed_point_result: &provider_fixed_point_result,
        effective_stack_mode,
        stack_provenance_result: &stack_provenance_result,
        content_bound_requirement_result: &content_bound_requirement_result,
        cairn_handoff_result: &cairn_handoff_result,
        stagex_result: stagex_result.as_ref(),
    });

    Ok(ReleaseVerifyEvaluation {
        resolved_bundle_dir,
        manifest,
        reproducibility,
        reproducibility_status,
        deterministic_result,
        provider_fixed_point_result,
        release_profile: request.release_profile,
        stack_provenance_result,
        content_bound_requirement_result,
        cairn_handoff_result,
        stagex_result,
        decision,
    })
}

fn selected_stagex_result(
    required: bool,
    manifest: &crate::release_evidence::ReleaseEvidenceManifest,
    bundle_dir: &Path,
    reproducibility: Option<&VerifiedReproducibilityReport>,
) -> Option<crunch_bootstrap_core::StagexNoQuorumResult> {
    required.then(|| evaluate_stagex_profile(manifest, bundle_dir, reproducibility))
}

struct ReleaseVerifyDecisionInput<'a> {
    request: &'a ReleaseVerifyRequest,
    manifest: &'a crate::release_evidence::ReleaseEvidenceManifest,
    reproducibility_status: ReproducibilityStatus,
    deterministic_result: &'a DeterministicReleaseVerifyResult,
    provider_fixed_point_result: &'a crate::cargo_free_self_build::ProviderFixedPointProofVerification,
    effective_stack_mode: &'a str,
    stack_provenance_result: &'a crunch_release_core::StackProvenanceReleaseVerification,
    content_bound_requirement_result: &'a crunch_release_core::ContentBoundRequirementVerificationV1,
    cairn_handoff_result: &'a crunch_release_core::CairnHandoffReleaseVerification,
    stagex_result: Option<&'a crunch_bootstrap_core::StagexNoQuorumResult>,
}

fn aggregate_release_verify_decision(input: ReleaseVerifyDecisionInput<'_>) -> ReleaseVerificationDecision {
    debug_assert!(!input.manifest.release_id.is_empty());
    debug_assert!(!input.effective_stack_mode.is_empty());
    let facts = ReleaseVerificationFacts {
        manifest_integrity: ReleaseVerificationFact::satisfied(),
        reproducibility: reproducibility_fact(input.reproducibility_status, input.request.require_reproducible),
        deterministic_release: deterministic_release_fact(
            input.deterministic_result,
            input.request.require_deterministic_release
                || input.request.release_profile == crunch_release_core::RELEASE_PROFILE_ONIX_STACK,
        ),
        provider_fixed_point_proof: provider_fixed_point_fact(
            input.provider_fixed_point_result,
            input.request.require_provider_fixed_point_proof,
        ),
        stack_provenance: stack_provenance_fact(input.stack_provenance_result),
        content_bound_requirements: content_bound_requirement_fact(input.content_bound_requirement_result),
        external_evidence_roles: external_evidence_roles_fact(
            input.manifest,
            &input.request.require_external_evidence_role,
        ),
        stagex_no_quorum: stagex_fact(input.stagex_result),
        function_address: ReleaseVerificationFact::not_evaluated(),
        cairn_handoff: cairn_handoff_fact(input.cairn_handoff_result),
    };
    let requirements = release_verification_requirements(&input);
    aggregate_release_verification(facts, requirements)
}

fn release_verification_requirements(input: &ReleaseVerifyDecisionInput<'_>) -> ReleaseVerificationRequirements {
    ReleaseVerificationRequirements {
        manifest_integrity: ReleaseVerificationRequirement::Mandatory,
        reproducibility: selected_or_advisory(input.request.require_reproducible),
        deterministic_release: selected_or_advisory(
            input.request.require_deterministic_release
                || input.request.release_profile == crunch_release_core::RELEASE_PROFILE_ONIX_STACK,
        ),
        provider_fixed_point_proof: selected_or_advisory(input.request.require_provider_fixed_point_proof),
        stack_provenance: selected_or_advisory(
            input.effective_stack_mode == crunch_release_core::STACK_PROVENANCE_MODE_REQUIRED
                || !input.stack_provenance_result.valid,
        ),
        content_bound_requirements: selected_or_advisory(
            input.request.requirement_coverage_mode == crunch_release_core::CONTENT_BOUND_REQUIREMENT_MODE_REQUIRED
                || !input.content_bound_requirement_result.valid,
        ),
        external_evidence_roles: selected_or_not_selected(!input.request.require_external_evidence_role.is_empty()),
        stagex_no_quorum: selected_or_not_selected(input.request.require_stagex_no_quorum),
        function_address: ReleaseVerificationRequirement::NotSelected,
        cairn_handoff: selected_or_advisory(
            input.request.release_profile == crunch_release_core::RELEASE_PROFILE_ONIX_STACK,
        ),
    }
}

fn selected_or_advisory(selected: bool) -> ReleaseVerificationRequirement {
    if selected {
        ReleaseVerificationRequirement::Required
    } else {
        ReleaseVerificationRequirement::Advisory
    }
}

fn selected_or_not_selected(selected: bool) -> ReleaseVerificationRequirement {
    if selected {
        ReleaseVerificationRequirement::Required
    } else {
        ReleaseVerificationRequirement::NotSelected
    }
}

fn reproducibility_fact(status: ReproducibilityStatus, required: bool) -> ReleaseVerificationFact {
    match status {
        ReproducibilityStatus::Matched => ReleaseVerificationFact::satisfied(),
        ReproducibilityStatus::Absent => ReleaseVerificationFact::absent(
            required
                .then(|| format!("reproducibility evidence required but status is {}", status.as_str()))
                .into_iter()
                .collect(),
        ),
        ReproducibilityStatus::Mismatched => ReleaseVerificationFact::rejected(required_policy_diagnostics(
            required.then(|| format!("reproducibility evidence required but status is {}", status.as_str())),
            &[],
        )),
    }
}

fn deterministic_release_fact(result: &DeterministicReleaseVerifyResult, required: bool) -> ReleaseVerificationFact {
    if result.eligible {
        return ReleaseVerificationFact::satisfied();
    }
    let diagnostics = required_policy_diagnostics(
        required.then(|| format!("deterministic release evidence required but status is {}", result.status)),
        &result.blockers,
    );
    if result.status == "absent" {
        return ReleaseVerificationFact::absent(diagnostics);
    }
    ReleaseVerificationFact::rejected(diagnostics)
}

fn provider_fixed_point_fact(
    result: &crate::cargo_free_self_build::ProviderFixedPointProofVerification,
    required: bool,
) -> ReleaseVerificationFact {
    if result.valid {
        return ReleaseVerificationFact::satisfied();
    }
    let diagnostics = required_policy_diagnostics(
        required.then(|| format!("provider fixed-point proof required but status is {}", result.status)),
        &result.blockers,
    );
    if result.status == "absent" {
        return ReleaseVerificationFact::absent(diagnostics);
    }
    ReleaseVerificationFact::rejected(diagnostics)
}

fn stack_provenance_fact(result: &crunch_release_core::StackProvenanceReleaseVerification) -> ReleaseVerificationFact {
    if result.valid && result.disposition == crunch_release_core::STACK_PROVENANCE_DISPOSITION_PRESENT {
        return ReleaseVerificationFact::satisfied();
    }
    if result.valid && result.disposition == crunch_release_core::STACK_PROVENANCE_DISPOSITION_ABSENT {
        return ReleaseVerificationFact::absent(Vec::new());
    }
    ReleaseVerificationFact::rejected(required_policy_diagnostics(
        Some(format!("stack provenance evidence status is {}", result.disposition)),
        &result.diagnostics,
    ))
}

fn content_bound_requirement_fact(
    result: &crunch_release_core::ContentBoundRequirementVerificationV1,
) -> ReleaseVerificationFact {
    if result.valid && result.disposition == crunch_release_core::CONTENT_BOUND_REQUIREMENT_DISPOSITION_PRESENT {
        return ReleaseVerificationFact::satisfied();
    }
    if result.valid && result.disposition == crunch_release_core::CONTENT_BOUND_REQUIREMENT_DISPOSITION_ABSENT {
        return ReleaseVerificationFact::absent(Vec::new());
    }
    let diagnostics = result.issues.iter().map(|issue| format!("{:?} at {}", issue.code, issue.field_path)).collect();
    ReleaseVerificationFact::rejected(diagnostics)
}

fn cairn_handoff_fact(result: &crunch_release_core::CairnHandoffReleaseVerification) -> ReleaseVerificationFact {
    if result.valid && result.disposition == crunch_release_core::CAIRN_HANDOFF_DISPOSITION_PRESENT {
        return ReleaseVerificationFact::satisfied();
    }
    if result.valid && result.disposition == crunch_release_core::CAIRN_HANDOFF_DISPOSITION_ABSENT {
        return ReleaseVerificationFact::absent(Vec::new());
    }
    ReleaseVerificationFact::rejected(required_policy_diagnostics(
        Some(format!("Cairn handoff evidence status is {}", result.disposition)),
        &result.diagnostics,
    ))
}

fn external_evidence_roles_fact(
    manifest: &crate::release_evidence::ReleaseEvidenceManifest,
    required_roles: &[String],
) -> ReleaseVerificationFact {
    if required_roles.is_empty() {
        return ReleaseVerificationFact::not_evaluated();
    }
    let Ok(role_count_max) = usize::try_from(crunch_release_core::MAX_RELEASE_VERIFICATION_REQUIRED_EXTERNAL_ROLES)
    else {
        return ReleaseVerificationFact::rejected(vec![
            "required external evidence role limit does not fit usize".to_string(),
        ]);
    };
    if required_roles.len() > role_count_max {
        return ReleaseVerificationFact::rejected(vec![format!(
            "required external evidence role count exceeds limit {}: observed {}",
            crunch_release_core::MAX_RELEASE_VERIFICATION_REQUIRED_EXTERNAL_ROLES,
            required_roles.len()
        )]);
    }
    debug_assert!(required_roles.len() <= role_count_max);
    debug_assert!(!manifest.release_id.is_empty());

    let missing = required_roles
        .iter()
        .filter(|required_role| {
            !manifest.external_evidence.iter().any(|evidence| evidence.role.as_str() == required_role.as_str())
        })
        .map(|required_role| format!("external evidence role required but missing: {required_role}"))
        .collect::<Vec<_>>();
    if missing.is_empty() {
        ReleaseVerificationFact::satisfied()
    } else {
        ReleaseVerificationFact::rejected(missing)
    }
}

fn stagex_fact(result: Option<&crunch_bootstrap_core::StagexNoQuorumResult>) -> ReleaseVerificationFact {
    let Some(result) = result else {
        return ReleaseVerificationFact::not_evaluated();
    };
    if result.status.is_satisfied() {
        return ReleaseVerificationFact::satisfied();
    }
    ReleaseVerificationFact::rejected(required_policy_diagnostics(
        Some("StageX no-quorum profile unsatisfied".to_string()),
        &result.failure_reasons,
    ))
}

fn required_policy_diagnostics(headline: Option<String>, details: &[String]) -> Vec<String> {
    let mut diagnostics = Vec::with_capacity(details.len().saturating_add(usize::from(headline.is_some())));
    diagnostics.extend(headline);
    diagnostics.extend(details.iter().cloned());
    diagnostics
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct DeterministicVerifyRequest {
    proof_path: Option<PathBuf>,
    isolation_evidence_path: Option<PathBuf>,
    required: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DeterministicProofSource {
    External,
    Bundled,
}

impl DeterministicProofSource {
    fn as_str(self) -> &'static str {
        match self {
            Self::External => "external",
            Self::Bundled => "bundled",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ResolvedDeterministicProofPaths {
    proof_path: PathBuf,
    isolation_evidence_path: PathBuf,
    source: DeterministicProofSource,
}

impl DeterministicVerifyRequest {
    fn new(
        current_dir: &Path,
        proof_path: Option<PathBuf>,
        isolation_evidence_path: Option<PathBuf>,
        required: bool,
    ) -> Self {
        Self {
            proof_path: proof_path.map(|path| resolve_input_path(current_dir, path)),
            isolation_evidence_path: isolation_evidence_path.map(|path| resolve_input_path(current_dir, path)),
            required,
        }
    }
}

fn evaluate_provider_fixed_point_proof(
    current_dir: &Path,
    bundle_dir: &Path,
    manifest: &crate::release_evidence::ReleaseEvidenceManifest,
    proof_path: Option<PathBuf>,
    required: bool,
) -> crate::cargo_free_self_build::ProviderFixedPointProofVerification {
    debug_assert!(current_dir.components().next().is_some());
    debug_assert!(bundle_dir.components().next().is_some());
    if let Some(proof_path) = proof_path {
        let resolved = resolve_input_path(current_dir, proof_path);
        return verify_provider_fixed_point_with_release_context(
            &resolved,
            PROVIDER_FIXED_POINT_SOURCE_EXTERNAL,
            None,
            None,
            &manifest.binaries,
        );
    }
    let Some(artifact) = &manifest.provider_fixed_point_proof else {
        return crate::cargo_free_self_build::ProviderFixedPointProofVerification::absent(required);
    };
    let resolved = bundle_dir.join(&artifact.relative_path);
    verify_provider_fixed_point_with_release_context(
        &resolved,
        PROVIDER_FIXED_POINT_SOURCE_BUNDLED,
        Some(artifact.digest_blake3.clone()),
        Some(artifact.evidence_role.clone()),
        &manifest.binaries,
    )
}

fn verify_provider_fixed_point_with_release_context(
    proof_dir: &Path,
    proof_source: &str,
    manifest_artifact_digest_blake3: Option<String>,
    bounded_evidence_role: Option<String>,
    release_binaries: &[crate::release_evidence::BundledArtifact],
) -> crate::cargo_free_self_build::ProviderFixedPointProofVerification {
    let artifact_digest =
        manifest_artifact_digest_blake3.or_else(|| crate::release_evidence::compute_path_blake3_digest(proof_dir).ok());
    let mut result = crate::cargo_free_self_build::verify_provider_fixed_point_proof_bundle(proof_dir);
    result.proof_source = proof_source.to_string();
    result.proof_artifact_digest_blake3 = artifact_digest;
    result.bounded_evidence_role = bounded_evidence_role;
    bind_provider_fixed_point_release_artifact(result, release_binaries)
}

fn bind_provider_fixed_point_release_artifact(
    mut result: crate::cargo_free_self_build::ProviderFixedPointProofVerification,
    release_binaries: &[crate::release_evidence::BundledArtifact],
) -> crate::cargo_free_self_build::ProviderFixedPointProofVerification {
    if !result.valid {
        return result;
    }
    let Some(stage_digest) = result.stage_binary_digest_blake3.as_deref() else {
        result.blockers.push("provider fixed-point proof stage binary digest is missing".to_string());
        return invalidate_provider_fixed_point_result(result);
    };
    debug_assert!(result.valid);
    debug_assert!(!stage_digest.is_empty());
    match validate_provider_fixed_point_release_artifact_binding(release_binaries, stage_digest) {
        Ok(binding) => {
            result.release_artifact_relative_path = Some(binding.relative_path);
            result.release_artifact_digest_blake3 = Some(binding.digest_blake3);
            result
        }
        Err(err) => {
            result.blockers.push(err.to_string());
            invalidate_provider_fixed_point_result(result)
        }
    }
}

fn invalidate_provider_fixed_point_result(
    mut result: crate::cargo_free_self_build::ProviderFixedPointProofVerification,
) -> crate::cargo_free_self_build::ProviderFixedPointProofVerification {
    result.valid = false;
    result.status = "invalid".to_string();
    result
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct DeterministicReleaseVerifyResult {
    status: &'static str,
    eligible: bool,
    proof_path: Option<PathBuf>,
    proof_digest_blake3: Option<String>,
    isolation_evidence_path: Option<PathBuf>,
    isolation_evidence_digest_blake3: Option<String>,
    proof_source: Option<&'static str>,
    blockers: Vec<String>,
}

impl DeterministicReleaseVerifyResult {
    fn absent(blockers: Vec<String>) -> Self {
        Self {
            status: "absent",
            eligible: false,
            proof_path: None,
            proof_digest_blake3: None,
            isolation_evidence_path: None,
            isolation_evidence_digest_blake3: None,
            proof_source: None,
            blockers,
        }
    }
}

// r[impl mantle.release_provenance.deterministic_rebuild_admission.validation]
fn evaluate_deterministic_release_claim(
    manifest: &crate::release_evidence::ReleaseEvidenceManifest,
    bundle_dir: &Path,
    request: DeterministicVerifyRequest,
) -> Result<DeterministicReleaseVerifyResult, RunError> {
    let missing_blockers = deterministic_missing_blockers(manifest, &request);
    let Some(resolved) = resolve_deterministic_proof_paths(manifest, bundle_dir, &request) else {
        return Ok(DeterministicReleaseVerifyResult::absent(missing_blockers));
    };

    let proof = load_canonical_deterministic_proof(&resolved.proof_path)?;
    let isolation_evidence = load_canonical_deterministic_isolation_evidence(&resolved.isolation_evidence_path)?;
    let release_digest_set =
        manifest.binaries.iter().map(|artifact| artifact.digest_blake3.clone()).collect::<Vec<_>>();
    let is_genuine_rebuild = deterministic_build_proof_has_genuine_rebuild_authority(proof.value.clone())
        .map_err(|err| RunError::Internal(format!("deterministic rebuild authority validation failed: {err}")))?;
    let is_eligible =
        deterministic_release_claim_eligible(&release_digest_set, &[proof.value], Some(&isolation_evidence.value))
            .map_err(|err| RunError::Internal(format!("deterministic release claim validation failed: {err}")))?;
    debug_assert!(resolved.proof_path.components().next().is_some());
    debug_assert!(resolved.isolation_evidence_path.components().next().is_some());
    let blockers = if is_eligible {
        Vec::new()
    } else if !is_genuine_rebuild {
        vec![
            "missing-genuine-rebuild-evidence: legacy path-bound or incomplete deterministic receipt is non-promoting"
                .to_string(),
        ]
    } else {
        vec!["deterministic proof artifacts do not prove the release artifact digest set".to_string()]
    };
    Ok(DeterministicReleaseVerifyResult {
        status: if is_eligible { "eligible" } else { "blocked" },
        eligible: is_eligible,
        proof_path: Some(resolved.proof_path),
        proof_digest_blake3: Some(proof.digest_blake3),
        isolation_evidence_path: Some(resolved.isolation_evidence_path),
        isolation_evidence_digest_blake3: Some(isolation_evidence.digest_blake3),
        proof_source: Some(resolved.source.as_str()),
        blockers,
    })
}

fn resolve_deterministic_proof_paths(
    manifest: &crate::release_evidence::ReleaseEvidenceManifest,
    bundle_dir: &Path,
    request: &DeterministicVerifyRequest,
) -> Option<ResolvedDeterministicProofPaths> {
    if request.proof_path.is_some() || request.isolation_evidence_path.is_some() {
        return Some(ResolvedDeterministicProofPaths {
            proof_path: request.proof_path.clone()?,
            isolation_evidence_path: request.isolation_evidence_path.clone()?,
            source: DeterministicProofSource::External,
        });
    }
    Some(ResolvedDeterministicProofPaths {
        proof_path: bundle_dir.join(&manifest.deterministic_build_proof.as_ref()?.relative_path),
        isolation_evidence_path: bundle_dir
            .join(&manifest.deterministic_sandbox_isolation_evidence.as_ref()?.relative_path),
        source: DeterministicProofSource::Bundled,
    })
}

fn deterministic_missing_blockers(
    manifest: &crate::release_evidence::ReleaseEvidenceManifest,
    request: &DeterministicVerifyRequest,
) -> Vec<String> {
    if request.proof_path.is_some() || request.isolation_evidence_path.is_some() {
        return explicit_deterministic_missing_blockers(request);
    }
    if !request.required {
        return Vec::new();
    }
    bundled_deterministic_missing_blockers(manifest)
}

fn explicit_deterministic_missing_blockers(request: &DeterministicVerifyRequest) -> Vec<String> {
    let mut blockers = Vec::new();
    if request.proof_path.is_none() {
        blockers.push("missing deterministic proof artifact".to_string());
    }
    if request.isolation_evidence_path.is_none() {
        blockers.push("missing deterministic sandbox isolation evidence".to_string());
    }
    blockers
}

fn bundled_deterministic_missing_blockers(manifest: &crate::release_evidence::ReleaseEvidenceManifest) -> Vec<String> {
    let mut blockers = Vec::new();
    if manifest.deterministic_build_proof.is_none() {
        blockers.push("missing bundled deterministic proof artifact".to_string());
    }
    if manifest.deterministic_sandbox_isolation_evidence.is_none() {
        blockers.push("missing bundled deterministic sandbox isolation evidence".to_string());
    }
    blockers
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct VerifiedDeterministicArtifact<T> {
    value: T,
    digest_blake3: String,
}

fn load_canonical_deterministic_proof(
    path: &Path,
) -> Result<VerifiedDeterministicArtifact<DeterministicBuildProofReceipt>, RunError> {
    let bytes = std::fs::read(path).map_err(|err| RunError::Internal(format!("reading {}: {err}", path.display())))?;
    let receipt: DeterministicBuildProofReceipt = serde_json::from_slice(&bytes)
        .map_err(|err| RunError::Internal(format!("parsing {}: {err}", path.display())))?;
    let canonical = deterministic_build_proof_receipt_canonical_bytes(receipt.clone()).map_err(|err| {
        RunError::Internal(format!("validating deterministic proof artifact {}: {err}", path.display()))
    })?;
    if bytes != canonical {
        return Err(RunError::Internal(format!(
            "deterministic proof artifact is not canonical compact JSON: {}",
            path.display()
        )));
    }
    Ok(VerifiedDeterministicArtifact {
        value: receipt,
        digest_blake3: blake3::hash(&canonical).to_hex().to_string(),
    })
}

fn load_canonical_deterministic_isolation_evidence(
    path: &Path,
) -> Result<VerifiedDeterministicArtifact<DeterministicSandboxIsolationEvidence>, RunError> {
    let bytes = std::fs::read(path).map_err(|err| RunError::Internal(format!("reading {}: {err}", path.display())))?;
    let evidence: DeterministicSandboxIsolationEvidence = serde_json::from_slice(&bytes)
        .map_err(|err| RunError::Internal(format!("parsing {}: {err}", path.display())))?;
    let canonical = deterministic_sandbox_isolation_evidence_canonical_bytes(evidence.clone()).map_err(|err| {
        RunError::Internal(format!("validating deterministic sandbox isolation evidence {}: {err}", path.display()))
    })?;
    if bytes != canonical {
        return Err(RunError::Internal(format!(
            "deterministic sandbox isolation evidence is not canonical compact JSON: {}",
            path.display()
        )));
    }
    Ok(VerifiedDeterministicArtifact {
        value: evidence,
        digest_blake3: blake3::hash(&canonical).to_hex().to_string(),
    })
}

// r[impl mantle.operator_diagnostics.release_verification.render_boundary]
// r[impl mantle.operator_diagnostics.release_verification.json_contract]
fn emit_release_verification(evaluation: &ReleaseVerifyEvaluation, is_json_output: bool) -> Result<(), RunError> {
    let is_valid = evaluation.decision.valid;
    if is_json_output {
        let rendered = render_release_verify_json(evaluation)?;
        println!("{rendered}");
    } else {
        let rendered = render_release_verify_human(evaluation)?;
        if is_valid {
            print!("{rendered}");
        } else {
            eprint!("{rendered}");
        }
    }
    if is_valid {
        Ok(())
    } else {
        Err(RunError::Reported(RELEASE_VERIFICATION_REJECTED_EXIT_CODE))
    }
}

fn render_release_verify_json(evaluation: &ReleaseVerifyEvaluation) -> Result<String, RunError> {
    debug_assert!(!evaluation.manifest.release_id.is_empty());
    debug_assert!(!evaluation.decision.schema.is_empty());
    let reproducibility_value = evaluation.reproducibility.as_ref().map(|report| {
        serde_json::json!({
            "path": report.path.display().to_string(),
            "digest_blake3": report.digest_blake3,
        })
    });
    let deterministic_result = &evaluation.deterministic_result;
    let deterministic_json = serde_json::json!({
        "status": deterministic_result.status,
        "eligible": deterministic_result.eligible,
        "proof_path": deterministic_result.proof_path.as_ref().map(|path| path.display().to_string()),
        "proof_digest_blake3": deterministic_result.proof_digest_blake3,
        "proof_source": deterministic_result.proof_source,
        "sandbox_isolation_evidence_path": deterministic_result
            .isolation_evidence_path
            .as_ref()
            .map(|path| path.display().to_string()),
        "sandbox_isolation_evidence_digest_blake3": deterministic_result.isolation_evidence_digest_blake3,
        "blockers": deterministic_result.blockers,
    });
    let decision = &evaluation.decision;
    let mut rendered = serde_json::json!({
        "kind": RELEASE_VERIFY_JSON_KIND,
        "decision_schema": decision.schema,
        "valid": decision.valid,
        "disposition": decision.disposition,
        "checks": decision.checks,
        "diagnostics": decision.diagnostics,
        "release_id": evaluation.manifest.release_id,
        "manifest": evaluation.manifest,
        "reproducibility_status": evaluation.reproducibility_status.as_str(),
        "reproducibility_report": reproducibility_value,
        "deterministic_release": deterministic_json,
        "provider_fixed_point_proof": evaluation.provider_fixed_point_result,
        "release_profile": evaluation.release_profile,
        "stack_provenance": evaluation.stack_provenance_result,
        "content_bound_requirements": evaluation.content_bound_requirement_result,
        "cairn_handoff": evaluation.cairn_handoff_result,
        "global_reproducibility": global_reproducibility_not_evaluated_json(),
    });
    if let Some(result) = &evaluation.stagex_result {
        rendered["stagex_no_quorum"] = serde_json::to_value(result)
            .map_err(|err| RunError::Internal(format!("serializing stagex profile: {err}")))?;
    }
    serde_json::to_string(&rendered)
        .map_err(|err| RunError::Internal(format!("serializing release verification decision: {err}")))
}

// r[impl mantle.operator_diagnostics.release_verification.terminal_verdict]
fn render_release_verify_human(evaluation: &ReleaseVerifyEvaluation) -> Result<String, RunError> {
    let mut output = String::new();
    append_release_verify_human(&mut output, evaluation)
        .map_err(|error| RunError::Internal(format!("rendering release verification output: {error}")))?;
    Ok(output)
}

fn append_release_verify_human(output: &mut String, evaluation: &ReleaseVerifyEvaluation) -> std::fmt::Result {
    debug_assert!(!evaluation.manifest.release_id.is_empty());
    debug_assert!(output.capacity() >= output.len());
    writeln!(output, "release verification verdict: {}", evaluation.decision.disposition.as_str())?;
    writeln!(output, "release id: {}", evaluation.manifest.release_id)?;
    writeln!(output, "binaries: {}", evaluation.manifest.binaries.len())?;
    writeln!(output, "source digest: {}", evaluation.manifest.source_archive.digest_blake3)?;
    writeln!(output, "stage2 digest: {}", evaluation.manifest.proof_linkage.stage2_binary_digest_blake3)?;
    writeln!(output, "proof mode: {}", evaluation.manifest.proof_linkage.proof_mode)?;
    writeln!(output, "release profile: {}", evaluation.release_profile)?;
    if !evaluation.manifest.external_evidence.is_empty() {
        writeln!(output, "external evidence: {}", evaluation.manifest.external_evidence.len())?;
    }
    append_reproducibility_summary(output, evaluation.reproducibility.as_ref())?;
    append_deterministic_release_summary(output, &evaluation.deterministic_result)?;
    append_provider_fixed_point_summary(output, &evaluation.provider_fixed_point_result)?;
    append_stack_provenance_summary(output, &evaluation.stack_provenance_result)?;
    append_content_bound_requirement_summary(output, &evaluation.content_bound_requirement_result)?;
    append_cairn_handoff_summary(output, &evaluation.cairn_handoff_result)?;
    append_global_reproducibility_summary(output)?;
    append_stagex_summary(output, evaluation.stagex_result.as_ref())?;
    append_release_verification_checks(output, &evaluation.decision)?;
    if evaluation.decision.valid {
        writeln!(output, "release evidence verified: {}", evaluation.resolved_bundle_dir.display())?;
    } else {
        writeln!(output, "release evidence rejected: {}", evaluation.resolved_bundle_dir.display())?;
    }
    Ok(())
}

fn append_release_verification_checks(output: &mut String, decision: &ReleaseVerificationDecision) -> std::fmt::Result {
    for check in &decision.checks {
        writeln!(
            output,
            "verification check: contributor={} requirement={} disposition={} blocking={}",
            check.contributor.as_str(),
            check.requirement.as_str(),
            check.disposition.as_str(),
            check.blocking
        )?;
    }
    for diagnostic in &decision.diagnostics {
        writeln!(output, "verification diagnostic: {diagnostic}")?;
    }
    Ok(())
}

fn append_provider_fixed_point_summary(
    output: &mut String,
    result: &crate::cargo_free_self_build::ProviderFixedPointProofVerification,
) -> std::fmt::Result {
    debug_assert!(!result.status.is_empty());
    debug_assert!(!result.proof_source.is_empty());
    writeln!(output, "provider fixed-point proof: {}", result.status)?;
    writeln!(output, "provider fixed-point proof source: {}", result.proof_source)?;
    if let Some(path) = &result.proof_dir {
        writeln!(output, "provider fixed-point proof bundle: {}", path.display())?;
    }
    if let Some(role) = &result.bounded_evidence_role {
        writeln!(output, "provider fixed-point bounded evidence role: {role}")?;
    }
    if let Some(digest) = &result.proof_artifact_digest_blake3 {
        writeln!(output, "provider fixed-point proof artifact digest: {digest}")?;
    }
    if let Some(digest) = &result.stage_binary_digest_blake3 {
        writeln!(output, "provider fixed-point stage binary digest: {digest}")?;
    }
    if let Some(path) = &result.release_artifact_relative_path {
        writeln!(output, "provider fixed-point release artifact: {path}")?;
    }
    if let Some(digest) = &result.release_artifact_digest_blake3 {
        writeln!(output, "provider fixed-point release artifact digest: {digest}")?;
    }
    if let Some(digest) = &result.closure_policy_digest_blake3 {
        writeln!(output, "provider fixed-point closure policy digest: {digest}")?;
    }
    for non_claim in &result.non_claims {
        writeln!(output, "provider fixed-point non-claim: {non_claim}")?;
    }
    for blocker in &result.blockers {
        writeln!(output, "  provider fixed-point blocker: {blocker}")?;
    }
    Ok(())
}

fn append_content_bound_requirement_summary(
    output: &mut String,
    result: &crunch_release_core::ContentBoundRequirementVerificationV1,
) -> Result<(), std::fmt::Error> {
    writeln!(
        output,
        "content-bound requirements: {} (mode {}, requirements {}, evidence rows {})",
        result.disposition, result.mode, result.requirement_count, result.evidence_row_count
    )?;
    for issue in &result.issues {
        writeln!(output, "content-bound requirement diagnostic: {:?} at {}", issue.code, issue.field_path)?;
    }
    Ok(())
}

fn append_cairn_handoff_summary(
    output: &mut String,
    result: &crunch_release_core::CairnHandoffReleaseVerification,
) -> std::fmt::Result {
    writeln!(output, "Cairn handoff: {}", result.disposition)?;
    writeln!(output, "Cairn handoff required: {}", result.required)?;
    writeln!(output, "Cairn handoff boundary: {}", result.boundary)?;
    if let Some(status) = &result.authentication_status {
        writeln!(output, "Cairn handoff authentication: {status}")?;
    }
    for diagnostic in &result.diagnostics {
        writeln!(output, "  Cairn handoff blocker: {diagnostic}")?;
    }
    Ok(())
}

// r[impl mantle.release_provenance.opaque_boundary.visible]
fn append_stack_provenance_summary(
    output: &mut String,
    result: &crunch_release_core::StackProvenanceReleaseVerification,
) -> std::fmt::Result {
    writeln!(output, "stack provenance: {}", result.disposition)?;
    writeln!(output, "stack provenance mode: {}", result.mode)?;
    if let Some(digest) = &result.sidecar_digest_blake3 {
        writeln!(output, "stack provenance sidecar digest: {digest}")?;
    }
    if let Some(digest) = &result.valence_receipt_digest_blake3 {
        writeln!(output, "stack provenance Valence receipt digest: {digest}")?;
    }
    if let Some(path) = &result.release_binary_relative_path {
        writeln!(output, "stack provenance release binary: {path}")?;
    }
    writeln!(output, "stack provenance boundary: {}", result.boundary)?;
    for diagnostic in &result.diagnostics {
        writeln!(output, "  stack provenance diagnostic: {diagnostic}")?;
    }
    Ok(())
}

fn append_global_reproducibility_summary(output: &mut String) -> std::fmt::Result {
    writeln!(output, "global reproducibility: {GLOBAL_REPRODUCIBILITY_STATUS_NOT_EVALUATED}")?;
    writeln!(output, "global reproducibility reason: {GLOBAL_REPRODUCIBILITY_RELEASE_VERIFY_REASON}")
}

fn append_reproducibility_summary(
    output: &mut String,
    reproducibility: Option<&VerifiedReproducibilityReport>,
) -> std::fmt::Result {
    match reproducibility {
        Some(report) => {
            writeln!(output, "reproducibility: {}", report.status.as_str())?;
            writeln!(output, "reproducibility report: {}", report.path.display())?;
            writeln!(output, "reproducibility report digest: {}", report.digest_blake3)
        }
        None => writeln!(output, "reproducibility: {}", ReproducibilityStatus::Absent.as_str()),
    }
}

fn append_deterministic_release_summary(
    output: &mut String,
    result: &DeterministicReleaseVerifyResult,
) -> std::fmt::Result {
    debug_assert!(!result.status.is_empty());
    debug_assert!(result.blockers.capacity() >= result.blockers.len());
    writeln!(output, "deterministic release: {}", result.status)?;
    if let Some(path) = &result.proof_path {
        writeln!(output, "deterministic proof: {}", path.display())?;
    }
    if let Some(digest) = &result.proof_digest_blake3 {
        writeln!(output, "deterministic proof digest: {digest}")?;
    }
    if let Some(source) = result.proof_source {
        writeln!(output, "deterministic proof source: {source}")?;
    }
    if let Some(path) = &result.isolation_evidence_path {
        writeln!(output, "deterministic sandbox isolation evidence: {}", path.display())?;
    }
    if let Some(digest) = &result.isolation_evidence_digest_blake3 {
        writeln!(output, "deterministic sandbox isolation evidence digest: {digest}")?;
    }
    for blocker in &result.blockers {
        writeln!(output, "  deterministic blocker: {blocker}")?;
    }
    Ok(())
}

fn append_stagex_summary(
    output: &mut String,
    result: Option<&crunch_bootstrap_core::StagexNoQuorumResult>,
) -> std::fmt::Result {
    let Some(result) = result else {
        return Ok(());
    };
    writeln!(output, "stagex no-quorum profile: {}", result.status)?;
    for reason in &result.failure_reasons {
        writeln!(output, "  failure: {reason}")?;
    }
    Ok(())
}

fn global_reproducibility_not_evaluated_json() -> serde_json::Value {
    serde_json::json!({
        "status": GLOBAL_REPRODUCIBILITY_STATUS_NOT_EVALUATED,
        "claim_class": "non-global",
        "reason": GLOBAL_REPRODUCIBILITY_RELEASE_VERIFY_REASON,
    })
}

fn reproducibility_status(reproducibility: Option<&VerifiedReproducibilityReport>) -> ReproducibilityStatus {
    reproducibility.map(|report| report.status).unwrap_or(ReproducibilityStatus::Absent)
}

fn evaluate_stagex_profile(
    manifest: &crate::release_evidence::ReleaseEvidenceManifest,
    bundle_dir: &Path,
    reproducibility: Option<&VerifiedReproducibilityReport>,
) -> crunch_bootstrap_core::StagexNoQuorumResult {
    let proof_block = extract_stagex_proof_block(manifest, bundle_dir);
    let is_repro_verified = reproducibility.map(|r| r.status == ReproducibilityStatus::Matched).unwrap_or(false);
    let repro_digest = reproducibility.map(|r| r.digest_blake3.as_str()).unwrap_or("");
    let artifact_set_digest = compute_artifact_set_digest(&manifest.binaries);
    crunch_bootstrap_core::evaluate_stagex_no_quorum(
        true, // bundle already verified by verify_release_evidence_bundle
        proof_block.as_ref(),
        is_repro_verified,
        repro_digest,
        &manifest.release_id,
        &artifact_set_digest,
    )
}

fn extract_stagex_proof_block(
    manifest: &crate::release_evidence::ReleaseEvidenceManifest,
    bundle_dir: &Path,
) -> Option<crunch_bootstrap_core::StagexLineageProofBlock> {
    debug_assert!(bundle_dir.components().next().is_some());
    debug_assert!(!manifest.proof_bundle.relative_path.is_empty());
    let proof_bundle_dir = bundle_dir.join(&manifest.proof_bundle.relative_path);
    let summary_path = proof_bundle_dir.join("summary.txt");
    let summary_text = std::fs::read_to_string(&summary_path).ok()?;
    let self_build_evidence = crate::self_build::SelfBuildReport::parse_proof_lines(&summary_text)?;
    let meta = self_build_evidence.stagex_metadata?;
    Some(crunch_bootstrap_core::StagexLineageProofBlock {
        seed_class: meta.seed_class,
        audit_seed_max_bytes: meta.audit_seed_max_bytes,
        seed_digest: meta.seed_digest_blake3,
        lineage_manifest_digest: meta.lineage_manifest_digest_blake3,
        stage_graph_digest: meta.stage_graph_digest_blake3,
        normalized_provider_digest: meta.provider_output_digest_blake3,
        staged_source_digest: meta.staged_source_digest_blake3,
        stage1_crunch_digest: meta.stage1_binary_digest_blake3,
        stage2_crunch_digest: meta.stage2_binary_digest_blake3,
        bootstrap_tool_digests: meta
            .bootstrap_tool_digests
            .into_iter()
            .map(|t| crunch_bootstrap_core::BootstrapToolDigest {
                name: t.name,
                digest: t.digest_blake3,
            })
            .collect(),
        protected_exec_audit_digest: Some(meta.protected_exec_audit_digest_blake3),
        proof_bundle_digest: meta.proof_bundle_digest_blake3,
    })
}

fn compute_artifact_set_digest(binaries: &[crate::release_evidence::BundledArtifact]) -> String {
    let mut hasher = blake3::Hasher::new();
    let mut sorted_digests: Vec<&str> = binaries.iter().map(|b| b.digest_blake3.as_str()).collect();
    sorted_digests.sort();
    for digest in &sorted_digests {
        hasher.update(digest.as_bytes());
        hasher.update(b"\n");
    }
    hasher.finalize().to_hex().to_string()
}

struct ReleaseReproduceCommand {
    bundle_dir: PathBuf,
    rebuild_output_dir: PathBuf,
    rebuild_command: PathBuf,
    rebuild_args: Vec<std::ffi::OsString>,
    workflow_version: String,
    report_path: Option<PathBuf>,
    deterministic_proof_runs: u32,
    deterministic_proof_dir: Option<PathBuf>,
}

fn release_reproduce_command(action: crate::ReleaseAction) -> Result<ReleaseReproduceCommand, RunError> {
    let crate::ReleaseAction::Reproduce {
        bundle_dir,
        rebuild_output_dir,
        rebuild_command,
        rebuild_args,
        workflow_version,
        report_path,
        deterministic_proof_runs,
        deterministic_proof_dir,
    } = action
    else {
        return Err(RunError::Internal("expected release reproduce action".to_string()));
    };
    debug_assert!(rebuild_args.capacity() >= rebuild_args.len());
    debug_assert!(workflow_version.capacity() >= workflow_version.len());
    Ok(ReleaseReproduceCommand {
        bundle_dir,
        rebuild_output_dir,
        rebuild_command,
        rebuild_args,
        workflow_version,
        report_path,
        deterministic_proof_runs,
        deterministic_proof_dir,
    })
}

fn cmd_release_reproduce(context: ReleaseCommandContext<'_>, command: ReleaseReproduceCommand) -> Result<(), RunError> {
    let request = ReleaseReproduceRequest {
        bundle_dir: resolve_input_path(context.current_dir, command.bundle_dir),
        rebuild_output_dir: resolve_input_path(context.current_dir, command.rebuild_output_dir),
        rebuild_command: resolve_input_path(context.current_dir, command.rebuild_command),
        rebuild_args: command.rebuild_args,
        workflow_version: normalize_workflow_value(&command.workflow_version, DEFAULT_REPRODUCIBILITY_WORKFLOW_VERSION),
        report_path: command.report_path.map(|path| resolve_input_path(context.current_dir, path)),
        deterministic_proof_runs: command.deterministic_proof_runs,
        deterministic_proof_dir: command
            .deterministic_proof_dir
            .map(|path| resolve_input_path(context.current_dir, path)),
    };
    let summary = reproduce_release_artifacts(&request)?;
    emit_release_reproduce_result(&summary, context.is_json_output)
}

fn emit_release_reproduce_result(summary: &ReleaseReproduceSummary, is_json_output: bool) -> Result<(), RunError> {
    debug_assert!(!summary.release_id.is_empty());
    debug_assert!(!summary.report_digest_blake3.is_empty());
    if is_json_output {
        return emit_release_reproduce_json(summary);
    }
    emit_release_reproduce_human(summary);
    Ok(())
}

fn emit_release_reproduce_json(summary: &ReleaseReproduceSummary) -> Result<(), RunError> {
    debug_assert!(!summary.release_id.is_empty());
    debug_assert!(!summary.report_digest_blake3.is_empty());
    let rendered = serde_json::json!({
        "release_id": summary.release_id,
        "report_path": summary.report_path.display().to_string(),
        "report_digest_blake3": summary.report_digest_blake3,
        "matched_count": summary.matched_count,
        "mismatched_count": summary.mismatched_count,
        "missing_count": summary.missing_count,
        "deterministic_proof_path": summary.deterministic_proof_path.as_ref().map(|path| path.display().to_string()),
        "deterministic_proof_digest_blake3": summary.deterministic_proof_digest_blake3,
        "deterministic_sandbox_isolation_evidence_path": summary.deterministic_sandbox_isolation_evidence_path.as_ref().map(|path| path.display().to_string()),
        "deterministic_sandbox_isolation_evidence_digest_blake3": summary.deterministic_sandbox_isolation_evidence_digest_blake3,
        "deterministic_proof_unit": summary.deterministic_proof_unit,
        "deterministic_proof_run_roots": summary.deterministic_proof_run_roots,
        "deterministic_proof_sandbox_profiles": summary.deterministic_proof_sandbox_profiles,
        "deterministic_proof_verdict": summary.deterministic_proof_verdict,
        "deterministic_proof_blockers": summary.deterministic_proof_blockers,
    });
    println!(
        "{}",
        serde_json::to_string(&rendered)
            .map_err(|err| RunError::Internal(format!("serializing reproducibility output: {err}")))?
    );
    Ok(())
}

fn emit_release_reproduce_human(summary: &ReleaseReproduceSummary) {
    debug_assert!(!summary.release_id.is_empty());
    debug_assert!(!summary.report_digest_blake3.is_empty());
    println!("release reproducibility report: {}", summary.report_path.display());
    println!("release id: {}", summary.release_id);
    println!("report digest: {}", summary.report_digest_blake3);
    println!("matched artifacts: {}", summary.matched_count);
    println!("mismatched artifacts: {}", summary.mismatched_count);
    println!("missing artifacts: {}", summary.missing_count);
    if let Some(path) = &summary.deterministic_proof_path {
        println!("deterministic proof: {}", path.display());
    }
    if let Some(digest) = &summary.deterministic_proof_digest_blake3 {
        println!("deterministic proof digest: {digest}");
    }
    if let Some(path) = &summary.deterministic_sandbox_isolation_evidence_path {
        println!("deterministic sandbox isolation evidence: {}", path.display());
    }
    if let Some(digest) = &summary.deterministic_sandbox_isolation_evidence_digest_blake3 {
        println!("deterministic sandbox isolation evidence digest: {digest}");
    }
    emit_optional_release_reproduce_human(summary);
}

fn emit_optional_release_reproduce_human(summary: &ReleaseReproduceSummary) {
    if let Some(verdict) = &summary.deterministic_proof_verdict {
        println!("deterministic proof verdict: {verdict}");
    }
    if let Some(proof_unit) = &summary.deterministic_proof_unit {
        println!("deterministic proof unit: {proof_unit}");
    }
    if let Some(run_roots) = &summary.deterministic_proof_run_roots {
        println!("deterministic proof run roots: {run_roots}");
    }
    if let Some(profiles) = &summary.deterministic_proof_sandbox_profiles {
        println!("deterministic proof sandbox profiles: {}", profiles.join(","));
    }
    if let Some(blockers) = &summary.deterministic_proof_blockers
        && !blockers.is_empty()
    {
        println!("deterministic proof blockers: {}", blockers.join("; "));
    }
}

fn release_nix_witness_command(action: crate::ReleaseAction) -> Result<ReleaseNixWitnessCommand, RunError> {
    let crate::ReleaseAction::NixWitness {
        bundle_dir,
        nix_output_dir,
        deterministic_proof,
        receipt_path,
        rust_toolchain_identity,
        target_triple,
        build_flags,
        linker_identity,
        strip_debug_policy,
        source_date_epoch_policy,
        nix_derivation_identity,
        nix_output_identity,
        require_match,
    } = action
    else {
        return Err(RunError::Internal("expected release Nix witness action".to_string()));
    };
    debug_assert!(build_flags.capacity() >= build_flags.len());
    debug_assert!(rust_toolchain_identity.capacity() >= rust_toolchain_identity.len());
    Ok(ReleaseNixWitnessCommand {
        bundle_dir,
        nix_output_dir,
        deterministic_proof,
        receipt_path,
        rust_toolchain_identity,
        target_triple,
        build_flags,
        linker_identity,
        strip_debug_policy,
        source_date_epoch_policy,
        nix_derivation_identity,
        nix_output_identity,
        is_require_match: require_match,
    })
}

struct ReleaseNixWitnessCommand {
    bundle_dir: PathBuf,
    nix_output_dir: PathBuf,
    deterministic_proof: PathBuf,
    receipt_path: Option<PathBuf>,
    rust_toolchain_identity: String,
    target_triple: String,
    build_flags: Vec<String>,
    linker_identity: Option<String>,
    strip_debug_policy: String,
    source_date_epoch_policy: String,
    nix_derivation_identity: String,
    nix_output_identity: String,
    is_require_match: bool,
}

fn cmd_release_nix_witness(
    context: ReleaseCommandContext<'_>,
    command: ReleaseNixWitnessCommand,
) -> Result<(), RunError> {
    let request = ReleaseNixWitnessRequest {
        bundle_dir: resolve_input_path(context.current_dir, command.bundle_dir),
        nix_output_dir: resolve_input_path(context.current_dir, command.nix_output_dir),
        deterministic_proof_path: resolve_input_path(context.current_dir, command.deterministic_proof),
        output_path: command.receipt_path.map(|path| resolve_input_path(context.current_dir, path)),
        rust_toolchain_identity: command.rust_toolchain_identity,
        target_triple: command.target_triple,
        build_flags: command.build_flags,
        linker_identity: command.linker_identity,
        strip_debug_policy: command.strip_debug_policy,
        source_date_epoch_policy: command.source_date_epoch_policy,
        nix_derivation_identity: command.nix_derivation_identity,
        nix_output_identity: command.nix_output_identity,
        require_match: command.is_require_match,
    };
    let summary = write_release_nix_cross_builder_witness(&request)?;
    debug_assert!(!summary.release_id.is_empty());
    debug_assert!(!summary.receipt_digest_blake3.is_empty());
    if context.is_json_output {
        let rendered = serde_json::json!({
            "kind": "mantle-nix-cross-builder-witness-run-v1",
            "release_id": summary.release_id,
            "receipt_path": summary.receipt_path.display().to_string(),
            "receipt_digest_blake3": summary.receipt_digest_blake3,
            "comparison_verdict": summary.verdict_label(),
            "proof_class": summary.proof_class,
            "mantle_artifact_count": summary.mantle_artifact_count,
            "nix_artifact_count": summary.nix_artifact_count,
            "bounded_claim": "Nix-built selected release artifacts matched Mantle artifacts only when comparison_verdict is nix-witness-match; this does not replace self-rebuild-match or prove global reproducibility.",
        });
        println!(
            "{}",
            serde_json::to_string(&rendered)
                .map_err(|err| RunError::Internal(format!("serializing nix witness output: {err}")))?
        );
        return Ok(());
    }
    println!("nix cross-builder witness: {}", summary.verdict_label());
    println!("release id: {}", summary.release_id);
    println!("receipt: {}", summary.receipt_path.display());
    println!("receipt digest: {}", summary.receipt_digest_blake3);
    if let Some(proof_class) = &summary.proof_class {
        println!("proof class: {proof_class}");
    }
    println!(
        "bounded claim: Nix-built selected release artifacts matched Mantle artifacts only when verdict is nix-witness-match; this does not replace self-rebuild-match or prove global reproducibility."
    );
    Ok(())
}

fn cmd_release_attest(
    context: ReleaseCommandContext<'_>,
    bundle_dir: PathBuf,
    verification_dir: Option<PathBuf>,
    signing_key: Option<PathBuf>,
) -> Result<(), RunError> {
    let current_dir = context.current_dir;
    let resolved_bundle_dir = resolve_input_path(current_dir, bundle_dir);
    let verified_manifest = verify_release_evidence_bundle(&resolved_bundle_dir)?;
    let resolved_verification_dir = match verification_dir {
        Some(path) => resolve_input_path(current_dir, path),
        None => default_verification_dir(current_dir, &verified_manifest.release_id),
    };
    debug_assert!(!verified_manifest.release_id.is_empty());
    debug_assert!(resolved_bundle_dir.components().next().is_some());
    let created = create_release_attestation(
        &verified_manifest,
        &resolved_verification_dir,
        signing_key.as_deref(),
        context.state_dir,
    )?;
    if context.is_json_output {
        let rendered = serde_json::json!({
            "release_id": created.attestation.release_id,
            "digest": created.digest_hex,
            "signer": created.signer_key_name,
            "attestation_path": created.attestation_path.display().to_string(),
            "signature_path": created.signature_path.display().to_string(),
        });
        println!(
            "{}",
            serde_json::to_string(&rendered)
                .map_err(|err| RunError::Internal(format!("serializing release attestation output: {err}")))?
        );
        return Ok(());
    }

    println!("release attestation: {}", created.attestation_path.display());
    println!("signature: {}", created.signature_path.display());
    println!("release id: {}", created.attestation.release_id);
    println!("digest: {}", created.digest_hex);
    println!("signer: {}", created.signer_key_name);
    Ok(())
}

fn cmd_release_witness_export(
    current_dir: &Path,
    is_json_output: bool,
    bundle_dir: PathBuf,
    verification_dir: Option<PathBuf>,
    request_dir: Option<PathBuf>,
) -> Result<(), RunError> {
    let resolved_bundle_dir = resolve_input_path(current_dir, bundle_dir);
    let verified_manifest = verify_release_evidence_bundle(&resolved_bundle_dir)?;
    let resolved_verification_dir = match verification_dir {
        Some(path) => resolve_input_path(current_dir, path),
        None => default_verification_dir(current_dir, &verified_manifest.release_id),
    };
    let resolved_request_dir = match request_dir {
        Some(path) => resolve_input_path(current_dir, path),
        None => default_witness_request_dir(current_dir, &verified_manifest.release_id),
    };
    debug_assert!(!verified_manifest.release_id.is_empty());
    debug_assert!(resolved_request_dir.components().next().is_some());
    let created = create_witness_request_directory(
        &verified_manifest,
        &resolved_bundle_dir,
        &resolved_verification_dir,
        &resolved_request_dir,
    )?;
    if is_json_output {
        let rendered = serde_json::json!({
            "kind": "mantle-witness-request",
            "release_id": created.release_id,
            "layout_version": created.layout_version,
            "request_dir": created.request_dir.display().to_string(),
            "request_path": created.request_path.display().to_string(),
            "release_bundle_path": created.release_bundle_path.display().to_string(),
            "verification_seed_path": created.verification_seed_path.display().to_string(),
        });
        println!(
            "{}",
            serde_json::to_string(&rendered)
                .map_err(|err| RunError::Internal(format!("serializing witness request output: {err}")))?
        );
        return Ok(());
    }

    println!("witness request: {}", created.request_dir.display());
    println!("request metadata: {}", created.request_path.display());
    println!("release id: {}", created.release_id);
    println!("layout version: {}", created.layout_version);
    println!("release bundle copy: {}", created.release_bundle_path.display());
    println!("verification seed: {}", created.verification_seed_path.display());
    Ok(())
}

fn release_witness_rebuild_command(action: crate::ReleaseAction) -> Result<ReleaseWitnessRebuildCommand, RunError> {
    let crate::ReleaseAction::WitnessRebuild {
        request_dir,
        scratch_dir,
        check,
        require_independent_source,
        require_git_source,
        identity,
        system,
        toolchain,
        host_class,
        signing_key,
    } = action
    else {
        return Err(RunError::Internal("expected release witness rebuild action".to_string()));
    };
    debug_assert!(identity.as_ref().is_none_or(|value| value.capacity() >= value.len()));
    debug_assert!(host_class.as_ref().is_none_or(|value| value.capacity() >= value.len()));
    Ok(ReleaseWitnessRebuildCommand {
        request_dir,
        scratch_dir,
        is_check: check,
        is_require_independent_source: require_independent_source,
        is_require_git_source: require_git_source,
        identity,
        system,
        toolchain,
        host_class,
        signing_key,
    })
}

struct ReleaseWitnessRebuildCommand {
    request_dir: PathBuf,
    scratch_dir: Option<PathBuf>,
    is_check: bool,
    is_require_independent_source: bool,
    is_require_git_source: bool,
    identity: Option<String>,
    system: Option<String>,
    toolchain: Option<String>,
    host_class: Option<String>,
    signing_key: Option<PathBuf>,
}

fn cmd_release_witness_rebuild(
    context: ReleaseCommandContext<'_>,
    command: ReleaseWitnessRebuildCommand,
) -> Result<(), RunError> {
    let resolved_request_dir = resolve_input_path(context.current_dir, command.request_dir);
    let resolved_scratch_dir =
        resolve_witness_scratch_dir(context.current_dir, &resolved_request_dir, command.scratch_dir)?;
    let plan = plan_witness_rebuild(
        &resolved_request_dir,
        &resolved_scratch_dir,
        command.is_require_independent_source,
        command.is_require_git_source,
    )?;
    debug_assert!(!plan.release_id.is_empty());
    debug_assert!(plan.request_dir.components().next().is_some());
    if command.is_check {
        return print_witness_rebuild_check(&plan, context.is_json_output);
    }

    let metadata = required_witness_rebuild_metadata(command.system, command.toolchain, command.host_class)?;
    if let Err(err) = prepare_witness_rebuild_scratch(&plan) {
        let failure_meta = build_prelaunch_failure_audit_meta(&plan, err.message())?;
        write_audit_meta(&audit_meta_path(&plan), &failure_meta)?;
        return Err(err);
    }
    let execution = run_witness_rebuild_workflow(&plan)?;
    if let Err(err) = validate_successful_rebuild(&plan, &execution) {
        let failure_meta = build_failure_audit_meta(
            &plan,
            &execution.workflow_driver_path,
            &execution.rebuilt_output_paths,
            execution.started_unix_ms,
            execution.finished_unix_ms,
            err.message(),
        )?;
        write_audit_meta(&audit_meta_path(&plan), &failure_meta)?;
        return Err(err);
    }
    let success = WitnessRebuildSuccess {
        rebuilt_output_paths: execution.rebuilt_output_paths,
        workflow_driver_path: execution.workflow_driver_path,
        started_unix_ms: execution.started_unix_ms,
        finished_unix_ms: execution.finished_unix_ms,
        output: execution.output,
    };
    let created = create_witness_attestation(
        &plan.scratch_layout.verification_dir,
        &success.rebuilt_output_paths,
        command.identity.as_deref(),
        &metadata.system,
        &metadata.toolchain,
        &metadata.host_class,
        source_acquisition_mode_for_plan(&plan),
        command.signing_key.as_deref(),
        context.state_dir,
    )?;
    let audit_meta = build_success_audit_meta(&plan, &success, &created.attestation_path, &created.signature_path)?;
    write_audit_meta(&audit_meta_path(&plan), &audit_meta)?;
    print_witness_rebuild_success(
        &plan,
        &success,
        &created.attestation_path,
        &created.signature_path,
        context.is_json_output,
    )
}

fn resolve_bundle_dir(current_dir: &Path, release_id: &str, bundle_dir: Option<PathBuf>) -> PathBuf {
    match bundle_dir {
        Some(path) => resolve_input_path(current_dir, path),
        None => current_dir.join("target").join("release-evidence").join(release_id),
    }
}

fn resolve_input_paths(current_dir: &Path, paths: Vec<PathBuf>) -> Vec<PathBuf> {
    paths.into_iter().map(|path| resolve_input_path(current_dir, path)).collect()
}

fn resolve_input_path(current_dir: &Path, path: PathBuf) -> PathBuf {
    if path.is_absolute() {
        path
    } else {
        current_dir.join(path)
    }
}

fn resolve_witness_scratch_dir(
    current_dir: &Path,
    request_dir: &Path,
    scratch_dir: Option<PathBuf>,
) -> Result<PathBuf, RunError> {
    if let Some(path) = scratch_dir {
        return Ok(resolve_input_path(current_dir, path));
    }
    if let Some(path_text) = std::env::var_os(WITNESS_SCRATCH_ENV) {
        return Ok(resolve_input_path(current_dir, PathBuf::from(path_text)));
    }
    default_witness_scratch_dir(request_dir)
}

fn normalize_workflow_value<V: AsRef<str>, D: AsRef<str>>(value: V, default_value: D) -> String {
    let value = value.as_ref();
    let default_value = default_value.as_ref();
    if value.trim().is_empty() {
        default_value.to_string()
    } else {
        value.to_string()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct WitnessRebuildMetadata {
    system: String,
    toolchain: String,
    host_class: String,
}

fn required_witness_rebuild_metadata(
    system: Option<String>,
    toolchain: Option<String>,
    host_class: Option<String>,
) -> Result<WitnessRebuildMetadata, RunError> {
    let system = require_metadata_value(system, "--system")?;
    let toolchain = require_metadata_value(toolchain, "--toolchain")?;
    let host_class = require_metadata_value(host_class, "--host-class")?;
    Ok(WitnessRebuildMetadata {
        system,
        toolchain,
        host_class,
    })
}

fn require_metadata_value(value: Option<String>, flag_name: &str) -> Result<String, RunError> {
    let Some(value) = value else {
        return Err(RunError::Internal(format!("{} is required unless --check is used", flag_name)));
    };
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(RunError::Internal(format!("{} must not be empty", flag_name)));
    }
    Ok(trimmed.to_string())
}

fn print_witness_rebuild_check(
    plan: &crate::witness_rebuild::WitnessRebuildPlan,
    is_json_output: bool,
) -> Result<(), RunError> {
    debug_assert!(!plan.release_id.is_empty());
    debug_assert!(plan.scratch_layout.scratch_root.components().next().is_some());
    let audit_path = audit_meta_path(plan);
    if is_json_output {
        let rendered = serde_json::json!({
            "kind": "mantle-witness-rebuild",
            "check_only": true,
            "release_id": plan.release_id,
            "request_dir": plan.request_dir.display().to_string(),
            "scratch_root": plan.scratch_layout.scratch_root.display().to_string(),
            "verification_dir": plan.scratch_layout.verification_dir.display().to_string(),
            "proof_bundle_dir": plan.scratch_layout.proof_bundle_dir.display().to_string(),
            "audit_meta_path": audit_path.display().to_string(),
            "workflow_command": plan.workflow_command,
            "workflow_version": plan.workflow_version,
            "require_independent_source": plan.require_independent_source,
            "require_git_source": plan.require_git_source,
            "source_acquisition_kind": plan.source_acquisition.as_ref().map(|source| source.kind.clone()),
            "source_acquisition_url": plan.source_acquisition.as_ref().map(|source| source.url.clone()),
            "source_acquisition_commit": plan.source_acquisition.as_ref().and_then(|source| source.commit.clone()),
        });
        println!(
            "{}",
            serde_json::to_string(&rendered)
                .map_err(|err| RunError::Internal(format!("serializing witness rebuild check output: {err}")))?
        );
        return Ok(());
    }

    println!("witness rebuild preflight OK: {}", plan.request_dir.display());
    println!("release id: {}", plan.release_id);
    println!("scratch root: {}", plan.scratch_layout.scratch_root.display());
    println!("verification output: {}", plan.scratch_layout.verification_dir.display());
    println!("proof bundle output: {}", plan.scratch_layout.proof_bundle_dir.display());
    println!("require independent source: {}", plan.require_independent_source);
    println!("require Git source: {}", plan.require_git_source);
    if let Some(source_acquisition) = &plan.source_acquisition {
        println!("source acquisition kind: {}", source_acquisition.kind);
        println!("source acquisition URL: {}", source_acquisition.url);
        if let Some(commit) = &source_acquisition.commit {
            println!("source acquisition commit: {commit}");
        }
    }
    println!("audit metadata: {}", audit_path.display());
    println!("check only: no rebuild executed, no witness sidecars written");
    Ok(())
}

fn print_witness_rebuild_success(
    plan: &crate::witness_rebuild::WitnessRebuildPlan,
    success: &WitnessRebuildSuccess,
    attestation_path: &Path,
    signature_path: &Path,
    is_json_output: bool,
) -> Result<(), RunError> {
    debug_assert!(!plan.release_id.is_empty());
    debug_assert!(!success.rebuilt_output_paths.is_empty());
    let audit_path = audit_meta_path(plan);
    if is_json_output {
        let rebuilt_outputs =
            success.rebuilt_output_paths.iter().map(|path| path.display().to_string()).collect::<Vec<_>>();
        let rendered = serde_json::json!({
            "kind": "mantle-witness-rebuild",
            "check_only": false,
            "release_id": plan.release_id,
            "request_dir": plan.request_dir.display().to_string(),
            "scratch_root": plan.scratch_layout.scratch_root.display().to_string(),
            "verification_dir": plan.scratch_layout.verification_dir.display().to_string(),
            "proof_bundle_dir": plan.scratch_layout.proof_bundle_dir.display().to_string(),
            "audit_meta_path": audit_path.display().to_string(),
            "workflow_driver_path": success.workflow_driver_path.display().to_string(),
            "attestation_path": attestation_path.display().to_string(),
            "signature_path": signature_path.display().to_string(),
            "rebuilt_outputs": rebuilt_outputs,
            "require_independent_source": plan.require_independent_source,
            "require_git_source": plan.require_git_source,
            "source_acquisition_kind": plan.source_acquisition.as_ref().map(|source| source.kind.clone()),
            "source_acquisition_url": plan.source_acquisition.as_ref().map(|source| source.url.clone()),
            "source_acquisition_commit": plan.source_acquisition.as_ref().and_then(|source| source.commit.clone()),
            "started_unix_ms": success.started_unix_ms,
            "finished_unix_ms": success.finished_unix_ms,
        });
        println!(
            "{}",
            serde_json::to_string(&rendered)
                .map_err(|err| RunError::Internal(format!("serializing witness rebuild output: {err}")))?
        );
        return Ok(());
    }

    println!("witness rebuild completed: {}", plan.request_dir.display());
    println!("release id: {}", plan.release_id);
    println!("scratch root: {}", plan.scratch_layout.scratch_root.display());
    println!("verification output: {}", plan.scratch_layout.verification_dir.display());
    println!("witness attestation: {}", attestation_path.display());
    println!("signature: {}", signature_path.display());
    println!("require independent source: {}", plan.require_independent_source);
    println!("require Git source: {}", plan.require_git_source);
    if let Some(source_acquisition) = &plan.source_acquisition {
        println!("source acquisition kind: {}", source_acquisition.kind);
        println!("source acquisition URL: {}", source_acquisition.url);
        if let Some(commit) = &source_acquisition.commit {
            println!("source acquisition commit: {commit}");
        }
    }
    println!("rebuild audit: {}", audit_path.display());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ast_grep_external_evidence_is_validated_before_release_attachment() {
        let temp = tempfile::tempdir().unwrap();
        let relative_path = PathBuf::from("ast-grep-evidence.json");
        let absolute_path = temp.path().join(&relative_path);
        let fixture = include_bytes!("../tests/fixtures/ast-grep-structural-evidence/positive-scan.json");
        std::fs::write(&absolute_path, fixture).unwrap();
        let non_claims = vec![
            crunch_release_core::AST_GREP_NON_CLAIM_SOURCE_BEHAVIOR.to_string(),
            crunch_release_core::AST_GREP_NON_CLAIM_BUILD_CORRECTNESS.to_string(),
            crunch_release_core::AST_GREP_NON_CLAIM_CACHE_CORRECTNESS.to_string(),
            crunch_release_core::AST_GREP_NON_CLAIM_RELEASE_ELIGIBILITY.to_string(),
        ];

        let valid = release_create_external_evidence(ExternalEvidenceCommand {
            current_dir: temp.path(),
            paths: vec![relative_path.clone()],
            roles: vec![AST_GREP_EXTERNAL_EVIDENCE_ROLE.to_string()],
            schemas: vec![AST_GREP_STRUCTURAL_EVIDENCE_SCHEMA.to_string()],
            claim_scopes: vec![AST_GREP_STRUCTURAL_CLAIM_SCOPE.to_string()],
            non_claims: non_claims.clone(),
        })
        .unwrap();
        let invalid = release_create_external_evidence(ExternalEvidenceCommand {
            current_dir: temp.path(),
            paths: vec![relative_path],
            roles: vec![AST_GREP_EXTERNAL_EVIDENCE_ROLE.to_string()],
            schemas: vec![AST_GREP_STRUCTURAL_EVIDENCE_SCHEMA.to_string()],
            claim_scopes: vec!["release-eligible".to_string()],
            non_claims,
        })
        .unwrap_err();

        assert_eq!(valid.len(), 1);
        assert_eq!(valid[0].path, absolute_path);
        assert!(invalid.to_string().contains("claim_scope"));
    }

    #[test]
    fn compute_artifact_set_digest_is_deterministic() {
        let a = crate::release_evidence::BundledArtifact {
            kind: crate::release_evidence::BundledArtifactKind::File,
            relative_path: "binaries/crunch".to_string(),
            size_bytes: 100,
            digest_blake3: "a".repeat(64),
        };
        let b = crate::release_evidence::BundledArtifact {
            kind: crate::release_evidence::BundledArtifactKind::File,
            relative_path: "binaries/crunch-alt".to_string(),
            size_bytes: 200,
            digest_blake3: "b".repeat(64),
        };
        let digest_ab = compute_artifact_set_digest(&[a.clone(), b.clone()]);
        let digest_ba = compute_artifact_set_digest(&[b, a]);
        assert_eq!(digest_ab, digest_ba, "order must not affect digest");
        assert_eq!(digest_ab.len(), 64, "must be lowercase BLAKE3 hex");
        assert!(digest_ab.chars().all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase()));
    }

    #[test]
    fn compute_artifact_set_digest_differs_for_different_sets() {
        let a = crate::release_evidence::BundledArtifact {
            kind: crate::release_evidence::BundledArtifactKind::File,
            relative_path: "binaries/crunch".to_string(),
            size_bytes: 100,
            digest_blake3: "a".repeat(64),
        };
        let b = crate::release_evidence::BundledArtifact {
            kind: crate::release_evidence::BundledArtifactKind::File,
            relative_path: "binaries/crunch".to_string(),
            size_bytes: 100,
            digest_blake3: "b".repeat(64),
        };
        assert_ne!(compute_artifact_set_digest(&[a]), compute_artifact_set_digest(&[b]),);
    }

    #[test]
    fn provider_fixed_point_request_absent_is_optional_by_default() {
        let manifest = test_manifest();
        let result =
            evaluate_provider_fixed_point_proof(Path::new("/repo"), Path::new("/bundle"), &manifest, None, false);

        assert!(!result.valid);
        assert_eq!(result.status, "absent");
        assert!(result.blockers.is_empty());
    }

    #[test]
    fn provider_fixed_point_request_absent_records_required_blocker() {
        let manifest = test_manifest();
        let result =
            evaluate_provider_fixed_point_proof(Path::new("/repo"), Path::new("/bundle"), &manifest, None, true);

        assert!(!result.valid);
        assert_eq!(result.status, "absent");
        assert!(result.blockers.iter().any(|blocker| blocker.contains("missing provider fixed-point proof")));
    }

    #[test]
    fn provider_fixed_point_binding_records_matching_release_artifact() {
        let manifest = test_manifest();
        let result = bind_provider_fixed_point_release_artifact(
            valid_provider_fixed_point_result("b".repeat(64)),
            &manifest.binaries,
        );

        let expected_digest = "b".repeat(64);

        assert!(result.valid);
        assert_eq!(result.status, "valid");
        assert_eq!(result.release_artifact_relative_path.as_deref(), Some("binaries/crunch"));
        assert_eq!(result.release_artifact_digest_blake3.as_deref(), Some(expected_digest.as_str()));
    }

    #[test]
    fn provider_fixed_point_binding_rejects_mismatched_release_artifact() {
        let manifest = test_manifest();
        let result = bind_provider_fixed_point_release_artifact(
            valid_provider_fixed_point_result("9".repeat(64)),
            &manifest.binaries,
        );

        assert!(!result.valid);
        assert_eq!(result.status, "invalid");
        assert!(result.release_artifact_relative_path.is_none());
        assert!(
            result
                .blockers
                .iter()
                .any(|blocker| blocker.contains("does not match any bundled release binary artifact")),
            "mismatched provider proof must block required verification: {:?}",
            result.blockers
        );
    }

    #[test]
    fn stagex_profile_json_never_says_quorum_satisfied() {
        let result = crunch_bootstrap_core::StagexNoQuorumResult::unsatisfied(vec!["test".to_string()]);
        let json = serde_json::to_string(&result).unwrap();
        assert!(!json.contains("quorum-satisfied"));
        assert!(!json.contains("quorum_satisfied"));
    }

    #[test]
    fn release_verify_json_keeps_global_reproducibility_non_global() {
        let json = global_reproducibility_not_evaluated_json();

        assert_eq!(json["status"], GLOBAL_REPRODUCIBILITY_STATUS_NOT_EVALUATED);
        assert_eq!(json["claim_class"], "non-global");
        assert!(json["reason"].as_str().unwrap().contains("release verification is scoped"));
    }

    #[test]
    fn extract_stagex_proof_block_returns_none_for_missing_summary() {
        let temp = tempfile::tempdir().unwrap();
        let bundle_dir = temp.path();
        let proof_dir = bundle_dir.join("proof/self-hosting");
        std::fs::create_dir_all(&proof_dir).unwrap();
        let manifest = crate::release_evidence::ReleaseEvidenceManifest {
            schema: "mantle-release-evidence-v1".to_string(),
            release_id: "test-release".to_string(),
            claim_scope: "self-hosting".to_string(),
            workflow: crunch_release_core::ReleaseWorkflowIdentity {
                command: "test".to_string(),
                version: "1".to_string(),
            },
            source_archive: crate::release_evidence::BundledArtifact {
                kind: crate::release_evidence::BundledArtifactKind::File,
                relative_path: "source.tar".to_string(),
                size_bytes: 1,
                digest_blake3: "a".repeat(64),
            },
            source_acquisition: None,
            binaries: vec![],
            proof_bundle: crate::release_evidence::BundledArtifact {
                kind: crate::release_evidence::BundledArtifactKind::Directory,
                relative_path: "proof/self-hosting".to_string(),
                size_bytes: 1,
                digest_blake3: "b".repeat(64),
            },
            prerequisite_inventory: crate::release_evidence::BundledArtifact {
                kind: crate::release_evidence::BundledArtifactKind::File,
                relative_path: "inventory.md".to_string(),
                size_bytes: 1,
                digest_blake3: "c".repeat(64),
            },
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
            proof_linkage: crunch_release_core::ReleaseProofLinkage {
                release_id: "test-release".to_string(),
                source_archive_digest_blake3: "a".repeat(64),
                proof_bundle_schema: "v2".to_string(),
                proof_mode: "fixed-point".to_string(),
                selected_provider_kind: "legacy-fetch".to_string(),
                staged_source: "src".to_string(),
                stage2_binary_digest_blake3: "d".repeat(64),
                prerequisite_inventory_digest_blake3: "e".repeat(64),
                proof_manifest_digest_blake3: "f".repeat(64),
            },
            provenance_coverage: None,
            content_bound_requirement_evidence: None,
        };
        let result = extract_stagex_proof_block(&manifest, bundle_dir);
        assert!(result.is_none(), "missing summary.txt must return None");
    }

    fn valid_provider_fixed_point_result(
        stage_digest: String,
    ) -> crate::cargo_free_self_build::ProviderFixedPointProofVerification {
        crate::cargo_free_self_build::ProviderFixedPointProofVerification {
            status: "valid".to_string(),
            valid: true,
            proof_source: "direct".to_string(),
            proof_dir: Some(PathBuf::from("/tmp/provider-proof")),
            proof_artifact_digest_blake3: Some("a".repeat(64)),
            bounded_evidence_role: Some("cargo-free-source-built-handoff-evidence".to_string()),
            meta_digest_blake3: Some("c".repeat(64)),
            closure_policy_digest_blake3: Some("d".repeat(64)),
            stage_binary_digest_blake3: Some(stage_digest),
            release_artifact_relative_path: None,
            release_artifact_digest_blake3: None,
            stage1_unit_count: Some(1),
            stage2_unit_count: Some(1),
            non_claims: vec!["not-release-reproducibility".to_string()],
            blockers: Vec::new(),
        }
    }

    fn test_manifest() -> crate::release_evidence::ReleaseEvidenceManifest {
        crate::release_evidence::ReleaseEvidenceManifest {
            schema: "mantle-release-evidence-v1".to_string(),
            release_id: "test-release".to_string(),
            claim_scope: "self-hosting".to_string(),
            workflow: crunch_release_core::ReleaseWorkflowIdentity {
                command: "test".to_string(),
                version: "1".to_string(),
            },
            source_archive: crate::release_evidence::BundledArtifact {
                kind: crate::release_evidence::BundledArtifactKind::File,
                relative_path: "source.tar".to_string(),
                size_bytes: 1,
                digest_blake3: "a".repeat(64),
            },
            source_acquisition: None,
            binaries: vec![crate::release_evidence::BundledArtifact {
                kind: crate::release_evidence::BundledArtifactKind::File,
                relative_path: "binaries/crunch".to_string(),
                size_bytes: 100,
                digest_blake3: "b".repeat(64),
            }],
            proof_bundle: crate::release_evidence::BundledArtifact {
                kind: crate::release_evidence::BundledArtifactKind::Directory,
                relative_path: "proof/self-hosting".to_string(),
                size_bytes: 1,
                digest_blake3: "c".repeat(64),
            },
            prerequisite_inventory: crate::release_evidence::BundledArtifact {
                kind: crate::release_evidence::BundledArtifactKind::File,
                relative_path: "inventory.md".to_string(),
                size_bytes: 1,
                digest_blake3: "d".repeat(64),
            },
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
            proof_linkage: crunch_release_core::ReleaseProofLinkage {
                release_id: "test-release".to_string(),
                source_archive_digest_blake3: "a".repeat(64),
                proof_bundle_schema: "v2".to_string(),
                proof_mode: "fixed-point".to_string(),
                selected_provider_kind: "legacy-fetch".to_string(),
                staged_source: "src".to_string(),
                stage2_binary_digest_blake3: "e".repeat(64),
                prerequisite_inventory_digest_blake3: "f".repeat(64),
                proof_manifest_digest_blake3: "1".repeat(64),
            },
            provenance_coverage: None,
            content_bound_requirement_evidence: None,
        }
    }

    fn test_release_verify_evaluation(require_reproducible: bool) -> ReleaseVerifyEvaluation {
        test_release_verify_evaluation_for_profile(require_reproducible, crunch_release_core::RELEASE_PROFILE_GENERIC)
    }

    fn test_release_verify_evaluation_for_profile(
        require_reproducible: bool,
        release_profile: &str,
    ) -> ReleaseVerifyEvaluation {
        test_release_verify_evaluation_for_requirement_mode(
            require_reproducible,
            release_profile,
            crunch_release_core::CONTENT_BOUND_REQUIREMENT_MODE_OPTIONAL,
        )
    }

    fn test_release_verify_evaluation_for_requirement_mode(
        require_reproducible: bool,
        release_profile: &str,
        requirement_coverage_mode: &str,
    ) -> ReleaseVerifyEvaluation {
        let manifest = test_manifest();
        let request = ReleaseVerifyRequest {
            current_dir: PathBuf::from("/tmp"),
            bundle_dir: PathBuf::from("/tmp/release-bundle"),
            require_reproducible,
            require_stagex_no_quorum: false,
            deterministic_proof: None,
            deterministic_sandbox_isolation_evidence: None,
            require_deterministic_release: false,
            provider_fixed_point_proof: None,
            require_external_evidence_role: Vec::new(),
            require_provider_fixed_point_proof: false,
            release_profile: release_profile.to_string(),
            stack_provenance_mode: crunch_release_core::STACK_PROVENANCE_MODE_OPTIONAL.to_string(),
            requirement_coverage_mode: requirement_coverage_mode.to_string(),
        };
        let deterministic_result = DeterministicReleaseVerifyResult::absent(Vec::new());
        let provider_fixed_point_result =
            crate::cargo_free_self_build::ProviderFixedPointProofVerification::absent(false);
        let effective_stack_mode = crunch_release_core::stack_provenance_mode_for_release_profile(
            release_profile,
            crunch_release_core::STACK_PROVENANCE_MODE_OPTIONAL,
        )
        .unwrap();
        let stack_provenance_result =
            crunch_release_core::evaluate_stack_provenance_release_evidence(&manifest, effective_stack_mode);
        let content_bound_requirement_result = crunch_release_core::evaluate_content_bound_requirement_release_evidence(
            &manifest,
            requirement_coverage_mode,
        );
        let cairn_binding = crunch_release_core::release_evidence_cairn_bundle_binding(&manifest).unwrap();
        let cairn_handoff_result = crunch_release_core::evaluate_cairn_handoff_release_evidence(
            manifest.cairn_handoff_validation.as_ref(),
            &cairn_binding,
            release_profile == crunch_release_core::RELEASE_PROFILE_ONIX_STACK,
        );
        let decision = aggregate_release_verify_decision(ReleaseVerifyDecisionInput {
            request: &request,
            manifest: &manifest,
            reproducibility_status: ReproducibilityStatus::Absent,
            deterministic_result: &deterministic_result,
            provider_fixed_point_result: &provider_fixed_point_result,
            effective_stack_mode,
            stack_provenance_result: &stack_provenance_result,
            content_bound_requirement_result: &content_bound_requirement_result,
            cairn_handoff_result: &cairn_handoff_result,
            stagex_result: None,
        });
        ReleaseVerifyEvaluation {
            resolved_bundle_dir: request.bundle_dir,
            manifest,
            reproducibility: None,
            reproducibility_status: ReproducibilityStatus::Absent,
            deterministic_result,
            provider_fixed_point_result,
            release_profile: request.release_profile,
            stack_provenance_result,
            content_bound_requirement_result,
            cairn_handoff_result,
            stagex_result: None,
            decision,
        }
    }

    // r[verify mantle.operator_diagnostics.release_verification.render_boundary.test]
    #[test]
    fn release_verify_renderers_preserve_completed_decision_and_terminal_verdict() {
        let accepted = test_release_verify_evaluation(false);
        let accepted_before = accepted.decision.clone();
        let accepted_human = render_release_verify_human(&accepted).unwrap();
        let accepted_json: serde_json::Value =
            serde_json::from_str(&render_release_verify_json(&accepted).unwrap()).unwrap();

        assert_eq!(accepted.decision, accepted_before);
        assert_eq!(accepted_human.matches("release evidence verified").count(), 1);
        assert!(accepted_human.trim_end().ends_with("release evidence verified: /tmp/release-bundle"));
        assert_eq!(accepted_json["valid"], true);
        assert_eq!(accepted_json["checks"].as_array().unwrap().len(), accepted.decision.checks.len());

        let rejected = test_release_verify_evaluation(true);
        let rejected_before = rejected.decision.clone();
        let rejected_human = render_release_verify_human(&rejected).unwrap();
        let rejected_json: serde_json::Value =
            serde_json::from_str(&render_release_verify_json(&rejected).unwrap()).unwrap();

        assert_eq!(rejected.decision, rejected_before);
        assert!(!rejected_human.contains("release evidence verified"));
        assert!(rejected_human.trim_end().ends_with("release evidence rejected: /tmp/release-bundle"));
        assert_eq!(rejected_json["valid"], false);
        assert_eq!(rejected_json["diagnostics"], serde_json::to_value(&rejected.decision.diagnostics).unwrap());
    }

    #[test]
    fn strict_requirement_coverage_rejects_legacy_only_release_manifest() {
        // r[verify mantle.release_provenance.legacy_coverage_boundary]
        let evaluation = test_release_verify_evaluation_for_requirement_mode(
            false,
            crunch_release_core::RELEASE_PROFILE_GENERIC,
            crunch_release_core::CONTENT_BOUND_REQUIREMENT_MODE_REQUIRED,
        );
        assert!(!evaluation.decision.valid);
        assert!(evaluation.decision.diagnostics.iter().any(|diagnostic| diagnostic.contains("CoverageMissing")));
    }

    // r[verify mantle.build_correctness.onix_release_strict_hermeticity]
    // r[verify mantle.release_provenance.cairn_evidence_handoff.bypass_protection]
    #[test]
    fn onix_profile_rejects_missing_cairn_handoff_and_strict_deterministic_evidence() {
        let evaluation =
            test_release_verify_evaluation_for_profile(false, crunch_release_core::RELEASE_PROFILE_ONIX_STACK);
        let diagnostics = evaluation.decision.diagnostics.join("; ");

        assert!(!evaluation.decision.valid);
        assert!(diagnostics.contains("Cairn handoff"));
        assert!(diagnostics.contains("deterministic release evidence required"));
        assert!(!diagnostics.contains("authenticated provenance proven"));
    }

    // r[verify mantle.build_correctness.onix_release_strict_hermeticity]
    #[test]
    fn onix_profile_admits_when_all_strict_handoff_facts_are_satisfied() {
        let manifest = test_manifest();
        let request = ReleaseVerifyRequest {
            current_dir: PathBuf::from("/tmp"),
            bundle_dir: PathBuf::from("/tmp/release-bundle"),
            require_reproducible: false,
            require_stagex_no_quorum: false,
            deterministic_proof: None,
            deterministic_sandbox_isolation_evidence: None,
            require_deterministic_release: false,
            provider_fixed_point_proof: None,
            require_external_evidence_role: Vec::new(),
            require_provider_fixed_point_proof: false,
            release_profile: crunch_release_core::RELEASE_PROFILE_ONIX_STACK.to_string(),
            stack_provenance_mode: crunch_release_core::STACK_PROVENANCE_MODE_OPTIONAL.to_string(),
            requirement_coverage_mode: crunch_release_core::CONTENT_BOUND_REQUIREMENT_MODE_OPTIONAL.to_string(),
        };
        let deterministic_result = DeterministicReleaseVerifyResult {
            status: "eligible",
            eligible: true,
            proof_path: Some(PathBuf::from("strict-proof.json")),
            proof_digest_blake3: Some("a".repeat(crunch_release_core::BLAKE3_HEX_LENGTH_CHARS)),
            isolation_evidence_path: Some(PathBuf::from("isolation.json")),
            isolation_evidence_digest_blake3: Some("b".repeat(crunch_release_core::BLAKE3_HEX_LENGTH_CHARS)),
            proof_source: Some("bundled"),
            blockers: Vec::new(),
        };
        let provider_result = crate::cargo_free_self_build::ProviderFixedPointProofVerification::absent(false);
        let stack_result = satisfied_stack_provenance_result();
        let content_bound_result = crunch_release_core::evaluate_content_bound_requirement_release_evidence(
            &manifest,
            crunch_release_core::CONTENT_BOUND_REQUIREMENT_MODE_OPTIONAL,
        );
        let cairn_result = satisfied_cairn_handoff_result();
        let decision = aggregate_release_verify_decision(ReleaseVerifyDecisionInput {
            request: &request,
            manifest: &manifest,
            reproducibility_status: ReproducibilityStatus::Absent,
            deterministic_result: &deterministic_result,
            provider_fixed_point_result: &provider_result,
            effective_stack_mode: crunch_release_core::STACK_PROVENANCE_MODE_REQUIRED,
            stack_provenance_result: &stack_result,
            content_bound_requirement_result: &content_bound_result,
            cairn_handoff_result: &cairn_result,
            stagex_result: None,
        });

        assert!(decision.valid);
        assert!(decision.diagnostics.is_empty());
    }

    fn satisfied_stack_provenance_result() -> crunch_release_core::StackProvenanceReleaseVerification {
        crunch_release_core::StackProvenanceReleaseVerification {
            mode: crunch_release_core::STACK_PROVENANCE_MODE_REQUIRED.to_string(),
            required: true,
            valid: true,
            disposition: crunch_release_core::STACK_PROVENANCE_DISPOSITION_PRESENT.to_string(),
            sidecar_role: None,
            sidecar_schema: None,
            sidecar_claim_scope: None,
            sidecar_digest_blake3: None,
            valence_receipt_role: None,
            valence_receipt_schema: None,
            valence_receipt_digest_blake3: None,
            release_binary_relative_path: None,
            release_binary_digest_blake3: None,
            boundary: crunch_release_core::STACK_PROVENANCE_OPAQUE_BOUNDARY.to_string(),
            diagnostics: Vec::new(),
        }
    }

    fn satisfied_cairn_handoff_result() -> crunch_release_core::CairnHandoffReleaseVerification {
        crunch_release_core::CairnHandoffReleaseVerification {
            required: true,
            valid: true,
            disposition: crunch_release_core::CAIRN_HANDOFF_DISPOSITION_PRESENT.to_string(),
            bundle_binding_blake3: None,
            receipt_blake3: None,
            authentication_status: Some(crunch_release_core::CAIRN_HANDOFF_AUTHENTICATION_STATUS.to_string()),
            boundary: crunch_release_core::CAIRN_HANDOFF_BOUNDARY.to_string(),
            diagnostics: Vec::new(),
        }
    }

    fn write_summary_with_provider_mode(proof_dir: &Path, provider_mode: &str) {
        let summary = format!(
            "self-build-proof: provider-mode={provider_mode}\n\
             self-build-proof: hermeticity-mode=practical\n\
             self-build-proof: invoking-binary=/tmp/crunch\n\
             self-build-proof: staged-source=/tmp/src\n\
             self-build-proof: bwrap-source=bootstrap\n\
             self-build-proof: fallback-event=none\n\
             self-build-proof: protected-transition=none\n\
             self-build-proof: protected-seccomp-event=none\n\
             self-build-proof: busybox-path=/tmp/busybox\n\
             self-build-proof: output-binary=/tmp/out\n\
             self-build-proof: stagex-metadata=none\n"
        );
        std::fs::write(proof_dir.join("summary.txt"), summary.as_bytes()).unwrap();
    }

    #[test]
    fn stagex_profile_rejects_missing_reproducibility() {
        let temp = tempfile::tempdir().unwrap();
        let bundle_dir = temp.path();
        let proof_dir = bundle_dir.join("proof/self-hosting");
        std::fs::create_dir_all(&proof_dir).unwrap();
        write_summary_with_provider_mode(&proof_dir, "legacy-fetch");
        let manifest = test_manifest();
        let result = evaluate_stagex_profile(&manifest, bundle_dir, None);
        assert!(!result.status.is_satisfied());
        assert!(result.failure_reasons.iter().any(|r| r.contains("reproducibility")));
    }

    #[test]
    fn stagex_profile_rejects_legacy_fetch_provider() {
        let temp = tempfile::tempdir().unwrap();
        let bundle_dir = temp.path();
        let proof_dir = bundle_dir.join("proof/self-hosting");
        std::fs::create_dir_all(&proof_dir).unwrap();
        write_summary_with_provider_mode(&proof_dir, "legacy-fetch");
        let manifest = test_manifest();
        let result = evaluate_stagex_profile(&manifest, bundle_dir, None);
        assert!(!result.status.is_satisfied());
        assert!(
            result.failure_reasons.iter().any(|r| r.contains("no StageX-class lineage proof")),
            "legacy-fetch must not produce a proof block: {:?}",
            result.failure_reasons
        );
    }

    #[test]
    fn stagex_profile_rejects_source_root_provider() {
        let temp = tempfile::tempdir().unwrap();
        let bundle_dir = temp.path();
        let proof_dir = bundle_dir.join("proof/self-hosting");
        std::fs::create_dir_all(&proof_dir).unwrap();
        write_summary_with_provider_mode(&proof_dir, "source-root");
        let manifest = test_manifest();
        let result = evaluate_stagex_profile(&manifest, bundle_dir, None);
        assert!(!result.status.is_satisfied());
        assert!(
            result.failure_reasons.iter().any(|r| r.contains("no StageX-class lineage proof")),
            "source-root must not produce a proof block: {:?}",
            result.failure_reasons
        );
    }

    #[test]
    fn stagex_profile_rejects_self_proof_only_evidence() {
        let temp = tempfile::tempdir().unwrap();
        let bundle_dir = temp.path();
        let proof_dir = bundle_dir.join("proof/self-hosting");
        std::fs::create_dir_all(&proof_dir).unwrap();
        write_summary_with_provider_mode(&proof_dir, "nix-packages");
        let manifest = test_manifest();
        let result = evaluate_stagex_profile(&manifest, bundle_dir, None);
        assert!(!result.status.is_satisfied());
        assert!(
            result.failure_reasons.iter().any(|r| r.contains("no StageX-class lineage proof")),
            "nix-packages must not produce a proof block: {:?}",
            result.failure_reasons
        );
    }

    #[test]
    fn stagex_profile_rejects_prerequisite_only_proof() {
        let temp = tempfile::tempdir().unwrap();
        let bundle_dir = temp.path();
        let proof_dir = bundle_dir.join("proof/self-hosting");
        std::fs::create_dir_all(&proof_dir).unwrap();
        // No summary.txt at all -> no proof block
        let manifest = test_manifest();
        let result = evaluate_stagex_profile(&manifest, bundle_dir, None);
        assert!(!result.status.is_satisfied());
        assert!(result.failure_reasons.iter().any(|r| r.contains("no StageX-class lineage proof")));
    }

    #[test]
    fn stagex_profile_rejects_witness_agreement_without_lineage_proof() {
        let temp = tempfile::tempdir().unwrap();
        let bundle_dir = temp.path();
        let proof_dir = bundle_dir.join("proof/self-hosting");
        std::fs::create_dir_all(&proof_dir).unwrap();
        write_summary_with_provider_mode(&proof_dir, "legacy-fetch");
        let manifest = test_manifest();
        let fake_repro = VerifiedReproducibilityReport {
            status: ReproducibilityStatus::Matched,
            path: temp.path().join("repro.json"),
            digest_blake3: "9".repeat(64),
        };
        let result = evaluate_stagex_profile(&manifest, bundle_dir, Some(&fake_repro));
        assert!(!result.status.is_satisfied());
        assert!(
            result.failure_reasons.iter().any(|r| r.contains("no StageX-class lineage proof")),
            "witness agreement without lineage proof must fail: {:?}",
            result.failure_reasons
        );
    }
}
