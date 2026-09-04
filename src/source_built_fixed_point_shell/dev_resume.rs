use super::*;

mod candidates;
mod io;
mod load;
mod manifest_io;
mod publish;

pub(super) use load::FixedPointResume;
pub(super) use load::PreparedDevResume;
pub(super) use load::prepare_dev_resume;
pub(super) use publish::publish_dev_resume_bundles;

const DEV_RESUME_REPORT_FILE: &str = "dev-resume-report.json";
const DEV_RESUME_STATUS: &str = "dev-only";
const DEV_RESUME_NON_CLAIMS: [&str; 3] = [
    "restored stages were not executed again in this attempt",
    "dev resume does not satisfy a promoted fixed-point proof",
    "runtime confirmation is bounded to the recorded source profile, host, plan, and policy",
];

// r[impl source_built_fixed_point_improved_iteration.dev_cross_run_resume]
pub(super) fn dev_resume_tree_limits(
    options: &SourceBuiltFixedPointOptions<'_>,
) -> crate::preserved_evidence_tree::PreservedEvidenceTreeLimits {
    crate::source_built_fixed_point_checkpoint_shell::provider_checkpoint_limits(options.disk_bytes_max).preserved_tree
}

// machine-artifact-public: bootstrap.dev-resume-report
// r[impl source_built_fixed_point_improved_iteration.dev_resume_runtime_confirmation]
pub(super) fn write_dev_resume_report(
    prepared: &PreparedAttempt,
    plan: &crunch_dev_resume_core::ResumePlan,
    published_bundle_identities_blake3: &[String],
    cache_adoption_disposition: &str,
) -> Result<PathBuf, RunError> {
    let report = crunch_dev_resume_core::DevResumeReport {
        schema: crunch_dev_resume_core::DEV_RESUME_REPORT_SCHEMA.to_string(),
        status: DEV_RESUME_STATUS.to_string(),
        mode: crunch_dev_resume_core::ResumeRunMode::Dev,
        plan_digest_blake3: prepared.plan.plan_digest_blake3.clone(),
        selected_bundle_identity_blake3: plan.selected_bundle_identity_blake3.clone(),
        published_bundle_identities_blake3: published_bundle_identities_blake3.to_vec(),
        restored_stages: plan.restored_stages.clone(),
        executed_stages: plan.executed_stages.clone(),
        first_incomplete_stage: plan.first_incomplete_stage,
        rejected_candidates: plan.rejected_candidates.clone(),
        cache_adoption_disposition: cache_adoption_disposition.to_string(),
        promoted_receipt_written: false,
        release_alias_updated: false,
        non_claims: DEV_RESUME_NON_CLAIMS.iter().map(|value| (*value).to_string()).collect(),
    };
    let path = prepared.staging_dir.join(DEV_RESUME_REPORT_FILE);
    write_json_create_new(&path, &report)?;
    debug_assert!(path.is_file());
    debug_assert!(!report.executed_stages.is_empty() || report.first_incomplete_stage.is_none());
    Ok(path)
}

#[cfg(test)]
mod tests;
