use std::path::Path;

use crunch_pipeline::BuildConfig;
use crunch_pipeline::PipelineResult;
use crunch_pipeline::drv_key_for;
use crunch_pipeline::label_for_key;
use serde::Serialize;

use crate::build_log::existing_log_file_path;
use crate::build_log::existing_log_file_path_from_drv_key;

#[derive(Debug, Serialize)]
pub struct BuildJsonReport {
    pub schema: &'static str,
    pub file: String,
    pub output_dir: String,
    pub state_dir: String,
    pub store_dir: String,
    pub hermeticity_mode: String,
    pub hermeticity_audit_events: Vec<BuildJsonHermeticityAuditEvent>,
    pub counts: BuildJsonCounts,
    pub outcomes: Vec<BuildJsonOutcome>,
    pub failed: Vec<BuildJsonFailure>,
    pub fod_mismatches: Vec<BuildJsonFodMismatch>,
}

#[derive(Debug, Serialize)]
pub struct BuildJsonCounts {
    pub succeeded_total: u32,
    pub built_total: u32,
    pub cached_total: u32,
    pub failed_total: u32,
}

#[derive(Debug, Serialize)]
pub struct BuildJsonHermeticityAuditEvent {
    pub kind: String,
    pub detail: String,
}

#[derive(Debug, Serialize)]
pub struct BuildJsonOutcome {
    pub drv_key: String,
    pub label: String,
    pub cached: bool,
    pub log_file: Option<String>,
    pub outputs: Vec<BuildJsonOutput>,
}

#[derive(Debug, Serialize)]
pub struct BuildJsonOutput {
    pub name: String,
    pub path: String,
    pub artifact_attestation: BuildJsonAttestationReference,
}

#[derive(Debug, Serialize)]
pub struct BuildJsonAttestationReference {
    pub logical_path: String,
    pub path: String,
}

