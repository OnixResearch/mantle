use std::fs;
use std::io::Write as _;
use std::path::Path;
use std::path::PathBuf;

use crunch_release_core::BuildEffect;
use crunch_release_core::ContentBoundRebuildDescriptor;
use crunch_release_core::DeterministicBuildProofReceipt;
use crunch_release_core::DeterministicBuildProofReceiptInit;
use crunch_release_core::DeterministicBuildProofVerdict;
use crunch_release_core::DeterministicBuildRunReceipt;
use crunch_release_core::DeterministicOutputDigest;
use crunch_release_core::DeterministicProofUnit;
use crunch_release_core::RebuildAuthorityPlan;
use crunch_release_core::RebuildContentIdentity;
use crunch_release_core::RebuildContentKind;
use crunch_release_core::RebuildInputRole;
use crunch_release_core::RebuildPolicyIdentities;
use crunch_release_core::RebuildRunRootIdentity;
use crunch_release_core::content_bound_rebuild_descriptor_digest_blake3;
use crunch_release_core::deterministic_build_proof_receipt_digest_blake3;
use crunch_release_core::rebuild_arguments_digest_blake3;
use crunch_release_core::rebuild_authority_plan_digest_blake3;
use serde::Serialize;

use crate::errors::RunError;
use crate::source_built_fixed_point::ProofOutputRole;
use crate::source_built_fixed_point::SourceBuiltFixedPointPlan;
use crate::source_built_fixed_point_trust_report::ActionEvidence;
use crate::source_built_fixed_point_trust_report::BLAKE3_HEX_LENGTH;
use crate::source_built_fixed_point_trust_report::ColdArtifactBinding;
use crate::source_built_fixed_point_trust_report::ColdProviderEvidence;
use crate::source_built_fixed_point_trust_report::EXTENSION_FIELD;
use crate::source_built_fixed_point_trust_report::EXTENSION_SCHEMA;
use crate::source_built_fixed_point_trust_report::FINAL_BUNDLE_DIGEST_FILE;
use crate::source_built_fixed_point_trust_report::FINAL_RECEIPT_FILE;
use crate::source_built_fixed_point_trust_report::FixedPointObservation;
use crate::source_built_fixed_point_trust_report::FixedPointStageObservation;
use crate::source_built_fixed_point_trust_report::HERMETICITY_MODE;
use crate::source_built_fixed_point_trust_report::MANTLE_OUTPUT_NAME;
use crate::source_built_fixed_point_trust_report::NON_CLAIMS;
use crate::source_built_fixed_point_trust_report::PHYSICAL_STORE_ISOLATION;
use crate::source_built_fixed_point_trust_report::PROOF_PLAN_FILE;
use crate::source_built_fixed_point_trust_report::PROOF_PROVIDER_KIND;
use crate::source_built_fixed_point_trust_report::PROOF_TARGET_IDENTITY;
use crate::source_built_fixed_point_trust_report::PROOF_WORKFLOW;
use crate::source_built_fixed_point_trust_report::ProviderIdentityBinding;
use crate::source_built_fixed_point_trust_report::REQUIRED_RUN_COUNT;
use crate::source_built_fixed_point_trust_report::RecipeOutputBinding;
use crate::source_built_fixed_point_trust_report::STAGE_EVIDENCE_COUNT;
use crate::source_built_fixed_point_trust_report::STAGE_EVIDENCE_FILE;
use crate::source_built_fixed_point_trust_report::STAGE_STATUS_COMPLETE;
use crate::source_built_fixed_point_trust_report::STAGE1_RUN_ID;
use crate::source_built_fixed_point_trust_report::STAGE2_RUN_ID;
use crate::source_built_fixed_point_trust_report::SourceBuiltReceiptExtension;
use crate::source_built_fixed_point_trust_report::SourceBuiltStageEvidence;
use crate::source_built_fixed_point_trust_report::evidence_path;
use crate::source_built_fixed_point_trust_report::hash_file;
use crate::source_built_fixed_point_trust_report::load_action_evidence;
use crate::source_built_fixed_point_trust_report::measure_artifact;
use crate::source_built_fixed_point_trust_report::observe_fixed_point;
use crate::source_built_fixed_point_trust_report::proof_bundle_digest;
use crate::source_built_fixed_point_trust_report::provider_identity_digest;
use crate::source_built_fixed_point_trust_report::receipt_arguments;
use crate::source_built_fixed_point_trust_report::receipt_error;
use crate::source_built_fixed_point_trust_report::recipe_output_path;
use crate::source_built_fixed_point_trust_report::reject_development_authority;
use crate::source_built_fixed_point_trust_report::relative_path;
use crate::source_built_fixed_point_trust_report::validate_stage_evidence;
use crate::source_built_fixed_point_trust_report::validate_stage_plan_alignment;
use crate::source_built_fixed_point_trust_report::verify_cold_provider;
use crate::source_built_fixed_point_trust_report::verify_cold_stage_artifacts;
use crate::source_built_fixed_point_trust_report::verify_source_built_fixed_point_receipt;

