use std::path::Path;

use crunch_pipeline::BuildConfig;
use crunch_pipeline::PipelineResult;
use crunch_pipeline::drv_key_for;
use crunch_pipeline::label_for_key;
use serde::Serialize;

use crate::build_failure::BuildFailureEnvelope;
use crate::build_failure::build_failure_envelopes;
use crate::build_log::existing_log_file_path;

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
    pub failed: Vec<BuildFailureEnvelope>,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub substitution: Option<BuildJsonSubstitution>,
}

#[derive(Debug, Serialize)]
pub struct BuildJsonSubstitution {
    pub mode: String,
    pub transferred_bytes: u64,
    pub reused_bytes: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fallback_reason: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct BuildJsonAttestationReference {
    pub logical_path: String,
    pub path: String,
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
    let failure_reports = build_failure_envelopes(result, &config.store_dir, logs_dir);
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

fn build_counts(outcomes: &[BuildJsonOutcome], failed: &[BuildFailureEnvelope]) -> BuildJsonCounts {
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
                        substitution: outcome.substitutions.get(name).map(|report| BuildJsonSubstitution {
                            mode: report.mode.as_str().to_string(),
                            transferred_bytes: report.transferred_bytes,
                            reused_bytes: report.reused_bytes,
                            fallback_reason: report.fallback_reason.clone(),
                        }),
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

fn success_log_file(logs_dir: &Path, outcome: &crunch_build::BuildOutcome) -> Option<String> {
    if outcome.log.is_none() && outcome.cached {
        return None;
    }
    existing_log_file_path(logs_dir, &outcome.drv_path)
}

fn count_as_u32(count: usize) -> u32 {
    assert!(count <= u32::MAX as usize, "count must fit in u32");
    count as u32
}

#[cfg(test)]
mod tests {
    use serde_json::json;

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
        let failed = vec![BuildFailureEnvelope {
            root: "c".into(),
            drv_key: "c".into(),
            phase: crate::build_failure::FailurePhase::Build,
            error_class: crate::build_failure::FailureClass::Builder,
            message: "boom".into(),
            saved_log_path: Some("c.log".into()),
        }];

        let counts = build_counts(&outcomes, &failed);
        assert_eq!(counts.succeeded_total, 2);
        assert_eq!(counts.built_total, 1);
        assert_eq!(counts.cached_total, 1);
        assert_eq!(counts.failed_total, 1);
    }

    #[test]
    fn success_log_file_is_none_when_log_is_missing() {
        use std::collections::HashMap;

        let logs_dir = tempfile::tempdir().unwrap();
        let drv_path = nix_compat::store_path::StorePath::from_name_and_digest_fixed("demo.drv", [1u8; 20]).unwrap();
        let outcome = crunch_build::BuildOutcome {
            drv_path,
            outputs: HashMap::new(),
            substitutions: HashMap::new(),
            cached: false,
            log: Some("body".into()),
        };
        let log_path = success_log_file(logs_dir.path(), &outcome);
        assert!(log_path.is_none());
    }

    #[test]
    fn build_json_failure_report_uses_typed_envelope_schema() {
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
        let drv_path = nix_compat::store_path::StorePath::from_name_and_digest_fixed("demo.drv", [9u8; 20]).unwrap();
        crate::build_log::write_log_file(logs_dir.path(), &drv_path, "demo", false, "failure body");
        let drv_key = drv_key_for(&config.store_dir, &drv_path);
        let result = PipelineResult {
            outcomes: Vec::new(),
            failed: vec![crunch_build::FailedGoal {
                drv_key: drv_key.clone(),
                error: "FOD hash mismatch for demo: expected sha256-a, got sha256-b".to_string(),
            }],
            fod_mismatches: vec![crunch_pipeline::FodMismatch {
                name: "demo".to_string(),
                expected_sri: "sha256-a".to_string(),
                actual_sri: "sha256-b".to_string(),
            }],
            root_labels: HashMap::from([(drv_key.clone(), "demo".to_string())]),
            hermeticity_mode: crunch_pipeline::HermeticityMode::Practical,
            hermeticity_audit_events: Vec::new(),
        };

        let json_report = render_build_json_report(&config, &result, logs_dir.path()).unwrap();
        let json_value: serde_json::Value = serde_json::from_str(&json_report).unwrap();
        let saved_log_path = crate::build_log::existing_log_file_path(logs_dir.path(), &drv_path).unwrap();

        assert_eq!(
            json_value["failed"][0],
            json!({
                "root": "demo",
                "drv_key": drv_key,
                "phase": "build",
                "error_class": "fixed-output-hash-mismatch",
                "message": "FOD hash mismatch for demo: expected sha256-a, got sha256-b",
                "saved_log_path": saved_log_path,
            })
        );
    }

    #[test]
    fn build_json_preflight_failure_omits_saved_log_path_field() {
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
        let drv_path = nix_compat::store_path::StorePath::from_name_and_digest_fixed("demo.drv", [10u8; 20]).unwrap();
        let drv_key = drv_key_for(&config.store_dir, &drv_path);
        let result = PipelineResult {
            outcomes: Vec::new(),
            failed: vec![crunch_build::FailedGoal {
                drv_key: drv_key.clone(),
                error: "strict mode does not permit in-memory PathInfo fallback: broken redb".to_string(),
            }],
            fod_mismatches: Vec::new(),
            root_labels: HashMap::from([(drv_key.clone(), "demo".to_string())]),
            hermeticity_mode: crunch_pipeline::HermeticityMode::Practical,
            hermeticity_audit_events: Vec::new(),
        };

        let json_report = render_build_json_report(&config, &result, logs_dir.path()).unwrap();
        let json_value: serde_json::Value = serde_json::from_str(&json_report).unwrap();
        assert_eq!(json_value["failed"][0]["phase"], "preflight");
        assert_eq!(json_value["failed"][0]["error_class"], "preflight");
        assert!(json_value["failed"][0].get("saved_log_path").is_none());
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
            substitutions: HashMap::from([("out".to_string(), crunch_store::OutputSubstitutionReport {
                mode: crunch_store::OutputSubstitutionMode::Delta,
                transferred_bytes: 12,
                reused_bytes: 34,
                fallback_reason: None,
            })]),
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
        let substitution = report.outcomes[0].outputs[0].substitution.as_ref().expect("substitution report");
        assert_eq!(substitution.mode, "delta");
        assert_eq!(substitution.transferred_bytes, 12);
        assert_eq!(substitution.reused_bytes, 34);
        assert!(substitution.fallback_reason.is_none());
    }

    #[test]
    fn render_build_json_report_serializes_full_substitution_fields_stably() {
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
        let drv_path = nix_compat::store_path::StorePath::from_name_and_digest_fixed("demo.drv", [3u8; 20]).unwrap();
        let output_path = nix_compat::store_path::StorePath::from_name_and_digest_fixed("demo", [4u8; 20]).unwrap();
        let logical_path = output_path.to_absolute_path_with_prefix(&config.store_dir);
        let exported_path = output_path.to_absolute_path_with_prefix(config.output_dir.to_str().unwrap());
        let attestation_path =
            crunch_store::artifact_attestation_file_path(state_dir.path(), &config.store_dir, &output_path)
                .display()
                .to_string();
        let path_info = snix_store::path_info::PathInfo {
            store_path: output_path.clone(),
            node: snix_castore::Node::Symlink {
                target: snix_castore::SymlinkTarget::try_from("target").unwrap(),
            },
            references: vec![],
            nar_size: 1,
            nar_sha256: [0x22; 32],
            signatures: vec![],
            deriver: None,
            ca: None,
        };
        let outcome = crunch_build::BuildOutcome {
            drv_path: drv_path.clone(),
            outputs: HashMap::from([("out".to_string(), path_info)]),
            substitutions: HashMap::from([("out".to_string(), crunch_store::OutputSubstitutionReport {
                mode: crunch_store::OutputSubstitutionMode::Full,
                transferred_bytes: 55,
                reused_bytes: 0,
                fallback_reason: Some("stream_application_failed".to_string()),
            })]),
            cached: true,
            log: None,
        };
        let result = PipelineResult {
            outcomes: vec![outcome],
            failed: Vec::new(),
            fod_mismatches: Vec::new(),
            root_labels: HashMap::from([(drv_key_for(&config.store_dir, &drv_path), "demo".to_string())]),
            hermeticity_mode: crunch_pipeline::HermeticityMode::Practical,
            hermeticity_audit_events: Vec::new(),
        };

        let json_report = render_build_json_report(&config, &result, logs_dir.path()).unwrap();
        let json_value: serde_json::Value = serde_json::from_str(&json_report).unwrap();

        assert_eq!(
            json_value,
            json!({
                "schema": "crunch-build-report-v1",
                "file": config.file.display().to_string(),
                "output_dir": config.output_dir.display().to_string(),
                "state_dir": config.state_dir.display().to_string(),
                "store_dir": config.store_dir.clone(),
                "hermeticity_mode": "practical",
                "hermeticity_audit_events": [],
                "counts": {
                    "succeeded_total": 1,
                    "built_total": 0,
                    "cached_total": 1,
                    "failed_total": 0,
                },
                "outcomes": [{
                    "drv_key": drv_key_for(&config.store_dir, &drv_path),
                    "label": "demo",
                    "cached": true,
                    "log_file": null,
                    "outputs": [{
                        "name": "out",
                        "path": exported_path,
                        "artifact_attestation": {
                            "logical_path": logical_path.clone(),
                            "path": attestation_path,
                        },
                        "substitution": {
                            "mode": "full",
                            "transferred_bytes": 55,
                            "reused_bytes": 0,
                            "fallback_reason": "stream_application_failed",
                        }
                    }]
                }],
                "failed": [],
                "fod_mismatches": [],
            })
        );
    }
}
