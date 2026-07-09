use std::collections::BTreeSet;

const MAX_HANDOFF_ROWS: usize = 128;
const MAX_COVERS: usize = 256;
const BLAKE3_HEX_LENGTH: usize = 64;
const REQUIRED_NON_CLAIM: &str = "not release correctness";
const CAIRN_BOUNDARY_NON_CLAIM: &str = "Cairn owns lifecycle readiness";
const SUPPORTED_ROLES: &[&str] = &[
    "cairn-release-readiness-receipt",
    "cairn-change-validation-receipt",
    "cairn-archive-evidence-index",
];
const SUPPORTED_SCHEMAS: &[&str] = &[
    "cairn.release-readiness.v1",
    "cairn.change-validation.v1",
    "cairn.archive-index.v1",
];
const OVERCLAIM_FRAGMENTS: &[&str] = &[
    "proves release correctness",
    "proves build correctness",
    "proves source correctness",
    "proves artifact correctness",
    "proves deployment safety",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CairnReleaseEvidenceRow {
    pub artifact_id: String,
    pub role: String,
    pub schema_id: String,
    pub artifact_digest_blake3: String,
    pub cairn_policy_digest_blake3: String,
    pub release_readiness_id: String,
    pub covers: Vec<String>,
    pub non_claims: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CairnReleaseEvidenceHandoff {
    pub rows: Vec<CairnReleaseEvidenceRow>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CairnReleaseEvidenceReport {
    pub valid: bool,
    pub diagnostics: Vec<String>,
}

// r[impl mantle.release_provenance.cairn_evidence_handoff.contract]
// r[impl mantle.release_provenance.cairn_evidence_handoff.validation]
pub(crate) fn validate_cairn_release_evidence_handoff(
    handoff: &CairnReleaseEvidenceHandoff,
) -> CairnReleaseEvidenceReport {
    let mut diagnostics = Vec::new();
    if handoff.rows.len() > MAX_HANDOFF_ROWS {
        diagnostics.push(format!("Cairn handoff row count exceeds {MAX_HANDOFF_ROWS}"));
    }
    let mut artifact_ids = BTreeSet::new();
    for row in &handoff.rows {
        validate_row(row, &mut artifact_ids, &mut diagnostics);
    }
    CairnReleaseEvidenceReport {
        valid: diagnostics.is_empty(),
        diagnostics,
    }
}

fn validate_row(row: &CairnReleaseEvidenceRow, artifact_ids: &mut BTreeSet<String>, diagnostics: &mut Vec<String>) {
    push_nonempty(&row.artifact_id, "artifact_id", diagnostics);
    validate_role(&row.role, diagnostics);
    validate_schema(&row.schema_id, diagnostics);
    validate_digest(&row.artifact_digest_blake3, "artifact_digest_blake3", diagnostics);
    validate_digest(&row.cairn_policy_digest_blake3, "cairn_policy_digest_blake3", diagnostics);
    push_nonempty(&row.release_readiness_id, "release_readiness_id", diagnostics);
    validate_covers(&row.covers, diagnostics);
    validate_non_claims(&row.non_claims, diagnostics);
    if !artifact_ids.insert(row.artifact_id.clone()) {
        diagnostics.push(format!("duplicate Cairn handoff artifact id: {}", row.artifact_id));
    }
}

fn validate_role(role: &str, diagnostics: &mut Vec<String>) {
    if !SUPPORTED_ROLES.contains(&role) {
        diagnostics.push(format!("unsupported Cairn handoff role: {role}"));
    }
}

fn validate_schema(schema: &str, diagnostics: &mut Vec<String>) {
    if !SUPPORTED_SCHEMAS.contains(&schema) {
        diagnostics.push(format!("unsupported Cairn handoff schema: {schema}"));
    }
}

fn validate_covers(covers: &[String], diagnostics: &mut Vec<String>) {
    if covers.is_empty() {
        diagnostics.push("Cairn handoff covers must not be empty".to_string());
    }
    if covers.len() > MAX_COVERS {
        diagnostics.push(format!("Cairn handoff covers exceed {MAX_COVERS}"));
    }
    let mut unique = BTreeSet::new();
    for cover in covers {
        push_nonempty(cover, "covers", diagnostics);
        if !unique.insert(cover) {
            diagnostics.push(format!("duplicate Cairn handoff cover id: {cover}"));
        }
    }
}

fn validate_non_claims(non_claims: &[String], diagnostics: &mut Vec<String>) {
    let has_release_non_claim = non_claims.iter().any(|value| value.contains(REQUIRED_NON_CLAIM));
    let has_cairn_boundary = non_claims.iter().any(|value| value.contains(CAIRN_BOUNDARY_NON_CLAIM));
    if !has_release_non_claim || !has_cairn_boundary {
        diagnostics.push("Cairn handoff missing release/Cairn ownership non-claims".to_string());
    }
    for non_claim in non_claims {
        push_no_overclaim(non_claim, "non_claims", diagnostics);
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
    fn accepts_complete_cairn_handoff_fixture() {
        let handoff = CairnReleaseEvidenceHandoff {
            rows: vec![release_readiness_row(), archive_index_row()],
        };

        let report = validate_cairn_release_evidence_handoff(&handoff);

        assert!(report.valid);
        assert!(report.diagnostics.is_empty());
    }

    #[test]
    fn rejects_invalid_cairn_handoff_fixture_matrix() {
        let cases: [(&str, fn(&mut CairnReleaseEvidenceRow), &str); 5] = [
            ("missing artifact", missing_artifact, "artifact_id"),
            ("stale digest", stale_digest, "BLAKE3"),
            ("wrong role", wrong_role, "unsupported"),
            ("wrong schema", wrong_schema, "unsupported"),
            ("weakened non claims", weakened_non_claims, "non-claims"),
        ];

        for (name, mutate, expected) in cases {
            let mut row = release_readiness_row();
            mutate(&mut row);
            let handoff = CairnReleaseEvidenceHandoff { rows: vec![row] };

            let report = validate_cairn_release_evidence_handoff(&handoff);

            assert!(!report.valid, "{name} should fail");
            assert!(
                report.diagnostics.iter().any(|diagnostic| diagnostic.contains(expected)),
                "{name} diagnostics should contain {expected:?}: {:?}",
                report.diagnostics
            );
        }
    }

    fn release_readiness_row() -> CairnReleaseEvidenceRow {
        CairnReleaseEvidenceRow {
            artifact_id: "cairn-release-readiness-main".to_string(),
            role: "cairn-release-readiness-receipt".to_string(),
            schema_id: "cairn.release-readiness.v1".to_string(),
            artifact_digest_blake3: digest('a'),
            cairn_policy_digest_blake3: digest('b'),
            release_readiness_id: "release-2026-07-09".to_string(),
            covers: vec!["mantle.release_provenance.cairn_evidence_handoff.contract".to_string()],
            non_claims: standard_non_claims(),
        }
    }

    fn archive_index_row() -> CairnReleaseEvidenceRow {
        CairnReleaseEvidenceRow {
            artifact_id: "cairn-archive-index".to_string(),
            role: "cairn-archive-evidence-index".to_string(),
            schema_id: "cairn.archive-index.v1".to_string(),
            artifact_digest_blake3: digest('c'),
            cairn_policy_digest_blake3: digest('d'),
            release_readiness_id: "release-2026-07-09".to_string(),
            covers: vec!["mantle.release_provenance.cairn_evidence_handoff.validation".to_string()],
            non_claims: standard_non_claims(),
        }
    }

    fn standard_non_claims() -> Vec<String> {
        vec![
            "Cairn owns lifecycle readiness; Mantle only binds bundle-local evidence".to_string(),
            "not release correctness".to_string(),
            "not build correctness".to_string(),
        ]
    }

    fn digest(ch: char) -> String {
        ch.to_string().repeat(BLAKE3_HEX_LENGTH)
    }

    fn missing_artifact(row: &mut CairnReleaseEvidenceRow) {
        row.artifact_id.clear();
    }

    fn stale_digest(row: &mut CairnReleaseEvidenceRow) {
        row.artifact_digest_blake3 = "bad".to_string();
    }

    fn wrong_role(row: &mut CairnReleaseEvidenceRow) {
        row.role = "mantle-release-correctness-proof".to_string();
    }

    fn wrong_schema(row: &mut CairnReleaseEvidenceRow) {
        row.schema_id = "mantle.release.v1".to_string();
    }

    fn weakened_non_claims(row: &mut CairnReleaseEvidenceRow) {
        row.non_claims = vec!["not release correctness".to_string()];
    }
}
