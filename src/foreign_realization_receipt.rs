//! Deterministic receipts for receipt-bound foreign realization.
// r[impl foreign_derivation_import.realization_receipt]

use std::collections::BTreeMap;
use std::collections::BTreeSet;

use crunch_build::FOREIGN_FETCH_ATTEMPT_LOG_SCHEMA;
use crunch_build::ForeignFetchAttemptLog;
use crunch_pipeline::RegisteredBuildResult;
use data_encoding::HEXLOWER;
use serde::Deserialize;
use serde::Serialize;

use crate::RunError;
use crate::foreign_executable_plan::ForeignExecutablePlan;
use crate::foreign_graph_compiler::CompiledForeignBuiltin;
use crate::foreign_realization_shell::ForeignSourceStoreFact;

pub(crate) const FOREIGN_REALIZATION_RECEIPT_SCHEMA: &str = "mantle-foreign-realization-receipt-v1";
const FOREIGN_REALIZATION_RECEIPT_DIGEST_DOMAIN: &[u8] = b"mantle-foreign-realization-receipt-v1\0";
pub(crate) const FOREIGN_REALIZATION_COMPLETE_STATUS: &str = "complete";
const PARTIAL_FAILURE_STATUS: &str = "partial-failure";
pub(crate) const FOREIGN_REALIZATION_REALIZED_STATE: &str = "realized";
const PARTIAL_REALIZATION_STRONGEST_STATE: &str = "partial-realization";
const FOREIGN_BUILD_REPORT_DIGEST_DOMAIN: &[u8] = b"mantle-foreign-build-report-v1\0";
const REALIZATION_NON_CLAIMS: [&str; 5] = [
    "This receipt does not prove foreign evaluator equivalence.",
    "This receipt does not prove package correctness.",
    "This receipt does not prove reproducibility beyond the recorded run.",
    "This receipt does not trust outputs only because a foreign derivation named their paths.",
    "This receipt does not grant remote execution authority.",
];

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ForeignRealizationReceipt {
    pub(crate) schema: String,
    pub(crate) status: String,
    pub(crate) strongest_state: String,
    pub(crate) receipt_blake3: String,
    pub(crate) plan_blake3: String,
    pub(crate) import_receipt_blake3: String,
    pub(crate) source_bundle_manifest_blake3: String,
    pub(crate) build_report_blake3: String,
    pub(crate) selected_root_node_ids: Vec<String>,
    pub(crate) selected_root_paths: Vec<String>,
    pub(crate) execution_profiles: Vec<RealizedExecutionProfile>,
    pub(crate) cache_policy: RealizedCachePolicy,
    pub(crate) sources: Vec<ForeignSourceStoreFact>,
    pub(crate) units: Vec<ForeignRealizedUnit>,
    pub(crate) failure: Option<ForeignRealizationFailure>,
    pub(crate) non_claims: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RealizedExecutionProfile {
    pub(crate) profile_id: String,
    pub(crate) digest_blake3: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RealizedCachePolicy {
    pub(crate) substitution_enabled: bool,
    pub(crate) offline: bool,
    pub(crate) ordered_cache_urls: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ForeignRealizedUnit {
    pub(crate) sequence: usize,
    pub(crate) node_id: String,
    pub(crate) target_derivation: String,
    pub(crate) execution_profile_id: String,
    pub(crate) execution_profile_digest_blake3: String,
    pub(crate) execution_class: String,
    pub(crate) fetch_attempts: Vec<crunch_build::ForeignFetchAttempt>,
    pub(crate) outputs: Vec<ForeignRealizedOutput>,
    pub(crate) failure: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ForeignRealizedOutput {
    pub(crate) output_name: String,
    pub(crate) target_path: String,
    pub(crate) nar_sha256: String,
    pub(crate) nar_size: u64,
    pub(crate) substitution_mode: Option<String>,
    pub(crate) substitution_transferred_bytes: Option<u64>,
    pub(crate) substitution_reused_bytes: Option<u64>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ForeignRealizationFailure {
    pub(crate) class: String,
    pub(crate) message: String,
    pub(crate) affected_roots: Vec<String>,
    pub(crate) completed_unit_count: usize,
}

pub(crate) struct ForeignReceiptInput<'a> {
    pub(crate) plan: &'a ForeignExecutablePlan,
    pub(crate) source_bundle_manifest_blake3: &'a str,
    pub(crate) selected_root_node_ids: &'a [String],
    pub(crate) source_facts: &'a [ForeignSourceStoreFact],
    pub(crate) build_result: &'a RegisteredBuildResult,
    pub(crate) substitution_enabled: bool,
    pub(crate) offline: bool,
    pub(crate) ordered_cache_urls: &'a [String],
}

pub(crate) fn build_foreign_realization_receipt(
    input: ForeignReceiptInput<'_>,
) -> Result<ForeignRealizationReceipt, RunError> {
    let outcome_by_drv = input
        .build_result
        .outcomes
        .iter()
        .map(|outcome| (outcome.drv_path.to_absolute_path_with_prefix(&input.plan.target_store_prefix), outcome))
        .collect::<BTreeMap<_, _>>();
    let failed_by_drv = input
        .build_result
        .failed_roots
        .iter()
        .map(|failed| (failed.drv_key.as_str(), failed))
        .collect::<BTreeMap<_, _>>();
    let failed_log_by_origin = input
        .build_result
        .failed_roots
        .iter()
        .filter_map(|failed| failed.build_log.as_deref().map(|build_log| (failed.origin_drv_key.as_str(), build_log)))
        .collect::<BTreeMap<_, _>>();
    let output_by_unit_and_name = input
        .build_result
        .outputs
        .iter()
        .map(|output| ((output.unit_id.as_str(), output.output_name.as_str()), output))
        .collect::<BTreeMap<_, _>>();
    let mut profiles = BTreeSet::new();
    let mut units = Vec::with_capacity(input.plan.native_units.len());
    for unit in &input.plan.native_units {
        profiles.insert((unit.execution_profile_id.clone(), unit.execution_profile_digest_blake3.clone()));
        let outcome = outcome_by_drv.get(&unit.target_derivation).copied();
        let failed_root = failed_by_drv.get(unit.target_derivation.as_str()).copied();
        let failure = failed_root.map(|failed| failed.error.as_str());
        let build_log = outcome
            .and_then(|value| value.log.as_deref())
            .or_else(|| failed_log_by_origin.get(unit.target_derivation.as_str()).copied());
        let fetch_attempts = parse_fetch_attempts(build_log)?;
        let mut outputs = Vec::with_capacity(unit.outputs.len());
        for (output_name, target_path) in &unit.outputs {
            let realized = output_by_unit_and_name.get(&(unit.node_id.as_str(), output_name.as_str())).copied();
            if outcome.is_some() && realized.is_none() {
                return Err(RunError::Internal(format!(
                    "successful foreign realization has no output fact for {}:{}",
                    unit.node_id, output_name
                )));
            }
            if let Some(realized) = realized {
                let substitution = outcome.and_then(|value| value.substitutions.get(output_name));
                outputs.push(ForeignRealizedOutput {
                    output_name: output_name.clone(),
                    target_path: target_path.clone(),
                    nar_sha256: HEXLOWER.encode(&realized.path_info.nar_sha256),
                    nar_size: realized.path_info.nar_size,
                    substitution_mode: substitution.map(|report| report.mode.as_str().to_string()),
                    substitution_transferred_bytes: substitution.map(|report| report.transferred_bytes),
                    substitution_reused_bytes: substitution.map(|report| report.reused_bytes),
                });
            }
        }
        let effective_failure = failure.map(str::to_string).or_else(|| {
            if outcome.is_none() && outputs.is_empty() && !input.build_result.failed_roots.is_empty() {
                Some("unit has no store output after a selected root failure".to_string())
            } else {
                None
            }
        });
        if outcome.is_none() && effective_failure.is_none() && outputs.is_empty() {
            return Err(RunError::Internal(format!(
                "foreign realization has no scheduler or store disposition for {}",
                unit.node_id
            )));
        }
        let execution_class = if effective_failure.is_some() {
            "failed-or-blocked"
        } else if outcome.is_none() {
            "already-present"
        } else if outcome.is_some_and(|value| !value.substitutions.is_empty()) {
            "substituted"
        } else if outcome.is_some_and(|value| value.cached) {
            "already-present"
        } else if matches!(
            unit.builtin,
            CompiledForeignBuiltin::Download { .. } | CompiledForeignBuiltin::GitDownload { .. }
        ) {
            "fetched"
        } else {
            "built"
        };
        units.push(ForeignRealizedUnit {
            sequence: unit.sequence,
            node_id: unit.node_id.clone(),
            target_derivation: unit.target_derivation.clone(),
            execution_profile_id: unit.execution_profile_id.clone(),
            execution_profile_digest_blake3: unit.execution_profile_digest_blake3.clone(),
            execution_class: execution_class.to_string(),
            fetch_attempts,
            outputs,
            failure: effective_failure,
        });
    }
    let selected_root_paths = input
        .plan
        .selected_roots
        .iter()
        .filter(|root| input.selected_root_node_ids.binary_search(&root.node_id).is_ok())
        .flat_map(|root| root.target_outputs.values().cloned())
        .collect::<Vec<_>>();
    let failure = if input.build_result.failed_roots.is_empty() {
        None
    } else {
        Some(ForeignRealizationFailure {
            class: "foreign-root-build-failed".to_string(),
            message: input
                .build_result
                .failed_roots
                .iter()
                .map(|failed| format!("{}: {}", failed.drv_key, failed.error))
                .collect::<Vec<_>>()
                .join("; "),
            affected_roots: input.build_result.failed_roots.iter().map(|failed| failed.drv_key.clone()).collect(),
            completed_unit_count: input.build_result.outcomes.len(),
        })
    };
    let status = if failure.is_some() {
        PARTIAL_FAILURE_STATUS
    } else {
        FOREIGN_REALIZATION_COMPLETE_STATUS
    };
    let mut receipt = ForeignRealizationReceipt {
        schema: FOREIGN_REALIZATION_RECEIPT_SCHEMA.to_string(),
        status: status.to_string(),
        strongest_state: if failure.is_some() {
            PARTIAL_REALIZATION_STRONGEST_STATE.to_string()
        } else {
            FOREIGN_REALIZATION_REALIZED_STATE.to_string()
        },
        receipt_blake3: String::new(),
        plan_blake3: input.plan.plan_identity.value.clone(),
        import_receipt_blake3: input.plan.accepted_import.receipt_digest_blake3.clone(),
        source_bundle_manifest_blake3: input.source_bundle_manifest_blake3.to_string(),
        build_report_blake3: foreign_build_report_digest(input.build_result)?,
        selected_root_node_ids: input.selected_root_node_ids.to_vec(),
        selected_root_paths,
        execution_profiles: profiles
            .into_iter()
            .map(|(profile_id, digest_blake3)| RealizedExecutionProfile {
                profile_id,
                digest_blake3,
            })
            .collect(),
        cache_policy: RealizedCachePolicy {
            substitution_enabled: input.substitution_enabled,
            offline: input.offline,
            ordered_cache_urls: input.ordered_cache_urls.to_vec(),
        },
        sources: input.source_facts.to_vec(),
        units,
        failure,
        non_claims: REALIZATION_NON_CLAIMS.iter().map(|value| (*value).to_string()).collect(),
    };
    receipt.receipt_blake3 = foreign_realization_receipt_digest(&receipt)?;
    Ok(receipt)
}

fn foreign_build_report_digest(result: &RegisteredBuildResult) -> Result<String, RunError> {
    #[derive(Serialize)]
    struct OutcomeMaterial {
        drv_path: String,
        cached: bool,
        outputs: Vec<OutputMaterial>,
        substitutions: Vec<SubstitutionMaterial>,
    }
    #[derive(Serialize)]
    struct OutputMaterial {
        output_name: String,
        store_path: String,
        nar_sha256: String,
        nar_size: u64,
    }
    #[derive(Serialize)]
    struct SubstitutionMaterial {
        output_name: String,
        mode: String,
        transferred_bytes: u64,
        reused_bytes: u64,
    }
    #[derive(Serialize)]
    struct FailureMaterial<'a> {
        drv_key: &'a str,
        origin_drv_key: &'a str,
        error: &'a str,
        build_log: Option<&'a str>,
    }
    #[derive(Serialize)]
    struct ReportMaterial<'a> {
        outcomes: Vec<OutcomeMaterial>,
        failures: Vec<FailureMaterial<'a>>,
    }

    let outcomes = result
        .outcomes
        .iter()
        .map(|outcome| OutcomeMaterial {
            drv_path: outcome.drv_path.to_string(),
            cached: outcome.cached,
            outputs: outcome
                .outputs
                .iter()
                .map(|(output_name, path_info)| OutputMaterial {
                    output_name: output_name.clone(),
                    store_path: path_info.store_path.to_string(),
                    nar_sha256: HEXLOWER.encode(&path_info.nar_sha256),
                    nar_size: path_info.nar_size,
                })
                .collect(),
            substitutions: outcome
                .substitutions
                .iter()
                .map(|(output_name, report)| SubstitutionMaterial {
                    output_name: output_name.clone(),
                    mode: report.mode.as_str().to_string(),
                    transferred_bytes: report.transferred_bytes,
                    reused_bytes: report.reused_bytes,
                })
                .collect(),
        })
        .collect();
    let failures = result
        .failed_roots
        .iter()
        .map(|failed| FailureMaterial {
            drv_key: &failed.drv_key,
            origin_drv_key: &failed.origin_drv_key,
            error: &failed.error,
            build_log: failed.build_log.as_deref(),
        })
        .collect();
    let canonical = serde_json::to_vec(&ReportMaterial { outcomes, failures })
        .map_err(|error| RunError::Internal(format!("serializing foreign build report: {error}")))?;
    let mut hasher = blake3::Hasher::new();
    hasher.update(FOREIGN_BUILD_REPORT_DIGEST_DOMAIN);
    hasher.update(&canonical);
    Ok(hasher.finalize().to_hex().to_string())
}

pub(crate) fn foreign_realization_receipt_digest(receipt: &ForeignRealizationReceipt) -> Result<String, RunError> {
    let mut material = receipt.clone();
    material.receipt_blake3.clear();
    let canonical = serde_json::to_vec(&material)
        .map_err(|error| RunError::Internal(format!("serializing foreign realization receipt: {error}")))?;
    let mut hasher = blake3::Hasher::new();
    hasher.update(FOREIGN_REALIZATION_RECEIPT_DIGEST_DOMAIN);
    hasher.update(&canonical);
    Ok(hasher.finalize().to_hex().to_string())
}

fn parse_fetch_attempts(log: Option<&str>) -> Result<Vec<crunch_build::ForeignFetchAttempt>, RunError> {
    let Some(log) = log else {
        return Ok(Vec::new());
    };
    let Ok(parsed) = serde_json::from_str::<ForeignFetchAttemptLog>(log) else {
        return Ok(Vec::new());
    };
    if parsed.schema != FOREIGN_FETCH_ATTEMPT_LOG_SCHEMA {
        return Err(RunError::Internal("foreign fetch attempt log schema changed during realization".to_string()));
    }
    Ok(parsed.attempts)
}
