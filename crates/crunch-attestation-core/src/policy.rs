use alloc::collections::BTreeSet;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;

use serde::Deserialize;
use serde::Serialize;

use crate::AttestationDigest;
use crate::Error;
use crate::release::BinaryDigest;
use crate::release::BinaryDigestMatchInput;
use crate::release::FinalClass;
use crate::release::IndependentAgreementStatus;
use crate::release::PolicyStatus;
use crate::release::ReleaseAttestation;
use crate::release::TechnicalClass;
use crate::release::TrustTier;
use crate::release::WitnessAttestation;
use crate::release::binary_digests_match;
use crate::release::release_attestation_canonical_digest;

pub const RELEASE_POLICY_SCHEMA: &str = "mantle-release-policy-v1";
pub const RELEASE_REVOCATIONS_SCHEMA: &str = "mantle-release-revocations-v1";

const MAX_SIGNER_COUNT: u32 = 256;
const MAX_REVOCATION_COUNT: u32 = 4_096;
const OPTIONAL_WITNESS_MINIMUM: u32 = 0;
const SINGLE_WITNESS_MINIMUM: u32 = 1;
const PROFILE_REQUIRED_RELEASE_SIGNER_COUNT: u32 = 1;
pub const POLICY_PROFILE_MAX_WITNESS_COUNT: u32 = MAX_SIGNER_COUNT;
const INDEPENDENCE_FIELD_WITNESS_IDENTITY: &str = "witness_identity";
const INDEPENDENCE_FIELD_SIGNER_KEY_NAME: &str = "signer_key_name";
const INDEPENDENCE_FIELD_REBUILD_HOST_CLASS: &str = "rebuild_environment_summary.host_class";

