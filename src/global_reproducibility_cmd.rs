use std::path::Path;
use std::path::PathBuf;

use crunch_release_core::GlobalReproducibilityClaimClass;
use crunch_release_core::GlobalReproducibilityEvaluationInput;
use crunch_release_core::GlobalReproducibilityPolicy;
use crunch_release_core::GlobalReproducibilityReport;
use crunch_release_core::GlobalReproducibilityUniverse;
use crunch_release_core::GlobalSurfaceEvidence;
use crunch_release_core::evaluate_global_reproducibility;
use crunch_release_core::global_reproducibility_policy_digest_blake3;
use crunch_release_core::global_reproducibility_report_canonical_bytes;
use crunch_release_core::global_reproducibility_report_digest_blake3;
use crunch_release_core::global_reproducibility_universe_digest_blake3;

use crate::errors::RunError;

const GLOBAL_REPRODUCIBILITY_OUTPUT_KIND: &str = "mantle-global-reproducibility-evaluation-v1";
const BLOCKED_GLOBAL_REPRODUCIBILITY_MESSAGE: &str = "global reproducibility claim blocked";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct GlobalReproducibilityCommandOutput {
    pub report: GlobalReproducibilityReport,
    pub report_digest_blake3: String,
    pub report_path: Option<PathBuf>,
}

impl GlobalReproducibilityCommandOutput {
    pub(crate) fn is_eligible(&self) -> bool {
        self.report.claim_class == GlobalReproducibilityClaimClass::Eligible
    }
}

pub(crate) fn render_global_reproducibility(
    output: &GlobalReproducibilityCommandOutput,
    json: bool,
) -> Result<(), RunError> {
    if json {
        print_global_reproducibility_json(output)?;
    } else {
        print_global_reproducibility_human(output);
    }
    if !output.is_eligible() {
        return Err(RunError::Internal(format!(
            "{BLOCKED_GLOBAL_REPRODUCIBILITY_MESSAGE}: {} blocker(s)",
            output.report.blockers.len()
        )));
    }
    Ok(())
}

pub(crate) fn evaluate_global_reproducibility_from_paths(
    current_dir: &Path,
    universe_path: PathBuf,
    policy_path: PathBuf,
    evidence_paths: Vec<PathBuf>,
    report_path: Option<PathBuf>,
) -> Result<GlobalReproducibilityCommandOutput, RunError> {
    debug_assert!(!GLOBAL_REPRODUCIBILITY_OUTPUT_KIND.is_empty());
    debug_assert!(!BLOCKED_GLOBAL_REPRODUCIBILITY_MESSAGE.is_empty());
    let universe = read_json_file::<GlobalReproducibilityUniverse>(&resolve_input_path(current_dir, universe_path))?;
    let policy = read_json_file::<GlobalReproducibilityPolicy>(&resolve_input_path(current_dir, policy_path))?;
    let universe_digest_blake3 = global_reproducibility_universe_digest_blake3(universe.clone()).map_err(core_error)?;
    let policy_digest_blake3 = global_reproducibility_policy_digest_blake3(policy.clone()).map_err(core_error)?;
    let surface_evidence = read_surface_evidence_files(current_dir, evidence_paths)?;
    let evaluation_result = evaluate_global_reproducibility(GlobalReproducibilityEvaluationInput {
        universe,
        policy,
        universe_digest_blake3,
        policy_digest_blake3,
        surface_evidence,
    })
    .map_err(core_error)?;
    let evaluation_digest_blake3 =
        global_reproducibility_report_digest_blake3(evaluation_result.clone()).map_err(core_error)?;
    let resolved_output_path = report_path.map(|path| resolve_input_path(current_dir, path));
    if let Some(path) = &resolved_output_path {
        write_report(path, evaluation_result.clone())?;
    }
    Ok(GlobalReproducibilityCommandOutput {
        report: evaluation_result,
        report_digest_blake3: evaluation_digest_blake3,
        report_path: resolved_output_path,
    })
}

fn read_surface_evidence_files(
    current_dir: &Path,
    evidence_paths: Vec<PathBuf>,
) -> Result<Vec<GlobalSurfaceEvidence>, RunError> {
    let mut evidence = Vec::with_capacity(evidence_paths.len());
    for path in evidence_paths {
        let resolved = resolve_input_path(current_dir, path);
        evidence.extend(read_surface_evidence_file(&resolved)?);
    }
    Ok(evidence)
}

