use std::collections::BTreeMap;
use std::collections::BTreeSet;

use serde::Deserialize;

pub(crate) const EARLY_NATIVE_ROW_RECEIPT_SCHEMA: &str = "mantle-early-native-row-receipt-v2";
const PASSED_STATUS: &str = "passed";
const REGENERATED_STATUS: &str = "regenerated";
const BLAKE3_HEX_LENGTH: usize = 64;
const SHA256_HEX_LENGTH: usize = 64;
const MAX_SOURCE_RECORD_COUNT: usize = 64;
const MAX_PREDECESSOR_COUNT: usize = 32;
const MAX_MATRIX_ITEM_COUNT: usize = 32;
const MAX_GENERATED_ARTIFACT_COUNT: usize = 32;

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct EarlyNativeRowReceipt {
    pub(crate) schema: String,
    pub(crate) row_id: String,
    pub(crate) derivation: String,
    pub(crate) source_records: Vec<SourceRecord>,
    pub(crate) predecessors: Vec<PredecessorRecord>,
    pub(crate) generated_artifacts: Vec<StatusRecord>,
    pub(crate) acceptance_evidence_path: String,
    pub(crate) output: OutputRecord,
    pub(crate) behavior: BehaviorRecord,
    pub(crate) trust: TrustRecord,
    pub(crate) fallback: FallbackRecord,
    pub(crate) bounded_claim: String,
    pub(crate) non_claims: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct SourceRecord {
    pub(crate) path: String,
    pub(crate) blake3: String,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct PredecessorRecord {
    pub(crate) role: String,
    pub(crate) logical_path: String,
    pub(crate) artifact_evidence_path: String,
    pub(crate) artifact_attestation_blake3: String,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct StatusRecord {
    pub(crate) id: String,
    pub(crate) status: String,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct OutputRecord {
    pub(crate) logical_path: String,
    pub(crate) artifact_evidence_path: String,
    pub(crate) artifact_attestation_blake3: String,
    pub(crate) nar_sha256: String,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct BehaviorRecord {
    pub(crate) positive: Vec<StatusRecord>,
    pub(crate) rejection: Vec<StatusRecord>,
}

#[derive(Debug, Clone, Deserialize, Eq, PartialEq)]
pub(crate) struct TrustRecord {
    pub(crate) trust_unsigned: bool,
    pub(crate) signing_key_selected: bool,
    pub(crate) substitutions_enabled: bool,
}

#[derive(Debug, Clone, Deserialize, Eq, PartialEq)]
pub(crate) struct FallbackRecord {
    pub(crate) host_tool_execution: bool,
    pub(crate) predecessor_delegation: bool,
    pub(crate) release_generated_substitution: bool,
    pub(crate) configure_bridge_compiler_use: bool,
    pub(crate) fabricated_objects: Vec<String>,
    pub(crate) omitted_artifacts: Vec<String>,
}

#[derive(Debug, Clone)]
pub(crate) struct RowExpectation {
    pub(crate) row_id: &'static str,
    pub(crate) derivation: &'static str,
    pub(crate) artifact_evidence_path: &'static str,
    pub(crate) acceptance_evidence_path: &'static str,
    pub(crate) success_marker: &'static str,
    pub(crate) source_paths: &'static [&'static str],
    pub(crate) predecessor_roles: &'static [&'static str],
    pub(crate) predecessor_evidence_paths: &'static [&'static str],
    pub(crate) generated_artifacts: &'static [&'static str],
    pub(crate) positive_matrix: &'static [&'static str],
    pub(crate) rejection_matrix: &'static [&'static str],
    pub(crate) required_non_claims: &'static [&'static str],
}

#[derive(Debug, Clone)]
pub(crate) struct ObservedArtifact {
    pub(crate) row_id: String,
    pub(crate) logical_path: String,
    pub(crate) artifact_attestation_blake3: String,
    pub(crate) nar_sha256: String,
}

#[derive(Debug, Clone)]
pub(crate) struct ObservedPredecessor {
    pub(crate) role: String,
    pub(crate) logical_path: String,
    pub(crate) artifact_attestation_blake3: String,
}

