//! Holder-attenuated remote grants at the real receiving boundary: real ticket
//! state on disk, the production admission functions, the operator-loaded
//! authority snapshot, and the receiver's own clock. Every rejected case also
//! proves the ticket state is byte-for-byte unchanged and no frame was written.
use std::io::Cursor;
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
use basalt::ucan::TokenSigner;
use basalt::ucan::TokenTimeBounds;
use basalt::ucan::VerificationLimits;
use basalt::ucan::{self};

use super::*;
use crate::nix_gateway::authority::GatewayVerifier;
use crate::nix_gateway::authority::OperationRequest;
use crate::remote_build::attenuated_admission::ATTENUATED_GRANT_DENIED;
use crate::remote_build::attenuated_admission::AttenuatedGrantPresentation;
use crate::remote_build::attenuated_admission::AttenuationAdmission;
use crate::remote_build::attenuated_admission::admit_presented_grant;
use crate::remote_build::attenuated_admission::bind_remote_job;
use crate::remote_build::attenuated_admission::open_operator_authority;
use crate::remote_build::authority_caveats::EffectiveCaveatChain;

const SCOPE: &str = "fixture-builds";
const ACCOUNT: &str = "account-a";
const AUDIENCE: &str = "build-service";
const GENERATION: u64 = 7;
const ROOT_SEED: u8 = 11;
const DELEGATE_SEED: u8 = 13;
const HOLDER_SEED: u8 = 17;
const TICKET_USES: u32 = 3;

struct Window {
    not_before: u64,
    expires: u64,
}

struct GrantFixture {
    state: PathBuf,
    verifier: GatewayVerifier,
    document: serde_json::Value,
    proofs: Vec<String>,
    revoked: Vec<Vec<u8>>,
    endpoint: String,
    next_nonce: u8,
}

fn filter(key: &str, payload: &str) -> ucan::CaveatDocument {
    ucan::CaveatDocument::new("mantle".into(), "filter-v1".into(), key.into(), payload.as_bytes().to_vec()).unwrap()
}

