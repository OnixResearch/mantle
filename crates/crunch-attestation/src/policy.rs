use std::collections::BTreeSet;

use serde::Deserialize;
use serde::Serialize;

use crate::AttestationDigest;
use crate::Canonicalize;
use crate::Error;
use crate::release::BinaryDigest;
use crate::release::FinalClass;
use crate::release::PolicyStatus;
use crate::release::ReleaseAttestation;
use crate::release::TechnicalClass;
use crate::release::TrustTier;
use crate::release::WitnessAttestation;
use crate::release::binary_digests_match;

pub const RELEASE_POLICY_SCHEMA: &str = "crunch-release-policy-v1";
pub const RELEASE_REVOCATIONS_SCHEMA: &str = "crunch-release-revocations-v1";

const MAX_SIGNER_COUNT: u32 = 256;
const MAX_REVOCATION_COUNT: u32 = 4_096;

// ---------------------------------------------------------------------------
// Policy schema
// ---------------------------------------------------------------------------

/// File-based social trust policy for release verification.
///
/// Loaded from `policy.json` in the verification directory. Policy fields
/// stay external to attestation digests: changing this file never changes
/// any release or witness attestation digest.
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

// ---------------------------------------------------------------------------
// Revocations schema
// ---------------------------------------------------------------------------

/// File-based revocation input for release verification.
///
/// Loaded from `revocations.json` in the verification directory.
/// Applied before quorum and independence evaluation.
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

// ---------------------------------------------------------------------------
// Validated witness
// ---------------------------------------------------------------------------

/// A witness attestation that has passed signature and schema validation.
///
/// The policy evaluator works with these, not raw `WitnessAttestation`s,
/// so callers must validate signatures before policy evaluation.
#[derive(Clone, Debug)]
pub struct ValidatedWitness {
    pub attestation: WitnessAttestation,
    pub attestation_digest: AttestationDigest,
    pub signer_key_name: String,
}

// ---------------------------------------------------------------------------
// Policy evaluation result
// ---------------------------------------------------------------------------

/// Detailed policy evaluation result.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PolicyEvaluation {
    pub trust_tier: TrustTier,
    pub matching_witness_count: u32,
    pub independent_witness_identities: u32,
    pub revoked_witness_count: u32,
    pub policy_failure_reason: Option<PolicyFailureReason>,
}

/// Why policy evaluation failed.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PolicyFailureReason {
    InsufficientQuorum { required: u32, matched: u32 },
    InsufficientIndependence { distinct_identities: u32, required: u32 },
}

// ---------------------------------------------------------------------------
// Policy evaluator
// ---------------------------------------------------------------------------

/// Evaluate social trust policy over a release attestation and its witnesses.
///
/// Steps:
/// 1. Apply revocations to filter out revoked witnesses.
/// 2. Check each remaining witness against the release attestation digest and binary digest set.
/// 3. Count matching witnesses and distinct witness identities.
/// 4. Determine technical class from witness match count.
/// 5. Evaluate quorum and independence policy.
/// 6. Resolve final class.
pub fn evaluate_policy(
    release: &ReleaseAttestation,
    witnesses: &[ValidatedWitness],
    policy: &ReleasePolicy,
    revocations: &ReleaseRevocations,
) -> Result<PolicyEvaluation, Error> {
    assert_policy_quorum(policy);
    validate_policy(policy)?;
    validate_revocations(revocations)?;

    let release_digest = release.canonical_digest()?;
    let release_binary_digests = normalize_for_comparison(&release.binary_digests);
    let (revoked_count, active_witnesses) = filter_active_witnesses(witnesses, revocations);
    let (matching_count, distinct_identities) =
        count_matching_witnesses(&release_digest, &release_binary_digests, &active_witnesses);
    assert_matching_witness_counts(matching_count, active_witnesses.len(), distinct_identities);
    assert!(policy.min_matching_witnesses >= 1, "policy quorum must be at least one witness");
    let active_witness_count = u32::try_from(active_witnesses.len()).unwrap_or(u32::MAX);
    assert!(matching_count <= active_witness_count, "matching count must fit active witnesses");

    let technical_class = if matching_count > 0 {
        TechnicalClass::ExternalWitnessMatch
    } else {
        TechnicalClass::SelfProofValid
    };
    let quorum_met = matching_count >= policy.min_matching_witnesses;
    let independence_met = distinct_identities >= policy.min_matching_witnesses;

    let (policy_status, failure_reason) = if !quorum_met {
        (
            PolicyStatus::Insufficient,
            Some(PolicyFailureReason::InsufficientQuorum {
                required: policy.min_matching_witnesses,
                matched: matching_count,
            }),
        )
    } else if !independence_met {
        (
            PolicyStatus::Insufficient,
            Some(PolicyFailureReason::InsufficientIndependence {
                distinct_identities,
                required: policy.min_matching_witnesses,
            }),
        )
    } else {
        (PolicyStatus::Satisfied, None)
    };

    Ok(PolicyEvaluation {
        trust_tier: TrustTier {
            technical_class,
            policy_status,
            final_class: FinalClass::resolve(technical_class, policy_status),
        },
        matching_witness_count: matching_count,
        independent_witness_identities: distinct_identities,
        revoked_witness_count: revoked_count,
        policy_failure_reason: failure_reason,
    })
}