#[derive(Debug, Clone)]
pub(crate) struct ObservedAcceptance {
    pub(crate) row_id: String,
    pub(crate) logical_path: String,
    pub(crate) success_marker: String,
    pub(crate) positive: BTreeSet<String>,
    pub(crate) rejection: BTreeSet<String>,
    pub(crate) trust: TrustRecord,
    pub(crate) fallback: FallbackRecord,
}

pub(crate) fn parse_receipt(text: &str) -> Result<EarlyNativeRowReceipt, String> {
    if text.trim().is_empty() {
        return Err("early-native row receipt must not be empty".to_string());
    }
    let receipt = serde_json::from_str(text).map_err(|error| format!("parse early-native row receipt: {error}"))?;
    assert!(!text.trim().is_empty());
    Ok(receipt)
}

pub(crate) fn validate_receipt(
    receipt: &EarlyNativeRowReceipt,
    expectation: &RowExpectation,
    observed_sources: &BTreeMap<String, String>,
    observed_predecessors: &BTreeMap<String, ObservedPredecessor>,
    observed_acceptance: &ObservedAcceptance,
    observed_artifact: &ObservedArtifact,
) -> Result<(), String> {
    validate_identity(receipt, expectation)?;
    validate_sources(receipt, expectation, observed_sources)?;
    validate_predecessors(receipt, expectation, observed_predecessors)?;
    validate_status_records(
        "generated_artifacts",
        &receipt.generated_artifacts,
        expectation.generated_artifacts,
        REGENERATED_STATUS,
        MAX_GENERATED_ARTIFACT_COUNT,
    )?;
    validate_output(receipt, expectation, observed_artifact)?;
    validate_status_records(
        "behavior.positive",
        &receipt.behavior.positive,
        expectation.positive_matrix,
        PASSED_STATUS,
        MAX_MATRIX_ITEM_COUNT,
    )?;
    validate_status_records(
        "behavior.rejection",
        &receipt.behavior.rejection,
        expectation.rejection_matrix,
        PASSED_STATUS,
        MAX_MATRIX_ITEM_COUNT,
    )?;
    validate_acceptance(receipt, expectation, observed_acceptance)?;
    validate_claims(receipt, expectation)?;
    assert_eq!(receipt.row_id, expectation.row_id);
    assert_eq!(receipt.output.logical_path, observed_artifact.logical_path);
    Ok(())
}

fn validate_identity(receipt: &EarlyNativeRowReceipt, expectation: &RowExpectation) -> Result<(), String> {
    require_equal("schema", &receipt.schema, EARLY_NATIVE_ROW_RECEIPT_SCHEMA)?;
    require_equal("row_id", &receipt.row_id, expectation.row_id)?;
    require_equal("derivation", &receipt.derivation, expectation.derivation)?;
    if receipt.bounded_claim.trim().is_empty() {
        return Err("bounded_claim must not be empty".to_string());
    }
    assert_eq!(receipt.schema, EARLY_NATIVE_ROW_RECEIPT_SCHEMA);
    assert!(!receipt.bounded_claim.trim().is_empty());
    Ok(())
}

fn validate_sources(
    receipt: &EarlyNativeRowReceipt,
    expectation: &RowExpectation,
    observed: &BTreeMap<String, String>,
) -> Result<(), String> {
    require_bounded_non_empty("source_records", receipt.source_records.len(), MAX_SOURCE_RECORD_COUNT)?;
    let expected_paths = expected_set("source paths", expectation.source_paths)?;
    let mut receipted = BTreeSet::new();
    for record in &receipt.source_records {
        validate_lower_hex("source_records.blake3", &record.blake3, BLAKE3_HEX_LENGTH)?;
        if !receipted.insert(record.path.as_str()) {
            return Err(format!("duplicate source record `{}`", record.path));
        }
        let observed_digest = observed
            .get(&record.path)
            .ok_or_else(|| format!("source record `{}` was not independently observed", record.path))?;
        require_equal("source record BLAKE3", &record.blake3, observed_digest)?;
    }
    let actual_paths = receipted.into_iter().collect::<BTreeSet<_>>();
    let expected_refs = expected_paths.iter().copied().collect::<BTreeSet<_>>();
    if actual_paths != expected_refs {
        return Err("source record path set does not match the row contract".to_string());
    }
    assert_eq!(receipt.source_records.len(), expectation.source_paths.len());
    assert_eq!(observed.len(), expectation.source_paths.len());
    Ok(())
}

