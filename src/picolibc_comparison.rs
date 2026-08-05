// r[impl bootstrap_inventory.picolibc_stagex_comparison]
//
// Pure comparison core for the research-only Picolibc StageX diagnostic. All
// functions are deterministic over in-memory facts: no I/O, no clocks, no
// environment access. The shell (`picolibc_comparison_shell`) owns file reads
// and report writes.
//
// The classifier selects exactly one outcome:
// - `candidate`: complete behavior parity, stable identities, fewer compiled units and rewrite
//   operations than the baseline, no new protected roles.
// - `rejected`: proven behavior failure, nondeterminism, host-libc dependence, or no surface
//   reduction against the baseline.
// - `blocked`: missing or inconclusive source, tool, license, behavior, identity, baseline, or
//   protected-route evidence.

use std::collections::BTreeSet;
use std::fmt;

use serde::Deserialize;
use serde::Serialize;

pub(crate) const COMPARISON_REPORT_SCHEMA: &str = "mantle-picolibc-stagex-comparison-v1";
pub(crate) const PICOLIBC_RELEASE: &str = "1.8.12";
pub(crate) const PICOLIBC_SOURCE_DIGEST: &str = "sha256-a2XLUN2U49Lpsyizzb2cQpMbww0cmOUUKdgIj6OHhpQ=";

const MAX_TOOL_ROLES: usize = 64;
const MAX_LICENSE_CLASSES: usize = 64;
const MAX_BEHAVIOR_FAILURES: usize = 256;
const MAX_REASONS: usize = 64;
const MAX_FIELD_CHARS: usize = 4_096;
const MAX_EVIDENCE_DIGESTS: usize = 64;
const BLAKE3_HEX_CHARS: usize = 64;

const ROLE_MESON: &str = "meson";
const ROLE_NINJA: &str = "ninja";
const ROLE_COMPILER: &str = "compiler";
const ROLE_LINKER: &str = "linker";
const ROLE_ARCHIVER: &str = "archiver";

/// Facts about the Picolibc diagnostic build, normalized to be
/// path-independent.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct DiagnosticFacts {
    pub(crate) release: String,
    pub(crate) source_digest: String,
    pub(crate) compiled_units: u32,
    pub(crate) rewrite_operations: u32,
    pub(crate) tool_roles: BTreeSet<String>,
    pub(crate) license_classes: BTreeSet<String>,
    pub(crate) host_libc_dependence: bool,
    pub(crate) new_protected_roles: u32,
}

/// Facts binding the native-musl StageX baseline.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct BaselineFacts {
    pub(crate) baseline_commit: String,
    pub(crate) compiled_units: u32,
    pub(crate) rewrite_operations: u32,
    pub(crate) evidence_digests: Vec<String>,
}

/// Behavior matrix outcomes for both contracts.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct BehaviorFacts {
    pub(crate) positive_passed: u32,
    pub(crate) positive_total: u32,
    pub(crate) negative_passed: u32,
    pub(crate) negative_total: u32,
    pub(crate) failures: Vec<String>,
}

/// Two isolated builds must produce identical digests.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct IdentityFacts {
    pub(crate) first_build_digest: String,
    pub(crate) second_build_digest: String,
}

/// Complete comparison input.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct ComparisonFacts {
    pub(crate) diagnostic: DiagnosticFacts,
    pub(crate) baseline: BaselineFacts,
    pub(crate) behavior: BehaviorFacts,
    pub(crate) identity: IdentityFacts,
}

/// The bounded outcome set.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum ComparisonOutcome {
    Candidate,
    Rejected,
    Blocked,
}

impl ComparisonOutcome {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Candidate => "candidate",
            Self::Rejected => "rejected",
            Self::Blocked => "blocked",
        }
    }
}

impl fmt::Display for ComparisonOutcome {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// The deterministic comparison report.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct ComparisonReport {
    pub(crate) schema: String,
    pub(crate) outcome: ComparisonOutcome,
    pub(crate) reasons: Vec<String>,
    pub(crate) facts: ComparisonFacts,
    pub(crate) non_claims: Vec<String>,
}

