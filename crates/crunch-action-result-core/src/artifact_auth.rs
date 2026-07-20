//! Pure compatibility adapter from Mantle action-result observations into artifact-auth.

use std::collections::BTreeSet;

use artifact_auth_core::ALGORITHM_BLAKE3;
use artifact_auth_core::ALGORITHM_SHA256;
use artifact_auth_core::ArtifactRef;
use artifact_auth_core::ArtifactStatement;
use artifact_auth_core::AuthenticationDecision;
use artifact_auth_core::AuthenticationPolicy;
use artifact_auth_core::AuthenticationScope;
use artifact_auth_core::CryptographicObservation;
use artifact_auth_core::ED25519_PUBLIC_KEY_PROFILE_V1;
use artifact_auth_core::KeyCurrentness;
use artifact_auth_core::POLICY_SCHEMA_V1;
use artifact_auth_core::STATEMENT_SCHEMA_V1;
use artifact_auth_core::SignatureEvidence;
use artifact_auth_core::TrustedKeyObservation;
use artifact_auth_core::evaluate_authentication;
use artifact_auth_core::required_non_claims as standalone_non_claims;

use crate::ActionResultRecord;
use crate::CandidateDecision;
use crate::OBJECT_REF_PREFIX;
use crate::PUBLICATION_POLICY_REF_PREFIX;
use crate::validate_action_result;

