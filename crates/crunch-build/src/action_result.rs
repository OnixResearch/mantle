//! Derivation adapter and evidence checks for shared action results.
//!
//! The canonical action-result model is provider-neutral. This module adapts
//! Mantle derivations and admitted PathInfo facts to that model. The explicit
//! signed-derivation admission below uses read-only store capabilities; it
//! never publishes an output or dispatches an executor.

use std::collections::BTreeMap;

use crunch_action_result_core::ACTION_RECEIPT_REF_PREFIX;
use crunch_action_result_core::ACTION_REF_PREFIX;
use crunch_action_result_core::ACTION_RESULT_POLICY_SCHEMA;
use crunch_action_result_core::ActionResultOutput;
use crunch_action_result_core::ActionResultRecord;
use crunch_action_result_core::ActionResultRecordInput;
use crunch_action_result_core::ActionResultTrustPolicy;
use crunch_action_result_core::CandidateAdmissionFacts;
use crunch_action_result_core::CandidateDecision;
use crunch_action_result_core::DetachedRecordSignature;
use crunch_action_result_core::DiscoveredActionResultCandidate;
use crunch_action_result_core::MAX_ACTION_RESULT_CANDIDATES;
use crunch_action_result_core::NETWORK_POLICY_REF_PREFIX;
use crunch_action_result_core::OBJECT_REF_PREFIX;
use crunch_action_result_core::PATH_INFO_REF_PREFIX;
use crunch_action_result_core::PRODUCER_POLICY_REF_PREFIX;
use crunch_action_result_core::PUBLICATION_POLICY_REF_PREFIX;
use crunch_action_result_core::REFERENCE_SCAN_REF_PREFIX;
use crunch_action_result_core::SANDBOX_POLICY_REF_PREFIX;
use crunch_action_result_core::SIGNATURE_REF_PREFIX;
use crunch_action_result_core::STRONG_CLAIM;
use crunch_action_result_core::SignedActionResultRecord;
use crunch_action_result_core::StrongReuseRequest;
use crunch_action_result_core::canonical_action_result;
use crunch_action_result_core::plan_strong_reuse;
use crunch_store::ActionResultPort;
use crunch_store::BuildServiceStore;
use crunch_store::BuildStore;
use crunch_store::OutputLookup;
use nix_compat::derivation::Derivation;
use nix_compat::narinfo::Signature;
use nix_compat::narinfo::VerifyingKey;
use nix_compat::narinfo::fingerprint_with_store_dir;
use nix_compat::store_path::StorePath;
use nix_compat::store_path::StorePathRef;
use serde::Deserialize;
use serde::Serialize;
use snix_castore::Node;
use snix_store::path_info::PathInfo;

use crate::HermeticityMode;
use crate::KeyPair;
use crate::signing;

pub const ACTION_RESULT_RUNTIME_REPORT_SCHEMA: &str = "mantle-action-result-runtime-report-v1";
pub const ACTION_RESULT_DISPOSITION_MISS: &str = "miss";
pub const ACTION_RESULT_DISPOSITION_REUSED: &str = "reused";
pub const ACTION_RESULT_DISPOSITION_CONFLICT: &str = "conflict";
pub const ACTION_RESULT_DISPOSITION_PUBLISHED: &str = "published";
pub const ACTION_RESULT_PHASE_DISCOVERY: &str = "discovery";
pub const ACTION_RESULT_PHASE_PUBLICATION: &str = "publication";

const MAX_SIGNED_DERIVATION_BYTES: u64 = 1_048_576;
const MAX_SIGNED_DERIVATION_NAR_BYTES: u64 = 2_097_152;
const MAX_SIGNED_DERIVATION_SIGNATURES: usize = 16;
const MAX_SIGNED_DERIVATION_SIGNER_BYTES: usize = 256;
const ACTION_DOMAIN: &[u8] = b"mantle.action-spec.derivation-adapter.v1";
const OBJECT_DOMAIN: &[u8] = b"mantle.action-result.object.v1";
const PATH_INFO_DOMAIN: &[u8] = b"mantle.action-result.pathinfo.v1";
const ACTION_RECEIPT_DOMAIN: &[u8] = b"mantle.action-result.receipt.v1";
const REFERENCE_SCAN_DOMAIN: &[u8] = b"mantle.action-result.reference-scan.v1";
const SANDBOX_POLICY_DOMAIN: &[u8] = b"mantle.action-result.sandbox-policy.v1";
const NETWORK_POLICY_DOMAIN: &[u8] = b"mantle.action-result.network-policy.v1";
const PRODUCER_POLICY_DOMAIN: &[u8] = b"mantle.action-result.producer-policy.v1";
const PUBLICATION_POLICY_DOMAIN: &[u8] = b"mantle.action-result.publication-policy.v1";
const SIGNATURE_DOMAIN: &[u8] = b"mantle.action-result.signature.v1";
const DOMAIN_SEPARATOR: u8 = 0;
const PRODUCER_POLICY_ID: &str = "ed25519-record-and-pathinfo-v1";
const PUBLICATION_POLICY_ID: &str = "record-before-index-atomic-no-clobber-v1";
const ACTION_RESULT_TRUST_POLICY_ID: &str = "mantle-action-result-default-v1";
const NO_TRUSTED_SIGNER_SENTINEL: &str = "no-trusted-action-result-signer-configured";
const NON_CLAIM_CA_MAPPING: &str = "ca-mapping-presence-is-not-output-trust";
const NON_CLAIM_EXECUTOR: &str = "executor-correctness";
const NON_CLAIM_INDEX: &str = "index-presence-is-not-output-trust";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ActionPolicyRefs {
    pub sandbox_policy_ref: String,
    pub network_policy_ref: String,
    pub producer_policy_ref: String,
    pub publication_policy_ref: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ActionResultTransferEvidence {
    pub output_count: u32,
    pub transferred_nar_bytes: u64,
    pub reused_nar_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ActionResultRuntimeReport {
    pub schema: String,
    pub phase: String,
    pub action_ref: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unresolved_derivation: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolved_derivation: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolved_identity: Option<String>,
    pub disposition: String,
    pub selected_result_ref: Option<String>,
    pub selected_source_id: Option<String>,
    pub selected_source_class: Option<String>,
    pub trust_basis: Vec<String>,
    pub conflict_class: Option<String>,
    pub candidate_decisions: Vec<CandidateDecision>,
    pub publication_result_refs: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transfer: Option<ActionResultTransferEvidence>,
    pub diagnostics: Vec<String>,
    pub non_claims: Vec<String>,
}

struct PathInfoRefInput<'a> {
    object_ref: &'a str,
    store_dir: &'a str,
}

struct ActionReceiptRefInput<'a> {
    action_ref: &'a str,
    outputs: &'a [ActionResultOutput],
    reference_scan_refs: &'a [String],
    policy_refs: &'a ActionPolicyRefs,
    producer_identity: &'a str,
    signature_refs: &'a [String],
}

