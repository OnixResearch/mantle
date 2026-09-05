use super::*;

const DEV_RESUME_CHECKPOINT_STORE_SUBDIR: &str = "dev-resume-checkpoints";
const DEV_RESUME_MANIFESTS_SUBDIR: &str = "dev-resume-manifests";
const DEV_RESUME_MANIFEST_TEMP_PREFIX: &str = ".dev-resume-manifest-tmp-";
const PROVIDER_CHECKPOINT_DESTINATION: &str = "provider-checkpoint";
const MANTLE_STAGE1_DESTINATION: &str = "fixed-point/stage1";
const FIXED_POINT_COMPLETE_DESTINATION: &str = "fixed-point";

pub(crate) fn checkpoint_store(cache: &Path) -> PathBuf {
    cache.join(DEV_RESUME_CHECKPOINT_STORE_SUBDIR)
}

const PUBLICATIONS_SUBDIR: &str = "dev-resume-publications";

// r[impl source_built_fixed_point_improved_iteration.dev_cross_run_resume]
pub(crate) fn publish_provider_prefix_manifest(
    options: &SourceBuiltFixedPointOptions<'_>,
    prepared: &PreparedAttempt,
    admitted: &crate::source_built_fixed_point_checkpoint_shell::AdmittedProviderCheckpoint,
) -> Result<(), RunError> {
    let index = admitted
        .manifest
        .stages
        .len()
        .checked_sub(1)
        .ok_or_else(|| proof_error("dev prefix has no completed stage".to_string()))?;
    let stage = *crunch_dev_resume_core::ResumeStage::ALL
        .get(index)
        .ok_or_else(|| proof_error("dev prefix stage count is invalid".to_string()))?;
    let record = &admitted.manifest.stages[index];
    let binding = crunch_dev_resume_core::ResumeStageBinding {
        stage,
        stage_id: stage.id().to_string(),
        producer_executable_digest_blake3: record.producer_executable_digest_blake3.clone(),
        output_digest_blake3: record.output_digest_blake3.clone(),
        execution_evidence_digest_blake3: record.execution_evidence_digest_blake3.clone(),
    };
    let manifest =
        sealed_manifest(prepared, &resume_policy_digest(&prepared.plan)?, binding, vec![provider_checkpoint_binding(
            admitted,
        )?])?;
    publish_and_record(options, prepared, &manifest)
}

pub(crate) struct FixedPointPublication<'a, 'b> {
    pub options: &'a SourceBuiltFixedPointOptions<'b>,
    pub prepared: &'a PreparedAttempt,
    pub resume_plan: &'a crunch_dev_resume_core::ResumePlan,
}

impl crate::cargo_free_self_build::FixedPointStage1Publisher for FixedPointPublication<'_, '_> {
    fn publish_stage1(&self) -> Result<(), RunError> {
        publish_fixed_point_boundary(self, crunch_dev_resume_core::ResumeStage::MantleStage1)
    }
}

pub(crate) fn publish_fixed_point_complete(context: &FixedPointPublication<'_, '_>) -> Result<(), RunError> {
    publish_fixed_point_boundary(context, crunch_dev_resume_core::ResumeStage::MantleStage2)
}

fn publish_fixed_point_boundary(
    context: &FixedPointPublication<'_, '_>,
    stage: crunch_dev_resume_core::ResumeStage,
) -> Result<(), RunError> {
    if !context.options.dev_resume {
        return Ok(());
    }
    let cache = context
        .options
        .dev_provider_cache
        .ok_or_else(|| proof_error("dev publication requires its cache".to_string()))?;
    let fixed_point_dir = context.prepared.staging_dir.join(FIXED_POINT_DIR);
    let (source, payload_id, destination, producer) = match stage {
        crunch_dev_resume_core::ResumeStage::MantleStage1 => (
            fixed_point_dir.join(crate::cargo_free_self_build::STAGE1_DIR),
            crunch_dev_resume_core::MANTLE_STAGE1_PAYLOAD_ID,
            MANTLE_STAGE1_DESTINATION,
            current_executable_digest()?,
        ),
        crunch_dev_resume_core::ResumeStage::MantleStage2 => (
            fixed_point_dir.clone(),
            crunch_dev_resume_core::FIXED_POINT_COMPLETE_PAYLOAD_ID,
            FIXED_POINT_COMPLETE_DESTINATION,
            fixed_point_binary_digest(&fixed_point_dir, crate::cargo_free_self_build::STAGE1_DIR)?,
        ),
        _ => return Err(proof_error("fixed-point publication requested for a provider stage".to_string())),
    };
    let payload =
        io::publish_tree_object(cache, &source, payload_id, destination, dev_resume_tree_limits(context.options))?;
    let provider = provider_payload_for_attempt(context)?;
    let binding = fixed_point_stage_binding(&fixed_point_dir, stage, &producer)?;
    let manifest = sealed_manifest(context.prepared, &resume_policy_digest(&context.prepared.plan)?, binding, vec![
        payload, provider,
    ])?;
    publish_and_record(context.options, context.prepared, &manifest)
}