fn identity_rewrite(key: &str, job: &str) -> ucan::CaveatDocument {
    let value = format!("/job/{job}/output/action");
    filter(key, &format!(r#"{{"kind":"rewrite","pattern":"{value}","template":"{value}"}}"#))
}

fn reject_job(key: &str, job: &str) -> ucan::CaveatDocument {
    filter(key, &format!(r#"{{"kind":"reject","pattern":"/job/{job}/output/action"}}"#))
}

fn signer(seed: u8) -> Ed25519InMemorySigner {
    Ed25519InMemorySigner::from_seed_bytes([seed; 32])
}

fn public_key(seed: u8) -> [u8; 32] {
    ed25519_dalek::SigningKey::from_bytes(&[seed; 32]).verifying_key().to_bytes()
}

fn install(path: &Path, document: &serde_json::Value) {
    use std::io::Write as _;
    let mut file = std::fs::OpenOptions::new().write(true).create(true).truncate(true).mode(0o600).open(path).unwrap();
    file.write_all(&serde_json::to_vec(document).unwrap()).unwrap();
    file.sync_all().unwrap();
}

fn now() -> u64 {
    crate::unix_time_now_s().unwrap()
}

impl GrantFixture {
    /// Provision the operator snapshot beside the ticket state and load it
    /// through the production loader, exactly as `remote serve` does.
    fn new(state: &Path, endpoint: &str) -> Self {
        let issuer = signer(ROOT_SEED);
        let document = serde_json::json!({
            "version": 1, "revision": 1, "generation": GENERATION,
            "issued_unix_s": now() - 1000, "expires_unix_s": now() + 7200,
            "endpoint": endpoint, "audience": AUDIENCE, "policy_json": basalt::DEFAULT_POLICY_JSON,
            "policy_blake3": blake3::hash(basalt::DEFAULT_POLICY_JSON.as_bytes()).to_hex().to_string(),
            "trusted_roots": [issuer.issuer().unwrap().as_str()],
            "keys": [public_key(ROOT_SEED), public_key(DELEGATE_SEED), public_key(HOLDER_SEED)],
            "proofs": [], "revoked": []
        });
        install(&state.join("attenuation-authority.json"), &document);
        let replay = state.join("attenuation-replay");
        std::fs::create_dir(&replay).unwrap();
        std::fs::set_permissions(&replay, std::fs::Permissions::from_mode(0o700)).unwrap();
        let verifier = open_operator_authority(state).unwrap().expect("provisioned authority loads");
        Self {
            state: state.to_path_buf(),
            verifier,
            document,
            proofs: Vec::new(),
            revoked: Vec::new(),
            endpoint: endpoint.to_string(),
            next_nonce: 1,
        }
    }

    fn publish(&mut self) {
        self.document["revision"] = serde_json::json!(self.document["revision"].as_u64().unwrap() + 1);
        self.document["proofs"] = serde_json::json!(self.proofs);
        self.document["revoked"] = serde_json::json!(self.revoked);
        install(&self.state.join("attenuation-authority.json"), &self.document);
    }

    fn operation(&self, request: &ConcreteBuildRequest) -> OperationRequest {
        let binding = bind_remote_job(request, &self.endpoint).unwrap();
        OperationRequest {
            operation: crate::nix_gateway::authority::Operation::BuildSubmit,
            subject: signer(HOLDER_SEED).issuer().unwrap().as_str().into(),
            account: ACCOUNT.into(),
            project: SCOPE.into(),
            endpoint: self.endpoint.clone(),
            audience: AUDIENCE.into(),
            generation: GENERATION,
            request_digest: binding.request_digest,
            target: binding.target,
        }
    }

    /// A parent grant (root to delegate) and a holder child, each preserving the
    /// caveat chains given, signed for exactly `request`.
    fn sign(
        &mut self,
        request: &ConcreteBuildRequest,
        parent: Vec<ucan::CaveatDocument>,
        child: Vec<ucan::CaveatDocument>,
        window: Window,
    ) -> (AttenuatedGrantPresentation, ucan::CompactToken) {
        let operation = self.operation(request);
        let exact = self.verifier.enforcement(&operation, now()).unwrap();
        let capability = |caveats| {
            CapabilitySet::new(vec![
                CapabilityDocument::with_caveats(exact.resource().into(), exact.ability().into(), caveats).unwrap(),
            ])
            .unwrap()
        };
        let holder = signer(HOLDER_SEED);
        let audience = AudienceDid::from(holder.issuer().unwrap());
        let proof = ucan::issue_token_with_signer(
            &signer(ROOT_SEED),
            &AudienceDid::from(signer(DELEGATE_SEED).issuer().unwrap()),
            &capability(parent),
            &ProofReferences::empty(),
            TokenTimeBounds::new(now() - 600, now() + 3600).unwrap(),
        )
        .unwrap();
        let token = ucan::issue_token_with_signer(
            &signer(DELEGATE_SEED),
            &audience,
            &capability(child),
            &ProofReferences::from_references(vec![ucan::proof_reference(&proof)]),
            TokenTimeBounds::new(window.not_before, window.expires).unwrap(),
        )
        .unwrap();
        self.proofs.push(proof.as_str().to_string());
        self.publish();
        let nonce_bytes = [self.next_nonce; 32];
        let nonce = HolderInvocationNonce::from_bytes(nonce_bytes);
        self.next_nonce += 1;
        let bound = basalt::ucan_enforcement::bind_enforcement_action(operation.operation.action(), &exact).unwrap();
        let signature = ucan::sign_holder_invocation(
            &HolderInvocationSigningRequest::new(
                token.as_str(),
                &audience,
                exact.resource(),
                exact.ability(),
                bound,
                nonce,
            ),
            &holder,
            VerificationLimits::default(),
            HolderInvocationLimits::default(),
        )
        .unwrap();
        let presentation = AttenuatedGrantPresentation {
            compact_token: token.as_str().to_string(),
            nonce: nonce_bytes,
            holder_signature: signature.as_bytes().to_vec(),
            subject: operation.subject,
            account: operation.account,
            project: operation.project,
            audience: operation.audience,
            generation: operation.generation,
        };
        (presentation, token)
    }

    fn current_window() -> Window {
        Window {
            not_before: now() - 300,
            expires: now() + 1800,
        }
    }
}

struct Receiver {
    dir: tempfile::TempDir,
    key: std::sync::Arc<crate::remote_credentials::TicketVerifierKey>,
    auth: TicketAuthRequest,
    request: ConcreteBuildRequest,
    grants: GrantFixture,
}

impl Receiver {
    fn new() -> Self {
        let dir = tempfile::tempdir().unwrap();
        let (key, auth, request) = persist_transaction_ticket(dir.path(), TICKET_USES);
        let grants = GrantFixture::new(dir.path(), &fixture_loopback_builder().endpoint_id);
        Self {
            dir,
            key,
            auth,
            request,
            grants,
        }
    }

    fn other_job(&self, request_id: &str) -> ConcreteBuildRequest {
        ConcreteBuildRequest {
            request_id: request_id.to_string(),
            ..self.request.clone()
        }
    }

    fn state(&self) -> RemoteTicketState {
        load_ticket_state(self.dir.path()).unwrap()
    }

    fn uses_remaining(&self) -> u32 {
        self.state().tickets[TRANSACTION_TICKET_ID].uses_remaining
    }

    /// Run the production ticket admission with the receiver-owned facts.
    fn admit(
        &self,
        request: &ConcreteBuildRequest,
        presentation: Option<AttenuatedGrantPresentation>,
        verifier: Option<&GatewayVerifier>,
    ) -> Result<CommittedRemoteTicketAdmission, String> {
        let mut auth = self.auth.clone();
        auth.attenuated_grant = presentation;
        let attenuation = AttenuationAdmission {
            verifier,
            endpoint_id: &self.grants.endpoint,
            project_scope: SCOPE,
        };
        let mut effects = crunch_remote_core::effect::EffectSession::new(&request.request_id).unwrap();
        commit_remote_ticket_admission_for_state(
            self.dir.path(),
            &self.key,
            None,
            &auth,
            request,
            &mut effects,
            Some(&attenuation),
        )
    }

    fn admit_configured(
        &self,
        request: &ConcreteBuildRequest,
        presentation: AttenuatedGrantPresentation,
    ) -> Result<CommittedRemoteTicketAdmission, String> {
        self.admit(request, Some(presentation), Some(&self.grants.verifier))
    }

    fn assert_refused_without_effect(&self, result: Result<CommittedRemoteTicketAdmission, String>, case: &str) {
        let before = self.state();
        assert_eq!(result.err().as_deref(), Some(ATTENUATED_GRANT_DENIED), "{case}");
        assert_eq!(self.state(), before, "{case}: ticket state changed by a refused request");
        assert_eq!(self.uses_remaining(), TICKET_USES, "{case}: a ticket use was redeemed");
    }
}

// A restricted grant authorizes its declared job: the ticket is redeemed once,
// with the receiver's own clock, and legacy bearer admission is unchanged.
// r[verify mantle.authority_attenuation.attenuated_remote_grant]
#[test]
fn restricted_grant_authorizes_its_declared_job_and_legacy_bearer_is_unchanged() {
    let mut receiver = Receiver::new();
    let request = receiver.request.clone();
    let (presentation, _) =
        receiver
            .grants
            .sign(&request, vec![], vec![identity_rewrite("00", "r1")], GrantFixture::current_window());
    let admitted = receiver.admit_configured(&request, presentation).expect("declared job is admitted");
    assert!(admitted.server_now_unix_s >= now() - 5);
    assert_eq!(receiver.uses_remaining(), TICKET_USES - 1);

    // Legacy bearer admission (no presentation) keeps working, with or without
    // an operator authority configured, and is never described as delegated.
    receiver.admit(&request, None, None).expect("legacy bearer admission unchanged");
    receiver
        .admit(&request, None, Some(&receiver.grants.verifier))
        .expect("legacy bearer admission unchanged");
    assert_eq!(receiver.uses_remaining(), TICKET_USES - 3);
    let legacy = EffectiveCaveatChain::legacy_bearer();
    assert_eq!(legacy.delegation, "legacy-bearer-not-delegated");
    assert_eq!(legacy.caveat_count, 0);
    assert!(legacy.caveats.is_empty());
}

// A further attenuation composes: the holder keeps the parent's job set and
// appends a reject. The newest restriction closes the second job even though a
// valid signature exists for it, while the first job stays admitted.
// r[verify mantle.authority_attenuation.composition_order]
#[test]
fn further_attenuation_composes_and_a_second_job_is_refused_without_effect() {
    let mut receiver = Receiver::new();
    let first = receiver.request.clone();
    let second = receiver.other_job("r2");
    let parent = vec![filter(
        "00",
        r#"{"kind":"alternatives","alternatives":[{"pattern":"/job/r1/output/action","template":"/job/r1/output/action"},{"pattern":"/job/r2/output/action","template":"/job/r2/output/action"}]}"#,
    )];
    let child = vec![parent[0].clone(), reject_job("01", "r2")];
    let (for_first, _) = receiver.grants.sign(&first, parent.clone(), child.clone(), GrantFixture::current_window());
    let (for_second, _) = receiver.grants.sign(&second, parent, child, GrantFixture::current_window());

    let before = receiver.state();
    receiver.assert_refused_without_effect(receiver.admit_configured(&second, for_second), "second job");
    assert_eq!(receiver.state(), before);
    receiver.admit_configured(&first, for_first).expect("declared first job stays admitted");
    assert_eq!(receiver.uses_remaining(), TICKET_USES - 1);
}

// Every grant reports its effective chain in preserved order, redacted: no
// compact token, signature or secret, only ordinals, kinds and digests.
// r[verify mantle.authority_attenuation.composition_order]
#[test]
fn issued_grant_reports_its_effective_caveat_chain_redacted() {
    let mut receiver = Receiver::new();
    let request = receiver.request.clone();
    let parent = vec![identity_rewrite("00", "r1")];
    let child = vec![parent[0].clone(), reject_job("01", "r9")];
    let (presentation, token) = receiver.grants.sign(&request, parent, child, GrantFixture::current_window());
    let attenuation = AttenuationAdmission {
        verifier: Some(&receiver.grants.verifier),
        endpoint_id: &receiver.grants.endpoint,
        project_scope: SCOPE,
    };
    let chain = admit_presented_grant(&attenuation, &presentation, &request, now()).expect("grant admits");
    assert_eq!(chain.delegation, "holder-attenuated-ucan");
    assert_eq!(chain.status, "bounded-filter-chain");
    assert_eq!(chain.caveat_count, 2);
    let described: Vec<_> = chain.caveats.iter().map(|entry| (entry.ordinal.as_str(), entry.kind)).collect();
    assert_eq!(described, [("00", "rewrite"), ("01", "reject")]);
    assert_eq!(chain.caveats[1].pattern.as_deref(), Some("/job/r9/output/action"));
    let report = serde_json::to_string(&chain).unwrap();
    assert!(!report.contains(token.as_str()));
    assert!(!report.contains(&data_encoding::HEXLOWER.encode(&presentation.holder_signature)));
    assert_eq!(chain.chain_blake3.len(), 64);

    // Unknown and malformed caveats report a deny-all chain, never their text.
    let hostile = [
        ucan::CaveatDocument::new("mantle".into(), "filter-v1".into(), "00".into(), br#"{"kind":"future"}"#.to_vec())
            .unwrap(),
        filter("01", r#"{"kind":"rewrite","pattern":"/job/secret-denied-path/output/action"}"#),
    ];
    let denied = EffectiveCaveatChain::from_documents(&hostile);
    assert_eq!(denied.status, "denies-everything");
    assert_eq!(denied.caveats.iter().map(|entry| entry.kind).collect::<Vec<_>>(), [
        "unrecognized-deny-all",
        "malformed-deny-all"
    ]);
    assert!(!serde_json::to_string(&denied).unwrap().contains("secret-denied-path"));
}

// An unknown caveat rejects everything; so do malformed, ordinal-gapped,
// over-bound, parent-widening, expired, revoked, replayed and re-targeted
// presentations. Each leaves the real ticket state unchanged and returns only
// the one constant refusal.
// r[verify mantle.authority_attenuation.caveat_filter_semantics]
#[test]
fn unsafe_presentations_are_refused_before_redemption_with_no_feedback() {
    let cases = [
        "unknown-caveat",
        "malformed-caveat",
        "ordinal-gap",
        "chain-over-bound",
        "parent-rejected-input-rewritten",
        "expired-deadline",
        "revoked-grant",
        "wrong-project-scope",
        "other-job-signature",
        "changed-request-field",
        "oversized-token",
    ];
    for case in cases {
        let mut receiver = Receiver::new();
        let request = receiver.request.clone();
        let window = if case == "expired-deadline" {
            Window {
                not_before: now() - 500,
                expires: now() - 60,
            }
        } else {
            GrantFixture::current_window()
        };
        let (parent, child) = match case {
            "unknown-caveat" => (vec![], vec![filter("00", r#"{"kind":"future"}"#)]),
            "malformed-caveat" => {
                (vec![], vec![filter("00", r#"{"kind":"rewrite","pattern":"/job/r1/output/action"}"#)])
            }
            "ordinal-gap" => (vec![], vec![identity_rewrite("01", "r1")]),
            "chain-over-bound" => {
                let chain: Vec<_> = (0..9).map(|index| identity_rewrite(&format!("{index:02}"), "r1")).collect();
                (chain.clone(), chain)
            }
            "parent-rejected-input-rewritten" => {
                let parent = reject_job("00", "r1");
                let child = filter(
                    "01",
                    r#"{"kind":"rewrite","pattern":"/job/r1/output/action","template":"/job/r1/output/action"}"#,
                );
                (vec![parent.clone()], vec![parent, child])
            }
            _ => (vec![], vec![identity_rewrite("00", "r1")]),
        };
        let signed_for = if case == "other-job-signature" {
            receiver.other_job("r2")
        } else {
            request.clone()
        };
        let (mut presentation, token) = receiver.grants.sign(&signed_for, parent, child, window);
        match case {
            "revoked-grant" => {
                receiver.grants.revoked.push(ucan::proof_reference(&token).as_bytes().to_vec());
                receiver.grants.publish();
            }
            "wrong-project-scope" => presentation.project = "another-project".into(),
            "oversized-token" => presentation.compact_token.push_str(&"A".repeat(40 * 1024)),
            _ => {}
        }
        let mut presented = request.clone();
        if case == "changed-request-field" {
            presented.build_time_limit_secs = presented.build_time_limit_secs.saturating_sub(1);
        }
        let result = receiver.admit_configured(&presented, presentation);
        receiver.assert_refused_without_effect(result, case);
    }
}

// Replaying an admitted holder invocation, presenting with no operator
// authority, and trusting a forged client clock all fail closed.
// r[verify mantle.authority_attenuation.attenuated_remote_grant]
#[test]
fn replay_missing_authority_and_forged_client_time_fail_closed() {
    let mut receiver = Receiver::new();
    let request = receiver.request.clone();
    let (presentation, _) =
        receiver
            .grants
            .sign(&request, vec![], vec![identity_rewrite("00", "r1")], GrantFixture::current_window());
    receiver.admit_configured(&request, presentation.clone()).expect("first presentation admits");
    assert_eq!(receiver.uses_remaining(), TICKET_USES - 1);
    let before = receiver.state();
    let replayed = receiver.admit_configured(&request, presentation.clone());
    assert_eq!(replayed.err().as_deref(), Some(ATTENUATED_GRANT_DENIED));
    assert_eq!(receiver.state(), before, "a replayed holder invocation redeemed a ticket use");

    // No operator authority configured: a presented grant is never trusted.
    let (second, _) =
        receiver
            .grants
            .sign(&request, vec![], vec![identity_rewrite("00", "r1")], GrantFixture::current_window());
    let before = receiver.state();
    let unconfigured = receiver.admit(&request, Some(second.clone()), None);
    assert_eq!(unconfigured.err().as_deref(), Some(ATTENUATED_GRANT_DENIED));
    assert_eq!(receiver.state(), before);

    // The sender's claimed clock never feeds admission: the receiver observes
    // its own time, so an absurd claim neither helps nor hurts.
    let mut forged = receiver.auth.clone();
    forged.now_unix_s = u64::MAX;
    forged.attenuated_grant = Some(second);
    let attenuation = AttenuationAdmission {
        verifier: Some(&receiver.grants.verifier),
        endpoint_id: &receiver.grants.endpoint,
        project_scope: SCOPE,
    };
    let mut effects = crunch_remote_core::effect::EffectSession::new(&request.request_id).unwrap();
    commit_remote_ticket_admission_for_state(
        receiver.dir.path(),
        &receiver.key,
        None,
        &forged,
        &request,
        &mut effects,
        Some(&attenuation),
    )
    .expect("valid grant admits regardless of the sender's clock claim");
    assert_eq!(receiver.uses_remaining(), TICKET_USES - 2);
}

// The chain bound is exact: eight preserved caveats admit, a ninth cannot.
// r[verify mantle.authority_attenuation.composition_order]
#[test]
fn chain_at_the_bound_admits() {
    let mut receiver = Receiver::new();
    let request = receiver.request.clone();
    let chain: Vec<_> = (0..crunch_remote_core::caveat::MAX_CHAIN)
        .map(|index| identity_rewrite(&format!("{index:02}"), "r1"))
        .collect();
    let (presentation, _) = receiver.grants.sign(&request, chain.clone(), chain, GrantFixture::current_window());
    receiver.admit_configured(&request, presentation).expect("a chain at the bound admits");
    assert_eq!(receiver.uses_remaining(), TICKET_USES - 1);
}

// The production serve path: a refused presentation writes no protocol frame,
// consumes no input, redeems nothing and surfaces only the constant refusal.
// r[verify mantle.authority_attenuation.caveat_filter_semantics]
#[test]
fn production_serve_refuses_unsafe_presentation_with_no_frame_and_no_state_change() {
    let mut receiver = Receiver::new();
    let builder = fixture_loopback_builder();
    let executor = fixture_local_build_executor();
    let config = endpoint_binding_tests::provision(receiver.dir.path(), &builder);
    let mut client = fixture_loopback_client(vec!["input-a".to_string()], vec!["builder-key".to_string()]);
    client.request.transfer_policy = Some(RemoteTransferPolicy::default());
    client.request.endpoint_binding = Some(config.build_binding().unwrap().clone());
    client.request.production_attempt = Some(RemoteProductionAttemptBinding {
        job_id: RemoteJobId::new("attenuation-test-job".to_string()).unwrap(),
        attempt_id: RemoteAttemptId::new("attenuation-test-attempt".to_string()).unwrap(),
        fence_generation: RemoteFenceGeneration::INITIAL,
    });
    let signed_request = client.request.clone();
    let (presentation, _) = receiver.grants.sign(
        &signed_request,
        vec![],
        vec![filter("00", r#"{"kind":"future"}"#)],
        GrantFixture::current_window(),
    );
    client.auth = receiver.auth.clone();
    client.auth.attenuated_grant = Some(presentation);
    let frames = vec![
        RemoteFrame::Hello { hello: client.hello },
        RemoteFrame::AuthTicket { auth: client.auth },
        RemoteFrame::BuildRequest {
            request: client.request,
        },
        RemoteFrame::InputManifest {
            manifest: client.input_manifest,
        },
    ];
    let encoded = encode_frame_stream(&frames);
    let mut reader = Cursor::new(encoded);
    let mut writer = Vec::new();
    let context = RemoteProductionServerContext {
        builder: &builder,
        verifier_key: &receiver.key,
        executor: &executor,
        secret_request: None,
        usage_cgroup_root: None,
        credential_state_dir: receiver.dir.path(),
        binding_config: Some(&config),
        execution_state_dir: receiver.dir.path(),
        authenticated_client_endpoint: None,
        attenuation_verifier: Some(&receiver.grants.verifier),
    };
    let before = receiver.state();
    let error = serve_stdio_remote_production_once(&mut reader, &mut writer, context).unwrap_err();
    assert_eq!(error, ATTENUATED_GRANT_DENIED);
    assert!(writer.is_empty(), "a refused presentation wrote a protocol frame");
    assert_eq!(receiver.state(), before);
    assert_eq!(receiver.uses_remaining(), TICKET_USES);
}

// The production wrapper takes the endpoint identity and project scope from the
// admitted endpoint binding, never from the sender, and admits a valid grant.
// r[verify mantle.authority_attenuation.attenuated_remote_grant]
#[test]
fn production_wrapper_uses_the_admitted_endpoint_binding_for_the_signed_request() {
    let mut receiver = Receiver::new();
    let builder = fixture_loopback_builder();
    let executor = fixture_local_build_executor();
    let config = endpoint_binding_tests::provision(receiver.dir.path(), &builder);
    let mut request = receiver.request.clone();
    request.endpoint_binding = Some(config.build_binding().unwrap().clone());
    let guard = config
        .admit_build(request.endpoint_binding.as_ref().unwrap(), now(), &builder.store_prefix)
        .unwrap();
    let (presentation, _) =
        receiver
            .grants
            .sign(&request, vec![], vec![identity_rewrite("00", "r1")], GrantFixture::current_window());
    let context = RemoteProductionServerContext {
        builder: &builder,
        verifier_key: &receiver.key,
        executor: &executor,
        secret_request: None,
        usage_cgroup_root: None,
        credential_state_dir: receiver.dir.path(),
        binding_config: Some(&config),
        execution_state_dir: receiver.dir.path(),
        authenticated_client_endpoint: None,
        attenuation_verifier: Some(&receiver.grants.verifier),
    };
    let mut auth = receiver.auth.clone();
    auth.attenuated_grant = Some(presentation);
    let mut effects = crunch_remote_core::effect::EffectSession::new(&request.request_id).unwrap();
    commit_remote_ticket_admission(&context, &auth, &request, &mut effects, &guard)
        .expect("grant signed for the admitted endpoint admits");
    assert_eq!(receiver.uses_remaining(), TICKET_USES - 1);
}