pub(crate) const PROVIDER_LINKAGE_SCHEMA: &str = "mantle-self-build-provider-kind-linkage-v1";
pub(crate) const RELEASE_INPUTS_SCHEMA: &str = "mantle-source-built-fixed-point-release-inputs-v1";
pub(crate) const STAGE1_PERTURBATION: &str = "normalized-host-control-a";
pub(crate) const STAGE2_PERTURBATION: &str = "normalized-host-control-b";
pub(crate) const STAGE1_OUTPUT_ROOT: &str = "proof-output:stage1";
pub(crate) const STAGE2_OUTPUT_ROOT: &str = "proof-output:stage2";
pub(crate) const STAGE1_STORE_ROOT: &str = "proof-store:stage1-clean-namespace";
pub(crate) const STAGE2_STORE_ROOT: &str = "proof-store:stage2-clean-namespace";
pub(crate) const STAGE1_SANDBOX: &str = "mantle-proof-sandbox-v1:source-built-stage1";
pub(crate) const STAGE2_SANDBOX: &str = "mantle-proof-sandbox-v1:source-built-stage2";
pub(crate) const EFFECT_POLICY_VERSION: &str = "mantle-build-effects-v1";
pub(crate) const REBUILD_DESCRIPTOR_SCHEMA: &str = "mantle-content-bound-rebuild-descriptor-v1";
pub(crate) const REBUILD_AUTHORITY_SCHEMA: &str = "mantle-rebuild-authority-plan-v1";
pub(crate) const APPROVED_READ_DIGEST_DOMAIN: &[u8] = b"mantle-source-built-fixed-point-approved-reads-v1\0";
pub(crate) const REQUIRED_SOURCE_COUNT: usize = 7;
pub(crate) const REQUIRED_PERTURBATIONS: &[&str] = &[
    "HOME",
    "PATH",
    "USER",
    "LOGNAME",
    "TZ",
    "LANG",
    "LC_ALL",
    "TMPDIR",
    "cwd",
    "umask",
    "env-noise",
];
pub(crate) const NORMALIZATION_ENVELOPE: &[&str] = &[
    "normalization:time",
    "normalization:timezone=UTC",
    "normalization:locale=C",
    "normalization:temp-roots=proof-local",
    "normalization:host-user-metadata=mantle-proof",
    "normalization:umask=0022",
    "normalization:modeled-randomness=none",
    "normalization:order-sensitive-output-processing=canonical-sort",
];

#[derive(Debug, Serialize)]
struct ProviderKindLinkage<'a> {
    schema: &'static str,
    proof_identity: ProviderKindValue<'a>,
    proof_linkage: ProviderKindValue<'a>,
    prerequisites: ProviderKindValue<'a>,
    source_built_plan_digest_blake3: &'a str,
}

#[derive(Debug, Serialize)]
struct ProviderKindValue<'a> {
    selected_provider_kind: &'a str,
}

#[derive(Debug, Serialize)]
struct ReleaseEvidenceInputs<'a> {
    schema: &'static str,
    deterministic_proof: &'a str,
    provider_kind_linkage: &'a str,
    source_built_plan: &'a str,
    toolchain_closure: &'a str,
    proof_bundle_digest: &'a str,
}

/// Producer-only observations of immutable prepared recipe bindings after execution.
/// These paths and identities select evidence; they never authorize execution.
pub(crate) struct ColdStageObservation {
    pub(crate) stage_id: String,
    pub(crate) output_path: PathBuf,
    pub(crate) producer_action_id: String,
    pub(crate) output_id: String,
    pub(crate) execution_journal_path: PathBuf,
    pub(crate) native_publication_audit_path: Option<PathBuf>,
}

pub(crate) struct ColdProviderObservation {
    pub(crate) stages: Vec<ColdStageObservation>,
    pub(crate) native_provider_path: PathBuf,
    pub(crate) rust_provider_path: PathBuf,
    pub(crate) rustc_path: PathBuf,
    pub(crate) toolchain_closure_path: PathBuf,
}

