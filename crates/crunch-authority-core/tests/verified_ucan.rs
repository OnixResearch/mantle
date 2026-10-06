#[cfg(test)]
mod tests {
    use std::cell::Cell;
    use std::fs;

    use basalt::authority_core::AUTHORITY_INPUT_SCHEMA;
    use basalt::authority_core::AuthorityCaveatDisposition;
    use basalt::authority_core::AuthorityClaim;
    use basalt::authority_core::AuthorityDenyDiagnostic;
    use basalt::authority_core::AuthorityInput;
    use basalt::authority_core::AuthorityReplayDisposition;
    use basalt::authority_core::AuthorityRequest;
    use basalt::authority_core::BasaltPolicyRef;
    use basalt::authority_core::REQUIRED_AUTHORITY_NON_CLAIMS;
    use basalt::authority_core::UcanVerificationReceiptRef;
    use basalt::authority_core::VerificationRefDisposition;
    use basalt::authority_core::VerifiedGrantKind;
    use basalt::authority_core::VerifiedGrantRef;
    use basalt::authority_core::evaluate_authority;
    use crunch_authority_core::FilterChain;
    use crunch_authority_core::MAX_CAVEAT_BYTES;
    use crunch_authority_core::MAX_CHAIN;
    use crunch_authority_core::WireCaveat;
    use crunch_authority_core::WireCaveatRecord;
    use crunch_authority_core::WireRewrite;
    use crunch_authority_core::sites::ProjectGoals;
    use crunch_authority_core::sites::RemoteJobScope;
    use crunch_authority_core::sites::StoreView;
    use serde::Deserialize;
    use ucan::AudienceDid;
    use ucan::AuthorizationDecision;
    use ucan::CapabilityDocument;
    use ucan::CapabilitySet;
    use ucan::CaveatDecision;
    use ucan::CaveatDocument;
    use ucan::CaveatIdentifier;
    use ucan::CaveatPolicy;
    use ucan::CaveatPolicySet;
    use ucan::ED25519_SECRET_KEY_BYTES;
    use ucan::Ed25519InMemorySigner;
    use ucan::Ed25519SigningKey;
    use ucan::Ed25519VerificationKey;
    use ucan::HolderInvocationLimits;
    use ucan::HolderInvocationNonce;
    use ucan::HolderInvocationPolicyContext;
    use ucan::HolderInvocationReplayAdmission;
    use ucan::HolderInvocationReplayCandidate;
    use ucan::HolderInvocationReplayError;
    use ucan::HolderInvocationRequest;
    use ucan::HolderInvocationSigningRequest;
    use ucan::HolderTokenVerificationContext;
    use ucan::IssueRequest;
    use ucan::KeyResolutionContext;
    use ucan::NoRevocations;
    use ucan::OperationBinding;
    use ucan::OperationProfileAdmission;
    use ucan::OperationProfileAdmissionError;
    use ucan::OperationProfileId;
    use ucan::ProofCollection;
    use ucan::ProofReferences;
    use ucan::RequestBindingDigest;
    use ucan::TokenSigner;
    use ucan::TokenTimeBounds;
    use ucan::VerificationContext;
    use ucan::VerificationLimits;
    use ucan::VerificationTime;
    use ucan::issue_token_with_signer;
    use ucan::sign_holder_invocation;
    use ucan::verify_holder_signed_invocation;