fn validate_predecessors(
    receipt: &EarlyNativeRowReceipt,
    expectation: &RowExpectation,
    observed: &BTreeMap<String, ObservedPredecessor>,
) -> Result<(), String> {
    require_bounded_non_empty("predecessors", receipt.predecessors.len(), MAX_PREDECESSOR_COUNT)?;
    let expected_roles = expected_set("predecessor roles", expectation.predecessor_roles)?;
    if expectation.predecessor_roles.len() != expectation.predecessor_evidence_paths.len() {
        return Err("predecessor role and evidence path contracts have different lengths".to_string());
    }
    let expected_paths = expectation
        .predecessor_roles
        .iter()
        .zip(expectation.predecessor_evidence_paths)
        .map(|(role, path)| (*role, *path))
        .collect::<BTreeMap<_, _>>();
    let mut actual_roles = BTreeSet::new();
    for predecessor in &receipt.predecessors {
        if !actual_roles.insert(predecessor.role.as_str()) {
            return Err(format!("duplicate predecessor role `{}`", predecessor.role));
        }
        if !predecessor.logical_path.starts_with("/mantle/store/") {
            return Err(format!("predecessor `{}` has non-Mantle logical path", predecessor.role));
        }
        let expected_path = expected_paths
            .get(predecessor.role.as_str())
            .ok_or_else(|| format!("unexpected predecessor role `{}`", predecessor.role))?;
        require_equal("predecessor artifact evidence path", &predecessor.artifact_evidence_path, expected_path)?;
        validate_lower_hex(
            "predecessors.artifact_attestation_blake3",
            &predecessor.artifact_attestation_blake3,
            BLAKE3_HEX_LENGTH,
        )?;
        let observed_predecessor = observed
            .get(&predecessor.role)
            .ok_or_else(|| format!("predecessor `{}` was not independently observed", predecessor.role))?;
        require_equal("observed predecessor role", &observed_predecessor.role, &predecessor.role)?;
        require_equal("predecessor logical path", &predecessor.logical_path, &observed_predecessor.logical_path)?;
        require_equal(
            "predecessor artifact attestation BLAKE3",
            &predecessor.artifact_attestation_blake3,
            &observed_predecessor.artifact_attestation_blake3,
        )?;
    }
    let expected_refs = expected_roles.iter().copied().collect::<BTreeSet<_>>();
    if actual_roles != expected_refs {
        return Err("predecessor role set does not match the row contract".to_string());
    }
    if observed.len() != expectation.predecessor_roles.len() {
        return Err("observed predecessor set does not match the row contract".to_string());
    }
    assert_eq!(receipt.predecessors.len(), expectation.predecessor_roles.len());
    assert_eq!(observed.len(), expectation.predecessor_roles.len());
    Ok(())
}

fn validate_output(
    receipt: &EarlyNativeRowReceipt,
    expectation: &RowExpectation,
    observed: &ObservedArtifact,
) -> Result<(), String> {
    require_equal(
        "output.artifact_evidence_path",
        &receipt.output.artifact_evidence_path,
        expectation.artifact_evidence_path,
    )?;
    require_equal("artifact evidence row_id", &observed.row_id, expectation.row_id)?;
    require_equal("output logical path", &receipt.output.logical_path, &observed.logical_path)?;
    require_equal(
        "output artifact attestation BLAKE3",
        &receipt.output.artifact_attestation_blake3,
        &observed.artifact_attestation_blake3,
    )?;
    require_equal("output NAR SHA-256", &receipt.output.nar_sha256, &observed.nar_sha256)?;
    validate_lower_hex(
        "output.artifact_attestation_blake3",
        &receipt.output.artifact_attestation_blake3,
        BLAKE3_HEX_LENGTH,
    )?;
    validate_lower_hex("output.nar_sha256", &receipt.output.nar_sha256, SHA256_HEX_LENGTH)?;
    assert!(receipt.output.logical_path.starts_with("/mantle/store/"));
    assert_eq!(receipt.output.artifact_attestation_blake3.len(), BLAKE3_HEX_LENGTH);
    Ok(())
}