pub(crate) fn comparison_non_claims() -> Vec<String> {
    [
        "This report does not claim final musl replacement.",
        "This report does not claim protected StageX admission.",
        "This report does not claim compiler, libc, or kernel correctness.",
        "This report does not claim full-bootstrap completion or release eligibility.",
        "A candidate outcome authorizes only a later Cairn change with separate evidence.",
    ]
    .iter()
    .map(ToString::to_string)
    .collect()
}

fn is_blake3_hex(value: &str) -> bool {
    value.len() == BLAKE3_HEX_CHARS && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn field_bounded(field: &str, value: &str) -> Result<(), String> {
    if value.is_empty() {
        return Err(format!("{field} must not be empty"));
    }
    if value.chars().count() > MAX_FIELD_CHARS {
        return Err(format!("{field} exceeds {MAX_FIELD_CHARS} chars"));
    }
    Ok(())
}

/// Validate fact shapes before classification. Malformed input is a hard
/// error, not a `blocked` outcome.
pub(crate) fn validate_facts(facts: &ComparisonFacts) -> Result<(), String> {
    field_bounded("baseline commit", &facts.baseline.baseline_commit)?;
    field_bounded("first build digest", &facts.identity.first_build_digest)?;
    field_bounded("second build digest", &facts.identity.second_build_digest)?;
    if facts.diagnostic.tool_roles.len() > MAX_TOOL_ROLES {
        return Err(format!("tool roles exceed {MAX_TOOL_ROLES}"));
    }
    if facts.diagnostic.license_classes.len() > MAX_LICENSE_CLASSES {
        return Err(format!("license classes exceed {MAX_LICENSE_CLASSES}"));
    }
    if facts.behavior.failures.len() > MAX_BEHAVIOR_FAILURES {
        return Err(format!("behavior failures exceed {MAX_BEHAVIOR_FAILURES}"));
    }
    if facts.baseline.evidence_digests.len() > MAX_EVIDENCE_DIGESTS {
        return Err(format!("evidence digests exceed {MAX_EVIDENCE_DIGESTS}"));
    }
    if facts.behavior.positive_passed > facts.behavior.positive_total
        || facts.behavior.negative_passed > facts.behavior.negative_total
    {
        return Err("behavior passed counts exceed totals".to_string());
    }
    Ok(())
}

fn required_tool_roles() -> BTreeSet<&'static str> {
    BTreeSet::from([ROLE_MESON, ROLE_NINJA, ROLE_COMPILER, ROLE_LINKER, ROLE_ARCHIVER])
}

fn blocked_reasons(facts: &ComparisonFacts) -> Vec<String> {
    let mut reasons: Vec<String> = Vec::new();
    let roles: BTreeSet<&str> = facts.diagnostic.tool_roles.iter().map(String::as_str).collect();
    for required in required_tool_roles() {
        if !roles.contains(required) {
            reasons.push(format!("missing tool role: {required}"));
        }
    }
    if facts.diagnostic.license_classes.is_empty() {
        reasons.push("missing license classification".to_string());
    }
    if facts.baseline.evidence_digests.is_empty() {
        reasons.push("missing baseline evidence digests".to_string());
    }
    if facts.behavior.positive_total == 0 || facts.behavior.negative_total == 0 {
        reasons.push("missing behavior matrix evidence".to_string());
    }
    if !is_blake3_hex(&facts.identity.first_build_digest) || !is_blake3_hex(&facts.identity.second_build_digest) {
        reasons.push("missing or malformed isolated-build identity digests".to_string());
    }
    debug_assert!(reasons.len() <= MAX_REASONS, "reasons stay bounded");
    reasons
}

fn rejected_reasons(facts: &ComparisonFacts) -> Vec<String> {
    let mut reasons: Vec<String> = Vec::new();
    if facts.behavior.positive_passed < facts.behavior.positive_total
        || facts.behavior.negative_passed < facts.behavior.negative_total
    {
        reasons.push(format!(
            "behavior matrix failed: {}/{} positive, {}/{} negative",
            facts.behavior.positive_passed,
            facts.behavior.positive_total,
            facts.behavior.negative_passed,
            facts.behavior.negative_total
        ));
    }
    if facts.diagnostic.host_libc_dependence {
        reasons.push("host-libc dependence detected".to_string());
    }
    if facts.identity.first_build_digest != facts.identity.second_build_digest {
        reasons.push("isolated builds produced different identities".to_string());
    }
    if facts.diagnostic.compiled_units >= facts.baseline.compiled_units {
        reasons.push(format!(
            "no surface reduction: {} diagnostic units vs {} baseline units",
            facts.diagnostic.compiled_units, facts.baseline.compiled_units
        ));
    }
    debug_assert!(reasons.len() <= MAX_REASONS, "reasons stay bounded");
    reasons
}