fn provider_payload_for_attempt(
    context: &FixedPointPublication<'_, '_>,
) -> Result<crunch_dev_resume_core::ResumePayloadBinding, RunError> {
    let manifest = if context
        .resume_plan
        .completed_stage
        .is_some_and(|stage| stage >= crunch_dev_resume_core::ResumeStage::FullSourceRustProvider)
    {
        let identity = context
            .resume_plan
            .selected_bundle_identity_blake3
            .as_deref()
            .ok_or_else(|| proof_error("restored provider plan has no bundle identity".to_string()))?;
        let cache = context
            .options
            .dev_provider_cache
            .ok_or_else(|| proof_error("dev publication requires its cache".to_string()))?;
        candidates::load_selected_manifest(cache, &context.prepared.plan.plan_digest_blake3, identity)?
    } else {
        manifest_io::read_manifest(&publication_root(
            context.prepared,
            crunch_dev_resume_core::ResumeStage::FullSourceRustProvider,
        ))?
    };
    crunch_dev_resume_core::validate_resume_manifest(&manifest).map_err(|error| proof_error(error.to_string()))?;
    manifest
        .payloads
        .into_iter()
        .find(|payload| payload.payload_id == crunch_dev_resume_core::PROVIDER_CHECKPOINT_PAYLOAD_ID)
        .ok_or_else(|| proof_error("completed provider publication has no checkpoint reference".to_string()))
}

fn publish_and_record(
    options: &SourceBuiltFixedPointOptions<'_>,
    prepared: &PreparedAttempt,
    manifest: &crunch_dev_resume_core::ResumeBundleManifest,
) -> Result<(), RunError> {
    let cache = options
        .dev_provider_cache
        .ok_or_else(|| proof_error("dev publication requires its cache".to_string()))?;
    publish_manifest(cache, manifest)?;
    let root = publication_root(prepared, manifest.completed_stage.stage);
    if root.exists() {
        return validate_existing_manifest(&root, manifest);
    }
    fs::create_dir_all(&root).map_err(|error| proof_error(format!("creating dev publication record: {error}")))?;
    write_json_create_new(&root.join(manifest_io::DEV_RESUME_MANIFEST_FILE), manifest)
}

fn publication_root(prepared: &PreparedAttempt, stage: crunch_dev_resume_core::ResumeStage) -> PathBuf {
    prepared.staging_dir.join(PUBLICATIONS_SUBDIR).join(stage.id())
}

pub(crate) fn published_bundle_identities(prepared: &PreparedAttempt) -> Result<Vec<String>, RunError> {
    let mut identities = Vec::with_capacity(crunch_dev_resume_core::ResumeStage::ALL.len());
    for stage in crunch_dev_resume_core::ResumeStage::ALL {
        let root = publication_root(prepared, stage);
        if !root.exists() {
            continue;
        }
        let manifest = manifest_io::read_manifest(&root)?;
        crunch_dev_resume_core::validate_resume_manifest(&manifest).map_err(|error| proof_error(error.to_string()))?;
        if manifest.completed_stage.stage != stage || manifest.plan_digest_blake3 != prepared.plan.plan_digest_blake3 {
            return Err(proof_error("dev publication record does not match this attempt".to_string()));
        }
        identities.push(manifest.bundle_identity_blake3);
    }
    Ok(identities)
}