    const RESOURCE: &str = "mantle://remote-build/receiver-1";
    const STORE_RESOURCE: &str = "mantle://store-view/a-allowed";
    const PROJECT_RESOURCE: &str = "mantle://project-goal/alpha";
    const ABILITY: &str = "remote/build";
    const PARENT: &[u8] =
        br#"{"kind":"rewrite","pattern":"/job/:id/output/public","template":"/job/:id/output/public"}"#;
    const ALTERED_PARENT: &[u8] =
        br#"{"kind":"rewrite","pattern":"/job/:id/output/private","template":"/job/:id/output/private"}"#;
    const CHILD: &[u8] = br#"{"kind":"reject","pattern":"/job/compile-2/output/public"}"#;
    const UNKNOWN: &[u8] = br#"{"kind":"future-restriction"}"#;

    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct RewriteDto<'a> {
        pattern: &'a str,
        template: &'a str,
    }

    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct CaveatDto<'a> {
        kind: &'a str,
        pattern: Option<&'a str>,
        template: Option<&'a str>,
        #[serde(default)]
        alternatives: Vec<RewriteDto<'a>>,
    }

    struct RegisteredPolicy<'a> {
        chain: &'a FilterChain,
        scope: &'a RemoteJobScope<'a>,
        job: &'a str,
        now_unix_s: u64,
    }

    impl CaveatPolicy for RegisteredPolicy<'_> {
        fn evaluate(&self, caveat: &CaveatDocument, _request: &ucan::AuthorizationRequest) -> CaveatDecision {
            if caveat.domain() != "mantle" || caveat.caveat_type() != "filter-v1" || caveat.key().len() != 2 {
                return CaveatDecision::Rejected {
                    message: "denied".to_owned(),
                };
            }
            if self.scope.admits(self.chain, self.job, "public", self.now_unix_s) {
                CaveatDecision::Satisfied
            } else {
                CaveatDecision::Rejected {
                    message: "denied".to_owned(),
                }
            }
        }
    }

    impl CaveatPolicySet for RegisteredPolicy<'_> {
        fn policy_for(&self, caveat: &CaveatIdentifier) -> Option<&dyn CaveatPolicy> {
            (caveat.domain() == "mantle" && caveat.caveat_type() == "filter-v1").then_some(self)
        }
    }

    fn caveat(key: &str, payload: &[u8]) -> CaveatDocument {
        CaveatDocument::new("mantle".to_owned(), "filter-v1".to_owned(), key.to_owned(), payload.to_vec()).unwrap()
    }

    fn capability(caveats: Vec<CaveatDocument>) -> CapabilitySet {
        CapabilitySet::new(vec![
            CapabilityDocument::with_caveats(RESOURCE.to_owned(), ABILITY.to_owned(), caveats).unwrap(),
        ])
        .unwrap()
    }

    fn signed_grant(
        parent_caveats: Vec<CaveatDocument>,
        child_caveats: Vec<CaveatDocument>,
    ) -> Result<ucan::VerifiedToken, ucan::TokenError> {
        let issuer = Ed25519SigningKey::from_bytes([1; ED25519_SECRET_KEY_BYTES]);
        let holder = Ed25519SigningKey::from_bytes([2; ED25519_SECRET_KEY_BYTES]);
        let receiver = Ed25519SigningKey::from_bytes([3; ED25519_SECRET_KEY_BYTES]);
        let parent = ucan::issue_token(&IssueRequest::new(
            issuer.clone(),
            AudienceDid::from(holder.issuer().unwrap()),
            capability(parent_caveats),
            ProofCollection::empty().into(),
            TokenTimeBounds::new(10, 100).unwrap(),
        ))
        .unwrap();
        let parent_copy = ucan::CompactToken::try_from(parent.as_str()).unwrap();
        let child = ucan::issue_token(&IssueRequest::new(
            holder.clone(),
            AudienceDid::from(receiver.issuer().unwrap()),
            capability(child_caveats),
            ProofCollection::from_tokens(vec![parent]).into(),
            TokenTimeBounds::new(20, 90).unwrap(),
        ))
        .unwrap();
        let context = VerificationContext::new(
            VerificationTime::from_unix_seconds(40),
            KeyResolutionContext::new(vec![
                Ed25519VerificationKey::from_signing_key(&issuer).unwrap(),
                Ed25519VerificationKey::from_signing_key(&holder).unwrap(),
            ]),
            ProofCollection::from_tokens(vec![parent_copy]),
        );
        ucan::verify_compact_token(&child, &context)
    }

    fn decode_chain(caveats: &[CaveatDocument]) -> Option<FilterChain> {
        if caveats.len() > MAX_CHAIN {
            return None;
        }
        let mut parsed = Vec::with_capacity(caveats.len());
        for caveat in caveats {
            if caveat.domain() != "mantle"
                || caveat.caveat_type() != "filter-v1"
                || caveat.payload().len() > MAX_CAVEAT_BYTES
            {
                return None;
            }
            parsed.push(serde_json::from_slice::<CaveatDto<'_>>(caveat.payload()).ok()?);
        }
        let alternatives: Vec<Vec<WireRewrite<'_>>> = parsed
            .iter()
            .map(|dto| {
                dto.alternatives
                    .iter()
                    .map(|rewrite| WireRewrite {
                        pattern: rewrite.pattern,
                        template: rewrite.template,
                    })
                    .collect()
            })
            .collect();
        let records: Vec<WireCaveatRecord<'_>> = caveats
            .iter()
            .zip(parsed.iter())
            .zip(alternatives.iter())
            .map(|((caveat, dto), rewrites)| WireCaveatRecord {
                key: caveat.key(),
                serialized_payload: caveat.payload(),
                caveat: WireCaveat {
                    kind: dto.kind,
                    pattern: dto.pattern,
                    template: dto.template,
                    alternatives: rewrites,
                },
            })
            .collect();
        FilterChain::from_records(&records).ok()
    }

    fn receiver_admits(verified: &ucan::VerifiedToken, job: &str, now_unix_s: u64) -> bool {
        let grants = verified.effective_delegation().capabilities();
        let mut matching = grants
            .as_slice()
            .iter()
            .filter(|candidate| candidate.resource == RESOURCE && candidate.ability == ABILITY);
        let Some(grant) = matching.next() else { return false };
        if matching.next().is_some() {
            return false;
        }
        let Some(chain) = decode_chain(&grant.caveats) else {
            return false;
        };
        let scope = RemoteJobScope {
            job_ids: &["compile-1", "compile-2"],
            output_class: "public",
            deadline_unix_s: 50,
        };
        // UCAN has no caveat callback for a bare grant, so enforce the
        // receiver scope even when the decoded caveat chain is empty.
        if !scope.admits(&chain, job, "public", now_unix_s) {
            return false;
        }
        let policies = RegisteredPolicy {
            chain: &chain,
            scope: &scope,
            job,
            now_unix_s,
        };
        matches!(
            verified.authorize_with_policies(RESOURCE, ABILITY, &policies),
            AuthorizationDecision::Allowed { .. }
        )
    }

    #[test]
    fn signed_attenuation_admits_one_job_and_denies_effects_for_other_jobs() {
        let parent = caveat("00", PARENT);
        let child = caveat("01", CHILD);
        let verified = signed_grant(vec![parent.clone()], vec![parent.clone(), child.clone()]).unwrap();
        let dir = tempfile::tempdir().unwrap();
        for job in ["compile-1", "compile-2", "unauthorized"] {
            if receiver_admits(&verified, job, 49) {
                fs::write(dir.path().join(job), b"job ran").unwrap();
            }
        }
        assert_eq!(fs::read(dir.path().join("compile-1")).unwrap(), b"job ran");
        assert!(!dir.path().join("compile-2").exists());
        assert!(!dir.path().join("unauthorized").exists());
        assert!(!receiver_admits(&verified, "compile-1", 50));
        assert!(signed_grant(vec![parent.clone()], vec![child]).is_err());
        assert!(signed_grant(vec![parent.clone()], vec![caveat("00", ALTERED_PARENT), caveat("01", CHILD)]).is_err());
        let unknown = signed_grant(vec![parent.clone()], vec![parent, caveat("01", UNKNOWN)]).unwrap();
        assert!(!receiver_admits(&unknown, "compile-1", 49));
        let bare = signed_grant(vec![], vec![]).unwrap();
        assert!(receiver_admits(&bare, "compile-1", 49));
        if receiver_admits(&bare, "unauthorized", 49) {
            fs::write(dir.path().join("bare-unauthorized"), b"bypass").unwrap();
        }
        assert!(!dir.path().join("bare-unauthorized").exists());
    }
    struct OnlyProfile(OperationProfileId);

    impl OperationProfileAdmission for OnlyProfile {
        fn admit_operation_profile(&self, profile: &OperationProfileId) -> Result<(), OperationProfileAdmissionError> {
            if profile == &self.0 {
                Ok(())
            } else {
                Err(OperationProfileAdmissionError::UnknownProfile)
            }
        }
    }

    #[derive(Default)]
    struct OnceReplay(Cell<bool>);

    impl HolderInvocationReplayAdmission for OnceReplay {
        fn admit_holder_invocation(
            &self,
            _candidate: &HolderInvocationReplayCandidate<'_>,
        ) -> Result<(), HolderInvocationReplayError> {
            if self.0.replace(true) {
                Err(HolderInvocationReplayError::Duplicate)
            } else {
                Ok(())
            }
        }
    }

    fn content_ref(bytes: &[u8]) -> String {
        format!("blake3:{}", blake3::hash(bytes).to_hex())
    }

    #[test]
    fn holder_signature_and_replay_precede_effect() {
        let issuer = Ed25519InMemorySigner::from_seed_bytes([4; ED25519_SECRET_KEY_BYTES]);
        let holder = Ed25519InMemorySigner::from_seed_bytes([5; ED25519_SECRET_KEY_BYTES]);
        let audience = AudienceDid::from(holder.issuer().unwrap());
        let token = issue_token_with_signer(
            &issuer,
            &audience,
            &capability(vec![caveat("00", PARENT)]),
            &ProofReferences::empty(),
            TokenTimeBounds::new(10, 100).unwrap(),
        )
        .unwrap();
        let keys =
            KeyResolutionContext::new(vec![issuer.verification_key().unwrap(), holder.verification_key().unwrap()]);
        let proofs = ProofCollection::empty();
        let verified = ucan::verify_compact_token(
            &token,
            &VerificationContext::new(VerificationTime::from_unix_seconds(40), keys.clone(), ProofCollection::empty()),
        )
        .unwrap();
        let chain = decode_chain(&verified.effective_delegation().capabilities().as_slice()[0].caveats).unwrap();
        let scope = RemoteJobScope {
            job_ids: &["compile-1"],
            output_class: "public",
            deadline_unix_s: 50,
        };
        let policy = RegisteredPolicy {
            chain: &chain,
            scope: &scope,
            job: "compile-1",
            now_unix_s: 40,
        };
        let profile_id = OperationProfileId::from_bytes(*blake3::hash(b"mantle.remote-build.v1").as_bytes());
        let operation = OperationBinding::new(
            profile_id,
            RequestBindingDigest::from_bytes(*blake3::hash(b"/job/compile-1/output/public").as_bytes()),
        );
        let nonce = HolderInvocationNonce::from_bytes([11; blake3::OUT_LEN]);
        let signature = sign_holder_invocation(
            &HolderInvocationSigningRequest::new(token.as_str(), &audience, RESOURCE, ABILITY, operation, nonce),
            &holder,
            VerificationLimits::default(),
            HolderInvocationLimits::default(),
        )
        .unwrap();
        let revocations = NoRevocations;
        let context = HolderTokenVerificationContext::new(
            VerificationTime::from_unix_seconds(40),
            &keys,
            &proofs,
            &revocations,
            VerificationLimits::default(),
        );
        let profiles = OnlyProfile(profile_id);
        let replay = OnceReplay::default();
        let policies =
            HolderInvocationPolicyContext::new(&policy, &profiles, &replay, HolderInvocationLimits::default());
        let request = HolderInvocationRequest::new(token.as_str(), RESOURCE, ABILITY, operation, nonce, &signature);
        let authorized = verify_holder_signed_invocation(&request, &context, &policies).unwrap();
        assert_eq!(authorized.audience().as_str(), audience.as_str());
        assert!(replay.0.get());
        assert!(verify_holder_signed_invocation(&request, &context, &policies).is_err());
        // Fixture-local refs represent the verified invocation above. A
        // production shell needs authentic verification receipts and its own
        // independently trusted policy reference; Basalt verifies neither.
        let runtime_policy = include_str!("../../../config/authority-caveats/generated/basalt-policy.json");
        let loaded_policy = basalt::parse_policy_json(runtime_policy).unwrap();
        let trusted_policy_ref = BasaltPolicyRef::new(
            content_ref(runtime_policy.as_bytes()),
            loaded_policy.schema_version.as_str(),
            "mantle-remote-build",
        );
        let request_ref = content_ref(b"/job/compile-1/output/public");
        let grant_ref = content_ref(token.as_str().as_bytes());
        let receipt_ref = content_ref(format!("ucan-verified-v1:{grant_ref}:{request_ref}").as_bytes());
        let authority = AuthorityInput {
            schema: AUTHORITY_INPUT_SCHEMA.to_owned(),
            request: AuthorityRequest {
                contract_id: "mantle-remote-build".to_owned(),
                resource: RESOURCE.to_owned(),
                ability: ABILITY.to_owned(),
                request_ref: request_ref.clone(),
            },
            verified_grants: vec![VerifiedGrantRef {
                grant_ref: grant_ref.clone(),
                verification_receipt_ref: receipt_ref.clone(),
                resource: RESOURCE.to_owned(),
                ability: ABILITY.to_owned(),
                kind: VerifiedGrantKind::Direct,
            }],
            verification_receipts: vec![UcanVerificationReceiptRef {
                receipt_ref,
                request_ref,
                verified_grant_refs: vec![grant_ref],
                disposition: VerificationRefDisposition::Current,
            }],
            policy_refs: vec![trusted_policy_ref.clone()],
            caveat: AuthorityCaveatDisposition::Satisfied,
            replay: AuthorityReplayDisposition::Admitted,
            claims: vec![AuthorityClaim::ExactRequest],
            non_claims: REQUIRED_AUTHORITY_NON_CLAIMS.map(str::to_owned).to_vec(),
        };
        let allowed = evaluate_authority(&loaded_policy, &trusted_policy_ref, &authority);
        assert!(allowed.is_allowed(), "{:?}", allowed.diagnostic);
        assert_eq!(allowed.binding.as_ref(), Some(&authority.request));
        let dir = tempfile::tempdir().unwrap();
        if allowed.is_allowed() {
            fs::write(dir.path().join("compile-1"), b"admitted").unwrap();
        }
        assert_eq!(fs::read(dir.path().join("compile-1")).unwrap(), b"admitted");

        let mut wrong_job = authority.clone();
        wrong_job.request.request_ref = content_ref(b"/job/compile-2/output/public");
        assert_eq!(
            evaluate_authority(&loaded_policy, &trusted_policy_ref, &wrong_job).diagnostic,
            Some(AuthorityDenyDiagnostic::VerificationRequestMismatch),
        );
        assert!(!scope.admits(&chain, "compile-2", "public", 40));
        assert!(!dir.path().join("compile-2").exists());
        let mut wrong_resource = authority.clone();
        wrong_resource.request.resource = STORE_RESOURCE.to_owned();
        assert_eq!(
            evaluate_authority(&loaded_policy, &trusted_policy_ref, &wrong_resource).diagnostic,
            Some(AuthorityDenyDiagnostic::WrongResource),
        );
        let mut wrong_ability = authority.clone();
        wrong_ability.request.ability = "store/read".to_owned();
        assert_eq!(
            evaluate_authority(&loaded_policy, &trusted_policy_ref, &wrong_ability).diagnostic,
            Some(AuthorityDenyDiagnostic::WrongAbility),
        );
        let mut denied_caveat = authority.clone();
        denied_caveat.caveat = AuthorityCaveatDisposition::Denied;
        assert_eq!(
            evaluate_authority(&loaded_policy, &trusted_policy_ref, &denied_caveat).diagnostic,
            Some(AuthorityDenyDiagnostic::CaveatDenied),
        );
        let mut stale_verification = authority.clone();
        stale_verification.verification_receipts[0].disposition = VerificationRefDisposition::Stale;
        assert_eq!(
            evaluate_authority(&loaded_policy, &trusted_policy_ref, &stale_verification).diagnostic,
            Some(AuthorityDenyDiagnostic::StaleVerificationRef),
        );
        let mut unavailable_replay = authority.clone();
        unavailable_replay.replay = AuthorityReplayDisposition::BackendError;
        assert_eq!(
            evaluate_authority(&loaded_policy, &trusted_policy_ref, &unavailable_replay).diagnostic,
            Some(AuthorityDenyDiagnostic::ReplayDenied),
        );
        let mut missing_policy = authority;
        missing_policy.policy_refs.clear();
        assert_eq!(
            evaluate_authority(&loaded_policy, &trusted_policy_ref, &missing_policy).diagnostic,
            Some(AuthorityDenyDiagnostic::MissingPolicyRef),
        );
    }
    struct SitePolicy(bool);

    impl CaveatPolicy for SitePolicy {
        fn evaluate(&self, caveat: &CaveatDocument, _request: &ucan::AuthorizationRequest) -> CaveatDecision {
            if self.0 && caveat.domain() == "mantle" && caveat.caveat_type() == "filter-v1" {
                CaveatDecision::Satisfied
            } else {
                CaveatDecision::Rejected {
                    message: "denied".to_owned(),
                }
            }
        }
    }

    impl CaveatPolicySet for SitePolicy {
        fn policy_for(&self, caveat: &CaveatIdentifier) -> Option<&dyn CaveatPolicy> {
            (caveat.domain() == "mantle" && caveat.caveat_type() == "filter-v1").then_some(self)
        }
    }

    fn policy_admits_verified_grant(
        loaded_policy: &basalt::Policy,
        verified: &ucan::VerifiedToken,
        contract_id: &str,
        resource: &str,
        ability: &str,
    ) -> bool {
        let grants = verified.effective_delegation().capabilities();
        let mut matching = grants
            .as_slice()
            .iter()
            .filter(|candidate| candidate.resource == resource && candidate.ability == ability);
        let Some(grant) = matching.next() else { return false };
        if matching.next().is_some() {
            return false;
        }
        let request = basalt::EnforcementRequest::new(contract_id, resource, ability)
            .with_capability(basalt::CapabilityGrant::new(grant.resource.as_str(), grant.ability.as_str()));
        basalt::enforce(loaded_policy, &request).is_ok_and(|receipt| receipt.is_allowed())
    }

    fn direct_verified_grant(resource: &str, ability: &str, payload: &[u8]) -> ucan::VerifiedToken {
        let issuer = Ed25519InMemorySigner::from_seed_bytes([7; ED25519_SECRET_KEY_BYTES]);
        let holder = Ed25519InMemorySigner::from_seed_bytes([8; ED25519_SECRET_KEY_BYTES]);
        let capability = CapabilitySet::new(vec![
            CapabilityDocument::with_caveats(resource.to_owned(), ability.to_owned(), vec![caveat("00", payload)])
                .unwrap(),
        ])
        .unwrap();
        let token = issue_token_with_signer(
            &issuer,
            &AudienceDid::from(holder.issuer().unwrap()),
            &capability,
            &ProofReferences::empty(),
            TokenTimeBounds::new(10, 100).unwrap(),
        )
        .unwrap();
        ucan::verify_compact_token(
            &token,
            &VerificationContext::new(
                VerificationTime::from_unix_seconds(40),
                KeyResolutionContext::new(vec![issuer.verification_key().unwrap()]),
                ProofCollection::empty(),
            ),
        )
        .unwrap()
    }

    #[test]
    fn signed_view_and_project_declaration_gate_real_filesystem_effects() {
        let loaded_policy =
            basalt::parse_policy_json(include_str!("../../../config/authority-caveats/generated/basalt-policy.json"))
                .unwrap();
        let store = direct_verified_grant(
            STORE_RESOURCE,
            "store/read",
            br#"{"kind":"rewrite","pattern":"/mantle/store/a-allowed","template":"/mantle/store/a-allowed"}"#,
        );
        let store_chain = decode_chain(&store.effective_delegation().capabilities().as_slice()[0].caveats).unwrap();
        assert_eq!(store_chain.admit("/mantle/store/a-allowed").as_deref(), Some("/mantle/store/a-allowed"));
        assert_eq!(store_chain.admit("/mantle/store/b-private"), None);
        let view = StoreView::new("/mantle/store", "/mantle/store/a-allowed", store_chain).unwrap();
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("a-allowed"), b"public bytes").unwrap();
        fs::write(dir.path().join("b-private"), b"secret bytes").unwrap();
        let request = |logical: &str| {
            let allowed_path = view.admits(logical);
            if !allowed_path {
                return None;
            }
            let policy = SitePolicy(allowed_path);
            if policy_admits_verified_grant(&loaded_policy, &store, "mantle-store-view", STORE_RESOURCE, "store/read")
                && matches!(
                    store.authorize_with_policies(STORE_RESOURCE, "store/read", &policy),
                    AuthorizationDecision::Allowed { .. }
                )
            {
                let basename = logical.strip_prefix("/mantle/store/").unwrap();
                fs::read(dir.path().join(basename)).ok()
            } else {
                None
            }
        };
        assert_eq!(request("/mantle/store/a-allowed"), Some(b"public bytes".to_vec()));
        assert_eq!(request("/mantle/store/b-private"), None);
        assert_eq!(request("/mantle/store/a-allowed/../b-private"), None);
        assert!(!policy_admits_verified_grant(
            &loaded_policy,
            &store,
            "mantle-project-goal",
            STORE_RESOURCE,
            "store/read"
        ));

        let project = direct_verified_grant(
            PROJECT_RESOURCE,
            "project/declare",
            br#"{"kind":"rewrite","pattern":"/project/alpha/declared/:goal","template":"/project/alpha/goal/:goal"}"#,
        );
        let goals = ProjectGoals::new(
            "alpha",
            decode_chain(&project.effective_delegation().capabilities().as_slice()[0].caveats).unwrap(),
        )
        .unwrap();
        for (owner, goal) in [("alpha", "build"), ("beta", "build")] {
            let Some(admitted) = goals.admit_goal(owner, goal) else {
                continue;
            };
            let policy = SitePolicy(true);
            let requested_resource = format!("mantle://project-goal/{owner}");
            if policy_admits_verified_grant(
                &loaded_policy,
                &project,
                "mantle-project-goal",
                &requested_resource,
                "project/declare",
            ) && matches!(
                project.authorize_with_policies(&requested_resource, "project/declare", &policy),
                AuthorizationDecision::Allowed { .. }
            ) {
                fs::write(dir.path().join(format!("{owner}-{goal}")), admitted).unwrap();
            }
        }
        assert!(!policy_admits_verified_grant(
            &loaded_policy,
            &project,
            "mantle-project-goal",
            "mantle://project-goal/beta",
            "project/declare",
        ));
        assert_eq!(fs::read_to_string(dir.path().join("alpha-build")).unwrap(), "/project/alpha/goal/build");
        assert!(!dir.path().join("beta-build").exists());
    }
}