fn candidate_failures(facts: &ComparisonFacts) -> Vec<String> {
    let mut failures: Vec<String> = Vec::new();
    if facts.diagnostic.rewrite_operations >= facts.baseline.rewrite_operations {
        failures.push(format!(
            "rewrite operations not reduced: {} diagnostic vs {} baseline",
            facts.diagnostic.rewrite_operations, facts.baseline.rewrite_operations
        ));
    }
    if facts.diagnostic.new_protected_roles > 0 {
        failures.push(format!("diagnostic adds {} protected executable roles", facts.diagnostic.new_protected_roles));
    }
    debug_assert!(failures.len() <= MAX_REASONS, "failures stay bounded");
    failures
}

/// Classify the comparison into exactly one bounded outcome with
/// deterministic reasons.
pub(crate) fn classify(facts: &ComparisonFacts) -> ComparisonReport {
    debug_assert!(validate_facts(facts).is_ok(), "facts must be validated first");
    let blocked = blocked_reasons(facts);
    let (outcome, reasons) = if !blocked.is_empty() {
        (ComparisonOutcome::Blocked, blocked)
    } else {
        let rejected = rejected_reasons(facts);
        if !rejected.is_empty() {
            (ComparisonOutcome::Rejected, rejected)
        } else {
            let failures = candidate_failures(facts);
            if failures.is_empty() {
                (ComparisonOutcome::Candidate, vec!["all behavior, identity, and reduction checks pass".to_string()])
            } else {
                (ComparisonOutcome::Blocked, failures)
            }
        }
    };
    debug_assert!(!reasons.is_empty(), "every outcome must carry reasons");
    ComparisonReport {
        schema: COMPARISON_REPORT_SCHEMA.to_string(),
        outcome,
        reasons,
        facts: facts.clone(),
        non_claims: comparison_non_claims(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // r[verify bootstrap_inventory.picolibc_stagex_comparison]

    const BASELINE_UNITS: u32 = 765;
    const DIAGNOSTIC_UNITS: u32 = 1107;

    fn tool_roles() -> BTreeSet<String> {
        required_tool_roles().iter().map(ToString::to_string).collect()
    }

    fn complete_facts() -> ComparisonFacts {
        ComparisonFacts {
            diagnostic: DiagnosticFacts {
                release: PICOLIBC_RELEASE.to_string(),
                source_digest: PICOLIBC_SOURCE_DIGEST.to_string(),
                compiled_units: 700,
                rewrite_operations: 0,
                tool_roles: tool_roles(),
                license_classes: BTreeSet::from(["BSD-3-Clause".to_string()]),
                host_libc_dependence: false,
                new_protected_roles: 0,
            },
            baseline: BaselineFacts {
                baseline_commit: "a".repeat(40),
                compiled_units: BASELINE_UNITS,
                rewrite_operations: 4,
                evidence_digests: vec!["b".repeat(BLAKE3_HEX_CHARS)],
            },
            behavior: BehaviorFacts {
                positive_passed: 3,
                positive_total: 3,
                negative_passed: 1,
                negative_total: 1,
                failures: Vec::new(),
            },
            identity: IdentityFacts {
                first_build_digest: "c".repeat(BLAKE3_HEX_CHARS),
                second_build_digest: "c".repeat(BLAKE3_HEX_CHARS),
            },
        }
    }

    #[test]
    fn complete_reduced_facts_select_candidate() {
        let report = classify(&complete_facts());
        assert_eq!(report.outcome, ComparisonOutcome::Candidate);
        assert!(!report.reasons.is_empty());
        assert!(!report.non_claims.is_empty());
    }

    #[test]
    fn missing_tool_role_blocks() {
        let mut facts = complete_facts();
        facts.diagnostic.tool_roles.remove(ROLE_NINJA);
        let report = classify(&facts);
        assert_eq!(report.outcome, ComparisonOutcome::Blocked);
        assert!(report.reasons.iter().any(|reason| reason.contains(ROLE_NINJA)));
    }

    #[test]
    fn missing_license_evidence_blocks() {
        let mut facts = complete_facts();
        facts.diagnostic.license_classes.clear();
        let report = classify(&facts);
        assert_eq!(report.outcome, ComparisonOutcome::Blocked);
    }

    #[test]
    fn missing_baseline_evidence_blocks() {
        let mut facts = complete_facts();
        facts.baseline.evidence_digests.clear();
        let report = classify(&facts);
        assert_eq!(report.outcome, ComparisonOutcome::Blocked);
    }

    #[test]
    fn missing_behavior_matrix_blocks() {
        let mut facts = complete_facts();
        facts.behavior.positive_total = 0;
        facts.behavior.positive_passed = 0;
        let report = classify(&facts);
        assert_eq!(report.outcome, ComparisonOutcome::Blocked);
    }

    #[test]
    fn malformed_identity_digest_blocks() {
        let mut facts = complete_facts();
        facts.identity.first_build_digest = "not-hex".to_string();
        let report = classify(&facts);
        assert_eq!(report.outcome, ComparisonOutcome::Blocked);
    }

    #[test]
    fn behavior_failure_rejects() {
        let mut facts = complete_facts();
        facts.behavior.positive_passed = 2;
        facts.behavior.failures = vec!["stdio exit status".to_string()];
        let report = classify(&facts);
        assert_eq!(report.outcome, ComparisonOutcome::Rejected);
    }

    #[test]
    fn identity_divergence_rejects() {
        let mut facts = complete_facts();
        facts.identity.second_build_digest = "d".repeat(BLAKE3_HEX_CHARS);
        let report = classify(&facts);
        assert_eq!(report.outcome, ComparisonOutcome::Rejected);
    }

    #[test]
    fn host_libc_dependence_rejects() {
        let mut facts = complete_facts();
        facts.diagnostic.host_libc_dependence = true;
        let report = classify(&facts);
        assert_eq!(report.outcome, ComparisonOutcome::Rejected);
    }

    #[test]
    fn no_surface_reduction_rejects() {
        let mut facts = complete_facts();
        facts.diagnostic.compiled_units = DIAGNOSTIC_UNITS;
        let report = classify(&facts);
        assert_eq!(report.outcome, ComparisonOutcome::Rejected);
        assert!(report.reasons.iter().any(|reason| reason.contains("no surface reduction")));
    }

    #[test]
    fn equal_units_rejects() {
        let mut facts = complete_facts();
        facts.diagnostic.compiled_units = BASELINE_UNITS;
        let report = classify(&facts);
        assert_eq!(report.outcome, ComparisonOutcome::Rejected);
    }

    #[test]
    fn unreduced_rewrites_block_candidate() {
        let mut facts = complete_facts();
        facts.diagnostic.rewrite_operations = 4;
        let report = classify(&facts);
        assert_eq!(report.outcome, ComparisonOutcome::Blocked);
    }

    #[test]
    fn new_protected_roles_block_candidate() {
        let mut facts = complete_facts();
        facts.diagnostic.new_protected_roles = 2;
        let report = classify(&facts);
        assert_eq!(report.outcome, ComparisonOutcome::Blocked);
        assert!(report.reasons.iter().any(|reason| reason.contains("protected")));
    }

    #[test]
    fn contradictory_behavior_counts_are_hard_errors() {
        let mut facts = complete_facts();
        facts.behavior.positive_passed = facts.behavior.positive_total + 1;
        assert!(validate_facts(&facts).is_err());
    }

    #[test]
    fn oversized_inputs_are_hard_errors() {
        let mut facts = complete_facts();
        facts.behavior.failures = vec!["x".to_string(); MAX_BEHAVIOR_FAILURES + 1];
        assert!(validate_facts(&facts).is_err());
    }

    #[test]
    fn empty_baseline_commit_is_a_hard_error() {
        let mut facts = complete_facts();
        facts.baseline.baseline_commit = String::new();
        assert!(validate_facts(&facts).is_err());
    }

    #[test]
    fn classification_is_deterministic() {
        let facts = complete_facts();
        assert_eq!(classify(&facts), classify(&facts));
    }
}