pub(crate) fn write_cold_source_built_fixed_point_receipt(
    proof_root: &Path,
    plan: &SourceBuiltFixedPointPlan,
    providers: &ColdProviderObservation,
) -> Result<(), RunError> {
    reject_development_authority(proof_root)?;
    let actions = load_action_evidence(proof_root, plan)?;
    let fixed_point = observe_fixed_point(proof_root, plan, &actions)?;
    let closure_digest =
        measure_artifact(proof_root, &relative_path(proof_root, &providers.toolchain_closure_path)?)?.1;
    let stages = cold_stage_evidence(proof_root, plan, providers, &fixed_point, &closure_digest, &actions)?;
    let rust_journal = &stages
        .iter()
        .find(|stage| stage.stage_id == "full-source-rust-provider")
        .ok_or_else(|| receipt_error("missing actual Rust stage journal".into()))?
        .execution_journal_path;
    let cold_provider = ColdProviderEvidence {
        native_provider_path: relative_path(proof_root, &providers.native_provider_path)?,
        rust_provider_path: relative_path(proof_root, &providers.rust_provider_path)?,
        rustc: measured_binding(
            proof_root,
            &providers.rustc_path,
            "full-source-rust-provider",
            rust_journal,
            &actions,
        )?,
        toolchain_closure: measured_binding(
            proof_root,
            &providers.toolchain_closure_path,
            "full-source-rust-provider",
            rust_journal,
            &actions,
        )?,
    };
    let measured = verify_cold_provider(proof_root, plan, &stages, &cold_provider, &actions)?;
    write_json_create_new(&proof_root.join(STAGE_EVIDENCE_FILE), &stages)?;
    let stage_evidence_digest_blake3 = hash_file(&proof_root.join(STAGE_EVIDENCE_FILE))?;
    write_provider_kind_linkage(proof_root, plan)?;
    write_release_inputs(proof_root, &cold_provider.toolchain_closure.path)?;
    let bundle = proof_bundle_digest(proof_root)?;
    write_text_create_new(&proof_root.join(FINAL_BUNDLE_DIGEST_FILE), &format!("{}  .\n", bundle.digest_blake3))?;
    let identity = provider_identity_digest(&ProviderIdentityBinding {
        native_digest_blake3: &measured.native_digest,
        rust_digest_blake3: &measured.rust_digest,
        closure_digest_blake3: &measured.closure_digest,
    });
    let extension = SourceBuiltReceiptExtension {
        schema: EXTENSION_SCHEMA.into(),
        plan_digest_blake3: plan.plan_digest_blake3.clone(),
        source_authority_digest_blake3: plan.source_authority_digest_blake3.clone(),
        provider_kind: PROOF_PROVIDER_KIND.into(),
        native_provider_digest_blake3: measured.native_digest,
        rust_provider_digest_blake3: measured.rust_digest,
        toolchain_closure_digest_blake3: measured.closure_digest,
        cold_provider,
        stage_evidence_path: STAGE_EVIDENCE_FILE.into(),
        stage_evidence_digest_blake3,
        final_proof_bundle_digest_blake3: bundle.digest_blake3,
        protected_execution_policy_digest_blake3: plan.policies.protected_execution_policy_digest_blake3.clone(),
        effect_policy_digest_blake3: plan.policies.effect_policy_digest_blake3.clone(),
        normalization_policy_digest_blake3: plan.policies.normalization_policy_digest_blake3.clone(),
        action_evidence: actions.binding,
        non_claims: NON_CLAIMS.iter().map(|value| (*value).to_string()).collect(),
    };
    let (descriptor, descriptor_digest, authority_plan, authority_plan_digest) = rebuild_authority(
        proof_root,
        plan,
        &fixed_point,
        &providers.rustc_path,
        measured.provider_size_bytes,
        &extension.rust_provider_digest_blake3,
        &identity,
        &extension,
    )?;
    let core = deterministic_receipt(
        plan,
        &fixed_point,
        &identity,
        descriptor,
        descriptor_digest,
        authority_plan,
        authority_plan_digest,
    )?;
    let receipt_path = proof_root.join(FINAL_RECEIPT_FILE);
    write_extended_receipt(&receipt_path, &core, &extension)?;
    verify_source_built_fixed_point_receipt(proof_root, &receipt_path).map(|_| ())
}

fn measured_binding(
    root: &Path,
    path: &Path,
    stage_id: &str,
    journal: &str,
    actions: &ActionEvidence,
) -> Result<ColdArtifactBinding, RunError> {
    let relative = relative_path(root, path)?;
    let digest = measure_artifact(root, &relative)?.1;
    let adapter = actions
        .plan
        .adapters
        .iter()
        .find(|adapter| adapter.stage_id == stage_id)
        .ok_or_else(|| receipt_error(format!("missing adapter {stage_id}")))?;
    for event in actions
        .events
        .iter()
        .filter(|event| event.kind == crate::source_built_fixed_point_action_trust::ExecutionEventKind::BuildExecution)
    {
        if let Some(action) =
            crate::source_built_fixed_point_action_trust::adapter_action_spec(adapter, &event.action_id)
                .map_err(receipt_error)?
        {
            for (output_id, observed) in &event.produced_outputs {
                if observed == &digest
                    && recipe_output_path(&RecipeOutputBinding {
                        journal_path: journal,
                        stage_id,
                        action_id: &action.id,
                        output_id,
                    })? == relative
                {
                    return Ok(ColdArtifactBinding {
                        path: relative,
                        producer_action_id: action.id.clone(),
                        output_id: output_id.clone(),
                    });
                }
            }
        }
    }
    Err(receipt_error(format!("actual artifact {} has no reconciled {stage_id} producer", path.display())))
}

