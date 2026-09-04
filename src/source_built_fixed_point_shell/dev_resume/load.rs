use super::*;

const DEV_RESUME_RESTORE_FALLBACK_FILE: &str = "dev-resume-restore-fallback.txt";
const DEV_RESUME_FALLBACK_CHARS_MAX: usize = 4_096;

pub(crate) enum FixedPointResume {
    None,
    Stage1,
    Complete,
}

pub(crate) struct PreparedDevResume {
    pub plan: crunch_dev_resume_core::ResumePlan,
    pub provider_resume: Option<super::super::checkpoint_integration::DevProviderResume>,
    pub fixed_point_resume: FixedPointResume,
}

struct RestoreContext<'a, 'b> {
    options: &'a SourceBuiltFixedPointOptions<'b>,
    prepared: &'a PreparedAttempt,
    cache: &'a Path,
    checkpoint_store: &'a Path,
    limits: crate::source_built_fixed_point_checkpoint_shell::ProviderCheckpointLimits,
}

// r[impl source_built_fixed_point_improved_iteration.dev_cross_run_resume]
pub(crate) fn prepare_dev_resume(
    options: &SourceBuiltFixedPointOptions<'_>,
    prepared: &PreparedAttempt,
) -> Result<PreparedDevResume, RunError> {
    if !options.dev_resume {
        return Ok(cold_preparation());
    }
    let cache = options
        .dev_provider_cache
        .ok_or_else(|| proof_error("dev resume requires the dev provider cache".to_string()))?;
    let checkpoint_store = publish::checkpoint_store(cache);
    let limits = crate::source_built_fixed_point_checkpoint_shell::provider_checkpoint_limits(options.disk_bytes_max);
    let admitted = match crate::source_built_fixed_point_checkpoint_shell::admit_dev_provider_checkpoint_store(
        &checkpoint_store,
        &prepared.plan,
        STAGEX_PROVIDER_EXPECTED_NORMALIZED_DIGEST,
        limits,
    ) {
        Ok(Some(admitted)) => admitted,
        Ok(None) | Err(_) => return Ok(cold_preparation()),
    };
    let loaded = match candidates::load_candidates(cache, prepared, &admitted, limits.preserved_tree) {
        Ok(loaded) => loaded,
        Err(_) => return Ok(cold_preparation()),
    };
    let plan = crunch_dev_resume_core::plan_resume(&crunch_dev_resume_core::ResumePlanInput {
        mode: crunch_dev_resume_core::ResumeRunMode::Dev,
        candidates: loaded.candidates,
        observed_rejections: loaded.observed_rejections,
    });
    if plan.disposition != crunch_dev_resume_core::ResumeDisposition::Restore {
        return Ok(PreparedDevResume {
            plan,
            provider_resume: None,
            fixed_point_resume: FixedPointResume::None,
        });
    }
    restore_selected_plan(
        &RestoreContext {
            options,
            prepared,
            cache,
            checkpoint_store: &checkpoint_store,
            limits,
        },
        plan,
    )
}

fn restore_selected_plan(
    context: &RestoreContext<'_, '_>,
    plan: crunch_dev_resume_core::ResumePlan,
) -> Result<PreparedDevResume, RunError> {
    let selected_identity = plan
        .selected_bundle_identity_blake3
        .as_deref()
        .ok_or_else(|| proof_error("dev resume plan omitted its selected identity".to_string()))?;
    let selected = match candidates::load_selected_manifest(
        context.cache,
        &context.prepared.plan.plan_digest_blake3,
        selected_identity,
    ) {
        Ok(selected) => selected,
        Err(error) => {
            return restore_fallback(
                context.prepared,
                &FixedPointResume::None,
                &plan,
                crunch_dev_resume_core::ResumeRejectReason::ManifestInvalid,
                error,
            );
        }
    };
    let completed_stage = selected.completed_stage.stage;
    let fixed_point_resume =
        match restore_fixed_point_payload(context.cache, context.prepared, &selected, context.limits.preserved_tree) {
            Ok(resume) => resume,
            Err(error) => {
                return restore_fallback(
                    context.prepared,
                    &FixedPointResume::None,
                    &plan,
                    crunch_dev_resume_core::ResumeRejectReason::PayloadMismatch,
                    error,
                );
            }
        };
    let provider_resume = match super::super::checkpoint_integration::restore_dev_provider_stage(
        context.options,
        context.prepared,
        context.checkpoint_store,
        completed_stage,
    ) {
        Ok(Some(resume)) => resume,
        Ok(None) => {
            let error = proof_error("selected dev provider checkpoint disappeared before restore".to_string());
            return restore_fallback(
                context.prepared,
                &fixed_point_resume,
                &plan,
                crunch_dev_resume_core::ResumeRejectReason::StageMismatch,
                error,
            );
        }
        Err(error) => {
            return restore_fallback(
                context.prepared,
                &fixed_point_resume,
                &plan,
                crunch_dev_resume_core::ResumeRejectReason::StageMismatch,
                error,
            );
        }
    };
    debug_assert_eq!(plan.completed_stage, Some(completed_stage));
    debug_assert!(!plan.restored_stages.is_empty());
    Ok(PreparedDevResume {
        plan,
        provider_resume: Some(provider_resume),
        fixed_point_resume,
    })
}

