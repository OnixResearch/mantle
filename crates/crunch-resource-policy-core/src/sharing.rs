use alloc::string::String;
use alloc::string::ToString;

use crate::MAX_SHARING_SUBJECTS;
use crate::ProducerConsumerEvidenceLink;
use crate::RESOURCE_POLICY_NON_CLAIM;
use crate::RESULT_REUSE_DECISION_SCHEMA;
use crate::RESULT_SHARING_POLICY_SCHEMA;
use crate::REUSE_DOMAIN;
use crate::ResourceFeatureControls;
use crate::ResourcePolicyError;
use crate::ResourcePolicyMode;
use crate::ResultReuseDecision;
use crate::ResultReuseFacts;
use crate::ResultSharingPolicy;
use crate::ReuseReasonCode;
use crate::SharingScopeKind;
use crate::canonical_blake3;
use crate::canonical_strings;
use crate::validate_digest;
use crate::validate_id;

pub fn decide_result_reuse(
    mut policy: ResultSharingPolicy,
    facts: ResultReuseFacts,
    controls: ResourceFeatureControls,
) -> Result<ResultReuseDecision, ResourcePolicyError> {
    validate_reuse_facts(&facts)?;
    policy = canonical_sharing_policy(policy)?;
    let reason = first_reuse_rejection(&policy, &facts, controls).unwrap_or(ReuseReasonCode::Usable);
    let is_usable = reason == ReuseReasonCode::Usable;
    let evidence_link = is_usable.then(|| ProducerConsumerEvidenceLink {
        producer_result_blake3: facts.producer_result_blake3.clone(),
        consumer_request_blake3: facts.consumer_request_blake3.clone(),
        authorization_decision_blake3: facts.authorization_decision_blake3.clone(),
        sharing_policy_id: policy.policy_id.clone(),
    });
    let mut decision = ResultReuseDecision {
        schema: RESULT_REUSE_DECISION_SCHEMA.to_string(),
        decision_blake3: String::new(),
        usable: is_usable,
        reason,
        evidence_link,
        non_claim: RESOURCE_POLICY_NON_CLAIM.to_string(),
    };
    decision.decision_blake3 = reuse_decision_identity(&decision)?;
    debug_assert_eq!(decision.usable, decision.evidence_link.is_some());
    debug_assert_eq!(decision.usable, decision.reason == ReuseReasonCode::Usable);
    Ok(decision)
}

pub fn validate_result_reuse_decision(
    decision: ResultReuseDecision,
) -> Result<ResultReuseDecision, ResourcePolicyError> {
    if decision.schema != RESULT_REUSE_DECISION_SCHEMA
        || decision.non_claim != RESOURCE_POLICY_NON_CLAIM
        || decision.usable != decision.evidence_link.is_some()
    {
        return Err(ResourcePolicyError::InvalidSchema);
    }
    validate_digest(&decision.decision_blake3)?;
    if let Some(link) = &decision.evidence_link {
        validate_digest(&link.producer_result_blake3)?;
        validate_digest(&link.consumer_request_blake3)?;
        validate_digest(&link.authorization_decision_blake3)?;
        validate_id(&link.sharing_policy_id)?;
    }
    if reuse_decision_identity(&decision)? != decision.decision_blake3 {
        return Err(ResourcePolicyError::InvalidDigest);
    }
    debug_assert_eq!(decision.usable, decision.reason == ReuseReasonCode::Usable);
    debug_assert_eq!(decision.usable, decision.evidence_link.is_some());
    Ok(decision)
}

