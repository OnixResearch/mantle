//! Derivation adapter and evidence checks for shared action results.
//!
//! The canonical action-result model is provider-neutral. This module adapts
//! Mantle derivations and admitted PathInfo facts to that model without doing
//! filesystem, network, or executor I/O.

use std::collections::BTreeMap;

use crunch_action_result_core::ACTION_RECEIPT_REF_PREFIX;
use crunch_action_result_core::ACTION_REF_PREFIX;
use crunch_action_result_core::ActionResultOutput;
use crunch_action_result_core::ActionResultRecord;
use crunch_action_result_core::ActionResultRecordInput;
use crunch_action_result_core::ActionResultTrustPolicy;
use crunch_action_result_core::CandidateAdmissionFacts;
use crunch_action_result_core::CandidateDecision;
use crunch_action_result_core::DetachedRecordSignature;
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
use crunch_action_result_core::canonical_action_result;
use nix_compat::derivation::Derivation;
use nix_compat::narinfo::Signature;
use nix_compat::narinfo::VerifyingKey;
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
    Ok(ActionPolicyRefs {
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
    })
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
    let record = canonical_action_result(input)?;
    let signature = keypair.signing_key.sign(record.result_ref.as_bytes()).to_owned();
    Ok(SignedActionResultRecord {
        record,
        record_signatures: vec![DetachedRecordSignature {
            key_name: producer_identity,
            signature: signature.to_string(),
        }],
    })
}

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
    let action_receipt_ref = action_receipt_ref(
        &action_ref,
        &result_outputs,
        &reference_scan_refs,
        &policy_refs,
        &producer_identity,
        &signature_refs,
    )?;
    Ok(ActionResultRecordInput {
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
    })
}

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
    let path_info_signatures_verified =
        outputs.map(|outputs| all_path_info_signatures_verified(outputs, trusted_keys)).unwrap_or(false);
    let producer_policy_admitted = expected.as_ref().is_some_and(|input| {
        input.producer_policy_ref == signed.record.producer_policy_ref
            && input.producer_identity == signed.record.producer_identity
            && verified_record_signers.contains(&signed.record.producer_identity)
    });
    CandidateAdmissionFacts {
        source_id,
        source_class,
        verified_record_signers,
        action_receipt_linked: expected
            .as_ref()
            .is_some_and(|input| input.action_receipt_ref == signed.record.action_receipt_ref),
        object_refs_complete: expected.as_ref().is_some_and(|input| same_object_refs(input, &signed.record)),
        path_info_refs_linked: expected.as_ref().is_some_and(|input| same_path_info_refs(input, &signed.record)),
        path_info_signatures_verified,
        producer_policy_admitted,
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
    }
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
    ActionResultTrustPolicy {
        trusted_producers: signer_names.clone(),
        trusted_record_signers: signer_names,
        allowed_source_classes: crunch_store::action_result_runtime_policy().sources.allowed_classes.clone(),
        allowed_sandbox_policy_refs: vec![policy_refs.sandbox_policy_ref.clone()],
        allowed_network_policy_refs: vec![policy_refs.network_policy_ref.clone()],
        ..ActionResultTrustPolicy::default()
    }
}

pub fn discovery_runtime_report(
    action_ref: String,
    disposition: &str,
    plan: crunch_action_result_core::StrongReusePlan,
    selected_source: Option<(String, String)>,
    transfer: Option<ActionResultTransferEvidence>,
    diagnostics: Vec<String>,
) -> ActionResultRuntimeReport {
    let trust_basis = plan
        .candidate_decisions
        .iter()
        .find(|decision| Some(&decision.result_ref) == plan.selected_result_ref.as_ref())
        .map(|decision| decision.trust_basis.clone())
        .unwrap_or_default();
    ActionResultRuntimeReport {
        schema: ACTION_RESULT_RUNTIME_REPORT_SCHEMA.to_string(),
        phase: ACTION_RESULT_PHASE_DISCOVERY.to_string(),
        action_ref,
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
    }
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
            path_info_ref: path_info_ref(path_info, &object_ref, store_dir)?,
        });
    }
    Ok(result)
}

