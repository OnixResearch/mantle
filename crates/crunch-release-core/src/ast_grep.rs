use alloc::collections::BTreeSet;
use alloc::format;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;

use serde::Deserialize;
use serde::Serialize;

use crate::ReleaseEvidenceError;
use crate::manifest::u32_count;
use crate::manifest::validate_blake3_hex;
use crate::manifest::validation_error;

pub const AST_GREP_STRUCTURAL_EVIDENCE_SCHEMA: &str = "mantle-ast-grep-structural-evidence-v1";
pub const AST_GREP_TOOL_PACKAGE: &str = "ast-grep";
pub const AST_GREP_TOOL_VERSION: &str = "0.42.1";
pub const AST_GREP_STRUCTURAL_CLAIM_SCOPE: &str = "structural-tool-evidence-only";
pub const AST_GREP_STRUCTURAL_CLAIM_LABEL: &str = "structural-tool-evidence";
pub const AST_GREP_EXTERNAL_EVIDENCE_ROLE: &str = "ast-grep-structural-evidence";
pub const AST_GREP_NON_CLAIM_SOURCE_BEHAVIOR: &str = "ast-grep.boundary.not-source-behavior";
pub const AST_GREP_NON_CLAIM_BUILD_CORRECTNESS: &str = "ast-grep.boundary.not-build-correctness";
pub const AST_GREP_NON_CLAIM_CACHE_CORRECTNESS: &str = "ast-grep.boundary.not-cache-correctness";
pub const AST_GREP_NON_CLAIM_RELEASE_ELIGIBILITY: &str = "ast-grep.boundary.not-release-eligibility";

const MAX_CLAIM_LABEL_COUNT: u32 = 8;
const MAX_NON_CLAIM_COUNT: u32 = 16;
const MAX_TEXT_BYTES: u32 = 4096;
const MAX_VERSION_BYTES: u32 = 128;
const MAX_FINDING_SUMMARY_COUNT: u32 = 1_000_000;
const SCAN_OUTPUT_FORMATS: &[&str] = &["json-compact", "json-pretty", "json-stream"];
const RULE_TEST_OUTPUT_FORMATS: &[&str] = &["text-color-never"];
const REQUIRED_NON_CLAIMS: &[&str] = &[
    AST_GREP_NON_CLAIM_SOURCE_BEHAVIOR,
    AST_GREP_NON_CLAIM_BUILD_CORRECTNESS,
    AST_GREP_NON_CLAIM_CACHE_CORRECTNESS,
    AST_GREP_NON_CLAIM_RELEASE_ELIGIBILITY,
];
const OVERCLAIM_FRAGMENTS: &[&str] = &[
    "proves source behavior",
    "proves build correctness",
    "proves cache correctness",
    "proves release eligibility",
    "release-eligible",
];

struct IdentityPair<'a> {
    actual: &'a str,
    expected: &'a str,
    field_name: &'static str,
}

struct BoundedTextField<'a> {
    value: &'a str,
    field_name: &'static str,
    maximum_bytes: u32,
}