fn canonical_sharing_policy(mut policy: ResultSharingPolicy) -> Result<ResultSharingPolicy, ResourcePolicyError> {
    if policy.schema != RESULT_SHARING_POLICY_SCHEMA {
        return Err(ResourcePolicyError::InvalidSchema);
    }
    validate_id(&policy.policy_id)?;
    if let Some(project_id) = &policy.project_id {
        validate_id(project_id)?;
    }
    policy.allowed_producers = canonical_strings(policy.allowed_producers, MAX_SHARING_SUBJECTS)?;
    policy.allowed_consumers = canonical_strings(policy.allowed_consumers, MAX_SHARING_SUBJECTS)?;
    match policy.scope {
        SharingScopeKind::Private => {
            if policy.project_id.is_some() {
                return Err(ResourcePolicyError::InvalidBounds);
            }
            if !policy.allowed_producers.is_empty() || !policy.allowed_consumers.is_empty() {
                return Err(ResourcePolicyError::InvalidBounds);
            }
        }
        SharingScopeKind::Project => {
            if policy.project_id.is_none() {
                return Err(ResourcePolicyError::InvalidBounds);
            }
            if !policy.allowed_producers.is_empty() || !policy.allowed_consumers.is_empty() {
                return Err(ResourcePolicyError::InvalidBounds);
            }
        }
        SharingScopeKind::Named => {
            if policy.project_id.is_some() {
                return Err(ResourcePolicyError::InvalidBounds);
            }
            if policy.allowed_producers.is_empty() || policy.allowed_consumers.is_empty() {
                return Err(ResourcePolicyError::InvalidBounds);
            }
        }
    }
    debug_assert!(u32::try_from(policy.allowed_producers.len()).is_ok_and(|count| count <= MAX_SHARING_SUBJECTS));
    debug_assert!(u32::try_from(policy.allowed_consumers.len()).is_ok_and(|count| count <= MAX_SHARING_SUBJECTS));
    Ok(policy)
}

fn validate_reuse_facts(facts: &ResultReuseFacts) -> Result<(), ResourcePolicyError> {
    validate_id(&facts.producer_project_id)?;
    validate_id(&facts.consumer_project_id)?;
    validate_digest(&facts.producer_result_blake3)?;
    validate_digest(&facts.consumer_request_blake3)?;
    validate_digest(&facts.authorization_decision_blake3)?;
    debug_assert!(!facts.producer_project_id.is_empty());
    debug_assert!(!facts.consumer_project_id.is_empty());
    Ok(())
}

fn first_reuse_rejection(
    policy: &ResultSharingPolicy,
    facts: &ResultReuseFacts,
    controls: ResourceFeatureControls,
) -> Option<ReuseReasonCode> {
    if controls.mode != ResourcePolicyMode::Enforce || !controls.project_opted_in || !controls.result_sharing_enabled {
        return Some(ReuseReasonCode::SharingDisabled);
    }
    if let Some(reason) = scope_rejection(policy, facts) {
        return Some(reason);
    }
    let checks = [
        (facts.request_identity_matches, ReuseReasonCode::RequestMismatch),
        (facts.action_identity_matches, ReuseReasonCode::ActionMismatch),
        (facts.producer_signature_trusted, ReuseReasonCode::SignatureUntrusted),
        (facts.policy_compatible, ReuseReasonCode::PolicyMismatch),
        (facts.platform_compatible, ReuseReasonCode::PlatformMismatch),
        (facts.output_identity_verified, ReuseReasonCode::OutputIdentityInvalid),
        (facts.cas_available, ReuseReasonCode::CasUnavailable),
        (facts.strong_reuse_admitted, ReuseReasonCode::StrongReuseRejected),
    ];
    checks.into_iter().find_map(|(accepted, reason)| (!accepted).then_some(reason))
}

fn scope_rejection(policy: &ResultSharingPolicy, facts: &ResultReuseFacts) -> Option<ReuseReasonCode> {
    match policy.scope {
        SharingScopeKind::Private => {
            if facts.producer_project_id == facts.consumer_project_id {
                None
            } else {
                Some(ReuseReasonCode::ConsumerScopeDenied)
            }
        }
        SharingScopeKind::Project => {
            let project = policy.project_id.as_deref();
            if project != Some(facts.producer_project_id.as_str()) {
                return Some(ReuseReasonCode::ProducerScopeDenied);
            }
            if project != Some(facts.consumer_project_id.as_str()) {
                return Some(ReuseReasonCode::ConsumerScopeDenied);
            }
            None
        }
        SharingScopeKind::Named => {
            if policy.allowed_producers.binary_search(&facts.producer_project_id).is_err() {
                return Some(ReuseReasonCode::ProducerScopeDenied);
            }
            if policy.allowed_consumers.binary_search(&facts.consumer_project_id).is_err() {
                return Some(ReuseReasonCode::ConsumerScopeDenied);
            }
            None
        }
    }
}

fn reuse_decision_identity(decision: &ResultReuseDecision) -> Result<String, ResourcePolicyError> {
    let mut material = decision.clone();
    material.decision_blake3.clear();
    let identity = canonical_blake3(REUSE_DOMAIN, &material)?;
    debug_assert!(crate::valid_blake3(&identity));
    debug_assert!(material.decision_blake3.is_empty());
    Ok(identity)
}
