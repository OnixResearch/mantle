//! Pure cross-run resume planning for the source-built fixed-point dev cache.
//!
//! Resume is a pure decision over published stage bundles and the current
//! plan: a stage may be restored only when its bundle revalidates against the
//! exact plan, source-authority, policy, producer, and output identities. The
//! first stage that fails to revalidate, and every stage after it, must
//! execute. Promoted-path bundles are never restored.

use crate::source_built_fixed_point_dev_cache::DevCachePolicies;

pub(crate) const RESUME_BUNDLE_SCHEMA: &str = "mantle-source-built-fixed-point-resume-bundle-v1";
pub(crate) const RESUME_PLAN_SCHEMA: &str = "mantle-source-built-fixed-point-resume-plan-v1";

/// Maximum admitted stages in one resume plan.
pub(crate) const MAX_RESUME_STAGES: u32 = 64;

/// Maximum admitted required outputs per stage bundle.
pub(crate) const MAX_RESUME_OUTPUTS_PER_STAGE: u32 = 64;

const BLAKE3_HEX_LENGTH: usize = 64;

/// One required output recorded in a published stage bundle.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct ResumeOutputReference {
    pub(crate) role: String,
    pub(crate) digest_blake3: String,
}

/// One published content-addressed stage bundle as observed from disk.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct StageBundleReference {
    pub(crate) schema: String,
    pub(crate) plan_digest_blake3: String,
    pub(crate) stage_id: String,
    pub(crate) source_authority_digest_blake3: String,
    pub(crate) producer_identity_digest_blake3: String,
    /// Digest over the policy cohort the bundle was produced under.
    pub(crate) policy_cohort_digest_blake3: String,
    /// Content address recorded inside the bundle.
    pub(crate) bundle_digest_blake3: String,
    /// Content address recomputed from the bytes found on disk.
    pub(crate) recomputed_digest_blake3: String,
    pub(crate) required_outputs: Vec<ResumeOutputReference>,
    /// Absolute promoted alias observed beside the bundle, if any.
    pub(crate) promoted_path: Option<String>,
}

/// Why one stage bundle cannot authorize a restore.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct ResumeBlocker {
    pub(crate) code: String,
    pub(crate) stage_id: String,
    pub(crate) detail: String,
}

impl ResumeBlocker {
    fn new(code: &str, stage_id: &str, detail: &str) -> Self {
        debug_assert!(!code.is_empty() && !detail.is_empty());
        Self {
            code: code.to_string(),
            stage_id: stage_id.to_string(),
            detail: detail.to_string(),
        }
    }
}

/// One stage decision inside a resume plan.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct ResumeStageDecision {
    pub(crate) stage_id: String,
    /// Content address of the restored bundle, when this stage is restored.
    pub(crate) restored_bundle_digest_blake3: Option<String>,
}

/// Resume plan: restored prefix plus the executed remainder.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct ResumePlan {
    pub(crate) schema: String,
    pub(crate) plan_digest_blake3: String,
    /// Stages restored from validated bundles, in plan order.
    pub(crate) restored_stages: Vec<ResumeStageDecision>,
    /// Stages that must execute, in plan order.
    pub(crate) executed_stages: Vec<String>,
    pub(crate) blockers: Vec<ResumeBlocker>,
}

