use std::path::Path;
use std::path::PathBuf;

use crunch_release_core::AdversarialHermeticityGauntletReport;
use crunch_release_core::BootstrapPressureGauntletReport;
use crunch_release_core::ContinuousGauntletContext;
use crunch_release_core::ContinuousReproducibilityGauntletReport;
use crunch_release_core::GauntletTrackEvidence;
use crunch_release_core::NixMantleComparisonCorpusReport;
use crunch_release_core::ReleaseRepeatabilityMatrixReport;
use crunch_release_core::StrictHermeticityRegressionCaseEvidence;
use crunch_release_core::StrictHermeticityRegressionSuitePlan;
use crunch_release_core::StrictHermeticityRegressionSuiteReport;
use crunch_release_core::SubstitutionCacheAttackGauntletReport;
use crunch_release_core::adversarial_hermeticity_gauntlet_report_canonical_bytes;
use crunch_release_core::bootstrap_pressure_gauntlet_report_canonical_bytes;
use crunch_release_core::continuous_reproducibility_gauntlet_report_canonical_bytes;
use crunch_release_core::continuous_reproducibility_gauntlet_report_digest_blake3;
use crunch_release_core::evaluate_continuous_reproducibility_gauntlet;
use crunch_release_core::evaluate_strict_hermeticity_regression_suite;
use crunch_release_core::nix_mantle_comparison_corpus_report_canonical_bytes;
use crunch_release_core::release_repeatability_matrix_report_canonical_bytes;
use crunch_release_core::strict_hermeticity_regression_suite_report_canonical_bytes;
use crunch_release_core::strict_hermeticity_regression_suite_report_digest_blake3;
use crunch_release_core::substitution_cache_attack_gauntlet_report_canonical_bytes;
use serde::de::DeserializeOwned;

use crate::GauntletReportKind;
use crate::ReleaseGauntletAction;
use crate::errors::RunError;

const CANONICALIZE_SUMMARY_KIND: &str = "mantle-gauntlet-canonicalize-v1";
const CONTINUOUS_SUMMARY_KIND: &str = "mantle-continuous-gauntlet-run-v1";
const STRICT_REGRESSION_SUMMARY_KIND: &str = "mantle-strict-hermeticity-regression-run-v1";
const DEFAULT_CONTINUOUS_REPORT_PATH: &str = "continuous-reproducibility-gauntlet-report.json";
const DEFAULT_STRICT_REGRESSION_REPORT_PATH: &str = "strict-hermeticity-regression-suite-report.json";

pub(crate) fn cmd_release_gauntlet(
    action: ReleaseGauntletAction,
    current_dir: &Path,
    json: bool,
) -> Result<(), RunError> {
    match action {
        ReleaseGauntletAction::Canonicalize { kind, input, output } => {
            cmd_gauntlet_canonicalize(current_dir, json, kind, input, output)
        }
        ReleaseGauntletAction::Continuous {
            context,
            track,
            report_path,
        } => cmd_gauntlet_continuous(current_dir, json, context, track, report_path),
        ReleaseGauntletAction::StrictHermeticityRegression {
            run_id,
            plan,
            evidence,
            report_path,
        } => cmd_strict_hermeticity_regression(current_dir, json, run_id, plan, evidence, report_path),
    }
}

fn cmd_gauntlet_canonicalize(
    current_dir: &Path,
    json: bool,
    kind: GauntletReportKind,
    input: PathBuf,
    output: Option<PathBuf>,
) -> Result<(), RunError> {
    let input_path = resolve_input_path(current_dir, input);
    let output_path = output.map(|path| resolve_input_path(current_dir, path));
    let input_bytes = read_file(&input_path)?;
    let canonical = canonicalize_report_bytes(kind, &input_bytes)?;
    let digest_blake3 = blake3::hash(&canonical).to_hex().to_string();
    write_or_print_json(output_path.as_deref(), &canonical)?;
    print_canonicalize_summary(json, kind, &input_path, output_path.as_deref(), &digest_blake3)?;
    Ok(())
}

