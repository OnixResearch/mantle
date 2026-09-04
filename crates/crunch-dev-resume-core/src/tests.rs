mod negative;
mod positive;

use alloc::format;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec;

use crate::*;

pub(super) const DIGEST_A: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
pub(super) const DIGEST_B: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
pub(super) const DIGEST_C: &str = "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc";

pub(super) fn payload(payload_id: &str) -> ResumePayloadBinding {
    ResumePayloadBinding {
        payload_id: payload_id.to_string(),
        destination_relative_path: format!("payload/{payload_id}"),
        kind: ResumePayloadKind::ContentReference,
        digest_blake3: DIGEST_C.to_string(),
        total_file_bytes: 1,
        entry_count: 1,
    }
}

pub(super) fn manifest(stage: ResumeStage) -> ResumeBundleManifest {
    let mut payloads = vec![payload(PROVIDER_CHECKPOINT_PAYLOAD_ID)];
    match stage {
        ResumeStage::MantleStage1 => payloads.push(payload(MANTLE_STAGE1_PAYLOAD_ID)),
        ResumeStage::MantleStage2 => payloads.push(payload(FIXED_POINT_COMPLETE_PAYLOAD_ID)),
        ResumeStage::StagexTransition
        | ResumeStage::StagexProvider
        | ResumeStage::FullSourceNativeProvider
        | ResumeStage::FullSourceRustProvider => {}
    }
    payloads.sort_by(|left, right| left.payload_id.cmp(&right.payload_id));
    seal_resume_bundle(ResumeBundleManifest {
        schema: String::new(),
        bundle_identity_blake3: String::new(),
        source_authority_digest_blake3: DIGEST_A.to_string(),
        plan_digest_blake3: DIGEST_B.to_string(),
        policy_digest_blake3: DIGEST_C.to_string(),
        completed_stage: ResumeStageBinding {
            stage,
            stage_id: stage.id().to_string(),
            producer_executable_digest_blake3: DIGEST_A.to_string(),
            output_digest_blake3: DIGEST_B.to_string(),
            execution_evidence_digest_blake3: DIGEST_C.to_string(),
        },
        payloads,
    })
    .unwrap()
}

pub(super) fn candidate(stage: ResumeStage) -> ResumeCandidateObservation {
    let manifest = manifest(stage);
    ResumeCandidateObservation {
        current_source_authority_digest_blake3: manifest.source_authority_digest_blake3.clone(),
        current_plan_digest_blake3: manifest.plan_digest_blake3.clone(),
        current_policy_digest_blake3: manifest.policy_digest_blake3.clone(),
        current_stage: manifest.completed_stage.clone(),
        observed_payloads: manifest.payloads.clone(),
        manifest,
    }
}