fn validate_status_records(
    label: &str,
    records: &[StatusRecord],
    expected_ids: &[&str],
    expected_status: &str,
    count_max: usize,
) -> Result<(), String> {
    require_bounded_non_empty(label, records.len(), count_max)?;
    let expected = expected_set(label, expected_ids)?;
    let mut actual = BTreeSet::new();
    for record in records {
        require_equal(&format!("{label}.status"), &record.status, expected_status)?;
        if !actual.insert(record.id.as_str()) {
            return Err(format!("{label} contains duplicate id `{}`", record.id));
        }
    }
    let expected_refs = expected.iter().copied().collect::<BTreeSet<_>>();
    if actual != expected_refs {
        return Err(format!("{label} id set does not match the row contract"));
    }
    assert_eq!(records.len(), expected_ids.len());
    assert!(records.iter().all(|record| record.status == expected_status));
    Ok(())
}

fn validate_acceptance(
    receipt: &EarlyNativeRowReceipt,
    expectation: &RowExpectation,
    observed: &ObservedAcceptance,
) -> Result<(), String> {
    require_equal("acceptance_evidence_path", &receipt.acceptance_evidence_path, expectation.acceptance_evidence_path)?;
    require_equal("observed acceptance row_id", &observed.row_id, expectation.row_id)?;
    require_equal("observed acceptance logical path", &observed.logical_path, &receipt.output.logical_path)?;
    require_equal("observed acceptance success marker", &observed.success_marker, expectation.success_marker)?;
    let receipted_positive = receipt.behavior.positive.iter().map(|item| item.id.clone()).collect::<BTreeSet<_>>();
    let receipted_rejection = receipt.behavior.rejection.iter().map(|item| item.id.clone()).collect::<BTreeSet<_>>();
    if receipted_positive != observed.positive {
        return Err("observed positive behavior matrix does not match the row receipt".to_string());
    }
    if receipted_rejection != observed.rejection {
        return Err("observed rejection behavior matrix does not match the row receipt".to_string());
    }
    if receipt.trust != observed.trust {
        return Err("observed runtime trust fingerprint does not match the row receipt".to_string());
    }
    if receipt.fallback != observed.fallback {
        return Err("observed fallback scan does not match the row receipt".to_string());
    }
    if observed.trust.trust_unsigned || !observed.trust.signing_key_selected || observed.trust.substitutions_enabled {
        return Err("row was not independently observed under signed no-substitution acceptance".to_string());
    }
    let fallback = &observed.fallback;
    if fallback.host_tool_execution
        || fallback.predecessor_delegation
        || fallback.release_generated_substitution
        || fallback.configure_bridge_compiler_use
        || !fallback.fabricated_objects.is_empty()
        || !fallback.omitted_artifacts.is_empty()
    {
        return Err(
            "independent observation reports a forbidden fallback, substitution, fabrication, or omission".to_string()
        );
    }
    assert_eq!(receipt.trust, observed.trust);
    assert_eq!(receipt.fallback, observed.fallback);
    Ok(())
}

fn validate_claims(receipt: &EarlyNativeRowReceipt, expectation: &RowExpectation) -> Result<(), String> {
    let non_claims = receipt.non_claims.iter().map(String::as_str).collect::<BTreeSet<_>>();
    for required in expectation.required_non_claims {
        if !non_claims.contains(required) {
            return Err(format!("missing required non-claim `{required}`"));
        }
    }
    if receipt.non_claims.len() != non_claims.len() {
        return Err("non_claims contains duplicates".to_string());
    }
    assert!(!receipt.bounded_claim.trim().is_empty());
    assert!(expectation.required_non_claims.iter().all(|item| non_claims.contains(item)));
    Ok(())
}

