use super::*;

const DEV_RESUME_STAGE_CANDIDATES_MAX: usize = 4;

pub(super) struct CandidateLoad {
    pub candidates: Vec<crunch_dev_resume_core::ResumeCandidateObservation>,
    pub observed_rejections: Vec<crunch_dev_resume_core::RejectedResumeCandidate>,
}

struct CandidateContext<'a> {
    cache: &'a Path,
    prepared: &'a PreparedAttempt,
    checkpoint_store: &'a Path,
    checkpoint_limits: crate::source_built_fixed_point_checkpoint_shell::ProviderCheckpointLimits,
    current_producer: &'a str,
    policy_digest: &'a str,
    tree_limits: crate::preserved_evidence_tree::PreservedEvidenceTreeLimits,
}

pub(super) fn load_candidates(
    cache: &Path,
    prepared: &PreparedAttempt,
    checkpoint_limits: crate::source_built_fixed_point_checkpoint_shell::ProviderCheckpointLimits,
) -> Result<CandidateLoad, RunError> {
    let manifests_root = publish::manifests_root(cache, &prepared.plan.plan_digest_blake3);
    if !require_candidate_directory(&manifests_root)? {
        return Ok(CandidateLoad {
            candidates: Vec::new(),
            observed_rejections: Vec::new(),
        });
    }
    let policy_digest = publish::resume_policy_digest(&prepared.plan)?;
    let checkpoint_store = publish::checkpoint_store(cache);
    let current_producer = publish::current_executable_digest()?;
    let context = CandidateContext {
        cache,
        prepared,
        checkpoint_store: &checkpoint_store,
        checkpoint_limits,
        current_producer: &current_producer,
        policy_digest: &policy_digest,
        tree_limits: checkpoint_limits.preserved_tree,
    };
    let candidate_items_max =
        crunch_dev_resume_core::ResumeStage::ALL.len().saturating_mul(DEV_RESUME_STAGE_CANDIDATES_MAX);
    let mut candidates = Vec::with_capacity(candidate_items_max);
    let mut observed_rejections = Vec::with_capacity(candidate_items_max);
    for stage in crunch_dev_resume_core::ResumeStage::ALL {
        load_stage_candidates(&context, &manifests_root, stage, &mut candidates, &mut observed_rejections)?;
    }
    debug_assert!(candidates.len() <= candidate_items_max);
    debug_assert!(observed_rejections.len() <= candidate_items_max);
    Ok(CandidateLoad {
        candidates,
        observed_rejections,
    })
}

fn load_stage_candidates(
    context: &CandidateContext<'_>,
    manifests_root: &Path,
    stage: crunch_dev_resume_core::ResumeStage,
    candidates: &mut Vec<crunch_dev_resume_core::ResumeCandidateObservation>,
    observed_rejections: &mut Vec<crunch_dev_resume_core::RejectedResumeCandidate>,
) -> Result<(), RunError> {
    let stage_root = manifests_root.join(stage.id());
    if !require_candidate_directory(&stage_root)? {
        return Ok(());
    }
    let mut entries = fs::read_dir(&stage_root)
        .map_err(|error| proof_error(format!("reading dev resume candidates {}: {error}", stage_root.display())))?
        .take(DEV_RESUME_STAGE_CANDIDATES_MAX.saturating_add(1))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| proof_error(format!("reading dev resume candidate: {error}")))?;
    entries.sort_by_key(fs::DirEntry::file_name);
    if entries.len() > DEV_RESUME_STAGE_CANDIDATES_MAX {
        observed_rejections.push(rejected_cache_entry(
            &stage_root,
            stage,
            crunch_dev_resume_core::ResumeRejectReason::CandidateLimitExceeded,
        ));
        return Ok(());
    }
    for entry in entries {
        match observe_candidate(context, &entry.path(), stage) {
            Ok(candidate) => candidates.push(candidate),
            Err(reason) => observed_rejections.push(rejected_cache_entry(&entry.path(), stage, reason)),
        }
    }
    debug_assert!(
        candidates.len()
            <= crunch_dev_resume_core::ResumeStage::ALL.len().saturating_mul(DEV_RESUME_STAGE_CANDIDATES_MAX)
    );
    Ok(())
}

fn require_candidate_directory(path: &Path) -> Result<bool, RunError> {
    let metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(false),
        Err(error) => return Err(proof_error(format!("reading dev resume candidate directory: {error}"))),
    };
    if !metadata.is_dir() || metadata.file_type().is_symlink() {
        return Err(proof_error("dev resume candidate directory must not be a symlink".to_string()));
    }
    Ok(true)
}