const ACTION_RESULT_REF_PREFIX: &str = "mantle-action-result://blake3/";
const ARTIFACT_AUTH_DOMAIN: &str = "mantle.action-result.artifact-auth.v1";
const ARTIFACT_AUTH_PURPOSE: &str = "action-result-record";
const ACTION_RESULT_PROFILE: &str = "mantle-action-result.v1";
const OBJECT_PROFILE: &str = "mantle-object.v1";
const PUBLICATION_POLICY_PROFILE: &str = "mantle-publication-policy.v1";
const OCI_MANIFEST_PROFILE: &str = "oci-manifest.v1";
const OCI_METADATA_PROFILE: &str = "oci-metadata-manifest.v1";
const KEY_CURRENTNESS_PROFILE: &str = "mantle-key-currentness.v1";
const DIGEST_HEX_CHARS: usize = 64;
const OCI_PARENT_COUNT: usize = 2;
const PREIMAGE_CLASS: &str = "mantle-action-result-preimage-vs-artifact-auth-statement";
const AUTHORITY_BOUNDARY: &str = "standalone authentication is diagnostic input only; Mantle retains OCI construction, repository authorization, registry routing, credentials, signing, cache/build admission, receipt composition, and release authority";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MantleSignerObservation {
    pub producer_id: String,
    pub key_id: String,
    pub key_identity_blake3: String,
    pub generation: u64,
    pub currentness: KeyCurrentness,
    pub currentness_blake3: String,
    pub standalone_cryptographic: CryptographicObservation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MantleArtifactAuthStatementInput<'a> {
    pub profile_id: &'a str,
    pub record: &'a ActionResultRecord,
    pub producer_id: &'a str,
    pub key_id: &'a str,
    pub key_identity_blake3: &'a str,
    pub oci_manifest_sha256: Option<&'a str>,
    pub metadata_manifest_sha256: Option<&'a str>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MantleArtifactAuthObservation<'a> {
    pub profile_id: &'a str,
    pub record: &'a ActionResultRecord,
    pub legacy: &'a CandidateDecision,
    pub signers: &'a [MantleSignerObservation],
    pub required_signer_labels: &'a [String],
    pub threshold: u16,
    pub oci_manifest_sha256: Option<&'a str>,
    pub metadata_manifest_sha256: Option<&'a str>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MantleArtifactAuthCompatibility {
    pub case_explained: bool,
    pub preimage_class: String,
    pub identity_drift_explained: bool,
    pub decision_drift: bool,
    pub issue_class: String,
    pub legacy_issue_causes: Vec<String>,
    pub standalone_issue_causes: Vec<String>,
    pub non_claim_drift: bool,
    pub blockers: Vec<String>,
    pub legacy_authoritative: bool,
    pub standalone_authority_admitted: bool,
    pub rollback_available: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MantleArtifactAuthReport {
    pub standalone: Option<AuthenticationDecision>,
    pub compatibility: MantleArtifactAuthCompatibility,
    pub nix_key_identities: Vec<String>,
    pub oci_manifest_sha256: Option<String>,
    pub metadata_manifest_sha256: Option<String>,
    pub repository_authority_retained: bool,
    pub build_authority_retained: bool,
    pub release_authority_retained: bool,
    pub authority_boundary: String,
}

// r[impl mantle.artifact_auth_shell.exact_verification]
/// Map one action-result signer to the exact standalone statement preimage.
///
/// This pure mapping never consumes legacy or standalone cryptographic results.
pub fn map_mantle_artifact_auth_statement(
    input: &MantleArtifactAuthStatementInput<'_>,
) -> Result<ArtifactStatement, Vec<String>> {
    validate_action_result(input.record).map_err(|error| vec![error])?;
    if input.profile_id.is_empty() {
        return Err(vec!["profile-id-empty".to_string()]);
    }
    if input.producer_id.is_empty() {
        return Err(vec!["producer-id-empty".to_string()]);
    }
    if input.key_id.is_empty() {
        return Err(vec!["key-id-empty".to_string()]);
    }
    if !valid_digest(input.key_identity_blake3) {
        return Err(vec!["key-identity-malformed".to_string()]);
    }
    let scope = map_scope(input.profile_id, input.record, input.oci_manifest_sha256, input.metadata_manifest_sha256)?;
    let statement = ArtifactStatement {
        schema: STATEMENT_SCHEMA_V1.to_string(),
        scope,
        producer_id: input.producer_id.to_string(),
        key_id: input.key_id.to_string(),
        key_identity: ArtifactRef {
            profile: ED25519_PUBLIC_KEY_PROFILE_V1.to_string(),
            algorithm: ALGORITHM_BLAKE3.to_string(),
            digest_hex: input.key_identity_blake3.to_string(),
        },
    };
    debug_assert_eq!(statement.producer_id, input.producer_id);
    debug_assert_eq!(statement.key_id, input.key_id);
    Ok(statement)
}

// r[impl mantle.artifact_auth_adoption.authority]
// r[impl mantle.artifact_auth_adoption.cutover]
#[must_use]
pub fn evaluate_mantle_artifact_auth(observation: &MantleArtifactAuthObservation<'_>) -> MantleArtifactAuthReport {
    let mapped = map_observation(observation);
    let standalone = mapped
        .as_ref()
        .ok()
        .map(|(policy, scope, evidence)| evaluate_authentication(policy, scope, evidence));
    let mapping_blockers = mapped.err().unwrap_or_default();
    let compatibility = compare(observation, standalone.as_ref(), mapping_blockers);
    let result = MantleArtifactAuthReport {
        standalone,
        compatibility,
        nix_key_identities: observation.signers.iter().map(|signer| signer.key_id.clone()).collect(),
        oci_manifest_sha256: observation.oci_manifest_sha256.map(str::to_string),
        metadata_manifest_sha256: observation.metadata_manifest_sha256.map(str::to_string),
        repository_authority_retained: true,
        build_authority_retained: true,
        release_authority_retained: true,
        authority_boundary: AUTHORITY_BOUNDARY.to_string(),
    };
    debug_assert!(result.compatibility.legacy_authoritative);
    debug_assert!(!result.compatibility.standalone_authority_admitted);
    result
}

type MappedAuthentication = (AuthenticationPolicy, AuthenticationScope, Vec<SignatureEvidence>);

struct TypedBlake3RefInput<'a> {
    profile: &'a str,
    prefix: &'a str,
    value: &'a str,
}

struct Sha256RefInput<'a> {
    profile: &'a str,
    digest_hex: &'a str,
}

fn map_observation(observation: &MantleArtifactAuthObservation<'_>) -> Result<MappedAuthentication, Vec<String>> {
    validate_mapping_inputs(observation)?;
    let scope = map_scope(
        observation.profile_id,
        observation.record,
        observation.oci_manifest_sha256,
        observation.metadata_manifest_sha256,
    )?;
    let (trusted_keys, evidence) = map_signers(observation)?;
    let policy = AuthenticationPolicy {
        schema: POLICY_SCHEMA_V1.to_string(),
        profile_id: observation.profile_id.to_string(),
        threshold: observation.threshold,
        trusted_keys,
    };
    debug_assert!(policy.threshold > 0);
    debug_assert_eq!(evidence.len(), observation.signers.len());
    Ok((policy, scope, evidence))
}