#[derive(Debug, Serialize)]
pub struct BuildJsonFailure {
    pub drv_key: String,
    pub label: String,
    pub error: String,
    pub log_file: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct BuildJsonFodMismatch {
    pub name: String,
    pub expected_sri: String,
    pub actual_sri: String,
}

pub fn render_build_json_report(
    config: &BuildConfig,
    result: &PipelineResult,
    logs_dir: &Path,
) -> Result<String, serde_json::Error> {
    let report = build_json_report(config, result, logs_dir);
    serde_json::to_string_pretty(&report)
}

fn build_json_report(config: &BuildConfig, result: &PipelineResult, logs_dir: &Path) -> BuildJsonReport {
    debug_assert_eq!(config.hermeticity_mode, result.hermeticity_mode, "config/result hermeticity modes must match");
    let outcome_reports = build_outcome_reports(config, result, logs_dir);
    let failure_reports = build_failure_reports(result, &config.store_dir, logs_dir);
    let counts = build_counts(&outcome_reports, &failure_reports);
    let hermeticity_audit_events = result
        .hermeticity_audit_events
        .iter()
        .map(|event| BuildJsonHermeticityAuditEvent {
            kind: event.kind.as_str().to_string(),
            detail: event.detail.clone(),
        })
        .collect();
    let fod_mismatches = result
        .fod_mismatches
        .iter()
        .map(|mismatch| BuildJsonFodMismatch {
            name: mismatch.name.clone(),
            expected_sri: mismatch.expected_sri.clone(),
            actual_sri: mismatch.actual_sri.clone(),
        })
        .collect();

    BuildJsonReport {
        schema: "crunch-build-report-v1",
        file: config.file.display().to_string(),
        output_dir: config.output_dir.display().to_string(),
        state_dir: config.state_dir.display().to_string(),
        store_dir: config.store_dir.clone(),
        hermeticity_mode: result.hermeticity_mode.as_str().to_string(),
        hermeticity_audit_events,
        counts,
        outcomes: outcome_reports,
        failed: failure_reports,
        fod_mismatches,
    }
}

fn build_counts(outcomes: &[BuildJsonOutcome], failed: &[BuildJsonFailure]) -> BuildJsonCounts {
    let succeeded_total = count_as_u32(outcomes.len());
    let failed_total = count_as_u32(failed.len());
    let cached_total = count_as_u32(outcomes.iter().filter(|outcome| outcome.cached).count());
    let built_total = succeeded_total.saturating_sub(cached_total);
    BuildJsonCounts {
        succeeded_total,
        built_total,
        cached_total,
        failed_total,
    }
}

fn build_outcome_reports(config: &BuildConfig, result: &PipelineResult, logs_dir: &Path) -> Vec<BuildJsonOutcome> {
    let output_dir_str = config.output_dir.to_str().unwrap_or(&config.store_dir);
    let mut reports: Vec<BuildJsonOutcome> = result
        .outcomes
        .iter()
        .map(|outcome| {
            let drv_key = drv_key_for(&config.store_dir, &outcome.drv_path);
            let label = label_for_key(result, &drv_key).unwrap_or(outcome.drv_path.name()).to_string();
            let mut outputs: Vec<BuildJsonOutput> = outcome
                .outputs
                .iter()
                .map(|(name, path_info)| {
                    let logical_path = path_info.store_path.to_absolute_path_with_prefix(&config.store_dir);
                    let attestation_path = crunch_store::artifact_attestation_file_path(
                        &config.state_dir,
                        &config.store_dir,
                        &path_info.store_path,
                    );
                    BuildJsonOutput {
                        name: name.clone(),
                        path: path_info.store_path.to_absolute_path_with_prefix(output_dir_str),
                        artifact_attestation: BuildJsonAttestationReference {
                            logical_path,
                            path: attestation_path.display().to_string(),
                        },
                    }
                })
                .collect();
            outputs.sort_by(|left, right| left.name.cmp(&right.name));
            let log_file = success_log_file(logs_dir, outcome);
            BuildJsonOutcome {
                drv_key,
                label,
                cached: outcome.cached,
                log_file,
                outputs,
            }
        })
        .collect();
    reports.sort_by(|left, right| left.label.cmp(&right.label).then(left.drv_key.cmp(&right.drv_key)));
    reports
}

fn build_failure_reports(result: &PipelineResult, store_dir: &str, logs_dir: &Path) -> Vec<BuildJsonFailure> {
    let mut reports: Vec<BuildJsonFailure> = result
        .failed
        .iter()
        .map(|failed| BuildJsonFailure {
            drv_key: failed.drv_key.clone(),
            label: label_for_key(result, &failed.drv_key).unwrap_or(&failed.drv_key).to_string(),
            error: failed.error.clone(),
            log_file: failure_log_file(logs_dir, store_dir, &failed.drv_key),
        })
        .collect();
    reports.sort_by(|left, right| left.label.cmp(&right.label).then(left.drv_key.cmp(&right.drv_key)));
    reports
}

fn success_log_file(logs_dir: &Path, outcome: &crunch_build::BuildOutcome) -> Option<String> {
    if outcome.log.is_none() && outcome.cached {
        return None;
    }
    existing_log_file_path(logs_dir, &outcome.drv_path)
}

fn failure_log_file(logs_dir: &Path, store_dir: &str, drv_key: &str) -> Option<String> {
    existing_log_file_path_from_drv_key(logs_dir, store_dir, drv_key)
}

fn count_as_u32(count: usize) -> u32 {
    assert!(count <= u32::MAX as usize, "count must fit in u32");
    count as u32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn count_as_u32_round_trips_small_values() {
        assert_eq!(count_as_u32(0), 0);
        assert_eq!(count_as_u32(7), 7);
    }

    #[test]
    fn build_counts_splits_cached_and_built() {
        let outcomes = vec![
            BuildJsonOutcome {
                drv_key: "a".into(),
                label: "a".into(),
                cached: false,
                log_file: None,
                outputs: Vec::new(),
            },
            BuildJsonOutcome {
                drv_key: "b".into(),
                label: "b".into(),
                cached: true,
                log_file: None,
                outputs: Vec::new(),
            },
        ];
        let failed = vec![BuildJsonFailure {
            drv_key: "c".into(),
            label: "c".into(),
            error: "boom".into(),
            log_file: Some("c.log".into()),
        }];

        let counts = build_counts(&outcomes, &failed);
        assert_eq!(counts.succeeded_total, 2);
        assert_eq!(counts.built_total, 1);
        assert_eq!(counts.cached_total, 1);
        assert_eq!(counts.failed_total, 1);
    }

    #[test]
    fn failure_log_file_is_none_for_unparsable_drv_key() {
        let logs_dir = tempfile::tempdir().unwrap();
        let log_path = failure_log_file(logs_dir.path(), "/crunch/store", "not-a-drv-key");
        assert!(log_path.is_none());
    }

    #[test]
    fn success_log_file_is_none_when_log_is_missing() {
        use std::collections::HashMap;

        let logs_dir = tempfile::tempdir().unwrap();
        let drv_path = nix_compat::store_path::StorePath::from_name_and_digest_fixed("demo.drv", [1u8; 20]).unwrap();
        let outcome = crunch_build::BuildOutcome {
            drv_path,
            outputs: HashMap::new(),
            cached: false,
            log: Some("body".into()),
        };
        let log_path = success_log_file(logs_dir.path(), &outcome);
        assert!(log_path.is_none());
    }

    #[test]
    fn build_json_report_includes_artifact_attestation_reference() {
        use std::collections::HashMap;

        let state_dir = tempfile::tempdir().unwrap();
        let output_dir = tempfile::tempdir().unwrap();
        let logs_dir = tempfile::tempdir().unwrap();
        let signing_key = crate::build_cmd::load_or_generate_signing_keypair(None, state_dir.path(), false).unwrap();
        let config = BuildConfig {
            file: output_dir.path().join("demo.ncl"),
            import_paths: Vec::new(),
            output_dir: output_dir.path().to_path_buf(),
            state_dir: state_dir.path().to_path_buf(),
            store_dir: "/crunch/store".to_string(),
            verbose: false,
            max_jobs: 1,
            substituter_url: None,
            hermeticity_mode: crunch_pipeline::HermeticityMode::Practical,
            keypair: signing_key,
            trusted_keys: Vec::new(),
            trust_unsigned: false,
            root_retention_source: None,
        };
        let drv_path = nix_compat::store_path::StorePath::from_name_and_digest_fixed("demo.drv", [1u8; 20]).unwrap();
        let output_path = nix_compat::store_path::StorePath::from_name_and_digest_fixed("demo", [2u8; 20]).unwrap();
        let path_info = snix_store::path_info::PathInfo {
            store_path: output_path.clone(),
            node: snix_castore::Node::Symlink {
                target: snix_castore::SymlinkTarget::try_from("target").unwrap(),
            },
            references: vec![],
            nar_size: 1,
            nar_sha256: [0x11; 32],
            signatures: vec![],
            deriver: None,
            ca: None,
        };
        let outcome = crunch_build::BuildOutcome {
            drv_path: drv_path.clone(),
            outputs: HashMap::from([("out".to_string(), path_info)]),
            cached: false,
            log: None,
        };
        let result = PipelineResult {
            outcomes: vec![outcome],
            failed: Vec::new(),
            fod_mismatches: Vec::new(),
            root_labels: HashMap::from([(drv_key_for(&config.store_dir, &drv_path), "demo".to_string())]),
            hermeticity_mode: crunch_pipeline::HermeticityMode::Practical,
            hermeticity_audit_events: vec![crunch_pipeline::HermeticityAuditEvent::new(
                crunch_pipeline::HermeticityAuditKind::HostToolFallback,
                "using external bwrap",
            )],
        };

        let report = build_json_report(&config, &result, logs_dir.path());
        let logical_path = output_path.to_absolute_path_with_prefix(&config.store_dir);

        assert_eq!(report.hermeticity_mode, "practical");
        assert_eq!(report.hermeticity_audit_events.len(), 1);
        assert_eq!(report.hermeticity_audit_events[0].kind, "host-tool-fallback");
        assert_eq!(report.outcomes.len(), 1);
        assert_eq!(report.outcomes[0].outputs.len(), 1);
        assert_eq!(report.outcomes[0].outputs[0].artifact_attestation.logical_path, logical_path);
        assert!(report.outcomes[0].outputs[0].artifact_attestation.path.contains("attestations/artifacts/"));
    }
}