pub fn action_ref_for_derivation(derivation: &Derivation, store_dir: &str) -> String {
    assert!(!store_dir.is_empty(), "store_dir must not be empty");
    assert!(store_dir.starts_with('/'), "store_dir must be absolute");
    domain_ref(ACTION_REF_PREFIX, ACTION_DOMAIN, &derivation.to_aterm_bytes_with_store_dir(store_dir))
}

pub fn policy_refs_for_derivation(
    derivation: &Derivation,
    store_dir: &str,
    hermeticity_mode: HermeticityMode,
) -> Result<ActionPolicyRefs, String> {
    let aterm_digest = domain_digest(ACTION_DOMAIN, &derivation.to_aterm_bytes_with_store_dir(store_dir));
    let sandbox_bytes = serde_json::to_vec(&serde_json::json!({
        "schema": "mantle-action-result-sandbox-policy-v1",
        "derivation_aterm_blake3": aterm_digest,
        "hermeticity_mode": hermeticity_mode.as_str(),
        "sandbox": derivation.system,
    }))
    .map_err(|error| format!("action-result-sandbox-policy-json:{error}"))?;
    let network_bytes = serde_json::to_vec(&serde_json::json!({
        "schema": "mantle-action-result-network-policy-v1",
        "derivation_aterm_blake3": aterm_digest,
        "fixed_output": derivation.outputs.values().any(|output| output.ca_hash.is_some()),
        "hermeticity_mode": hermeticity_mode.as_str(),
    }))
    .map_err(|error| format!("action-result-network-policy-json:{error}"))?;
    let policy_refs = ActionPolicyRefs {
        sandbox_policy_ref: domain_ref(SANDBOX_POLICY_REF_PREFIX, SANDBOX_POLICY_DOMAIN, &sandbox_bytes),
        network_policy_ref: domain_ref(NETWORK_POLICY_REF_PREFIX, NETWORK_POLICY_DOMAIN, &network_bytes),
        producer_policy_ref: domain_ref(
            PRODUCER_POLICY_REF_PREFIX,
            PRODUCER_POLICY_DOMAIN,
            PRODUCER_POLICY_ID.as_bytes(),
        ),
        publication_policy_ref: domain_ref(
            PUBLICATION_POLICY_REF_PREFIX,
            PUBLICATION_POLICY_DOMAIN,
            PUBLICATION_POLICY_ID.as_bytes(),
        ),
    };
    debug_assert!(policy_refs.sandbox_policy_ref.starts_with(SANDBOX_POLICY_REF_PREFIX));
    debug_assert!(policy_refs.network_policy_ref.starts_with(NETWORK_POLICY_REF_PREFIX));
    Ok(policy_refs)
}

pub fn signed_record_for_outputs(
    derivation: &Derivation,
    outputs: &BTreeMap<String, PathInfo>,
    store_dir: &str,
    hermeticity_mode: HermeticityMode,
    keypair: &KeyPair,
) -> Result<SignedActionResultRecord, String> {
    let action_ref = action_ref_for_derivation(derivation, store_dir);
    let producer_identity = keypair.verifying_key.name().to_string();
    let input = record_input_for_outputs(
        action_ref,
        derivation,
        outputs,
        store_dir,
        hermeticity_mode,
        producer_identity.clone(),
    )?;
    let record = canonical_action_result(input).map_err(|error| error.code().to_string())?;
    let signature = keypair.signing_key.sign(record.result_ref.as_bytes()).to_owned();
    let signed_record = SignedActionResultRecord {
        record,
        record_signatures: vec![DetachedRecordSignature {
            key_name: producer_identity,
            signature: signature.to_string(),
        }],
    };
    debug_assert_eq!(signed_record.record.action_ref, action_ref_for_derivation(derivation, store_dir));
    debug_assert_eq!(signed_record.record_signatures.len(), 1);
    Ok(signed_record)
}

#[expect(
    tigerstyle::too_many_parameters,
    reason = "stable public adapter signature; named private inputs remove ambiguity below this boundary"
)]
pub fn record_input_for_outputs(
    action_ref: String,
    derivation: &Derivation,
    outputs: &BTreeMap<String, PathInfo>,
    store_dir: &str,
    hermeticity_mode: HermeticityMode,
    producer_identity: String,
) -> Result<ActionResultRecordInput, String> {
    if outputs.is_empty() {
        return Err("action-result-output-set-empty".to_string());
    }
    if !outputs_match_declared_paths(derivation, outputs) {
        return Err("action-result-output-store-path-does-not-match-action".to_string());
    }
    let policy_refs = policy_refs_for_derivation(derivation, store_dir, hermeticity_mode)?;
    let result_outputs = canonical_outputs(outputs, store_dir)?;
    let signature_refs = output_signature_refs(outputs);
    let reference_scan_refs = reference_scan_refs(outputs, &result_outputs, store_dir)?;
    let action_receipt_ref = action_receipt_ref(ActionReceiptRefInput {
        action_ref: &action_ref,
        outputs: &result_outputs,
        reference_scan_refs: &reference_scan_refs,
        policy_refs: &policy_refs,
        producer_identity: &producer_identity,
        signature_refs: &signature_refs,
    })?;
    let record_input = ActionResultRecordInput {
        action_ref,
        outputs: result_outputs,
        action_receipt_ref,
        reference_scan_refs,
        sandbox_policy_ref: policy_refs.sandbox_policy_ref,
        network_policy_ref: policy_refs.network_policy_ref,
        producer_identity,
        producer_policy_ref: policy_refs.producer_policy_ref,
        signature_refs,
        publication_policy_ref: policy_refs.publication_policy_ref,
        non_claims: runtime_non_claims(),
    };
    debug_assert_eq!(record_input.outputs.len(), outputs.len());
    debug_assert!(!record_input.outputs.is_empty());
    Ok(record_input)
}