fn read_surface_evidence_file(path: &Path) -> Result<Vec<GlobalSurfaceEvidence>, RunError> {
    let bytes = read_file(path)?;
    let value: serde_json::Value = serde_json::from_slice(&bytes)
        .map_err(|err| RunError::Internal(format!("parsing {}: {err}", path.display())))?;
    if value.is_array() {
        serde_json::from_value::<Vec<GlobalSurfaceEvidence>>(value)
            .map_err(|err| RunError::Internal(format!("parsing {}: {err}", path.display())))
    } else {
        let evidence = serde_json::from_value::<GlobalSurfaceEvidence>(value)
            .map_err(|err| RunError::Internal(format!("parsing {}: {err}", path.display())))?;
        Ok(vec![evidence])
    }
}

fn read_json_file<T>(path: &Path) -> Result<T, RunError>
where T: serde::de::DeserializeOwned {
    let bytes = read_file(path)?;
    serde_json::from_slice(&bytes).map_err(|err| RunError::Internal(format!("parsing {}: {err}", path.display())))
}

fn read_file(path: &Path) -> Result<Vec<u8>, RunError> {
    std::fs::read(path).map_err(|err| RunError::Internal(format!("reading {}: {err}", path.display())))
}

fn write_report(path: &Path, report: GlobalReproducibilityReport) -> Result<(), RunError> {
    let bytes = global_reproducibility_report_canonical_bytes(report).map_err(core_error)?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|err| RunError::Internal(format!("creating {}: {err}", parent.display())))?;
    }
    std::fs::write(path, bytes).map_err(|err| RunError::Internal(format!("writing {}: {err}", path.display())))
}

fn print_global_reproducibility_json(output: &GlobalReproducibilityCommandOutput) -> Result<(), RunError> {
    let rendered = serde_json::json!({
        "kind": GLOBAL_REPRODUCIBILITY_OUTPUT_KIND,
        "report_digest_blake3": output.report_digest_blake3,
        "report_path": output.report_path.as_ref().map(|path| path.display().to_string()),
        "report": output.report,
    });
    println!(
        "{}",
        serde_json::to_string(&rendered)
            .map_err(|err| RunError::Internal(format!("serializing global reproducibility output: {err}")))?
    );
    Ok(())
}

fn print_global_reproducibility_human(output: &GlobalReproducibilityCommandOutput) {
    println!("global reproducibility: {}", output.report.claim_class.as_str());
    println!("global report digest: {}", output.report_digest_blake3);
    if let Some(path) = &output.report_path {
        println!("global report: {}", path.display());
    }
    println!("universe digest: {}", output.report.universe_digest_blake3);
    println!("policy digest: {}", output.report.policy_digest_blake3);
    println!("included surfaces: {}", output.report.included_surface_count);
    println!("witness policy: {}", output.report.witness_policy);
    println!("boundary: claim applies only to this universe and policy, not future code or undeclared surfaces");
    for blocker in &output.report.blockers {
        println!("  blocker [{}]: {}", blocker.evidence_class, blocker.message);
    }
    for non_claim in &output.report.non_claims {
        println!("  non-claim: {non_claim}");
    }
}

fn resolve_input_path(current_dir: &Path, path: PathBuf) -> PathBuf {
    if path.is_absolute() {
        path
    } else {
        current_dir.join(path)
    }
}

fn core_error(error: crunch_release_core::ReleaseEvidenceError) -> RunError {
    RunError::Internal(error.to_string())
}

#[cfg(test)]
mod tests {
    use std::fs;

    use crunch_release_core::GLOBAL_REPRODUCIBILITY_POLICY_SCHEMA;
    use crunch_release_core::GLOBAL_REPRODUCIBILITY_SURFACE_EVIDENCE_SCHEMA;
    use crunch_release_core::GLOBAL_REPRODUCIBILITY_UNIVERSE_SCHEMA;
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::*;

    const SAMPLE_SIZE_BYTES: usize = 32;

    fn digest(seed: u8) -> String {
        blake3::hash(&[seed; SAMPLE_SIZE_BYTES]).to_hex().to_string()
    }

    fn write_json(path: &Path, value: serde_json::Value) {
        let bytes = serde_json::to_vec(&value).unwrap();
        fs::write(path, bytes).unwrap();
    }

