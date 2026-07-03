use std::collections::BTreeSet;

use serde::Deserialize;
use serde::Serialize;

const SUMMARY_SCHEMA: &str = "mantle-nix-free-demo-summary-v1";
const DEMO_PROFILE: &str = "source-root-cargo-free-fixed-point";
const SUCCESS_VERDICT: &str = "success";
const DENIED_GUARD_STATUS: &str = "denied";
const BLAKE3_HEX_LEN: usize = 64;
const REQUIRED_GUARD_COUNT: usize = 4;
const REQUIRED_GUARDS: [&str; REQUIRED_GUARD_COUNT] = ["cargo", "nix", "rustup", "ambient-wrapper"];
const MISSING_FIXED_POINT_EVIDENCE: &str = "missing-fixed-point-evidence";
const MISSING_GUARD_EVIDENCE: &str = "missing-guard-evidence";
const MISSING_NON_CLAIMS: &str = "missing-non-claims";
const INVALID_SOURCE_ROOT_IDENTITY: &str = "invalid-source-root-identity";
const INVALID_TOOLCHAIN_POLICY_DIGEST: &str = "invalid-toolchain-policy-digest";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct NixFreeDemoMachineSummary {
    pub(crate) schema: String,
    pub(crate) profile: String,
    pub(crate) fixed_point_verdict: String,
    pub(crate) stage1_binary_blake3: Option<String>,
    pub(crate) stage2_binary_blake3: Option<String>,
    pub(crate) source_root_identity: String,
    pub(crate) toolchain_policy_digest_blake3: String,
    #[serde(default)]
    pub(crate) command_owned_wrappers: Vec<NixFreeDemoNamedDigest>,
    #[serde(default)]
    pub(crate) guards: Vec<NixFreeDemoGuardEvidence>,
    #[serde(default)]
    pub(crate) replay_hints: Vec<String>,
    #[serde(default)]
    pub(crate) non_claims: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct NixFreeDemoNamedDigest {
    pub(crate) name: String,
    pub(crate) digest_blake3: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct NixFreeDemoGuardEvidence {
    pub(crate) guard: String,
    pub(crate) status: String,
    pub(crate) diagnostic: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct NixFreeDemoValidation {
    pub(crate) schema: String,
    pub(crate) profile: String,
    pub(crate) demo_claimable: bool,
    pub(crate) diagnostics: Vec<NixFreeDemoDiagnostic>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct NixFreeDemoDiagnostic {
    pub(crate) code: String,
    pub(crate) message: String,
}

pub(crate) fn validate_nix_free_demo_bundle(summary: &NixFreeDemoMachineSummary) -> NixFreeDemoValidation {
    let mut diagnostics = Vec::new();
    validate_fixed_point(summary, &mut diagnostics);
    validate_source_root(summary, &mut diagnostics);
    validate_required_guards(summary, &mut diagnostics);
    validate_non_claims(summary, &mut diagnostics);
    NixFreeDemoValidation {
        schema: SUMMARY_SCHEMA.to_string(),
        profile: DEMO_PROFILE.to_string(),
        demo_claimable: diagnostics.is_empty(),
        diagnostics,
    }
}

pub(crate) fn render_nix_free_demo_readme(
    summary: &NixFreeDemoMachineSummary,
    validation: &NixFreeDemoValidation,
) -> String {
    let mut lines = Vec::new();
    lines.push("# Mantle fixed-point demo bundle".to_string());
    lines.push(String::new());
    if validation.demo_claimable {
        lines.push("Nix-free fixed-point demo: claimable".to_string());
    } else {
        lines.push("Demo claim: not claimable".to_string());
    }
    lines.push(format!("fixed-point verdict: {}", summary.fixed_point_verdict));
    lines.push(format!("stage1 binary BLAKE3: {}", summary.stage1_binary_blake3.as_deref().unwrap_or("missing")));
    lines.push(format!("stage2 binary BLAKE3: {}", summary.stage2_binary_blake3.as_deref().unwrap_or("missing")));
    lines.push(format!("source root: {}", summary.source_root_identity));
    lines.push(format!("toolchain policy BLAKE3: {}", summary.toolchain_policy_digest_blake3));
    lines.push("guards:".to_string());
    for guard in sorted_guards(&summary.guards) {
        lines.push(format!("- {}: {} ({})", guard.guard, guard.status, guard.diagnostic));
    }
    lines.push("replay hints:".to_string());
    for hint in &summary.replay_hints {
        lines.push(format!("- {hint}"));
    }
    lines.push("non-claims:".to_string());
    for non_claim in &summary.non_claims {
        lines.push(format!("- {non_claim}"));
    }
    if !validation.diagnostics.is_empty() {
        lines.push("diagnostics:".to_string());
        for diagnostic in &validation.diagnostics {
            lines.push(format!("- {}: {}", diagnostic.code, diagnostic.message));
        }
    }
    lines.push(String::new());
    lines.join("\n")
}

pub(crate) fn nix_free_demo_claim(summary: &NixFreeDemoMachineSummary) -> Option<String> {
    let validation = validate_nix_free_demo_bundle(summary);
    if !validation.demo_claimable {
        return None;
    }
    Some(format!(
        "Nix-free fixed-point demo profile `{}` is claimable for source root `{}`",
        DEMO_PROFILE, summary.source_root_identity
    ))
}

fn validate_fixed_point(summary: &NixFreeDemoMachineSummary, diagnostics: &mut Vec<NixFreeDemoDiagnostic>) {
    if summary.fixed_point_verdict != SUCCESS_VERDICT {
        diagnostics.push(diagnostic(MISSING_FIXED_POINT_EVIDENCE, "fixed-point verdict is not success"));
        return;
    }
    let Some(stage1) = summary.stage1_binary_blake3.as_deref() else {
        diagnostics.push(diagnostic(MISSING_FIXED_POINT_EVIDENCE, "stage1 binary digest is missing"));
        return;
    };
    let Some(stage2) = summary.stage2_binary_blake3.as_deref() else {
        diagnostics.push(diagnostic(MISSING_FIXED_POINT_EVIDENCE, "stage2 binary digest is missing"));
        return;
    };
    if stage1 != stage2 || !is_blake3_hex(stage1) {
        diagnostics.push(diagnostic(
            MISSING_FIXED_POINT_EVIDENCE,
            "stage1/stage2 binary digests are absent, invalid, or different",
        ));
    }
}

fn validate_source_root(summary: &NixFreeDemoMachineSummary, diagnostics: &mut Vec<NixFreeDemoDiagnostic>) {
    if summary.source_root_identity.trim().is_empty() {
        diagnostics.push(diagnostic(INVALID_SOURCE_ROOT_IDENTITY, "source-root identity is missing"));
    }
    if !is_blake3_hex(&summary.toolchain_policy_digest_blake3) {
        diagnostics
            .push(diagnostic(INVALID_TOOLCHAIN_POLICY_DIGEST, "toolchain policy digest is not a BLAKE3 hex digest"));
    }
}

fn validate_required_guards(summary: &NixFreeDemoMachineSummary, diagnostics: &mut Vec<NixFreeDemoDiagnostic>) {
    let denied = summary
        .guards
        .iter()
        .filter(|guard| guard.status == DENIED_GUARD_STATUS)
        .map(|guard| guard.guard.as_str())
        .collect::<BTreeSet<_>>();
    for guard in REQUIRED_GUARDS {
        if !denied.contains(guard) {
            diagnostics
                .push(diagnostic(MISSING_GUARD_EVIDENCE, &format!("required guard `{guard}` lacks denial evidence")));
        }
    }
}

fn validate_non_claims(summary: &NixFreeDemoMachineSummary, diagnostics: &mut Vec<NixFreeDemoDiagnostic>) {
    if summary.non_claims.is_empty() {
        diagnostics.push(diagnostic(MISSING_NON_CLAIMS, "explicit non-claims are missing"));
    }
}

fn sorted_guards(guards: &[NixFreeDemoGuardEvidence]) -> Vec<&NixFreeDemoGuardEvidence> {
    let mut sorted = guards.iter().collect::<Vec<_>>();
    sorted.sort_by(|left, right| left.guard.cmp(&right.guard));
    sorted
}

fn diagnostic(code: &str, message: &str) -> NixFreeDemoDiagnostic {
    NixFreeDemoDiagnostic {
        code: code.to_string(),
        message: message.to_string(),
    }
}

fn is_blake3_hex(value: &str) -> bool {
    value.len() == BLAKE3_HEX_LEN && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

#[cfg(test)]
mod tests {
    use super::*;

    const DIGEST_A: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    const DIGEST_B: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";

    #[test]
    fn demo_bundle_validator_accepts_matching_fixed_point_and_guard_denials() {
        let summary = claimable_summary();

        let validation = validate_nix_free_demo_bundle(&summary);
        let claim = nix_free_demo_claim(&summary).unwrap();
        let readme = render_nix_free_demo_readme(&summary, &validation);

        assert!(validation.demo_claimable);
        assert!(validation.diagnostics.is_empty());
        assert!(claim.contains("Nix-free fixed-point demo profile"));
        assert!(readme.contains("Nix-free fixed-point demo: claimable"));
        assert!(readme.contains(DIGEST_A));
        assert!(readme.contains("cargo: denied"));
    }

    #[test]
    fn demo_bundle_validator_rejects_missing_fixed_point_evidence() {
        let mut summary = claimable_summary();
        summary.stage2_binary_blake3 = Some(DIGEST_B.to_string());

        let validation = validate_nix_free_demo_bundle(&summary);
        let readme = render_nix_free_demo_readme(&summary, &validation);

        assert!(!validation.demo_claimable);
        assert!(diagnostic_codes(&validation).contains(MISSING_FIXED_POINT_EVIDENCE));
        assert!(nix_free_demo_claim(&summary).is_none());
        assert!(!readme.contains("Nix-free fixed-point demo: claimable"));
    }

    #[test]
    fn demo_bundle_validator_rejects_missing_guard_denial() {
        let mut summary = claimable_summary();
        summary.guards.retain(|guard| guard.guard != "rustup");

        let validation = validate_nix_free_demo_bundle(&summary);

        assert!(!validation.demo_claimable);
        assert!(diagnostic_codes(&validation).contains(MISSING_GUARD_EVIDENCE));
        assert!(validation.diagnostics.iter().any(|diagnostic| diagnostic.message.contains("rustup")));
    }

    #[test]
    fn generated_readme_is_derived_from_machine_summary() {
        let mut summary = claimable_summary();
        summary.replay_hints = vec!["mantle self-build --cargo-free --fixed-point".to_string()];
        summary.non_claims.push("not release reproducibility".to_string());
        summary.command_owned_wrappers.push(NixFreeDemoNamedDigest {
            name: "rustc-normalized".to_string(),
            digest_blake3: DIGEST_B.to_string(),
        });
        let validation = validate_nix_free_demo_bundle(&summary);

        let readme = render_nix_free_demo_readme(&summary, &validation);

        assert!(readme.contains("mantle self-build --cargo-free --fixed-point"));
        assert!(readme.contains("not release reproducibility"));
        assert!(readme.contains("toolchain policy BLAKE3"));
    }

    fn claimable_summary() -> NixFreeDemoMachineSummary {
        NixFreeDemoMachineSummary {
            schema: SUMMARY_SCHEMA.to_string(),
            profile: DEMO_PROFILE.to_string(),
            fixed_point_verdict: SUCCESS_VERDICT.to_string(),
            stage1_binary_blake3: Some(DIGEST_A.to_string()),
            stage2_binary_blake3: Some(DIGEST_A.to_string()),
            source_root_identity: "source-root-v1".to_string(),
            toolchain_policy_digest_blake3: DIGEST_B.to_string(),
            command_owned_wrappers: Vec::new(),
            guards: REQUIRED_GUARDS
                .iter()
                .map(|guard| NixFreeDemoGuardEvidence {
                    guard: (*guard).to_string(),
                    status: DENIED_GUARD_STATUS.to_string(),
                    diagnostic: format!("{guard} denied by fixture"),
                })
                .collect(),
            replay_hints: vec!["copy bundle and rerun validator".to_string()],
            non_claims: vec!["not compiler correctness".to_string()],
        }
    }

    fn diagnostic_codes(validation: &NixFreeDemoValidation) -> BTreeSet<&str> {
        validation.diagnostics.iter().map(|diagnostic| diagnostic.code.as_str()).collect()
    }
}
