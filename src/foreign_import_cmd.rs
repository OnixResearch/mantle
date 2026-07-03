use std::fs;
use std::path::Path;
use std::path::PathBuf;

use clap::Subcommand;
use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::errors::RunError;
use crate::foreign_derivation_import::ForeignDerivationGraph;
use crate::foreign_derivation_import::ImportDiagnostic;
use crate::foreign_derivation_import::ImportReceipt;
use crate::foreign_derivation_import::MantleForeignPlan;
use crate::foreign_derivation_import::PackageIndex;
use crate::foreign_derivation_import::TranslationPolicy;
use crate::foreign_derivation_import::admit_translated_graph;
use crate::foreign_derivation_import::foreign_import_non_claims;
use crate::foreign_derivation_import::plan_mantle_foreign_import;
use crate::foreign_derivation_import::translate_foreign_graph;

const CLI_REPORT_SCHEMA: &str = "mantle-foreign-import-cli-v1";
const VALIDATE_COMMAND: &str = "validate";
const PLAN_COMMAND: &str = "plan";
const ACCEPTED_VERDICT: &str = "accepted";
const REJECTED_VERDICT: &str = "rejected";
const FAILURE_EXIT_CODE: u8 = 1;
const JSON_SERIALIZATION_CONTEXT: &str = "serializing foreign import CLI report";
const DEFAULT_PACKAGE_NAME: &str = "hello";
const DEFAULT_SYSTEM: &str = "x86_64-linux";

#[derive(Subcommand, Debug, Clone)]
pub(crate) enum ForeignImportAction {
    /// Validate lowered foreign derivation graph, package-index, policy, and optional receipt JSON
    Validate {
        /// Path to `foreign-derivation-graph-v1` JSON
        #[arg(long)]
        graph: PathBuf,

        /// Path to `foreign-package-index-v1` JSON
        #[arg(long = "package-index")]
        package_index: Option<PathBuf>,

        /// Path to translation policy JSON
        #[arg(long)]
        policy: PathBuf,

        /// Optional existing `foreign-derivation-import-receipt-v1` JSON to admit against the
        /// inputs
        #[arg(long)]
        receipt: Option<PathBuf>,
    },

    /// Emit a receipt-bound Mantle adapter plan from lowered foreign import artifacts
    Plan {
        /// Path to `foreign-derivation-graph-v1` JSON
        #[arg(long)]
        graph: PathBuf,

        /// Path to `foreign-package-index-v1` JSON
        #[arg(long = "package-index")]
        package_index: PathBuf,

        /// Path to translation policy JSON
        #[arg(long)]
        policy: PathBuf,

        /// Package name or alias to select from the package index
        #[arg(long, default_value = DEFAULT_PACKAGE_NAME)]
        package: String,

        /// System to select from the package index
        #[arg(long, default_value = DEFAULT_SYSTEM)]
        system: String,
    },
}

#[derive(Debug, Serialize, PartialEq, Eq)]
struct ForeignImportCliReport {
    schema: String,
    command: String,
    verdict: String,
    accepted: bool,
    diagnostics: Vec<ImportDiagnostic>,
    receipt: Option<ImportReceipt>,
    plan: Option<MantleForeignPlan>,
    non_claims: Vec<String>,
}

pub(crate) fn cmd_foreign_import(action: ForeignImportAction, json: bool) -> Result<(), RunError> {
    match action {
        ForeignImportAction::Validate {
            graph,
            package_index,
            policy,
            receipt,
        } => run_validate(&graph, package_index.as_deref(), &policy, receipt.as_deref(), json),
        ForeignImportAction::Plan {
            graph,
            package_index,
            policy,
            package,
            system,
        } => run_plan(&graph, &package_index, &policy, &package, &system, json),
    }
}

fn run_validate(
    graph_path: &Path,
    index_path: Option<&Path>,
    policy_path: &Path,
    receipt_path: Option<&Path>,
    json: bool,
) -> Result<(), RunError> {
    let graph = match read_json::<ForeignDerivationGraph>(graph_path, "graph", VALIDATE_COMMAND)? {
        Ok(graph) => graph,
        Err(report) => return emit_report(report, json),
    };
    let index = match read_optional_index(index_path, VALIDATE_COMMAND)? {
        Ok(index) => index,
        Err(report) => return emit_report(report, json),
    };
    let policy = match read_json::<TranslationPolicy>(policy_path, "policy", VALIDATE_COMMAND)? {
        Ok(policy) => policy,
        Err(report) => return emit_report(report, json),
    };
    let receipt = match read_optional_receipt(receipt_path, VALIDATE_COMMAND)? {
        Ok(receipt) => receipt,
        Err(report) => return emit_report(report, json),
    };
    let report = validate_inputs(graph, index, policy, receipt);
    emit_report(report, json)
}

