use std::fs;
use std::path::Path;
use std::path::PathBuf;

use clap::Subcommand;
use serde::Serialize;

use crate::errors::RunError;
use crate::nix_free_demo_bundle::NixFreeDemoDiagnostic;
use crate::nix_free_demo_bundle::NixFreeDemoMachineSummary;
use crate::nix_free_demo_bundle::NixFreeDemoValidation;
use crate::nix_free_demo_bundle::render_nix_free_demo_readme;
use crate::nix_free_demo_bundle::validate_nix_free_demo_bundle;

const CLI_REPORT_SCHEMA: &str = "mantle-nix-free-demo-cli-v1";
const UNKNOWN_PROFILE: &str = "unknown";
const MALFORMED_VERDICT: &str = "malformed";
const MALFORMED_SUMMARY_CODE: &str = "malformed-summary";
const NON_CLAIMABLE_EXIT_CODE: u8 = 1;
const JSON_SERIALIZATION_CONTEXT: &str = "serializing Nix-free demo CLI report";

#[derive(Subcommand, Debug, Clone)]
pub(crate) enum NixFreeDemoAction {
    /// Validate a Nix-free demo bundle machine summary JSON file
    Validate {
        /// Path to `mantle-nix-free-demo-summary-v1` JSON
        summary: PathBuf,
    },

    /// Render the operator README from a Nix-free demo bundle machine summary JSON file
    Readme {
        /// Path to `mantle-nix-free-demo-summary-v1` JSON
        summary: PathBuf,
    },
}

#[derive(Debug, Serialize, PartialEq, Eq)]
struct NixFreeDemoCliReport {
    schema: String,
    profile: String,
    verdict: String,
    demo_claimable: bool,
    diagnostics: Vec<NixFreeDemoDiagnostic>,
}

pub(crate) fn cmd_nix_free_demo(action: NixFreeDemoAction, json: bool) -> Result<(), RunError> {
    match action {
        NixFreeDemoAction::Validate { summary } => run_validate(&summary, json),
        NixFreeDemoAction::Readme { summary } => run_readme(&summary, json),
    }
}

fn run_validate(summary_path: &Path, json: bool) -> Result<(), RunError> {
    let contents = read_summary(summary_path)?;
    let (summary, validation) = match parse_and_validate_summary(&contents) {
        Ok(pair) => pair,
        Err(report) => return emit_report(report, json),
    };
    let report = report_for_summary(&summary, &validation);
    emit_report(report, json)
}

fn run_readme(summary_path: &Path, json: bool) -> Result<(), RunError> {
    let contents = read_summary(summary_path)?;
    let (summary, validation) = match parse_and_validate_summary(&contents) {
        Ok(pair) => pair,
        Err(report) => return emit_report(report, json),
    };
    let readme = render_nix_free_demo_readme(&summary, &validation);
    println!("{readme}");
    if validation.demo_claimable {
        Ok(())
    } else {
        Err(RunError::Reported(NON_CLAIMABLE_EXIT_CODE))
    }
}

fn read_summary(summary_path: &Path) -> Result<String, RunError> {
    fs::read_to_string(summary_path).map_err(|error| {
        RunError::Internal(format!("reading Nix-free demo summary {}: {error}", summary_path.display()))
    })
}

fn parse_and_validate_summary(
    contents: &str,
) -> Result<(NixFreeDemoMachineSummary, NixFreeDemoValidation), NixFreeDemoCliReport> {
    match serde_json::from_str::<NixFreeDemoMachineSummary>(contents) {
        Ok(summary) => {
            let validation = validate_nix_free_demo_bundle(&summary);
            Ok((summary, validation))
        }
        Err(error) => Err(malformed_report(&error.to_string())),
    }
}

fn report_for_summary(summary: &NixFreeDemoMachineSummary, validation: &NixFreeDemoValidation) -> NixFreeDemoCliReport {
    NixFreeDemoCliReport {
        schema: CLI_REPORT_SCHEMA.to_string(),
        profile: validation.profile.clone(),
        verdict: summary.fixed_point_verdict.clone(),
        demo_claimable: validation.demo_claimable,
        diagnostics: validation.diagnostics.clone(),
    }
}