fn observe_candidate(
    context: &CandidateContext<'_>,
    path: &Path,
    stage: crunch_dev_resume_core::ResumeStage,
) -> Result<crunch_dev_resume_core::ResumeCandidateObservation, crunch_dev_resume_core::ResumeRejectReason> {
    let manifest = read_manifest_candidate(path, stage)
        .map_err(|_| crunch_dev_resume_core::ResumeRejectReason::ManifestInvalid)?;
    crunch_dev_resume_core::validate_resume_manifest(&manifest)
        .map_err(|_| crunch_dev_resume_core::ResumeRejectReason::ManifestInvalid)?;
    let checkpoint_digest = manifest
        .payloads
        .iter()
        .find(|payload| payload.payload_id == crunch_dev_resume_core::PROVIDER_CHECKPOINT_PAYLOAD_ID)
        .map(|payload| payload.digest_blake3.as_str())
        .ok_or(crunch_dev_resume_core::ResumeRejectReason::ManifestInvalid)?;
    let admitted = admit_candidate_checkpoint(context, checkpoint_digest)
        .map_err(|_| crunch_dev_resume_core::ResumeRejectReason::PayloadMismatch)?;
    let provider_payload = publish::provider_checkpoint_binding(&admitted)
        .map_err(|_| crunch_dev_resume_core::ResumeRejectReason::PayloadMismatch)?;
    let observed_payloads = observe_manifest_payloads(context.cache, &manifest, &provider_payload, context.tree_limits)
        .map_err(|_| crunch_dev_resume_core::ResumeRejectReason::PayloadMismatch)?;
    let current_stage =
        current_stage_binding(context.cache, &manifest, &admitted, context.current_producer, context.tree_limits)
            .map_err(|_| crunch_dev_resume_core::ResumeRejectReason::StageMismatch)?;
    Ok(crunch_dev_resume_core::ResumeCandidateObservation {
        manifest,
        current_source_authority_digest_blake3: context.prepared.plan.source_authority_digest_blake3.clone(),
        current_plan_digest_blake3: context.prepared.plan.plan_digest_blake3.clone(),
        current_policy_digest_blake3: context.policy_digest.to_string(),
        current_stage,
        observed_payloads,
    })
}

fn admit_candidate_checkpoint(
    context: &CandidateContext<'_>,
    digest: &str,
) -> Result<crate::source_built_fixed_point_checkpoint_shell::AdmittedProviderCheckpoint, RunError> {
    super::super::checkpoint_integration::admit_dev_checkpoint_identity(
        context.checkpoint_store,
        &context.prepared.plan,
        digest,
        context.checkpoint_limits,
    )?
    .ok_or_else(|| proof_error("dev resume provider checkpoint is missing".to_string()))
}

fn rejected_cache_entry(
    path: &Path,
    stage: crunch_dev_resume_core::ResumeStage,
    reason: crunch_dev_resume_core::ResumeRejectReason,
) -> crunch_dev_resume_core::RejectedResumeCandidate {
    let name = path.file_name().unwrap_or_default();
    let identity = name
        .to_str()
        .filter(|value| is_lower_blake3(value))
        .map_or_else(|| blake3::hash(name.as_encoded_bytes()).to_hex().to_string(), str::to_string);
    crunch_dev_resume_core::RejectedResumeCandidate {
        bundle_identity_blake3: identity,
        completed_stage: stage,
        reason,
    }
}

fn is_lower_blake3(value: &str) -> bool {
    value.len() == blake3::OUT_LEN.saturating_mul(2)
        && value.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn read_manifest_candidate(
    root: &Path,
    expected_stage: crunch_dev_resume_core::ResumeStage,
) -> Result<crunch_dev_resume_core::ResumeBundleManifest, RunError> {
    let name = root
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| proof_error("dev resume candidate name is not UTF-8".to_string()))?;
    let manifest = manifest_io::read_manifest(root)?;
    if manifest.bundle_identity_blake3 != name || manifest.completed_stage.stage != expected_stage {
        return Err(proof_error("dev resume manifest path identity mismatch".to_string()));
    }
    Ok(manifest)
}

fn observe_manifest_payloads(
    cache: &Path,
    manifest: &crunch_dev_resume_core::ResumeBundleManifest,
    provider_payload: &crunch_dev_resume_core::ResumePayloadBinding,
    tree_limits: crate::preserved_evidence_tree::PreservedEvidenceTreeLimits,
) -> Result<Vec<crunch_dev_resume_core::ResumePayloadBinding>, RunError> {
    let mut observed = Vec::with_capacity(manifest.payloads.len());
    for payload in &manifest.payloads {
        let value = if payload.payload_id == crunch_dev_resume_core::PROVIDER_CHECKPOINT_PAYLOAD_ID {
            provider_payload.clone()
        } else {
            io::observe_tree_object(cache, payload, tree_limits)?
        };
        observed.push(value);
    }
    observed.sort_by(|left, right| left.payload_id.cmp(&right.payload_id));
    Ok(observed)
}