fn validate_mapping_inputs(observation: &MantleArtifactAuthObservation<'_>) -> Result<(), Vec<String>> {
    validate_action_result(observation.record).map_err(|error| vec![error])?;
    if observation.legacy.result_ref != observation.record.result_ref {
        return Err(vec!["legacy-result-ref-mismatch".to_string()]);
    }
    if observation.profile_id.is_empty() {
        return Err(vec!["profile-id-empty".to_string()]);
    }
    if observation.threshold == 0 || usize::from(observation.threshold) > observation.signers.len() {
        return Err(vec!["threshold-invalid".to_string()]);
    }
    let signer_labels = observation.signers.iter().map(|signer| signer.key_id.as_str()).collect::<BTreeSet<_>>();
    if signer_labels.len() != observation.signers.len() {
        return Err(vec!["duplicate-signer-label".to_string()]);
    }
    if observation.required_signer_labels.iter().any(|required| !signer_labels.contains(required.as_str())) {
        return Err(vec!["required-signer-label-missing".to_string()]);
    }
    debug_assert!(!observation.profile_id.is_empty());
    debug_assert_eq!(signer_labels.len(), observation.signers.len());
    Ok(())
}

fn map_scope(
    profile_id: &str,
    record: &ActionResultRecord,
    oci_manifest_sha256: Option<&str>,
    metadata_manifest_sha256: Option<&str>,
) -> Result<AuthenticationScope, Vec<String>> {
    let subject = typed_blake3_ref(TypedBlake3RefInput {
        profile: ACTION_RESULT_PROFILE,
        prefix: ACTION_RESULT_REF_PREFIX,
        value: &record.result_ref,
    })?;
    let verifier_context = typed_blake3_ref(TypedBlake3RefInput {
        profile: PUBLICATION_POLICY_PROFILE,
        prefix: PUBLICATION_POLICY_REF_PREFIX,
        value: &record.publication_policy_ref,
    })?;
    let scope = AuthenticationScope {
        domain: ARTIFACT_AUTH_DOMAIN.to_string(),
        purpose: ARTIFACT_AUTH_PURPOSE.to_string(),
        profile_id: profile_id.to_string(),
        subject,
        parents: map_parent_identities(record, oci_manifest_sha256, metadata_manifest_sha256)?,
        verifier_context,
    };
    debug_assert_eq!(scope.profile_id, profile_id);
    Ok(scope)
}

fn map_parent_identities(
    record: &ActionResultRecord,
    oci_manifest_sha256: Option<&str>,
    metadata_manifest_sha256: Option<&str>,
) -> Result<Vec<ArtifactRef>, Vec<String>> {
    let mut parents = Vec::with_capacity(record.outputs.len().saturating_add(OCI_PARENT_COUNT));
    let mut parent_identities = BTreeSet::new();
    for output in &record.outputs {
        let parent = typed_blake3_ref(TypedBlake3RefInput {
            profile: OBJECT_PROFILE,
            prefix: OBJECT_REF_PREFIX,
            value: &output.object_ref,
        })?;
        if !parent_identities.insert(parent.digest_hex.clone()) {
            return Err(vec!["parent-object-identity-duplicate".to_string()]);
        }
        parents.push(parent);
    }
    match (oci_manifest_sha256, metadata_manifest_sha256) {
        (Some(oci), Some(metadata)) => {
            parents.push(sha256_ref(Sha256RefInput {
                profile: OCI_MANIFEST_PROFILE,
                digest_hex: oci,
            })?);
            parents.push(sha256_ref(Sha256RefInput {
                profile: OCI_METADATA_PROFILE,
                digest_hex: metadata,
            })?);
        }
        (None, None) => {}
        _ => return Err(vec!["oci-manifest-pair-incomplete".to_string()]),
    }
    debug_assert_eq!(parent_identities.len(), record.outputs.len());
    debug_assert!(parents.len() >= record.outputs.len());
    Ok(parents)
}