// ---------------------------------------------------------------------------
// Validation helpers
// ---------------------------------------------------------------------------

fn filter_active_witnesses<'a>(
    witnesses: &'a [ValidatedWitness],
    revocations: &ReleaseRevocations,
) -> (u32, Vec<&'a ValidatedWitness>) {
    let revoked_keys: BTreeSet<&str> = revocations.revoked_witness_keys.iter().map(|key| key.as_str()).collect();
    let revoked_digests: BTreeSet<&str> = revocations
        .revoked_witness_attestation_digests_blake3
        .iter()
        .map(|digest| digest.as_str())
        .collect();
    let mut revoked_count: u32 = 0;
    let mut active_witnesses: Vec<&ValidatedWitness> = Vec::new();

    for witness in witnesses {
        let key_revoked = revoked_keys.contains(witness.signer_key_name.as_str());
        let digest_hex = witness.attestation_digest.to_hex();
        let digest_revoked = revoked_digests.contains(digest_hex.as_str());
        if key_revoked || digest_revoked {
            revoked_count = revoked_count.saturating_add(1);
            continue;
        }
        active_witnesses.push(witness);
    }

    (revoked_count, active_witnesses)
}

fn count_matching_witnesses(
    release_digest: &AttestationDigest,
    release_binary_digests: &[BinaryDigest],
    active_witnesses: &[&ValidatedWitness],
) -> (u32, u32) {
    let mut matching_count: u32 = 0;
    let mut matching_identities: BTreeSet<&str> = BTreeSet::new();

    for witness in active_witnesses {
        let digest_matches = witness.attestation.release_attestation_digest_blake3 == *release_digest;
        let rebuilt = normalize_for_comparison(&witness.attestation.rebuilt_digests);
        if !digest_matches || !binary_digests_match(release_binary_digests, &rebuilt) {
            continue;
        }
        matching_count = matching_count.saturating_add(1);
        matching_identities.insert(&witness.attestation.witness_identity);
    }

    let distinct_identities = u32::try_from(matching_identities.len()).unwrap_or(u32::MAX);
    (matching_count, distinct_identities)
}

fn assert_policy_quorum(policy: &ReleasePolicy) {
    assert!(policy.min_matching_witnesses >= 1, "policy quorum must be at least one witness");
    assert!(
        usize::try_from(policy.min_matching_witnesses).is_ok(),
        "policy quorum must fit in usize"
    );
}

fn assert_matching_witness_counts(matching_count: u32, active_witness_count: usize, distinct_identities: u32) {
    let active_witness_count_u32 = u32::try_from(active_witness_count).unwrap_or(u32::MAX);
    assert!(matching_count <= active_witness_count_u32, "matching witness count must stay bounded");
    assert!(distinct_identities <= matching_count, "distinct witness identities must not exceed matches");
}

fn validate_policy(policy: &ReleasePolicy) -> Result<(), Error> {
    if policy.schema != RELEASE_POLICY_SCHEMA {
        return Err(Error::SchemaTagMismatch {
            expected: RELEASE_POLICY_SCHEMA,
            actual: policy.schema.clone(),
        });
    }
    validate_signer_list(&policy.trusted_release_signers, "trusted_release_signers")?;
    validate_signer_list(&policy.trusted_witness_signers, "trusted_witness_signers")?;
    if policy.independence_field.is_empty() {
        return Err(Error::EmptyField {
            field: "independence_field",
        });
    }
    Ok(())
}

