use alloc::vec;
use alloc::vec::Vec;

use super::*;

#[test]
// r[verify source_built_fixed_point_improved_iteration.dev_cross_run_resume]
fn stale_source_plan_policy_producer_and_output_each_fail_closed() {
    let mutations = [
        ResumeRejectReason::SourceMismatch,
        ResumeRejectReason::PlanMismatch,
        ResumeRejectReason::PolicyMismatch,
        ResumeRejectReason::ProducerMismatch,
        ResumeRejectReason::OutputMismatch,
        ResumeRejectReason::ExecutionEvidenceMismatch,
    ];
    for expected_reason in mutations {
        let mut observed = candidate(ResumeStage::StagexTransition);
        match expected_reason {
            ResumeRejectReason::SourceMismatch => observed.current_source_authority_digest_blake3 = DIGEST_B.into(),
            ResumeRejectReason::PlanMismatch => observed.current_plan_digest_blake3 = DIGEST_A.into(),
            ResumeRejectReason::PolicyMismatch => observed.current_policy_digest_blake3 = DIGEST_A.into(),
            ResumeRejectReason::ProducerMismatch => {
                observed.current_stage.producer_executable_digest_blake3 = DIGEST_B.into();
            }
            ResumeRejectReason::OutputMismatch => observed.current_stage.output_digest_blake3 = DIGEST_A.into(),
            ResumeRejectReason::ExecutionEvidenceMismatch => {
                observed.current_stage.execution_evidence_digest_blake3 = DIGEST_A.into();
            }
            _ => unreachable!(),
        }
        assert_eq!(validate_resume_candidate(&observed), Err(expected_reason));
        assert_eq!(
            plan_resume(&ResumePlanInput {
                mode: ResumeRunMode::Dev,
                candidates: vec![observed],
                observed_rejections: Vec::new(),
            })
            .disposition,
            ResumeDisposition::ExecuteCold
        );
    }
}

#[test]
fn modified_payload_and_bundle_identity_are_rejected() {
    let mut payload_changed = candidate(ResumeStage::StagexTransition);
    payload_changed.observed_payloads[0].digest_blake3 = DIGEST_A.into();
    let mut identity_changed = candidate(ResumeStage::StagexTransition);
    identity_changed.manifest.bundle_identity_blake3 = DIGEST_A.into();

    assert_eq!(validate_resume_candidate(&payload_changed), Err(ResumeRejectReason::PayloadMismatch));
    assert_eq!(validate_resume_candidate(&identity_changed), Err(ResumeRejectReason::BundleIdentityMismatch));
}

#[test]
fn malformed_rejected_identity_is_normalized_for_machine_reporting() {
    let mut malformed = candidate(ResumeStage::StagexTransition);
    malformed.manifest.bundle_identity_blake3 = "malformed".into();
    let plan = plan_resume(&ResumePlanInput {
        mode: ResumeRunMode::Dev,
        candidates: vec![malformed],
        observed_rejections: Vec::new(),
    });
    let reported = &plan.rejected_candidates[0].bundle_identity_blake3;

    assert_eq!(reported.len(), usize::try_from(BLAKE3_HEX_LENGTH_CHARS).unwrap());
    assert!(reported.bytes().all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase()));
}

#[test]
fn missing_required_payload_and_unknown_stage_fail_before_selection() {
    let mut missing = manifest(ResumeStage::StagexTransition);
    missing.payloads.clear();
    missing.bundle_identity_blake3.clear();
    let unknown_json = r#"{
        "schema":"mantle-dev-stage-resume-bundle-v1",
        "bundle_identity_blake3":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        "source_authority_digest_blake3":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        "plan_digest_blake3":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        "policy_digest_blake3":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        "completed_stage":{
            "stage":"unknown-stage",
            "stage_id":"unknown-stage",
            "producer_executable_digest_blake3":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "output_digest_blake3":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "execution_evidence_digest_blake3":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
        },
        "payloads":[]
    }"#;

    assert!(seal_resume_bundle(missing).is_err());
    assert!(serde_json::from_str::<ResumeBundleManifest>(unknown_json).is_err());
}