fn map_signers(
    observation: &MantleArtifactAuthObservation<'_>,
) -> Result<(Vec<TrustedKeyObservation>, Vec<SignatureEvidence>), Vec<String>> {
    let mut trusted_keys = Vec::with_capacity(observation.signers.len());
    let mut evidence = Vec::with_capacity(observation.signers.len());
    let mut full_key_identities = BTreeSet::new();
    for signer in observation.signers {
        if !valid_digest(&signer.key_identity_blake3) || !valid_digest(&signer.currentness_blake3) {
            return Err(vec!["signer-identity-or-currentness-malformed".to_string()]);
        }
        if !full_key_identities.insert(signer.key_identity_blake3.as_str()) {
            return Err(vec!["duplicate-full-key-identity".to_string()]);
        }
        let statement = map_mantle_artifact_auth_statement(&MantleArtifactAuthStatementInput {
            profile_id: observation.profile_id,
            record: observation.record,
            producer_id: &signer.producer_id,
            key_id: &signer.key_id,
            key_identity_blake3: &signer.key_identity_blake3,
            oci_manifest_sha256: observation.oci_manifest_sha256,
            metadata_manifest_sha256: observation.metadata_manifest_sha256,
        })?;
        let key_identity = statement.key_identity.clone();
        trusted_keys.push(TrustedKeyObservation {
            producer_id: signer.producer_id.clone(),
            key_id: signer.key_id.clone(),
            key_identity: key_identity.clone(),
            allowed_purposes: vec![ARTIFACT_AUTH_PURPOSE.to_string()],
            generation: signer.generation,
            currentness: signer.currentness,
            currentness_ref: ArtifactRef {
                profile: KEY_CURRENTNESS_PROFILE.to_string(),
                algorithm: ALGORITHM_BLAKE3.to_string(),
                digest_hex: signer.currentness_blake3.clone(),
            },
        });
        evidence.push(SignatureEvidence {
            statement,
            generation: signer.generation,
            cryptographic: signer.standalone_cryptographic.clone(),
        });
    }
    debug_assert_eq!(trusted_keys.len(), observation.signers.len());
    debug_assert_eq!(evidence.len(), observation.signers.len());
    Ok((trusted_keys, evidence))
}

fn typed_blake3_ref(input: TypedBlake3RefInput<'_>) -> Result<ArtifactRef, Vec<String>> {
    let Some(digest_hex) = input.value.strip_prefix(input.prefix) else {
        return Err(vec![format!("typed-ref-prefix-mismatch:{}", input.profile)]);
    };
    if !valid_digest(digest_hex) {
        return Err(vec![format!("typed-ref-digest-malformed:{}", input.profile)]);
    }
    Ok(ArtifactRef {
        profile: input.profile.to_string(),
        algorithm: ALGORITHM_BLAKE3.to_string(),
        digest_hex: digest_hex.to_string(),
    })
}

fn sha256_ref(input: Sha256RefInput<'_>) -> Result<ArtifactRef, Vec<String>> {
    if !valid_digest(input.digest_hex) {
        return Err(vec![format!("sha256-digest-malformed:{}", input.profile)]);
    }
    Ok(ArtifactRef {
        profile: input.profile.to_string(),
        algorithm: ALGORITHM_SHA256.to_string(),
        digest_hex: input.digest_hex.to_string(),
    })
}

fn valid_digest(value: &str) -> bool {
    value.len() == DIGEST_HEX_CHARS && value.bytes().all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
}

