//! Holder-attenuated remote grants at the real receiving boundary: real ticket
//! state on disk, the production admission functions, the operator-loaded
//! authority snapshot, and the receiver's own clock. Every rejected case
//! snapshots the ticket state and the replay-ledger directory BEFORE the
//! admission and asserts both are unchanged afterwards. A verified-then-failed
//! admission would consume a ledger nonce; no negative below reaches that point.
use std::collections::BTreeMap;
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
use crate::remote_build::attenuated_admission::DenialReason;
use crate::remote_build::attenuated_admission::admit_presented_grant;
use crate::remote_build::attenuated_admission::bind_remote_job;
use crate::remote_build::attenuated_admission::open_operator_authority;
use crate::remote_build::attenuated_admission::output_set_id;
use crate::remote_build::authority_caveats::EffectiveCaveatChain;

const SCOPE: &str = "fixture-builds";
const ACCOUNT: &str = "account-a";
const AUDIENCE: &str = "build-service";
const GENERATION: u64 = 7;
const ROOT_SEED: u8 = 11;
const DELEGATE_SEED: u8 = 13;
const HOLDER_SEED: u8 = 17;
const TICKET_USES: u32 = 3;
const ENDPOINT: &str = "builder-1";
const START_DENIED: &str = "remote-attenuation-authority-invalid";

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
    last_proof_reference: Vec<u8>,
    endpoint: String,
    next_nonce: u8,
    /// Issuer of the parent grant. Only `ROOT_SEED` is a trusted root.
    root_seed: u8,
    /// Holder key. Only `HOLDER_SEED` is provisioned in the snapshot keys.
    holder_seed: u8,
    /// Whether the parent proof is published in the receiver snapshot.
    publish_parent_proof: bool,
}

fn filter(key: &str, payload: &str) -> ucan::CaveatDocument {
    ucan::CaveatDocument::new("mantle".into(), "filter-v1".into(), key.into(), payload.as_bytes().to_vec()).unwrap()
}

/// The filter-visible target the receiver derives for `request`.
fn target_of(request: &ConcreteBuildRequest) -> String {
    bind_remote_job(request, ENDPOINT).unwrap().target
}

