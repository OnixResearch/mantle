use alloc::vec;
use alloc::vec::Vec;

use super::*;

#[test]
// r[verify source_built_fixed_point_improved_iteration.dev_cross_run_resume]
fn fresh_transition_bundle_selects_the_first_incomplete_stage() {
    let input = ResumePlanInput {
        mode: ResumeRunMode::Dev,
        candidates: vec![candidate(ResumeStage::StagexTransition)],
        observed_rejections: Vec::new(),
    };
    let plan = plan_resume(&input);

    assert_eq!(plan.disposition, ResumeDisposition::Restore);
    assert_eq!(plan.restored_stages, vec![ResumeStage::StagexTransition]);
    assert_eq!(plan.first_incomplete_stage, Some(ResumeStage::StagexProvider));
    assert_eq!(plan.executed_stages.len(), ResumeStage::ALL.len().saturating_sub(1));
}

#[test]
fn highest_valid_bundle_restores_a_contiguous_prefix() {
    let input = ResumePlanInput {
        mode: ResumeRunMode::Dev,
        candidates: vec![
            candidate(ResumeStage::StagexTransition),
            candidate(ResumeStage::FullSourceRustProvider),
            candidate(ResumeStage::MantleStage1),
        ],
        observed_rejections: Vec::new(),
    };
    let plan = plan_resume(&input);

    assert_eq!(plan.completed_stage, Some(ResumeStage::MantleStage1));
    assert_eq!(plan.restored_stages.len(), ResumeStage::MantleStage1.index().saturating_add(1));
    assert_eq!(plan.executed_stages, vec![ResumeStage::MantleStage2]);
    assert_eq!(plan.first_incomplete_stage, Some(ResumeStage::MantleStage2));
}

#[test]
fn complete_bundle_has_no_executed_stage() {
    let plan = plan_resume(&ResumePlanInput {
        mode: ResumeRunMode::Dev,
        candidates: vec![candidate(ResumeStage::MantleStage2)],
        observed_rejections: Vec::new(),
    });

    assert_eq!(plan.restored_stages, ResumeStage::ALL);
    assert!(plan.executed_stages.is_empty());
    assert_eq!(plan.first_incomplete_stage, None);
    assert_eq!(plan.cold_reason, None);
}

#[test]
fn dev_resume_report_roundtrips_with_restored_and_executed_stages() {
    let report = DevResumeReport {
        schema: DEV_RESUME_REPORT_SCHEMA.into(),
        status: "dev-only".into(),
        mode: ResumeRunMode::Dev,
        plan_digest_blake3: DIGEST_A.into(),
        selected_bundle_identity_blake3: Some(DIGEST_B.into()),
        published_bundle_identities_blake3: vec![DIGEST_C.into()],
        restored_stages: vec![ResumeStage::StagexTransition],
        executed_stages: vec![ResumeStage::StagexProvider],
        first_incomplete_stage: Some(ResumeStage::StagexProvider),
        rejected_candidates: vec![RejectedResumeCandidate {
            bundle_identity_blake3: DIGEST_A.into(),
            completed_stage: ResumeStage::StagexProvider,
            reason: ResumeRejectReason::ManifestInvalid,
        }],
        cache_adoption_disposition: "resume-restored".into(),
        promoted_receipt_written: false,
        release_alias_updated: false,
        non_claims: vec!["bounded".into(), "dev-only".into(), "not-promoted".into()],
    };
    let encoded = serde_json::to_vec(&report).unwrap();
    let decoded: DevResumeReport = serde_json::from_slice(&encoded).unwrap();

    assert_eq!(decoded, report);
    assert_eq!(decoded.first_incomplete_stage, Some(ResumeStage::StagexProvider));
}

#[test]
fn policy_identity_is_stable_and_role_sensitive() {
    let policies = ResumePolicyDigests {
        closure_policy_digest_blake3: DIGEST_A.into(),
        hermeticity_policy_digest_blake3: DIGEST_A.into(),
        protected_execution_policy_digest_blake3: DIGEST_A.into(),
        effect_policy_digest_blake3: DIGEST_A.into(),
        normalization_policy_digest_blake3: DIGEST_A.into(),
    };
    let first = compute_resume_policy_identity(&policies).unwrap();
    let second = compute_resume_policy_identity(&policies).unwrap();
    let mut changed = policies;
    changed.effect_policy_digest_blake3 = DIGEST_B.into();

    assert_eq!(first, second);
    assert_ne!(first, compute_resume_policy_identity(&changed).unwrap());
}

#[test]
fn bundle_identity_is_stable_and_payload_sensitive() {
    let first = manifest(ResumeStage::StagexTransition);
    let second = manifest(ResumeStage::StagexTransition);
    let mut changed = first.clone();
    changed.payloads[0].digest_blake3 = DIGEST_B.into();
    let changed = seal_resume_bundle(changed).unwrap();

    assert_eq!(first.bundle_identity_blake3, second.bundle_identity_blake3);
    assert_ne!(first.bundle_identity_blake3, changed.bundle_identity_blake3);
    assert!(validate_resume_manifest(&first).is_ok());
    assert!(validate_resume_manifest(&changed).is_ok());
}
