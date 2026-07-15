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
const BUNDLE_MANIFEST_SCHEMA: &str = "mantle-nix-free-demo-bundle-manifest-v1";
const GENERATED_VALIDATION_PATH: &str = "validation.json";
const GENERATED_SUMMARY_PATH: &str = "summary.json";
const GENERATED_README_PATH: &str = "README.md";
const GENERATED_MANIFEST_PATH: &str = "manifest.json";
const SUCCESS_STAGE_DIGEST_COUNT: usize = 2;
const README_BASE_LINE_COUNT: usize = 13;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct NixFreeDemoMachineSummary {
    pub(crate) schema: String,
    pub(crate) profile: String,
    pub(crate) fixed_point_verdict: String,
    pub(crate) stage1_binary_blake3: Option<String>,
    pub(crate) stage2_binary_blake3: Option<String>,
    pub(crate) source_root_identity: String,
    pub(crate) toolchain_policy_digest_blake3: String,
    #[serde(default = "empty_demo_values")]
    pub(crate) command_owned_wrappers: Vec<NixFreeDemoNamedDigest>,
    #[serde(default = "empty_demo_values")]
    pub(crate) guards: Vec<NixFreeDemoGuardEvidence>,
    #[serde(default = "empty_demo_values")]
    pub(crate) replay_hints: Vec<String>,
    #[serde(default = "empty_demo_values")]
    pub(crate) non_claims: Vec<String>,
}