fn cold_preparation() -> PreparedDevResume {
    PreparedDevResume {
        plan: crunch_dev_resume_core::plan_resume(&crunch_dev_resume_core::ResumePlanInput {
            mode: crunch_dev_resume_core::ResumeRunMode::Dev,
            candidates: Vec::new(),
            observed_rejections: Vec::new(),
        }),
        provider_resume: None,
        fixed_point_resume: FixedPointResume::None,
    }
}

fn restore_fallback(
    prepared: &PreparedAttempt,
    fixed_point_resume: &FixedPointResume,
    selected_plan: &crunch_dev_resume_core::ResumePlan,
    reason: crunch_dev_resume_core::ResumeRejectReason,
    error: RunError,
) -> Result<PreparedDevResume, RunError> {
    rollback_fixed_point_resume(&prepared.staging_dir, fixed_point_resume)?;
    let detail = format!("dev resume fell back to execution: {error}");
    let bounded = detail.chars().take(DEV_RESUME_FALLBACK_CHARS_MAX).collect::<String>();
    fs::write(prepared.staging_dir.join(DEV_RESUME_RESTORE_FALLBACK_FILE), format!("{bounded}\n"))
        .map_err(|write_error| proof_error(format!("writing dev resume fallback evidence: {write_error}")))?;
    let bundle_identity_blake3 = selected_plan
        .selected_bundle_identity_blake3
        .clone()
        .ok_or_else(|| proof_error("restore fallback plan omitted bundle identity".to_string()))?;
    let completed_stage = selected_plan
        .completed_stage
        .ok_or_else(|| proof_error("restore fallback plan omitted completed stage".to_string()))?;
    let rejected = crunch_dev_resume_core::RejectedResumeCandidate {
        bundle_identity_blake3,
        completed_stage,
        reason,
    };
    debug_assert!(!bounded.is_empty());
    Ok(cold_preparation_with_rejection(rejected))
}

fn cold_preparation_with_rejection(rejected: crunch_dev_resume_core::RejectedResumeCandidate) -> PreparedDevResume {
    PreparedDevResume {
        plan: crunch_dev_resume_core::plan_resume(&crunch_dev_resume_core::ResumePlanInput {
            mode: crunch_dev_resume_core::ResumeRunMode::Dev,
            candidates: Vec::new(),
            observed_rejections: vec![rejected],
        }),
        provider_resume: None,
        fixed_point_resume: FixedPointResume::None,
    }
}

pub(super) fn rollback_fixed_point_resume(
    staging_dir: &Path,
    fixed_point_resume: &FixedPointResume,
) -> Result<(), RunError> {
    let destination = match fixed_point_resume {
        FixedPointResume::None => return Ok(()),
        FixedPointResume::Stage1 => staging_dir.join(FIXED_POINT_DIR).join(crate::cargo_free_self_build::STAGE1_DIR),
        FixedPointResume::Complete => staging_dir.join(FIXED_POINT_DIR),
    };
    io::remove_path(&destination)?;
    debug_assert!(!destination.exists());
    Ok(())
}

fn restore_fixed_point_payload(
    cache: &Path,
    prepared: &PreparedAttempt,
    manifest: &crunch_dev_resume_core::ResumeBundleManifest,
    limits: crate::preserved_evidence_tree::PreservedEvidenceTreeLimits,
) -> Result<FixedPointResume, RunError> {
    let stage = manifest.completed_stage.stage;
    let (payload_id, destination, outcome) = match stage {
        crunch_dev_resume_core::ResumeStage::MantleStage1 => (
            crunch_dev_resume_core::MANTLE_STAGE1_PAYLOAD_ID,
            prepared.staging_dir.join(FIXED_POINT_DIR).join(crate::cargo_free_self_build::STAGE1_DIR),
            FixedPointResume::Stage1,
        ),
        crunch_dev_resume_core::ResumeStage::MantleStage2 => (
            crunch_dev_resume_core::FIXED_POINT_COMPLETE_PAYLOAD_ID,
            prepared.staging_dir.join(FIXED_POINT_DIR),
            FixedPointResume::Complete,
        ),
        _ => return Ok(FixedPointResume::None),
    };
    let payload = manifest
        .payloads
        .iter()
        .find(|payload| payload.payload_id == payload_id)
        .ok_or_else(|| proof_error("selected dev resume payload is missing".to_string()))?;
    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| proof_error(format!("creating dev resume destination parent: {error}")))?;
    }
    io::restore_tree_object(cache, payload, &destination, limits)?;
    Ok(outcome)
}