fn compare(
    observation: &MantleArtifactAuthObservation<'_>,
    standalone: Option<&AuthenticationDecision>,
    mut blockers: Vec<String>,
) -> MantleArtifactAuthCompatibility {
    let is_standalone_passed = standalone.is_some_and(|decision| decision.passed);
    let is_decision_drift = standalone.is_some() && observation.legacy.admitted != is_standalone_passed;
    if standalone.is_none() {
        blockers.push("standalone-evaluation-unavailable".to_string());
    }
    if is_decision_drift {
        blockers.push("decision-drift".to_string());
    }
    let is_identity_drift_explained = observation
        .signers
        .iter()
        .all(|signer| signer.standalone_cryptographic.key_identity.digest_hex == signer.key_identity_blake3);
    if !is_identity_drift_explained {
        blockers.push("identity-drift".to_string());
    }
    let is_non_claim_drift = standalone.is_none_or(|decision| decision.non_claims != standalone_non_claims());
    if is_non_claim_drift {
        blockers.push("non-claim-drift".to_string());
    }
    let legacy_causes = legacy_causes(observation.legacy);
    let standalone_causes = standalone.map_or_else(BTreeSet::new, standalone_causes);
    if !observation.legacy.admitted && !is_standalone_passed && legacy_causes.is_disjoint(&standalone_causes) {
        blockers.push("unrelated-rejection-causes".to_string());
    }
    blockers.sort();
    blockers.dedup();
    let compatibility = MantleArtifactAuthCompatibility {
        case_explained: blockers.is_empty(),
        preimage_class: PREIMAGE_CLASS.to_string(),
        identity_drift_explained: is_identity_drift_explained,
        decision_drift: is_decision_drift,
        issue_class: if observation.legacy.admitted && is_standalone_passed {
            "no-issues".to_string()
        } else {
            "consumer-specific-taxonomy".to_string()
        },
        legacy_issue_causes: legacy_causes.into_iter().collect(),
        standalone_issue_causes: standalone_causes.into_iter().collect(),
        non_claim_drift: is_non_claim_drift,
        blockers,
        legacy_authoritative: true,
        standalone_authority_admitted: false,
        rollback_available: true,
    };
    debug_assert!(compatibility.legacy_authoritative);
    debug_assert!(!compatibility.standalone_authority_admitted);
    compatibility
}

fn legacy_causes(decision: &CandidateDecision) -> BTreeSet<String> {
    decision.diagnostics.iter().map(|diagnostic| legacy_issue_class(diagnostic)).collect()
}

fn legacy_issue_class(diagnostic: &str) -> String {
    if diagnostic.contains("signature") || diagnostic.contains("signer") {
        return "signature".to_string();
    }
    if diagnostic.contains("producer") {
        return "producer".to_string();
    }
    if diagnostic.contains("policy") {
        return "policy".to_string();
    }
    if diagnostic.contains("pathinfo") {
        return "path-info".to_string();
    }
    if diagnostic.contains("object") {
        return "object".to_string();
    }
    "consumer-admission".to_string()
}

fn standalone_causes(decision: &AuthenticationDecision) -> BTreeSet<String> {
    decision.issues.iter().map(|issue| standalone_issue_class(&issue.code)).collect()
}

fn standalone_issue_class(issue_code: &str) -> String {
    if issue_code.contains("crypto") || issue_code.contains("signature") {
        return "signature".to_string();
    }
    if issue_code.contains("current") || issue_code.contains("revoked") {
        return "currentness".to_string();
    }
    if issue_code.contains("generation") {
        return "generation".to_string();
    }
    if issue_code.contains("threshold") {
        return "threshold".to_string();
    }
    "standalone-policy".to_string()
}

#[cfg(test)]
mod tests {
    use artifact_auth_core::ALGORITHM_ED25519;
    use artifact_auth_core::CryptographicObservation;

    use super::*;
    use crate::ACTION_RECEIPT_REF_PREFIX;
    use crate::ActionResultOutput;
    use crate::ActionResultRecordInput;
    use crate::NETWORK_POLICY_REF_PREFIX;
    use crate::PATH_INFO_REF_PREFIX;
    use crate::PRODUCER_POLICY_REF_PREFIX;
    use crate::REFERENCE_SCAN_REF_PREFIX;
    use crate::SANDBOX_POLICY_REF_PREFIX;
    use crate::SIGNATURE_REF_PREFIX;
    use crate::canonical_action_result;
    use crate::required_non_claims;

    const PROFILE_ID: &str = "mantle-action-result-artifact-auth-v1";
    const SIGNER_LABEL: &str = "builder-key-1";
    const GENERATION_ONE: u64 = 1;
    const THRESHOLD_ONE: u16 = 1;

    fn digest(character: char) -> String {
        character.to_string().repeat(DIGEST_HEX_CHARS)
    }

    fn typed(prefix: &str, character: char) -> String {
        format!("{prefix}{}", digest(character))
    }