fn run_plan(
    graph_path: &Path,
    index_path: &Path,
    policy_path: &Path,
    package: &str,
    system: &str,
    json: bool,
) -> Result<(), RunError> {
    let graph = match read_json::<ForeignDerivationGraph>(graph_path, "graph", PLAN_COMMAND)? {
        Ok(graph) => graph,
        Err(report) => return emit_report(report, json),
    };
    let index = match read_json::<PackageIndex>(index_path, "package-index", PLAN_COMMAND)? {
        Ok(index) => index,
        Err(report) => return emit_report(report, json),
    };
    let policy = match read_json::<TranslationPolicy>(policy_path, "policy", PLAN_COMMAND)? {
        Ok(policy) => policy,
        Err(report) => return emit_report(report, json),
    };
    let report = plan_inputs(graph, index, policy, package, system);
    emit_report(report, json)
}

fn validate_inputs(
    graph: ForeignDerivationGraph,
    index: Option<PackageIndex>,
    policy: TranslationPolicy,
    receipt: Option<ImportReceipt>,
) -> ForeignImportCliReport {
    let translated = match translate_foreign_graph(&graph, index.as_ref(), &policy) {
        Ok((_, receipt)) => receipt,
        Err(diagnostic) => return rejected_report(VALIDATE_COMMAND, diagnostic),
    };
    if let Some(receipt) = receipt.as_ref() {
        if let Err(diagnostic) = admit_translated_graph(&graph, index.as_ref(), &policy, receipt) {
            return rejected_report(VALIDATE_COMMAND, diagnostic);
        }
    }
    accepted_report(VALIDATE_COMMAND, Some(translated), None)
}

fn plan_inputs(
    graph: ForeignDerivationGraph,
    index: PackageIndex,
    policy: TranslationPolicy,
    package: &str,
    system: &str,
) -> ForeignImportCliReport {
    let (translated, receipt) = match translate_foreign_graph(&graph, Some(&index), &policy) {
        Ok(pair) => pair,
        Err(diagnostic) => return rejected_report(PLAN_COMMAND, diagnostic),
    };
    let plan = match plan_mantle_foreign_import(&translated, &index, package, system) {
        Ok(plan) => plan,
        Err(diagnostic) => return rejected_report(PLAN_COMMAND, diagnostic),
    };
    accepted_report(PLAN_COMMAND, Some(receipt), Some(plan))
}

fn read_optional_index(
    path: Option<&Path>,
    command: &str,
) -> Result<Result<Option<PackageIndex>, ForeignImportCliReport>, RunError> {
    let Some(path) = path else {
        return Ok(Ok(None));
    };
    read_json::<PackageIndex>(path, "package-index", command).map(|result| result.map(Some))
}

fn read_optional_receipt(
    path: Option<&Path>,
    command: &str,
) -> Result<Result<Option<ImportReceipt>, ForeignImportCliReport>, RunError> {
    let Some(path) = path else {
        return Ok(Ok(None));
    };
    read_json::<ImportReceipt>(path, "receipt", command).map(|result| result.map(Some))
}

fn read_json<T: DeserializeOwned>(
    path: &Path,
    artifact: &str,
    command: &str,
) -> Result<Result<T, ForeignImportCliReport>, RunError> {
    let contents = fs::read_to_string(path).map_err(|error| {
        RunError::Internal(format!("reading foreign import {artifact} {}: {error}", path.display()))
    })?;
    match serde_json::from_str::<T>(&contents) {
        Ok(value) => Ok(Ok(value)),
        Err(error) => {
            Ok(Err(rejected_report(command, diagnostic("malformed-json", None, &format!("{artifact}: {error}")))))
        }
    }
}

fn accepted_report(
    command: &str,
    receipt: Option<ImportReceipt>,
    plan: Option<MantleForeignPlan>,
) -> ForeignImportCliReport {
    ForeignImportCliReport {
        schema: CLI_REPORT_SCHEMA.to_string(),
        command: command.to_string(),
        verdict: ACCEPTED_VERDICT.to_string(),
        accepted: true,
        diagnostics: Vec::new(),
        receipt,
        plan,
        non_claims: foreign_import_non_claims(),
    }
}