fn malformed_report(message: &str) -> NixFreeDemoCliReport {
    NixFreeDemoCliReport {
        schema: CLI_REPORT_SCHEMA.to_string(),
        profile: UNKNOWN_PROFILE.to_string(),
        verdict: MALFORMED_VERDICT.to_string(),
        demo_claimable: false,
        diagnostics: vec![NixFreeDemoDiagnostic {
            code: MALFORMED_SUMMARY_CODE.to_string(),
            message: message.to_string(),
        }],
    }
}

fn emit_report(report: NixFreeDemoCliReport, json: bool) -> Result<(), RunError> {
    if json {
        println!("{}", serialize_report(&report)?);
    } else {
        println!("{}", human_report(&report));
    }
    if report.demo_claimable {
        Ok(())
    } else {
        Err(RunError::Reported(NON_CLAIMABLE_EXIT_CODE))
    }
}

fn serialize_report(report: &NixFreeDemoCliReport) -> Result<String, RunError> {
    serde_json::to_string_pretty(report)
        .map_err(|error| RunError::Internal(format!("{JSON_SERIALIZATION_CONTEXT}: {error}")))
}

fn human_report(report: &NixFreeDemoCliReport) -> String {
    let mut lines = Vec::new();
    if report.demo_claimable {
        lines.push("Nix-free demo bundle: claimable".to_string());
    } else {
        lines.push("Nix-free demo bundle: not claimable".to_string());
    }
    lines.push(format!("profile: {}", report.profile));
    lines.push(format!("fixed-point verdict: {}", report.verdict));
    if report.diagnostics.is_empty() {
        lines.push("diagnostics: none".to_string());
    } else {
        lines.push("diagnostics:".to_string());
        for diagnostic in &report.diagnostics {
            lines.push(format!("- {}: {}", diagnostic.code, diagnostic.message));
        }
    }
    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    const DIGEST_A: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    const DIGEST_B: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";

    #[test]
    fn report_for_summary_preserves_claimability_and_verdict() {
        let summary = claimable_summary();
        let validation = validate_nix_free_demo_bundle(&summary);

        let report = report_for_summary(&summary, &validation);
        let human = human_report(&report);

        assert_eq!(report.schema, CLI_REPORT_SCHEMA);
        assert_eq!(report.profile, "source-root-cargo-free-fixed-point");
        assert_eq!(report.verdict, "success");
        assert!(report.demo_claimable);
        assert!(report.diagnostics.is_empty());
        assert!(human.contains("Nix-free demo bundle: claimable"));
        assert!(human.contains("diagnostics: none"));
    }

    #[test]
    fn malformed_summary_report_fails_closed_with_stable_code() {
        let report = parse_and_validate_summary("not-json").unwrap_err();
        let encoded = serialize_report(&report).unwrap();

        assert_eq!(report.schema, CLI_REPORT_SCHEMA);
        assert_eq!(report.profile, UNKNOWN_PROFILE);
        assert_eq!(report.verdict, MALFORMED_VERDICT);
        assert!(!report.demo_claimable);
        assert_eq!(report.diagnostics.len(), 1);
        assert_eq!(report.diagnostics[0].code, MALFORMED_SUMMARY_CODE);
        assert!(encoded.contains(MALFORMED_SUMMARY_CODE));
    }

    fn claimable_summary() -> NixFreeDemoMachineSummary {
        NixFreeDemoMachineSummary {
            schema: "mantle-nix-free-demo-summary-v1".to_string(),
            profile: "source-root-cargo-free-fixed-point".to_string(),
            fixed_point_verdict: "success".to_string(),
            stage1_binary_blake3: Some(DIGEST_A.to_string()),
            stage2_binary_blake3: Some(DIGEST_A.to_string()),
            source_root_identity: "source-root-v1".to_string(),
            toolchain_policy_digest_blake3: DIGEST_B.to_string(),
            command_owned_wrappers: Vec::new(),
            guards: ["cargo", "nix", "rustup", "ambient-wrapper"]
                .iter()
                .map(|guard| crate::nix_free_demo_bundle::NixFreeDemoGuardEvidence {
                    guard: (*guard).to_string(),
                    status: "denied".to_string(),
                    diagnostic: format!("{guard} denied by fixture"),
                })
                .collect(),
            replay_hints: vec!["copy bundle and rerun validator".to_string()],
            non_claims: vec!["not compiler correctness".to_string()],
        }
    }
}