fn cold_stage_evidence(
    root: &Path,
    plan: &SourceBuiltFixedPointPlan,
    providers: &ColdProviderObservation,
    fixed_point: &FixedPointObservation,
    closure_digest: &str,
    actions: &ActionEvidence,
) -> Result<Vec<SourceBuiltStageEvidence>, RunError> {
    if providers.stages.len() != plan.stages.len() {
        return Err(receipt_error("cold observations omit or add stages".into()));
    }
    let mut stages = Vec::with_capacity(STAGE_EVIDENCE_COUNT);
    let closure_path = relative_path(root, &providers.toolchain_closure_path)?;
    for (planned, observed) in plan.stages.iter().zip(&providers.stages) {
        if observed.stage_id != planned.stage_id {
            return Err(receipt_error("cold observation stage order differs from proof".into()));
        }
        let journal = relative_path(root, &observed.execution_journal_path)?;
        let output_path = relative_path(root, &observed.output_path)?;
        let event = actions
            .events
            .iter()
            .find(|event| {
                event.action_id == observed.producer_action_id
                    && event.kind == crate::source_built_fixed_point_action_trust::ExecutionEventKind::BuildExecution
            })
            .ok_or_else(|| receipt_error("cold output lacks an actual reconciled producer completion".into()))?;
        let mut stage = match planned.output {
            ProofOutputRole::MantleStage1 => mantle_stage_evidence(
                "mantle-stage1",
                planned.output,
                "host-mantle",
                &fixed_point.stage1,
                &closure_path,
                closure_digest,
            ),
            ProofOutputRole::MantleStage2 => mantle_stage_evidence(
                "mantle-stage2",
                planned.output,
                "stage1-mantle",
                &fixed_point.stage2,
                &closure_path,
                closure_digest,
            ),
            _ => SourceBuiltStageEvidence {
                stage_id: observed.stage_id.clone(),
                output_role: planned.output,
                status: STAGE_STATUS_COMPLETE.into(),
                orchestrator: "host-mantle".into(),
                executable_identity: format!("blake3:{}", event.executable_digest_blake3),
                transcript_path: journal.clone(),
                transcript_digest_blake3: hash_file(&evidence_path(root, &journal)?)?,
                audit_paths: Vec::new(),
                audit_digests_blake3: Vec::new(),
                output_path: output_path.clone(),
                output_digest_blake3: measure_artifact(root, &output_path)?.1,
                producer_action_id: String::new(),
                output_id: String::new(),
                execution_journal_path: String::new(),
                execution_journal_digest_blake3: String::new(),
                authority_violations: Vec::new(),
                fallback_events: Vec::new(),
            },
        };
        if stage.output_path != output_path {
            return Err(receipt_error("cold Mantle output differs from actual manifest".into()));
        }
        stage.producer_action_id = observed.producer_action_id.clone();
        stage.output_id = observed.output_id.clone();
        stage.execution_journal_path = journal;
        stage.execution_journal_digest_blake3 = hash_file(&observed.execution_journal_path)?;
        if let Some(audit) = &observed.native_publication_audit_path {
            if planned.output != ProofOutputRole::FullSourceNativeProvider {
                return Err(receipt_error("native publication audit attached to a different stage".into()));
            }
            stage.audit_paths.push(relative_path(root, audit)?);
            stage.audit_digests_blake3.push(hash_file(audit)?);
        }
        stages.push(stage);
    }
    validate_stage_evidence(&stages)?;
    validate_stage_plan_alignment(plan, &stages)?;
    verify_cold_stage_artifacts(root, plan, &stages, actions)?;
    Ok(stages)
}