/// Validate one bundle against the current plan and add any blocker.
fn validate_bundle(
    reference: &StageBundleReference,
    plan_digest_blake3: &str,
    source_authority_digest_blake3: &str,
    policies: &DevCachePolicies,
    expected_producer_identity_digest_blake3: &str,
    blockers: &mut Vec<ResumeBlocker>,
) {
    let stage_id = reference.stage_id.as_str();
    if reference.schema != RESUME_BUNDLE_SCHEMA {
        blockers.push(ResumeBlocker::new("resume-unknown-schema", stage_id, "bundle schema is unsupported"));
    }
    if reference.plan_digest_blake3 != plan_digest_blake3 {
        blockers.push(ResumeBlocker::new("resume-stale-plan", stage_id, "bundle belongs to another plan"));
    }
    if reference.source_authority_digest_blake3 != source_authority_digest_blake3 {
        blockers.push(ResumeBlocker::new(
            "resume-stale-source",
            stage_id,
            "bundle belongs to another source authority",
        ));
    }
    if reference.producer_identity_digest_blake3 != expected_producer_identity_digest_blake3 {
        blockers.push(ResumeBlocker::new(
            "resume-producer-mismatch",
            stage_id,
            "bundle records a different producer identity",
        ));
    }
    if reference.promoted_path.is_some() {
        blockers.push(ResumeBlocker::new(
            "resume-promoted-path",
            stage_id,
            "promoted aliases never authorize a dev restore",
        ));
    }
    if !is_lower_hex_digest(&reference.bundle_digest_blake3)
        || !is_lower_hex_digest(&reference.recomputed_digest_blake3)
    {
        blockers.push(ResumeBlocker::new(
            "resume-malformed-digest",
            stage_id,
            "bundle digests must be lowercase blake3 hex",
        ));
    } else if reference.bundle_digest_blake3 != reference.recomputed_digest_blake3 {
        blockers.push(ResumeBlocker::new(
            "resume-mutated-bundle",
            stage_id,
            "recorded bundle digest differs from the bytes on disk",
        ));
    }
    if reference.required_outputs.is_empty() {
        blockers.push(ResumeBlocker::new("resume-missing-outputs", stage_id, "bundle records no required outputs"));
    }
    if u32::try_from(reference.required_outputs.len()).map_or(true, |count| count > MAX_RESUME_OUTPUTS_PER_STAGE) {
        blockers.push(ResumeBlocker::new("resume-output-limit", stage_id, "bundle exceeds the required-output bound"));
    }
    for output in &reference.required_outputs {
        if output.role.is_empty() || !is_lower_hex_digest(&output.digest_blake3) {
            blockers.push(ResumeBlocker::new(
                "resume-partial-outputs",
                stage_id,
                "bundle records an incomplete or malformed required output",
            ));
            break;
        }
    }
    let expected_policy_digest = policy_cohort_digest(policies);
    if reference.policy_cohort_digest_blake3 != expected_policy_digest {
        blockers.push(ResumeBlocker::new(
            "resume-stale-policy",
            stage_id,
            "bundle was produced under a different policy cohort",
        ));
    }
}

/// Digest over the current policy cohort, used to reject policy drift.
pub(crate) fn policy_cohort_digest(policies: &DevCachePolicies) -> String {
    const POLICY_COHORT_CONTEXT: &str = "mantle-source-built-fixed-point-resume-policy-v1";
    let mut hasher = blake3::Hasher::new_derive_key(POLICY_COHORT_CONTEXT);
    for digest in policies.ordered_digests() {
        debug_assert!(is_lower_hex_digest(digest));
        hasher.update(digest.as_bytes());
        hasher.update(b"\n");
    }
    let cohort = hasher.finalize().to_hex().to_string();
    debug_assert_eq!(cohort.len(), BLAKE3_HEX_LENGTH);
    cohort
}

/// Plan a resume over the plan's stage order and the bundles found on disk.
///
/// Stages restore only while every earlier bundle revalidates; the first
/// invalid or missing stage and every stage after it execute.
pub(crate) fn plan_stage_resume(
    plan_digest_blake3: &str,
    source_authority_digest_blake3: &str,
    stage_ids: &[String],
    expected_producer_identity_digest_blake3: &str,
    policies: &DevCachePolicies,
    bundles: &[StageBundleReference],
) -> ResumePlan {
    let mut blockers: Vec<ResumeBlocker> = Vec::new();
    if u32::try_from(stage_ids.len()).map_or(true, |count| count > MAX_RESUME_STAGES) {
        blockers.push(ResumeBlocker::new("resume-stage-limit", "plan", "stage list exceeds the resume bound"));
    }
    let mut restored_stages: Vec<ResumeStageDecision> = Vec::new();
    let mut executed_stages: Vec<String> = Vec::with_capacity(stage_ids.len());
    let mut resume_is_closed = false;
    for stage_id in stage_ids {
        let matching = bundles.iter().filter(|reference| &reference.stage_id == stage_id).collect::<Vec<_>>();
        let mut stage_blockers: Vec<ResumeBlocker> = Vec::new();
        match matching.as_slice() {
            [reference] => validate_bundle(
                reference,
                plan_digest_blake3,
                source_authority_digest_blake3,
                policies,
                expected_producer_identity_digest_blake3,
                &mut stage_blockers,
            ),
            [] => stage_blockers.push(ResumeBlocker::new(
                "resume-missing-bundle",
                stage_id,
                "no published bundle exists for this stage",
            )),
            _ => stage_blockers.push(ResumeBlocker::new(
                "resume-duplicate-bundle",
                stage_id,
                "more than one bundle claims this stage",
            )),
        }
        if resume_is_closed || !stage_blockers.is_empty() {
            resume_is_closed = true;
            blockers.extend(stage_blockers);
            executed_stages.push(stage_id.clone());
            continue;
        }
        let reference = matching[0];
        restored_stages.push(ResumeStageDecision {
            stage_id: stage_id.clone(),
            restored_bundle_digest_blake3: Some(reference.bundle_digest_blake3.clone()),
        });
    }
    for bundle in bundles {
        if !stage_ids.iter().any(|stage_id| stage_id == &bundle.stage_id) {
            blockers.push(ResumeBlocker::new(
                "resume-unknown-stage",
                &bundle.stage_id,
                "bundle names a stage outside the current plan",
            ));
        }
    }
    blockers.sort_by(|left, right| (&left.code, &left.stage_id).cmp(&(&right.code, &right.stage_id)));
    debug_assert_eq!(restored_stages.len() + executed_stages.len(), stage_ids.len());
    ResumePlan {
        schema: RESUME_PLAN_SCHEMA.to_string(),
        plan_digest_blake3: plan_digest_blake3.to_string(),
        restored_stages,
        executed_stages,
        blockers,
    }
}