#[test]
fn dev_resume_report_rejects_unknown_fields() {
    let value = serde_json::json!({
        "schema": DEV_RESUME_REPORT_SCHEMA,
        "status": "dev-only",
        "mode": "dev",
        "plan_digest_blake3": DIGEST_A,
        "selected_bundle_identity_blake3": null,
        "published_bundle_identities_blake3": [],
        "restored_stages": [],
        "executed_stages": ["stagex-transition"],
        "first_incomplete_stage": "stagex-transition",
        "rejected_candidates": [],
        "cache_adoption_disposition": "cold-executed",
        "promoted_receipt_written": false,
        "release_alias_updated": false,
        "non_claims": ["bounded", "dev-only", "not-promoted"],
        "secret": "reject"
    });

    assert!(serde_json::from_value::<DevResumeReport>(value).is_err());
}

#[test]
fn promoted_mode_ignores_a_valid_dev_candidate() {
    let plan = plan_resume(&ResumePlanInput {
        mode: ResumeRunMode::Promoted,
        candidates: vec![candidate(ResumeStage::MantleStage2)],
        observed_rejections: Vec::new(),
    });

    assert_eq!(plan.disposition, ResumeDisposition::ExecuteCold);
    assert_eq!(plan.cold_reason, Some(ResumeRejectReason::PromotedMode));
    assert!(plan.restored_stages.is_empty());
    assert_eq!(plan.executed_stages, ResumeStage::ALL);
}

#[test]
fn conflicting_same_stage_candidates_do_not_authorize_restore() {
    let first = candidate(ResumeStage::StagexTransition);
    let mut second = candidate(ResumeStage::StagexTransition);
    second.manifest.payloads[0].digest_blake3 = DIGEST_B.into();
    second.observed_payloads = second.manifest.payloads.clone();
    second.manifest = seal_resume_bundle(second.manifest).unwrap();
    let plan = plan_resume(&ResumePlanInput {
        mode: ResumeRunMode::Dev,
        candidates: vec![first, second],
        observed_rejections: Vec::new(),
    });

    assert_eq!(plan.disposition, ResumeDisposition::ExecuteCold);
    assert_eq!(plan.cold_reason, Some(ResumeRejectReason::NoCandidate));
    assert_eq!(plan.rejected_candidates.len(), 2);
    assert_eq!(plan.executed_stages, ResumeStage::ALL);
}

#[test]
fn shell_observed_invalid_identity_is_preserved_in_the_cold_plan() {
    let rejected = RejectedResumeCandidate {
        bundle_identity_blake3: DIGEST_A.into(),
        completed_stage: ResumeStage::StagexTransition,
        reason: ResumeRejectReason::ManifestInvalid,
    };
    let plan = plan_resume(&ResumePlanInput {
        mode: ResumeRunMode::Dev,
        candidates: Vec::new(),
        observed_rejections: vec![rejected.clone()],
    });

    assert_eq!(plan.disposition, ResumeDisposition::ExecuteCold);
    assert_eq!(plan.rejected_candidates, vec![rejected]);
}

#[test]
fn candidate_bound_rejects_unbounded_input() {
    let candidates = (0..=DEV_RESUME_CANDIDATES_MAX).map(|_| candidate(ResumeStage::StagexTransition)).collect();
    let plan = plan_resume(&ResumePlanInput {
        mode: ResumeRunMode::Dev,
        candidates,
        observed_rejections: Vec::new(),
    });

    assert_eq!(plan.disposition, ResumeDisposition::ExecuteCold);
    assert_eq!(plan.cold_reason, Some(ResumeRejectReason::CandidateLimitExceeded));
}