fn mantle_stage_evidence(
    stage_id: &str,
    role: ProofOutputRole,
    orchestrator: &str,
    observed: &FixedPointStageObservation,
    closure_path: &str,
    closure_digest: &str,
) -> SourceBuiltStageEvidence {
    let audit_count = observed.audit_files.len().saturating_add(1);
    let mut audit_paths = Vec::with_capacity(audit_count);
    let mut audit_digests_blake3 = Vec::with_capacity(audit_count);
    for (path, digest) in &observed.audit_files {
        audit_paths.push(path.clone());
        audit_digests_blake3.push(digest.clone());
    }
    audit_paths.push(closure_path.into());
    audit_digests_blake3.push(closure_digest.into());
    SourceBuiltStageEvidence {
        stage_id: stage_id.into(),
        output_role: role,
        status: STAGE_STATUS_COMPLETE.into(),
        orchestrator: orchestrator.into(),
        executable_identity: format!("blake3:{}", observed.orchestrator_digest_blake3),
        transcript_path: observed.manifest_relative_path.clone(),
        transcript_digest_blake3: observed.manifest_digest_blake3.clone(),
        audit_paths,
        audit_digests_blake3,
        output_path: observed.binary_relative_path.clone(),
        output_digest_blake3: observed.binary_digest_blake3.clone(),
        producer_action_id: String::new(),
        output_id: String::new(),
        execution_journal_path: String::new(),
        execution_journal_digest_blake3: String::new(),
        authority_violations: Vec::new(),
        fallback_events: Vec::new(),
    }
}

fn rebuild_authority(
    proof_root: &Path,
    plan: &SourceBuiltFixedPointPlan,
    fixed_point: &FixedPointObservation,
    rustc_path: &Path,
    provider_size_bytes: u64,
    rust_provider_digest_blake3: &str,
    provider_identity_digest_blake3: &str,
    extension: &SourceBuiltReceiptExtension,
) -> Result<(ContentBoundRebuildDescriptor, String, RebuildAuthorityPlan, String), RunError> {
    let arguments = receipt_arguments(plan, &fixed_point.closure_policy_digest_blake3, extension)?;
    let stage1_binary_path = crate::source_built_fixed_point_trust_report::evidence_path(
        proof_root,
        &fixed_point.stage1.binary_relative_path,
    )?;
    let stage1_binary_size_bytes = file_size(&stage1_binary_path)?;
    let arguments_blake3 = rebuild_arguments_digest_blake3(arguments.clone())
        .map_err(|error| receipt_error(format!("digesting rebuild arguments: {error}")))?;
    let source_inputs = plan.source_inputs.iter().map(rebuild_source_identity).collect::<Vec<_>>();
    if source_inputs.len() != REQUIRED_SOURCE_COUNT {
        return Err(receipt_error("rebuild descriptor source input count is incomplete".to_string()));
    }
    let target = RebuildContentIdentity {
        name: MANTLE_OUTPUT_NAME.to_string(),
        role: RebuildInputRole::PublishedTarget,
        kind: RebuildContentKind::RegularFile,
        digest_blake3: fixed_point.stage1.binary_digest_blake3.clone(),
        size_bytes: stage1_binary_size_bytes,
    };
    let recipe = RebuildContentIdentity {
        name: PROOF_PLAN_FILE.to_string(),
        role: RebuildInputRole::Recipe,
        kind: RebuildContentKind::RegularFile,
        digest_blake3: plan.plan_digest_blake3.clone(),
        size_bytes: 1,
    };
    let executable = RebuildContentIdentity {
        name: "source-built-rustc".to_string(),
        role: RebuildInputRole::Executable,
        kind: RebuildContentKind::RegularFile,
        digest_blake3: hash_file(rustc_path)?,
        size_bytes: file_size(rustc_path)?,
    };
    let stage1_orchestrator = RebuildContentIdentity {
        name: "stage1-mantle-orchestrator".to_string(),
        role: RebuildInputRole::Tool,
        kind: RebuildContentKind::RegularFile,
        digest_blake3: fixed_point.stage1.binary_digest_blake3.clone(),
        size_bytes: stage1_binary_size_bytes,
    };
    let provider = RebuildContentIdentity {
        name: "source-built-native-and-rust-provider-closure".to_string(),
        role: RebuildInputRole::Provider,
        kind: RebuildContentKind::Directory,
        digest_blake3: provider_identity_digest_blake3.to_string(),
        size_bytes: provider_size_bytes,
    };
    let run_roots = vec![
        RebuildRunRootIdentity {
            run_id: STAGE1_RUN_ID.to_string(),
            output_root_identity: STAGE1_OUTPUT_ROOT.to_string(),
            store_root_identity: STAGE1_STORE_ROOT.to_string(),
        },
        RebuildRunRootIdentity {
            run_id: STAGE2_RUN_ID.to_string(),
            output_root_identity: STAGE2_OUTPUT_ROOT.to_string(),
            store_root_identity: STAGE2_STORE_ROOT.to_string(),
        },
    ];
    let descriptor = ContentBoundRebuildDescriptor {
        schema: REBUILD_DESCRIPTOR_SCHEMA.to_string(),
        target_artifacts: vec![target],
        recipe,
        executable,
        tools: vec![stage1_orchestrator],
        ordered_arguments: arguments,
        arguments_blake3,
        source_inputs,
        provider,
        policies: RebuildPolicyIdentities {
            sandbox_policy_blake3: plan.policies.protected_execution_policy_digest_blake3.clone(),
            effect_policy_blake3: plan.policies.effect_policy_digest_blake3.clone(),
            normalization_policy_blake3: plan.policies.normalization_policy_digest_blake3.clone(),
        },
        run_roots,
    };
    let descriptor_digest = content_bound_rebuild_descriptor_digest_blake3(descriptor.clone())
        .map_err(|error| receipt_error(format!("validating rebuild descriptor: {error}")))?;
    let approved_read_identities = approved_read_identities(plan, &descriptor, rust_provider_digest_blake3);
    let approved_read_paths_blake3 = digest_string_set(APPROVED_READ_DIGEST_DOMAIN, &approved_read_identities);
    let authority_plan = RebuildAuthorityPlan {
        schema: REBUILD_AUTHORITY_SCHEMA.to_string(),
        descriptor_blake3: descriptor_digest.clone(),
        approved_read_identities,
        approved_read_paths_blake3,
        fresh_write_root_identities: vec![STAGE1_OUTPUT_ROOT.to_string(), STAGE2_OUTPUT_ROOT.to_string()],
        target_authority_excluded: true,
        blockers: Vec::new(),
    };
    let authority_plan_digest = rebuild_authority_plan_digest_blake3(authority_plan.clone())
        .map_err(|error| receipt_error(format!("validating rebuild authority plan: {error}")))?;
    assert!(authority_plan.target_authority_excluded);
    debug_assert!(authority_plan.blockers.is_empty());
    Ok((descriptor, descriptor_digest, authority_plan, authority_plan_digest))
}