fn current_stage_binding(
    cache: &Path,
    manifest: &crunch_dev_resume_core::ResumeBundleManifest,
    admitted: &crate::source_built_fixed_point_checkpoint_shell::AdmittedProviderCheckpoint,
    current_producer: &str,
    tree_limits: crate::preserved_evidence_tree::PreservedEvidenceTreeLimits,
) -> Result<crunch_dev_resume_core::ResumeStageBinding, RunError> {
    let stage = manifest.completed_stage.stage;
    let stage_index = usize::try_from(stage.ordinal())
        .map_err(|_| proof_error("dev resume stage ordinal does not fit usize".to_string()))?;
    if stage_index < crate::source_built_fixed_point_checkpoint::PROVIDER_CHECKPOINT_STAGE_COUNT {
        let record = admitted
            .manifest
            .stages
            .get(stage_index)
            .ok_or_else(|| proof_error("dev provider checkpoint omitted a stage record".to_string()))?;
        return Ok(crunch_dev_resume_core::ResumeStageBinding {
            stage,
            stage_id: stage.id().to_string(),
            producer_executable_digest_blake3: current_producer.to_string(),
            output_digest_blake3: record.output_digest_blake3.clone(),
            execution_evidence_digest_blake3: record.execution_evidence_digest_blake3.clone(),
        });
    }
    let payload_id = match stage {
        crunch_dev_resume_core::ResumeStage::MantleStage1 => crunch_dev_resume_core::MANTLE_STAGE1_PAYLOAD_ID,
        crunch_dev_resume_core::ResumeStage::MantleStage2 => crunch_dev_resume_core::FIXED_POINT_COMPLETE_PAYLOAD_ID,
        _ => return Err(proof_error("dev resume stage index is inconsistent".to_string())),
    };
    let payload = manifest
        .payloads
        .iter()
        .find(|payload| payload.payload_id == payload_id)
        .ok_or_else(|| proof_error("dev resume fixed-point payload is missing".to_string()))?;
    let object = io::object_path(cache, &payload.digest_blake3);
    fixed_point_binding_from_object(&object, stage, current_producer, tree_limits)
}

fn fixed_point_binding_from_object(
    object: &Path,
    stage: crunch_dev_resume_core::ResumeStage,
    current_producer: &str,
    _tree_limits: crate::preserved_evidence_tree::PreservedEvidenceTreeLimits,
) -> Result<crunch_dev_resume_core::ResumeStageBinding, RunError> {
    let (stage_root, producer) = match stage {
        crunch_dev_resume_core::ResumeStage::MantleStage1 => (object.to_path_buf(), current_producer.to_string()),
        crunch_dev_resume_core::ResumeStage::MantleStage2 => {
            let stage1_digest = crate::protected_exec::blake3_file_hex(
                &object
                    .join(crate::cargo_free_self_build::STAGE1_DIR)
                    .join(crate::cargo_free_self_build::PRODUCED_MANTLE_FILE),
            )
            .map_err(|error| proof_error(format!("hashing resumed stage1 producer: {error}")))?;
            (object.join(crate::cargo_free_self_build::STAGE2_DIR), stage1_digest)
        }
        _ => return Err(proof_error("fixed-point object requested for a provider stage".to_string())),
    };
    let output =
        crate::protected_exec::blake3_file_hex(&stage_root.join(crate::cargo_free_self_build::PRODUCED_MANTLE_FILE))
            .map_err(|error| proof_error(format!("hashing resumed Mantle output: {error}")))?;
    let evidence = crate::protected_exec::blake3_file_hex(&stage_root.join(crate::cargo_free_self_build::RECEIPT_FILE))
        .map_err(|error| proof_error(format!("hashing resumed Mantle receipt: {error}")))?;
    Ok(crunch_dev_resume_core::ResumeStageBinding {
        stage,
        stage_id: stage.id().to_string(),
        producer_executable_digest_blake3: producer,
        output_digest_blake3: output,
        execution_evidence_digest_blake3: evidence,
    })
}

pub(super) fn load_selected_manifest(
    cache: &Path,
    plan_digest: &str,
    selected_identity: &str,
) -> Result<crunch_dev_resume_core::ResumeBundleManifest, RunError> {
    for stage in crunch_dev_resume_core::ResumeStage::ALL {
        let root = publish::manifests_root(cache, plan_digest).join(stage.id()).join(selected_identity);
        if root.is_dir() {
            return read_manifest_candidate(&root, stage);
        }
    }
    Err(proof_error("selected dev resume manifest is missing".to_string()))
}