    fn record() -> ActionResultRecord {
        canonical_action_result(ActionResultRecordInput {
            action_ref: typed(crate::ACTION_REF_PREFIX, '1'),
            outputs: vec![ActionResultOutput {
                name: "out".to_string(),
                object_ref: typed(OBJECT_REF_PREFIX, '2'),
                store_path: "/mantle/store/result".to_string(),
                path_info_ref: typed(PATH_INFO_REF_PREFIX, '3'),
            }],
            action_receipt_ref: typed(ACTION_RECEIPT_REF_PREFIX, '4'),
            reference_scan_refs: vec![typed(REFERENCE_SCAN_REF_PREFIX, '5')],
            sandbox_policy_ref: typed(SANDBOX_POLICY_REF_PREFIX, '6'),
            network_policy_ref: typed(NETWORK_POLICY_REF_PREFIX, '7'),
            producer_identity: "builder".to_string(),
            producer_policy_ref: typed(PRODUCER_POLICY_REF_PREFIX, '8'),
            signature_refs: vec![typed(SIGNATURE_REF_PREFIX, '9')],
            publication_policy_ref: typed(PUBLICATION_POLICY_REF_PREFIX, 'a'),
            non_claims: required_non_claims(),
        })
        .expect("record")
    }

    fn crypto(key_identity: &str, verified: bool) -> CryptographicObservation {
        CryptographicObservation {
            algorithm: ALGORITHM_ED25519.to_string(),
            key_identity: ArtifactRef {
                profile: ED25519_PUBLIC_KEY_PROFILE_V1.to_string(),
                algorithm: ALGORITHM_BLAKE3.to_string(),
                digest_hex: key_identity.to_string(),
            },
            verified,
            failure_code: (!verified).then(|| "signature-invalid".to_string()),
        }
    }

    fn signer(verified: bool) -> MantleSignerObservation {
        let key_identity = digest('b');
        MantleSignerObservation {
            producer_id: "builder".to_string(),
            key_id: SIGNER_LABEL.to_string(),
            key_identity_blake3: key_identity.clone(),
            generation: GENERATION_ONE,
            currentness: KeyCurrentness::Current,
            currentness_blake3: digest('c'),
            standalone_cryptographic: crypto(&key_identity, verified),
        }
    }

    fn legacy(record: &ActionResultRecord, admitted: bool, diagnostics: Vec<String>) -> CandidateDecision {
        CandidateDecision {
            result_ref: record.result_ref.clone(),
            source_id: "local-source".to_string(),
            source_class: "configured-source".to_string(),
            admitted,
            diagnostics,
            trust_basis: vec![SIGNER_LABEL.to_string()],
            output_set_digest_blake3: admitted.then(|| digest('e')),
        }
    }

    // r[verify mantle.artifact_auth_shell.exact_verification]
    #[test]
    fn standalone_statement_mapping_binds_action_result_key_and_policy_identities() {
        let record = record();
        let signer = signer(true);
        let statement = map_mantle_artifact_auth_statement(&MantleArtifactAuthStatementInput {
            profile_id: PROFILE_ID,
            record: &record,
            producer_id: &signer.producer_id,
            key_id: &signer.key_id,
            key_identity_blake3: &signer.key_identity_blake3,
            oci_manifest_sha256: Some(&digest('f')),
            metadata_manifest_sha256: Some(&digest('0')),
        })
        .expect("valid statement mapping");

        assert!(artifact_auth_core::canonical_statement_bytes(&statement).is_ok());
        assert_eq!(statement.producer_id, signer.producer_id);
        assert_eq!(statement.key_id, signer.key_id);
        assert_eq!(statement.key_identity.digest_hex, signer.key_identity_blake3);
        assert_eq!(statement.scope.subject.digest_hex, record.result_ref[ACTION_RESULT_REF_PREFIX.len()..]);
        assert_eq!(
            statement.scope.verifier_context.digest_hex,
            record.publication_policy_ref[PUBLICATION_POLICY_REF_PREFIX.len()..]
        );
        assert_eq!(statement.scope.parents.len(), record.outputs.len().saturating_add(OCI_PARENT_COUNT));
    }

