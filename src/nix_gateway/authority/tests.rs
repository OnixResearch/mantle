use std::os::unix::fs::PermissionsExt as _;

use basalt::ucan::AudienceDid;
use basalt::ucan::CapabilityDocument;
use basalt::ucan::CapabilitySet;
use basalt::ucan::Ed25519InMemorySigner;
use basalt::ucan::HolderInvocationLimits;
use basalt::ucan::HolderInvocationNonce;
use basalt::ucan::HolderInvocationSigningRequest;
use basalt::ucan::ProofReferences;
use basalt::ucan::TokenSigner;
use basalt::ucan::TokenTimeBounds;
use basalt::ucan::VerificationLimits;

use super::*;

fn install(path: &Path, document: &serde_json::Value) {
    let mut file = OpenOptions::new().write(true).create(true).truncate(true).mode(0o600).open(path).unwrap();
    use std::io::Write as _;
    file.write_all(&serde_json::to_vec(document).unwrap()).unwrap();
    file.sync_all().unwrap();
}
#[test]
fn private_signed_session_checks_live_revocation_generation_and_operation_without_replay_respend() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::set_permissions(dir.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
    let replay_dir = dir.path().join("replay");
    std::fs::create_dir(&replay_dir).unwrap();
    std::fs::set_permissions(&replay_dir, std::fs::Permissions::from_mode(0o700)).unwrap();
    let path = dir.path().join("authority.json");
    let issuer = Ed25519InMemorySigner::from_seed_bytes([11; 32]);
    let holder = Ed25519InMemorySigner::from_seed_bytes([17; 32]);
    let issuer_key = ed25519_dalek::SigningKey::from_bytes(&[11; 32]).verifying_key().to_bytes();
    let holder_key = ed25519_dalek::SigningKey::from_bytes(&[17; 32]).verifying_key().to_bytes();
    let mut document = serde_json::json!({
        "version": 1, "revision": 1, "generation": 7, "issued_unix_s": 1000, "expires_unix_s": 2000,
        "endpoint": "private-gateway", "audience": "build-service", "policy_json": basalt::DEFAULT_POLICY_JSON,
        "policy_blake3": blake3::hash(basalt::DEFAULT_POLICY_JSON.as_bytes()).to_hex().to_string(),
        "trusted_roots": [issuer.issuer().unwrap().as_str()], "keys": [issuer_key, holder_key], "proofs": [], "revoked": []
    });
    install(&path, &document);
    let verifier = GatewayVerifier::open(GatewayAuthorityConfig {
        authority_file: path.clone(),
        replay_directory: replay_dir,
    })
    .unwrap();
    let request = OperationRequest {
        operation: Operation::BuildSubmit,
        subject: holder.issuer().unwrap().as_str().into(),
        account: "account-a".into(),
        project: "project-a".into(),
        endpoint: "private-gateway".into(),
        audience: "build-service".into(),
        generation: 7,
        request_digest: [41; 32],
        target: "/jobs/approved".into(),
    };
    let exact = verifier.enforcement(&request, 1500).unwrap();
    let audience = AudienceDid::from(holder.issuer().unwrap());
    let token = ucan::issue_token_with_signer(
        &issuer,
        &audience,
        &CapabilitySet::new(vec![CapabilityDocument::new(exact.resource().into(), exact.ability().into()).unwrap()])
            .unwrap(),
        &ProofReferences::empty(),
        TokenTimeBounds::new(1100, 1900).unwrap(),
    )
    .unwrap();
    let nonce = HolderInvocationNonce::from_bytes([19; 32]);
    let operation = basalt::ucan_enforcement::bind_enforcement_action(request.operation.action(), &exact).unwrap();
    let signature = ucan::sign_holder_invocation(
        &HolderInvocationSigningRequest::new(
            token.as_str(),
            &audience,
            exact.resource(),
            exact.ability(),
            operation,
            nonce,
        ),
        &holder,
        VerificationLimits::default(),
        HolderInvocationLimits::default(),
    )
    .unwrap();
    // A rejected lifetime must not consume the exact holder invocation. Use
    // explicit clocks and this fixture's private ledger, never process time.
    for now in [999, 1099, 1900, 1901] {
        assert!(
            verifier
                .verify(
                    request.clone(),
                    Invocation {
                        compact_token: token.as_str(),
                        nonce,
                        holder_signature: &signature,
                    },
                    now,
                )
                .is_err()
        );
    }
    let session = verifier
        .verify(
            request.clone(),
            Invocation {
                compact_token: token.as_str(),
                nonce,
                holder_signature: &signature,
            },
            1500,
        )
        .unwrap();
    assert!(
        verifier
            .verify(
                request.clone(),
                Invocation {
                    compact_token: token.as_str(),
                    nonce,
                    holder_signature: &signature,
                },
                1500,
            )
            .is_err()
    );
    let authority = verifier.revalidate(&session, 1501).unwrap();
    assert!(authority.permits(crunch_nix_gateway_core::Permission::Build));
    assert!(!authority.permits(crunch_nix_gateway_core::Permission::Admin));
    session.require(Operation::BuildSubmit, &request.target, request.request_digest).unwrap();
    assert!(session.require(Operation::Admin, &request.target, request.request_digest).is_err());
    assert!(session.require(Operation::BuildSubmit, "/jobs/private", request.request_digest).is_err());
    assert!(session.require(Operation::BuildSubmit, &request.target, [42; 32]).is_err());
    assert!(verifier.revalidate(&session, 1899).is_ok());
    assert!(verifier.revalidate(&session, 1900).is_err());
    assert!(verifier.revalidate(&session, 1901).is_err());
    assert!(!format!("{session:?}").contains(token.as_str()));
    document["revision"] = serde_json::json!(2);
    document["revoked"] = serde_json::json!([ucan::proof_reference(&token).as_bytes()]);
    install(&path, &document);
    assert_eq!(verifier.revalidate(&session, 1502).unwrap_err().to_string(), "gateway-authority-denied");
    document["revision"] = serde_json::json!(3);
    document["revoked"] = serde_json::json!([]);
    document["generation"] = serde_json::json!(8);
    install(&path, &document);
    assert!(verifier.revalidate(&session, 1503).is_err());
    document["revision"] = serde_json::json!(1);
    document["generation"] = serde_json::json!(7);
    install(&path, &document);
    assert!(verifier.revalidate(&session, 1504).is_err());
}