fn deterministic_receipt(
    plan: &SourceBuiltFixedPointPlan,
    fixed_point: &FixedPointObservation,
    provider_identity_digest_blake3: &str,
    descriptor: ContentBoundRebuildDescriptor,
    descriptor_digest: String,
    authority_plan: RebuildAuthorityPlan,
    authority_plan_digest: String,
) -> Result<DeterministicBuildProofReceipt, RunError> {
    let approved_reads = authority_plan.approved_read_identities.clone();
    let output_digest = fixed_point.stage1.binary_digest_blake3.clone();
    let runs = vec![
        deterministic_run(
            STAGE1_RUN_ID,
            STAGE1_PERTURBATION,
            &format!("proof://{}", fixed_point.stage1.binary_relative_path),
            STAGE1_OUTPUT_ROOT,
            STAGE1_SANDBOX,
            &output_digest,
            &descriptor_digest,
            &authority_plan_digest,
            &approved_reads,
        ),
        deterministic_run(
            STAGE2_RUN_ID,
            STAGE2_PERTURBATION,
            &format!("proof://{}", fixed_point.stage2.binary_relative_path),
            STAGE2_OUTPUT_ROOT,
            STAGE2_SANDBOX,
            &output_digest,
            &descriptor_digest,
            &authority_plan_digest,
            &approved_reads,
        ),
    ];
    let toolchain_provider_identity = format!(
        "provider-kind={PROOF_PROVIDER_KIND};identity-blake3={provider_identity_digest_blake3};closure-policy-blake3={}",
        fixed_point.closure_policy_digest_blake3
    );
    let init = DeterministicBuildProofReceiptInit {
        proof_unit: DeterministicProofUnit {
            target_artifact_identity: PROOF_TARGET_IDENTITY.to_string(),
            output_identities: vec![format!("{MANTLE_OUTPUT_NAME}:{output_digest}")],
        },
        derivation_identity: format!("source-built-plan:{}", plan.plan_digest_blake3),
        hermeticity_mode: HERMETICITY_MODE.to_string(),
        workflow_version: PROOF_WORKFLOW.to_string(),
        selected_provider_kind: PROOF_PROVIDER_KIND.to_string(),
        source_blake3: plan.source_authority_digest_blake3.clone(),
        vendor_blake3: plan.receipt_contract.vendor_blake3.clone(),
        toolchain_provider_identity,
        toolchain_stage_roots: vec![
            format!("stagex-provider:{}", plan.policies.protected_execution_policy_digest_blake3),
            format!("native-provider:{}", plan.policies.expected_native_provider_digest_blake3),
            format!("closure-policy:{}", fixed_point.closure_policy_digest_blake3),
        ],
        logical_store_prefix: plan.logical_store_prefix.clone(),
        physical_store_isolation: PHYSICAL_STORE_ISOLATION.to_string(),
        effect_policy_version: EFFECT_POLICY_VERSION.to_string(),
        declared_effects: vec![
            BuildEffect::ReadStore,
            BuildEffect::WriteOutput,
            BuildEffect::Environment,
        ],
        observed_effects: Some(vec![
            BuildEffect::ReadStore,
            BuildEffect::WriteOutput,
            BuildEffect::Environment,
        ]),
        normalized_execution_envelope: NORMALIZATION_ENVELOPE.iter().map(|value| (*value).to_string()).collect(),
        ambient_host_perturbations: REQUIRED_PERTURBATIONS.iter().map(|value| (*value).to_string()).collect(),
        sandbox_profile_identities: vec![STAGE1_SANDBOX.to_string(), STAGE2_SANDBOX.to_string()],
        runs,
        rebuild_descriptor: descriptor,
        rebuild_descriptor_blake3: descriptor_digest,
        rebuild_authority_plan: authority_plan,
        rebuild_authority_plan_blake3: authority_plan_digest,
    };
    let receipt = DeterministicBuildProofReceipt::new(init);
    if receipt.verdict != DeterministicBuildProofVerdict::SelfRebuildMatch || !receipt.blocking_reasons.is_empty() {
        return Err(receipt_error(format!(
            "source-built deterministic receipt classification failed: {:?}: {}",
            receipt.verdict,
            receipt.blocking_reasons.join("; ")
        )));
    }
    let digest = deterministic_build_proof_receipt_digest_blake3(receipt.clone())
        .map_err(|error| receipt_error(format!("digesting deterministic receipt: {error}")))?;
    let mut receipt = receipt;
    receipt.receipt_blake3 = Some(digest);
    let canonical = crunch_release_core::canonical_deterministic_build_proof_receipt(receipt)
        .map_err(|error| receipt_error(format!("canonicalizing deterministic receipt: {error}")))?;
    assert_eq!(canonical.runs.len(), REQUIRED_RUN_COUNT);
    debug_assert_eq!(canonical.verdict, DeterministicBuildProofVerdict::SelfRebuildMatch);
    Ok(canonical)
}

