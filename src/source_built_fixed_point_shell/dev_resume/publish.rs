use super::*;

const DEV_RESUME_CHECKPOINT_STORE_SUBDIR: &str = "dev-resume-checkpoints";
const DEV_RESUME_MANIFESTS_SUBDIR: &str = "dev-resume-manifests";
const DEV_RESUME_MANIFEST_TEMP_PREFIX: &str = ".dev-resume-manifest-tmp-";
const PROVIDER_CHECKPOINT_DESTINATION: &str = "provider-checkpoint";
const MANTLE_STAGE1_DESTINATION: &str = "fixed-point/stage1";
const FIXED_POINT_COMPLETE_DESTINATION: &str = "fixed-point";

pub(super) fn checkpoint_store(cache: &Path) -> PathBuf {
    cache.join(DEV_RESUME_CHECKPOINT_STORE_SUBDIR)
}

// r[impl source_built_fixed_point_improved_iteration.dev_cross_run_resume]
pub(crate) fn publish_dev_resume_bundles(
    options: &SourceBuiltFixedPointOptions<'_>,
    prepared: &PreparedAttempt,
    providers: &ConstructedProviders,
    fixed_point_dir: &Path,
) -> Result<Vec<String>, RunError> {
    let cache = options
        .dev_provider_cache
        .ok_or_else(|| proof_error("dev resume publication requires the dev cache".to_string()))?;
    let checkpoint_store = checkpoint_store(cache);
    let published = super::super::checkpoint_integration::publish_dev_resume_provider_checkpoint(
        options,
        prepared,
        providers,
        &checkpoint_store,
    )?;
    let admitted = crate::source_built_fixed_point_checkpoint_shell::admit_dev_provider_checkpoint_store(
        &checkpoint_store,
        &prepared.plan,
        STAGEX_PROVIDER_EXPECTED_NORMALIZED_DIGEST,
        crate::source_built_fixed_point_checkpoint_shell::provider_checkpoint_limits(options.disk_bytes_max),
    )?
    .ok_or_else(|| proof_error("published dev provider checkpoint was not admitted".to_string()))?;
    if admitted.admission.checkpoint_digest_blake3 != published.checkpoint_digest_blake3 {
        return Err(proof_error("published dev provider checkpoint identity changed".to_string()));
    }
    let policy_digest = resume_policy_digest(&prepared.plan)?;
    let provider_payload = provider_checkpoint_binding(&admitted)?;
    let producer_digest = current_executable_digest()?;
    let mut manifests = provider_stage_manifests(prepared, &admitted, &policy_digest, &provider_payload)?;
    let tree_limits = dev_resume_tree_limits(options);
    let stage1_payload = io::publish_tree_object(
        cache,
        &fixed_point_dir.join(crate::cargo_free_self_build::STAGE1_DIR),
        crunch_dev_resume_core::MANTLE_STAGE1_PAYLOAD_ID,
        MANTLE_STAGE1_DESTINATION,
        tree_limits,
    )?;
    let complete_payload = io::publish_tree_object(
        cache,
        fixed_point_dir,
        crunch_dev_resume_core::FIXED_POINT_COMPLETE_PAYLOAD_ID,
        FIXED_POINT_COMPLETE_DESTINATION,
        tree_limits,
    )?;
    let stage1_binding = fixed_point_stage_binding(
        fixed_point_dir,
        crunch_dev_resume_core::ResumeStage::MantleStage1,
        &producer_digest,
    )?;
    manifests.push(sealed_manifest(prepared, &policy_digest, stage1_binding, vec![
        stage1_payload,
        provider_payload.clone(),
    ])?);
    let stage1_digest = fixed_point_binary_digest(fixed_point_dir, crate::cargo_free_self_build::STAGE1_DIR)?;
    let stage2_binding =
        fixed_point_stage_binding(fixed_point_dir, crunch_dev_resume_core::ResumeStage::MantleStage2, &stage1_digest)?;
    manifests
        .push(sealed_manifest(prepared, &policy_digest, stage2_binding, vec![complete_payload, provider_payload])?);
    manifests.sort_by_key(|manifest| manifest.completed_stage.stage);
    for manifest in &manifests {
        publish_manifest(cache, manifest)?;
    }
    let identities = manifests.iter().map(|manifest| manifest.bundle_identity_blake3.clone()).collect::<Vec<_>>();
    debug_assert_eq!(identities.len(), crunch_dev_resume_core::ResumeStage::ALL.len());
    debug_assert!(identities.iter().all(|identity| !identity.is_empty()));
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

fn provider_stage_manifests(
    prepared: &PreparedAttempt,
    admitted: &crate::source_built_fixed_point_checkpoint_shell::AdmittedProviderCheckpoint,
    policy_digest: &str,
    provider_payload: &crunch_dev_resume_core::ResumePayloadBinding,
) -> Result<Vec<crunch_dev_resume_core::ResumeBundleManifest>, RunError> {
    let mut manifests = Vec::with_capacity(crunch_dev_resume_core::ResumeStage::ALL.len());
    for (stage, record) in crunch_dev_resume_core::ResumeStage::ALL
        .iter()
        .take(crate::source_built_fixed_point_checkpoint::PROVIDER_CHECKPOINT_STAGE_COUNT)
        .zip(&admitted.manifest.stages)
    {
        let binding = crunch_dev_resume_core::ResumeStageBinding {
            stage: *stage,
            stage_id: stage.id().to_string(),
            producer_executable_digest_blake3: record.producer_executable_digest_blake3.clone(),
            output_digest_blake3: record.output_digest_blake3.clone(),
            execution_evidence_digest_blake3: record.execution_evidence_digest_blake3.clone(),
        };
        manifests.push(sealed_manifest(prepared, policy_digest, binding, vec![provider_payload.clone()])?);
    }
    debug_assert_eq!(manifests.len(), crate::source_built_fixed_point_checkpoint::PROVIDER_CHECKPOINT_STAGE_COUNT);
    Ok(manifests)
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

pub(super) fn current_executable_digest() -> Result<String, RunError> {
    let executable = std::env::current_exe()
        .map_err(|error| proof_error(format!("resolving dev resume producer executable: {error}")))?;
    crate::protected_exec::blake3_file_hex(&executable)
        .map_err(|error| proof_error(format!("hashing dev resume producer executable: {error}")))
}

fn publish_manifest(cache: &Path, manifest: &crunch_dev_resume_core::ResumeBundleManifest) -> Result<(), RunError> {
    let parent = cache
        .join(DEV_RESUME_MANIFESTS_SUBDIR)
        .join(&manifest.plan_digest_blake3)
        .join(manifest.completed_stage.stage.id());
    fs::create_dir_all(&parent)
        .map_err(|error| proof_error(format!("creating dev resume manifest parent {}: {error}", parent.display())))?;
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
