//! Actual holder-invocation binding regression, not a mock grant fixture.
use std::os::unix::fs::OpenOptionsExt as _;
use std::os::unix::fs::PermissionsExt as _;

use basalt::ucan::AudienceDid;
use basalt::ucan::CapabilityDocument;
use basalt::ucan::CapabilitySet;
use basalt::ucan::Ed25519InMemorySigner;
use basalt::ucan::HolderInvocationLimits;
use basalt::ucan::HolderInvocationNonce;
use basalt::ucan::HolderInvocationSigningRequest;
use basalt::ucan::ProofReferences;
use basalt::ucan::TokenSigner as _;
use basalt::ucan::TokenTimeBounds;
use basalt::ucan::VerificationLimits;
use basalt::ucan::{self};
use crunch_nix_gateway::api::Authorization as _;

use super::*;

#[test]
fn signed_submit_cannot_follow_endpoint_rotation_or_changed_idempotency_key() {
    let directory = tempfile::tempdir().unwrap();
    std::fs::set_permissions(directory.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
    let replay = directory.path().join("replay");
    std::fs::create_dir(&replay).unwrap();
    std::fs::set_permissions(&replay, std::fs::Permissions::from_mode(0o700)).unwrap();
    let issuer = Ed25519InMemorySigner::from_seed_bytes([11; 32]);
    let holder = Ed25519InMemorySigner::from_seed_bytes([17; 32]);
    let issuer_key = ed25519_dalek::SigningKey::from_bytes(&[11; 32]).verifying_key().to_bytes();
    let holder_key = ed25519_dalek::SigningKey::from_bytes(&[17; 32]).verifying_key().to_bytes();
    let snapshot = serde_json::json!({
        "version":1,"revision":1,"generation":7,"issued_unix_s":1000,"expires_unix_s":2000,
        "endpoint":"private-gateway","audience":"build-service","policy_json":basalt::DEFAULT_POLICY_JSON,
        "policy_blake3":blake3::hash(basalt::DEFAULT_POLICY_JSON.as_bytes()).to_hex().to_string(),
        "trusted_roots":[issuer.issuer().unwrap().as_str()],"keys":[issuer_key,holder_key],"proofs":[],"revoked":[]
    });
    let authority_file = directory.path().join("authority.json");
    {
        use std::io::Write as _;
        let mut file =
            std::fs::OpenOptions::new().write(true).create_new(true).mode(0o600).open(&authority_file).unwrap();
        file.write_all(&serde_json::to_vec(&snapshot).unwrap()).unwrap();
        file.sync_all().unwrap();
    }
    let verifier = GatewayVerifier::open(GatewayAuthorityConfig {
        authority_file,
        replay_directory: replay,
    })
    .unwrap();
    let drv = "/nix/store/00000000000000000000000000000000-fixture.drv";
    let request = Request::Submit {
        drv_path: drv,
        idempotency_key: "opaque-key-one",
    };
    let (operation, target, digest) = request_binding(request, "project-a", 3).unwrap();
    let signed_request = super::super::authority::OperationRequest {
        operation,
        subject: holder.issuer().unwrap().as_str().into(),
        account: "account-a".into(),
        project: "project-a".into(),
        endpoint: "private-gateway".into(),
        audience: "build-service".into(),
        generation: 7,
        request_digest: digest,
        target,
    };
    let exact = verifier.enforcement(&signed_request, 1500).unwrap();
    let audience = AudienceDid::from(holder.issuer().unwrap());
    let token = ucan::issue_token_with_signer(
        &issuer,
        &audience,
        &CapabilitySet::new(vec![CapabilityDocument::new(exact.resource().into(), exact.ability().into()).unwrap()])
            .unwrap(),
        &ProofReferences::empty(),
        TokenTimeBounds::new(1000, 1900).unwrap(),
    )
    .unwrap();
    let nonce = HolderInvocationNonce::from_bytes([19; 32]);
    let binding = basalt::ucan_enforcement::bind_enforcement_action(operation.action(), &exact).unwrap();
    let signature = ucan::sign_holder_invocation(
        &HolderInvocationSigningRequest::new(
            token.as_str(),
            &audience,
            exact.resource(),
            exact.ability(),
            binding,
            nonce,
        ),
        &holder,
        VerificationLimits::default(),
        HolderInvocationLimits::default(),
    )
    .unwrap();
    let invocation = super::super::authority::Invocation {
        compact_token: token.as_str(),
        nonce,
        holder_signature: &signature,
    };
    let session = verifier.verify(signed_request.clone(), invocation, 1500).unwrap();
    let authorization = GatewayAuthorization {
        session: Arc::new(session),
    };
    authorization.require_target(Operation::BuildSubmit, drv).unwrap();
    assert!(authorization.require_target(Operation::LogRead, drv).is_err());
    authorization.session.require(operation, &signed_request.target, digest).unwrap();
    let (_, _, rotated) = request_binding(request, "project-a", 4).unwrap();
    assert!(authorization.session.require(operation, &signed_request.target, rotated).is_err());
    let (_, _, changed_key) = request_binding(
        Request::Submit {
            drv_path: drv,
            idempotency_key: "opaque-key-two",
        },
        "project-a",
        3,
    )
    .unwrap();
    assert!(authorization.session.require(operation, &signed_request.target, changed_key).is_err());
    // A pure projection is narrower than the actual live session retained by
    // the guard; it never creates read or administration permission.
    assert!(authorization.facts().permits(crunch_nix_gateway_core::Permission::Build));
    assert!(!authorization.facts().permits(crunch_nix_gateway_core::Permission::StatusRead));
    assert!(!authorization.facts().permits(crunch_nix_gateway_core::Permission::Admin));
    assert!(request_binding(request, "project-a", 0).is_err());
}

#[test]
fn private_gateway_call_rejects_ambiguous_or_unknown_peer_outcomes() {
    use std::io::Write as _;
    let responses: &[(&[u8], Result<bool, super::Error>)] = &[
        (br#"{"version":1,"response":{"Submitted":{"attempt_id":"attempt-1","recovered":false}}}"#, Ok(true)),
        (br#"{"response":{"Submitted":{"attempt_id":"attempt-1","recovered":false}}}"#, Ok(false)),
        (br#"{"version":1,"error":"unknown-private-error"}"#, Err(super::Error::Integrity)),
        (br#"{"version":2,"error":"gateway-authority-denied"}"#, Err(super::Error::Integrity)),
        (br#"{"version":1,"response":{},"error":"gateway-authority-denied"}"#, Err(super::Error::Integrity)),
        (br#"{"version":1,"error":null}"#, Err(super::Error::Integrity)),
    ];
    for &(response, expected) in responses {
        let directory = tempfile::tempdir().unwrap();
        std::fs::set_permissions(directory.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
        let socket = directory.path().join("gateway.sock");
        let envelope = directory.path().join("envelope.json");
        std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(&envelope)
            .unwrap()
            .write_all(b"{\"credential\":\"private-fixture\"}")
            .unwrap();
        let listener = std::os::unix::net::UnixListener::bind(&socket).unwrap();
        let response = response.to_vec();
        let peer = std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            admit_peer(&stream).unwrap();
            let _request = receive_frame(&mut stream, MAX_ENVELOPE_BYTES).unwrap();
            send_frame(&mut stream, &response).unwrap();
        });
        let plan = plan_gateway_operation(GatewayOperation::Call).unwrap();
        let observed = call_private(&socket, &envelope)
            .map(|result| classify_gateway_operation(&plan, &result.evidence) == ApplicationOutcome::Completed);
        peer.join().unwrap();
        assert_eq!(observed, expected);
    }
}

#[test]
fn gateway_requires_exact_canonical_resources_for_each_reviewed_system() {
    let features = BTreeMap::from([("x86_64-linux".into(), crunch_remote_core::resource_policy::Features {
        architecture: "x86_64".into(),
        platform: "linux".into(),
        trust_domain: "test".into(),
        isolation: "sandbox".into(),
        kvm: false,
    })]);
    let requirements: remote_build::RemoteResourceRequirements = serde_json::from_value(serde_json::json!({
        "quantities":{"cpu_units":1,"memory_bytes":1024,"scratch_bytes":1024,"accelerators":[],"named_tokens":[]},
        "semantic_accelerator_classes":[]
    }))
    .unwrap();
    let declared = BTreeMap::from([("x86_64-linux".into(), requirements.clone())]);
    assert!(validate_required_resources(&declared, &features).is_ok());
    assert!(validate_required_resources(&BTreeMap::new(), &features).is_err());
    let wrong_system = BTreeMap::from([("aarch64-linux".into(), requirements.clone())]);
    assert!(validate_required_resources(&wrong_system, &features).is_err());
    let mut invalid = requirements;
    invalid.quantities.cpu_units = 0;
    assert!(validate_required_resources(&BTreeMap::from([("x86_64-linux".into(), invalid)]), &features).is_err());
}