#[expect(
    tigerstyle::too_many_parameters,
    reason = "stable public admission API; all internal multi-value helpers use named inputs"
)]
pub fn candidate_admission_facts(
    source_id: String,
    source_class: String,
    signed: &SignedActionResultRecord,
    derivation: &Derivation,
    outputs: Option<&BTreeMap<String, PathInfo>>,
    store_dir: &str,
    hermeticity_mode: HermeticityMode,
    trusted_keys: &[VerifyingKey],
) -> CandidateAdmissionFacts {
    let verified_record_signers = verify_record_signatures(signed, trusted_keys);
    let expected = outputs.and_then(|outputs| {
        record_input_for_outputs(
            action_ref_for_derivation(derivation, store_dir),
            derivation,
            outputs,
            store_dir,
            hermeticity_mode,
            signed.record.producer_identity.clone(),
        )
        .ok()
    });
    let is_path_info_signatures_verified = outputs
        .map(|outputs| all_path_info_signatures_verified(outputs, store_dir, trusted_keys))
        .unwrap_or(false);
    let is_producer_policy_admitted = expected.as_ref().is_some_and(|input| {
        input.producer_policy_ref == signed.record.producer_policy_ref
            && input.producer_identity == signed.record.producer_identity
            && verified_record_signers.contains(&signed.record.producer_identity)
    });
    let facts = CandidateAdmissionFacts {
        source_id,
        source_class,
        verified_record_signers,
        action_receipt_linked: expected
            .as_ref()
            .is_some_and(|input| input.action_receipt_ref == signed.record.action_receipt_ref),
        object_refs_complete: expected.as_ref().is_some_and(|input| same_object_refs(input, &signed.record)),
        path_info_refs_linked: expected.as_ref().is_some_and(|input| same_path_info_refs(input, &signed.record)),
        path_info_signatures_verified: is_path_info_signatures_verified,
        producer_policy_admitted: is_producer_policy_admitted,
        publication_policy_admitted: expected
            .as_ref()
            .is_some_and(|input| input.publication_policy_ref == signed.record.publication_policy_ref),
        sandbox_policy_admitted: expected
            .as_ref()
            .is_some_and(|input| input.sandbox_policy_ref == signed.record.sandbox_policy_ref),
        network_policy_admitted: expected
            .as_ref()
            .is_some_and(|input| input.network_policy_ref == signed.record.network_policy_ref),
        reference_scans_admitted: expected
            .as_ref()
            .is_some_and(|input| input.reference_scan_refs == signed.record.reference_scan_refs),
        claim_strength: STRONG_CLAIM.to_string(),
    };
    debug_assert!(facts.verified_record_signers.len() <= signed.record_signatures.len());
    debug_assert_eq!(facts.claim_strength, STRONG_CLAIM);
    facts
}

pub fn trust_policy_for_action(
    policy_refs: &ActionPolicyRefs,
    trusted_keys: &[VerifyingKey],
) -> ActionResultTrustPolicy {
    let mut signer_names = trusted_keys.iter().map(|key| key.name().to_string()).collect::<Vec<_>>();
    signer_names.sort();
    signer_names.dedup();
    if signer_names.is_empty() {
        signer_names.push(NO_TRUSTED_SIGNER_SENTINEL.to_string());
    }
    let trust_policy = ActionResultTrustPolicy {
        schema: ACTION_RESULT_POLICY_SCHEMA.to_string(),
        policy_id: ACTION_RESULT_TRUST_POLICY_ID.to_string(),
        trusted_producers: signer_names.clone(),
        trusted_record_signers: signer_names,
        allowed_source_classes: crunch_store::action_result_runtime_policy().sources.allowed_classes.clone(),
        allowed_sandbox_policy_refs: vec![policy_refs.sandbox_policy_ref.clone()],
        allowed_network_policy_refs: vec![policy_refs.network_policy_ref.clone()],
        required_non_claims: runtime_non_claims(),
        require_record_signature: true,
        require_path_info_signature: true,
    };
    debug_assert!(trust_policy.require_record_signature);
    debug_assert!(trust_policy.require_path_info_signature);
    trust_policy
}

/// Read-only capabilities used to establish the exact bytes and identity of
/// a signed derivation before any action-result lookup can occur.
pub struct SignedDerivationSource<'a> {
    pub build_store: &'a BuildStore,
    pub lookup: &'a OutputLookup,
    pub content: &'a BuildServiceStore,
    pub store_dir: &'a str,
    pub trusted_keys: &'a [VerifyingKey],
}

/// Its fields cannot be supplied by a caller: only measured, signed CAS bytes
/// from `verify_signed_store_derivation` can construct this witness.
pub struct VerifiedStoreDerivation<'a> {
    path: StorePath<String>,
    derivation: Derivation,
    store_dir: &'a str,
    trusted_keys: &'a [VerifyingKey],
}

impl VerifiedStoreDerivation<'_> {
    pub fn path(&self) -> &StorePath<String> {
        &self.path
    }

    pub fn derivation(&self) -> &Derivation {
        &self.derivation
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignedDerivationAdmissionError {
    Identity,
    MissingDerivation,
    UntrustedDerivation,
    IncompleteDerivation,
    Discovery,
    UntrustedResult,
    AmbiguousResults,
}

impl SignedDerivationAdmissionError {
    pub fn code(self) -> &'static str {
        match self {
            Self::Identity => "signed-derivation-identity-invalid",
            Self::MissingDerivation => "signed-derivation-missing",
            Self::UntrustedDerivation => "signed-derivation-untrusted",
            Self::IncompleteDerivation => "signed-derivation-incomplete",
            Self::Discovery => "signed-derivation-discovery-failed",
            Self::UntrustedResult => "signed-derivation-result-untrusted",
            Self::AmbiguousResults => "signed-derivation-results-ambiguous",
        }
    }
}

/// Bind the requested logical `.drv` to actual bounded ATerm bytes in CAS.
/// Metadata signatures are verified with full configured public keys, then
/// the NAR digest is measured rather than accepted from signed metadata alone.
pub async fn verify_signed_store_derivation<'a>(
    source: SignedDerivationSource<'a>,
    path: &StorePath<String>,
) -> Result<VerifiedStoreDerivation<'a>, SignedDerivationAdmissionError> {
    use SignedDerivationAdmissionError as Failure;

    if !source.store_dir.starts_with('/')
        || !path.name().ends_with(".drv")
        || source.trusted_keys.is_empty()
        || source.trusted_keys.len() > MAX_SIGNED_DERIVATION_SIGNATURES
    {
        return Err(Failure::Identity);
    }
    let info = source
        .lookup
        .find(path)
        .await
        .map_err(|_| Failure::UntrustedDerivation)?
        .ok_or(Failure::MissingDerivation)?;
    let Node::File { size, .. } = &info.node else {
        return Err(Failure::Identity);
    };
    if *size == 0
        || *size > MAX_SIGNED_DERIVATION_BYTES
        || info.nar_size == 0
        || info.nar_size > MAX_SIGNED_DERIVATION_NAR_BYTES
        || info.ca.is_some()
        || info.signatures.is_empty()
        || info.signatures.len() > MAX_SIGNED_DERIVATION_SIGNATURES
        || info.signatures.iter().any(|signature| signature.name().len() > MAX_SIGNED_DERIVATION_SIGNER_BYTES)
    {
        return Err(Failure::Identity);
    }
    let references: Vec<StorePathRef<'_>> = info.references.iter().map(StorePath::as_ref).collect();
    let fingerprint = fingerprint_with_store_dir(
        &info.store_path.as_ref(),
        &info.nar_sha256,
        info.nar_size,
        references.iter(),
        source.store_dir,
    );
    if !info
        .signatures
        .iter()
        .any(|signature| source.trusted_keys.iter().any(|key| key.verify(&fingerprint, &signature.as_ref())))
    {
        return Err(Failure::UntrustedDerivation);
    }
    if !source.content.has_complete_content(&info).await.map_err(|_| Failure::IncompleteDerivation)? {
        return Err(Failure::IncompleteDerivation);
    }
    let measured = source.build_store.calculate_nar(&info.node).await.map_err(|_| Failure::IncompleteDerivation)?;
    if measured != (info.nar_size, info.nar_sha256) {
        return Err(Failure::UntrustedDerivation);
    }
    let bytes = source
        .build_store
        .read_file_node(&info.node, MAX_SIGNED_DERIVATION_BYTES)
        .await
        .map_err(|_| Failure::IncompleteDerivation)?;
    let derivation = Derivation::from_aterm_bytes(&bytes).map_err(|_| Failure::Identity)?;
    if derivation.to_aterm_bytes_with_store_dir(source.store_dir) != bytes {
        return Err(Failure::Identity);
    }
    Ok(VerifiedStoreDerivation {
        path: path.clone(),
        derivation,
        store_dir: source.store_dir,
        trusted_keys: source.trusted_keys,
    })
}