fn empty_demo_values<T>() -> Vec<T> {
    Vec::new()
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
pub(crate) struct NixFreeDemoEvidenceRef {
    pub(crate) name: String,
    pub(crate) digest_blake3: String,
    pub(crate) bundle_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct NixFreeDemoBundleManifest {
    pub(crate) schema: String,
    pub(crate) profile: String,
    pub(crate) proof_status: String,
    pub(crate) synthetic: bool,
    pub(crate) summary_path: String,
    pub(crate) readme_path: String,
    pub(crate) validation_path: String,
    pub(crate) transcripts: Vec<NixFreeDemoEvidenceRef>,
    pub(crate) receipt_digests: Vec<NixFreeDemoNamedDigest>,
    pub(crate) artifact_digests: Vec<NixFreeDemoNamedDigest>,
    pub(crate) non_claims: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NixFreeDemoManifestInput {
    pub(crate) proof_status: String,
    pub(crate) stage1_binary_blake3: Option<String>,
    pub(crate) stage2_binary_blake3: Option<String>,
    pub(crate) source_root_identity: String,
    pub(crate) toolchain_policy_digest_blake3: String,
    pub(crate) command_owned_wrappers: Vec<NixFreeDemoNamedDigest>,
    pub(crate) guards: Vec<NixFreeDemoGuardEvidence>,
    pub(crate) replay_hints: Vec<String>,
    pub(crate) non_claims: Vec<String>,
    pub(crate) transcripts: Vec<NixFreeDemoEvidenceRef>,
    pub(crate) receipt_digests: Vec<NixFreeDemoNamedDigest>,
    pub(crate) artifact_digests: Vec<NixFreeDemoNamedDigest>,
    pub(crate) synthetic: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct NixFreeDemoGeneratedBundle {
    pub(crate) summary: NixFreeDemoMachineSummary,
    pub(crate) validation: NixFreeDemoValidation,
    pub(crate) manifest: NixFreeDemoBundleManifest,
    pub(crate) readme: String,
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

pub(crate) fn build_nix_free_demo_bundle(
    input: NixFreeDemoManifestInput,
) -> Result<NixFreeDemoGeneratedBundle, NixFreeDemoDiagnostic> {
    validate_manifest_input(&input)?;
    let summary = summary_from_manifest_input(&input);
    let validation = validate_nix_free_demo_bundle(&summary);
    if input.proof_status == SUCCESS_VERDICT && !validation.demo_claimable {
        return Err(diagnostic("contradictory-proof-status", "success status lacks claimable fixed-point evidence"));
    }
    let manifest = manifest_from_input(&input);
    let readme = render_generated_nix_free_demo_readme(&summary, &validation, &manifest);
    Ok(NixFreeDemoGeneratedBundle {
        summary,
        validation,
        manifest,
        readme,
    })
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
    let line_count = README_BASE_LINE_COUNT
        .saturating_add(summary.guards.len())
        .saturating_add(summary.replay_hints.len())
        .saturating_add(summary.non_claims.len())
        .saturating_add(validation.diagnostics.len());
    let mut lines = Vec::with_capacity(line_count);
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
    debug_assert!(lines.len() <= line_count);
    debug_assert_eq!(validation.demo_claimable, validation.diagnostics.is_empty());
    lines.join("\n")
}

fn render_generated_nix_free_demo_readme(
    summary: &NixFreeDemoMachineSummary,
    validation: &NixFreeDemoValidation,
    manifest: &NixFreeDemoBundleManifest,
) -> String {
    debug_assert_eq!(manifest.schema, BUNDLE_MANIFEST_SCHEMA);
    debug_assert_eq!(manifest.profile, DEMO_PROFILE);
    let mut readme = render_nix_free_demo_readme(summary, validation);
    readme.push_str("generated bundle:\n");
    readme.push_str(&format!("- manifest: {}\n", GENERATED_MANIFEST_PATH));
    readme.push_str(&format!("- summary: {}\n", GENERATED_SUMMARY_PATH));
    readme.push_str(&format!("- validation: {}\n", GENERATED_VALIDATION_PATH));
    readme.push_str(&format!("- synthetic evidence: {}\n", manifest.synthetic));
    readme.push_str("transcripts:\n");
    for transcript in &manifest.transcripts {
        readme.push_str(&format!("- {}: {} ({})\n", transcript.name, transcript.digest_blake3, transcript.bundle_path));
    }
    readme.push_str("receipt digests:\n");
    for receipt in &manifest.receipt_digests {
        readme.push_str(&format!("- {}: {}\n", receipt.name, receipt.digest_blake3));
    }
    readme.push_str("artifact digests:\n");
    for artifact in &manifest.artifact_digests {
        readme.push_str(&format!("- {}: {}\n", artifact.name, artifact.digest_blake3));
    }
    debug_assert!(readme.starts_with("# Mantle fixed-point demo bundle"));
    debug_assert!(readme.contains("generated bundle:"));
    readme
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

fn validate_manifest_input(input: &NixFreeDemoManifestInput) -> Result<(), NixFreeDemoDiagnostic> {
    debug_assert!(!SUCCESS_VERDICT.is_empty());
    debug_assert_eq!(BLAKE3_HEX_LEN, blake3::OUT_LEN.saturating_mul(2));
    if input.proof_status.trim().is_empty() {
        return Err(diagnostic("missing-proof-status", "proof status is required"));
    }
    if input.transcripts.is_empty() {
        return Err(diagnostic("missing-transcript", "at least one transcript is required"));
    }
    validate_named_digests(&input.command_owned_wrappers, "command-wrapper-digest")?;
    validate_named_digests(&input.receipt_digests, "receipt-digest")?;
    validate_named_digests(&input.artifact_digests, "artifact-digest")?;
    validate_evidence_refs(&input.transcripts)?;
    if input.proof_status != SUCCESS_VERDICT && input.non_claims.is_empty() {
        return Err(diagnostic("missing-non-claims", "non-success evidence requires explicit non-claims"));
    }
    if input.synthetic && input.non_claims.is_empty() {
        return Err(diagnostic("missing-non-claims", "synthetic evidence requires explicit non-claims"));
    }
    if input.proof_status != SUCCESS_VERDICT && has_matching_stage_digests(input) {
        return Err(diagnostic(
            "contradictory-proof-status",
            "non-success status includes matching fixed-point stage digests",
        ));
    }
    Ok(())
}

fn validate_named_digests(digests: &[NixFreeDemoNamedDigest], code: &str) -> Result<(), NixFreeDemoDiagnostic> {
    for digest in digests {
        if digest.name.trim().is_empty() {
            return Err(diagnostic(code, "digest name is required"));
        }
        if !is_blake3_hex(&digest.digest_blake3) {
            return Err(diagnostic(code, "digest value is not a BLAKE3 hex digest"));
        }
    }
    Ok(())
}

fn validate_evidence_refs(refs: &[NixFreeDemoEvidenceRef]) -> Result<(), NixFreeDemoDiagnostic> {
    for reference in refs {
        if reference.name.trim().is_empty() {
            return Err(diagnostic("transcript-digest", "transcript name is required"));
        }
        if reference.bundle_path.trim().is_empty() {
            return Err(diagnostic("transcript-path", "transcript bundle path is required"));
        }
        if !is_blake3_hex(&reference.digest_blake3) {
            return Err(diagnostic("transcript-digest", "transcript digest is not a BLAKE3 hex digest"));
        }
    }
    Ok(())
}

fn has_matching_stage_digests(input: &NixFreeDemoManifestInput) -> bool {
    let stage_digests = [
        input.stage1_binary_blake3.as_deref(),
        input.stage2_binary_blake3.as_deref(),
    ];
    let valid = stage_digests.iter().flatten().filter(|digest| is_blake3_hex(digest)).count();
    valid == SUCCESS_STAGE_DIGEST_COUNT && input.stage1_binary_blake3 == input.stage2_binary_blake3
}

fn summary_from_manifest_input(input: &NixFreeDemoManifestInput) -> NixFreeDemoMachineSummary {
    NixFreeDemoMachineSummary {
        schema: SUMMARY_SCHEMA.to_string(),
        profile: DEMO_PROFILE.to_string(),
        fixed_point_verdict: input.proof_status.clone(),
        stage1_binary_blake3: input.stage1_binary_blake3.clone(),
        stage2_binary_blake3: input.stage2_binary_blake3.clone(),
        source_root_identity: input.source_root_identity.clone(),
        toolchain_policy_digest_blake3: input.toolchain_policy_digest_blake3.clone(),
        command_owned_wrappers: sorted_named_digests(input.command_owned_wrappers.clone()),
        guards: sorted_guard_evidence(input.guards.clone()),
        replay_hints: sorted_strings(input.replay_hints.clone()),
        non_claims: sorted_strings(input.non_claims.clone()),
    }
}

fn manifest_from_input(input: &NixFreeDemoManifestInput) -> NixFreeDemoBundleManifest {
    NixFreeDemoBundleManifest {
        schema: BUNDLE_MANIFEST_SCHEMA.to_string(),
        profile: DEMO_PROFILE.to_string(),
        proof_status: input.proof_status.clone(),
        synthetic: input.synthetic,
        summary_path: GENERATED_SUMMARY_PATH.to_string(),
        readme_path: GENERATED_README_PATH.to_string(),
        validation_path: GENERATED_VALIDATION_PATH.to_string(),
        transcripts: sorted_evidence_refs(input.transcripts.clone()),
        receipt_digests: sorted_named_digests(input.receipt_digests.clone()),
        artifact_digests: sorted_named_digests(input.artifact_digests.clone()),
        non_claims: sorted_strings(input.non_claims.clone()),
    }
}

fn sorted_named_digests(mut values: Vec<NixFreeDemoNamedDigest>) -> Vec<NixFreeDemoNamedDigest> {
    values.sort_by(|left, right| left.name.cmp(&right.name));
    values
}

fn sorted_guard_evidence(mut values: Vec<NixFreeDemoGuardEvidence>) -> Vec<NixFreeDemoGuardEvidence> {
    values.sort_by(|left, right| left.guard.cmp(&right.guard));
    values
}

fn sorted_evidence_refs(mut values: Vec<NixFreeDemoEvidenceRef>) -> Vec<NixFreeDemoEvidenceRef> {
    values.sort_by(|left, right| left.name.cmp(&right.name));
    values
}

fn sorted_strings(mut values: Vec<String>) -> Vec<String> {
    values.sort();
    values
}

fn validate_fixed_point(summary: &NixFreeDemoMachineSummary, diagnostics: &mut Vec<NixFreeDemoDiagnostic>) {
    debug_assert!(!SUCCESS_VERDICT.is_empty());
    debug_assert_eq!(BLAKE3_HEX_LEN, blake3::OUT_LEN.saturating_mul(2));
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

fn diagnostic(code: &str, message: impl AsRef<str>) -> NixFreeDemoDiagnostic {
    NixFreeDemoDiagnostic {
        code: code.to_string(),
        message: message.as_ref().to_string(),
    }
}

pub(crate) fn is_blake3_hex(value: &str) -> bool {
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