pub(super) fn resume_policy_digest(plan: &SourceBuiltFixedPointPlan) -> Result<String, RunError> {
    crunch_dev_resume_core::compute_resume_policy_identity(&crunch_dev_resume_core::ResumePolicyDigests {
        closure_policy_digest_blake3: plan.policies.closure_policy_digest_blake3.clone(),
        hermeticity_policy_digest_blake3: plan.policies.hermeticity_policy_digest_blake3.clone(),
        protected_execution_policy_digest_blake3: plan.policies.protected_execution_policy_digest_blake3.clone(),
        effect_policy_digest_blake3: plan.policies.effect_policy_digest_blake3.clone(),
        normalization_policy_digest_blake3: plan.policies.normalization_policy_digest_blake3.clone(),
    })
    .map_err(|error| proof_error(format!("computing dev resume policy identity: {error}")))
}

fn sealed_manifest(
    prepared: &PreparedAttempt,
    policy_digest: &str,
    completed_stage: crunch_dev_resume_core::ResumeStageBinding,
    mut payloads: Vec<crunch_dev_resume_core::ResumePayloadBinding>,
) -> Result<crunch_dev_resume_core::ResumeBundleManifest, RunError> {
    payloads.sort_by(|left, right| left.payload_id.cmp(&right.payload_id));
    crunch_dev_resume_core::seal_resume_bundle(crunch_dev_resume_core::ResumeBundleManifest {
        schema: String::new(),
        bundle_identity_blake3: String::new(),
        source_authority_digest_blake3: prepared.plan.source_authority_digest_blake3.clone(),
        plan_digest_blake3: prepared.plan.plan_digest_blake3.clone(),
        policy_digest_blake3: policy_digest.to_string(),
        completed_stage,
        payloads,
    })
    .map_err(|error| proof_error(format!("sealing dev resume manifest: {error}")))
}

pub(super) fn provider_checkpoint_binding(
    admitted: &crate::source_built_fixed_point_checkpoint_shell::AdmittedProviderCheckpoint,
) -> Result<crunch_dev_resume_core::ResumePayloadBinding, RunError> {
    let (total_file_bytes, entry_count) =
        admitted.manifest.payloads.iter().try_fold((0_u64, 0_u32), |(bytes, entries), payload| {
            Ok::<_, RunError>((
                bytes
                    .checked_add(payload.total_file_bytes)
                    .ok_or_else(|| proof_error("dev provider checkpoint byte count overflow".to_string()))?,
                entries
                    .checked_add(payload.entry_count)
                    .ok_or_else(|| proof_error("dev provider checkpoint entry count overflow".to_string()))?,
            ))
        })?;
    Ok(crunch_dev_resume_core::ResumePayloadBinding {
        payload_id: crunch_dev_resume_core::PROVIDER_CHECKPOINT_PAYLOAD_ID.to_string(),
        destination_relative_path: PROVIDER_CHECKPOINT_DESTINATION.to_string(),
        kind: crunch_dev_resume_core::ResumePayloadKind::ContentReference,
        digest_blake3: admitted.admission.checkpoint_digest_blake3.clone(),
        total_file_bytes,
        entry_count,
    })
}

fn fixed_point_stage_binding(
    fixed_point_dir: &Path,
    stage: crunch_dev_resume_core::ResumeStage,
    producer_digest: &str,
) -> Result<crunch_dev_resume_core::ResumeStageBinding, RunError> {
    let stage_name = match stage {
        crunch_dev_resume_core::ResumeStage::MantleStage1 => crate::cargo_free_self_build::STAGE1_DIR,
        crunch_dev_resume_core::ResumeStage::MantleStage2 => crate::cargo_free_self_build::STAGE2_DIR,
        _ => return Err(proof_error("fixed-point binding requested for a provider stage".to_string())),
    };
    Ok(crunch_dev_resume_core::ResumeStageBinding {
        stage,
        stage_id: stage.id().to_string(),
        producer_executable_digest_blake3: producer_digest.to_string(),
        output_digest_blake3: fixed_point_binary_digest(fixed_point_dir, stage_name)?,
        execution_evidence_digest_blake3: crate::protected_exec::blake3_file_hex(
            &fixed_point_dir.join(stage_name).join(crate::cargo_free_self_build::RECEIPT_FILE),
        )
        .map_err(|error| proof_error(format!("hashing dev resume stage receipt: {error}")))?,
    })
}

