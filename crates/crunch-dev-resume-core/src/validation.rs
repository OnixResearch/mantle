use crate::BLAKE3_HEX_LENGTH_CHARS;
use crate::DEV_RESUME_BUNDLE_SCHEMA;
use crate::DEV_RESUME_PAYLOADS_MAX;
use crate::DEV_RESUME_TEXT_BYTES_MAX;
use crate::FIXED_POINT_COMPLETE_PAYLOAD_ID;
use crate::MANTLE_STAGE1_PAYLOAD_ID;
use crate::PROVIDER_CHECKPOINT_PAYLOAD_ID;
use crate::ResumeBundleManifest;
use crate::ResumeCandidateObservation;
use crate::ResumeError;
use crate::ResumePayloadBinding;
use crate::ResumeRejectReason;
use crate::ResumeStage;
use crate::ResumeStageBinding;
use crate::compute_resume_bundle_identity;

pub fn validate_resume_manifest(manifest: &ResumeBundleManifest) -> Result<(), ResumeError> {
    validate_resume_manifest_material(manifest)?;
    validate_identity_digest(&manifest.bundle_identity_blake3)?;
    let expected = compute_resume_bundle_identity(manifest)?;
    if manifest.bundle_identity_blake3 != expected {
        return Err(ResumeError::new("resume bundle identity mismatch"));
    }
    debug_assert_eq!(manifest.schema, DEV_RESUME_BUNDLE_SCHEMA);
    debug_assert!(!manifest.payloads.is_empty());
    Ok(())
}

pub(crate) fn validate_resume_manifest_material(manifest: &ResumeBundleManifest) -> Result<(), ResumeError> {
    if manifest.schema != DEV_RESUME_BUNDLE_SCHEMA {
        return Err(ResumeError::new("resume bundle schema mismatch"));
    }
    validate_identity_digest(&manifest.source_authority_digest_blake3)?;
    validate_identity_digest(&manifest.plan_digest_blake3)?;
    validate_identity_digest(&manifest.policy_digest_blake3)?;
    validate_stage_binding(&manifest.completed_stage)?;
    validate_payloads(&manifest.payloads)?;
    validate_required_payloads(manifest.completed_stage.stage, &manifest.payloads)?;
    debug_assert_eq!(manifest.completed_stage.stage_id, manifest.completed_stage.stage.id());
    debug_assert!(!manifest.payloads.is_empty());
    Ok(())
}

pub fn validate_resume_candidate(candidate: &ResumeCandidateObservation) -> Result<(), ResumeRejectReason> {
    validate_resume_manifest_material(&candidate.manifest).map_err(|_| ResumeRejectReason::ManifestInvalid)?;
    let expected_identity =
        compute_resume_bundle_identity(&candidate.manifest).map_err(|_| ResumeRejectReason::ManifestInvalid)?;
    if !is_digest(&candidate.manifest.bundle_identity_blake3)
        || candidate.manifest.bundle_identity_blake3 != expected_identity
    {
        return Err(ResumeRejectReason::BundleIdentityMismatch);
    }
    validate_candidate_authority(candidate)?;
    validate_payloads(&candidate.observed_payloads).map_err(|_| ResumeRejectReason::PayloadMismatch)?;
    if candidate.manifest.payloads != candidate.observed_payloads {
        return Err(ResumeRejectReason::PayloadMismatch);
    }
    debug_assert_eq!(candidate.manifest.completed_stage, candidate.current_stage);
    debug_assert_eq!(candidate.manifest.payloads, candidate.observed_payloads);
    Ok(())
}

fn validate_candidate_authority(candidate: &ResumeCandidateObservation) -> Result<(), ResumeRejectReason> {
    if candidate.manifest.source_authority_digest_blake3 != candidate.current_source_authority_digest_blake3 {
        return Err(ResumeRejectReason::SourceMismatch);
    }
    if candidate.manifest.plan_digest_blake3 != candidate.current_plan_digest_blake3 {
        return Err(ResumeRejectReason::PlanMismatch);
    }
    if candidate.manifest.policy_digest_blake3 != candidate.current_policy_digest_blake3 {
        return Err(ResumeRejectReason::PolicyMismatch);
    }
    let stored = &candidate.manifest.completed_stage;
    let current = &candidate.current_stage;
    if stored.stage != current.stage || stored.stage_id != current.stage_id {
        return Err(ResumeRejectReason::StageMismatch);
    }
    if stored.producer_executable_digest_blake3 != current.producer_executable_digest_blake3 {
        return Err(ResumeRejectReason::ProducerMismatch);
    }
    if stored.output_digest_blake3 != current.output_digest_blake3 {
        return Err(ResumeRejectReason::OutputMismatch);
    }
    if stored.execution_evidence_digest_blake3 != current.execution_evidence_digest_blake3 {
        return Err(ResumeRejectReason::ExecutionEvidenceMismatch);
    }
    debug_assert_eq!(stored.stage, current.stage);
    debug_assert_eq!(stored.stage_id, current.stage_id);
    Ok(())
}