fn object_ref_for_node(node: &Node) -> Result<String, String> {
    let bytes = serde_json::to_vec(node).map_err(|error| format!("action-result-object-json:{error}"))?;
    Ok(domain_ref(OBJECT_REF_PREFIX, OBJECT_DOMAIN, &bytes))
}

fn path_info_ref(path_info: &PathInfo, object_ref: &str, store_dir: &str) -> Result<String, String> {
    let mut references = path_info
        .references
        .iter()
        .map(|reference| reference.to_absolute_path_with_prefix(store_dir))
        .collect::<Vec<_>>();
    references.sort();
    let mut signatures = path_info.signatures.iter().map(ToString::to_string).collect::<Vec<_>>();
    signatures.sort();
    let bytes = serde_json::to_vec(&serde_json::json!({
        "schema": "mantle-action-result-pathinfo-v1",
        "store_path": path_info.store_path.to_absolute_path_with_prefix(store_dir),
        "object_ref": object_ref,
        "references": references,
        "nar_size": path_info.nar_size,
        "nar_sha256": data_encoding::HEXLOWER.encode(&path_info.nar_sha256),
        "signatures": signatures,
        "deriver": path_info.deriver.as_ref().map(|path| path.to_absolute_path_with_prefix(store_dir)),
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
    Ok(refs)
}

fn action_receipt_ref(
    action_ref: &str,
    outputs: &[ActionResultOutput],
    reference_scan_refs: &[String],
    policy_refs: &ActionPolicyRefs,
    producer_identity: &str,
    signature_refs: &[String],
) -> Result<String, String> {
    let bytes = serde_json::to_vec(&serde_json::json!({
        "schema": "mantle-action-receipt-v1",
        "action_ref": action_ref,
        "outputs": outputs,
        "reference_scan_refs": reference_scan_refs,
        "sandbox_policy_ref": policy_refs.sandbox_policy_ref,
        "network_policy_ref": policy_refs.network_policy_ref,
        "producer_identity": producer_identity,
        "producer_policy_ref": policy_refs.producer_policy_ref,
        "publication_policy_ref": policy_refs.publication_policy_ref,
        "signature_refs": signature_refs,
        "execution_status": "success",
    }))
    .map_err(|error| format!("action-result-receipt-json:{error}"))?;
    Ok(domain_ref(ACTION_RECEIPT_REF_PREFIX, ACTION_RECEIPT_DOMAIN, &bytes))
}

fn verify_record_signatures(signed: &SignedActionResultRecord, trusted_keys: &[VerifyingKey]) -> Vec<String> {
    let mut verified = Vec::new();
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

fn all_path_info_signatures_verified(outputs: &BTreeMap<String, PathInfo>, trusted_keys: &[VerifyingKey]) -> bool {
    if trusted_keys.is_empty() {
        return false;
    }
    outputs
        .values()
        .all(|path_info| signing::verify_pathinfo_signatures(path_info, trusted_keys).is_trusted())
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

    use nix_compat::derivation::Output;
    use nix_compat::store_path::StorePath;
    use snix_castore::Node;

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
        signing::sign_pathinfo(&mut path_info, &keypair().signing_key);
        path_info
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

    #[test]
    fn trust_policy_never_promotes_source_or_ca_mapping_presence() {
        let derivation = derivation();
        let refs = policy_refs_for_derivation(&derivation, "/mantle/store", HermeticityMode::Strict).unwrap();
        let policy = trust_policy_for_action(&refs, std::slice::from_ref(&keypair().verifying_key));

        assert_eq!(policy.trusted_producers, vec!["builder-key-1".to_string()]);
        assert!(policy.allowed_source_classes.contains(&SOURCE_CLASS_LOCAL.to_string()));
        assert!(policy.allowed_source_classes.contains(&SOURCE_CLASS_HTTP.to_string()));
        assert!(policy.required_non_claims.contains(&NON_CLAIM_CA_MAPPING.to_string()));
        assert!(policy.required_non_claims.contains(&NON_CLAIM_INDEX.to_string()));
    }
}