fn validate_revocations(revocations: &ReleaseRevocations) -> Result<(), Error> {
    assert!(!RELEASE_REVOCATIONS_SCHEMA.is_empty(), "revocation schema tag must not be empty");
    assert!(MAX_REVOCATION_COUNT >= 1, "revocation limit must be positive");
    if revocations.schema != RELEASE_REVOCATIONS_SCHEMA {
        return Err(Error::SchemaTagMismatch {
            expected: RELEASE_REVOCATIONS_SCHEMA,
            actual: revocations.schema.clone(),
        });
    }
    let key_count = u32::try_from(revocations.revoked_witness_keys.len()).unwrap_or(u32::MAX);
    if key_count > MAX_REVOCATION_COUNT {
        return Err(Error::CollectionTooLarge {
            limit: MAX_REVOCATION_COUNT,
            actual: key_count,
        });
    }
    let digest_count = u32::try_from(revocations.revoked_witness_attestation_digests_blake3.len()).unwrap_or(u32::MAX);
    if digest_count > MAX_REVOCATION_COUNT {
        return Err(Error::CollectionTooLarge {
            limit: MAX_REVOCATION_COUNT,
            actual: digest_count,
        });
    }
    Ok(())
}

fn validate_signer_list(signers: &[String], _field: &'static str) -> Result<(), Error> {
    let count = u32::try_from(signers.len()).unwrap_or(u32::MAX);
    if count > MAX_SIGNER_COUNT {
        return Err(Error::CollectionTooLarge {
            limit: MAX_SIGNER_COUNT,
            actual: count,
        });
    }
    Ok(())
}