/// A signed action result bound to the verified `.drv` bytes and complete
/// signed PathInfo. A remote-only output still needs local admission before
/// a Nix worker may claim that store path present.
pub struct AdmittedSignedDerivationOutputs {
    pub result_ref: String,
    pub outputs: BTreeMap<String, PathInfo>,
}

/// `None` means the verified derivation has no discovered action-result
/// records. Invalid records, source errors and distinct admitted candidates
/// are errors, never an apparently successful cache miss.
pub async fn admit_signed_derivation_outputs(
    results: &ActionResultPort,
    verified: &VerifiedStoreDerivation<'_>,
    mode: HermeticityMode,
) -> Result<Option<AdmittedSignedDerivationOutputs>, SignedDerivationAdmissionError> {
    use SignedDerivationAdmissionError as Failure;

    let derivation = &verified.derivation;
    let action_ref = action_ref_for_derivation(derivation, verified.store_dir);
    let discovery = results.discover(&action_ref).await;
    if !discovery.diagnostics.is_empty() || discovery.lookups.iter().any(|lookup| !lookup.diagnostics.is_empty()) {
        return Err(Failure::Discovery);
    }
    let candidate_count = discovery
        .lookups
        .iter()
        .try_fold(0usize, |count, lookup| count.checked_add(lookup.records.len()))
        .ok_or(Failure::Discovery)?;
    if candidate_count > MAX_ACTION_RESULT_CANDIDATES {
        return Err(Failure::Discovery);
    }
    if candidate_count == 0 {
        return Ok(None);
    }
    let policy_refs =
        policy_refs_for_derivation(derivation, verified.store_dir, mode).map_err(|_| Failure::UntrustedResult)?;
    let request = StrongReuseRequest {
        action_ref,
        output_names: derivation.outputs.keys().cloned().collect(),
        policy: trust_policy_for_action(&policy_refs, verified.trusted_keys),
    };
    let mut candidates = Vec::with_capacity(candidate_count);
    let mut probes = BTreeMap::new();
    for lookup in discovery.lookups {
        for signed_record in lookup.records {
            let result_ref = signed_record.record.result_ref.clone();
            let probe = results.probe_outputs(&signed_record.record).await.ok();
            let facts = candidate_admission_facts(
                lookup.source_id.clone(),
                lookup.source_class.clone(),
                &signed_record,
                derivation,
                probe.as_ref().map(|probe| &probe.outputs),
                verified.store_dir,
                mode,
                verified.trusted_keys,
            );
            if let Some(probe) = probe {
                probes.entry(result_ref).or_insert(probe.outputs);
            }
            candidates.push(DiscoveredActionResultCandidate { signed_record, facts });
        }
    }
    let plan = plan_strong_reuse(request, candidates).map_err(|_| Failure::UntrustedResult)?;
    if plan.conflict_class.is_some() || plan.candidate_decisions.iter().filter(|decision| decision.admitted).count() > 1
    {
        return Err(Failure::AmbiguousResults);
    }
    let selected = plan.selected_result_ref.ok_or(Failure::UntrustedResult)?;
    let outputs = probes.remove(&selected).ok_or(Failure::UntrustedResult)?;
    if outputs.len() != derivation.outputs.len()
        || derivation.outputs.iter().any(|(name, declaration)| {
            outputs
                .get(name)
                .is_none_or(|info| declaration.path.as_ref().is_some_and(|expected| expected != &info.store_path))
        })
    {
        return Err(Failure::UntrustedResult);
    }
    Ok(Some(AdmittedSignedDerivationOutputs {
        result_ref: selected,
        outputs,
    }))
}

#[expect(
    tigerstyle::too_many_parameters,
    reason = "stable public report adapter signature retained for existing plan and runtime callers"
)]
pub fn discovery_runtime_report(
    action_ref: String,
    disposition: &str,
    plan: crunch_action_result_core::StrongReusePlan,
    selected_source: Option<(String, String)>,
    transfer: Option<ActionResultTransferEvidence>,
    diagnostics: Vec<String>,
) -> ActionResultRuntimeReport {
    let candidate_decision_count = plan.candidate_decisions.len();
    let non_claim_count = plan.non_claims.len();
    let trust_basis = plan
        .candidate_decisions
        .iter()
        .find(|decision| Some(&decision.result_ref) == plan.selected_result_ref.as_ref())
        .map(|decision| decision.trust_basis.clone())
        .unwrap_or_default();
    let runtime_evidence = ActionResultRuntimeReport {
        schema: ACTION_RESULT_RUNTIME_REPORT_SCHEMA.to_string(),
        phase: ACTION_RESULT_PHASE_DISCOVERY.to_string(),
        action_ref,
        unresolved_derivation: None,
        resolved_derivation: None,
        resolved_identity: None,
        disposition: disposition.to_string(),
        selected_result_ref: plan.selected_result_ref,
        selected_source_id: selected_source.as_ref().map(|source| source.0.clone()),
        selected_source_class: selected_source.map(|source| source.1),
        trust_basis,
        conflict_class: plan.conflict_class,
        candidate_decisions: plan.candidate_decisions,
        publication_result_refs: Vec::new(),
        transfer,
        diagnostics,
        non_claims: plan.non_claims,
    };
    debug_assert_eq!(runtime_evidence.candidate_decisions.len(), candidate_decision_count);
    debug_assert_eq!(runtime_evidence.non_claims.len(), non_claim_count);
    runtime_evidence
}

pub fn publication_runtime_report(
    action_ref: String,
    result_ref: String,
    diagnostics: Vec<String>,
) -> ActionResultRuntimeReport {
    ActionResultRuntimeReport {
        schema: ACTION_RESULT_RUNTIME_REPORT_SCHEMA.to_string(),
        phase: ACTION_RESULT_PHASE_PUBLICATION.to_string(),
        action_ref,
        unresolved_derivation: None,
        resolved_derivation: None,
        resolved_identity: None,
        disposition: ACTION_RESULT_DISPOSITION_PUBLISHED.to_string(),
        selected_result_ref: None,
        selected_source_id: None,
        selected_source_class: None,
        trust_basis: Vec::new(),
        conflict_class: None,
        candidate_decisions: Vec::new(),
        publication_result_refs: vec![result_ref],
        transfer: None,
        diagnostics,
        non_claims: runtime_non_claims(),
    }
}