fn deterministic_run(
    run_id: &str,
    perturbation_case: &str,
    output_store_path: &str,
    output_root_identity: &str,
    sandbox_profile_identity: &str,
    output_digest: &str,
    descriptor_digest: &str,
    authority_plan_digest: &str,
    approved_reads: &[String],
) -> DeterministicBuildRunReceipt {
    assert!(!approved_reads.is_empty());
    assert_eq!(output_digest.len(), BLAKE3_HEX_LENGTH);
    DeterministicBuildRunReceipt {
        run_id: run_id.to_string(),
        perturbation_case: perturbation_case.to_string(),
        output_store_paths: vec![output_store_path.to_string()],
        output_root_identity: output_root_identity.to_string(),
        sandbox_profile_identity: sandbox_profile_identity.to_string(),
        output_digests: vec![DeterministicOutputDigest {
            name: MANTLE_OUTPUT_NAME.to_string(),
            digest_blake3: output_digest.to_string(),
        }],
        substituted_dependency_identities: Vec::new(),
        hermeticity_audit_events: Vec::new(),
        observed_effects: Some(vec![
            BuildEffect::ReadStore,
            BuildEffect::WriteOutput,
            BuildEffect::Environment,
        ]),
        rebuild_descriptor_blake3: Some(descriptor_digest.to_string()),
        rebuild_authority_plan_blake3: Some(authority_plan_digest.to_string()),
        observed_read_identities: approved_reads.to_vec(),
        authority_violations: Vec::new(),
    }
}

fn rebuild_source_identity(input: &crate::source_built_fixed_point::SourceAuthorityInput) -> RebuildContentIdentity {
    let kind = match input.kind {
        crate::source_built_fixed_point::SourceContentKind::RegularFile => RebuildContentKind::RegularFile,
        crate::source_built_fixed_point::SourceContentKind::Directory => RebuildContentKind::Directory,
    };
    RebuildContentIdentity {
        name: format!("{:?}:{}", input.role, input.id),
        role: RebuildInputRole::Source,
        kind,
        digest_blake3: input.digest_blake3.clone(),
        size_bytes: input.size_bytes,
    }
}

fn approved_read_identities(
    plan: &SourceBuiltFixedPointPlan,
    descriptor: &ContentBoundRebuildDescriptor,
    rust_provider_digest_blake3: &str,
) -> Vec<String> {
    let mut reads = plan
        .source_inputs
        .iter()
        .map(|input| format!("source:{:?}:{}:{}", input.role, input.id, input.digest_blake3))
        .collect::<Vec<_>>();
    reads.extend([
        format!("recipe:{}", plan.plan_digest_blake3),
        format!("provider:{}", descriptor.provider.digest_blake3),
        format!("rust-provider:{rust_provider_digest_blake3}"),
        format!("policy:sandbox:{}", descriptor.policies.sandbox_policy_blake3),
        format!("policy:effect:{}", descriptor.policies.effect_policy_blake3),
        format!("policy:normalization:{}", descriptor.policies.normalization_policy_blake3),
        format!("stage1-orchestrator:{}", descriptor.tools[0].digest_blake3),
    ]);
    reads.sort();
    reads.dedup();
    assert!(reads.len() > REQUIRED_SOURCE_COUNT);
    debug_assert!(reads.windows(2).all(|pair| pair[0] < pair[1]));
    reads
}