struct LiteralField<'a> {
    value: &'a str,
    expected: &'static str,
    field_name: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AstGrepCommandKind {
    Scan,
    RuleTest,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AstGrepToolIdentity {
    pub package: String,
    pub version: String,
    pub expected_version: String,
    pub binary_digest_blake3: String,
    pub expected_binary_digest_blake3: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AstGrepCommandIdentity {
    pub kind: AstGrepCommandKind,
    pub argv_digest_blake3: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AstGrepRuleBundleIdentity {
    pub digest_blake3: String,
    pub expected_digest_blake3: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AstGrepScanScope {
    pub description: String,
    pub input_digest_blake3: String,
    pub expected_input_digest_blake3: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AstGrepReceiptIdentity {
    pub digest_blake3: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AstGrepFindingSummary {
    pub rule_count: u32,
    pub finding_count: u32,
    pub test_case_count: u32,
    pub passed_test_count: u32,
    pub failed_test_count: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AstGrepOutputEvidence {
    pub format: String,
    pub finding_summary: AstGrepFindingSummary,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AstGrepStructuralEvidence {
    pub schema: String,
    pub claim_scope: String,
    pub claim_labels: Vec<String>,
    pub tool: AstGrepToolIdentity,
    pub command: AstGrepCommandIdentity,
    pub rule_bundle: AstGrepRuleBundleIdentity,
    pub scan_scope: AstGrepScanScope,
    pub output: AstGrepOutputEvidence,
    pub receipt: AstGrepReceiptIdentity,
    pub non_claims: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AstGrepReleaseAttachment {
    pub role: String,
    pub schema: String,
    pub claim_scope: String,
    pub non_claims: Vec<String>,
}

// r[impl mantle.ast_grep_structural_rails.sidecar]
// r[impl mantle.ast_grep_structural_rails.identity]
pub fn parse_ast_grep_structural_evidence_json(
    json: String,
) -> Result<AstGrepStructuralEvidence, ReleaseEvidenceError> {
    let evidence = serde_json::from_str::<AstGrepStructuralEvidence>(&json)
        .map_err(|error| ReleaseEvidenceError::Parse(format!("parsing ast-grep structural evidence: {error}")))?;
    validate_ast_grep_structural_evidence(evidence)
}

pub fn validate_ast_grep_structural_evidence(
    mut evidence: AstGrepStructuralEvidence,
) -> Result<AstGrepStructuralEvidence, ReleaseEvidenceError> {
    evidence.claim_labels.sort();
    evidence.non_claims.sort();
    validate_evidence_fields(&evidence)?;
    debug_assert!(evidence.claim_labels.windows(2).all(|pair| pair[0] < pair[1]));
    debug_assert!(evidence.non_claims.windows(2).all(|pair| pair[0] < pair[1]));
    Ok(evidence)
}

pub fn ast_grep_structural_evidence_canonical_bytes(
    evidence: AstGrepStructuralEvidence,
) -> Result<Vec<u8>, ReleaseEvidenceError> {
    let canonical = validate_ast_grep_structural_evidence(evidence)?;
    serde_json::to_vec(&canonical)
        .map_err(|error| validation_error(format!("serializing canonical ast-grep structural evidence: {error}")))
}

pub fn ast_grep_structural_evidence_digest_blake3(
    evidence: AstGrepStructuralEvidence,
) -> Result<String, ReleaseEvidenceError> {
    let canonical_bytes = ast_grep_structural_evidence_canonical_bytes(evidence)?;
    debug_assert!(!canonical_bytes.is_empty());
    debug_assert_eq!(canonical_bytes.first(), Some(&b'{'));
    Ok(blake3::hash(&canonical_bytes).to_hex().to_string())
}

pub fn validate_ast_grep_release_attachment(
    evidence: AstGrepStructuralEvidence,
    mut attachment: AstGrepReleaseAttachment,
) -> Result<AstGrepStructuralEvidence, ReleaseEvidenceError> {
    let canonical = validate_ast_grep_structural_evidence(evidence)?;
    attachment.non_claims.sort();
    validate_literal(LiteralField {
        value: &attachment.role,
        expected: AST_GREP_EXTERNAL_EVIDENCE_ROLE,
        field_name: "release attachment role",
    })?;
    validate_literal(LiteralField {
        value: &attachment.schema,
        expected: AST_GREP_STRUCTURAL_EVIDENCE_SCHEMA,
        field_name: "release attachment schema",
    })?;
    validate_literal(LiteralField {
        value: &attachment.claim_scope,
        expected: AST_GREP_STRUCTURAL_CLAIM_SCOPE,
        field_name: "release attachment claim_scope",
    })?;
    validate_non_claims(&attachment.non_claims)?;
    if attachment.non_claims != canonical.non_claims {
        return Err(validation_error(
            "ast-grep release attachment non_claims do not preserve the validated sidecar boundary".to_string(),
        ));
    }
    debug_assert_eq!(attachment.schema, canonical.schema);
    debug_assert_eq!(attachment.claim_scope, canonical.claim_scope);
    Ok(canonical)
}

fn validate_evidence_fields(evidence: &AstGrepStructuralEvidence) -> Result<(), ReleaseEvidenceError> {
    validate_literal(LiteralField {
        value: &evidence.schema,
        expected: AST_GREP_STRUCTURAL_EVIDENCE_SCHEMA,
        field_name: "schema",
    })?;
    validate_literal(LiteralField {
        value: &evidence.claim_scope,
        expected: AST_GREP_STRUCTURAL_CLAIM_SCOPE,
        field_name: "claim_scope",
    })?;
    validate_claim_labels(&evidence.claim_labels)?;
    validate_tool_identity(&evidence.tool)?;
    validate_blake3_hex(&evidence.command.argv_digest_blake3, "command.argv_digest_blake3")?;
    validate_identity_pair(IdentityPair {
        actual: &evidence.rule_bundle.digest_blake3,
        expected: &evidence.rule_bundle.expected_digest_blake3,
        field_name: "rule_bundle",
    })?;
    validate_scan_scope(&evidence.scan_scope)?;
    validate_output(&evidence.command.kind, &evidence.output)?;
    validate_blake3_hex(&evidence.receipt.digest_blake3, "receipt.digest_blake3")?;
    validate_non_claims(&evidence.non_claims)?;
    debug_assert_eq!(evidence.schema, AST_GREP_STRUCTURAL_EVIDENCE_SCHEMA);
    debug_assert_eq!(evidence.claim_scope, AST_GREP_STRUCTURAL_CLAIM_SCOPE);
    Ok(())
}

fn validate_tool_identity(tool: &AstGrepToolIdentity) -> Result<(), ReleaseEvidenceError> {
    validate_literal(LiteralField {
        value: &tool.package,
        expected: AST_GREP_TOOL_PACKAGE,
        field_name: "tool.package",
    })?;
    validate_bounded_text(BoundedTextField {
        value: &tool.version,
        field_name: "tool.version",
        maximum_bytes: MAX_VERSION_BYTES,
    })?;
    validate_bounded_text(BoundedTextField {
        value: &tool.expected_version,
        field_name: "tool.expected_version",
        maximum_bytes: MAX_VERSION_BYTES,
    })?;
    validate_literal(LiteralField {
        value: &tool.expected_version,
        expected: AST_GREP_TOOL_VERSION,
        field_name: "tool.expected_version",
    })?;
    if tool.version != tool.expected_version {
        return Err(validation_error(
            "ast-grep tool.version is stale or does not match tool.expected_version".to_string(),
        ));
    }
    validate_identity_pair(IdentityPair {
        actual: &tool.binary_digest_blake3,
        expected: &tool.expected_binary_digest_blake3,
        field_name: "tool.binary",
    })?;
    debug_assert_eq!(tool.version, tool.expected_version);
    debug_assert!(!tool.version.is_empty());
    Ok(())
}

fn validate_scan_scope(scan_scope: &AstGrepScanScope) -> Result<(), ReleaseEvidenceError> {
    validate_bounded_text(BoundedTextField {
        value: &scan_scope.description,
        field_name: "scan_scope.description",
        maximum_bytes: MAX_TEXT_BYTES,
    })?;
    validate_identity_pair(IdentityPair {
        actual: &scan_scope.input_digest_blake3,
        expected: &scan_scope.expected_input_digest_blake3,
        field_name: "scan_scope.input",
    })
}

fn validate_output(kind: &AstGrepCommandKind, output: &AstGrepOutputEvidence) -> Result<(), ReleaseEvidenceError> {
    match kind {
        AstGrepCommandKind::Scan => {
            if !SCAN_OUTPUT_FORMATS.contains(&output.format.as_str()) {
                return Err(unsupported_output_format(kind, output));
            }
        }
        AstGrepCommandKind::RuleTest => {
            if !RULE_TEST_OUTPUT_FORMATS.contains(&output.format.as_str()) {
                return Err(unsupported_output_format(kind, output));
            }
        }
    }
    validate_finding_summary(kind, &output.finding_summary)
}

fn unsupported_output_format(kind: &AstGrepCommandKind, output: &AstGrepOutputEvidence) -> ReleaseEvidenceError {
    validation_error(format!("ast-grep output.format has unsupported output format for {kind:?}: {}", output.format))
}

fn validate_finding_summary(
    kind: &AstGrepCommandKind,
    summary: &AstGrepFindingSummary,
) -> Result<(), ReleaseEvidenceError> {
    if summary.rule_count == 0 {
        return Err(validation_error("ast-grep finding_summary.rule_count must be non-zero".to_string()));
    }
    let counts = [
        ("rule_count", summary.rule_count),
        ("finding_count", summary.finding_count),
        ("test_case_count", summary.test_case_count),
        ("passed_test_count", summary.passed_test_count),
        ("failed_test_count", summary.failed_test_count),
    ];
    for (field_name, count) in counts {
        if count > MAX_FINDING_SUMMARY_COUNT {
            return Err(validation_error(format!(
                "ast-grep finding_summary.{field_name} exceeds {MAX_FINDING_SUMMARY_COUNT}"
            )));
        }
    }
    let observed_test_count = summary
        .passed_test_count
        .checked_add(summary.failed_test_count)
        .ok_or_else(|| validation_error("ast-grep finding_summary test count overflowed u32".to_string()))?;
    if observed_test_count != summary.test_case_count {
        return Err(validation_error(
            "ast-grep finding_summary.test_case_count does not equal passed_test_count plus failed_test_count"
                .to_string(),
        ));
    }
    debug_assert!(summary.rule_count > 0);
    debug_assert_eq!(observed_test_count, summary.test_case_count);
    validate_command_summary(kind, summary)
}

fn validate_command_summary(
    kind: &AstGrepCommandKind,
    summary: &AstGrepFindingSummary,
) -> Result<(), ReleaseEvidenceError> {
    match kind {
        AstGrepCommandKind::Scan => {
            if summary.test_case_count != 0 {
                return Err(validation_error(
                    "ast-grep scan finding summary must not report rule-test cases".to_string(),
                ));
            }
        }
        AstGrepCommandKind::RuleTest => {
            if summary.test_case_count == 0 {
                return Err(validation_error(
                    "ast-grep rule-test finding summary must report at least one test case".to_string(),
                ));
            }
        }
    }
    Ok(())
}

fn validate_claim_labels(labels: &[String]) -> Result<(), ReleaseEvidenceError> {
    validate_collection_count(labels.len(), "claim_labels", MAX_CLAIM_LABEL_COUNT)?;
    let mut seen = BTreeSet::new();
    for label in labels {
        validate_bounded_text(BoundedTextField {
            value: label,
            field_name: "claim_labels",
            maximum_bytes: MAX_TEXT_BYTES,
        })?;
        if label != AST_GREP_STRUCTURAL_CLAIM_LABEL {
            return Err(validation_error(format!(
                "ast-grep claim_labels contains unsupported or overclaimed label {label:?}"
            )));
        }
        if !seen.insert(label.clone()) {
            return Err(validation_error(format!("ast-grep claim_labels contains duplicate label {label:?}")));
        }
    }
    if !seen.contains(AST_GREP_STRUCTURAL_CLAIM_LABEL) {
        return Err(validation_error(format!("ast-grep claim_labels must contain {AST_GREP_STRUCTURAL_CLAIM_LABEL}")));
    }
    debug_assert!(!labels.is_empty());
    debug_assert!(labels.iter().all(|label| label == AST_GREP_STRUCTURAL_CLAIM_LABEL));
    Ok(())
}

fn validate_non_claims(non_claims: &[String]) -> Result<(), ReleaseEvidenceError> {
    validate_collection_count(non_claims.len(), "non_claims", MAX_NON_CLAIM_COUNT)?;
    let mut seen = BTreeSet::new();
    for non_claim in non_claims {
        validate_bounded_text(BoundedTextField {
            value: non_claim,
            field_name: "non_claims",
            maximum_bytes: MAX_TEXT_BYTES,
        })?;
        validate_no_overclaim(non_claim)?;
        if !seen.insert(non_claim.clone()) {
            return Err(validation_error(format!("ast-grep non_claims contains duplicate entry {non_claim:?}")));
        }
    }
    for required in REQUIRED_NON_CLAIMS {
        if !seen.contains(*required) {
            return Err(validation_error(format!("ast-grep non_claims is missing required boundary {required}")));
        }
    }
    debug_assert!(non_claims.len() >= REQUIRED_NON_CLAIMS.len());
    debug_assert!(REQUIRED_NON_CLAIMS.iter().all(|required| seen.contains(*required)));
    Ok(())
}

fn validate_no_overclaim(value: &str) -> Result<(), ReleaseEvidenceError> {
    let lower = value.to_ascii_lowercase();
    for fragment in OVERCLAIM_FRAGMENTS {
        if lower.contains(fragment) {
            return Err(validation_error(format!("ast-grep non_claims contains overclaim fragment {fragment:?}")));
        }
    }
    Ok(())
}

fn validate_identity_pair(identity: IdentityPair<'_>) -> Result<(), ReleaseEvidenceError> {
    validate_blake3_hex(identity.actual, &format!("{}_digest_blake3", identity.field_name))?;
    validate_blake3_hex(identity.expected, &format!("{}_expected_digest_blake3", identity.field_name))?;
    if identity.actual != identity.expected {
        return Err(validation_error(format!(
            "ast-grep {}_digest_blake3 is stale or does not match {}_expected_digest_blake3",
            identity.field_name, identity.field_name
        )));
    }
    Ok(())
}

fn validate_collection_count(count: usize, field_name: &str, maximum: u32) -> Result<(), ReleaseEvidenceError> {
    let count_u32 = u32_count(count, &format!("ast-grep {field_name} count overflowed u32"))?;
    if count_u32 == 0 {
        return Err(validation_error(format!("ast-grep {field_name} must not be empty")));
    }
    if count_u32 > maximum {
        return Err(validation_error(format!("ast-grep {field_name} count exceeds {maximum}")));
    }
    Ok(())
}

fn validate_bounded_text(field: BoundedTextField<'_>) -> Result<(), ReleaseEvidenceError> {
    if field.value.trim().is_empty() {
        return Err(validation_error(format!("ast-grep {} must not be empty", field.field_name)));
    }
    if field.value.chars().any(|character| character.is_control()) {
        return Err(validation_error(format!("ast-grep {} must not contain control characters", field.field_name)));
    }
    let byte_count = u32_count(field.value.len(), &format!("ast-grep {} length overflowed u32", field.field_name))?;
    if byte_count > field.maximum_bytes {
        return Err(validation_error(format!("ast-grep {} exceeds {} bytes", field.field_name, field.maximum_bytes)));
    }
    Ok(())
}

fn validate_literal(field: LiteralField<'_>) -> Result<(), ReleaseEvidenceError> {
    if field.value != field.expected {
        return Err(validation_error(format!(
            "ast-grep {} must be {:?}, got {:?}",
            field.field_name, field.expected, field.value
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use alloc::vec;

    use super::*;

    const POSITIVE_SCAN_JSON: &str =
        include_str!("../../../tests/fixtures/ast-grep-structural-evidence/positive-scan.json");
    const POSITIVE_RULE_TEST_JSON: &str =
        include_str!("../../../tests/fixtures/ast-grep-structural-evidence/positive-rule-test.json");
    const NEGATIVE_STALE_RULE_BUNDLE_JSON: &str =
        include_str!("../../../tests/fixtures/ast-grep-structural-evidence/negative-stale-rule-bundle.json");
    const NEGATIVE_WRONG_TOOL_IDENTITY_JSON: &str =
        include_str!("../../../tests/fixtures/ast-grep-structural-evidence/negative-wrong-tool-identity.json");
    const NEGATIVE_MISSING_NON_CLAIMS_JSON: &str =
        include_str!("../../../tests/fixtures/ast-grep-structural-evidence/negative-missing-non-claims.json");
    const NEGATIVE_UNSUPPORTED_OUTPUT_FORMAT_JSON: &str =
        include_str!("../../../tests/fixtures/ast-grep-structural-evidence/negative-unsupported-output-format.json");
    const NEGATIVE_RELEASE_OVERCLAIM_JSON: &str =
        include_str!("../../../tests/fixtures/ast-grep-structural-evidence/negative-release-overclaim.json");
    const NEGATIVE_MALFORMED_DIGEST_JSON: &str =
        include_str!("../../../tests/fixtures/ast-grep-structural-evidence/negative-malformed-digest.json");
    const NEGATIVE_UNKNOWN_OVERCLAIM_FIELD_JSON: &str =
        include_str!("../../../tests/fixtures/ast-grep-structural-evidence/negative-unknown-overclaim-field.json");
    const MISMATCHED_TEST_CASE_COUNT: u32 = 2;

    // r[verify mantle.ast_grep_structural_rails.sidecar]
    // r[verify mantle.ast_grep_structural_rails.identity]
    // r[impl mantle.ast_grep_structural_rails.fixtures]
    // r[verify mantle.ast_grep_structural_rails.fixtures]
    #[test]
    fn positive_scan_and_rule_test_fixtures_validate() {
        let cases = [
            ("scan", POSITIVE_SCAN_JSON, AstGrepCommandKind::Scan),
            ("rule-test", POSITIVE_RULE_TEST_JSON, AstGrepCommandKind::RuleTest),
        ];

        for (name, json, expected_kind) in cases {
            let evidence = parse_ast_grep_structural_evidence_json(json.to_string())
                .unwrap_or_else(|error| panic!("{name} fixture should validate: {error}"));
            let digest = ast_grep_structural_evidence_digest_blake3(evidence.clone()).unwrap();
            let mut reordered = evidence.clone();
            reordered.non_claims.reverse();
            let reordered_digest = ast_grep_structural_evidence_digest_blake3(reordered.clone()).unwrap();

            assert_eq!(evidence.command.kind, expected_kind, "{name} command kind");
            assert_ne!(evidence.non_claims, reordered.non_claims, "{name} fixture exercises canonical ordering");
            assert_eq!(digest, reordered_digest, "{name} canonical identity ignores input ordering");
            assert_eq!(digest.len(), crate::BLAKE3_HEX_LENGTH_CHARS, "{name} canonical digest length");
            assert!(digest.bytes().all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase()));
        }
    }

    #[test]
    fn negative_fixture_matrix_fails_closed_with_deterministic_diagnostics() {
        let cases = [
            ("stale rule bundle", NEGATIVE_STALE_RULE_BUNDLE_JSON, "rule_bundle_digest_blake3 is stale"),
            ("wrong tool identity", NEGATIVE_WRONG_TOOL_IDENTITY_JSON, "tool.binary_digest_blake3 is stale"),
            ("missing non-claims", NEGATIVE_MISSING_NON_CLAIMS_JSON, "missing required boundary"),
            ("unsupported output", NEGATIVE_UNSUPPORTED_OUTPUT_FORMAT_JSON, "unsupported output format"),
            ("release overclaim", NEGATIVE_RELEASE_OVERCLAIM_JSON, "overclaimed label"),
            ("malformed digest", NEGATIVE_MALFORMED_DIGEST_JSON, "must be 64 lowercase hex chars"),
            ("unknown overclaim field", NEGATIVE_UNKNOWN_OVERCLAIM_FIELD_JSON, "unknown field `release_eligible`"),
        ];

        for (name, json, expected_diagnostic) in cases {
            let error = parse_ast_grep_structural_evidence_json(json.to_string()).unwrap_err();
            let diagnostic = error.to_string();

            assert!(diagnostic.contains(expected_diagnostic), "{name}: {diagnostic}");
            assert!(!diagnostic.is_empty(), "{name} diagnostic must not be empty");
        }
    }

    #[test]
    fn canonical_sidecar_identity_is_order_independent() {
        let mut reordered = serde_json::from_str::<AstGrepStructuralEvidence>(POSITIVE_SCAN_JSON).unwrap();
        reordered.non_claims.reverse();
        let original = serde_json::from_str::<AstGrepStructuralEvidence>(POSITIVE_SCAN_JSON).unwrap();

        let first = ast_grep_structural_evidence_canonical_bytes(original).unwrap();
        let second = ast_grep_structural_evidence_canonical_bytes(reordered).unwrap();
        let digest = ast_grep_structural_evidence_digest_blake3(
            parse_ast_grep_structural_evidence_json(POSITIVE_SCAN_JSON.to_string()).unwrap(),
        )
        .unwrap();

        assert_eq!(first, second);
        assert_eq!(digest, blake3::hash(&first).to_hex().to_string());
        assert!(!first.contains(&b'\n'));
    }

    #[test]
    fn release_attachment_must_preserve_sidecar_non_claims() {
        let evidence = parse_ast_grep_structural_evidence_json(POSITIVE_SCAN_JSON.to_string()).unwrap();
        let attachment = AstGrepReleaseAttachment {
            role: AST_GREP_EXTERNAL_EVIDENCE_ROLE.to_string(),
            schema: AST_GREP_STRUCTURAL_EVIDENCE_SCHEMA.to_string(),
            claim_scope: AST_GREP_STRUCTURAL_CLAIM_SCOPE.to_string(),
            non_claims: evidence.non_claims.clone(),
        };

        let attached = validate_ast_grep_release_attachment(evidence.clone(), attachment).unwrap();
        let weakened = AstGrepReleaseAttachment {
            role: AST_GREP_EXTERNAL_EVIDENCE_ROLE.to_string(),
            schema: AST_GREP_STRUCTURAL_EVIDENCE_SCHEMA.to_string(),
            claim_scope: AST_GREP_STRUCTURAL_CLAIM_SCOPE.to_string(),
            non_claims: vec![AST_GREP_NON_CLAIM_SOURCE_BEHAVIOR.to_string()],
        };
        let error = validate_ast_grep_release_attachment(evidence, weakened).unwrap_err();

        assert_eq!(attached.claim_scope, AST_GREP_STRUCTURAL_CLAIM_SCOPE);
        assert!(error.to_string().contains("missing required boundary"));
    }

    #[test]
    fn self_consistent_unpinned_tool_version_fails_closed() {
        let mut evidence = serde_json::from_str::<AstGrepStructuralEvidence>(POSITIVE_SCAN_JSON).unwrap();
        evidence.tool.version = "0.41.0".to_string();
        evidence.tool.expected_version = evidence.tool.version.clone();

        let error = validate_ast_grep_structural_evidence(evidence).unwrap_err();
        let diagnostic = error.to_string();

        assert!(diagnostic.contains("tool.expected_version"));
        assert!(diagnostic.contains(AST_GREP_TOOL_VERSION));
    }

    #[test]
    fn rule_test_summary_counts_must_balance() {
        let mut evidence = serde_json::from_str::<AstGrepStructuralEvidence>(POSITIVE_RULE_TEST_JSON).unwrap();
        evidence.output.finding_summary.test_case_count = MISMATCHED_TEST_CASE_COUNT;

        let error = validate_ast_grep_structural_evidence(evidence).unwrap_err();

        assert!(error.to_string().contains("does not equal"));
        assert!(error.to_string().contains("passed_test_count"));
    }
}
