use alloc::string::ToString;
use alloc::vec::Vec;

use crate::DEV_RESUME_CANDIDATES_MAX;
use crate::RejectedResumeCandidate;
use crate::ResumeCandidateObservation;
use crate::ResumeDisposition;
use crate::ResumePlan;
use crate::ResumePlanInput;
use crate::ResumeRejectReason;
use crate::ResumeRunMode;
use crate::ResumeStage;
use crate::model::DEV_RESUME_CANDIDATES_MAX_USIZE;
use crate::model::DEV_RESUME_STAGE_COUNT_USIZE;
use crate::validate_resume_candidate;

const STAGE_COUNT: usize = DEV_RESUME_STAGE_COUNT_USIZE;

// r[impl source_built_fixed_point_improved_iteration.dev_cross_run_resume]
pub fn plan_resume(input: &ResumePlanInput) -> ResumePlan {
    if input.mode == ResumeRunMode::Promoted {
        return cold_plan(ResumeRejectReason::PromotedMode, bounded_rejections(&input.observed_rejections));
    }
    let Some(total_count) = input.candidates.len().checked_add(input.observed_rejections.len()) else {
        return cold_plan(ResumeRejectReason::CandidateLimitExceeded, Vec::new());
    };
    let Ok(candidate_count) = u32::try_from(total_count) else {
        return cold_plan(ResumeRejectReason::CandidateLimitExceeded, Vec::new());
    };
    if candidate_count > DEV_RESUME_CANDIDATES_MAX {
        return cold_plan(ResumeRejectReason::CandidateLimitExceeded, bounded_rejections(&input.observed_rejections));
    }
    let (selected, rejected) = select_candidates(&input.candidates, &input.observed_rejections);
    let Some(candidate) = highest_candidate(&selected) else {
        return cold_plan(ResumeRejectReason::NoCandidate, rejected);
    };
    let completed_stage = candidate.manifest.completed_stage.stage;
    let completed_index = completed_stage.index();
    let restored_stages = ResumeStage::ALL.iter().copied().take(completed_index.saturating_add(1)).collect::<Vec<_>>();
    let executed_stages = ResumeStage::ALL.iter().copied().skip(completed_index.saturating_add(1)).collect::<Vec<_>>();
    let plan = ResumePlan {
        disposition: ResumeDisposition::Restore,
        selected_bundle_identity_blake3: Some(candidate.manifest.bundle_identity_blake3.clone()),
        completed_stage: Some(completed_stage),
        restored_stages,
        first_incomplete_stage: executed_stages.first().copied(),
        executed_stages,
        cold_reason: None,
        rejected_candidates: rejected,
    };
    debug_assert_eq!(plan.restored_stages.last().copied(), Some(completed_stage));
    debug_assert_eq!(plan.first_incomplete_stage, plan.executed_stages.first().copied());
    plan
}

fn bounded_rejections(rejections: &[RejectedResumeCandidate]) -> Vec<RejectedResumeCandidate> {
    let result = rejections
        .iter()
        .take(DEV_RESUME_CANDIDATES_MAX_USIZE)
        .cloned()
        .map(normalize_rejected_candidate)
        .collect::<Vec<_>>();
    debug_assert!(result.len() <= DEV_RESUME_CANDIDATES_MAX_USIZE);
    result
}

fn select_candidates<'a>(
    candidates: &'a [ResumeCandidateObservation],
    observed_rejections: &[RejectedResumeCandidate],
) -> ([Option<&'a ResumeCandidateObservation>; STAGE_COUNT], Vec<RejectedResumeCandidate>) {
    let mut selected = [None; STAGE_COUNT];
    let mut conflicted = [false; STAGE_COUNT];
    let rejected_items_max = candidates.len().saturating_add(observed_rejections.len());
    let mut rejected = Vec::with_capacity(rejected_items_max);
    rejected.extend_from_slice(observed_rejections);
    for candidate in candidates {
        let stage = candidate.manifest.completed_stage.stage;
        let stage_index = stage.index();
        match validate_resume_candidate(candidate) {
            Ok(()) if !conflicted[stage_index] && selected[stage_index].is_none() => {
                selected[stage_index] = Some(candidate);
            }
            Ok(()) if selected[stage_index].is_some() => {
                let previous = selected[stage_index].take();
                conflicted[stage_index] = true;
                if let Some(previous) = previous {
                    rejected.push(rejected_candidate(previous, ResumeRejectReason::ConflictingCandidate));
                }
                rejected.push(rejected_candidate(candidate, ResumeRejectReason::ConflictingCandidate));
            }
            Ok(()) => {
                rejected.push(rejected_candidate(candidate, ResumeRejectReason::ConflictingCandidate));
            }
            Err(reason) => rejected.push(rejected_candidate(candidate, reason)),
        }
    }
    debug_assert_eq!(selected.len(), STAGE_COUNT);
    debug_assert!(rejected.len() <= rejected_items_max);
    (selected, rejected)
}

fn highest_candidate<'a>(
    selected: &'a [Option<&'a ResumeCandidateObservation>; STAGE_COUNT],
) -> Option<&'a ResumeCandidateObservation> {
    selected.iter().rev().find_map(|candidate| *candidate)
}

fn rejected_candidate(candidate: &ResumeCandidateObservation, reason: ResumeRejectReason) -> RejectedResumeCandidate {
    normalize_rejected_candidate(RejectedResumeCandidate {
        bundle_identity_blake3: candidate.manifest.bundle_identity_blake3.clone(),
        completed_stage: candidate.manifest.completed_stage.stage,
        reason,
    })
}

fn normalize_rejected_candidate(mut rejected: RejectedResumeCandidate) -> RejectedResumeCandidate {
    if !is_lower_blake3(&rejected.bundle_identity_blake3) {
        rejected.bundle_identity_blake3 = blake3::hash(rejected.bundle_identity_blake3.as_bytes()).to_hex().to_string();
    }
    debug_assert!(is_lower_blake3(&rejected.bundle_identity_blake3));
    rejected
}

fn is_lower_blake3(value: &str) -> bool {
    value.len() == blake3::OUT_LEN.saturating_mul(2)
        && value.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn cold_plan(reason: ResumeRejectReason, rejected_candidates: Vec<RejectedResumeCandidate>) -> ResumePlan {
    let executed_stages = ResumeStage::ALL.to_vec();
    let plan = ResumePlan {
        disposition: ResumeDisposition::ExecuteCold,
        selected_bundle_identity_blake3: None,
        completed_stage: None,
        restored_stages: Vec::new(),
        first_incomplete_stage: executed_stages.first().copied(),
        executed_stages,
        cold_reason: Some(reason),
        rejected_candidates,
    };
    debug_assert!(plan.restored_stages.is_empty());
    debug_assert_eq!(plan.executed_stages.len(), STAGE_COUNT);
    plan
}