    fn write_universe(path: &Path) {
        write_json(
            path,
            json!({
                "schema": GLOBAL_REPRODUCIBILITY_UNIVERSE_SCHEMA,
                "name": "tiny universe",
                "included_surfaces": [{
                    "id": "surface-a",
                    "target_system": "x86_64-linux",
                    "source_acquisition_mode": "git-archive",
                    "toolchain_route": "source-built-rust",
                    "cache_substitution_mode": "no-substitute",
                    "release_artifact_set": "binaries/01-mantle"
                }],
                "excluded_surfaces": []
            }),
        );
    }

    fn write_policy(path: &Path) {
        write_json(
            path,
            json!({
                "schema": GLOBAL_REPRODUCIBILITY_POLICY_SCHEMA,
                "policy_id": "single-witness",
                "witness_policy": "one-domain",
                "required_operator_domains": 1,
                "required_host_classes": 1,
                "required_perturbation_axes": ["PATH"],
                "require_strict_hermeticity": true,
                "require_fresh_rebuild_store": true
            }),
        );
    }

    fn write_evidence(root: &Path, universe_path: &Path, policy_path: &Path, evidence_path: &Path) {
        let universe = read_json_file::<GlobalReproducibilityUniverse>(universe_path).unwrap();
        let policy = read_json_file::<GlobalReproducibilityPolicy>(policy_path).unwrap();
        let universe_digest = global_reproducibility_universe_digest_blake3(universe).unwrap();
        let policy_digest = global_reproducibility_policy_digest_blake3(policy).unwrap();
        let output_digest = digest(10);
        let evidence = json!({
            "schema": GLOBAL_REPRODUCIBILITY_SURFACE_EVIDENCE_SCHEMA,
            "surface_id": "surface-a",
            "universe_digest_blake3": universe_digest,
            "policy_digest_blake3": policy_digest,
            "action_receipt_digest_blake3": digest(1),
            "source_acquisition_digest_blake3": digest(2),
            "toolchain_provenance_digest_blake3": digest(3),
            "hermeticity_evidence_digest_blake3": digest(4),
            "output_digest_set_blake3": [output_digest],
            "strict_hermeticity": true,
            "fresh_rebuild_store": true,
            "witnesses": [{
                "identity": "witness-a",
                "operator_domain": "domain-a",
                "host_class": "nixos",
                "perturbation_axes": ["PATH"],
                "trust_status": "valid",
                "output_digest_set_blake3": [digest(10)]
            }]
        });
        assert!(root.is_dir());
        write_json(evidence_path, evidence);
    }

    #[test]
    fn shell_loads_evidence_and_writes_canonical_report() {
        let temp = tempfile::tempdir().unwrap();
        let universe_path = temp.path().join("universe.json");
        let policy_path = temp.path().join("policy.json");
        let evidence_path = temp.path().join("evidence.json");
        let report_path = temp.path().join("reports/global.json");
        write_universe(&universe_path);
        write_policy(&policy_path);
        write_evidence(temp.path(), &universe_path, &policy_path, &evidence_path);

        let output = evaluate_global_reproducibility_from_paths(
            temp.path(),
            PathBuf::from("universe.json"),
            PathBuf::from("policy.json"),
            vec![PathBuf::from("evidence.json")],
            Some(PathBuf::from("reports/global.json")),
        )
        .unwrap();

        assert_eq!(output.report.claim_class, GlobalReproducibilityClaimClass::Eligible);
        assert_eq!(output.report_digest_blake3.len(), digest(0).len());
        assert!(report_path.is_file());
    }

    #[test]
    fn shell_allows_missing_evidence_but_report_blocks_global_claim() {
        let temp = tempfile::tempdir().unwrap();
        let universe_path = temp.path().join("universe.json");
        let policy_path = temp.path().join("policy.json");
        write_universe(&universe_path);
        write_policy(&policy_path);

        let output = evaluate_global_reproducibility_from_paths(
            temp.path(),
            PathBuf::from("universe.json"),
            PathBuf::from("policy.json"),
            Vec::new(),
            None,
        )
        .unwrap();

        assert_eq!(output.report.claim_class, GlobalReproducibilityClaimClass::Blocked);
        assert!(output.report.blockers.iter().any(|blocker| blocker.evidence_class == "surface-evidence"));
    }
}
