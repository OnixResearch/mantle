use alloc::format;

use artifact_auth_core::KeyCurrentness;
use artifact_auth_core::TrustedKeyObservation;
use ed25519_dalek::Signer;
use ed25519_dalek::SigningKey;

use super::*;

// Deterministic TEST ONLY keys; negative cases may reseal synthetic receipts to
// prove their identity alone cannot confer authority. The positive compatibility
// case uses the pinned Valence verifier; real Cairn CLI interop is a separate smoke.
struct Fixture {
    manifest: ReleaseEvidenceManifest,
    attachment: SourceReviewAttachment,
    policy: SourceReviewVerificationPolicy,
    receipt: Vec<u8>,
}

fn digest(bytes: &[u8]) -> Blake3Digest {
    Blake3Digest::parse(blake3::hash(bytes).to_hex().to_string()).unwrap()
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn signed_review(policy: &SourceReviewVerificationPolicy, seed: u8, disposition: &str) -> SignedSourceReview {
    let key = SigningKey::from_bytes(&[seed; 32]);
    let public = key.verifying_key().to_bytes();
    let claim = SourceReviewClaim {
        schema: "cairn.source-review-claim.v1".to_string(),
        subject: policy.subject.clone(),
        claim_root: policy.claim_root.clone(),
        review_policy_blake3: policy.review_policy_blake3.clone(),
        disposition: disposition.to_string(),
        non_claims: policy.non_claims.clone(),
    };
    let statement = ArtifactStatement {
        schema: artifact_auth_core::STATEMENT_SCHEMA_V1.to_string(),
        scope: expected_review_scope(&claim, &policy.verifier_context).unwrap(),
        producer_id: "test-reviewer".to_string(),
        key_id: "same-display-label".to_string(),
        key_identity: artifact_auth_ed25519::public_key_identity(&public),
    };
    let signature = key.sign(&artifact_auth_core::canonical_statement_bytes(&statement).unwrap());
    SignedSourceReview {
        claim,
        statement,
        generation: 7,
        public_key_hex: hex(&public),
        signature_hex: hex(&signature.to_bytes()),
    }
}

impl Fixture {
    fn new() -> Self {
        let non_claims = SOURCE_REVIEW_NON_CLAIMS.iter().map(|value| (*value).to_string()).collect::<Vec<_>>();
        let context = reference(ProfileDigest {
            profile: "test-review-currentness.v1",
            digest: digest(b"current test key observations").as_str(),
        });
        let subject = SourceReviewSubject {
            schema: "cairn.source-review-subject.v1".to_string(),
            source_archive_blake3: digest(b"test source archive"),
            source_revision: None,
            source_tree_blake3: None,
            snapshot_profile: "mantle-release-source".to_string(),
            snapshot_profile_version: 1,
            projection: "source".to_string(),
        };
        let trusted_keys = [1_u8, 2]
            .into_iter()
            .map(|seed| {
                let public = SigningKey::from_bytes(&[seed; 32]).verifying_key().to_bytes();
                TrustedKeyObservation {
                    producer_id: "test-reviewer".to_string(),
                    key_id: "same-display-label".to_string(),
                    key_identity: artifact_auth_ed25519::public_key_identity(&public),
                    allowed_purposes: vec!["source-review-approval".to_string()],
                    generation: 7,
                    currentness: KeyCurrentness::Current,
                    currentness_ref: context.clone(),
                }
            })
            .collect();
        let policy = SourceReviewVerificationPolicy {
            schema: SOURCE_REVIEW_POLICY_SCHEMA.to_string(),
            subject,
            claim_root: SourceReviewClaimRoot {
                claim_identity: digest(b"test claim root"),
                subject_root: digest(b"test existing DAG subject root"),
                revision: 3,
            },
            review_policy_blake3: digest(b"test review policy"),
            authentication_policy: AuthenticationPolicy {
                schema: artifact_auth_core::POLICY_SCHEMA_V1.to_string(),
                profile_id: "cairn.source-review.v1".to_string(),
                threshold: 2,
                trusted_keys,
            },
            verifier_context: context,
            author_key_blake3: vec![],
            exclude_authors: true,
            required_approvals: 2,
            non_claims: non_claims.clone(),
        };
        let attachment = SourceReviewAttachment {
            schema: SOURCE_REVIEW_ATTACHMENT_SCHEMA.to_string(),
            reviews: vec![
                signed_review(&policy, 2, "approved"),
                signed_review(&policy, 1, "approved"),
            ],
            non_claims,
        };
        let artifact = serde_json::json!({"kind":"file","relative_path":"source/test.tar","size_bytes":19,"digest_blake3":policy.subject.source_archive_blake3});
        let manifest = serde_json::from_value(serde_json::json!({
            "schema": crate::RELEASE_EVIDENCE_SCHEMA, "release_id":"unit-test-only", "claim_scope":crate::CLAIM_SCOPE_PACKAGED_INTEGRITY,
            "workflow":{"command":crate::DEFAULT_PROOF_WORKFLOW_COMMAND,"version":crate::DEFAULT_PROOF_WORKFLOW_VERSION},
            "source_archive":artifact,"binaries":[],"proof_bundle":artifact,"prerequisite_inventory":artifact,
            "proof_linkage":{"release_id":"unit-test-only","source_archive_digest_blake3":policy.subject.source_archive_blake3,
                "proof_bundle_schema":crate::FULL_SELF_HOSTING_PROOF_SCHEMA,"proof_mode":"fixed-point","selected_provider_kind":"test-only",
                "staged_source":"source","stage2_binary_digest_blake3":digest(b"binary"),"prerequisite_inventory_digest_blake3":digest(b"inventory"),
                "proof_manifest_digest_blake3":digest(b"proof")}
        })).unwrap();
        let mut fixture = Self {
            manifest,
            attachment,
            policy,
            receipt: vec![],
        };
        fixture.seal_synthetic_receipt();
        fixture
    }

    fn seal_synthetic_receipt(&mut self) {
        let attachment_bytes = serde_json::to_vec(&self.attachment).unwrap();
        let policy_bytes = serde_json::to_vec(&self.policy).unwrap();
        let mut attachment = self.attachment.clone();
        attachment.reviews.sort_by(|left, right| {
            left.statement.key_identity.digest_hex.cmp(&right.statement.key_identity.digest_hex)
        });
        let mut policy = self.policy.clone();
        policy.author_key_blake3.sort();
        policy
            .authentication_policy
            .trusted_keys
            .sort_by(|left, right| left.key_identity.cmp(&right.key_identity));
        let keys = attachment
            .reviews
            .iter()
            .filter(|row| row.claim.disposition == "approved")
            .map(|row| row.statement.key_identity.digest_hex.clone())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();
        let statements = attachment
            .reviews
            .iter()
            .map(|row| artifact_auth_core::statement_identity(&row.statement).unwrap())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();
        let claims = attachment
            .reviews
            .iter()
            .map(|row| source_review_identity(b"cairn.source-review-claim.v1\0", &row.claim).unwrap())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();
        let receipt = ValenceSourceReviewAcceptance {
            schema: SOURCE_REVIEW_VALENCE_SCHEMA.to_string(),
            outcome: "accepted_manual_review".to_string(),
            verification_role: "manual_review".to_string(),
            subject: Some(policy.subject.clone()),
            subject_identity: Some(
                source_review_identity(b"cairn.source-review-subject.v1\0", &policy.subject).unwrap(),
            ),
            claim_root: Some(policy.claim_root.clone()),
            claim_root_identity: Some(source_review_identity(b"cairn.claim-root.v1\0", &policy.claim_root).unwrap()),
            review_policy_blake3: Some(policy.review_policy_blake3.as_str().to_string()),
            verification_policy_identity: Some(
                source_review_identity(b"cairn.source-review-verification-policy.v1\0", &policy).unwrap(),
            ),
            authentication_policy_identity: Some(
                artifact_auth_core::policy_identity(&policy.authentication_policy).unwrap(),
            ),
            approved_key_blake3: keys,
            signed_statement_blake3: statements,
            claim_identity: claims,
            attachment_identity: Some(
                source_review_identity(b"cairn.source-review-attachment.v1\0", &attachment).unwrap(),
            ),
            attachment_artifact_blake3: blake3::hash(&attachment_bytes).to_hex().to_string(),
            policy_artifact_blake3: blake3::hash(&policy_bytes).to_hex().to_string(),
            receipt_blake3: String::new(),
            non_claims: policy.non_claims.clone(),
            issues: vec![],
        };
        let mut receipt = serde_json::to_value(receipt).unwrap();
        receipt.as_object_mut().unwrap().remove("receipt_blake3");
        let identity = source_review_identity(b"valence.source-review-acceptance.v1\0", &receipt).unwrap();
        receipt["receipt_blake3"] = Value::String(identity);
        self.receipt = serde_json::to_vec(&receipt).unwrap();
    }

    fn verify(&self) -> SourceReviewVerification {
        let policy = serde_json::to_vec(&self.policy).unwrap();
        verify_release_source_review(SourceReviewVerificationRequest {
            manifest: self.manifest.clone(),
            attachment_bytes: Some(serde_json::to_vec(&self.attachment).unwrap()),
            policy_bytes: Some(policy.clone()),
            valence_bytes: Some(self.receipt.clone()),
            trusted_policy_bytes: Some(policy),
            require_review: true,
        })
        .verification
    }
}

// r[verify verification_evidence.release_source_review_evidence]
#[test]
fn distinct_full_keys_with_identical_labels_satisfy_only_the_selected_review_gate() {
    let mut fixture = Fixture::new();
    let acceptance = valence_core::source_review::accept_source_review_bytes(
        &serde_json::to_vec(&fixture.attachment).unwrap(),
        &serde_json::to_vec(&fixture.policy).unwrap(),
    );
    assert_eq!(
        acceptance.outcome,
        valence_core::source_review::SourceReviewOutcome::AcceptedManualReview,
        "{:?}",
        acceptance.issues
    );
    fixture.receipt = serde_json::to_vec(&acceptance).unwrap();
    let result = fixture.verify();
    assert_eq!(result.status, SourceReviewStatus::Satisfied, "{:?}", result.diagnostics);
    assert!(result.valid);
    assert_eq!(result.approved_key_blake3.len(), 2);
    assert_eq!(result.approved_key_blake3, acceptance.approved_key_blake3);
    let optional = verify_release_source_review(SourceReviewVerificationRequest {
        manifest: fixture.manifest.clone(),
        attachment_bytes: None,
        policy_bytes: None,
        valence_bytes: None,
        trusted_policy_bytes: None,
        require_review: false,
    });
    assert_eq!(optional.manifest, fixture.manifest, "the evaluated manifest is returned unchanged");
    assert_eq!(optional.verification.status, SourceReviewStatus::NotRequired);
    assert!(optional.verification.valid);
    let required = verify_release_source_review(SourceReviewVerificationRequest {
        manifest: fixture.manifest.clone(),
        attachment_bytes: None,
        policy_bytes: None,
        valence_bytes: None,
        trusted_policy_bytes: None,
        require_review: true,
    })
    .verification;
    assert_eq!(required.status, SourceReviewStatus::Missing);
    assert!(!required.valid);
}

#[test]
fn a_producer_acceptance_string_cannot_replace_actual_signature_or_trust() {
    let mut fixture = Fixture::new();
    fixture.attachment.reviews[0].signature_hex = "00".repeat(64);
    fixture.seal_synthetic_receipt(); // even a self-consistent forged identity receipt cannot authenticate this
    let result = fixture.verify();
    assert!(!result.valid);
    assert!(result.diagnostics[0].starts_with("source-review-authentication-rejected:"));
}

#[test]
fn needs_revision_is_authenticated_but_cannot_fulfill_approval_threshold() {
    let mut fixture = Fixture::new();
    fixture.attachment.reviews[0] = signed_review(&fixture.policy, 2, "needs_revision");
    let result = fixture.verify();
    assert_eq!(result.status, SourceReviewStatus::Insufficient);
    assert!(!result.valid);
}

#[test]
fn duplicate_full_key_aliases_never_become_two_reviewers() {
    let mut fixture = Fixture::new();
    fixture.attachment.reviews[0] = fixture.attachment.reviews[1].clone();
    let result = fixture.verify();
    assert!(!result.valid);
    assert_eq!(result.diagnostics, ["source-review-duplicate-full-key"]);
    assert!(matches!(
        verify_review_rows(&fixture.attachment.reviews, &fixture.policy),
        Err(SourceReviewError::DuplicateFullKey)
    ));
}

#[test]
fn release_archive_and_current_claim_root_are_independent_mandatory_bindings() {
    let mut fixture = Fixture::new();
    fixture.manifest.source_archive.digest_blake3 = digest(b"different release source").as_str().to_string();
    assert_eq!(fixture.verify().status, SourceReviewStatus::Stale);
    let mut fixture = Fixture::new();
    fixture.policy.claim_root.revision += 1;
    let result = fixture.verify();
    assert_eq!(result.status, SourceReviewStatus::Stale);
    assert_eq!(result.diagnostics, ["source-review-stale-subject-root-or-policy"]);
}

#[test]
fn authors_revocation_generation_and_role_are_enforced_without_receipt_authority() {
    let mut fixture = Fixture::new();
    let author = fixture.attachment.reviews[0].statement.key_identity.digest_hex.clone();
    fixture.policy.author_key_blake3.push(Blake3Digest::parse(author).unwrap());
    assert_eq!(fixture.verify().diagnostics, ["source-review-author-excluded"]);
    for currentness in [
        KeyCurrentness::Revoked,
        KeyCurrentness::Superseded,
        KeyCurrentness::Unknown,
    ] {
        let mut fixture = Fixture::new();
        fixture.policy.authentication_policy.trusted_keys[0].currentness = currentness;
        assert!(!fixture.verify().valid, "{currentness:?}");
    }
    let mut fixture = Fixture::new();
    fixture.attachment.reviews[0].generation += 1;
    assert!(!fixture.verify().valid);
    let mut fixture = Fixture::new();
    fixture.policy.authentication_policy.trusted_keys[0].allowed_purposes = vec!["release-signature".to_string()];
    assert_eq!(fixture.verify().diagnostics, ["source-review-reviewer-key-role-invalid"]);
    let mut fixture = Fixture::new();
    fixture.policy.authentication_policy.trusted_keys[0].currentness_ref = reference(ProfileDigest {
        profile: "test-review-currentness.v1",
        digest: digest(b"other observation").as_str(),
    });
    assert_eq!(fixture.verify().diagnostics, ["source-review-reviewer-key-role-invalid"]);
}

#[test]
fn explicit_policy_and_exact_valence_artifact_linkage_cannot_be_substituted() {
    let fixture = Fixture::new();
    let attachment = serde_json::to_vec(&fixture.attachment).unwrap();
    let policy = serde_json::to_vec(&fixture.policy).unwrap();
    let result = verify_release_source_review(SourceReviewVerificationRequest {
        manifest: fixture.manifest.clone(),
        attachment_bytes: Some(attachment),
        policy_bytes: Some(policy),
        valence_bytes: Some(fixture.receipt.clone()),
        trusted_policy_bytes: Some(b"other policy".to_vec()),
        require_review: true,
    })
    .verification;
    assert_eq!(result.status, SourceReviewStatus::Stale);
    let mut fixture = Fixture::new();
    let mut receipt: Value = serde_json::from_slice(&fixture.receipt).unwrap();
    receipt["verification_role"] = Value::String("property".to_string());
    receipt.as_object_mut().unwrap().remove("receipt_blake3");
    let identity = source_review_identity(b"valence.source-review-acceptance.v1\0", &receipt).unwrap();
    receipt["receipt_blake3"] = Value::String(identity);
    fixture.receipt = serde_json::to_vec(&receipt).unwrap();
    assert_eq!(fixture.verify().diagnostics, ["source-review-valence-linkage-mismatch:verification_role"]);
}

#[test]
fn closed_receipts_reject_duplicate_fields_even_with_a_matching_last_value() {
    let mut fixture = Fixture::new();
    let text = core::str::from_utf8(&fixture.receipt).unwrap();
    fixture.receipt = format!("{{\"outcome\":\"invalid\",{}", &text[1..]).into_bytes();
    assert_eq!(fixture.verify().diagnostics, ["source-review-valence-malformed"]);
}

#[test]
fn canonical_observation_is_accepted_but_an_arbitrary_identity_is_not() {
    let mut fixture = Fixture::new();
    let acquisition = crate::SourceAcquisition::external_archive(
        "https://example.invalid/test-source.tar".to_string(),
        fixture.manifest.source_archive.digest_blake3.clone(),
    )
    .with_admitted_source_observation()
    .unwrap();
    fixture.manifest.source_acquisition = Some(acquisition);
    assert!(fixture.verify().valid);
    fixture
        .manifest
        .source_acquisition
        .as_mut()
        .unwrap()
        .source_observation
        .as_mut()
        .unwrap()
        .observation_blake3 = digest(b"unrelated observation").as_str().to_string();
    let result = fixture.verify();
    assert_eq!(result.status, SourceReviewStatus::Stale);
    assert_eq!(result.diagnostics, ["source-review-release-acquisition-or-observation-invalid"]);
}

#[test]
fn unsupported_snapshot_profiles_and_weakened_nonclaims_fail_closed() {
    let mut fixture = Fixture::new();
    fixture.policy.subject.snapshot_profile_version = 2;
    assert_eq!(fixture.verify().status, SourceReviewStatus::Unsupported);
    let mut fixture = Fixture::new();
    fixture.attachment.non_claims.pop();
    assert_eq!(fixture.verify().diagnostics, ["source-review-non-claims-invalid"]);
}