pub(super) fn fixed_point_binary_digest(fixed_point_dir: &Path, stage_name: &str) -> Result<String, RunError> {
    crate::protected_exec::blake3_file_hex(
        &fixed_point_dir.join(stage_name).join(crate::cargo_free_self_build::PRODUCED_MANTLE_FILE),
    )
    .map_err(|error| proof_error(format!("hashing dev resume fixed-point binary: {error}")))
}

pub(crate) fn current_executable_digest() -> Result<String, RunError> {
    let executable = std::env::current_exe()
        .map_err(|error| proof_error(format!("resolving dev resume producer executable: {error}")))?;
    crate::protected_exec::blake3_file_hex(&executable)
        .map_err(|error| proof_error(format!("hashing dev resume producer executable: {error}")))
}

fn publish_manifest(cache: &Path, manifest: &crunch_dev_resume_core::ResumeBundleManifest) -> Result<(), RunError> {
    let parent = create_manifest_parent(cache, manifest)?;
    let destination = parent.join(&manifest.bundle_identity_blake3);
    if destination.exists() {
        return validate_existing_manifest(&destination, manifest);
    }
    let temporary = parent.join(format!(
        "{DEV_RESUME_MANIFEST_TEMP_PREFIX}{}-{}",
        manifest.bundle_identity_blake3,
        std::process::id()
    ));
    remove_stale_manifest_temp(&temporary)?;
    fs::create_dir(&temporary)
        .map_err(|error| proof_error(format!("creating dev resume manifest temp {}: {error}", temporary.display())))?;
    write_json_create_new(&temporary.join(manifest_io::DEV_RESUME_MANIFEST_FILE), manifest)?;
    if let Err(error) = crate::linux_rename::rename_path_no_replace(&temporary, &destination) {
        if error.kind() != std::io::ErrorKind::AlreadyExists {
            return Err(proof_error(format!("publishing dev resume manifest {}: {error}", destination.display())));
        }
        fs::remove_dir_all(&temporary)
            .map_err(|cleanup| proof_error(format!("cleaning duplicate dev resume manifest: {cleanup}")))?;
        validate_existing_manifest(&destination, manifest)?;
    }
    debug_assert!(destination.join(manifest_io::DEV_RESUME_MANIFEST_FILE).is_file());
    Ok(())
}

fn create_manifest_parent(
    cache: &Path,
    manifest: &crunch_dev_resume_core::ResumeBundleManifest,
) -> Result<PathBuf, RunError> {
    fs::create_dir_all(cache).map_err(|error| proof_error(format!("creating dev cache root: {error}")))?;
    manifest_io::require_manifest_root(cache)?;
    let mut parent = cache.to_path_buf();
    for component in [
        DEV_RESUME_MANIFESTS_SUBDIR,
        manifest.plan_digest_blake3.as_str(),
        manifest.completed_stage.stage.id(),
    ] {
        parent.push(component);
        match fs::create_dir(&parent) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
            Err(error) => return Err(proof_error(format!("creating dev manifest parent: {error}"))),
        }
        manifest_io::require_manifest_root(&parent)?;
    }
    Ok(parent)
}

fn remove_stale_manifest_temp(path: &Path) -> Result<(), RunError> {
    let metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(proof_error(format!("reading dev resume manifest temp: {error}"))),
    };
    if !metadata.is_dir() || metadata.file_type().is_symlink() {
        return Err(proof_error("dev resume manifest temp has an invalid type".to_string()));
    }
    fs::remove_dir_all(path).map_err(|error| proof_error(format!("removing stale dev resume manifest temp: {error}")))
}

fn validate_existing_manifest(
    root: &Path,
    expected: &crunch_dev_resume_core::ResumeBundleManifest,
) -> Result<(), RunError> {
    let observed = manifest_io::read_manifest(root)?;
    if observed != *expected {
        return Err(proof_error("conflicting dev resume manifest".to_string()));
    }
    Ok(())
}

pub(super) fn manifests_root(cache: &Path, plan_digest: &str) -> PathBuf {
    cache.join(DEV_RESUME_MANIFESTS_SUBDIR).join(plan_digest)
}