fn cmd_gauntlet_continuous(
    current_dir: &Path,
    json: bool,
    context: PathBuf,
    track_paths: Vec<PathBuf>,
    report_path: Option<PathBuf>,
) -> Result<(), RunError> {
    let context_path = resolve_input_path(current_dir, context);
    let resolved_track_paths =
        track_paths.into_iter().map(|path| resolve_input_path(current_dir, path)).collect::<Vec<_>>();
    let output_path = report_path
        .map(|path| resolve_input_path(current_dir, path))
        .unwrap_or_else(|| current_dir.join(DEFAULT_CONTINUOUS_REPORT_PATH));
    let context = read_json_file::<ContinuousGauntletContext>(&context_path)?;
    let tracks = read_track_evidence_files(&resolved_track_paths)?;
    let report = evaluate_continuous_reproducibility_gauntlet(context, tracks)
        .map_err(|err| RunError::Internal(format!("evaluating continuous gauntlet: {err}")))?;
    let digest_blake3 = continuous_reproducibility_gauntlet_report_digest_blake3(report.clone())
        .map_err(|err| RunError::Internal(format!("digesting continuous gauntlet report: {err}")))?;
    let canonical = continuous_reproducibility_gauntlet_report_canonical_bytes(report.clone())
        .map_err(|err| RunError::Internal(format!("serializing continuous gauntlet report: {err}")))?;
    write_or_print_json(Some(&output_path), &canonical)?;
    print_continuous_summary(json, &report, &output_path, &digest_blake3)?;
    Ok(())
}

fn cmd_strict_hermeticity_regression(
    current_dir: &Path,
    json: bool,
    run_id: String,
    plan: PathBuf,
    evidence_paths: Vec<PathBuf>,
    report_path: Option<PathBuf>,
) -> Result<(), RunError> {
    let plan_path = resolve_input_path(current_dir, plan);
    let resolved_evidence_paths =
        evidence_paths.into_iter().map(|path| resolve_input_path(current_dir, path)).collect::<Vec<_>>();
    let output_path = report_path
        .map(|path| resolve_input_path(current_dir, path))
        .unwrap_or_else(|| current_dir.join(DEFAULT_STRICT_REGRESSION_REPORT_PATH));
    let plan = read_json_file::<StrictHermeticityRegressionSuitePlan>(&plan_path)?;
    let evidence = read_strict_regression_evidence_files(&resolved_evidence_paths)?;
    let report = evaluate_strict_hermeticity_regression_suite(run_id, plan, evidence)
        .map_err(|err| RunError::Internal(format!("evaluating strict hermeticity regression suite: {err}")))?;
    let digest_blake3 = strict_hermeticity_regression_suite_report_digest_blake3(report.clone())
        .map_err(|err| RunError::Internal(format!("digesting strict hermeticity regression report: {err}")))?;
    let canonical = strict_hermeticity_regression_suite_report_canonical_bytes(report.clone())
        .map_err(|err| RunError::Internal(format!("serializing strict hermeticity regression report: {err}")))?;
    write_or_print_json(Some(&output_path), &canonical)?;
    print_strict_regression_summary(json, &report, &output_path, &digest_blake3)?;
    Ok(())
}