fn outputs_match_declared_paths(derivation: &Derivation, outputs: &BTreeMap<String, PathInfo>) -> bool {
    if derivation.outputs.len() != outputs.len() {
        return false;
    }
    derivation.outputs.iter().all(|(output_name, output)| {
        let Some(path_info) = outputs.get(output_name) else {
            return false;
        };
        output.path.as_ref().is_none_or(|declared_path| declared_path == &path_info.store_path)
    })
}

fn canonical_outputs(outputs: &BTreeMap<String, PathInfo>, store_dir: &str) -> Result<Vec<ActionResultOutput>, String> {
    let mut result = Vec::with_capacity(outputs.len());
    for (name, path_info) in outputs {
        let object_ref = object_ref_for_node(&path_info.node)?;
        result.push(ActionResultOutput {
            name: name.clone(),
            object_ref: object_ref.clone(),
            store_path: path_info.store_path.to_absolute_path_with_prefix(store_dir),
            path_info_ref: path_info_ref(path_info, PathInfoRefInput {
                object_ref: &object_ref,
                store_dir,
            })?,
        });
    }
    Ok(result)
}

fn object_ref_for_node(node: &Node) -> Result<String, String> {
    let bytes = serde_json::to_vec(node).map_err(|error| format!("action-result-object-json:{error}"))?;
    Ok(domain_ref(OBJECT_REF_PREFIX, OBJECT_DOMAIN, &bytes))
}

fn path_info_ref(path_info: &PathInfo, input: PathInfoRefInput<'_>) -> Result<String, String> {
    let mut references = path_info
        .references
        .iter()
        .map(|reference| reference.to_absolute_path_with_prefix(input.store_dir))
        .collect::<Vec<_>>();
    references.sort();
    let mut signatures = path_info.signatures.iter().map(ToString::to_string).collect::<Vec<_>>();
    signatures.sort();
    debug_assert!(references.windows(2).all(|window| window[0] <= window[1]));
    debug_assert!(signatures.windows(2).all(|window| window[0] <= window[1]));
    let bytes = serde_json::to_vec(&serde_json::json!({
        "schema": "mantle-action-result-pathinfo-v1",
        "store_path": path_info.store_path.to_absolute_path_with_prefix(input.store_dir),
        "object_ref": input.object_ref,
        "references": references,
        "nar_size": path_info.nar_size,
        "nar_sha256": data_encoding::HEXLOWER.encode(&path_info.nar_sha256),
        "signatures": signatures,
        "deriver": path_info.deriver.as_ref().map(|path| path.to_absolute_path_with_prefix(input.store_dir)),
    }))
    .map_err(|error| format!("action-result-pathinfo-json:{error}"))?;
    Ok(domain_ref(PATH_INFO_REF_PREFIX, PATH_INFO_DOMAIN, &bytes))
}

fn output_signature_refs(outputs: &BTreeMap<String, PathInfo>) -> Vec<String> {
    let mut refs = outputs
        .values()
        .flat_map(|path_info| path_info.signatures.iter())
        .map(|signature| domain_ref(SIGNATURE_REF_PREFIX, SIGNATURE_DOMAIN, signature.to_string().as_bytes()))
        .collect::<Vec<_>>();
    refs.sort();
    refs.dedup();
    refs
}

fn reference_scan_refs(
    outputs: &BTreeMap<String, PathInfo>,
    result_outputs: &[ActionResultOutput],
    store_dir: &str,
) -> Result<Vec<String>, String> {
    let object_refs = result_outputs
        .iter()
        .map(|output| (output.name.as_str(), output.object_ref.as_str()))
        .collect::<BTreeMap<_, _>>();
    let mut refs = Vec::with_capacity(outputs.len());
    for (name, path_info) in outputs {
        let mut references = path_info
            .references
            .iter()
            .map(|reference| reference.to_absolute_path_with_prefix(store_dir))
            .collect::<Vec<_>>();
        references.sort();
        let bytes = serde_json::to_vec(&serde_json::json!({
            "schema": "mantle-action-result-reference-scan-v1",
            "output_name": name,
            "object_ref": object_refs.get(name.as_str()),
            "references": references,
            "status": "accepted",
        }))
        .map_err(|error| format!("action-result-reference-scan-json:{error}"))?;
        refs.push(domain_ref(REFERENCE_SCAN_REF_PREFIX, REFERENCE_SCAN_DOMAIN, &bytes));
    }
    refs.sort();
    refs.dedup();
    debug_assert!(refs.len() <= outputs.len());
    debug_assert!(refs.windows(2).all(|window| window[0] < window[1]));
    Ok(refs)
}

fn action_receipt_ref(input: ActionReceiptRefInput<'_>) -> Result<String, String> {
    let bytes = serde_json::to_vec(&serde_json::json!({
        "schema": "mantle-action-receipt-v1",
        "action_ref": input.action_ref,
        "outputs": input.outputs,
        "reference_scan_refs": input.reference_scan_refs,
        "sandbox_policy_ref": input.policy_refs.sandbox_policy_ref,
        "network_policy_ref": input.policy_refs.network_policy_ref,
        "producer_identity": input.producer_identity,
        "producer_policy_ref": input.policy_refs.producer_policy_ref,
        "publication_policy_ref": input.policy_refs.publication_policy_ref,
        "signature_refs": input.signature_refs,
        "execution_status": "success",
    }))
    .map_err(|error| format!("action-result-receipt-json:{error}"))?;
    Ok(domain_ref(ACTION_RECEIPT_REF_PREFIX, ACTION_RECEIPT_DOMAIN, &bytes))
}

fn verify_record_signatures(signed: &SignedActionResultRecord, trusted_keys: &[VerifyingKey]) -> Vec<String> {
    let mut verified = Vec::with_capacity(signed.record_signatures.len());
    for detached in &signed.record_signatures {
        let Ok(signature) = Signature::<String>::parse(&detached.signature) else {
            continue;
        };
        if signature.name().as_str() != detached.key_name {
            continue;
        }
        let signature_ref = signature.as_ref();
        if trusted_keys.iter().any(|key| key.verify(&signed.record.result_ref, &signature_ref)) {
            verified.push(detached.key_name.clone());
        }
    }
    verified.sort();
    verified.dedup();
    verified
}

fn all_path_info_signatures_verified(
    outputs: &BTreeMap<String, PathInfo>,
    store_dir: &str,
    trusted_keys: &[VerifyingKey],
) -> bool {
    if trusted_keys.is_empty() {
        return false;
    }
    outputs.values().all(|path_info| {
        signing::verify_pathinfo_signatures_with_store_dir(path_info, trusted_keys, store_dir).is_trusted()
    })
}