fn is_lower_hex_digest(value: &str) -> bool {
    value.len() == BLAKE3_HEX_LENGTH && value.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

#[cfg(test)]
mod tests {
    use super::*;

    const DIGEST: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    const OTHER_DIGEST: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";

    fn policies() -> DevCachePolicies {
        DevCachePolicies {
            closure_policy_digest_blake3: DIGEST.to_string(),
            hermeticity_policy_digest_blake3: DIGEST.to_string(),
            protected_execution_policy_digest_blake3: DIGEST.to_string(),
            effect_policy_digest_blake3: DIGEST.to_string(),
            normalization_policy_digest_blake3: DIGEST.to_string(),
        }
    }

    fn stage_ids() -> Vec<String> {
        vec![
            "stagex-transition".to_string(),
            "native-provider".to_string(),
            "rust-provider".to_string(),
        ]
    }

    fn bundle(stage_id: &str) -> StageBundleReference {
        StageBundleReference {
            schema: RESUME_BUNDLE_SCHEMA.to_string(),
            plan_digest_blake3: DIGEST.to_string(),
            stage_id: stage_id.to_string(),
            source_authority_digest_blake3: DIGEST.to_string(),
            producer_identity_digest_blake3: DIGEST.to_string(),
            policy_cohort_digest_blake3: policy_cohort_digest(&policies()),
            bundle_digest_blake3: DIGEST.to_string(),
            recomputed_digest_blake3: DIGEST.to_string(),
            required_outputs: vec![ResumeOutputReference {
                role: "transition-execution-tree".to_string(),
                digest_blake3: DIGEST.to_string(),
            }],
            promoted_path: None,
        }
    }

    fn plan_with(bundles: &[StageBundleReference]) -> ResumePlan {
        plan_stage_resume(DIGEST, DIGEST, &stage_ids(), DIGEST, &policies(), bundles)
    }

    #[test]
    fn a_fresh_directory_restores_every_validated_stage() {
        let bundles = stage_ids().iter().map(|stage_id| bundle(stage_id)).collect::<Vec<_>>();
        let plan = plan_with(&bundles);
        assert!(plan.blockers.is_empty());
        assert_eq!(plan.restored_stages.len(), stage_ids().len());
        assert!(plan.executed_stages.is_empty());
    }

    #[test]
    fn the_first_invalid_stage_and_every_later_stage_execute() {
        let bundles = vec![bundle("stagex-transition"), bundle("rust-provider")];
        let plan = plan_with(&bundles);
        assert_eq!(plan.restored_stages.len(), 1);
        assert_eq!(plan.restored_stages[0].stage_id, "stagex-transition");
        assert_eq!(plan.executed_stages, vec!["native-provider", "rust-provider"]);
        assert!(plan.blockers.iter().any(|blocker| blocker.code == "resume-missing-bundle"));
        assert!(
            plan.blockers
                .iter()
                .any(|blocker| blocker.code == "resume-missing-bundle" && blocker.stage_id == "native-provider")
        );
    }

    #[test]
    fn stale_plan_source_and_policy_bundles_are_rejected() {
        let mut stale_plan = bundle("stagex-transition");
        stale_plan.plan_digest_blake3 = OTHER_DIGEST.to_string();
        let plan = plan_with(&[stale_plan, bundle("native-provider"), bundle("rust-provider")]);
        assert!(plan.blockers.iter().any(|blocker| blocker.code == "resume-stale-plan"));
        assert!(plan.restored_stages.is_empty());
        assert_eq!(plan.executed_stages.len(), 3);

        let mut stale_source = bundle("stagex-transition");
        stale_source.source_authority_digest_blake3 = OTHER_DIGEST.to_string();
        let plan = plan_with(&[stale_source, bundle("native-provider"), bundle("rust-provider")]);
        assert!(plan.blockers.iter().any(|blocker| blocker.code == "resume-stale-source"));
    }

    #[test]
    fn mutated_partial_and_malformed_bundles_are_rejected() {
        let mut mutated = bundle("stagex-transition");
        mutated.recomputed_digest_blake3 = OTHER_DIGEST.to_string();
        let plan = plan_with(&[mutated, bundle("native-provider"), bundle("rust-provider")]);
        assert!(plan.blockers.iter().any(|blocker| blocker.code == "resume-mutated-bundle"));

        let mut partial = bundle("stagex-transition");
        partial.required_outputs = Vec::new();
        let plan = plan_with(&[partial, bundle("native-provider"), bundle("rust-provider")]);
        assert!(plan.blockers.iter().any(|blocker| blocker.code == "resume-missing-outputs"));

        let mut malformed = bundle("stagex-transition");
        malformed.required_outputs = vec![ResumeOutputReference {
            role: String::new(),
            digest_blake3: OTHER_DIGEST.to_string(),
        }];
        let plan = plan_with(&[malformed, bundle("native-provider"), bundle("rust-provider")]);
        assert!(plan.blockers.iter().any(|blocker| blocker.code == "resume-partial-outputs"));

        let mut bad_digest = bundle("stagex-transition");
        bad_digest.bundle_digest_blake3 = String::from("not-hex");
        let plan = plan_with(&[bad_digest, bundle("native-provider"), bundle("rust-provider")]);
        assert!(plan.blockers.iter().any(|blocker| blocker.code == "resume-malformed-digest"));
    }

    #[test]
    fn promoted_paths_and_unknown_stages_are_rejected() {
        let mut promoted = bundle("stagex-transition");
        promoted.promoted_path = Some("/mantle/store/promoted-stagex".to_string());
        let plan = plan_with(&[promoted, bundle("native-provider"), bundle("rust-provider")]);
        assert!(plan.blockers.iter().any(|blocker| blocker.code == "resume-promoted-path"));
        assert_eq!(plan.restored_stages.len(), 0);

        let mut foreign = bundle("stagex-transition");
        foreign.stage_id = "not-in-plan".to_string();
        let plan = plan_with(&[bundle("stagex-transition"), bundle("native-provider"), foreign]);
        assert!(plan.blockers.iter().any(|blocker| blocker.code == "resume-unknown-stage"));
    }

    #[test]
    fn policy_cohort_drift_is_rejected() {
        let mut drifted = bundle("stagex-transition");
        drifted.policy_cohort_digest_blake3 = OTHER_DIGEST.to_string();
        let plan = plan_with(&[drifted, bundle("native-provider"), bundle("rust-provider")]);
        assert!(plan.blockers.iter().any(|blocker| blocker.code == "resume-stale-policy"));
        assert!(plan.restored_stages.is_empty());
    }

    #[test]
    fn duplicate_and_producer_mismatched_bundles_are_rejected() {
        let plan = plan_with(&[
            bundle("stagex-transition"),
            bundle("stagex-transition"),
            bundle("native-provider"),
            bundle("rust-provider"),
        ]);
        assert!(plan.blockers.iter().any(|blocker| blocker.code == "resume-duplicate-bundle"));

        let mut wrong_producer = bundle("stagex-transition");
        wrong_producer.producer_identity_digest_blake3 = OTHER_DIGEST.to_string();
        let plan = plan_with(&[wrong_producer, bundle("native-provider"), bundle("rust-provider")]);
        assert!(plan.blockers.iter().any(|blocker| blocker.code == "resume-producer-mismatch"));
    }
}
