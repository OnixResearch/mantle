use std::collections::BTreeSet;

const MAX_CARRIER_ROWS: usize = 128;
const MAX_ADAPTER_FACTS: usize = 256;
const BLAKE3_HEX_LENGTH: usize = 64;
const NON_CLAIM_FRAGMENT: &str = "not release correctness";
const OPACITY_FRAGMENT: &str = "opaque carrier";
const SUPPORTED_ROLES: &[&str] = &[
    "preserves-opaque-release-evidence",
    "preserves-adapter-backed-release-evidence",
];
const OVERCLAIM_FRAGMENTS: &[&str] = &[
    "proves release correctness",
    "proves artifact correctness",
    "proves deployment safety",
    "proves full reproducibility",
    "proves semantic correctness",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PreservesCarrierRow {
    pub carrier_id: String,
    pub role: String,
    pub schema_id: String,
    pub payload_digest_blake3: String,
    pub canonical_digest_blake3: String,
    pub adapter_id: Option<String>,
    pub adapter_digest_blake3: Option<String>,
    pub adapter_facts: Vec<String>,
    pub non_claims: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PreservesCarrierValidationInput {
    pub carriers: Vec<PreservesCarrierRow>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PreservesCarrierValidationReport {
    pub valid: bool,
    pub diagnostics: Vec<String>,
}

// r[impl mantle.release_provenance.preserves_carriers.contract]
// r[impl mantle.release_provenance.preserves_carriers.validation]
pub(crate) fn validate_preserves_release_carriers(
    input: &PreservesCarrierValidationInput,
) -> PreservesCarrierValidationReport {
    let mut diagnostics = Vec::new();
    if input.carriers.len() > MAX_CARRIER_ROWS {
        diagnostics.push(format!("preserves carrier count exceeds {MAX_CARRIER_ROWS}"));
    }
    let mut carrier_ids = BTreeSet::new();
    for carrier in &input.carriers {
        validate_carrier(carrier, &mut carrier_ids, &mut diagnostics);
    }
    PreservesCarrierValidationReport {
        valid: diagnostics.is_empty(),
        diagnostics,
    }
}

fn validate_carrier(carrier: &PreservesCarrierRow, carrier_ids: &mut BTreeSet<String>, diagnostics: &mut Vec<String>) {
    push_nonempty(&carrier.carrier_id, "carrier_id", diagnostics);
    push_nonempty(&carrier.schema_id, "schema_id", diagnostics);
    validate_role(&carrier.role, diagnostics);
    validate_digest(&carrier.payload_digest_blake3, "payload_digest_blake3", diagnostics);
    validate_digest(&carrier.canonical_digest_blake3, "canonical_digest_blake3", diagnostics);
    validate_adapter_shape(carrier, diagnostics);
    validate_non_claims(&carrier.non_claims, diagnostics);
    if !carrier_ids.insert(carrier.carrier_id.clone()) {
        diagnostics.push(format!("duplicate Preserves carrier id: {}", carrier.carrier_id));
    }
}

fn validate_role(role: &str, diagnostics: &mut Vec<String>) {
    if !SUPPORTED_ROLES.contains(&role) {
        diagnostics.push(format!("unsupported Preserves carrier role: {role}"));
    }
}

fn validate_adapter_shape(carrier: &PreservesCarrierRow, diagnostics: &mut Vec<String>) {
    let adapter_required = carrier.role == "preserves-adapter-backed-release-evidence";
    if adapter_required {
        if carrier.adapter_id.as_deref().unwrap_or_default().is_empty() {
            diagnostics.push("adapter-backed carrier missing adapter_id".to_string());
        }
        match &carrier.adapter_digest_blake3 {
            Some(digest) => validate_digest(digest, "adapter_digest_blake3", diagnostics),
            None => diagnostics.push("adapter-backed carrier missing adapter_digest_blake3".to_string()),
        }
        if carrier.adapter_facts.is_empty() {
            diagnostics.push("adapter-backed carrier missing adapter_facts".to_string());
        }
    }
    if !adapter_required && (carrier.adapter_id.is_some() || carrier.adapter_digest_blake3.is_some()) {
        diagnostics.push("opaque Preserves carrier must not claim adapter-backed facts".to_string());
    }
    if carrier.adapter_facts.len() > MAX_ADAPTER_FACTS {
        diagnostics.push(format!("Preserves adapter fact count exceeds {MAX_ADAPTER_FACTS}"));
    }
}

fn validate_non_claims(non_claims: &[String], diagnostics: &mut Vec<String>) {
    let has_non_claim = non_claims.iter().any(|value| value.contains(NON_CLAIM_FRAGMENT));
    let has_opacity = non_claims.iter().any(|value| value.contains(OPACITY_FRAGMENT));
    if !has_non_claim || !has_opacity {
        diagnostics.push("Preserves carrier missing opacity/release-correctness non-claims".to_string());
    }
    for value in non_claims {
        push_no_overclaim(value, "non_claims", diagnostics);
    }
}

fn validate_digest(value: &str, field_name: &str, diagnostics: &mut Vec<String>) {
    if value.len() != BLAKE3_HEX_LENGTH || !value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        diagnostics.push(format!("{field_name} is not BLAKE3 hex"));
    }
}

fn push_nonempty(value: &str, field_name: &str, diagnostics: &mut Vec<String>) {
    if value.trim().is_empty() {
        diagnostics.push(format!("{field_name} must not be empty"));
    }
}

fn push_no_overclaim(value: &str, field_name: &str, diagnostics: &mut Vec<String>) {
    let lower = value.to_ascii_lowercase();
    for fragment in OVERCLAIM_FRAGMENTS {
        if lower.contains(fragment) {
            diagnostics.push(format!("{field_name} contains overclaim fragment {fragment:?}"));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_opaque_and_adapter_backed_preserves_carriers() {
        let input = PreservesCarrierValidationInput {
            carriers: vec![opaque_carrier(), adapter_backed_carrier()],
        };

        let report = validate_preserves_release_carriers(&input);

        assert!(report.valid);
        assert!(report.diagnostics.is_empty());
    }

    #[test]
    fn rejects_invalid_preserves_carrier_fixture_matrix() {
        let cases: [(&str, fn(&mut PreservesCarrierRow), &str); 5] = [
            ("stale digest", stale_digest, "BLAKE3"),
            ("wrong schema", wrong_schema, "schema_id"),
            ("wrong role", wrong_role, "unsupported"),
            ("missing non-claims", missing_non_claims, "non-claims"),
            ("semantic overclaim", semantic_overclaim, "overclaim"),
        ];

        for (name, mutate, expected) in cases {
            let mut carrier = adapter_backed_carrier();
            mutate(&mut carrier);
            let input = PreservesCarrierValidationInput {
                carriers: vec![carrier],
            };

            let report = validate_preserves_release_carriers(&input);

            assert!(!report.valid, "{name} should be rejected");
            assert!(
                report.diagnostics.iter().any(|diagnostic| diagnostic.contains(expected)),
                "{name} diagnostics should mention {expected:?}: {:?}",
                report.diagnostics
            );
        }
    }

    fn opaque_carrier() -> PreservesCarrierRow {
        PreservesCarrierRow {
            carrier_id: "opaque-preserves-v1".to_string(),
            role: "preserves-opaque-release-evidence".to_string(),
            schema_id: "preserves.release.evidence.v1".to_string(),
            payload_digest_blake3: digest('a'),
            canonical_digest_blake3: digest('b'),
            adapter_id: None,
            adapter_digest_blake3: None,
            adapter_facts: Vec::new(),
            non_claims: standard_non_claims(),
        }
    }

    fn adapter_backed_carrier() -> PreservesCarrierRow {
        PreservesCarrierRow {
            carrier_id: "adapter-preserves-v1".to_string(),
            role: "preserves-adapter-backed-release-evidence".to_string(),
            schema_id: "preserves.release.evidence.v1".to_string(),
            payload_digest_blake3: digest('c'),
            canonical_digest_blake3: digest('d'),
            adapter_id: Some("preserves-json-adapter-v1".to_string()),
            adapter_digest_blake3: Some(digest('e')),
            adapter_facts: vec!["field-map=release-targets".to_string()],
            non_claims: standard_non_claims(),
        }
    }

    fn standard_non_claims() -> Vec<String> {
        vec![
            "opaque carrier: Preserves bytes are release evidence payload material".to_string(),
            "not release correctness".to_string(),
            "not artifact correctness".to_string(),
        ]
    }

    fn digest(ch: char) -> String {
        ch.to_string().repeat(BLAKE3_HEX_LENGTH)
    }

    fn stale_digest(carrier: &mut PreservesCarrierRow) {
        carrier.payload_digest_blake3 = "stale".to_string();
    }

    fn wrong_schema(carrier: &mut PreservesCarrierRow) {
        carrier.schema_id.clear();
    }

    fn wrong_role(carrier: &mut PreservesCarrierRow) {
        carrier.role = "preserves-proof-of-correctness".to_string();
    }

    fn missing_non_claims(carrier: &mut PreservesCarrierRow) {
        carrier.non_claims.clear();
    }

    fn semantic_overclaim(carrier: &mut PreservesCarrierRow) {
        carrier.non_claims.push("proves release correctness".to_string());
    }
}