fn canonicalize_report_bytes(kind: GauntletReportKind, input: &[u8]) -> Result<Vec<u8>, RunError> {
    match kind {
        GauntletReportKind::AdversarialHermeticity => {
            let report = parse_json_slice::<AdversarialHermeticityGauntletReport>(input, kind.as_str())?;
            adversarial_hermeticity_gauntlet_report_canonical_bytes(report)
        }
        GauntletReportKind::BootstrapPressure => {
            let report = parse_json_slice::<BootstrapPressureGauntletReport>(input, kind.as_str())?;
            bootstrap_pressure_gauntlet_report_canonical_bytes(report)
        }
        GauntletReportKind::SubstitutionCacheAttack => {
            let report = parse_json_slice::<SubstitutionCacheAttackGauntletReport>(input, kind.as_str())?;
            substitution_cache_attack_gauntlet_report_canonical_bytes(report)
        }
        GauntletReportKind::NixMantleComparison => {
            let report = parse_json_slice::<NixMantleComparisonCorpusReport>(input, kind.as_str())?;
            nix_mantle_comparison_corpus_report_canonical_bytes(report)
        }
        GauntletReportKind::ReleaseRepeatability => {
            let report = parse_json_slice::<ReleaseRepeatabilityMatrixReport>(input, kind.as_str())?;
            release_repeatability_matrix_report_canonical_bytes(report)
        }
        GauntletReportKind::Continuous => {
            let report = parse_json_slice::<ContinuousReproducibilityGauntletReport>(input, kind.as_str())?;
            continuous_reproducibility_gauntlet_report_canonical_bytes(report)
        }
        GauntletReportKind::StrictHermeticityRegression => {
            let report = parse_json_slice::<StrictHermeticityRegressionSuiteReport>(input, kind.as_str())?;
            strict_hermeticity_regression_suite_report_canonical_bytes(report)
        }
    }
    .map_err(|err| RunError::Internal(format!("canonicalizing {} gauntlet report: {err}", kind.as_str())))
}

fn read_track_evidence_files(paths: &[PathBuf]) -> Result<Vec<GauntletTrackEvidence>, RunError> {
    if paths.is_empty() {
        return Err(RunError::Internal("continuous gauntlet requires at least one --track evidence file".to_string()));
    }
    let mut tracks = Vec::new();
    for path in paths {
        let bytes = read_file(path)?;
        let mut file_tracks = parse_track_evidence_slice(&bytes, path)?;
        tracks.append(&mut file_tracks);
    }
    Ok(tracks)
}

fn parse_track_evidence_slice(bytes: &[u8], path: &Path) -> Result<Vec<GauntletTrackEvidence>, RunError> {
    parse_object_or_array_json_slice(bytes, path, "track evidence", "track array")
}

fn read_strict_regression_evidence_files(
    paths: &[PathBuf],
) -> Result<Vec<StrictHermeticityRegressionCaseEvidence>, RunError> {
    if paths.is_empty() {
        return Err(RunError::Internal(
            "strict hermeticity regression requires at least one --evidence file".to_string(),
        ));
    }
    let mut evidence = Vec::new();
    for path in paths {
        let bytes = read_file(path)?;
        let mut file_evidence = parse_strict_regression_evidence_slice(&bytes, path)?;
        evidence.append(&mut file_evidence);
    }
    Ok(evidence)
}

fn parse_strict_regression_evidence_slice(
    bytes: &[u8],
    path: &Path,
) -> Result<Vec<StrictHermeticityRegressionCaseEvidence>, RunError> {
    parse_object_or_array_json_slice(bytes, path, "strict regression evidence", "strict regression evidence array")
}

fn parse_object_or_array_json_slice<T: DeserializeOwned>(
    bytes: &[u8],
    path: &Path,
    object_label: &str,
    array_label: &str,
) -> Result<Vec<T>, RunError> {
    let value = serde_json::from_slice::<serde_json::Value>(bytes)
        .map_err(|err| RunError::Internal(format!("parsing {}: {err}", path.display())))?;
    if value.is_array() {
        return serde_json::from_value::<Vec<T>>(value)
            .map_err(|err| RunError::Internal(format!("parsing {} {array_label}: {err}", path.display())));
    }
    let item = serde_json::from_value::<T>(value)
        .map_err(|err| RunError::Internal(format!("parsing {} {object_label}: {err}", path.display())))?;
    Ok(vec![item])
}

fn read_json_file<T: DeserializeOwned>(path: &Path) -> Result<T, RunError> {
    let bytes = read_file(path)?;
    parse_json_slice(&bytes, &path.display().to_string())
}

fn parse_json_slice<T: DeserializeOwned>(bytes: &[u8], label: &str) -> Result<T, RunError> {
    serde_json::from_slice(bytes).map_err(|err| RunError::Internal(format!("parsing {label}: {err}")))
}