const _: () = assert!(MAX_REVOCATION_COUNT >= 1, "revocation limit must be positive");
const _: () = assert!(POLICY_PROFILE_MAX_WITNESS_COUNT > OPTIONAL_WITNESS_MINIMUM);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum IndependenceSelector {
    WitnessIdentity,
    SignerKeyName,
    RebuildHostClass,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ReleasePolicy {
    pub schema: String,
    pub min_matching_witnesses: u32,
    pub independence_field: String,
    pub trusted_release_signers: Vec<String>,
    pub trusted_witness_signers: Vec<String>,
}

impl ReleasePolicy {
    pub fn new(
        min_matching_witnesses: u32,
        independence_field: String,
        trusted_release_signers: Vec<String>,
        trusted_witness_signers: Vec<String>,
    ) -> Self {
        Self {
            schema: RELEASE_POLICY_SCHEMA.to_string(),
            min_matching_witnesses,
            independence_field,
            trusted_release_signers,
            trusted_witness_signers,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReleasePolicyProfile {
    SelfProofOnly,
    OptionalWitness,
    SingleWitness,
    WitnessQuorum,
}

impl ReleasePolicyProfile {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::SelfProofOnly => "self-proof-only",
            Self::OptionalWitness => "optional-witness",
            Self::SingleWitness => "single-witness",
            Self::WitnessQuorum => "witness-quorum",
        }
    }

    const fn rejects_duplicate_names(self) -> bool {
        matches!(self, Self::OptionalWitness | Self::WitnessQuorum)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReleasePolicyProfileInput {
    pub profile: ReleasePolicyProfile,
    pub min_matching_witnesses: Option<u32>,
    pub independence_field: Option<String>,
    pub trusted_release_signers: Vec<String>,
    pub trusted_witness_identities: Vec<String>,
}

pub fn build_release_policy_profile(input: ReleasePolicyProfileInput) -> Result<ReleasePolicy, Error> {
    assert!(!input.profile.as_str().is_empty(), "policy profile name must not be empty");
    let is_duplicate_rejection_required = input.profile.rejects_duplicate_names();
    let trusted_release_signers = normalize_profile_names(
        input.trusted_release_signers,
        "trusted_release_signers",
        PROFILE_REQUIRED_RELEASE_SIGNER_COUNT,
        is_duplicate_rejection_required,
    )?;
    let trusted_witness_identities = normalize_profile_names(
        input.trusted_witness_identities,
        "trusted_witness_identities",
        OPTIONAL_WITNESS_MINIMUM,
        is_duplicate_rejection_required,
    )?;
    let witness_count = count_profile_values(&trusted_witness_identities, "trusted_witness_identities")?;
    let (min_matching_witnesses, independence_field) =
        profile_policy_terms(input.profile, input.min_matching_witnesses, input.independence_field, witness_count)?;
    let policy = ReleasePolicy::new(
        min_matching_witnesses,
        independence_field,
        trusted_release_signers,
        trusted_witness_identities,
    );
    validate_policy(&policy)?;
    assert!(policy.trusted_release_signers.windows(2).all(|window| window[0] < window[1]));
    assert!(policy.trusted_witness_signers.windows(2).all(|window| window[0] < window[1]));
    Ok(policy)
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ReleaseRevocations {
    pub schema: String,
    pub revoked_witness_keys: Vec<String>,
    pub revoked_witness_attestation_digests_blake3: Vec<String>,
}

impl ReleaseRevocations {
    pub fn new(revoked_witness_keys: Vec<String>, revoked_witness_attestation_digests_blake3: Vec<String>) -> Self {
        Self {
            schema: RELEASE_REVOCATIONS_SCHEMA.to_string(),
            revoked_witness_keys,
            revoked_witness_attestation_digests_blake3,
        }
    }

    pub fn empty() -> Self {
        Self::new(Vec::new(), Vec::new())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidatedWitness {
    pub attestation: WitnessAttestation,
    pub attestation_digest: AttestationDigest,
    pub signer_key_name: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PolicyEvaluation {
    pub trust_tier: TrustTier,
    pub witness_quorum_status: IndependentAgreementStatus,
    pub matching_witness_count: u32,
    pub independent_witness_identities: u32,
    pub revoked_witness_count: u32,
    pub policy_failure_reason: Option<PolicyFailureReason>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PolicyFailureReason {
    InsufficientQuorum { required: u32, matched: u32 },
    InsufficientIndependence { distinct_identities: u32, required: u32 },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PolicyEvaluationInput {
    pub release: ReleaseAttestation,
    pub witnesses: Vec<ValidatedWitness>,
    pub policy: ReleasePolicy,
    pub revocations: ReleaseRevocations,
}

pub fn evaluate_policy(input: PolicyEvaluationInput) -> Result<PolicyEvaluation, Error> {
    assert_policy_quorum(&input.policy);
    let independence_selector = validate_policy(&input.policy)?;
    validate_revocations(&input.revocations)?;

    let release_digest = release_attestation_canonical_digest(input.release.clone())?;
    let release_binary_digests = normalize_for_comparison(input.release.binary_digests);
    let (revoked_count, active_witnesses) = filter_active_witnesses(input.witnesses, &input.revocations);
    let (matching_count, distinct_identities) =
        count_matching_witnesses(&release_digest, &release_binary_digests, &active_witnesses, independence_selector)?;
    assert_matching_witness_counts(matching_count, active_witnesses.len(), distinct_identities);
    let active_witness_count = match u32::try_from(active_witnesses.len()) {
        Ok(count) => count,
        Err(_) => input.policy.min_matching_witnesses.saturating_add(matching_count),
    };
    assert!(matching_count <= active_witness_count, "matching count must fit active witnesses");
    assert!(distinct_identities <= matching_count, "independence domains must fit matching witnesses");

    let technical_class = if matching_count > 0 {
        TechnicalClass::ExternalWitnessMatch
    } else {
        TechnicalClass::SelfProofValid
    };
    let is_quorum_met = matching_count >= input.policy.min_matching_witnesses;
    let is_independence_met = distinct_identities >= input.policy.min_matching_witnesses;

    let (policy_status, failure_reason) = if !is_quorum_met {
        (
            PolicyStatus::Insufficient,
            Some(PolicyFailureReason::InsufficientQuorum {
                required: input.policy.min_matching_witnesses,
                matched: matching_count,
            }),
        )
    } else if !is_independence_met {
        (
            PolicyStatus::Insufficient,
            Some(PolicyFailureReason::InsufficientIndependence {
                distinct_identities,
                required: input.policy.min_matching_witnesses,
            }),
        )
    } else {
        (PolicyStatus::Satisfied, None)
    };

    let witness_quorum_status = classify_witness_quorum(input.policy.min_matching_witnesses, policy_status);
    let final_class = resolve_witness_final_class(technical_class, witness_quorum_status);
    if witness_quorum_status == IndependentAgreementStatus::NotRequired {
        assert_ne!(final_class, FinalClass::QuorumSatisfied, "optional witness evidence must not claim quorum");
    }

    Ok(PolicyEvaluation {
        trust_tier: TrustTier {
            technical_class,
            policy_status,
            final_class,
        },
        witness_quorum_status,
        matching_witness_count: matching_count,
        independent_witness_identities: distinct_identities,
        revoked_witness_count: revoked_count,
        policy_failure_reason: failure_reason,
    })
}

fn normalize_profile_names(
    requested_values: Vec<String>,
    field: &'static str,
    required_count: u32,
    reject_duplicates: bool,
) -> Result<Vec<String>, Error> {
    let requested_count = count_profile_values(&requested_values, field)?;
    let mut normalized_values = BTreeSet::new();
    for requested_value in requested_values {
        let normalized = requested_value.trim();
        if normalized.is_empty() {
            return Err(Error::EmptyField {
                field: field.to_string(),
            });
        }
        if normalized.chars().any(char::is_control) {
            return Err(Error::PolicyNameContainsControlCharacter {
                field: field.to_string(),
            });
        }
        let value = normalized.to_string();
        let was_inserted = normalized_values.insert(value.clone());
        if !was_inserted && reject_duplicates {
            return Err(Error::DuplicatePolicyName {
                field: field.to_string(),
                value,
            });
        }
    }
    let normalized = normalized_values.into_iter().collect::<Vec<_>>();
    let normalized_count = count_profile_values(&normalized, field)?;
    if normalized_count < required_count {
        return Err(Error::InsufficientPolicyValues {
            field: field.to_string(),
            required: required_count,
            actual: normalized_count,
        });
    }
    assert!(normalized_count <= requested_count, "normalization must not add policy values");
    assert!(normalized.windows(2).all(|window| window[0] < window[1]));
    Ok(normalized)
}

fn count_profile_values(values: &[String], field: &'static str) -> Result<u32, Error> {
    let actual = count_with_overflow_marker(values.len(), POLICY_PROFILE_MAX_WITNESS_COUNT);
    if actual > POLICY_PROFILE_MAX_WITNESS_COUNT {
        return Err(Error::CollectionTooLarge {
            limit: POLICY_PROFILE_MAX_WITNESS_COUNT,
            actual,
        });
    }
    assert!(actual <= POLICY_PROFILE_MAX_WITNESS_COUNT, "profile value count must fit the limit");
    assert!(!field.is_empty(), "profile field name must not be empty");
    Ok(actual)
}

fn profile_policy_terms(
    profile: ReleasePolicyProfile,
    requested_minimum: Option<u32>,
    requested_independence_field: Option<String>,
    witness_count: u32,
) -> Result<(u32, String), Error> {
    match profile {
        ReleasePolicyProfile::SelfProofOnly => {
            reject_profile_parameter(profile, "min_matching_witnesses", requested_minimum.is_some())?;
            reject_profile_parameter(profile, "independence_field", requested_independence_field.is_some())?;
            reject_profile_parameter(profile, "trusted_witness_identities", witness_count > 0)?;
            Ok((OPTIONAL_WITNESS_MINIMUM, INDEPENDENCE_FIELD_WITNESS_IDENTITY.to_string()))
        }
        ReleasePolicyProfile::OptionalWitness => {
            reject_profile_parameter(profile, "min_matching_witnesses", requested_minimum.is_some())?;
            let field = optional_profile_independence_field(requested_independence_field)?;
            Ok((OPTIONAL_WITNESS_MINIMUM, field))
        }
        ReleasePolicyProfile::SingleWitness => {
            reject_profile_parameter(profile, "min_matching_witnesses", requested_minimum.is_some())?;
            reject_profile_parameter(profile, "independence_field", requested_independence_field.is_some())?;
            require_witness_count(WitnessCountRequirement {
                actual: witness_count,
                required: SINGLE_WITNESS_MINIMUM,
            })?;
            Ok((SINGLE_WITNESS_MINIMUM, INDEPENDENCE_FIELD_WITNESS_IDENTITY.to_string()))
        }
        ReleasePolicyProfile::WitnessQuorum => {
            let minimum = requested_minimum.ok_or_else(|| Error::MissingPolicyParameter {
                profile: profile.as_str().to_string(),
                parameter: "min_matching_witnesses".to_string(),
            })?;
            validate_profile_threshold(minimum)?;
            require_witness_count(WitnessCountRequirement {
                actual: witness_count,
                required: minimum,
            })?;
            let field = requested_independence_field.ok_or_else(|| Error::MissingPolicyParameter {
                profile: profile.as_str().to_string(),
                parameter: "independence_field".to_string(),
            })?;
            parse_independence_selector(&field)?;
            Ok((minimum, field))
        }
    }
}

fn reject_profile_parameter(
    profile: ReleasePolicyProfile,
    parameter: &'static str,
    is_present: bool,
) -> Result<(), Error> {
    if is_present {
        return Err(Error::UnexpectedPolicyParameter {
            profile: profile.as_str().to_string(),
            parameter: parameter.to_string(),
        });
    }
    Ok(())
}

fn optional_profile_independence_field(requested: Option<String>) -> Result<String, Error> {
    let field = requested.unwrap_or_else(|| INDEPENDENCE_FIELD_WITNESS_IDENTITY.to_string());
    parse_independence_selector(&field)?;
    Ok(field)
}

fn validate_profile_threshold(minimum: u32) -> Result<(), Error> {
    if minimum == OPTIONAL_WITNESS_MINIMUM || minimum > POLICY_PROFILE_MAX_WITNESS_COUNT {
        return Err(Error::InvalidPolicyThreshold {
            minimum: SINGLE_WITNESS_MINIMUM,
            maximum: POLICY_PROFILE_MAX_WITNESS_COUNT,
            actual: minimum,
        });
    }
    Ok(())
}

struct WitnessCountRequirement {
    actual: u32,
    required: u32,
}

fn require_witness_count(requirement: WitnessCountRequirement) -> Result<(), Error> {
    if requirement.actual < requirement.required {
        return Err(Error::InsufficientPolicyValues {
            field: "trusted_witness_identities".to_string(),
            required: requirement.required,
            actual: requirement.actual,
        });
    }
    Ok(())
}

fn classify_witness_quorum(required: u32, policy_status: PolicyStatus) -> IndependentAgreementStatus {
    if required == OPTIONAL_WITNESS_MINIMUM {
        return IndependentAgreementStatus::NotRequired;
    }
    if policy_status == PolicyStatus::Satisfied {
        IndependentAgreementStatus::Satisfied
    } else {
        IndependentAgreementStatus::Insufficient
    }
}

fn resolve_witness_final_class(
    technical_class: TechnicalClass,
    quorum_status: IndependentAgreementStatus,
) -> FinalClass {
    if quorum_status == IndependentAgreementStatus::Satisfied {
        return FinalClass::QuorumSatisfied;
    }
    match technical_class {
        TechnicalClass::BundleConsistent => FinalClass::BundleConsistent,
        TechnicalClass::SelfProofValid => FinalClass::SelfProofValid,
        TechnicalClass::ExternalWitnessMatch => FinalClass::ExternalWitnessMatch,
    }
}

fn filter_active_witnesses(
    witnesses: Vec<ValidatedWitness>,
    revocations: &ReleaseRevocations,
) -> (u32, Vec<ValidatedWitness>) {
    assert!(
        revocations.revoked_witness_keys.len()
            <= witnesses.len().saturating_add(revocations.revoked_witness_keys.len())
    );
    let revoked_keys: BTreeSet<&str> = revocations.revoked_witness_keys.iter().map(|key| key.as_str()).collect();
    let revoked_digests: BTreeSet<&str> = revocations
        .revoked_witness_attestation_digests_blake3
        .iter()
        .map(|digest| digest.as_str())
        .collect();
    let mut revoked_count: u32 = 0;
    let mut active_witnesses: Vec<ValidatedWitness> = Vec::with_capacity(witnesses.len());
    assert!(active_witnesses.is_empty(), "active witness list must start empty");
    assert!(
        revoked_keys.len() <= revocations.revoked_witness_keys.len(),
        "revoked key set must deduplicate only"
    );

    for witness in witnesses {
        let is_key_revoked = revoked_keys.contains(witness.signer_key_name.as_str());
        let digest_hex = witness.attestation_digest.to_hex();
        let is_digest_revoked = revoked_digests.contains(digest_hex.as_str());
        if is_key_revoked || is_digest_revoked {
            revoked_count = revoked_count.saturating_add(1);
            continue;
        }
        active_witnesses.push(witness);
    }

    let active_witness_count = count_with_overflow_marker(active_witnesses.len(), revoked_count);
    let witness_total = active_witness_count.saturating_add(revoked_count);
    assert!(revoked_count <= witness_total, "revoked count must stay bounded");
    (revoked_count, active_witnesses)
}

fn count_matching_witnesses(
    release_digest: &AttestationDigest,
    release_binary_digests: &[BinaryDigest],
    active_witnesses: &[ValidatedWitness],
    independence_selector: IndependenceSelector,
) -> Result<(u32, u32), Error> {
    let mut matching_count: u32 = 0;
    let mut matching_domains: BTreeSet<&str> = BTreeSet::new();
    assert_eq!(matching_count, 0, "matching witness count must start at zero");
    assert!(matching_domains.is_empty(), "matching domains must start empty");

    for witness in active_witnesses {
        let is_release_digest_match = witness.attestation.release_attestation_digest_blake3 == *release_digest;
        let rebuilt = normalize_for_comparison(witness.attestation.rebuilt_digests.clone());
        if !is_release_digest_match
            || !binary_digests_match(BinaryDigestMatchInput {
                published: release_binary_digests.to_vec(),
                rebuilt,
            })
        {
            continue;
        }
        let domain = independence_domain(witness, independence_selector)?;
        matching_count = matching_count.saturating_add(1);
        matching_domains.insert(domain);
    }

    let distinct_identities = count_with_overflow_marker(matching_domains.len(), matching_count);
    assert!(distinct_identities <= matching_count, "distinct domains must not exceed matching witnesses");
    Ok((matching_count, distinct_identities))
}

fn independence_domain(witness: &ValidatedWitness, selector: IndependenceSelector) -> Result<&str, Error> {
    let (field, domain) = match selector {
        IndependenceSelector::WitnessIdentity => ("witness_identity", witness.attestation.witness_identity.as_str()),
        IndependenceSelector::SignerKeyName => ("signer_key_name", witness.signer_key_name.as_str()),
        IndependenceSelector::RebuildHostClass => (
            "rebuild_environment_summary.host_class",
            witness.attestation.rebuild_environment_summary.host_class.as_str(),
        ),
    };
    if domain.is_empty() {
        return Err(Error::EmptyField {
            field: field.to_string(),
        });
    }
    Ok(domain)
}

fn parse_independence_selector(field: &str) -> Result<IndependenceSelector, Error> {
    assert!(!field.is_empty(), "policy independence field must not be empty before parsing");
    match field {
        INDEPENDENCE_FIELD_WITNESS_IDENTITY => Ok(IndependenceSelector::WitnessIdentity),
        INDEPENDENCE_FIELD_SIGNER_KEY_NAME => Ok(IndependenceSelector::SignerKeyName),
        INDEPENDENCE_FIELD_REBUILD_HOST_CLASS => Ok(IndependenceSelector::RebuildHostClass),
        _ => Err(Error::UnsupportedPolicyField {
            field: "independence_field".to_string(),
            value: field.to_string(),
        }),
    }
}

fn assert_policy_quorum(policy: &ReleasePolicy) {
    assert!(usize::try_from(policy.min_matching_witnesses).is_ok(), "policy quorum must fit in usize");
}

fn assert_matching_witness_counts(matching_count: u32, active_witness_count: usize, distinct_identities: u32) {
    let active_witness_total = count_with_overflow_marker(active_witness_count, matching_count);
    assert!(matching_count <= active_witness_total, "matching witness count must stay bounded");
    assert!(distinct_identities <= matching_count, "distinct witness identities must not exceed matches");
}

fn validate_policy(policy: &ReleasePolicy) -> Result<IndependenceSelector, Error> {
    if policy.schema != RELEASE_POLICY_SCHEMA {
        return Err(Error::SchemaTagMismatch {
            expected: RELEASE_POLICY_SCHEMA.to_string(),
            actual: policy.schema.clone(),
        });
    }
    validate_signer_list(&policy.trusted_release_signers)?;
    validate_signer_list(&policy.trusted_witness_signers)?;
    if policy.independence_field.is_empty() {
        return Err(Error::EmptyField {
            field: "independence_field".to_string(),
        });
    }
    parse_independence_selector(&policy.independence_field)
}

fn validate_revocations(revocations: &ReleaseRevocations) -> Result<(), Error> {
    assert!(!RELEASE_REVOCATIONS_SCHEMA.is_empty(), "revocation schema tag must not be empty");
    if revocations.schema != RELEASE_REVOCATIONS_SCHEMA {
        return Err(Error::SchemaTagMismatch {
            expected: RELEASE_REVOCATIONS_SCHEMA.to_string(),
            actual: revocations.schema.clone(),
        });
    }
    let key_count = count_with_overflow_marker(revocations.revoked_witness_keys.len(), MAX_REVOCATION_COUNT);
    if key_count > MAX_REVOCATION_COUNT {
        return Err(Error::CollectionTooLarge {
            limit: MAX_REVOCATION_COUNT,
            actual: key_count,
        });
    }
    assert!(key_count <= MAX_REVOCATION_COUNT, "validated revoked key count must fit the limit");
    let digest_count =
        count_with_overflow_marker(revocations.revoked_witness_attestation_digests_blake3.len(), MAX_REVOCATION_COUNT);
    if digest_count > MAX_REVOCATION_COUNT {
        return Err(Error::CollectionTooLarge {
            limit: MAX_REVOCATION_COUNT,
            actual: digest_count,
        });
    }
    assert!(digest_count <= MAX_REVOCATION_COUNT, "validated revoked digest count must fit the limit");
    Ok(())
}

fn validate_signer_list(signers: &[String]) -> Result<(), Error> {
    let count = count_with_overflow_marker(signers.len(), MAX_SIGNER_COUNT);
    if count > MAX_SIGNER_COUNT {
        return Err(Error::CollectionTooLarge {
            limit: MAX_SIGNER_COUNT,
            actual: count,
        });
    }
    Ok(())
}

fn count_with_overflow_marker(value_count: usize, overflow_floor: u32) -> u32 {
    match u32::try_from(value_count) {
        Ok(value) => value,
        Err(_) => overflow_floor.saturating_add(1),
    }
}

fn normalize_for_comparison(digests: Vec<BinaryDigest>) -> Vec<BinaryDigest> {
    let mut sorted = digests;
    sorted.sort();
    sorted
}

#[cfg(test)]
mod tests {
    use alloc::vec;

    use super::*;
    use crate::release::RebuildEnvironmentSummary;

    const TEST_DIGEST_BYTE_COUNT: usize = 32;
    const TEST_QUORUM: u32 = 2;
    use crate::release::Workflow;

    #[test]
    fn policy_json_round_trip() {
        let policy = sample_policy();
        let json = serde_json::to_string(&policy).unwrap();
        let parsed: ReleasePolicy = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, policy);
    }

    #[test]
    fn revocations_json_round_trip() {
        let rev = sample_revocations();
        let json = serde_json::to_string(&rev).unwrap();
        let parsed: ReleaseRevocations = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, rev);
    }

    #[test]
    fn compatibility_profiles_keep_canonical_policy_fields() {
        let self_proof = build_release_policy_profile(ReleasePolicyProfileInput {
            profile: ReleasePolicyProfile::SelfProofOnly,
            min_matching_witnesses: None,
            independence_field: None,
            trusted_release_signers: vec![
                "release-b".to_string(),
                "release-a".to_string(),
                "release-a".to_string(),
            ],
            trusted_witness_identities: Vec::new(),
        })
        .unwrap();
        assert_eq!(self_proof.min_matching_witnesses, OPTIONAL_WITNESS_MINIMUM);
        assert_eq!(self_proof.independence_field, INDEPENDENCE_FIELD_WITNESS_IDENTITY);
        assert_eq!(self_proof.trusted_release_signers, vec!["release-a".to_string(), "release-b".to_string()]);
        assert!(self_proof.trusted_witness_signers.is_empty());

        let single = build_release_policy_profile(ReleasePolicyProfileInput {
            profile: ReleasePolicyProfile::SingleWitness,
            min_matching_witnesses: None,
            independence_field: None,
            trusted_release_signers: vec!["release-a".to_string()],
            trusted_witness_identities: vec!["witness-a".to_string()],
        })
        .unwrap();
        assert_eq!(single.min_matching_witnesses, SINGLE_WITNESS_MINIMUM);
        assert_eq!(single.independence_field, INDEPENDENCE_FIELD_WITNESS_IDENTITY);
        assert_eq!(single.trusted_witness_signers, vec!["witness-a".to_string()]);
    }

    #[test]
    fn optional_profile_accepts_trusted_witnesses_without_quorum() {
        let policy = build_release_policy_profile(ReleasePolicyProfileInput {
            profile: ReleasePolicyProfile::OptionalWitness,
            min_matching_witnesses: None,
            independence_field: Some(INDEPENDENCE_FIELD_SIGNER_KEY_NAME.to_string()),
            trusted_release_signers: vec!["release-a".to_string()],
            trusted_witness_identities: vec!["witness-b".to_string(), "witness-a".to_string()],
        })
        .unwrap();

        assert_eq!(policy.min_matching_witnesses, OPTIONAL_WITNESS_MINIMUM);
        assert_eq!(policy.independence_field, INDEPENDENCE_FIELD_SIGNER_KEY_NAME);
        assert_eq!(policy.trusted_witness_signers, vec!["witness-a".to_string(), "witness-b".to_string()]);
    }

    #[test]
    fn quorum_profile_accepts_each_supported_selector() {
        let selectors = [
            INDEPENDENCE_FIELD_WITNESS_IDENTITY,
            INDEPENDENCE_FIELD_SIGNER_KEY_NAME,
            INDEPENDENCE_FIELD_REBUILD_HOST_CLASS,
        ];
        for selector in selectors {
            let policy = build_release_policy_profile(ReleasePolicyProfileInput {
                profile: ReleasePolicyProfile::WitnessQuorum,
                min_matching_witnesses: Some(TEST_QUORUM),
                independence_field: Some(selector.to_string()),
                trusted_release_signers: vec!["release-a".to_string()],
                trusted_witness_identities: vec!["witness-a".to_string(), "witness-b".to_string()],
            })
            .unwrap();
            assert_eq!(policy.min_matching_witnesses, TEST_QUORUM);
            assert_eq!(policy.independence_field, selector);
        }
    }

    #[test]
    fn quorum_profile_succeeds_under_each_supported_selector() {
        let release = sample_release();
        let selector_witnesses = [
            (INDEPENDENCE_FIELD_WITNESS_IDENTITY, vec![
                make_matching_witness_with_details(&release, "witness-a", "key-a", "host-a", b"identity-a"),
                make_matching_witness_with_details(&release, "witness-b", "key-a", "host-a", b"identity-b"),
            ]),
            (INDEPENDENCE_FIELD_SIGNER_KEY_NAME, vec![
                make_matching_witness_with_details(&release, "witness-a", "key-a", "host-a", b"signer-a"),
                make_matching_witness_with_details(&release, "witness-b", "key-b", "host-a", b"signer-b"),
            ]),
            (INDEPENDENCE_FIELD_REBUILD_HOST_CLASS, vec![
                make_matching_witness_with_details(&release, "witness-a", "key-a", "host-a", b"host-a"),
                make_matching_witness_with_details(&release, "witness-b", "key-a", "host-b", b"host-b"),
            ]),
        ];

        for (selector, witnesses) in selector_witnesses {
            let policy = build_release_policy_profile(ReleasePolicyProfileInput {
                profile: ReleasePolicyProfile::WitnessQuorum,
                min_matching_witnesses: Some(TEST_QUORUM),
                independence_field: Some(selector.to_string()),
                trusted_release_signers: vec!["release-a".to_string()],
                trusted_witness_identities: vec!["witness-a".to_string(), "witness-b".to_string()],
            })
            .unwrap();
            let result = evaluate_policy(PolicyEvaluationInput {
                release: release.clone(),
                witnesses,
                policy,
                revocations: ReleaseRevocations::empty(),
            })
            .unwrap();

            assert_eq!(result.witness_quorum_status, IndependentAgreementStatus::Satisfied);
            assert_eq!(result.trust_tier.final_class, FinalClass::QuorumSatisfied);
            assert_eq!(result.independent_witness_identities, TEST_QUORUM);
        }
    }

    #[test]
    fn quorum_profile_rejects_missing_or_invalid_parameters() {
        let missing_minimum = build_release_policy_profile(ReleasePolicyProfileInput {
            profile: ReleasePolicyProfile::WitnessQuorum,
            min_matching_witnesses: None,
            independence_field: Some(INDEPENDENCE_FIELD_WITNESS_IDENTITY.to_string()),
            trusted_release_signers: vec!["release-a".to_string()],
            trusted_witness_identities: vec!["witness-a".to_string()],
        })
        .unwrap_err();
        assert!(matches!(missing_minimum, Error::MissingPolicyParameter { .. }));

        let zero_minimum = build_release_policy_profile(ReleasePolicyProfileInput {
            profile: ReleasePolicyProfile::WitnessQuorum,
            min_matching_witnesses: Some(OPTIONAL_WITNESS_MINIMUM),
            independence_field: Some(INDEPENDENCE_FIELD_WITNESS_IDENTITY.to_string()),
            trusted_release_signers: vec!["release-a".to_string()],
            trusted_witness_identities: vec!["witness-a".to_string()],
        })
        .unwrap_err();
        assert_eq!(zero_minimum, Error::InvalidPolicyThreshold {
            minimum: SINGLE_WITNESS_MINIMUM,
            maximum: POLICY_PROFILE_MAX_WITNESS_COUNT,
            actual: OPTIONAL_WITNESS_MINIMUM,
        });

        let unsupported_selector = build_release_policy_profile(ReleasePolicyProfileInput {
            profile: ReleasePolicyProfile::WitnessQuorum,
            min_matching_witnesses: Some(SINGLE_WITNESS_MINIMUM),
            independence_field: Some("ambient_organization".to_string()),
            trusted_release_signers: vec!["release-a".to_string()],
            trusted_witness_identities: vec!["witness-a".to_string()],
        })
        .unwrap_err();
        assert!(matches!(unsupported_selector, Error::UnsupportedPolicyField { .. }));

        let too_few_identities = build_release_policy_profile(ReleasePolicyProfileInput {
            profile: ReleasePolicyProfile::WitnessQuorum,
            min_matching_witnesses: Some(TEST_QUORUM),
            independence_field: Some(INDEPENDENCE_FIELD_WITNESS_IDENTITY.to_string()),
            trusted_release_signers: vec!["release-a".to_string()],
            trusted_witness_identities: vec!["witness-a".to_string()],
        })
        .unwrap_err();
        assert_eq!(too_few_identities, Error::InsufficientPolicyValues {
            field: "trusted_witness_identities".to_string(),
            required: TEST_QUORUM,
            actual: SINGLE_WITNESS_MINIMUM,
        });
    }

    #[test]
    fn new_profiles_reject_duplicate_names_and_unbounded_thresholds() {
        let duplicate = build_release_policy_profile(ReleasePolicyProfileInput {
            profile: ReleasePolicyProfile::OptionalWitness,
            min_matching_witnesses: None,
            independence_field: None,
            trusted_release_signers: vec!["release-a".to_string()],
            trusted_witness_identities: vec!["witness-a".to_string(), " witness-a ".to_string()],
        })
        .unwrap_err();
        assert_eq!(duplicate, Error::DuplicatePolicyName {
            field: "trusted_witness_identities".to_string(),
            value: "witness-a".to_string(),
        });

        let unbounded = build_release_policy_profile(ReleasePolicyProfileInput {
            profile: ReleasePolicyProfile::WitnessQuorum,
            min_matching_witnesses: Some(u32::MAX),
            independence_field: Some(INDEPENDENCE_FIELD_WITNESS_IDENTITY.to_string()),
            trusted_release_signers: vec!["release-a".to_string()],
            trusted_witness_identities: vec!["witness-a".to_string()],
        })
        .unwrap_err();
        assert_eq!(unbounded, Error::InvalidPolicyThreshold {
            minimum: SINGLE_WITNESS_MINIMUM,
            maximum: POLICY_PROFILE_MAX_WITNESS_COUNT,
            actual: u32::MAX,
        });
    }

    #[test]
    fn policy_change_does_not_affect_attestation_digests() {
        let release = sample_release();
        let digest_before = release_attestation_canonical_digest(release.clone()).unwrap();

        let _policy_loose = ReleasePolicy::new(1, "witness_identity".to_string(), vec!["signer-1".to_string()], vec![
            "witness-a".to_string(),
        ]);
        let _policy_strict = ReleasePolicy::new(3, "witness_identity".to_string(), vec!["signer-1".to_string()], vec![
            "witness-a".to_string(),
            "witness-b".to_string(),
            "witness-c".to_string(),
        ]);

        let digest_after = release_attestation_canonical_digest(release).unwrap();
        assert_eq!(digest_before, digest_after);
    }

    #[test]
    fn zero_witnesses_yield_self_proof_valid() {
        let release = sample_release();
        let policy = ReleasePolicy::new(0, "witness_identity".to_string(), vec!["signer-1".to_string()], Vec::new());
        let revocations = ReleaseRevocations::empty();

        let result = evaluate_policy(PolicyEvaluationInput {
            release,
            witnesses: Vec::new(),
            policy,
            revocations,
        })
        .unwrap();

        assert_eq!(result.trust_tier.technical_class, TechnicalClass::SelfProofValid);
        assert_eq!(result.trust_tier.policy_status, PolicyStatus::Satisfied);
        assert_eq!(result.witness_quorum_status, IndependentAgreementStatus::NotRequired);
        assert_eq!(result.trust_tier.final_class, FinalClass::SelfProofValid);
        assert_eq!(result.matching_witness_count, 0);
    }

    #[test]
    fn one_matching_witness_raises_technical_class() {
        let release = sample_release();
        let witness = make_matching_witness(&release, "witness-a");
        let policy = ReleasePolicy::new(1, "witness_identity".to_string(), vec!["signer-1".to_string()], vec![
            "witness-a".to_string(),
        ]);
        let revocations = ReleaseRevocations::empty();

        let result = evaluate_policy(PolicyEvaluationInput {
            release,
            witnesses: vec![witness],
            policy,
            revocations,
        })
        .unwrap();

        assert_eq!(result.trust_tier.technical_class, TechnicalClass::ExternalWitnessMatch);
        assert_eq!(result.matching_witness_count, 1);
    }

    #[test]
    fn optional_policy_keeps_one_or_multiple_valid_witnesses_without_quorum_claim() {
        let release = sample_release();
        let witness_sets = [vec![make_matching_witness(&release, "witness-a")], vec![
            make_matching_witness(&release, "witness-a"),
            make_matching_witness(&release, "witness-b"),
        ]];

        for witnesses in witness_sets {
            let result = evaluate_policy(PolicyEvaluationInput {
                release: release.clone(),
                witnesses,
                policy: ReleasePolicy::new(
                    OPTIONAL_WITNESS_MINIMUM,
                    INDEPENDENCE_FIELD_WITNESS_IDENTITY.to_string(),
                    vec!["signer-1".to_string()],
                    vec!["witness-a".to_string(), "witness-b".to_string()],
                ),
                revocations: ReleaseRevocations::empty(),
            })
            .unwrap();

            assert_eq!(result.witness_quorum_status, IndependentAgreementStatus::NotRequired);
            assert_eq!(result.trust_tier.technical_class, TechnicalClass::ExternalWitnessMatch);
            assert_eq!(result.trust_tier.final_class, FinalClass::ExternalWitnessMatch);
            assert!(result.matching_witness_count >= SINGLE_WITNESS_MINIMUM);
        }
    }

    #[test]
    fn insufficient_quorum_fails_policy_with_technical_success() {
        let release = sample_release();
        let witness = make_matching_witness(&release, "witness-a");
        let policy = ReleasePolicy::new(2, "witness_identity".to_string(), vec!["signer-1".to_string()], vec![
            "witness-a".to_string(),
        ]);
        let revocations = ReleaseRevocations::empty();

        let result = evaluate_policy(PolicyEvaluationInput {
            release,
            witnesses: vec![witness],
            policy,
            revocations,
        })
        .unwrap();

        assert_eq!(result.trust_tier.technical_class, TechnicalClass::ExternalWitnessMatch);
        assert_eq!(result.witness_quorum_status, IndependentAgreementStatus::Insufficient);
        assert_eq!(result.trust_tier.policy_status, PolicyStatus::Insufficient);
        assert_eq!(result.trust_tier.final_class, FinalClass::ExternalWitnessMatch);
        assert_eq!(
            result.policy_failure_reason,
            Some(PolicyFailureReason::InsufficientQuorum {
                required: 2,
                matched: 1,
            })
        );
    }

    #[test]
    fn satisfied_quorum_promotes_final_class() {
        let release = sample_release();
        let w_a = make_matching_witness(&release, "witness-a");
        let w_b = make_matching_witness(&release, "witness-b");
        let policy = ReleasePolicy::new(2, "witness_identity".to_string(), vec!["signer-1".to_string()], vec![
            "witness-a".to_string(),
            "witness-b".to_string(),
        ]);
        let revocations = ReleaseRevocations::empty();

        let result = evaluate_policy(PolicyEvaluationInput {
            release,
            witnesses: vec![w_a, w_b],
            policy,
            revocations,
        })
        .unwrap();

        assert_eq!(result.trust_tier.policy_status, PolicyStatus::Satisfied);
        assert_eq!(result.witness_quorum_status, IndependentAgreementStatus::Satisfied);
        assert_eq!(result.trust_tier.final_class, FinalClass::QuorumSatisfied);
        assert_eq!(result.matching_witness_count, 2);
        assert_eq!(result.independent_witness_identities, 2);
    }

    #[test]
    fn same_identity_cannot_satisfy_independence() {
        let release = sample_release();
        let w1 = make_matching_witness(&release, "witness-a");
        let w2 = make_matching_witness_with_details(
            &release,
            "witness-a",
            "witness-a-key-2",
            "other-host",
            b"witness-a-second",
        );
        let policy =
            ReleasePolicy::new(2, INDEPENDENCE_FIELD_WITNESS_IDENTITY.to_string(), vec!["signer-1".to_string()], vec![
                "witness-a".to_string(),
            ]);
        let revocations = ReleaseRevocations::empty();

        let result = evaluate_policy(PolicyEvaluationInput {
            release,
            witnesses: vec![w1, w2],
            policy,
            revocations,
        })
        .unwrap();

        assert_eq!(result.matching_witness_count, 2);
        assert_eq!(result.independent_witness_identities, 1);
        assert_eq!(result.trust_tier.policy_status, PolicyStatus::Insufficient);
        assert_eq!(
            result.policy_failure_reason,
            Some(PolicyFailureReason::InsufficientIndependence {
                distinct_identities: 1,
                required: 2,
            })
        );
    }

    #[test]
    fn same_signer_key_cannot_satisfy_signer_key_independence() {
        let release = sample_release();
        let w_a = make_matching_witness_with_details(&release, "witness-a", "shared-key", "org-a", b"witness-a");
        let w_b = make_matching_witness_with_details(&release, "witness-b", "shared-key", "org-b", b"witness-b");
        let policy = ReleasePolicy::new(2, INDEPENDENCE_FIELD_SIGNER_KEY_NAME.to_string(), Vec::new(), Vec::new());

        let result = evaluate_policy(PolicyEvaluationInput {
            release,
            witnesses: vec![w_a, w_b],
            policy,
            revocations: ReleaseRevocations::empty(),
        })
        .unwrap();

        assert_eq!(result.matching_witness_count, 2);
        assert_eq!(result.independent_witness_identities, 1);
        assert_eq!(result.trust_tier.policy_status, PolicyStatus::Insufficient);
        assert_eq!(
            result.policy_failure_reason,
            Some(PolicyFailureReason::InsufficientIndependence {
                distinct_identities: 1,
                required: 2,
            })
        );
    }

    #[test]
    fn distinct_signer_keys_can_satisfy_signer_key_independence() {
        let release = sample_release();
        let w_a = make_matching_witness_with_details(&release, "shared-identity", "key-a", "org-a", b"key-a");
        let w_b = make_matching_witness_with_details(&release, "shared-identity", "key-b", "org-a", b"key-b");
        let policy = ReleasePolicy::new(2, INDEPENDENCE_FIELD_SIGNER_KEY_NAME.to_string(), Vec::new(), Vec::new());

        let result = evaluate_policy(PolicyEvaluationInput {
            release,
            witnesses: vec![w_a, w_b],
            policy,
            revocations: ReleaseRevocations::empty(),
        })
        .unwrap();

        assert_eq!(result.matching_witness_count, 2);
        assert_eq!(result.independent_witness_identities, 2);
        assert_eq!(result.trust_tier.policy_status, PolicyStatus::Satisfied);
        assert_eq!(result.trust_tier.final_class, FinalClass::QuorumSatisfied);
    }

    #[test]
    fn empty_selected_independence_field_is_rejected() {
        let release = sample_release();
        let witness = make_matching_witness_with_details(&release, "witness-a", "", "org-a", b"empty-signer");
        let policy = ReleasePolicy::new(1, INDEPENDENCE_FIELD_SIGNER_KEY_NAME.to_string(), Vec::new(), Vec::new());

        let err = evaluate_policy(PolicyEvaluationInput {
            release,
            witnesses: vec![witness],
            policy,
            revocations: ReleaseRevocations::empty(),
        })
        .unwrap_err();

        assert_eq!(err, Error::EmptyField {
            field: "signer_key_name".to_string()
        });
    }

    #[test]
    fn shared_host_class_cannot_satisfy_host_class_independence() {
        let release = sample_release();
        let w_a = make_matching_witness_with_details(&release, "witness-a", "key-a", "shared-org", b"org-a");
        let w_b = make_matching_witness_with_details(&release, "witness-b", "key-b", "shared-org", b"org-b");
        let policy = ReleasePolicy::new(2, INDEPENDENCE_FIELD_REBUILD_HOST_CLASS.to_string(), Vec::new(), Vec::new());

        let result = evaluate_policy(PolicyEvaluationInput {
            release,
            witnesses: vec![w_a, w_b],
            policy,
            revocations: ReleaseRevocations::empty(),
        })
        .unwrap();

        assert_eq!(result.matching_witness_count, 2);
        assert_eq!(result.independent_witness_identities, 1);
        assert_eq!(result.trust_tier.policy_status, PolicyStatus::Insufficient);
        assert_eq!(
            result.policy_failure_reason,
            Some(PolicyFailureReason::InsufficientIndependence {
                distinct_identities: 1,
                required: 2,
            })
        );
    }

    #[test]
    fn revoked_key_degrades_policy_without_changing_technical_result() {
        let release = sample_release();
        let w_a = make_matching_witness(&release, "witness-a");
        let w_b = make_matching_witness(&release, "witness-b");
        let policy = ReleasePolicy::new(2, "witness_identity".to_string(), vec!["signer-1".to_string()], vec![
            "witness-a".to_string(),
            "witness-b".to_string(),
        ]);

        let before = evaluate_policy(PolicyEvaluationInput {
            release: release.clone(),
            witnesses: vec![w_a.clone(), w_b.clone()],
            policy: policy.clone(),
            revocations: ReleaseRevocations::empty(),
        })
        .unwrap();
        assert_eq!(before.trust_tier.policy_status, PolicyStatus::Satisfied);

        let after = evaluate_policy(PolicyEvaluationInput {
            release,
            witnesses: vec![w_a, w_b],
            policy,
            revocations: ReleaseRevocations::new(vec!["witness-a-key".to_string()], Vec::new()),
        })
        .unwrap();

        assert_eq!(after.trust_tier.technical_class, TechnicalClass::ExternalWitnessMatch);
        assert_eq!(after.trust_tier.policy_status, PolicyStatus::Insufficient);
        assert_eq!(after.revoked_witness_count, 1);
        assert_eq!(after.matching_witness_count, 1);
    }

    #[test]
    fn revoked_attestation_digest_degrades_policy() {
        let release = sample_release();
        let w_a = make_matching_witness(&release, "witness-a");
        let revoked_digest = w_a.attestation_digest.to_hex();
        let w_b = make_matching_witness(&release, "witness-b");
        let policy = ReleasePolicy::new(2, "witness_identity".to_string(), vec!["signer-1".to_string()], vec![
            "witness-a".to_string(),
            "witness-b".to_string(),
        ]);

        let result = evaluate_policy(PolicyEvaluationInput {
            release,
            witnesses: vec![w_a, w_b],
            policy,
            revocations: ReleaseRevocations::new(Vec::new(), vec![revoked_digest]),
        })
        .unwrap();

        assert_eq!(result.revoked_witness_count, 1);
        assert_eq!(result.matching_witness_count, 1);
        assert_eq!(result.trust_tier.policy_status, PolicyStatus::Insufficient);
    }

    #[test]
    fn revocations_applied_before_quorum_counting() {
        let release = sample_release();
        let w_a = make_matching_witness(&release, "witness-a");
        let w_b = make_matching_witness(&release, "witness-b");
        let policy = ReleasePolicy::new(2, "witness_identity".to_string(), vec!["signer-1".to_string()], vec![
            "witness-a".to_string(),
            "witness-b".to_string(),
        ]);

        let result = evaluate_policy(PolicyEvaluationInput {
            release,
            witnesses: vec![w_a, w_b],
            policy,
            revocations: ReleaseRevocations::new(
                vec!["witness-a-key".to_string(), "witness-b-key".to_string()],
                Vec::new(),
            ),
        })
        .unwrap();

        assert_eq!(result.revoked_witness_count, 2);
        assert_eq!(result.matching_witness_count, 0);
        assert_eq!(result.trust_tier.technical_class, TechnicalClass::SelfProofValid);
        assert_eq!(result.trust_tier.policy_status, PolicyStatus::Insufficient);
    }

    #[test]
    fn witness_with_wrong_release_reference_does_not_count() {
        let release = sample_release();
        let wrong_ref = ValidatedWitness {
            attestation: WitnessAttestation::new(
                AttestationDigest::from_canonical_bytes(b"wrong-release".to_vec()),
                "witness-a".to_string(),
                release.binary_digests.clone(),
                sample_env(),
            ),
            attestation_digest: AttestationDigest::from_canonical_bytes(b"witness-a-att".to_vec()),
            signer_key_name: "witness-a-key".to_string(),
        };
        let policy = ReleasePolicy::new(1, "witness_identity".to_string(), vec!["signer-1".to_string()], vec![
            "witness-a".to_string(),
        ]);

        let result = evaluate_policy(PolicyEvaluationInput {
            release,
            witnesses: vec![wrong_ref],
            policy,
            revocations: ReleaseRevocations::empty(),
        })
        .unwrap();

        assert_eq!(result.matching_witness_count, 0);
        assert_eq!(result.trust_tier.technical_class, TechnicalClass::SelfProofValid);
    }

    #[test]
    fn witness_with_wrong_rebuilt_digest_does_not_count() {
        let release = sample_release();
        let wrong_digests = ValidatedWitness {
            attestation: WitnessAttestation::new(
                release_attestation_canonical_digest(release.clone()).unwrap(),
                "witness-a".to_string(),
                vec![crate::release::BinaryDigest {
                    name: "crunch".to_string(),
                    algorithm: "blake3".to_string(),
                    digest: "ff".repeat(32),
                }],
                sample_env(),
            ),
            attestation_digest: AttestationDigest::from_canonical_bytes(b"witness-a-att".to_vec()),
            signer_key_name: "witness-a-key".to_string(),
        };
        let policy = ReleasePolicy::new(1, "witness_identity".to_string(), vec!["signer-1".to_string()], vec![
            "witness-a".to_string(),
        ]);

        let result = evaluate_policy(PolicyEvaluationInput {
            release,
            witnesses: vec![wrong_digests],
            policy,
            revocations: ReleaseRevocations::empty(),
        })
        .unwrap();

        assert_eq!(result.matching_witness_count, 0);
    }

    #[test]
    fn optional_policy_excludes_stale_mismatched_and_revoked_witnesses_without_failure() {
        let release = sample_release();
        let wrong_release = ValidatedWitness {
            attestation: WitnessAttestation::new(
                AttestationDigest::from_canonical_bytes(b"wrong-release".to_vec()),
                "witness-a".to_string(),
                release.binary_digests.clone(),
                sample_env(),
            ),
            attestation_digest: AttestationDigest::from_canonical_bytes(b"witness-a-att".to_vec()),
            signer_key_name: "witness-a-key".to_string(),
        };
        let wrong_rebuild = ValidatedWitness {
            attestation: WitnessAttestation::new(
                release_attestation_canonical_digest(release.clone()).unwrap(),
                "witness-b".to_string(),
                vec![crate::release::BinaryDigest {
                    name: "crunch".to_string(),
                    algorithm: "blake3".to_string(),
                    digest: "ff".repeat(TEST_DIGEST_BYTE_COUNT),
                }],
                sample_env(),
            ),
            attestation_digest: AttestationDigest::from_canonical_bytes(b"witness-b-att".to_vec()),
            signer_key_name: "witness-b-key".to_string(),
        };
        let revoked = make_matching_witness(&release, "witness-c");
        let revoked_key = revoked.signer_key_name.clone();
        let policy = ReleasePolicy::new(
            OPTIONAL_WITNESS_MINIMUM,
            INDEPENDENCE_FIELD_WITNESS_IDENTITY.to_string(),
            vec!["signer-1".to_string()],
            vec![
                "witness-a".to_string(),
                "witness-b".to_string(),
                "witness-c".to_string(),
            ],
        );

        let result = evaluate_policy(PolicyEvaluationInput {
            release,
            witnesses: vec![wrong_release, wrong_rebuild, revoked],
            policy,
            revocations: ReleaseRevocations::new(vec![revoked_key], Vec::new()),
        })
        .unwrap();

        assert_eq!(result.witness_quorum_status, IndependentAgreementStatus::NotRequired);
        assert_eq!(result.trust_tier.policy_status, PolicyStatus::Satisfied);
        assert_eq!(result.trust_tier.final_class, FinalClass::SelfProofValid);
        assert_eq!(result.matching_witness_count, 0);
        assert_eq!(result.revoked_witness_count, 1);
        assert!(result.policy_failure_reason.is_none());
    }

    #[test]
    fn policy_rejects_wrong_schema_tag() {
        let release = sample_release();
        let mut policy = sample_policy();
        policy.schema = "wrong".to_string();

        let err = evaluate_policy(PolicyEvaluationInput {
            release,
            witnesses: Vec::new(),
            policy,
            revocations: ReleaseRevocations::empty(),
        })
        .unwrap_err();
        assert_eq!(err, Error::SchemaTagMismatch {
            expected: RELEASE_POLICY_SCHEMA.to_string(),
            actual: "wrong".to_string(),
        });
    }

    #[test]
    fn revocations_rejects_wrong_schema_tag() {
        let release = sample_release();
        let policy = sample_policy();
        let mut rev = ReleaseRevocations::empty();
        rev.schema = "wrong".to_string();

        let err = evaluate_policy(PolicyEvaluationInput {
            release,
            witnesses: Vec::new(),
            policy,
            revocations: rev,
        })
        .unwrap_err();
        assert_eq!(err, Error::SchemaTagMismatch {
            expected: RELEASE_REVOCATIONS_SCHEMA.to_string(),
            actual: "wrong".to_string(),
        });
    }

    #[test]
    fn policy_rejects_unsupported_independence_field() {
        let release = sample_release();
        let mut policy = sample_policy();
        policy.independence_field = "rebuild_environment_summary.system".to_string();

        let err = evaluate_policy(PolicyEvaluationInput {
            release,
            witnesses: Vec::new(),
            policy,
            revocations: ReleaseRevocations::empty(),
        })
        .unwrap_err();
        assert_eq!(err, Error::UnsupportedPolicyField {
            field: "independence_field".to_string(),
            value: "rebuild_environment_summary.system".to_string(),
        });
    }

    fn sample_release() -> ReleaseAttestation {
        ReleaseAttestation::new(crate::release::ReleaseAttestationInit {
            release_id: "mantle-0.1.0".to_string(),
            release_evidence_manifest_digest_blake3: AttestationDigest::from_canonical_bytes(b"manifest".to_vec()),
            proof_bundle_digest_blake3: AttestationDigest::from_canonical_bytes(b"proof".to_vec()),
            proof_mode: "fixed-point".to_string(),
            declared_effect_claims: None,
            observed_effect_facts: None,
            workflow: Workflow {
                command: "crunch self-build".to_string(),
                version: "0.1.0".to_string(),
            },
            binary_digests: vec![crate::release::BinaryDigest {
                name: "crunch".to_string(),
                algorithm: "blake3".to_string(),
                digest: "aa".repeat(32),
            }],
        })
    }

    fn sample_policy() -> ReleasePolicy {
        ReleasePolicy::new(
            2,
            INDEPENDENCE_FIELD_WITNESS_IDENTITY.to_string(),
            vec!["release-signer-1".to_string()],
            vec!["witness-a".to_string(), "witness-b".to_string()],
        )
    }

    fn sample_revocations() -> ReleaseRevocations {
        ReleaseRevocations::new(vec!["witness-a".to_string()], vec!["aa".repeat(32)])
    }

    fn sample_env() -> RebuildEnvironmentSummary {
        sample_env_with_host_class("nixos-25.05")
    }

    fn sample_env_with_host_class(host_class: &str) -> RebuildEnvironmentSummary {
        RebuildEnvironmentSummary {
            system: "x86_64-linux".to_string(),
            toolchain: "rust-1.91.1".to_string(),
            host_class: host_class.to_string(),
        }
    }

    fn make_matching_witness(release: &ReleaseAttestation, identity: &str) -> ValidatedWitness {
        make_matching_witness_with_details(
            release,
            identity,
            &alloc::format!("{identity}-key"),
            "nixos-25.05",
            identity.as_bytes(),
        )
    }

    fn make_matching_witness_with_details(
        release: &ReleaseAttestation,
        identity: &str,
        signer_key_name: &str,
        host_class: &str,
        digest_seed: &[u8],
    ) -> ValidatedWitness {
        let release_digest = release_attestation_canonical_digest(release.clone()).unwrap();
        let attestation = WitnessAttestation::new(
            release_digest,
            identity.to_string(),
            release.binary_digests.clone(),
            sample_env_with_host_class(host_class),
        );
        let attestation_digest = AttestationDigest::from_canonical_bytes(digest_seed.to_vec());
        ValidatedWitness {
            attestation,
            attestation_digest,
            signer_key_name: signer_key_name.to_string(),
        }
    }
}