fn normalize_for_comparison(digests: &[BinaryDigest]) -> Vec<BinaryDigest> {
    let mut sorted = digests.to_vec();
    sorted.sort();
    sorted
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::release::RebuildEnvironmentSummary;
    use crate::release::Workflow;

    // -- Policy schema serde -----------------------------------------------

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

    // -- Policy does not affect attestation digests ------------------------

    #[test]
    fn policy_change_does_not_affect_attestation_digests() {
        let release = sample_release();
        let digest_before = release.canonical_digest().unwrap();

        let _policy_loose = ReleasePolicy::new(1, "witness_identity".to_string(), vec!["signer-1".to_string()], vec![
            "witness-a".to_string(),
        ]);
        let _policy_strict = ReleasePolicy::new(3, "witness_identity".to_string(), vec!["signer-1".to_string()], vec![
            "witness-a".to_string(),
            "witness-b".to_string(),
            "witness-c".to_string(),
        ]);

        let digest_after = release.canonical_digest().unwrap();
        assert_eq!(digest_before, digest_after);
    }

    // -- Zero witnesses → self-proof-valid ---------------------------------

    #[test]
    fn zero_witnesses_yield_self_proof_valid() {
        let release = sample_release();
        let policy = ReleasePolicy::new(0, "witness_identity".to_string(), vec!["signer-1".to_string()], Vec::new());
        let revocations = ReleaseRevocations::empty();

        let result = evaluate_policy(&release, &[], &policy, &revocations).unwrap();

        assert_eq!(result.trust_tier.technical_class, TechnicalClass::SelfProofValid);
        assert_eq!(result.trust_tier.policy_status, PolicyStatus::Satisfied);
        assert_eq!(result.trust_tier.final_class, FinalClass::QuorumSatisfied);
        assert_eq!(result.matching_witness_count, 0);
    }

    // -- One matching witness → external-witness-match ---------------------

    #[test]
    fn one_matching_witness_raises_technical_class() {
        let release = sample_release();
        let witness = make_matching_witness(&release, "witness-a");
        let policy = ReleasePolicy::new(1, "witness_identity".to_string(), vec!["signer-1".to_string()], vec![
            "witness-a".to_string(),
        ]);
        let revocations = ReleaseRevocations::empty();

        let result = evaluate_policy(&release, &[witness], &policy, &revocations).unwrap();

        assert_eq!(result.trust_tier.technical_class, TechnicalClass::ExternalWitnessMatch);
        assert_eq!(result.matching_witness_count, 1);
    }

    // -- Insufficient quorum -----------------------------------------------

    #[test]
    fn insufficient_quorum_fails_policy_with_technical_success() {
        let release = sample_release();
        let witness = make_matching_witness(&release, "witness-a");
        let policy = ReleasePolicy::new(2, "witness_identity".to_string(), vec!["signer-1".to_string()], vec![
            "witness-a".to_string(),
        ]);
        let revocations = ReleaseRevocations::empty();

        let result = evaluate_policy(&release, &[witness], &policy, &revocations).unwrap();

        assert_eq!(result.trust_tier.technical_class, TechnicalClass::ExternalWitnessMatch);
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

    // -- Satisfied quorum → quorum-satisfied -------------------------------

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

        let result = evaluate_policy(&release, &[w_a, w_b], &policy, &revocations).unwrap();

        assert_eq!(result.trust_tier.policy_status, PolicyStatus::Satisfied);
        assert_eq!(result.trust_tier.final_class, FinalClass::QuorumSatisfied);
        assert_eq!(result.matching_witness_count, 2);
        assert_eq!(result.independent_witness_identities, 2);
    }

    // -- Same actor cannot satisfy independence ----------------------------

    #[test]
    fn same_identity_cannot_satisfy_independence() {
        let release = sample_release();
        // Two witness attestations from the same identity.
        let w1 = make_matching_witness(&release, "witness-a");
        let w2 = ValidatedWitness {
            attestation: WitnessAttestation::new(
                release.canonical_digest().unwrap(),
                "witness-a".to_string(),
                release.binary_digests.clone(),
                RebuildEnvironmentSummary {
                    system: "x86_64-linux".to_string(),
                    toolchain: "rust-1.91.1".to_string(),
                    host_class: "other-host".to_string(),
                },
            ),
            attestation_digest: AttestationDigest::from_canonical_bytes(b"witness-a-second"),
            signer_key_name: "witness-a-key-2".to_string(),
        };
        let policy = ReleasePolicy::new(2, "witness_identity".to_string(), vec!["signer-1".to_string()], vec![
            "witness-a".to_string(),
        ]);
        let revocations = ReleaseRevocations::empty();

        let result = evaluate_policy(&release, &[w1, w2], &policy, &revocations).unwrap();

        // Two matching witnesses but only one distinct identity.
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

    // -- Revoked witness key degrades policy -------------------------------

    #[test]
    fn revoked_key_degrades_policy_without_changing_technical_result() {
        let release = sample_release();
        let w_a = make_matching_witness(&release, "witness-a");
        let w_b = make_matching_witness(&release, "witness-b");
        let policy = ReleasePolicy::new(2, "witness_identity".to_string(), vec!["signer-1".to_string()], vec![
            "witness-a".to_string(),
            "witness-b".to_string(),
        ]);

        // Without revocation: satisfied.
        let no_rev = ReleaseRevocations::empty();
        let before = evaluate_policy(&release, &[w_a.clone(), w_b.clone()], &policy, &no_rev).unwrap();
        assert_eq!(before.trust_tier.policy_status, PolicyStatus::Satisfied);

        // Revoke witness-a's key.
        let rev = ReleaseRevocations::new(vec!["witness-a-key".to_string()], Vec::new());
        let after = evaluate_policy(&release, &[w_a, w_b], &policy, &rev).unwrap();

        // Technical class unchanged (one remaining match).
        assert_eq!(after.trust_tier.technical_class, TechnicalClass::ExternalWitnessMatch);
        // Policy now insufficient.
        assert_eq!(after.trust_tier.policy_status, PolicyStatus::Insufficient);
        assert_eq!(after.revoked_witness_count, 1);
        assert_eq!(after.matching_witness_count, 1);
    }

    // -- Revoked witness digest degrades policy ----------------------------

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
        let rev = ReleaseRevocations::new(Vec::new(), vec![revoked_digest]);

        let result = evaluate_policy(&release, &[w_a, w_b], &policy, &rev).unwrap();

        assert_eq!(result.revoked_witness_count, 1);
        assert_eq!(result.matching_witness_count, 1);
        assert_eq!(result.trust_tier.policy_status, PolicyStatus::Insufficient);
    }

    // -- Revocations applied before quorum counting ------------------------

    #[test]
    fn revocations_applied_before_quorum_counting() {
        let release = sample_release();
        let w_a = make_matching_witness(&release, "witness-a");
        let w_b = make_matching_witness(&release, "witness-b");
        let policy = ReleasePolicy::new(2, "witness_identity".to_string(), vec!["signer-1".to_string()], vec![
            "witness-a".to_string(),
            "witness-b".to_string(),
        ]);
        // Revoke both.
        let rev = ReleaseRevocations::new(vec!["witness-a-key".to_string(), "witness-b-key".to_string()], Vec::new());

        let result = evaluate_policy(&release, &[w_a, w_b], &policy, &rev).unwrap();

        assert_eq!(result.revoked_witness_count, 2);
        assert_eq!(result.matching_witness_count, 0);
        assert_eq!(result.trust_tier.technical_class, TechnicalClass::SelfProofValid);
        assert_eq!(result.trust_tier.policy_status, PolicyStatus::Insufficient);
    }

    // -- Mismatched witness does not count ---------------------------------

    #[test]
    fn witness_with_wrong_release_reference_does_not_count() {
        let release = sample_release();
        let wrong_ref = ValidatedWitness {
            attestation: WitnessAttestation::new(
                AttestationDigest::from_canonical_bytes(b"wrong-release"),
                "witness-a".to_string(),
                release.binary_digests.clone(),
                sample_env(),
            ),
            attestation_digest: AttestationDigest::from_canonical_bytes(b"witness-a-att"),
            signer_key_name: "witness-a-key".to_string(),
        };
        let policy = ReleasePolicy::new(1, "witness_identity".to_string(), vec!["signer-1".to_string()], vec![
            "witness-a".to_string(),
        ]);
        let rev = ReleaseRevocations::empty();

        let result = evaluate_policy(&release, &[wrong_ref], &policy, &rev).unwrap();

        assert_eq!(result.matching_witness_count, 0);
        assert_eq!(result.trust_tier.technical_class, TechnicalClass::SelfProofValid);
    }

    #[test]
    fn witness_with_wrong_rebuilt_digest_does_not_count() {
        let release = sample_release();
        let wrong_digests = ValidatedWitness {
            attestation: WitnessAttestation::new(
                release.canonical_digest().unwrap(),
                "witness-a".to_string(),
                vec![crate::release::BinaryDigest {
                    name: "crunch".to_string(),
                    algorithm: "blake3".to_string(),
                    digest: "ff".repeat(32),
                }],
                sample_env(),
            ),
            attestation_digest: AttestationDigest::from_canonical_bytes(b"witness-a-att"),
            signer_key_name: "witness-a-key".to_string(),
        };
        let policy = ReleasePolicy::new(1, "witness_identity".to_string(), vec!["signer-1".to_string()], vec![
            "witness-a".to_string(),
        ]);
        let rev = ReleaseRevocations::empty();

        let result = evaluate_policy(&release, &[wrong_digests], &policy, &rev).unwrap();

        assert_eq!(result.matching_witness_count, 0);
    }

    // -- Schema validation -------------------------------------------------

    #[test]
    fn policy_rejects_wrong_schema_tag() {
        let release = sample_release();
        let mut policy = sample_policy();
        policy.schema = "wrong".to_string();

        let err = evaluate_policy(&release, &[], &policy, &ReleaseRevocations::empty()).unwrap_err();
        assert_eq!(err, Error::SchemaTagMismatch {
            expected: RELEASE_POLICY_SCHEMA,
            actual: "wrong".to_string(),
        });
    }

    #[test]
    fn revocations_rejects_wrong_schema_tag() {
        let release = sample_release();
        let policy = sample_policy();
        let mut rev = ReleaseRevocations::empty();
        rev.schema = "wrong".to_string();

        let err = evaluate_policy(&release, &[], &policy, &rev).unwrap_err();
        assert_eq!(err, Error::SchemaTagMismatch {
            expected: RELEASE_REVOCATIONS_SCHEMA,
            actual: "wrong".to_string(),
        });
    }

    // -- Helpers -----------------------------------------------------------

    fn sample_release() -> ReleaseAttestation {
        ReleaseAttestation::new(
            "crunch-0.1.0".to_string(),
            AttestationDigest::from_canonical_bytes(b"manifest"),
            AttestationDigest::from_canonical_bytes(b"proof"),
            "fixed-point".to_string(),
            Workflow {
                command: "crunch self-build".to_string(),
                version: "0.1.0".to_string(),
            },
            vec![crate::release::BinaryDigest {
                name: "crunch".to_string(),
                algorithm: "blake3".to_string(),
                digest: "aa".repeat(32),
            }],
        )
    }

    fn sample_policy() -> ReleasePolicy {
        ReleasePolicy::new(2, "witness_identity".to_string(), vec!["release-signer-1".to_string()], vec![
            "witness-a".to_string(),
            "witness-b".to_string(),
        ])
    }

    fn sample_revocations() -> ReleaseRevocations {
        ReleaseRevocations::new(vec!["witness-a".to_string()], vec!["aa".repeat(32)])
    }

    fn sample_env() -> RebuildEnvironmentSummary {
        RebuildEnvironmentSummary {
            system: "x86_64-linux".to_string(),
            toolchain: "rust-1.91.1".to_string(),
            host_class: "nixos-25.05".to_string(),
        }
    }

    fn make_matching_witness(release: &ReleaseAttestation, identity: &str) -> ValidatedWitness {
        let release_digest = release.canonical_digest().unwrap();
        let attestation =
            WitnessAttestation::new(release_digest, identity.to_string(), release.binary_digests.clone(), sample_env());
        let attestation_digest = attestation.canonical_digest().unwrap();
        ValidatedWitness {
            attestation,
            attestation_digest,
            signer_key_name: format!("{identity}-key"),
        }
    }
}
