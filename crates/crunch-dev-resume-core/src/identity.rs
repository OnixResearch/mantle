use alloc::string::String;
use alloc::string::ToString;

use crate::DEV_RESUME_BUNDLE_SCHEMA;
use crate::ResumeBundleManifest;
use crate::ResumeError;
use crate::ResumePolicyDigests;
use crate::validation::validate_resume_manifest_material;

const BUNDLE_IDENTITY_CONTEXT: &str = "mantle-dev-stage-resume-bundle-identity-v1";
const POLICY_IDENTITY_CONTEXT: &str = "mantle-dev-stage-resume-policy-identity-v1";
const HEX_CHARS_PER_BYTE: usize = 2;

pub fn compute_resume_policy_identity(policies: &ResumePolicyDigests) -> Result<String, ResumeError> {
    let mut hasher = blake3::Hasher::new_derive_key(POLICY_IDENTITY_CONTEXT);
    for digest in [
        &policies.closure_policy_digest_blake3,
        &policies.hermeticity_policy_digest_blake3,
        &policies.protected_execution_policy_digest_blake3,
        &policies.effect_policy_digest_blake3,
        &policies.normalization_policy_digest_blake3,
    ] {
        crate::validation::validate_identity_digest(digest)?;
        frame_text(&mut hasher, digest)?;
    }
    let identity = hasher.finalize().to_hex().to_string();
    debug_assert!(!policies.closure_policy_digest_blake3.is_empty());
    debug_assert_eq!(identity.len(), blake3::OUT_LEN.saturating_mul(HEX_CHARS_PER_BYTE));
    Ok(identity)
}

pub fn seal_resume_bundle(mut manifest: ResumeBundleManifest) -> Result<ResumeBundleManifest, ResumeError> {
    manifest.schema = DEV_RESUME_BUNDLE_SCHEMA.to_string();
    manifest.bundle_identity_blake3.clear();
    validate_resume_manifest_material(&manifest)?;
    manifest.bundle_identity_blake3 = compute_resume_bundle_identity(&manifest)?;
    debug_assert!(!manifest.bundle_identity_blake3.is_empty());
    debug_assert_eq!(manifest.schema, DEV_RESUME_BUNDLE_SCHEMA);
    Ok(manifest)
}

pub fn compute_resume_bundle_identity(manifest: &ResumeBundleManifest) -> Result<String, ResumeError> {
    validate_resume_manifest_material(manifest)?;
    let mut hasher = blake3::Hasher::new_derive_key(BUNDLE_IDENTITY_CONTEXT);
    frame_text(&mut hasher, &manifest.schema)?;
    frame_text(&mut hasher, &manifest.source_authority_digest_blake3)?;
    frame_text(&mut hasher, &manifest.plan_digest_blake3)?;
    frame_text(&mut hasher, &manifest.policy_digest_blake3)?;
    hasher.update(&[manifest.completed_stage.stage.code()]);
    frame_text(&mut hasher, &manifest.completed_stage.stage_id)?;
    frame_text(&mut hasher, &manifest.completed_stage.producer_executable_digest_blake3)?;
    frame_text(&mut hasher, &manifest.completed_stage.output_digest_blake3)?;
    frame_text(&mut hasher, &manifest.completed_stage.execution_evidence_digest_blake3)?;
    let payload_count = u32::try_from(manifest.payloads.len())
        .map_err(|_| ResumeError::new("resume payload count does not fit u32"))?;
    hasher.update(&payload_count.to_le_bytes());
    for payload in &manifest.payloads {
        frame_text(&mut hasher, &payload.payload_id)?;
        frame_text(&mut hasher, &payload.destination_relative_path)?;
        hasher.update(&[payload.kind.code()]);
        frame_text(&mut hasher, &payload.digest_blake3)?;
        hasher.update(&payload.total_file_bytes.to_le_bytes());
        hasher.update(&payload.entry_count.to_le_bytes());
    }
    let identity = hasher.finalize().to_hex().to_string();
    debug_assert!(!manifest.payloads.is_empty());
    debug_assert_eq!(identity.len(), blake3::OUT_LEN.saturating_mul(HEX_CHARS_PER_BYTE));
    Ok(identity)
}

fn frame_text(hasher: &mut blake3::Hasher, value: &str) -> Result<(), ResumeError> {
    let length_bytes =
        u64::try_from(value.len()).map_err(|_| ResumeError::new("resume text length does not fit u64"))?;
    hasher.update(&length_bytes.to_le_bytes());
    hasher.update(value.as_bytes());
    Ok(())
}