fn read_file(path: &Path) -> Result<Vec<u8>, RunError> {
    std::fs::read(path).map_err(|err| RunError::Internal(format!("reading {}: {err}", path.display())))
}

fn write_or_print_json(path: Option<&Path>, bytes: &[u8]) -> Result<(), RunError> {
    if let Some(path) = path {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|err| RunError::Internal(format!("creating {}: {err}", parent.display())))?;
        }
        std::fs::write(path, bytes).map_err(|err| RunError::Internal(format!("writing {}: {err}", path.display())))?;
        return Ok(());
    }
    println!(
        "{}",
        std::str::from_utf8(bytes).map_err(|err| RunError::Internal(format!("rendering canonical JSON: {err}")))?
    );
    Ok(())
}

fn print_canonicalize_summary(
    json: bool,
    kind: GauntletReportKind,
    input_path: &Path,
    output_path: Option<&Path>,
    digest_blake3: &str,
) -> Result<(), RunError> {
    let Some(output_path) = output_path else {
        return Ok(());
    };
    if json {
        let rendered = serde_json::json!({
            "kind": CANONICALIZE_SUMMARY_KIND,
            "report_kind": kind.as_str(),
            "input_path": input_path.display().to_string(),
            "output_path": output_path.display().to_string(),
            "digest_blake3": digest_blake3,
        });
        println!("{}", serde_json::to_string(&rendered).map_err(summary_ser_error)?);
        return Ok(());
    }
    println!("canonical gauntlet report: {}", output_path.display());
    println!("report kind: {}", kind.as_str());
    println!("report digest: {digest_blake3}");
    Ok(())
}

fn print_continuous_summary(
    json: bool,
    report: &ContinuousReproducibilityGauntletReport,
    output_path: &Path,
    digest_blake3: &str,
) -> Result<(), RunError> {
    if json {
        let rendered = serde_json::json!({
            "kind": CONTINUOUS_SUMMARY_KIND,
            "report_path": output_path.display().to_string(),
            "report_digest_blake3": digest_blake3,
            "current_claim_status": report.current_claim_status,
            "track_count": report.tracks.len(),
            "blocker_count": report.blockers.len(),
        });
        println!("{}", serde_json::to_string(&rendered).map_err(summary_ser_error)?);
        return Ok(());
    }
    println!("continuous gauntlet report: {}", output_path.display());
    println!("report digest: {digest_blake3}");
    println!("current claim status: {:?}", report.current_claim_status);
    println!("tracks: {}", report.tracks.len());
    println!("blockers: {}", report.blockers.len());
    Ok(())
}

fn print_strict_regression_summary(
    json: bool,
    report: &StrictHermeticityRegressionSuiteReport,
    output_path: &Path,
    digest_blake3: &str,
) -> Result<(), RunError> {
    if json {
        let rendered = serde_json::json!({
            "kind": STRICT_REGRESSION_SUMMARY_KIND,
            "report_path": output_path.display().to_string(),
            "report_digest_blake3": digest_blake3,
            "strict_regression_evidence_eligible": report.strict_regression_evidence_eligible,
            "case_count": report.cases.len(),
            "blocker_count": report.blockers.len(),
            "non_claim_count": report.non_claims.len(),
        });
        println!("{}", serde_json::to_string(&rendered).map_err(summary_ser_error)?);
        return Ok(());
    }
    println!("strict hermeticity regression report: {}", output_path.display());
    println!("report digest: {digest_blake3}");
    println!("eligible: {}", report.strict_regression_evidence_eligible);
    println!("cases: {}", report.cases.len());
    println!("blockers: {}", report.blockers.len());
    println!("non-claims: {}", report.non_claims.len());
    Ok(())
}

fn summary_ser_error(err: serde_json::Error) -> RunError {
    RunError::Internal(format!("serializing gauntlet summary: {err}"))
}