#[test]
fn operator_authority_rejects_nonprivate_configuration() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("public.json");
    std::fs::write(&path, b"{}").unwrap();
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o644)).unwrap();
    assert!(private_read(&path).is_err());
}

struct ProjectFixture {
    _directory: tempfile::TempDir,
    verifier: GatewayVerifier,
    config: crate::project_build::authority::ProjectAuthorityConfig,
    request: OperationRequest,
    document: serde_json::Value,
    path: PathBuf,
}
impl ProjectFixture {
    fn new() -> Self {
        use crate::project_build::authority::ProjectAuthorityConfig;
        use crate::project_build::authority::declaration_digest;
        let directory = tempfile::tempdir().unwrap();
        std::fs::set_permissions(directory.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
        let replay = directory.path().join("replay");
        std::fs::create_dir(&replay).unwrap();
        std::fs::set_permissions(&replay, std::fs::Permissions::from_mode(0o700)).unwrap();
        let root = directory.path().join("mantle-project.ncl");
        std::fs::write(&root, "{ packages = { check = 42 } }").unwrap();
        let config = ProjectAuthorityConfig::open("project-a", &root).unwrap();
        let issuer = Ed25519InMemorySigner::from_seed_bytes([11; 32]);
        let holder = Ed25519InMemorySigner::from_seed_bytes([17; 32]);
        let keys: Vec<_> = [11, 13, 17]
            .into_iter()
            .map(|seed| ed25519_dalek::SigningKey::from_bytes(&[seed; 32]).verifying_key().to_bytes())
            .collect();
        let document = serde_json::json!({
            "version": 1, "revision": 1, "generation": 7, "issued_unix_s": 1000, "expires_unix_s": 2000,
            "endpoint": "private-gateway", "audience": "build-service", "policy_json": basalt::DEFAULT_POLICY_JSON,
            "policy_blake3": blake3::hash(basalt::DEFAULT_POLICY_JSON.as_bytes()).to_hex().to_string(),
            "trusted_roots": [issuer.issuer().unwrap().as_str()], "keys": keys, "proofs": [], "revoked": []
        });
        let path = directory.path().join("authority.json");
        install(&path, &document);
        let verifier = GatewayVerifier::open(GatewayAuthorityConfig {
            authority_file: path.clone(),
            replay_directory: replay,
        })
        .unwrap();
        let target = "/project/project-a/goal/packages/check";
        let request = OperationRequest {
            operation: Operation::ProjectDeclare,
            subject: holder.issuer().unwrap().as_str().into(),
            account: "account-a".into(),
            project: "project-a".into(),
            endpoint: "private-gateway".into(),
            audience: "build-service".into(),
            generation: 7,
            request_digest: declaration_digest(&config, Self::original(), target).unwrap(),
            target: target.into(),
        };
        Self {
            _directory: directory,
            verifier,
            config,
            request,
            document,
            path,
        }
    }
    fn original() -> &'static str {
        "/project/project-a/declared/check"
    }
    fn rewrite() -> ucan::CaveatDocument {
        filter("00", br#"{"kind":"rewrite","pattern":"/project/project-a/declared/:name","template":"/project/project-a/goal/packages/check"}"#)
    }
    fn sign(
        &mut self,
        operation: Operation,
        caveats: Vec<ucan::CaveatDocument>,
        parent: Vec<ucan::CaveatDocument>,
    ) -> (ucan::CompactToken, ucan::SignerSignature) {
        self.sign_with_extra(operation, caveats, parent, vec![])
    }
    fn sign_with_extra(
        &mut self,
        operation: Operation,
        caveats: Vec<ucan::CaveatDocument>,
        parent: Vec<ucan::CaveatDocument>,
        extra: Vec<CapabilityDocument>,
    ) -> (ucan::CompactToken, ucan::SignerSignature) {
        let issuer = Ed25519InMemorySigner::from_seed_bytes([11; 32]);
        let delegate = Ed25519InMemorySigner::from_seed_bytes([13; 32]);
        let holder = Ed25519InMemorySigner::from_seed_bytes([17; 32]);
        let mut request = self.request.clone();
        request.operation = operation;
        let exact = self.verifier.enforcement(&request, 1500).unwrap();
        let capability = |caveats| {
            let mut capabilities = extra.clone();
            capabilities.push(
                CapabilityDocument::with_caveats(exact.resource().into(), exact.ability().into(), caveats).unwrap(),
            );
            CapabilitySet::new(capabilities).unwrap()
        };
        let proof = ucan::issue_token_with_signer(
            &issuer,
            &AudienceDid::from(delegate.issuer().unwrap()),
            &capability(parent),
            &ProofReferences::empty(),
            TokenTimeBounds::new(1000, 1900).unwrap(),
        )
        .unwrap();
        let audience = AudienceDid::from(holder.issuer().unwrap());
        let token = ucan::issue_token_with_signer(
            &delegate,
            &audience,
            &capability(caveats),
            &ProofReferences::from_references(vec![ucan::proof_reference(&proof)]),
            TokenTimeBounds::new(1000, 1900).unwrap(),
        )
        .unwrap();
        self.document["revision"] = serde_json::json!(self.document["revision"].as_u64().unwrap() + 1);
        self.document["proofs"] = serde_json::json!([proof.as_str()]);
        install(&self.path, &self.document);
        let operation = basalt::ucan_enforcement::bind_enforcement_action(request.operation.action(), &exact).unwrap();
        let signature = ucan::sign_holder_invocation(
            &HolderInvocationSigningRequest::new(
                token.as_str(),
                &audience,
                exact.resource(),
                exact.ability(),
                operation,
                HolderInvocationNonce::from_bytes([19; 32]),
            ),
            &holder,
            VerificationLimits::default(),
            HolderInvocationLimits::default(),
        )
        .unwrap();
        (token, signature)
    }
}
fn filter(key: &str, payload: &[u8]) -> ucan::CaveatDocument {
    ucan::CaveatDocument::new("mantle".into(), "filter-v1".into(), key.into(), payload.into()).unwrap()
}
fn project_invocation<'a>(token: &'a ucan::CompactToken, signature: &'a ucan::SignerSignature) -> Invocation<'a> {
    Invocation {
        compact_token: token.as_str(),
        nonce: HolderInvocationNonce::from_bytes([19; 32]),
        holder_signature: signature,
    }
}

#[test]
fn signed_project_rewrite_drives_real_selector_and_isolates_owner_root_and_replay() {
    use crate::project_build::authority::ProjectAuthorityConfig;
    use crate::project_build::authority::resolve_project_target_with_authority;
    let mut fixture = ProjectFixture::new();
    let (token, signature) = fixture.sign(Operation::ProjectDeclare, vec![ProjectFixture::rewrite()], vec![]);
    let session = fixture
        .verifier
        .verify_project_declaration(
            fixture.request.clone(),
            ProjectFixture::original(),
            project_invocation(&token, &signature),
            1500,
        )
        .unwrap();
    let resolved =
        resolve_project_target_with_authority(&fixture.config, &fixture.verifier, &session, 1501, &[]).unwrap();
    assert_eq!(
        resolved.target,
        crate::project_build::ProjectTarget::Attribute(vec!["packages".into(), "check".into()])
    );
    assert_eq!(resolved.root_file, fixture.config.root_file());
    let expression = crate::project_build::generate_extraction_expr(&resolved.root_file, &resolved.target);
    assert!(expression.ends_with("_proj.\"packages\".\"check\""));
    assert!(fixture.verifier.revalidate(&session, 1502).is_ok());
    assert!(
        fixture
            .verifier
            .verify_project_declaration(
                fixture.request.clone(),
                ProjectFixture::original(),
                project_invocation(&token, &signature),
                1502
            )
            .is_err()
    );
    let wrong_owner = ProjectAuthorityConfig::open("project-b", fixture.config.root_file()).unwrap();
    assert!(resolve_project_target_with_authority(&wrong_owner, &fixture.verifier, &session, 1502, &[]).is_err());
    let other = tempfile::tempdir().unwrap();
    let root = other.path().join("mantle-project.ncl");
    std::fs::write(&root, "{}").unwrap();
    let wrong_root = ProjectAuthorityConfig::open("project-a", &root).unwrap();
    assert!(resolve_project_target_with_authority(&wrong_root, &fixture.verifier, &session, 1502, &[]).is_err());
    std::fs::remove_file(fixture.config.root_file()).unwrap();
    // Digest/owner denial must precede root discovery (which would disclose a
    // missing project), and all live sessions still retain their signed grant.
    assert_eq!(
        resolve_project_target_with_authority(&wrong_owner, &fixture.verifier, &session, 1502, &[])
            .err()
            .unwrap()
            .message(),
        "gateway-authority-denied"
    );
}

#[test]
fn signed_project_digest_substitution_and_build_relabel_fail_before_resolution() {
    use crate::project_build::authority::resolve_project_target_with_authority;
    let mut fixture = ProjectFixture::new();
    let (token, signature) = fixture.sign(Operation::ProjectDeclare, vec![ProjectFixture::rewrite()], vec![]);
    for field in 0..3 {
        let mut substituted = fixture.request.clone();
        match field {
            0 => substituted.request_digest[0] ^= 1,
            1 => substituted.project = "project-b".into(),
            _ => substituted.target = "/project/project-a/goal/packages/other".into(),
        }
        assert!(
            fixture
                .verifier
                .verify_project_declaration(
                    substituted,
                    ProjectFixture::original(),
                    project_invocation(&token, &signature),
                    1500
                )
                .is_err()
        );
    }
    for original in [
        "/project/project-a/declared/../check",
        "/project/project-a/declared/%2fcheck",
        "/project/project-a/declared//check",
        "/project/project-b/declared/check",
    ] {
        assert!(
            fixture
                .verifier
                .verify_project_declaration(
                    fixture.request.clone(),
                    original,
                    project_invocation(&token, &signature),
                    1500
                )
                .is_err()
        );
    }
    // This signed rewrite maps two originals to the SAME final selector. Only
    // the receiver's digest check can distinguish this original substitution.
    let session = fixture
        .verifier
        .verify_project_declaration(
            fixture.request.clone(),
            "/project/project-a/declared/other",
            project_invocation(&token, &signature),
            1500,
        )
        .unwrap();
    std::fs::remove_file(fixture.config.root_file()).unwrap();
    assert_eq!(
        resolve_project_target_with_authority(&fixture.config, &fixture.verifier, &session, 1501, &[])
            .err()
            .unwrap()
            .message(),
        "gateway-authority-denied"
    );
    let mut fixture = ProjectFixture::new();
    let (token, signature) = fixture.sign(Operation::BuildSubmit, vec![ProjectFixture::rewrite()], vec![]);
    assert!(
        fixture
            .verifier
            .verify_project_declaration(
                fixture.request.clone(),
                ProjectFixture::original(),
                project_invocation(&token, &signature),
                1500
            )
            .is_err()
    );
}

#[test]
fn signed_project_unknown_empty_cross_owner_and_parent_widening_are_silent() {
    for case in 0..5 {
        let mut fixture = ProjectFixture::new();
        let (caveats, parent) = match case {
            0 => (vec![], vec![]),
            1 => (vec![filter("00", br#"{"kind":"future"}"#)], vec![]),
            2 => (vec![filter("00", br#"{"kind":"rewrite","pattern":"/project/project-a/declared/:name","template":"/project/project-b/goal/:name"}"#)], vec![]),
            3 => {
                let parent = filter("00", br#"{"kind":"reject","pattern":"/project/project-a/declared/check"}"#);
                let child = filter("01", br#"{"kind":"rewrite","pattern":"/project/project-a/declared/check","template":"/project/project-a/goal/packages/check"}"#);
                (vec![parent.clone(), child], vec![parent])
            }
            _ => (vec![ProjectFixture::rewrite()], vec![filter("00", br#"{"kind":"reject","pattern":"/project/project-a/declared/check"}"#)]),
        };
        let (token, signature) = fixture.sign(Operation::ProjectDeclare, caveats, parent);
        std::fs::remove_file(fixture.config.root_file()).unwrap();
        let denied = fixture
            .verifier
            .verify_project_declaration(
                fixture.request.clone(),
                ProjectFixture::original(),
                project_invocation(&token, &signature),
                1500,
            )
            .unwrap_err();
        assert_eq!(denied.to_string(), "gateway-authority-denied");
    }
}

#[test]
fn signed_project_selects_only_first_canonical_matching_grant_in_mixed_capability_token() {
    use crate::project_build::authority::resolve_project_target_with_authority;
    for unrelated_caveats in [vec![], vec![
        ProjectFixture::rewrite(),
        filter("01", br#"{"kind":"reject","pattern":"/project/project-a/declared/check"}"#),
    ]] {
        let mut fixture = ProjectFixture::new();
        let unrelated = CapabilityDocument::with_caveats(
            "mantle://unrelated-store/v1".into(),
            "mantle/store/read".into(),
            unrelated_caveats,
        )
        .unwrap();
        let (token, signature) =
            fixture
                .sign_with_extra(Operation::ProjectDeclare, vec![ProjectFixture::rewrite()], vec![], vec![unrelated]);
        let session = fixture
            .verifier
            .verify_project_declaration(
                fixture.request.clone(),
                ProjectFixture::original(),
                project_invocation(&token, &signature),
                1500,
            )
            .unwrap();
        let resolved =
            resolve_project_target_with_authority(&fixture.config, &fixture.verifier, &session, 1501, &[]).unwrap();
        assert_eq!(
            resolved.target,
            crate::project_build::ProjectTarget::Attribute(vec!["packages".into(), "check".into()])
        );
        assert!(fixture.verifier.revalidate(&session, 1502).is_ok());
    }
}