fn expected_set<'a>(label: &str, values: &'a [&'a str]) -> Result<BTreeSet<&'a str>, String> {
    if values.is_empty() {
        return Err(format!("{label} contract must not be empty"));
    }
    let set = values.iter().copied().collect::<BTreeSet<_>>();
    if set.len() != values.len() {
        return Err(format!("{label} contract contains duplicates"));
    }
    assert!(!set.is_empty());
    assert_eq!(set.len(), values.len());
    Ok(set)
}

fn require_bounded_non_empty(label: &str, count: usize, count_max: usize) -> Result<(), String> {
    if count == 0 || count > count_max {
        return Err(format!("{label} count {count} is outside 1..={count_max}"));
    }
    assert!(count > 0);
    assert!(count <= count_max);
    Ok(())
}

fn validate_lower_hex(label: &str, value: &str, expected_length: usize) -> Result<(), String> {
    let is_lower_hex = value.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte));
    if value.len() != expected_length || !is_lower_hex {
        return Err(format!("{label} must be {expected_length}-character lowercase hex"));
    }
    assert_eq!(value.len(), expected_length);
    assert!(is_lower_hex);
    Ok(())
}

fn require_equal(label: &str, actual: &str, expected: &str) -> Result<(), String> {
    if actual != expected {
        return Err(format!("{label} is `{actual}`, expected `{expected}`"));
    }
    assert_eq!(actual, expected);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const SOURCES: &[&str] = &["bootstrap/root.ncl"];
    const PREDECESSORS: &[&str] = &["compiler"];
    const PREDECESSOR_EVIDENCE: &[&str] = &["bootstrap/evidence/predecessor-compiler.json"];
    const GENERATED: &[&str] = &["parser"];
    const POSITIVE: &[&str] = &["happy"];
    const REJECTION: &[&str] = &["malformed"];
    const NON_CLAIMS: &[&str] = &["general correctness"];
    const DIGEST: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    const NAR_DIGEST: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";

    fn expectation() -> RowExpectation {
        RowExpectation {
            row_id: "row-a",
            derivation: "bootstrap/root.ncl",
            artifact_evidence_path: "bootstrap/evidence/row-a.json",
            acceptance_evidence_path: "bootstrap/evidence/row-a-acceptance.json",
            success_marker: "row-a matrix passed",
            source_paths: SOURCES,
            predecessor_roles: PREDECESSORS,
            predecessor_evidence_paths: PREDECESSOR_EVIDENCE,
            generated_artifacts: GENERATED,
            positive_matrix: POSITIVE,
            rejection_matrix: REJECTION,
            required_non_claims: NON_CLAIMS,
        }
    }

    fn receipt() -> EarlyNativeRowReceipt {
        parse_receipt(&format!(
            r#"{{
              "schema":"{EARLY_NATIVE_ROW_RECEIPT_SCHEMA}",
              "row_id":"row-a",
              "derivation":"bootstrap/root.ncl",
              "source_records":[{{"path":"bootstrap/root.ncl","blake3":"{DIGEST}"}}],
              "predecessors":[{{"role":"compiler","logical_path":"/mantle/store/compiler","artifact_evidence_path":"bootstrap/evidence/predecessor-compiler.json","artifact_attestation_blake3":"{DIGEST}"}}],
              "generated_artifacts":[{{"id":"parser","status":"regenerated"}}],
              "acceptance_evidence_path":"bootstrap/evidence/row-a-acceptance.json",
              "output":{{"logical_path":"/mantle/store/output","artifact_evidence_path":"bootstrap/evidence/row-a.json","artifact_attestation_blake3":"{DIGEST}","nar_sha256":"{NAR_DIGEST}"}},
              "behavior":{{"positive":[{{"id":"happy","status":"passed"}}],"rejection":[{{"id":"malformed","status":"passed"}}]}},
              "trust":{{"trust_unsigned":false,"signing_key_selected":true,"substitutions_enabled":false}},
              "fallback":{{"host_tool_execution":false,"predecessor_delegation":false,"release_generated_substitution":false,"configure_bridge_compiler_use":false,"fabricated_objects":[],"omitted_artifacts":[]}},
              "bounded_claim":"bounded row behavior",
              "non_claims":["general correctness"]
            }}"#
        ))
        .unwrap()
    }

    fn observed_sources() -> BTreeMap<String, String> {
        BTreeMap::from([("bootstrap/root.ncl".to_string(), DIGEST.to_string())])
    }

    fn observed_predecessors() -> BTreeMap<String, ObservedPredecessor> {
        BTreeMap::from([("compiler".to_string(), ObservedPredecessor {
            role: "compiler".to_string(),
            logical_path: "/mantle/store/compiler".to_string(),
            artifact_attestation_blake3: DIGEST.to_string(),
        })])
    }

    fn observed_acceptance() -> ObservedAcceptance {
        ObservedAcceptance {
            row_id: "row-a".to_string(),
            logical_path: "/mantle/store/output".to_string(),
            success_marker: "row-a matrix passed".to_string(),
            positive: BTreeSet::from(["happy".to_string()]),
            rejection: BTreeSet::from(["malformed".to_string()]),
            trust: TrustRecord {
                trust_unsigned: false,
                signing_key_selected: true,
                substitutions_enabled: false,
            },
            fallback: FallbackRecord {
                host_tool_execution: false,
                predecessor_delegation: false,
                release_generated_substitution: false,
                configure_bridge_compiler_use: false,
                fabricated_objects: Vec::new(),
                omitted_artifacts: Vec::new(),
            },
        }
    }

    fn observed_artifact() -> ObservedArtifact {
        ObservedArtifact {
            row_id: "row-a".to_string(),
            logical_path: "/mantle/store/output".to_string(),
            artifact_attestation_blake3: DIGEST.to_string(),
            nar_sha256: NAR_DIGEST.to_string(),
        }
    }

    #[test]
    fn valid_receipt_accepts_independently_observed_inputs() {
        validate_receipt(
            &receipt(),
            &expectation(),
            &observed_sources(),
            &observed_predecessors(),
            &observed_acceptance(),
            &observed_artifact(),
        )
        .unwrap();
    }

    #[test]
    fn cross_row_receipt_is_rejected() {
        let mut expected = expectation();
        expected.row_id = "row-b";
        let error = validate_receipt(
            &receipt(),
            &expected,
            &observed_sources(),
            &observed_predecessors(),
            &observed_acceptance(),
            &observed_artifact(),
        )
        .unwrap_err();
        assert!(error.contains("row_id"));
    }

    #[test]
    fn source_digest_mismatch_is_rejected() {
        let observed = BTreeMap::from([("bootstrap/root.ncl".to_string(), NAR_DIGEST.to_string())]);
        let error = validate_receipt(
            &receipt(),
            &expectation(),
            &observed,
            &observed_predecessors(),
            &observed_acceptance(),
            &observed_artifact(),
        )
        .unwrap_err();
        assert!(error.contains("source record BLAKE3"));
    }

    #[test]
    fn forbidden_fallback_is_rejected() {
        let mut candidate = receipt();
        candidate.fallback.predecessor_delegation = true;
        let mut observed = observed_acceptance();
        observed.fallback.predecessor_delegation = true;
        let error = validate_receipt(
            &candidate,
            &expectation(),
            &observed_sources(),
            &observed_predecessors(),
            &observed,
            &observed_artifact(),
        )
        .unwrap_err();
        assert!(error.contains("forbidden fallback"));
    }

    #[test]
    fn predecessor_digest_mismatch_is_rejected() {
        let mut observed = observed_predecessors();
        observed.get_mut("compiler").unwrap().artifact_attestation_blake3 = NAR_DIGEST.to_string();
        let error = validate_receipt(
            &receipt(),
            &expectation(),
            &observed_sources(),
            &observed,
            &observed_acceptance(),
            &observed_artifact(),
        )
        .unwrap_err();
        assert!(error.contains("predecessor artifact attestation BLAKE3"));
    }

    #[test]
    fn runtime_trust_fingerprint_mismatch_is_rejected() {
        let mut observed = observed_acceptance();
        observed.trust.trust_unsigned = true;
        let error = validate_receipt(
            &receipt(),
            &expectation(),
            &observed_sources(),
            &observed_predecessors(),
            &observed,
            &observed_artifact(),
        )
        .unwrap_err();
        assert!(error.contains("runtime trust fingerprint"));
    }
}