fn resolve_input_path(current_dir: &Path, path: PathBuf) -> PathBuf {
    if path.is_absolute() {
        return path;
    }
    current_dir.join(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    const STRICT_REGRESSION_AXIS_COUNT: usize = 9;
    const TEST_DIGEST_A: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

    fn strict_regression_axes() -> [&'static str; STRICT_REGRESSION_AXIS_COUNT] {
        [
            "clean-strict",
            "environment-poisoning",
            "path-poisoning",
            "undeclared-host-execution",
            "network-access",
            "missing-closure-facts",
            "umask-drift",
            "temp-root-dependence",
            "nondeterministic-output",
        ]
    }

    #[test]
    fn report_kind_labels_are_stable() {
        assert_eq!(GauntletReportKind::Continuous.as_str(), "continuous");
        assert_eq!(GauntletReportKind::ReleaseRepeatability.as_str(), "release-repeatability");
        assert_eq!(GauntletReportKind::StrictHermeticityRegression.as_str(), "strict-hermeticity-regression");
    }

    #[test]
    fn strict_regression_command_writes_eligible_report() {
        let temp = tempfile::tempdir().unwrap();
        let axes = strict_regression_axes();
        let plan_path = temp.path().join("plan.json");
        let evidence_path = temp.path().join("evidence.json");
        let report_path = temp.path().join("report.json");
        std::fs::write(&plan_path, serde_json::to_vec(&strict_regression_plan_json(&axes)).unwrap()).unwrap();
        std::fs::write(&evidence_path, serde_json::to_vec(&strict_regression_evidence_json(&axes)).unwrap()).unwrap();

        cmd_release_gauntlet(
            ReleaseGauntletAction::StrictHermeticityRegression {
                run_id: "strict-regression-run".to_string(),
                plan: plan_path,
                evidence: vec![evidence_path],
                report_path: Some(report_path.clone()),
            },
            temp.path(),
            false,
        )
        .unwrap();
        let report = read_json_file::<StrictHermeticityRegressionSuiteReport>(&report_path).unwrap();

        assert!(report.strict_regression_evidence_eligible);
        assert_eq!(report.cases.len(), STRICT_REGRESSION_AXIS_COUNT);
        assert!(report.blockers.is_empty());
        assert!(report.non_claims.is_empty());
    }

    fn strict_regression_plan_json(axes: &[&str]) -> serde_json::Value {
        let cases = axes.iter().map(|axis| strict_regression_plan_case_json(axis)).collect::<Vec<_>>();
        serde_json::json!({
            "schema": "mantle-strict-hermeticity-regression-suite-plan-v1",
            "suite_id": "edit-time-strict-hermeticity",
            "plan_version": "test-plan",
            "cases": cases,
        })
    }

    fn strict_regression_plan_case_json(axis: &str) -> serde_json::Value {
        if axis == "clean-strict" {
            return serde_json::json!({
                "case_id": axis,
                "perturbation_axis": axis,
                "mode": "strict",
                "expected_verdict": "accepted",
                "unsupported_host_non_claim": format!("untested-axis:{axis}"),
                "expected_output_digest_blake3": TEST_DIGEST_A,
            });
        }
        serde_json::json!({
            "case_id": axis,
            "perturbation_axis": axis,
            "mode": "strict",
            "expected_verdict": "blocked",
            "required_blocker_or_audit_class": format!("{axis}-blocked"),
            "unsupported_host_non_claim": format!("untested-axis:{axis}"),
        })
    }

    fn strict_regression_evidence_json(axes: &[&str]) -> serde_json::Value {
        let evidence = axes.iter().map(|axis| strict_regression_evidence_case_json(axis)).collect::<Vec<_>>();
        serde_json::Value::Array(evidence)
    }

    fn strict_regression_evidence_case_json(axis: &str) -> serde_json::Value {
        if axis == "clean-strict" {
            return serde_json::json!({
                "case_id": axis,
                "observed_verdict": "accepted",
                "output_digest_blake3": TEST_DIGEST_A,
                "repeated_output_digest_blake3": TEST_DIGEST_A,
            });
        }
        serde_json::json!({
            "case_id": axis,
            "observed_verdict": "blocked",
            "observed_blocker_or_audit_classes": [format!("{axis}-blocked")],
        })
    }
}