fn write_provider_kind_linkage(proof_root: &Path, plan: &SourceBuiltFixedPointPlan) -> Result<(), RunError> {
    let linkage = ProviderKindLinkage {
        schema: PROVIDER_LINKAGE_SCHEMA,
        proof_identity: ProviderKindValue {
            selected_provider_kind: PROOF_PROVIDER_KIND,
        },
        proof_linkage: ProviderKindValue {
            selected_provider_kind: PROOF_PROVIDER_KIND,
        },
        prerequisites: ProviderKindValue {
            selected_provider_kind: PROOF_PROVIDER_KIND,
        },
        source_built_plan_digest_blake3: &plan.plan_digest_blake3,
    };
    write_json_create_new(&proof_root.join(crate::source_built_fixed_point_shell::PROVIDER_KIND_LINKAGE_FILE), &linkage)
}

fn write_release_inputs(proof_root: &Path, closure_path: &str) -> Result<(), RunError> {
    let inputs = ReleaseEvidenceInputs {
        schema: RELEASE_INPUTS_SCHEMA,
        deterministic_proof: FINAL_RECEIPT_FILE,
        provider_kind_linkage: crate::source_built_fixed_point_shell::PROVIDER_KIND_LINKAGE_FILE,
        source_built_plan: PROOF_PLAN_FILE,
        toolchain_closure: closure_path,
        proof_bundle_digest: FINAL_BUNDLE_DIGEST_FILE,
    };
    write_json_create_new(&proof_root.join(crate::source_built_fixed_point_shell::RELEASE_INPUTS_FILE), &inputs)
}

fn write_extended_receipt(
    path: &Path,
    core: &DeterministicBuildProofReceipt,
    extension: &SourceBuiltReceiptExtension,
) -> Result<(), RunError> {
    let mut value = serde_json::to_value(core)
        .map_err(|error| receipt_error(format!("serializing deterministic receipt: {error}")))?;
    let object = value
        .as_object_mut()
        .ok_or_else(|| receipt_error("deterministic receipt is not a JSON object".to_string()))?;
    object.insert(
        EXTENSION_FIELD.to_string(),
        serde_json::to_value(extension)
            .map_err(|error| receipt_error(format!("serializing source-built extension: {error}")))?,
    );
    write_json_create_new(path, &value)
}

fn digest_string_set(domain: &[u8], values: &[String]) -> String {
    let mut sorted = values.to_vec();
    sorted.sort();
    sorted.dedup();
    let mut hasher = blake3::Hasher::new();
    hasher.update(domain);
    for value in sorted {
        hasher.update(value.as_bytes());
        hasher.update(&[0]);
    }
    let digest = hasher.finalize().to_hex().to_string();
    assert_eq!(digest.len(), BLAKE3_HEX_LENGTH);
    debug_assert!(!values.is_empty());
    digest
}

fn file_size(path: &Path) -> Result<u64, RunError> {
    let size = fs::metadata(path)
        .map_err(|error| receipt_error(format!("reading file size {}: {error}", path.display())))?
        .len();
    if size == 0 {
        return Err(receipt_error(format!("required file is empty: {}", path.display())));
    }
    assert!(size > 0);
    debug_assert!(path.is_file());
    Ok(size)
}

fn write_json_create_new<T: Serialize>(path: &Path, value: &T) -> Result<(), RunError> {
    let mut bytes = serde_json::to_vec_pretty(value)
        .map_err(|error| receipt_error(format!("serializing {}: {error}", path.display())))?;
    bytes.push(b'\n');
    write_bytes_create_new(path, &bytes)
}

fn write_text_create_new(path: &Path, text: &str) -> Result<(), RunError> {
    write_bytes_create_new(path, text.as_bytes())
}

fn write_bytes_create_new(path: &Path, bytes: &[u8]) -> Result<(), RunError> {
    let mut file = fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(path)
        .map_err(|error| receipt_error(format!("creating {}: {error}", path.display())))?;
    file.write_all(bytes)
        .map_err(|error| receipt_error(format!("writing {}: {error}", path.display())))?;
    file.sync_all().map_err(|error| receipt_error(format!("syncing {}: {error}", path.display())))?;
    assert!(path.is_file());
    debug_assert_eq!(fs::metadata(path).ok().map(|metadata| metadata.len()), Some(bytes.len() as u64));
    Ok(())
}
