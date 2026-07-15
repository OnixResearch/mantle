use std::collections::BTreeSet;

const MAX_NIX_EVIDENCE_ROWS: usize = 256;
const STORE_HASH_LENGTH: usize = 32;
const BLAKE3_HEX_LENGTH: usize = 64;
const NIX_HASH_ALPHABET: &str = "0123456789abcdfghijklmnpqrsvwxyz";
const REQUIRED_NON_CLAIM: &str = "not build correctness";
const NIX_BOUNDARY_NON_CLAIM: &str = "Nix evidence is realization identity only";
const SUPPORTED_ROLES: &[&str] = &[
    "mantle-build-output",
    "release-bundle-output",
    "nix-realization-sidecar",
    "external-nix-realization",
];
const SUPPORTED_ADAPTERS: &[&str] = &[
    "mantle-build-report",
    "release-provenance-row",
    "cairn-nix-gate",
    "molten-promotion-evidence",
    "valence-provenance-input",
];
const OVERCLAIM_FRAGMENTS: &[&str] = &[
    "proves build correctness",
    "proves evaluation correctness",
    "proves release correctness",
    "proves semantic correctness",
    "proves substituter trust",
];

const _: () = {
    assert!(MAX_NIX_EVIDENCE_ROWS > 0);
    assert!(STORE_HASH_LENGTH > 0);
    assert!(BLAKE3_HEX_LENGTH == blake3::OUT_LEN.saturating_mul(2));
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NixStorePathRef {
    pub store_prefix: String,
    pub logical_path: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NixEvidenceRow {
    pub row_id: String,
    pub adapter_kind: String,
    pub store_path: NixStorePathRef,
    pub derivation_identity: String,
    pub output_name: String,
    pub expected_output_name: String,
    pub realization_role: String,
    pub artifact_digest_blake3: String,
    pub measured_artifact_digest_blake3: String,
    pub derivation_supported: bool,
    pub caveats: Vec<String>,
    pub non_claims: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NixEvidenceInput {
    pub rows: Vec<NixEvidenceRow>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NixEvidenceReport {
    pub valid: bool,
    pub diagnostics: Vec<String>,
}

// r[impl mantle.release_provenance.nix_evidence_core.contract]
// r[impl mantle.release_provenance.nix_evidence_core.validation]
// r[impl mantle.release_provenance.nix_evidence_core.adapters]
pub(crate) fn validate_nix_evidence(input: &NixEvidenceInput) -> NixEvidenceReport {
    let mut diagnostics = Vec::new();
    if input.rows.len() > MAX_NIX_EVIDENCE_ROWS {
        diagnostics.push(format!("Nix evidence row count exceeds {MAX_NIX_EVIDENCE_ROWS}"));
    }
    let mut row_ids = BTreeSet::new();
    for row in &input.rows {
        validate_row(row, &mut row_ids, &mut diagnostics);
    }
    NixEvidenceReport {
        valid: diagnostics.is_empty(),
        diagnostics,
    }
}

fn validate_row(row: &NixEvidenceRow, row_ids: &mut BTreeSet<String>, diagnostics: &mut Vec<String>) {
    let row_id_count_before = row_ids.len();
    let diagnostic_count_before = diagnostics.len();
    push_nonempty(&row.row_id, "row_id", diagnostics);
    validate_adapter(&row.adapter_kind, diagnostics);
    validate_store_path(&row.store_path, diagnostics);
    push_nonempty(&row.derivation_identity, "derivation_identity", diagnostics);
    validate_output_name(row, diagnostics);
    validate_role(&row.realization_role, diagnostics);
    validate_digest(&row.artifact_digest_blake3, "artifact_digest_blake3", diagnostics);
    validate_digest(&row.measured_artifact_digest_blake3, "measured_artifact_digest_blake3", diagnostics);
    if row.artifact_digest_blake3 != row.measured_artifact_digest_blake3 {
        diagnostics.push("artifact digest mismatch".to_string());
    }
    if !row.derivation_supported {
        diagnostics.push("unsupported derivation identity".to_string());
    }
    validate_caveats(&row.caveats, diagnostics);
    validate_non_claims(&row.non_claims, diagnostics);
    if !row_ids.insert(row.row_id.clone()) {
        diagnostics.push(format!("duplicate Nix evidence row id: {}", row.row_id));
    }
    debug_assert!(diagnostics.len() >= diagnostic_count_before);
    debug_assert!(row_ids.len() >= row_id_count_before);
    debug_assert!(row_ids.len() <= row_id_count_before.saturating_add(1));
}

fn validate_store_path(path_ref: &NixStorePathRef, diagnostics: &mut Vec<String>) {
    debug_assert!(!NIX_HASH_ALPHABET.is_empty());
    push_nonempty(&path_ref.store_prefix, "store_prefix", diagnostics);
    push_nonempty(&path_ref.logical_path, "logical_path", diagnostics);
    if !path_ref.store_prefix.starts_with('/') {
        diagnostics.push("store_prefix must be absolute".to_string());
    }
    let expected_prefix = format!("{}/", path_ref.store_prefix.trim_end_matches('/'));
    if !path_ref.logical_path.starts_with(&expected_prefix) {
        diagnostics.push("logical_path does not use store_prefix".to_string());
        return;
    }
    let basename = &path_ref.logical_path[expected_prefix.len()..];
    let Some((hash, name)) = basename.split_once('-') else {
        diagnostics.push("logical_path missing store hash/name separator".to_string());
        return;
    };
    if hash.len() != STORE_HASH_LENGTH || !hash.chars().all(|ch| NIX_HASH_ALPHABET.contains(ch)) {
        diagnostics.push("logical_path has malformed Nix store hash".to_string());
    }
    if name.trim().is_empty() || name.contains('/') || name.contains("..") {
        diagnostics.push("logical_path has unsafe store name".to_string());
    }
}

fn validate_adapter(adapter: &str, diagnostics: &mut Vec<String>) {
    if !SUPPORTED_ADAPTERS.contains(&adapter) {
        diagnostics.push(format!("unsupported Nix evidence adapter: {adapter}"));
    }
}

fn validate_output_name(row: &NixEvidenceRow, diagnostics: &mut Vec<String>) {
    push_nonempty(&row.output_name, "output_name", diagnostics);
    push_nonempty(&row.expected_output_name, "expected_output_name", diagnostics);
    if row.output_name != row.expected_output_name {
        diagnostics.push(format!("wrong output name: expected {}, got {}", row.expected_output_name, row.output_name));
    }
}

fn validate_role(role: &str, diagnostics: &mut Vec<String>) {
    if !SUPPORTED_ROLES.contains(&role) {
        diagnostics.push(format!("ambiguous or unsupported realization role: {role}"));
    }
}

fn validate_caveats(caveats: &[String], diagnostics: &mut Vec<String>) {
    if caveats.is_empty() {
        diagnostics.push("Nix evidence caveats must not be empty".to_string());
    }
    for caveat in caveats {
        push_nonempty(caveat, "caveats", diagnostics);
    }
}

fn validate_non_claims(non_claims: &[String], diagnostics: &mut Vec<String>) {
    let has_build_non_claim = non_claims.iter().any(|value| value.contains(REQUIRED_NON_CLAIM));
    let has_identity_boundary = non_claims.iter().any(|value| value.contains(NIX_BOUNDARY_NON_CLAIM));
    if !has_build_non_claim || !has_identity_boundary {
        diagnostics.push("Nix evidence missing realization-identity non-claims".to_string());
    }
    for non_claim in non_claims {
        push_no_overclaim(non_claim, "non_claims", diagnostics);
    }
}

fn validate_digest(value: &str, field_name: impl AsRef<str>, diagnostics: &mut Vec<String>) {
    if value.len() != BLAKE3_HEX_LENGTH || !value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        diagnostics.push(format!("{} is not BLAKE3 hex", field_name.as_ref()));
    }
}

fn push_nonempty(value: &str, field_name: impl AsRef<str>, diagnostics: &mut Vec<String>) {
    if value.trim().is_empty() {
        diagnostics.push(format!("{} must not be empty", field_name.as_ref()));
    }
}

fn push_no_overclaim(value: &str, field_name: impl AsRef<str>, diagnostics: &mut Vec<String>) {
    let lower = value.to_ascii_lowercase();
    for fragment in OVERCLAIM_FRAGMENTS {
        if lower.contains(fragment) {
            diagnostics.push(format!("{} contains overclaim fragment {fragment:?}", field_name.as_ref()));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_supported_nix_evidence_adapter_rows() {
        let input = NixEvidenceInput {
            rows: vec![
                row("build-report", "mantle-build-report", "mantle-build-output"),
                row("release-row", "release-provenance-row", "release-bundle-output"),
                row("cairn-row", "cairn-nix-gate", "external-nix-realization"),
                row("molten-row", "molten-promotion-evidence", "nix-realization-sidecar"),
                row("valence-row", "valence-provenance-input", "external-nix-realization"),
            ],
        };

        let report = validate_nix_evidence(&input);

        assert!(report.valid);
        assert!(report.diagnostics.is_empty());
    }

    #[test]
    fn rejects_invalid_nix_evidence_fixture_matrix() {
        let cases: [(&str, fn(&mut NixEvidenceRow), &str); 8] = [
            ("malformed store path", malformed_store_path, "store hash"),
            ("wrong output", wrong_output, "wrong output"),
            ("digest mismatch", digest_mismatch, "digest mismatch"),
            ("unsupported derivation", unsupported_derivation, "unsupported derivation"),
            ("missing caveats", missing_caveats, "caveats"),
            ("ambiguous role", ambiguous_role, "role"),
            ("missing non claims", missing_non_claims, "non-claims"),
            ("semantic overclaim", semantic_overclaim, "overclaim"),
        ];

        for (name, mutate, expected) in cases {
            let mut candidate = row("invalid-row", "mantle-build-report", "mantle-build-output");
            mutate(&mut candidate);
            let input = NixEvidenceInput { rows: vec![candidate] };

            let report = validate_nix_evidence(&input);

            assert!(!report.valid, "{name} should fail");
            assert!(
                report.diagnostics.iter().any(|diagnostic| diagnostic.contains(expected)),
                "{name} diagnostics should contain {expected:?}: {:?}",
                report.diagnostics
            );
        }
    }

    fn row(id: &str, adapter: &str, role: &str) -> NixEvidenceRow {
        NixEvidenceRow {
            row_id: id.to_string(),
            adapter_kind: adapter.to_string(),
            store_path: NixStorePathRef {
                store_prefix: "/nix/store".to_string(),
                logical_path: "/nix/store/0123456789abcdfghijklmnpqrsvwxyz-mantle".to_string(),
            },
            derivation_identity: "drv:mantle:out".to_string(),
            output_name: "out".to_string(),
            expected_output_name: "out".to_string(),
            realization_role: role.to_string(),
            artifact_digest_blake3: digest('a'),
            measured_artifact_digest_blake3: digest('a'),
            derivation_supported: true,
            caveats: vec!["identity-only".to_string()],
            non_claims: standard_non_claims(),
        }
    }

    fn standard_non_claims() -> Vec<String> {
        vec![
            "Nix evidence is realization identity only".to_string(),
            "not build correctness".to_string(),
            "not evaluation correctness".to_string(),
        ]
    }

    fn digest(ch: char) -> String {
        ch.to_string().repeat(BLAKE3_HEX_LENGTH)
    }

    fn malformed_store_path(row: &mut NixEvidenceRow) {
        row.store_path.logical_path = "/nix/store/not-a-valid-store-path".to_string();
    }

    fn wrong_output(row: &mut NixEvidenceRow) {
        row.output_name = "dev".to_string();
    }

    fn digest_mismatch(row: &mut NixEvidenceRow) {
        row.measured_artifact_digest_blake3 = digest('b');
    }

    fn unsupported_derivation(row: &mut NixEvidenceRow) {
        row.derivation_supported = false;
    }

    fn missing_caveats(row: &mut NixEvidenceRow) {
        row.caveats.clear();
    }

    fn ambiguous_role(row: &mut NixEvidenceRow) {
        row.realization_role = "realized".to_string();
    }

    fn missing_non_claims(row: &mut NixEvidenceRow) {
        row.non_claims.clear();
    }

    fn semantic_overclaim(row: &mut NixEvidenceRow) {
        row.non_claims.push("proves build correctness".to_string());
    }
}