    // r[verify mantle.artifact_auth_shell.exact_verification]
    // r[verify mantle.artifact_auth_shell.adversarial]
    #[test]
    fn standalone_statement_mapping_rejects_malformed_key_and_incomplete_oci_pair() {
        let record = record();
        let malformed_key = map_mantle_artifact_auth_statement(&MantleArtifactAuthStatementInput {
            profile_id: PROFILE_ID,
            record: &record,
            producer_id: "builder",
            key_id: SIGNER_LABEL,
            key_identity_blake3: "short",
            oci_manifest_sha256: None,
            metadata_manifest_sha256: None,
        });
        assert_eq!(malformed_key, Err(vec!["key-identity-malformed".to_string()]));

        let incomplete_oci = map_mantle_artifact_auth_statement(&MantleArtifactAuthStatementInput {
            profile_id: PROFILE_ID,
            record: &record,
            producer_id: "builder",
            key_id: SIGNER_LABEL,
            key_identity_blake3: &digest('b'),
            oci_manifest_sha256: Some(&digest('f')),
            metadata_manifest_sha256: None,
        });
        assert_eq!(incomplete_oci, Err(vec!["oci-manifest-pair-incomplete".to_string()]));
    }

    // r[verify mantle.artifact_auth_adoption.authority]
    // r[verify mantle.artifact_auth_adoption.cutover]
    #[test]
    fn passing_dual_run_preserves_oci_identity_and_product_authority() {
        let record = record();
        let legacy = legacy(&record, true, Vec::new());
        let signers = vec![signer(true)];
        let labels = vec![SIGNER_LABEL.to_string()];
        let report = evaluate_mantle_artifact_auth(&MantleArtifactAuthObservation {
            profile_id: PROFILE_ID,
            record: &record,
            legacy: &legacy,
            signers: &signers,
            required_signer_labels: &labels,
            threshold: THRESHOLD_ONE,
            oci_manifest_sha256: Some(&digest('f')),
            metadata_manifest_sha256: Some(&digest('0')),
        });

        assert!(report.standalone.as_ref().is_some_and(|decision| decision.passed));
        assert!(report.compatibility.case_explained);
        assert!(!report.compatibility.decision_drift);
        assert!(!report.compatibility.non_claim_drift);
        assert!(report.compatibility.legacy_authoritative);
        assert!(!report.compatibility.standalone_authority_admitted);
        assert!(report.compatibility.rollback_available);
        assert!(report.repository_authority_retained);
        assert!(report.build_authority_retained);
        assert!(report.release_authority_retained);
        assert_eq!(report.nix_key_identities, labels);
    }

    // r[verify mantle.artifact_auth_adoption.cutover]
    #[test]
    fn signature_tamper_is_mapped_but_unrelated_failure_and_false_parity_block() {
        let record = record();
        let signers = vec![signer(false)];
        let labels = vec![SIGNER_LABEL.to_string()];
        let signature_failure = legacy(&record, false, vec!["action-result-record-signature-untrusted".to_string()]);
        let mapped = evaluate_mantle_artifact_auth(&MantleArtifactAuthObservation {
            profile_id: PROFILE_ID,
            record: &record,
            legacy: &signature_failure,
            signers: &signers,
            required_signer_labels: &labels,
            threshold: THRESHOLD_ONE,
            oci_manifest_sha256: None,
            metadata_manifest_sha256: None,
        });
        assert!(mapped.compatibility.case_explained);

        let producer_failure = legacy(&record, false, vec!["action-result-producer-untrusted".to_string()]);
        let unrelated = evaluate_mantle_artifact_auth(&MantleArtifactAuthObservation {
            profile_id: PROFILE_ID,
            record: &record,
            legacy: &producer_failure,
            signers: &signers,
            required_signer_labels: &labels,
            threshold: THRESHOLD_ONE,
            oci_manifest_sha256: None,
            metadata_manifest_sha256: None,
        });
        assert!(!unrelated.compatibility.case_explained);
        assert!(unrelated.compatibility.blockers.contains(&"unrelated-rejection-causes".to_string()));

        let passing_signers = vec![signer(true)];
        let false_parity = evaluate_mantle_artifact_auth(&MantleArtifactAuthObservation {
            profile_id: PROFILE_ID,
            record: &record,
            legacy: &signature_failure,
            signers: &passing_signers,
            required_signer_labels: &labels,
            threshold: THRESHOLD_ONE,
            oci_manifest_sha256: None,
            metadata_manifest_sha256: None,
        });
        assert!(false_parity.compatibility.decision_drift);
    }