fn rejected_report(command: &str, diagnostic: ImportDiagnostic) -> ForeignImportCliReport {
    ForeignImportCliReport {
        schema: CLI_REPORT_SCHEMA.to_string(),
        command: command.to_string(),
        verdict: REJECTED_VERDICT.to_string(),
        accepted: false,
        diagnostics: vec![diagnostic],
        receipt: None,
        plan: None,
        non_claims: foreign_import_non_claims(),
    }
}

fn emit_report(report: ForeignImportCliReport, json: bool) -> Result<(), RunError> {
    if json {
        println!("{}", serialize_report(&report)?);
    } else {
        println!("{}", human_report(&report));
    }
    if report.accepted {
        Ok(())
    } else {
        Err(RunError::Reported(FAILURE_EXIT_CODE))
    }
}

fn serialize_report(report: &ForeignImportCliReport) -> Result<String, RunError> {
    serde_json::to_string_pretty(report)
        .map_err(|error| RunError::Internal(format!("{JSON_SERIALIZATION_CONTEXT}: {error}")))
}

fn human_report(report: &ForeignImportCliReport) -> String {
    let mut lines = Vec::new();
    lines.push(format!("foreign import {}: {}", report.command, report.verdict));
    if report.diagnostics.is_empty() {
        lines.push("diagnostics: none".to_string());
    } else {
        lines.push("diagnostics:".to_string());
        for diagnostic in &report.diagnostics {
            lines.push(format!("- {}: {}", diagnostic.class, diagnostic.message));
        }
    }
    lines.push("non-claims:".to_string());
    for non_claim in &report.non_claims {
        lines.push(format!("- {non_claim}"));
    }
    lines.join("\n")
}

fn diagnostic(class: &str, node_id: Option<&str>, message: &str) -> ImportDiagnostic {
    ImportDiagnostic {
        class: class.to_string(),
        node_id: node_id.map(ToOwned::to_owned),
        message: message.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const VALID_DIGEST: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

    #[test]
    fn human_report_preserves_diagnostics_and_non_claims() {
        let report = rejected_report(VALIDATE_COMMAND, diagnostic("malformed-json", None, "graph: invalid"));

        let human = human_report(&report);

        assert!(human.contains("foreign import validate: rejected"));
        assert!(human.contains("malformed-json"));
        assert!(human.contains("non-claims"));
        assert!(human.contains("not-build-success"));
    }

    #[test]
    fn plan_inputs_returns_receipt_and_plan_without_process_invocations() {
        let (graph, index) = crate::foreign_derivation_import::guix_like_hello_fixture();
        let policy = fixture_policy();

        let report = plan_inputs(graph, index, policy, DEFAULT_PACKAGE_NAME, DEFAULT_SYSTEM);
        let plan = report.plan.as_ref().expect("plan should be present");

        assert!(report.accepted);
        assert_eq!(report.verdict, ACCEPTED_VERDICT);
        assert!(report.receipt.is_some());
        assert!(plan.forbidden_process_invocations.is_empty());
        assert!(plan.non_claims.contains(&"not-build-success".to_string()));
    }

    fn fixture_policy() -> TranslationPolicy {
        let mut builtin_mappings = std::collections::BTreeMap::new();
        builtin_mappings.insert("fixed-output-fetch".to_string(), "mantle.fetch".to_string());
        TranslationPolicy {
            source_prefixes: vec!["/gnu/store".to_string(), "/nix/store".to_string()],
            target_prefix: "/mantle/store".to_string(),
            rewrite_builder: true,
            rewrite_args: true,
            rewrite_env: true,
            rewrite_sources: true,
            rewrite_declared_references: true,
            allow_embedded_source_payload_rewrite: false,
            builtin_mappings,
            output_path_recompute_mode: "recompute-blake3-v1".to_string(),
            trusted_cache_scopes: std::collections::BTreeSet::new(),
            allowed_sandbox_capabilities: std::collections::BTreeSet::new(),
        }
    }

    #[test]
    fn malformed_report_is_rejected() {
        let report = rejected_report(VALIDATE_COMMAND, diagnostic("malformed-json", None, VALID_DIGEST));

        assert!(!report.accepted);
        assert_eq!(report.diagnostics[0].class, "malformed-json");
        assert!(report.receipt.is_none());
        assert!(report.plan.is_none());
    }
}