fn identity_rewrite(key: &str, target: &str) -> ucan::CaveatDocument {
    filter(key, &format!(r#"{{"kind":"rewrite","pattern":"{target}","template":"{target}"}}"#))
}

fn reject_target(key: &str, target: &str) -> ucan::CaveatDocument {
    filter(key, &format!(r#"{{"kind":"reject","pattern":"{target}"}}"#))
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

fn authority_document(endpoint: &str) -> serde_json::Value {
    let issuer = signer(ROOT_SEED);
    serde_json::json!({
        "version": 1, "revision": 1, "generation": GENERATION,
        "issued_unix_s": now() - 1000, "expires_unix_s": now() + 7200,
        "endpoint": endpoint, "audience": AUDIENCE, "policy_json": basalt::DEFAULT_POLICY_JSON,
        "policy_blake3": blake3::hash(basalt::DEFAULT_POLICY_JSON.as_bytes()).to_hex().to_string(),
        "trusted_roots": [issuer.issuer().unwrap().as_str()],
        "keys": [public_key(ROOT_SEED), public_key(DELEGATE_SEED), public_key(HOLDER_SEED)],
        "proofs": [], "revoked": []
    })
}

fn private_dir(path: &Path) {
    std::fs::create_dir(path).unwrap();
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o700)).unwrap();
}

impl GrantFixture {
    /// Provision the operator snapshot beside the ticket state and load it
    /// through the production loader, exactly as `remote serve` does.
    fn new(state: &Path, endpoint: &str) -> Self {
        let document = authority_document(endpoint);
        install(&state.join("attenuation-authority.json"), &document);
        private_dir(&state.join("attenuation-replay"));
        let verifier = open_operator_authority(state).unwrap().expect("provisioned authority loads");
        Self {
            state: state.to_path_buf(),
            verifier,
            document,
            proofs: Vec::new(),
            revoked: Vec::new(),
            last_proof_reference: Vec::new(),
            endpoint: endpoint.to_string(),
            next_nonce: 1,
            root_seed: ROOT_SEED,
            holder_seed: HOLDER_SEED,
            publish_parent_proof: true,
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
            subject: signer(self.holder_seed).issuer().unwrap().as_str().into(),
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
        let holder = signer(self.holder_seed);
        let audience = AudienceDid::from(holder.issuer().unwrap());
        let proof = ucan::issue_token_with_signer(
            &signer(self.root_seed),
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
        self.last_proof_reference = ucan::proof_reference(&proof).as_bytes().to_vec();
        if self.publish_parent_proof {
            self.proofs.push(proof.as_str().to_string());
        }
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

/// Everything a refused presentation must leave alone.
#[derive(Debug, PartialEq, Eq)]
struct Snapshot {
    tickets: RemoteTicketState,
    ledger: BTreeMap<String, Vec<u8>>,
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

    fn with_attempt(&self, job_id: &str) -> ConcreteBuildRequest {
        ConcreteBuildRequest {
            production_attempt: Some(RemoteProductionAttemptBinding {
                job_id: RemoteJobId::new(job_id.to_string()).unwrap(),
                attempt_id: RemoteAttemptId::new("attempt-1".to_string()).unwrap(),
                fence_generation: RemoteFenceGeneration::INITIAL,
            }),
            ..self.request.clone()
        }
    }

    fn state(&self) -> RemoteTicketState {
        load_ticket_state(self.dir.path()).unwrap()
    }

    fn uses_remaining(&self) -> u32 {
        self.state().tickets[TRANSACTION_TICKET_ID].uses_remaining
    }

    fn ledger(&self) -> BTreeMap<String, Vec<u8>> {
        let mut files = BTreeMap::new();
        for entry in std::fs::read_dir(self.dir.path().join("attenuation-replay")).unwrap() {
            let entry = entry.unwrap();
            files.insert(entry.file_name().to_string_lossy().into_owned(), std::fs::read(entry.path()).unwrap());
        }
        files
    }

    fn snapshot(&self) -> Snapshot {
        Snapshot {
            tickets: self.state(),
            ledger: self.ledger(),
        }
    }

    /// Run the production ticket admission with the receiver-owned facts.
    fn admit_at(
        &self,
        request: &ConcreteBuildRequest,
        presentation: Option<AttenuatedGrantPresentation>,
        verifier: Option<&GatewayVerifier>,
        endpoint_id: &str,
    ) -> Result<CommittedRemoteTicketAdmission, String> {
        let mut auth = self.auth.clone();
        auth.attenuated_grant = presentation;
        let attenuation = AttenuationAdmission {
            verifier,
            endpoint_id,
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

    fn admit(
        &self,
        request: &ConcreteBuildRequest,
        presentation: Option<AttenuatedGrantPresentation>,
        verifier: Option<&GatewayVerifier>,
    ) -> Result<CommittedRemoteTicketAdmission, String> {
        self.admit_at(request, presentation, verifier, &self.grants.endpoint)
    }

    fn admit_configured(
        &self,
        request: &ConcreteBuildRequest,
        presentation: AttenuatedGrantPresentation,
    ) -> Result<CommittedRemoteTicketAdmission, String> {
        self.admit(request, Some(presentation), Some(&self.grants.verifier))
    }

    /// Direct verifier call (no request validation), for requests that the
    /// ticket path would reject for unrelated reasons before attenuation runs.
    fn verify_directly(
        &self,
        request: &ConcreteBuildRequest,
        presentation: &AttenuatedGrantPresentation,
    ) -> Result<EffectiveCaveatChain, DenialReason> {
        self.verify_directly_at(request, presentation, &self.grants.endpoint)
    }

    fn verify_directly_at(
        &self,
        request: &ConcreteBuildRequest,
        presentation: &AttenuatedGrantPresentation,
        endpoint_id: &str,
    ) -> Result<EffectiveCaveatChain, DenialReason> {
        let attenuation = AttenuationAdmission {
            verifier: Some(&self.grants.verifier),
            endpoint_id,
            project_scope: SCOPE,
        };
        admit_presented_grant(&attenuation, presentation, request, now())
    }

    /// `before` MUST be captured before the admission ran. A refused request
    /// returns only the constant string, redeems nothing and consumes no nonce.
    fn assert_refused_without_effect(
        &self,
        result: Result<CommittedRemoteTicketAdmission, String>,
        before: &Snapshot,
        case: &str,
    ) {
        assert_eq!(result.err().as_deref(), Some(ATTENUATED_GRANT_DENIED), "{case}");
        assert_eq!(&self.snapshot(), before, "{case}: ticket state or replay ledger changed by a refused request");
        assert_eq!(self.uses_remaining(), TICKET_USES, "{case}: a ticket use was redeemed");
    }
}

// A restricted grant authorizes its declared job: the ticket is redeemed once,
// with the receiver's own clock, exactly one replay record is added, and legacy
// bearer admission is unchanged.
// r[verify mantle.authority_attenuation.attenuated_remote_grant]
#[test]
fn restricted_grant_authorizes_its_declared_job_and_legacy_bearer_is_unchanged() {
    let mut receiver = Receiver::new();
    let request = receiver.request.clone();
    let rewrite = identity_rewrite("00", &target_of(&request));
    let (presentation, _) = receiver.grants.sign(&request, vec![], vec![rewrite], GrantFixture::current_window());
    let ledger_before = receiver.ledger();
    let admitted = receiver.admit_configured(&request, presentation).expect("declared job is admitted");
    assert!(admitted.server_now_unix_s >= now() - 5);
    assert_eq!(receiver.uses_remaining(), TICKET_USES - 1);
    assert_ne!(receiver.ledger(), ledger_before, "an admitted holder invocation must be recorded");

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
    let (first_target, second_target) = (target_of(&first), target_of(&second));
    let parent = vec![filter(
        "00",
        &format!(
            r#"{{"kind":"alternatives","alternatives":[{{"pattern":"{first_target}","template":"{first_target}"}},{{"pattern":"{second_target}","template":"{second_target}"}}]}}"#
        ),
    )];
    let child = vec![parent[0].clone(), reject_target("01", &second_target)];
    let (for_first, _) = receiver.grants.sign(&first, parent.clone(), child.clone(), GrantFixture::current_window());
    let (for_second, _) = receiver.grants.sign(&second, parent, child, GrantFixture::current_window());

    let before = receiver.snapshot();
    let refused = receiver.admit_configured(&second, for_second);
    receiver.assert_refused_without_effect(refused, &before, "second job");
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
    let parent = vec![identity_rewrite("00", &target_of(&request))];
    let child = vec![
        parent[0].clone(),
        reject_target("01", "/job/r9/class/action/outputs/out"),
    ];
    let (presentation, token) = receiver.grants.sign(&request, parent, child, GrantFixture::current_window());
    let chain = receiver.verify_directly(&request, &presentation).expect("grant admits");
    assert_eq!(chain.delegation, "holder-attenuated-ucan");
    assert_eq!(chain.status, "bounded-filter-chain");
    assert_eq!(chain.caveat_count, 2);
    let described: Vec<_> = chain.caveats.iter().map(|entry| (entry.ordinal.as_str(), entry.kind)).collect();
    assert_eq!(described, [("00", "rewrite"), ("01", "reject")]);
    assert_eq!(chain.caveats[1].pattern.as_deref(), Some("/job/r9/class/action/outputs/out"));
    let report = serde_json::to_string(&chain).unwrap();
    assert!(!report.contains(token.as_str()));
    assert!(!report.contains(&data_encoding::HEXLOWER.encode(&presentation.holder_signature)));
    assert_eq!(chain.chain_blake3.len(), 64);

    // A UCAN whose capability carries no caveats restricts nothing and is not
    // described as attenuated.
    let unrestricted = EffectiveCaveatChain::from_documents(&[]);
    assert_eq!(unrestricted.delegation, "ucan-unrestricted");
    assert_eq!(unrestricted.status, "no-caveats");

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

/// Mutations applied to a validly signed presentation or to the receiver's
/// facts. Each one MUST be refused before redemption and before nonce use.
fn refusal_cases() -> Vec<&'static str> {
    vec![
        "unknown-caveat",
        "malformed-caveat",
        "ordinal-gap",
        "chain-over-bound",
        "parent-rejected-input-rewritten",
        "expired-deadline",
        "not-yet-valid",
        "revoked-grant",
        "revoked-parent-proof",
        "wrong-project-scope",
        "wrong-endpoint-at-receiver",
        "wrong-audience",
        "wrong-generation",
        "wrong-holder-identity",
        "flipped-signature",
        "signature-of-another-invocation",
        "other-job-signature",
        "changed-request-field",
        "oversized-token",
        "parent-caveat-dropped",
        "parent-caveat-reordered",
        "parent-caveat-modified",
        "untrusted-root-issuer",
        "unknown-holder-key",
        "parent-proof-absent-from-snapshot",
        "tampered-token-body",
        "unregistered-caveat-domain",
        "unregistered-caveat-type",
        "duplicate-ordinal",
    ]
}

fn case_window(case: &str) -> Window {
    match case {
        "expired-deadline" => Window {
            not_before: now() - 500,
            expires: now() - 60,
        },
        "not-yet-valid" => Window {
            not_before: now() + 300,
            expires: now() + 1800,
        },
        _ => GrantFixture::current_window(),
    }
}

fn raw_caveat(domain: &str, caveat_type: &str, key: &str, target: &str) -> ucan::CaveatDocument {
    let payload = format!(r#"{{"kind":"rewrite","pattern":"{target}","template":"{target}"}}"#);
    ucan::CaveatDocument::new(domain.into(), caveat_type.into(), key.into(), payload.into_bytes()).unwrap()
}

fn case_chains(case: &str, target: &str) -> (Vec<ucan::CaveatDocument>, Vec<ucan::CaveatDocument>) {
    let allow = identity_rewrite("00", target);
    let other = reject_target("01", "/job/r9/class/action/outputs/out");
    match case {
        "unknown-caveat" => (vec![], vec![filter("00", r#"{"kind":"future"}"#)]),
        "unregistered-caveat-domain" => (vec![], vec![raw_caveat("other", "filter-v1", "00", target)]),
        "unregistered-caveat-type" => (vec![], vec![raw_caveat("mantle", "filter-v2", "00", target)]),
        "duplicate-ordinal" => (vec![], vec![allow.clone(), allow.clone()]),
        "malformed-caveat" => (vec![], vec![filter("00", &format!(r#"{{"kind":"rewrite","pattern":"{target}"}}"#))]),
        "ordinal-gap" => (vec![], vec![identity_rewrite("01", target)]),
        "chain-over-bound" => {
            let chain: Vec<_> = (0..9).map(|index| identity_rewrite(&format!("{index:02}"), target)).collect();
            (chain.clone(), chain)
        }
        "parent-rejected-input-rewritten" => {
            let parent = reject_target("00", target);
            (vec![parent.clone()], vec![parent, identity_rewrite("01", target)])
        }
        "parent-caveat-dropped" => (vec![allow.clone(), other], vec![allow]),
        "parent-caveat-reordered" => {
            let swapped_other = reject_target("00", "/job/r9/class/action/outputs/out");
            let swapped_allow = identity_rewrite("01", target);
            (vec![allow, other], vec![swapped_other, swapped_allow])
        }
        "parent-caveat-modified" => {
            let weakened = reject_target("01", "/job/r8/class/action/outputs/out");
            (vec![allow.clone(), other], vec![allow, weakened])
        }
        _ => (vec![], vec![allow]),
    }
}

// An unknown caveat rejects everything; so do malformed, ordinal-gapped,
// over-bound, parent-widening, dropped/reordered/modified parent caveats,
// not-yet-valid, expired, revoked (grant and parent proof), wrong endpoint,
// audience, generation, holder identity and signature, replayed and re-targeted
// presentations. The ticket state AND the replay-ledger directory are captured
// BEFORE each admission and must be identical afterwards: nothing is redeemed
// and no holder nonce is consumed. The sender sees only the constant refusal.
// r[verify mantle.authority_attenuation.caveat_filter_semantics]
#[test]
fn unsafe_presentations_are_refused_before_redemption_and_nonce_use_with_no_feedback() {
    for case in refusal_cases() {
        let mut receiver = Receiver::new();
        let request = receiver.request.clone();
        let signed_for = if case == "other-job-signature" {
            receiver.other_job("r2")
        } else {
            request.clone()
        };
        let (parent, child) = case_chains(case, &target_of(&signed_for));
        match case {
            "untrusted-root-issuer" => {
                receiver.grants.root_seed = 99;
                receiver.grants.document["keys"].as_array_mut().unwrap().push(serde_json::json!(public_key(99)));
            }
            "unknown-holder-key" => receiver.grants.holder_seed = 21,
            "parent-proof-absent-from-snapshot" => receiver.grants.publish_parent_proof = false,
            _ => {}
        }
        let (mut presentation, token) = receiver.grants.sign(&signed_for, parent, child, case_window(case));
        let mut endpoint = receiver.grants.endpoint.clone();
        match case {
            "revoked-grant" => {
                receiver.grants.revoked.push(ucan::proof_reference(&token).as_bytes().to_vec());
                receiver.grants.publish();
            }
            "revoked-parent-proof" => {
                let parent_reference = receiver.grants.last_proof_reference.clone();
                receiver.grants.revoked.push(parent_reference);
                receiver.grants.publish();
            }
            "wrong-project-scope" => presentation.project = "another-project".into(),
            "wrong-endpoint-at-receiver" => endpoint = "another-builder".into(),
            "wrong-audience" => presentation.audience = "another-service".into(),
            "wrong-generation" => presentation.generation += 1,
            "wrong-holder-identity" => presentation.subject = signer(19).issuer().unwrap().as_str().into(),
            "flipped-signature" => presentation.holder_signature[0] ^= 1,
            "signature-of-another-invocation" => {
                let (second, _) = receiver.grants.sign(
                    &signed_for,
                    vec![],
                    vec![identity_rewrite("00", &target_of(&signed_for))],
                    GrantFixture::current_window(),
                );
                presentation.holder_signature = second.holder_signature.clone();
            }
            "oversized-token" => presentation.compact_token.push_str(&"A".repeat(40 * 1024)),
            "tampered-token-body" => {
                let middle = presentation.compact_token.len() / 2;
                let original = presentation.compact_token.as_bytes()[middle];
                let replacement = if original == b'A' { "B" } else { "A" };
                presentation.compact_token.replace_range(middle..=middle, replacement);
            }
            _ => {}
        }
        let mut presented = request.clone();
        if case == "changed-request-field" {
            presented.build_time_limit_secs = presented.build_time_limit_secs.saturating_sub(1);
        }
        // Attribution: each case is refused by the intended check, not merely
        // refused. Everything the verifier rejects reports `Verification`
        // (the UCAN/Basalt verifier is one undifferentiated denial); the two
        // receiver-side pre-checks have their own reasons.
        let expected = match case {
            "oversized-token" => DenialReason::PresentationMalformed,
            "wrong-project-scope" => DenialReason::ProjectScopeMismatch,
            _ => DenialReason::Verification,
        };
        let before = receiver.snapshot();
        assert_eq!(receiver.verify_directly_at(&presented, &presentation, &endpoint).err(), Some(expected), "{case}");
        let result = receiver.admit_at(&presented, Some(presentation), Some(&receiver.grants.verifier), &endpoint);
        receiver.assert_refused_without_effect(result, &before, case);
    }
}

// Replaying an admitted holder invocation, presenting with no operator
// authority, and trusting a forged client clock all fail closed.
// r[verify mantle.authority_attenuation.attenuated_remote_grant]
#[test]
fn replay_missing_authority_and_forged_client_time_fail_closed() {
    let mut receiver = Receiver::new();
    let request = receiver.request.clone();
    let rewrite = identity_rewrite("00", &target_of(&request));
    let (presentation, _) =
        receiver.grants.sign(&request, vec![], vec![rewrite.clone()], GrantFixture::current_window());
    receiver.admit_configured(&request, presentation.clone()).expect("first presentation admits");
    assert_eq!(receiver.uses_remaining(), TICKET_USES - 1);
    let before = receiver.snapshot();
    let replayed = receiver.admit_configured(&request, presentation.clone());
    assert_eq!(replayed.err().as_deref(), Some(ATTENUATED_GRANT_DENIED));
    assert_eq!(receiver.snapshot(), before, "a replayed holder invocation redeemed a use or changed the ledger");

    // No operator authority configured: a presented grant is never trusted.
    let (second, _) = receiver.grants.sign(&request, vec![], vec![rewrite], GrantFixture::current_window());
    let before = receiver.snapshot();
    let unconfigured = receiver.admit(&request, Some(second.clone()), None);
    assert_eq!(unconfigured.err().as_deref(), Some(ATTENUATED_GRANT_DENIED));
    assert_eq!(receiver.snapshot(), before);

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
    let target = target_of(&request);
    let chain: Vec<_> = (0..crunch_remote_core::caveat::MAX_CHAIN)
        .map(|index| identity_rewrite(&format!("{index:02}"), &target))
        .collect();
    let (presentation, _) = receiver.grants.sign(&request, chain.clone(), chain, GrantFixture::current_window());
    receiver.admit_configured(&request, presentation).expect("a chain at the bound admits");
    assert_eq!(receiver.uses_remaining(), TICKET_USES - 1);
}

fn attempt_request(receiver: &Receiver, job_id: &str, outputs: &[&str]) -> ConcreteBuildRequest {
    let mut request = receiver.with_attempt(job_id);
    request.expected_outputs = outputs
        .iter()
        .map(|name| RemoteExpectedOutput {
            name: (*name).to_string(),
            logical_path: None,
        })
        .collect();
    request
}

// The caveat-visible job identity carries the production attempt job id and the
// declared output names: a grant for one job id and output set refuses another
// job id, no attempt, and any different output set, and refuses without
// consuming the holder nonce. (Direct verifier calls: the ticket path would
// reject some of these requests for unrelated validation reasons first.)
// r[verify mantle.authority_attenuation.attenuated_remote_grant]
#[test]
fn grant_for_one_job_id_and_output_set_refuses_another() {
    let mut receiver = Receiver::new();
    let granted = attempt_request(&receiver, "job-a", &["out", "dev"]);
    let rewrite = identity_rewrite("00", &target_of(&granted));
    let (presentation, _) = receiver.grants.sign(&granted, vec![], vec![rewrite], GrantFixture::current_window());
    let before = receiver.ledger();
    for (case, other) in [
        ("other-job-id", attempt_request(&receiver, "job-b", &["out", "dev"])),
        ("no-attempt", ConcreteBuildRequest {
            production_attempt: None,
            ..granted.clone()
        }),
        ("fewer-outputs", attempt_request(&receiver, "job-a", &["out"])),
        ("more-outputs", attempt_request(&receiver, "job-a", &["out", "dev", "doc"])),
        ("renamed-output", attempt_request(&receiver, "job-a", &["out", "bin"])),
        ("other-request-id", ConcreteBuildRequest {
            request_id: "r2".into(),
            ..granted.clone()
        }),
    ] {
        assert_eq!(receiver.verify_directly(&other, &presentation).err(), Some(DenialReason::Verification), "{case}");
        assert_eq!(receiver.ledger(), before, "{case}: nonce consumed by a refused request");
    }
    // Output order is not part of the filter-visible identity, but the digest
    // still binds the declared order: the original signature does not transfer.
    let reordered = attempt_request(&receiver, "job-a", &["dev", "out"]);
    assert_eq!(target_of(&reordered), target_of(&granted));
    assert_eq!(receiver.verify_directly(&reordered, &presentation).err(), Some(DenialReason::Verification));
    assert_eq!(receiver.ledger(), before);
}

// A holder who signs the exact request is admitted at the matching job id.
// r[verify mantle.authority_attenuation.attenuated_remote_grant]
#[test]
fn grant_for_a_production_job_id_admits_exactly_that_request() {
    let mut receiver = Receiver::new();
    let granted = attempt_request(&receiver, "job-a", &["out"]);
    let rewrite = identity_rewrite("00", &target_of(&granted));
    let (presentation, _) = receiver.grants.sign(&granted, vec![], vec![rewrite], GrantFixture::current_window());
    let chain = receiver.verify_directly(&granted, &presentation).expect("exact request admits");
    assert_eq!(chain.caveat_count, 1);
}

// Target shape and rejection rules. Unrepresentable identities are rejected,
// never truncated or escaped, and every produced target is a valid filter value
// of a FIXED segment count: only the request id, job id and kind vary in
// content, the output set is one 64-hex segment whatever its size.
// r[verify mantle.authority_attenuation.attenuated_remote_grant]
#[test]
fn job_target_is_canonical_fixed_shape_bounded_and_rejects_what_filters_cannot_express() {
    let receiver = Receiver::new();
    let plain = target_of(&receiver.request);
    let set_out = output_set_id(["out"]).unwrap();
    let set_both = output_set_id(["dev", "out"]).unwrap();
    assert_eq!(plain, format!("/job/r1/class/action/outputs/{set_out}"));
    let attempt = target_of(&attempt_request(&receiver, "job-a", &["out", "dev"]));
    assert_eq!(attempt, format!("/job/r1/attempt/job-a/class/action/outputs/{set_both}"));
    for target in [&plain, &attempt] {
        assert!(crunch_remote_core::caveat::Pattern::parse(target).is_ok(), "{target}");
    }
    let derivation = ConcreteBuildRequest {
        payload: RemoteConcreteBuildPayload::Derivation {
            drv_path: "x.drv".into(),
            drv_json: "{}".into(),
        },
        ..receiver.request.clone()
    };
    assert_eq!(target_of(&derivation), format!("/job/r1/class/derivation/outputs/{set_out}"));

    let bind = |request: &ConcreteBuildRequest| bind_remote_job(request, ENDPOINT);
    let names = |names: &[String]| {
        let refs: Vec<&str> = names.iter().map(String::as_str).collect();
        attempt_request(&receiver, "job-a", &refs)
    };
    // Segment count does not depend on the number of outputs; there is no cap
    // below the production limit of 128, and sets of any size stay one segment.
    let segments = |request: &ConcreteBuildRequest| target_of(request).split('/').count();
    let baseline = segments(&names(&["o0".into()]));
    for count in [1, 2, 9, 10, 20, 128] {
        let set: Vec<String> = (0..count).map(|index| format!("o{index}")).collect();
        assert_eq!(segments(&names(&set)), baseline, "{count} outputs");
        assert!(target_of(&names(&set)).len() <= crunch_remote_core::caveat::MAX_VALUE_BYTES);
    }
    assert!(bind(&names(&["out".into(), "out".into()])).is_err());
    assert!(bind(&names(&[])).is_err());
    // Output names need not be filter vocabulary: they are hashed, not embedded.
    assert!(bind(&names(&["out put".into(), "\u{e9}".into()])).is_ok());
    // Request and job ids ARE embedded: characters outside the filter
    // vocabulary, dot segments and over-long values are rejected.
    for bad_id in ["r 1", "r/1", ".", "..", ":bind", "r\u{e9}", ""] {
        assert!(bind(&receiver.other_job(bad_id)).is_err(), "request id {bad_id:?}");
    }
    assert!(bind(&receiver.with_attempt("job/x")).is_err());
    assert!(bind(&receiver.other_job(&"j".repeat(600))).is_err());
    assert!(bind(&receiver.with_attempt(&"j".repeat(128))).is_ok());
    assert!(bind_remote_job(&receiver.request, "").is_err());
}

// Golden vectors an external signer can reproduce. The set id is BLAKE3 over the
// domain `mantle/remote/attenuated-output-set/v1\0` and each SORTED name framed
// as an 8-byte big-endian length plus the bytes (computed independently with
// `b3sum`); order of declaration does not matter.
// r[verify mantle.authority_attenuation.attenuated_remote_grant]
#[test]
fn output_set_id_matches_the_golden_vectors_and_ignores_declaration_order() {
    assert_eq!(output_set_id(["out"]).unwrap(), "5aadee160983d4cbda395f623dee917c0ed67af797e6142d522209d267fbd248");
    assert_eq!(
        output_set_id(["dev", "out"]).unwrap(),
        "37da46b8c8c74bf492cf7db06f9d45832615605e609bcea258ed70e5c9e22d08"
    );
    assert_eq!(output_set_id(["out", "dev"]), output_set_id(["dev", "out"]));
    assert_ne!(output_set_id(["out"]), output_set_id(["out", "dev"]));
    assert_ne!(output_set_id(["a", "bc"]), output_set_id(["ab", "c"]), "framing must separate names");
    assert_eq!(output_set_id([]), None);
    assert_eq!(output_set_id(["out", "out"]), None);
}

// A reject on a job id is COMPLETE: the same reject pattern matches the target
// whether 1, 2 or 20 outputs are declared, so padding the output set cannot
// evade it. A non-matching job id is untouched by the same reject.
// r[verify mantle.authority_attenuation.composition_order]
#[test]
fn reject_on_a_job_id_refuses_regardless_of_declared_output_count() {
    let mut receiver = Receiver::new();
    let reject = reject_target("01", "/job/secret/attempt/job-a/class/action/outputs/:set");
    let allow_any = filter(
        "00",
        "{\"kind\":\"alternatives\",\"alternatives\":[{\"pattern\":\"/job/:r/attempt/:j/class/action/outputs/:s\",\"template\":\"/job/:r/attempt/:j/class/action/outputs/:s\"}]}",
    );
    for count in [1_usize, 2, 20] {
        let set: Vec<String> = (0..count).map(|index| format!("o{index}")).collect();
        let refs: Vec<&str> = set.iter().map(String::as_str).collect();
        let mut denied = attempt_request(&receiver, "job-a", &refs);
        denied.request_id = "secret".into();
        let (presentation, _) = receiver.grants.sign(
            &denied,
            vec![allow_any.clone()],
            vec![allow_any.clone(), reject.clone()],
            GrantFixture::current_window(),
        );
        let ledger = receiver.ledger();
        assert_eq!(
            receiver.verify_directly(&denied, &presentation).err(),
            Some(DenialReason::Verification),
            "{count} outputs"
        );
        assert_eq!(receiver.ledger(), ledger, "{count} outputs: nonce consumed");
        let mut allowed = attempt_request(&receiver, "job-a", &refs);
        allowed.request_id = "other".into();
        let (presentation, _) = receiver.grants.sign(
            &allowed,
            vec![allow_any.clone()],
            vec![allow_any.clone(), reject.clone()],
            GrantFixture::current_window(),
        );
        receiver
            .verify_directly(&allowed, &presentation)
            .expect("a different job id passes the same reject");
    }
}

// The digest covers the WHOLE validated request: every field that the first
// version left unbound, each changed after signing, is refused and consumes no
// nonce. The control admits the untouched request.
// r[verify mantle.authority_attenuation.attenuated_remote_grant]
#[test]
fn every_request_field_changed_after_signing_is_refused() {
    type Mutation = (&'static str, fn(&mut ConcreteBuildRequest));
    let mutations: Vec<Mutation> = vec![
        ("contains_raw_frontend_eval", |r| r.contains_raw_frontend_eval = true),
        ("attempt_id", |r| {
            r.production_attempt.as_mut().unwrap().attempt_id = RemoteAttemptId::new("attempt-2".to_string()).unwrap()
        }),
        ("fence_generation", |r| {
            r.production_attempt.as_mut().unwrap().fence_generation = RemoteFenceGeneration::INITIAL.advance().unwrap()
        }),
        ("transfer_policy", |r| r.transfer_policy = Some(RemoteTransferPolicy::default())),
        ("resource_requirements", |r| {
            r.resource_requirements = Some(crunch_build::distributed::RemoteResourceRequirements {
                quantities: crunch_build::distributed::RemoteResourceVector {
                    cpu_units: 1,
                    memory_bytes: 1,
                    scratch_bytes: 1,
                    accelerators: Vec::new(),
                    named_tokens: Vec::new(),
                },
                semantic_accelerator_classes: Vec::new(),
            })
        }),
        ("locality_scope", |r| {
            r.locality_scope = Some(crunch_build::distributed::RemoteLocalityScope {
                manifest_digest_blake3: "a".repeat(64),
                policy_digest_blake3: "b".repeat(64),
            })
        }),
        ("failure_debug_policy", |r| r.failure_debug_policy.replay_enabled = true),
        ("failure_replay", |r| {
            let digest =
                |c: char| crunch_build::distributed::RemoteFailureDebugDigest::new(c.to_string().repeat(64)).unwrap();
            r.failure_replay = Some(RemoteFailureReplayBinding {
                source_bundle_blake3: digest('c'),
                execution_blake3: digest('d'),
            })
        }),
        ("endpoint_binding_generation", |r| r.endpoint_binding.as_mut().unwrap().generation += 1),
        ("endpoint_binding_scope", |r| r.endpoint_binding.as_mut().unwrap().spec.scope = "other-scope".into()),
        ("upload_bytes", |r| r.upload_bytes += 1),
        ("input_refs", |r| r.input_refs.push("input-b".into())),
        ("store_prefix", |r| r.store_prefix = "/other/store".into()),
        ("expected_output_path", |r| r.expected_outputs[0].logical_path = Some("/mantle/store/other".into())),
    ];
    let mut receiver = Receiver::new();
    let mut signed = attempt_request(&receiver, "job-a", &["out"]);
    signed.endpoint_binding = Some(endpoint_binding_tests::descriptor(ENDPOINT, "/mantle/store"));
    let rewrite = identity_rewrite("00", &target_of(&signed));
    let mut presentations = Vec::new();
    for _ in 0..=mutations.len() {
        let (presentation, _) =
            receiver.grants.sign(&signed, vec![], vec![rewrite.clone()], GrantFixture::current_window());
        presentations.push(presentation);
    }
    let ledger_before = receiver.ledger();
    for ((field, mutate), presentation) in mutations.iter().zip(presentations.iter()) {
        let mut changed = signed.clone();
        mutate(&mut changed);
        assert_ne!(changed, signed, "{field}: mutation changed nothing");
        assert_eq!(receiver.verify_directly(&changed, presentation).err(), Some(DenialReason::Verification), "{field}");
        assert_eq!(receiver.ledger(), ledger_before, "{field}: nonce consumed by a refused request");
    }
    let control = presentations.last().unwrap();
    receiver.verify_directly(&signed, control).expect("the untouched request admits");
}

// The request encoding is deterministic: equal requests bind identically across
// independent clones, and the endpoint id is part of the digest.
// r[verify mantle.authority_attenuation.attenuated_remote_grant]
#[test]
fn request_digest_is_deterministic_and_endpoint_bound() {
    let receiver = Receiver::new();
    let request = attempt_request(&receiver, "job-a", &["out"]);
    let first = bind_remote_job(&request, ENDPOINT).unwrap();
    assert_eq!(first, bind_remote_job(&request.clone(), ENDPOINT).unwrap());
    let round_tripped: ConcreteBuildRequest = serde_json::from_slice(&serde_json::to_vec(&request).unwrap()).unwrap();
    assert_eq!(first, bind_remote_job(&round_tripped, ENDPOINT).unwrap());
    assert_ne!(first.request_digest, bind_remote_job(&request, "other-endpoint").unwrap().request_digest);
    assert_eq!(first.target, bind_remote_job(&request, "other-endpoint").unwrap().target);
}

fn provisioned_start(mutate: impl FnOnce(&Path, &mut serde_json::Value)) -> Result<bool, String> {
    let dir = tempfile::tempdir().unwrap();
    let mut document = authority_document(ENDPOINT);
    private_dir(&dir.path().join("attenuation-replay"));
    mutate(dir.path(), &mut document);
    let file = dir.path().join("attenuation-authority.json");
    if std::fs::symlink_metadata(&file).is_err() {
        install(&file, &document);
    }
    match open_operator_authority(dir.path()) {
        Ok(verifier) => Ok(verifier.is_some()),
        Err(crate::errors::RunError::Internal(message)) => Err(message),
        Err(other) => Err(other.to_string()),
    }
}

// A provisioned authority must really fail the server start when unusable: not
// only unreadable or non-private, but malformed, expired, not yet issued, with a
// wrong policy hash, symlinked, oversized, or with a bad replay directory. An
// absent snapshot is the unprovisioned case and starts without attenuation.
// r[verify mantle.authority_attenuation.attenuated_remote_grant]
#[test]
fn provisioned_but_invalid_authority_fails_the_server_start() {
    assert_eq!(provisioned_start(|_, _| {}), Ok(true));
    let dir = tempfile::tempdir().unwrap();
    assert!(open_operator_authority(dir.path()).unwrap().is_none(), "unprovisioned starts without attenuation");

    type StartMutation = Box<dyn FnOnce(&Path, &mut serde_json::Value)>;
    let failing: Vec<(&str, StartMutation)> = vec![
        (
            "malformed",
            Box::new(|dir, _| {
                std::fs::write(dir.join("attenuation-authority.json"), b"{not json").unwrap();
                std::fs::set_permissions(
                    dir.join("attenuation-authority.json"),
                    std::fs::Permissions::from_mode(0o600),
                )
                .unwrap();
            }),
        ),
        ("expired", Box::new(|_, doc| doc["expires_unix_s"] = serde_json::json!(now() - 10))),
        ("not-yet-issued", Box::new(|_, doc| doc["issued_unix_s"] = serde_json::json!(now() + 600))),
        ("wrong-policy-hash", Box::new(|_, doc| doc["policy_blake3"] = serde_json::json!("0".repeat(64)))),
        ("unsupported-version", Box::new(|_, doc| doc["version"] = serde_json::json!(2))),
        ("no-trusted-roots", Box::new(|_, doc| doc["trusted_roots"] = serde_json::json!([]))),
        ("no-keys", Box::new(|_, doc| doc["keys"] = serde_json::json!([]))),
        (
            "mode-0644",
            Box::new(|dir, doc| {
                let file = dir.join("attenuation-authority.json");
                install(&file, doc);
                std::fs::set_permissions(&file, std::fs::Permissions::from_mode(0o644)).unwrap();
            }),
        ),
        (
            "symlink",
            Box::new(|dir, doc| {
                let real = dir.join("real-authority.json");
                install(&real, doc);
                std::os::unix::fs::symlink(&real, dir.join("attenuation-authority.json")).unwrap();
            }),
        ),
        (
            "oversized",
            Box::new(|dir, doc| {
                let mut bytes = serde_json::to_vec(doc).unwrap();
                bytes.extend(std::iter::repeat_n(b' ', 1_048_576 + 1));
                let file = dir.join("attenuation-authority.json");
                std::fs::write(&file, bytes).unwrap();
                std::fs::set_permissions(&file, std::fs::Permissions::from_mode(0o600)).unwrap();
            }),
        ),
        (
            "replay-directory-not-private",
            Box::new(|dir, _| {
                std::fs::set_permissions(dir.join("attenuation-replay"), std::fs::Permissions::from_mode(0o755))
                    .unwrap();
            }),
        ),
        (
            "replay-directory-missing",
            Box::new(|dir, _| {
                std::fs::remove_dir(dir.join("attenuation-replay")).unwrap();
            }),
        ),
    ];
    for (case, mutate) in failing {
        assert_eq!(provisioned_start(mutate), Err(START_DENIED.to_string()), "{case}");
    }
}

// A presented grant is never silently ignored by an admission path that cannot
// enforce it: frame-level admission (the fixture executor path) and the loopback
// session refuse it with the constant string and redeem nothing.
// r[verify mantle.authority_attenuation.attenuated_remote_grant]
#[test]
fn presented_grant_is_refused_on_every_non_enforcing_path() {
    let junk = || AttenuatedGrantPresentation {
        compact_token: "not-a-real-token".into(),
        nonce: [1; 32],
        holder_signature: vec![2; 64],
        subject: "s".into(),
        account: "a".into(),
        project: SCOPE.into(),
        audience: AUDIENCE.into(),
        generation: GENERATION,
    };
    let builder = fixture_loopback_builder();

    // Loopback: the control admits, the presentation is refused untouched.
    let mut ticket = fixture_ticket();
    let control = fixture_loopback_client(vec!["input-a".to_string()], vec!["builder-key".to_string()]);
    run_loopback_remote_session(&builder, &mut ticket.clone(), &control).expect("control loopback session admits");
    let mut presented = fixture_loopback_client(vec!["input-a".to_string()], vec!["builder-key".to_string()]);
    presented.auth.attenuated_grant = Some(junk());
    let before = ticket.clone();
    let refused = run_loopback_remote_session(&builder, &mut ticket, &presented).unwrap_err();
    assert_eq!(refused, ATTENUATED_GRANT_DENIED);
    assert_eq!(ticket, before);

    // Frame-level admission (fixture executor path): the complete client frame
    // sequence. The control admits and redeems the REAL ticket; with a
    // presentation the same frames are refused with the constant string and the
    // real ticket is untouched.
    let client = fixture_loopback_client(vec!["input-a".to_string()], vec!["builder-key".to_string()]);
    let frames = |auth: TicketAuthRequest| {
        vec![
            RemoteFrame::Hello {
                hello: client.hello.clone(),
            },
            RemoteFrame::AuthTicket { auth },
            RemoteFrame::BuildRequest {
                request: client.request.clone(),
            },
            RemoteFrame::InputManifest {
                manifest: client.input_manifest.clone(),
            },
            RemoteFrame::InputUpload {
                upload: RemoteInputUpload {
                    request_id: client.request.request_id.clone(),
                    refs: client.request.input_refs.clone(),
                    byte_count: client.request.upload_bytes,
                    artifacts: Vec::new(),
                    streamed: false,
                },
            },
        ]
    };
    let admit = |ticket: &mut RemoteTicket, frames: &[RemoteFrame]| {
        let endpoint = ticket.bound_client_endpoint.clone();
        let facts = RemoteTicketAuthFacts {
            server_now_unix_s: ticket.created_unix_s.saturating_add(1),
            authenticated_client_endpoint: endpoint.as_deref(),
        };
        admit_remote_builder_frames_with_auth_facts(
            &builder,
            ticket,
            frames,
            RemoteTransferCapabilities::delta_and_full(),
            facts,
        )
        .map(|admission| admission.ticket_uses_remaining)
    };
    let mut control_ticket = fixture_ticket();
    assert_eq!(admit(&mut control_ticket, &frames(client.auth.clone())), Ok(0), "control frames must admit");
    assert_eq!(control_ticket.uses_remaining, 0, "control redeems the real ticket");
    let mut ticket = fixture_ticket();
    let mut auth = client.auth.clone();
    auth.attenuated_grant = Some(junk());
    assert_eq!(admit(&mut ticket, &frames(auth)), Err(ATTENUATED_GRANT_DENIED.to_string()));
    assert_eq!(ticket, fixture_ticket(), "a refused presentation must leave the real ticket untouched");
}

struct ServeFixture {
    reader: Cursor<Vec<u8>>,
    builder: RemoteLoopbackBuilder,
    executor: RemoteLocalBuildExecutor,
    config: endpoint_config::RemoteEndpointBindingConfig,
}

fn serve_fixture(receiver: &mut Receiver, caveats: Vec<ucan::CaveatDocument>, allow_target: bool) -> ServeFixture {
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
    let child = if allow_target {
        vec![identity_rewrite("00", &target_of(&client.request))]
    } else {
        caveats
    };
    let (presentation, _) =
        receiver.grants.sign(&client.request.clone(), vec![], child, GrantFixture::current_window());
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
    ServeFixture {
        reader: Cursor::new(encode_frame_stream(&frames).to_vec()),
        builder,
        executor,
        config,
    }
}

// The production serve path: a refused presentation writes no protocol frame,
// consumes no input, redeems nothing, consumes no nonce and surfaces only the
// constant refusal.
// r[verify mantle.authority_attenuation.caveat_filter_semantics]
#[test]
fn production_serve_refuses_unsafe_presentation_with_no_frame_and_no_state_change() {
    let mut receiver = Receiver::new();
    let mut fixture = serve_fixture(&mut receiver, vec![filter("00", r#"{"kind":"future"}"#)], false);
    let mut writer = Vec::new();
    let context = RemoteProductionServerContext {
        builder: &fixture.builder,
        verifier_key: &receiver.key,
        executor: &fixture.executor,
        secret_request: None,
        usage_cgroup_root: None,
        credential_state_dir: receiver.dir.path(),
        binding_config: Some(&fixture.config),
        execution_state_dir: receiver.dir.path(),
        authenticated_client_endpoint: None,
        attenuation_verifier: Some(&receiver.grants.verifier),
    };
    let before = receiver.snapshot();
    let error = serve_stdio_remote_production_once(&mut fixture.reader, &mut writer, context).unwrap_err();
    assert_eq!(error, ATTENUATED_GRANT_DENIED);
    assert!(writer.is_empty(), "a refused presentation wrote a protocol frame");
    assert_eq!(receiver.snapshot(), before);
    assert_eq!(receiver.uses_remaining(), TICKET_USES);
}

// The production serve path admits a valid presentation: AuthOk is the first
// frame written, exactly one ticket use is redeemed and the holder nonce is
// recorded. The stream then ends (no inputs follow), which is a later, different
// failure than the constant refusal.
// r[verify mantle.authority_attenuation.attenuated_remote_grant]
#[test]
fn production_serve_admits_a_valid_presentation_and_writes_auth_ok() {
    let mut receiver = Receiver::new();
    let mut fixture = serve_fixture(&mut receiver, Vec::new(), true);
    let mut writer = Vec::new();
    let context = RemoteProductionServerContext {
        builder: &fixture.builder,
        verifier_key: &receiver.key,
        executor: &fixture.executor,
        secret_request: None,
        usage_cgroup_root: None,
        credential_state_dir: receiver.dir.path(),
        binding_config: Some(&fixture.config),
        execution_state_dir: receiver.dir.path(),
        authenticated_client_endpoint: None,
        attenuation_verifier: Some(&receiver.grants.verifier),
    };
    let ledger_before = receiver.ledger();
    let error = serve_stdio_remote_production_once(&mut fixture.reader, &mut writer, context).unwrap_err();
    assert_ne!(error, ATTENUATED_GRANT_DENIED, "the admitted request must pass attenuation");
    let frames = decode_remote_frame_stream(&writer).expect("written frames decode");
    assert!(matches!(frames.first(), Some(RemoteFrame::AuthOk { .. })), "AuthOk must be written first");
    assert_eq!(receiver.uses_remaining(), TICKET_USES - 1);
    assert_ne!(receiver.ledger(), ledger_before);
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
    let rewrite = identity_rewrite("00", &target_of(&request));
    let (presentation, _) = receiver.grants.sign(&request, vec![], vec![rewrite], GrantFixture::current_window());
    let guard = config
        .admit_build(request.endpoint_binding.as_ref().unwrap(), now(), &builder.store_prefix)
        .unwrap();
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

/// Collects formatted tracing output so tests can see exactly what a process
/// would print at a given level.
#[derive(Clone, Default)]
struct LogCapture(std::sync::Arc<std::sync::Mutex<Vec<u8>>>);

impl std::io::Write for LogCapture {
    fn write(&mut self, data: &[u8]) -> std::io::Result<usize> {
        self.0.lock().unwrap().extend_from_slice(data);
        Ok(data.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

fn captured_at(level: tracing::Level, run: impl FnOnce()) -> String {
    let capture = LogCapture::default();
    let sink = capture.clone();
    let subscriber = tracing_subscriber::fmt()
        .with_max_level(level)
        .with_ansi(false)
        .with_writer(move || sink.clone())
        .finish();
    tracing::subscriber::with_default(subscriber, run);
    let bytes = capture.0.lock().unwrap().clone();
    String::from_utf8(bytes).unwrap()
}

fn serve_denied_once(unknown_caveat: bool) {
    let mut receiver = Receiver::new();
    let mut fixture = serve_fixture(&mut receiver, vec![filter("00", r#"{"kind":"future"}"#)], !unknown_caveat);
    let mut writer = Vec::new();
    let context = RemoteProductionServerContext {
        builder: &fixture.builder,
        verifier_key: &receiver.key,
        executor: &fixture.executor,
        secret_request: None,
        usage_cgroup_root: None,
        credential_state_dir: receiver.dir.path(),
        binding_config: Some(&fixture.config),
        execution_state_dir: receiver.dir.path(),
        authenticated_client_endpoint: None,
        attenuation_verifier: if unknown_caveat {
            Some(&receiver.grants.verifier)
        } else {
            None
        },
    };
    let error = serve_stdio_remote_production_once(&mut fixture.reader, &mut writer, context).unwrap_err();
    assert_eq!(error, ATTENUATED_GRANT_DENIED);
    assert!(writer.is_empty());
}

// Denial reason codes are debug-level only. At the default process level
// (`warn`) and at `info`, a denied presentation on the production serve path and
// on a non-enforcing path prints NOTHING, so an ssh-stdio client that relays the
// remote's stderr learns no reason; at `debug` the reason code is present, which
// proves the capture works and that debug logging must stay off for an
// untrusted peer.
// r[verify mantle.authority_attenuation.attenuated_remote_grant]
#[test]
fn denial_reason_codes_appear_only_at_debug_level() {
    for (case, code) in [
        ("verification", "verification-denied"),
        ("no-authority", "no-operator-authority"),
    ] {
        let unknown = case == "verification";
        for level in [tracing::Level::WARN, tracing::Level::INFO] {
            let output = captured_at(level, || serve_denied_once(unknown));
            assert!(output.is_empty(), "{case} at {level}: default-level output leaked: {output:?}");
        }
        let output = captured_at(tracing::Level::DEBUG, || serve_denied_once(unknown));
        assert!(output.contains(code), "{case}: debug output must carry the reason: {output:?}");
        assert!(!output.contains("future"), "{case}: debug output must not carry caveat text: {output:?}");
    }
    let refuse = || {
        let mut client = fixture_loopback_client(vec!["input-a".to_string()], vec!["builder-key".to_string()]);
        client.auth.attenuated_grant = Some(AttenuatedGrantPresentation {
            compact_token: "x".into(),
            nonce: [1; 32],
            holder_signature: vec![2; 64],
            subject: "s".into(),
            account: "a".into(),
            project: SCOPE.into(),
            audience: AUDIENCE.into(),
            generation: GENERATION,
        });
        let error = run_loopback_remote_session(&fixture_loopback_builder(), &mut fixture_ticket(), &client);
        assert_eq!(error.unwrap_err(), ATTENUATED_GRANT_DENIED);
    };
    assert!(captured_at(tracing::Level::WARN, refuse).is_empty());
    assert!(captured_at(tracing::Level::DEBUG, refuse).contains("non-enforcing-admission-path"));
}

// Golden vector for the whole-request digest, reproducible by an external
// signer: BLAKE3 over domain `mantle/remote/attenuated-build-request/v2\0`, then
// the endpoint id and the compact `serde_json` bytes of the request, each framed
// as an 8-byte big-endian length plus the bytes. The JSON is a function of
// struct field declaration order, so reordering or adding a serialized field of
// `ConcreteBuildRequest` (or a nested type) changes this vector on purpose and
// invalidates every outstanding presentation. The expected digest was computed
// independently with `b3sum`.
// r[verify mantle.authority_attenuation.attenuated_remote_grant]
#[test]
fn whole_request_digest_matches_the_golden_vector() {
    let request = ConcreteBuildRequest {
        request_id: "r1".into(),
        store_prefix: "/mantle/store".into(),
        input_refs: Vec::new(),
        source_input_refs: Vec::new(),
        upload_bytes: 0,
        build_time_limit_secs: 60,
        contains_raw_frontend_eval: false,
        payload: RemoteConcreteBuildPayload::Action {
            action_id: "a".into(),
            spec_json: "{}".into(),
        },
        expected_outputs: vec![RemoteExpectedOutput {
            name: "out".into(),
            logical_path: None,
        }],
        production_attempt: Some(RemoteProductionAttemptBinding {
            job_id: RemoteJobId::new("job-a".to_string()).unwrap(),
            attempt_id: RemoteAttemptId::new("attempt-1".to_string()).unwrap(),
            fence_generation: RemoteFenceGeneration::INITIAL,
        }),
        transfer_policy: None,
        resource_requirements: None,
        locality_scope: None,
        failure_debug_policy: crunch_build::distributed::RemoteFailureDebugPolicy::default(),
        failure_replay: None,
        endpoint_binding: None,
    };
    let json = String::from_utf8(serde_json::to_vec(&request).unwrap()).unwrap();
    assert_eq!(json, GOLDEN_REQUEST_JSON);
    let binding = bind_remote_job(&request, "builder-1").unwrap();
    assert_eq!(data_encoding::HEXLOWER.encode(&binding.request_digest), GOLDEN_REQUEST_DIGEST);
    assert_eq!(
        binding.target,
        format!("/job/r1/attempt/job-a/class/action/outputs/{}", output_set_id(["out"]).unwrap())
    );
}

const GOLDEN_REQUEST_JSON: &str = r#"{"request_id":"r1","store_prefix":"/mantle/store","input_refs":[],"upload_bytes":0,"build_time_limit_secs":60,"contains_raw_frontend_eval":false,"payload":{"kind":"action","action_id":"a","spec_json":"{}"},"expected_outputs":[{"name":"out"}],"production_attempt":{"job_id":"job-a","attempt_id":"attempt-1","fence_generation":1},"failure_debug_policy":{"policy_name":"mantle-remote-failure-debug-default-v1","metadata_bytes_max":1048576,"object_bytes_max":67108864,"retention_secs":604800,"replay_enabled":false,"capture":{"enabled":false,"allowed_relative_paths":[],"sensitivity":"restricted-diagnostic","file_count_max":32,"total_bytes_max":16777216,"file_bytes_max":8388608,"depth_max":16,"failure_mode":"diagnostic-only"}}}"#;
const GOLDEN_REQUEST_DIGEST: &str = "19634435d3643f1de010868bf63a38db19cfce470f7ec56c7a34c71f6a861997";