fn same_object_refs(input: &ActionResultRecordInput, record: &ActionResultRecord) -> bool {
    input
        .outputs
        .iter()
        .map(|output| (&output.name, &output.object_ref))
        .eq(record.outputs.iter().map(|output| (&output.name, &output.object_ref)))
}

fn same_path_info_refs(input: &ActionResultRecordInput, record: &ActionResultRecord) -> bool {
    input
        .outputs
        .iter()
        .map(|output| (&output.name, &output.path_info_ref, &output.store_path))
        .eq(record.outputs.iter().map(|output| (&output.name, &output.path_info_ref, &output.store_path)))
        && input.signature_refs == record.signature_refs
}

fn runtime_non_claims() -> Vec<String> {
    vec![
        NON_CLAIM_CA_MAPPING.to_string(),
        NON_CLAIM_EXECUTOR.to_string(),
        NON_CLAIM_INDEX.to_string(),
    ]
}

fn domain_ref(prefix: &str, domain: &[u8], bytes: &[u8]) -> String {
    format!("{prefix}{}", domain_digest(domain, bytes))
}

fn domain_digest(domain: &[u8], bytes: &[u8]) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(domain);
    hasher.update(&[DOMAIN_SEPARATOR]);
    hasher.update(bytes);
    hasher.finalize().to_hex().to_string()
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;
    use std::num::NonZeroUsize;
    use std::sync::Arc;

    use crunch_store::PipelineStoreParts;
    use crunch_store::StoreHandle;
    use crunch_store::StoreHandleServices;
    use nix_compat::derivation::Output;
    use nix_compat::nixhash::CAHash;
    use nix_compat::nixhash::NixHash;
    use nix_compat::store_path::StorePath;
    use snix_castore::Node;
    use snix_castore::blobservice::BlobService;
    use snix_castore::blobservice::MemoryBlobService;
    use snix_castore::directoryservice::DirectoryService;
    use snix_castore::directoryservice::RedbDirectoryService;
    use snix_castore::directoryservice::RedbDirectoryServiceConfig;
    use snix_store::nar::NarCalculationService;
    use snix_store::nar::SimpleRenderer;
    use snix_store::pathinfoservice::LruPathInfoService;
    use snix_store::pathinfoservice::PathInfoService;
    use tokio::io::AsyncWriteExt;

    use super::*;

    const SOURCE_CLASS_HTTP: &str = "http";
    const SOURCE_CLASS_LOCAL: &str = "local";

    fn derivation() -> Derivation {
        let mut outputs = BTreeMap::new();
        outputs.insert("out".to_string(), Output {
            path: None,
            ca_hash: None,
        });
        Derivation {
            outputs,
            input_derivations: BTreeMap::new(),
            input_sources: BTreeSet::new(),
            system: "x86_64-linux".to_string(),
            builder: "/bin/sh".to_string(),
            arguments: vec!["-c".to_string(), "echo ok > $out".to_string()],
            environment: BTreeMap::from([("name".to_string(), b"demo".to_vec().into())]),
        }
    }

    fn keypair() -> KeyPair {
        let secret = ed25519_dalek::SigningKey::from_bytes(&[17u8; 32]);
        let signing_key = nix_compat::narinfo::SigningKey::new("builder-key-1".to_string(), secret.clone());
        let verifying_key = nix_compat::narinfo::VerifyingKey::new("builder-key-1".to_string(), secret.verifying_key());
        KeyPair {
            signing_key,
            verifying_key,
        }
    }

    fn other_keypair() -> KeyPair {
        let secret = ed25519_dalek::SigningKey::from_bytes(&[23u8; 32]);
        let signing_key = nix_compat::narinfo::SigningKey::new("other-key-1".to_string(), secret.clone());
        let verifying_key = nix_compat::narinfo::VerifyingKey::new("other-key-1".to_string(), secret.verifying_key());
        KeyPair {
            signing_key,
            verifying_key,
        }
    }

    fn path_info() -> PathInfo {
        let output_path = StorePath::from_name_and_digest_fixed("demo", [7u8; 20]).unwrap();
        let mut path_info = PathInfo {
            store_path: output_path,
            node: Node::Symlink {
                target: snix_castore::SymlinkTarget::try_from("target").unwrap(),
            },
            references: Vec::new(),
            nar_size: 1,
            nar_sha256: [9u8; 32],
            signatures: Vec::new(),
            deriver: None,
            ca: None,
        };
        signing::sign_pathinfo_with_store_dir(&mut path_info, &keypair().signing_key, "/mantle/store");
        path_info
    }

    #[derive(Clone, Copy)]
    enum SignedCase {
        Valid,
        MissingResult,
        ForgedRecordKey,
        UnresolvedCa,
        AlteredDrvBytes,
        AlteredOutput,
        AmbiguousOutputs,
    }

    struct SignedFixture {
        _temp: tempfile::TempDir,
        parts: PipelineStoreParts,
        drv_path: StorePath<String>,
        output_path: StorePath<String>,
        trusted: Vec<VerifyingKey>,
    }

    impl SignedFixture {
        async fn verified(&self) -> Result<VerifiedStoreDerivation<'_>, SignedDerivationAdmissionError> {
            verify_signed_store_derivation(
                SignedDerivationSource {
                    build_store: &self.parts.build_store,
                    lookup: &self.parts.output_lookup,
                    content: &self.parts.build_service_store,
                    store_dir: "/nix/store",
                    trusted_keys: &self.trusted,
                },
                &self.drv_path,
            )
            .await
        }
    }

    async fn measured_output(
        path: StorePath<String>,
        target: &str,
        signer: &KeyPair,
        renderer: &SimpleRenderer<Arc<dyn BlobService>, Arc<dyn DirectoryService>>,
    ) -> PathInfo {
        let node = Node::Symlink {
            target: snix_castore::SymlinkTarget::try_from(target).unwrap(),
        };
        let (nar_size, nar_sha256) = renderer.calculate_nar(&node).await.unwrap();
        let mut info = PathInfo {
            store_path: path,
            node,
            references: Vec::new(),
            nar_size,
            nar_sha256,
            signatures: Vec::new(),
            deriver: None,
            ca: None,
        };
        signing::sign_pathinfo_with_store_dir(&mut info, &signer.signing_key, "/nix/store");
        info
    }

    async fn signed_fixture(case: SignedCase) -> SignedFixture {
        let temp = tempfile::tempdir().unwrap();
        let blobs = Arc::new(MemoryBlobService::default()) as Arc<dyn BlobService>;
        let directories = Arc::new(
            RedbDirectoryService::new_temporary(
                "signed-derivation-fixture".to_string(),
                RedbDirectoryServiceConfig::default(),
            )
            .unwrap(),
        ) as Arc<dyn DirectoryService>;
        let infos = Arc::new(LruPathInfoService::with_capacity(
            "signed-derivation-fixture".to_string(),
            NonZeroUsize::new(16).unwrap(),
        )) as Arc<dyn PathInfoService>;
        let renderer = SimpleRenderer::new(blobs.clone(), directories.clone());
        let signer = keypair();
        let output_path = StorePath::from_name_and_digest_fixed("demo-output", [7_u8; 20]).unwrap();
        let mut derivation = derivation();
        if matches!(case, SignedCase::UnresolvedCa) {
            derivation.outputs.get_mut("out").unwrap().ca_hash = Some(CAHash::Nar(NixHash::Sha256([0x19_u8; 32])));
        } else {
            derivation.outputs.get_mut("out").unwrap().path = Some(output_path.clone());
        }
        let bytes = derivation.to_aterm_bytes_with_store_dir("/nix/store");
        let mut writer = blobs.open_write().await;
        writer.write_all(&bytes).await.unwrap();
        let digest = writer.close().await.unwrap();
        let drv_node = Node::File {
            digest,
            size: bytes.len() as u64,
            executable: false,
        };
        let (nar_size, nar_sha256) = renderer.calculate_nar(&drv_node).await.unwrap();
        let drv_path = StorePath::from_name_and_digest_fixed("demo.drv", [11_u8; 20]).unwrap();
        let mut drv_info = PathInfo {
            store_path: drv_path.clone(),
            node: drv_node,
            references: Vec::new(),
            nar_size,
            nar_sha256,
            signatures: Vec::new(),
            deriver: None,
            ca: None,
        };
        signing::sign_pathinfo_with_store_dir(&mut drv_info, &signer.signing_key, "/nix/store");
        if matches!(case, SignedCase::AlteredDrvBytes) {
            let altered = b"tampered ATerm bytes";
            let mut writer = blobs.open_write().await;
            writer.write_all(altered).await.unwrap();
            let altered_digest = writer.close().await.unwrap();
            drv_info.node = Node::File {
                digest: altered_digest,
                size: altered.len() as u64,
                executable: false,
            };
        }
        infos.put(drv_info).await.unwrap();

        let mut output = measured_output(output_path.clone(), "real-output", &signer, &renderer).await;
        let other_signer = matches!(case, SignedCase::AmbiguousOutputs).then(other_keypair);
        if let Some(other) = &other_signer {
            signing::sign_pathinfo_with_store_dir(&mut output, &other.signing_key, "/nix/store");
        }
        let original_outputs = BTreeMap::from([("out".to_string(), output.clone())]);
        let signed = (!matches!(case, SignedCase::UnresolvedCa)).then(|| {
            signed_record_for_outputs(&derivation, &original_outputs, "/nix/store", HermeticityMode::Strict, &signer)
                .unwrap()
        });
        let mut visible_output = output;
        if matches!(case, SignedCase::AlteredOutput) {
            visible_output.nar_sha256 = [0x5a_u8; 32];
        }
        infos.put(visible_output).await.unwrap();
        let handle = StoreHandle::from_services_with_store_dir(
            StoreHandleServices {
                blob_service: blobs,
                directory_service: directories,
                pathinfo_service: infos.clone(),
                remote_pathinfo: None,
                state_dir: temp.path().to_path_buf(),
                output_dir_str: temp.path().display().to_string(),
                publishers: Vec::new(),
            },
            "/nix/store".to_string(),
        );
        let parts = handle.into_pipeline_store_parts();
        if !matches!(case, SignedCase::MissingResult)
            && let Some(mut signed) = signed
        {
            if matches!(case, SignedCase::ForgedRecordKey) {
                let imposter = nix_compat::narinfo::SigningKey::new(
                    signer.verifying_key.name().to_string(),
                    ed25519_dalek::SigningKey::from_bytes(&[23_u8; 32]),
                );
                signed.record_signatures = vec![DetachedRecordSignature {
                    key_name: signer.verifying_key.name().to_string(),
                    signature: imposter.sign(signed.record.result_ref.as_bytes()).to_string(),
                }];
            }
            parts.action_results.publish_local(&signed).await.unwrap();
        }
        if let Some(other) = &other_signer {
            let second =
                signed_record_for_outputs(&derivation, &original_outputs, "/nix/store", HermeticityMode::Strict, other)
                    .unwrap();
            parts.action_results.publish_local(&second).await.unwrap();
        }
        let mut trusted = vec![signer.verifying_key];
        if let Some(other) = other_signer {
            trusted.push(other.verifying_key);
        }
        SignedFixture {
            _temp: temp,
            parts,
            drv_path,
            output_path,
            trusted,
        }
    }

    #[tokio::test]
    async fn signed_drv_bytes_and_result_prove_exact_present_output() {
        let fixture = signed_fixture(SignedCase::Valid).await;
        let verified = fixture.verified().await.unwrap();
        assert_eq!(verified.path(), &fixture.drv_path);
        let admitted =
            admit_signed_derivation_outputs(&fixture.parts.action_results, &verified, HermeticityMode::Strict)
                .await
                .unwrap()
                .unwrap();
        assert_eq!(admitted.outputs["out"].store_path, fixture.output_path);
        assert!(!admitted.result_ref.is_empty());
        let no_result = signed_fixture(SignedCase::MissingResult).await;
        let verified = no_result.verified().await.unwrap();
        assert!(
            admit_signed_derivation_outputs(&no_result.parts.action_results, &verified, HermeticityMode::Strict,)
                .await
                .unwrap()
                .is_none()
        );
    }

    #[tokio::test]
    async fn signed_result_rejects_forged_key_output_and_ambiguity() {
        for (case, expected) in [
            (SignedCase::ForgedRecordKey, SignedDerivationAdmissionError::UntrustedResult),
            (SignedCase::AlteredOutput, SignedDerivationAdmissionError::UntrustedResult),
            (SignedCase::AmbiguousOutputs, SignedDerivationAdmissionError::AmbiguousResults),
        ] {
            let fixture = signed_fixture(case).await;
            let verified = fixture.verified().await.unwrap();
            let error =
                admit_signed_derivation_outputs(&fixture.parts.action_results, &verified, HermeticityMode::Strict)
                    .await
                    .err()
                    .expect("forged or conflicting outputs cannot be reused");
            assert_eq!(error, expected);
        }
    }

    #[tokio::test]
    async fn altered_drv_cas_bytes_cannot_reuse_a_signed_action_result() {
        let fixture = signed_fixture(SignedCase::AlteredDrvBytes).await;
        let error = fixture.verified().await.err().expect("signed NAR cannot authorize changed bytes");
        assert_eq!(error, SignedDerivationAdmissionError::UntrustedDerivation);
    }

    #[tokio::test]
    async fn unresolved_ca_output_from_signed_cas_is_rejected_by_strict_aterm_parser() {
        let fixture = signed_fixture(SignedCase::UnresolvedCa).await;
        let error = fixture.verified().await.err().expect("an unresolved CA .drv is not yet parseable");
        assert_eq!(error, SignedDerivationAdmissionError::Identity);
    }

    #[test]
    fn derivation_adapter_record_and_signature_are_deterministic_and_verifiable() {
        let derivation = derivation();
        let outputs = BTreeMap::from([("out".to_string(), path_info())]);
        let keypair = keypair();
        let left = signed_record_for_outputs(&derivation, &outputs, "/mantle/store", HermeticityMode::Strict, &keypair)
            .unwrap();
        let right =
            signed_record_for_outputs(&derivation, &outputs, "/mantle/store", HermeticityMode::Strict, &keypair)
                .unwrap();
        let verified = verify_record_signatures(&left, std::slice::from_ref(&keypair.verifying_key));

        assert_eq!(left, right);
        assert_eq!(verified, vec!["builder-key-1".to_string()]);
        assert_eq!(left.record.outputs.len(), 1);
        assert!(left.record.action_ref.starts_with(ACTION_REF_PREFIX));
        assert!(left.record.non_claims.contains(&NON_CLAIM_CA_MAPPING.to_string()));
    }

    #[test]
    fn record_construction_rejects_output_path_not_declared_by_action() {
        let mut derivation = derivation();
        derivation.outputs.get_mut("out").unwrap().path =
            Some(StorePath::from_name_and_digest_fixed("expected", [11u8; 20]).unwrap());
        let outputs = BTreeMap::from([("out".to_string(), path_info())]);

        let error =
            signed_record_for_outputs(&derivation, &outputs, "/mantle/store", HermeticityMode::Strict, &keypair())
                .unwrap_err();

        assert_eq!(error, "action-result-output-store-path-does-not-match-action");
        assert_ne!(derivation.outputs["out"].path.as_ref(), Some(&outputs["out"].store_path));
    }

    #[test]
    fn candidate_facts_reject_object_pathinfo_receipt_and_signature_drift() {
        let derivation = derivation();
        let outputs = BTreeMap::from([("out".to_string(), path_info())]);
        let keypair = keypair();
        let mut signed =
            signed_record_for_outputs(&derivation, &outputs, "/mantle/store", HermeticityMode::Strict, &keypair)
                .unwrap();
        signed.record.outputs[0].object_ref = domain_ref(OBJECT_REF_PREFIX, OBJECT_DOMAIN, b"poison");
        signed.record.action_receipt_ref = domain_ref(ACTION_RECEIPT_REF_PREFIX, ACTION_RECEIPT_DOMAIN, b"poison");
        signed.record.publication_policy_ref =
            domain_ref(PUBLICATION_POLICY_REF_PREFIX, PUBLICATION_POLICY_DOMAIN, b"poison");
        signed.record_signatures.clear();
        let facts = candidate_admission_facts(
            "poison-source".to_string(),
            SOURCE_CLASS_HTTP.to_string(),
            &signed,
            &derivation,
            Some(&outputs),
            "/mantle/store",
            HermeticityMode::Strict,
            std::slice::from_ref(&keypair.verifying_key),
        );

        assert!(!facts.object_refs_complete);
        assert!(!facts.action_receipt_linked);
        assert!(facts.path_info_refs_linked);
        assert!(facts.path_info_signatures_verified);
        assert!(!facts.publication_policy_admitted);
        assert!(facts.verified_record_signers.is_empty());
    }

    #[test]
    fn trusted_signer_cannot_impersonate_a_different_producer_identity() {
        let derivation = derivation();
        let outputs = BTreeMap::from([("out".to_string(), path_info())]);
        let producer = keypair();
        let other = other_keypair();
        let mut signed =
            signed_record_for_outputs(&derivation, &outputs, "/mantle/store", HermeticityMode::Strict, &producer)
                .unwrap();
        let replacement = other.signing_key.sign(signed.record.result_ref.as_bytes()).to_owned();
        signed.record_signatures = vec![DetachedRecordSignature {
            key_name: other.verifying_key.name().to_string(),
            signature: replacement.to_string(),
        }];
        let facts = candidate_admission_facts(
            "source".to_string(),
            SOURCE_CLASS_LOCAL.to_string(),
            &signed,
            &derivation,
            Some(&outputs),
            "/mantle/store",
            HermeticityMode::Strict,
            &[producer.verifying_key, other.verifying_key],
        );

        assert_eq!(facts.verified_record_signers, vec!["other-key-1".to_string()]);
        assert!(!facts.producer_policy_admitted);
        assert!(facts.path_info_signatures_verified);
    }

    // r[verify mantle.ca_input_resolution.realisation_records]
    // r[verify mantle.ca_input_resolution.negative_controls]
    #[test]
    fn action_result_display_name_cannot_replace_trusted_public_key_material() {
        let derivation = derivation();
        let outputs = BTreeMap::from([("out".to_string(), path_info())]);
        let producer = keypair();
        let mut signed =
            signed_record_for_outputs(&derivation, &outputs, "/mantle/store", HermeticityMode::Strict, &producer)
                .unwrap();
        let other_secret = ed25519_dalek::SigningKey::from_bytes(&[23u8; 32]);
        let same_name_wrong_key =
            nix_compat::narinfo::SigningKey::new(producer.verifying_key.name().to_string(), other_secret);
        signed.record_signatures = vec![DetachedRecordSignature {
            key_name: producer.verifying_key.name().to_string(),
            signature: same_name_wrong_key.sign(signed.record.result_ref.as_bytes()).to_string(),
        }];
        let facts = candidate_admission_facts(
            "source".to_string(),
            SOURCE_CLASS_LOCAL.to_string(),
            &signed,
            &derivation,
            Some(&outputs),
            "/mantle/store",
            HermeticityMode::Strict,
            std::slice::from_ref(&producer.verifying_key),
        );

        assert!(facts.verified_record_signers.is_empty());
        assert!(!facts.producer_policy_admitted);
        assert!(facts.path_info_signatures_verified, "only the record signer was forged");
    }

    #[test]
    fn trust_policy_never_promotes_source_or_ca_mapping_presence() {
        let derivation = derivation();
        let refs = policy_refs_for_derivation(&derivation, "/mantle/store", HermeticityMode::Strict).unwrap();
        let policy = trust_policy_for_action(&refs, std::slice::from_ref(&keypair().verifying_key));

        assert_eq!(policy.schema, ACTION_RESULT_POLICY_SCHEMA);
        assert_eq!(policy.policy_id, ACTION_RESULT_TRUST_POLICY_ID);
        assert_eq!(policy.trusted_producers, vec!["builder-key-1".to_string()]);
        assert!(policy.allowed_source_classes.contains(&SOURCE_CLASS_LOCAL.to_string()));
        assert!(policy.allowed_source_classes.contains(&SOURCE_CLASS_HTTP.to_string()));
        assert_eq!(policy.required_non_claims, runtime_non_claims());
        assert!(policy.require_record_signature);
        assert!(policy.require_path_info_signature);
    }
}