    // r[verify mantle.artifact_auth_adoption.authority]
    // r[verify mantle.artifact_auth_adoption.cutover]
    #[test]
    fn duplicate_full_keys_missing_labels_revocation_and_incomplete_oci_pair_fail_closed() {
        let record = record();
        let legacy = legacy(&record, false, vec!["action-result-record-signature-untrusted".to_string()]);
        let mut first = signer(true);
        first.currentness = KeyCurrentness::Revoked;
        let mut second = first.clone();
        second.key_id = "builder-key-2".to_string();
        let duplicate_signers = vec![first, second];
        let labels = vec![SIGNER_LABEL.to_string()];
        let duplicate = evaluate_mantle_artifact_auth(&MantleArtifactAuthObservation {
            profile_id: PROFILE_ID,
            record: &record,
            legacy: &legacy,
            signers: &duplicate_signers,
            required_signer_labels: &labels,
            threshold: THRESHOLD_ONE,
            oci_manifest_sha256: None,
            metadata_manifest_sha256: None,
        });
        assert!(duplicate.standalone.is_none());
        assert!(!duplicate.compatibility.case_explained);

        let mut first_label = signer(true);
        let mut second_label = signer(true);
        second_label.key_identity_blake3 = digest('d');
        second_label.standalone_cryptographic = crypto(&second_label.key_identity_blake3, true);
        first_label.key_id = SIGNER_LABEL.to_string();
        second_label.key_id = SIGNER_LABEL.to_string();
        let duplicate_labels = vec![first_label, second_label];
        let duplicate_label_report = evaluate_mantle_artifact_auth(&MantleArtifactAuthObservation {
            profile_id: PROFILE_ID,
            record: &record,
            legacy: &legacy,
            signers: &duplicate_labels,
            required_signer_labels: &labels,
            threshold: THRESHOLD_ONE,
            oci_manifest_sha256: None,
            metadata_manifest_sha256: None,
        });
        assert!(duplicate_label_report.standalone.is_none());

        let mut revoked_signer = signer(true);
        revoked_signer.currentness = KeyCurrentness::Revoked;
        let revoked_signers = vec![revoked_signer];
        let revoked = evaluate_mantle_artifact_auth(&MantleArtifactAuthObservation {
            profile_id: PROFILE_ID,
            record: &record,
            legacy: &legacy,
            signers: &revoked_signers,
            required_signer_labels: &labels,
            threshold: THRESHOLD_ONE,
            oci_manifest_sha256: None,
            metadata_manifest_sha256: None,
        });
        assert!(revoked.standalone.as_ref().is_some_and(|decision| !decision.passed));
        assert!(!revoked.compatibility.case_explained);

        let signers = vec![signer(true)];
        let missing_labels = vec!["required-other-key".to_string()];
        let missing = evaluate_mantle_artifact_auth(&MantleArtifactAuthObservation {
            profile_id: PROFILE_ID,
            record: &record,
            legacy: &legacy,
            signers: &signers,
            required_signer_labels: &missing_labels,
            threshold: THRESHOLD_ONE,
            oci_manifest_sha256: None,
            metadata_manifest_sha256: None,
        });
        assert!(missing.standalone.is_none());

        let incomplete_oci = evaluate_mantle_artifact_auth(&MantleArtifactAuthObservation {
            profile_id: PROFILE_ID,
            record: &record,
            legacy: &legacy,
            signers: &signers,
            required_signer_labels: &labels,
            threshold: THRESHOLD_ONE,
            oci_manifest_sha256: Some(&digest('f')),
            metadata_manifest_sha256: None,
        });
        assert!(incomplete_oci.standalone.is_none());

        let malformed_oci = evaluate_mantle_artifact_auth(&MantleArtifactAuthObservation {
            profile_id: PROFILE_ID,
            record: &record,
            legacy: &legacy,
            signers: &signers,
            required_signer_labels: &labels,
            threshold: THRESHOLD_ONE,
            oci_manifest_sha256: Some("short-digest"),
            metadata_manifest_sha256: Some(&digest('0')),
        });
        assert!(malformed_oci.standalone.is_none());
        assert!(malformed_oci.authority_boundary.contains("repository authorization"));
    }
}