fn validate_stage_binding(binding: &ResumeStageBinding) -> Result<(), ResumeError> {
    if binding.stage_id != binding.stage.id() {
        return Err(ResumeError::new("resume stage identifier mismatch"));
    }
    validate_text(&binding.stage_id)?;
    validate_identity_digest(&binding.producer_executable_digest_blake3)?;
    validate_identity_digest(&binding.output_digest_blake3)?;
    validate_identity_digest(&binding.execution_evidence_digest_blake3)?;
    Ok(())
}

fn validate_payloads(payloads: &[ResumePayloadBinding]) -> Result<(), ResumeError> {
    let payload_count =
        u32::try_from(payloads.len()).map_err(|_| ResumeError::new("resume payload count does not fit u32"))?;
    if payload_count == 0 || payload_count > DEV_RESUME_PAYLOADS_MAX {
        return Err(ResumeError::new("resume payload count is outside bounds"));
    }
    let mut previous_id = None;
    for payload in payloads {
        validate_text(&payload.payload_id)?;
        validate_relative_path(&payload.destination_relative_path)?;
        validate_identity_digest(&payload.digest_blake3)?;
        if payload.total_file_bytes == 0 || payload.entry_count == 0 {
            return Err(ResumeError::new("resume payload has empty bounds"));
        }
        if previous_id.is_some_and(|previous| previous >= payload.payload_id.as_str()) {
            return Err(ResumeError::new("resume payload identifiers are not strictly ordered"));
        }
        previous_id = Some(payload.payload_id.as_str());
    }
    debug_assert!(!payloads.is_empty());
    debug_assert!(payload_count <= DEV_RESUME_PAYLOADS_MAX);
    Ok(())
}

fn validate_required_payloads(stage: ResumeStage, payloads: &[ResumePayloadBinding]) -> Result<(), ResumeError> {
    require_payload(payloads, PROVIDER_CHECKPOINT_PAYLOAD_ID)?;
    match stage {
        ResumeStage::StagexTransition
        | ResumeStage::StagexProvider
        | ResumeStage::FullSourceNativeProvider
        | ResumeStage::FullSourceRustProvider => {}
        ResumeStage::MantleStage1 => require_payload(payloads, MANTLE_STAGE1_PAYLOAD_ID)?,
        ResumeStage::MantleStage2 => require_payload(payloads, FIXED_POINT_COMPLETE_PAYLOAD_ID)?,
    }
    debug_assert!(payloads.iter().any(|payload| payload.payload_id == PROVIDER_CHECKPOINT_PAYLOAD_ID));
    debug_assert!(!payloads.is_empty());
    Ok(())
}

fn require_payload(payloads: &[ResumePayloadBinding], required_id: &str) -> Result<(), ResumeError> {
    let match_count = payloads.iter().filter(|payload| payload.payload_id == required_id).count();
    if match_count != 1 {
        return Err(ResumeError::new("resume payload set is incomplete"));
    }
    Ok(())
}

fn validate_relative_path(value: &str) -> Result<(), ResumeError> {
    validate_text(value)?;
    if value.starts_with('/') || value.ends_with('/') {
        return Err(ResumeError::new("resume payload path is not relative"));
    }
    if value.split('/').any(|segment| segment.is_empty() || segment == "." || segment == "..") {
        return Err(ResumeError::new("resume payload path contains an unsafe segment"));
    }
    Ok(())
}

fn validate_text(value: &str) -> Result<(), ResumeError> {
    let length_bytes =
        u32::try_from(value.len()).map_err(|_| ResumeError::new("resume text length does not fit u32"))?;
    if length_bytes == 0 || length_bytes > DEV_RESUME_TEXT_BYTES_MAX || value.chars().any(char::is_control) {
        return Err(ResumeError::new("resume text is outside bounds"));
    }
    Ok(())
}

pub(crate) fn validate_identity_digest(value: &str) -> Result<(), ResumeError> {
    if !is_digest(value) {
        return Err(ResumeError::new("resume identity is not a lowercase BLAKE3 digest"));
    }
    Ok(())
}

fn is_digest(value: &str) -> bool {
    let Ok(length_chars) = u32::try_from(value.len()) else {
        return false;
    };
    length_chars == BLAKE3_HEX_LENGTH_CHARS
        && value.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}
