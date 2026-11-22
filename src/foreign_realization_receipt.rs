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
use crate::foreign_realization_shell::CacheClosureHydration;
use crate::foreign_realization_shell::ForeignCacheClosureFact;
use crate::foreign_realization_shell::ForeignSourceStoreFact;

pub(crate) const FOREIGN_REALIZATION_RECEIPT_SCHEMA: &str = "mantle-foreign-realization-receipt-v1";
const FOREIGN_REALIZATION_RECEIPT_DIGEST_DOMAIN: &[u8] = b"mantle-foreign-realization-receipt-v1\0";
pub(crate) const FOREIGN_REALIZATION_RECEIPT_V2_SCHEMA: &str = "mantle-foreign-realization-receipt-v2";
const FOREIGN_REALIZATION_RECEIPT_V2_DIGEST_DOMAIN: &[u8] = b"mantle-foreign-realization-receipt-v2\0";
const REALIZATION_INPUT_CLOSURE_DIGEST_DOMAIN: &[u8] = b"mantle-realization-input-closure-v1\0";
pub(crate) const ROOT_DISPOSITION_BUILT: &str = "built";
pub(crate) const ROOT_DISPOSITION_REUSED: &str = "reused";
pub(crate) const BUILDER_SIGNER_ROLE: &str = "builder";
pub(crate) const REUSED_ROOT_SOURCE_LOCAL: &str = "local";
pub(crate) const EXECUTION_CLASS_ALREADY_PRESENT: &str = "already-present";
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) cache_closure_policy_blake3: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(crate) cache_closure: Vec<ForeignCacheClosureFact>,
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) realization_route: Option<String>,
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

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ForeignRealizationReceiptV2 {
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) cache_closure_policy_blake3: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(crate) cache_closure: Vec<ForeignCacheClosureFact>,
    pub(crate) sources: Vec<ForeignSourceStoreFact>,
    pub(crate) units: Vec<ForeignRealizedUnit>,
    pub(crate) roots: Vec<ForeignRealizationRoot>,
    pub(crate) failure: Option<ForeignRealizationFailure>,
    pub(crate) non_claims: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ForeignRealizationRoot {
    pub(crate) node_id: String,
    pub(crate) target_derivation: String,
    pub(crate) output_name: String,
    pub(crate) path: String,
    pub(crate) nar_sha256: String,
    pub(crate) nar_size: u64,
    pub(crate) references: Vec<String>,
    pub(crate) disposition: String,
    pub(crate) signer: ForeignRealizationSigner,
    pub(crate) trust_policy_blake3: String,
    pub(crate) built: Option<ForeignRootBuilt>,
    pub(crate) reused: Option<ForeignRootReused>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ForeignRealizationSigner {
    pub(crate) key_name: String,
    pub(crate) public_key_blake3: String,
    pub(crate) role: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ForeignRootBuilt {
    pub(crate) mantle_version: String,
    pub(crate) execution_profile_digest_blake3: String,
    pub(crate) input_closure_blake3: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ForeignRootReused {
    pub(crate) original_receipt_blake3: String,
    pub(crate) source: String,
    pub(crate) builds: u64,
    pub(crate) downloads: u64,
}

pub(crate) struct ForeignReceiptV2Input<'a> {
    pub(crate) base: ForeignReceiptInput<'a>,
    pub(crate) roots: Vec<ForeignRealizationRoot>,
}

pub(crate) fn build_foreign_realization_receipt_v2(
    input: ForeignReceiptV2Input<'_>,
) -> Result<ForeignRealizationReceiptV2, RunError> {
    let base = build_foreign_realization_receipt(input.base)?;
    let mut receipt = ForeignRealizationReceiptV2 {
        schema: FOREIGN_REALIZATION_RECEIPT_V2_SCHEMA.to_string(),
        status: base.status,
        strongest_state: base.strongest_state,
        receipt_blake3: String::new(),
        plan_blake3: base.plan_blake3,
        import_receipt_blake3: base.import_receipt_blake3,
        source_bundle_manifest_blake3: base.source_bundle_manifest_blake3,
        build_report_blake3: base.build_report_blake3,
        selected_root_node_ids: base.selected_root_node_ids,
        selected_root_paths: base.selected_root_paths,
        execution_profiles: base.execution_profiles,
        cache_policy: base.cache_policy,
        cache_closure_policy_blake3: base.cache_closure_policy_blake3,
        cache_closure: base.cache_closure,
        sources: base.sources,
        units: base.units,
        roots: input.roots,
        failure: base.failure,
        non_claims: base.non_claims,
    };
    receipt.receipt_blake3 = foreign_realization_receipt_v2_digest(&receipt)?;
    Ok(receipt)
}

pub(crate) fn foreign_realization_receipt_v2_digest(receipt: &ForeignRealizationReceiptV2) -> Result<String, RunError> {
    let mut material = receipt.clone();
    material.receipt_blake3.clear();
    let canonical = serde_json::to_vec(&material)
        .map_err(|error| RunError::Internal(format!("serializing foreign realization receipt v2: {error}")))?;
    let mut hasher = blake3::Hasher::new();
    hasher.update(FOREIGN_REALIZATION_RECEIPT_V2_DIGEST_DOMAIN);
    hasher.update(&canonical);
    Ok(hasher.finalize().to_hex().to_string())
}

/// Parse one route-v2 receipt, rejecting unknown fields and inconsistent root dispositions.
pub(crate) fn parse_foreign_realization_receipt_v2(bytes: &[u8]) -> Result<ForeignRealizationReceiptV2, RunError> {
    let receipt: ForeignRealizationReceiptV2 = serde_json::from_slice(bytes)
        .map_err(|error| RunError::Internal(format!("parsing foreign realization receipt v2: {error}")))?;
    if receipt.schema != FOREIGN_REALIZATION_RECEIPT_V2_SCHEMA {
        return Err(RunError::Internal(format!("unsupported foreign realization receipt schema: {}", receipt.schema)));
    }
    validate_receipt_v2_root_dispositions(&receipt)?;
    Ok(receipt)
}

/// Enforce that every root entry records exactly one of `built`/`reused`, matching its disposition.
pub(crate) fn validate_receipt_v2_root_dispositions(receipt: &ForeignRealizationReceiptV2) -> Result<(), RunError> {
    for root in &receipt.roots {
        let consistent = match root.disposition.as_str() {
            ROOT_DISPOSITION_BUILT => root.built.is_some() && root.reused.is_none(),
            ROOT_DISPOSITION_REUSED => root.built.is_none() && root.reused.is_some(),
            _ => false,
        };
        if !consistent {
            return Err(RunError::Internal(format!(
                "foreign realization receipt v2 root {}:{} has an inconsistent disposition {}",
                root.target_derivation, root.output_name, root.disposition
            )));
        }
    }
    Ok(())
}

/// Serialize a receipt exactly as `--receipt-out` publishes it, so the built-root index holds the
/// same bytes.
pub(crate) fn foreign_realization_receipt_any_bytes(
    receipt: &ForeignRealizationReceiptAny,
) -> Result<Vec<u8>, RunError> {
    let mut bytes = serde_json::to_vec_pretty(receipt)
        .map_err(|error| RunError::Internal(format!("serializing foreign realization receipt: {error}")))?;
    bytes.push(b'\n');
    Ok(bytes)
}

/// Either receipt generation a realization can publish.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(untagged)]
pub(crate) enum ForeignRealizationReceiptAny {
    V1(ForeignRealizationReceipt),
    V2(ForeignRealizationReceiptV2),
}

impl ForeignRealizationReceiptAny {
    pub(crate) fn status(&self) -> &str {
        match self {
            Self::V1(receipt) => &receipt.status,
            Self::V2(receipt) => &receipt.status,
        }
    }

    pub(crate) fn failure(&self) -> Option<&ForeignRealizationFailure> {
        match self {
            Self::V1(receipt) => receipt.failure.as_ref(),
            Self::V2(receipt) => receipt.failure.as_ref(),
        }
    }

    pub(crate) fn receipt_blake3(&self) -> &str {
        match self {
            Self::V1(receipt) => &receipt.receipt_blake3,
            Self::V2(receipt) => &receipt.receipt_blake3,
        }
    }

    pub(crate) fn selected_root_node_ids(&self) -> &[String] {
        match self {
            Self::V1(receipt) => &receipt.selected_root_node_ids,
            Self::V2(receipt) => &receipt.selected_root_node_ids,
        }
    }

    pub(crate) fn units(&self) -> &[ForeignRealizedUnit] {
        match self {
            Self::V1(receipt) => &receipt.units,
            Self::V2(receipt) => &receipt.units,
        }
    }

    pub(crate) fn sources(&self) -> &[ForeignSourceStoreFact] {
        match self {
            Self::V1(receipt) => &receipt.sources,
            Self::V2(receipt) => &receipt.sources,
        }
    }
}

pub(crate) struct BuiltRootEntryInput<'a> {
    pub(crate) node_id: &'a str,
    pub(crate) target_derivation: &'a str,
    pub(crate) output_name: &'a str,
    pub(crate) path: &'a str,
    pub(crate) nar_sha256: &'a str,
    pub(crate) nar_size: u64,
    pub(crate) references: &'a [String],
    pub(crate) signer: ForeignRealizationSigner,
    pub(crate) trust_policy_blake3: &'a str,
    pub(crate) built: ForeignRootBuilt,
}

pub(crate) fn built_root_entry(input: BuiltRootEntryInput<'_>) -> ForeignRealizationRoot {
    assert!(!input.path.is_empty(), "built root path must not be empty");
    let mut references = input.references.to_vec();
    references.sort();
    references.dedup();
    ForeignRealizationRoot {
        node_id: input.node_id.to_string(),
        target_derivation: input.target_derivation.to_string(),
        output_name: input.output_name.to_string(),
        path: input.path.to_string(),
        nar_sha256: input.nar_sha256.to_string(),
        nar_size: input.nar_size,
        references,
        disposition: ROOT_DISPOSITION_BUILT.to_string(),
        signer: input.signer,
        trust_policy_blake3: input.trust_policy_blake3.to_string(),
        built: Some(input.built),
        reused: None,
    }
}

pub(crate) struct ReusedRootEntryInput<'a> {
    pub(crate) node_id: &'a str,
    pub(crate) target_derivation: &'a str,
    pub(crate) output_name: &'a str,
    pub(crate) path: &'a str,
    pub(crate) nar_sha256: &'a str,
    pub(crate) nar_size: u64,
    pub(crate) references: &'a [String],
    pub(crate) signer: ForeignRealizationSigner,
    pub(crate) trust_policy_blake3: &'a str,
    pub(crate) original_receipt_blake3: &'a str,
}

pub(crate) fn reused_root_entry(input: ReusedRootEntryInput<'_>) -> ForeignRealizationRoot {
    assert!(!input.original_receipt_blake3.is_empty(), "reused root link must not be empty");
    let mut references = input.references.to_vec();
    references.sort();
    references.dedup();
    ForeignRealizationRoot {
        node_id: input.node_id.to_string(),
        target_derivation: input.target_derivation.to_string(),
        output_name: input.output_name.to_string(),
        path: input.path.to_string(),
        nar_sha256: input.nar_sha256.to_string(),
        nar_size: input.nar_size,
        references,
        disposition: ROOT_DISPOSITION_REUSED.to_string(),
        signer: input.signer,
        trust_policy_blake3: input.trust_policy_blake3.to_string(),
        built: None,
        reused: Some(ForeignRootReused {
            original_receipt_blake3: input.original_receipt_blake3.to_string(),
            source: REUSED_ROOT_SOURCE_LOCAL.to_string(),
            builds: 0,
            downloads: 0,
        }),
    }
}

/// A reused root's unit is observed as already present, never re-executed.
pub(crate) fn mark_reused_units_already_present(units: &mut [ForeignRealizedUnit], reused_node_ids: &BTreeSet<String>) {
    for unit in units.iter_mut() {
        if reused_node_ids.contains(unit.node_id.as_str()) {
            unit.execution_class = EXECUTION_CLASS_ALREADY_PRESENT.to_string();
        }
    }
}

/// Digest of the admitted input closure a built root was realized from.
pub(crate) fn realization_input_closure_blake3(facts: &[ForeignCacheClosureFact]) -> Result<String, RunError> {
    #[derive(Serialize)]
    struct InputMaterial<'a> {
        path: &'a str,
        nar_sha256: &'a str,
        nar_size: u64,
    }
    let mut inputs = facts
        .iter()
        .map(|fact| InputMaterial {
            path: &fact.logical_path,
            nar_sha256: &fact.nar_sha256,
            nar_size: fact.nar_size,
        })
        .collect::<Vec<_>>();
    inputs.sort_by(|left, right| left.path.cmp(right.path));
    let canonical = serde_json::to_vec(&inputs)
        .map_err(|error| RunError::Internal(format!("serializing realization input closure: {error}")))?;
    let mut hasher = blake3::Hasher::new();
    hasher.update(REALIZATION_INPUT_CLOSURE_DIGEST_DOMAIN);
    hasher.update(&canonical);
    Ok(hasher.finalize().to_hex().to_string())
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
    pub(crate) cache_closure: Option<&'a CacheClosureHydration>,
    /// A reuse route may leave present input units out of the scheduler entirely.
    pub(crate) allow_pruned_units: bool,
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
    let failed_by_origin = failures_by_origin(&input.build_result.failed_roots);
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
    let cache_fact_by_path = input
        .cache_closure
        .into_iter()
        .flat_map(|hydration| hydration.facts.iter())
        .map(|fact| (fact.logical_path.as_str(), fact))
        .collect::<BTreeMap<_, _>>();
    let mut profiles = BTreeSet::new();
    let mut units = Vec::with_capacity(input.plan.native_units.len());
    for unit in &input.plan.native_units {
        profiles.insert((unit.execution_profile_id.clone(), unit.execution_profile_digest_blake3.clone()));
        let outcome = outcome_by_drv.get(&unit.target_derivation).copied();
        let failed_root = failed_by_drv.get(unit.target_derivation.as_str()).copied();
        let failed_origin = failed_by_origin.get(unit.target_derivation.as_str()).copied();
        let failure = failed_origin
            .map(|failed| failed.origin_error.as_str())
            .or_else(|| failed_root.map(|failed| failed.error.as_str()));
        let build_log = outcome
            .and_then(|value| value.log.as_deref())
            .or_else(|| failed_log_by_origin.get(unit.target_derivation.as_str()).copied());
        let fetch_attempts = parse_fetch_attempts(build_log)?;
        let mut outputs = Vec::with_capacity(unit.outputs.len());
        for (output_name, target_path) in &unit.outputs {
            let realized = output_by_unit_and_name.get(&(unit.node_id.as_str(), output_name.as_str())).copied();
            let cache_fact = cache_fact_by_path.get(target_path.as_str()).copied();
            if outcome.is_some() && realized.is_none() && cache_fact.is_none() {
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
            } else if let Some(cache_fact) = cache_fact {
                let is_remote = cache_fact.disposition == "remote-substituted";
                outputs.push(ForeignRealizedOutput {
                    output_name: output_name.clone(),
                    target_path: target_path.clone(),
                    nar_sha256: cache_fact.nar_sha256.clone(),
                    nar_size: cache_fact.nar_size,
                    substitution_mode: Some("full".to_string()),
                    substitution_transferred_bytes: Some(if is_remote { cache_fact.nar_size } else { 0 }),
                    substitution_reused_bytes: Some(if is_remote { 0 } else { cache_fact.nar_size }),
                });
            }
        }
        let effective_failure = failure.map(str::to_string).or_else(|| {
            if input.cache_closure.is_none()
                && outcome.is_none()
                && outputs.is_empty()
                && !input.build_result.failed_roots.is_empty()
            {
                Some("unit has no store output after a selected root failure".to_string())
            } else {
                None
            }
        });
        let cache_only_not_required =
            input.cache_closure.is_some() && outcome.is_none() && effective_failure.is_none() && outputs.is_empty();
        if outcome.is_none()
            && effective_failure.is_none()
            && outputs.is_empty()
            && !cache_only_not_required
            && !input.allow_pruned_units
        {
            return Err(RunError::Internal(format!(
                "foreign realization has no scheduler or store disposition for {}",
                unit.node_id
            )));
        }
        let execution_class = if effective_failure.is_some() {
            "failed-or-blocked"
        } else if cache_only_not_required {
            "not-required-cache-only"
        } else if outcome.is_none() && outputs.iter().any(|output| output.substitution_transferred_bytes != Some(0)) {
            "substituted"
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
                .map(|failed| {
                    format!(
                        "root {}: {}; origin {}: {}",
                        failed.drv_key, failed.error, failed.origin_drv_key, failed.origin_error
                    )
                })
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
            realization_route: input.plan.realization_route.clone(),
        },
        cache_closure_policy_blake3: input.cache_closure.map(|hydration| hydration.policy_blake3.clone()),
        cache_closure: input.cache_closure.map(|hydration| hydration.facts.clone()).unwrap_or_default(),
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
        origin_error: &'a str,
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
            origin_error: &failed.origin_error,
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

fn failures_by_origin(failures: &[crunch_build::FailedGoal]) -> BTreeMap<&str, &crunch_build::FailedGoal> {
    let mut by_origin = BTreeMap::new();
    for failure in failures {
        by_origin.entry(failure.origin_drv_key.as_str()).or_insert(failure);
    }
    debug_assert!(by_origin.len() <= failures.len());
    by_origin
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

#[cfg(test)]
mod tests {
    use super::*;

    const ROOT_DRV: &str = "/mantle/store/root.drv";
    const ORIGIN_DRV: &str = "/mantle/store/origin.drv";
    const FAILURE_MESSAGE: &str = "Not all outputs produced";

    fn fixture_failure() -> crunch_build::FailedGoal {
        crunch_build::FailedGoal {
            drv_key: ROOT_DRV.to_string(),
            origin_drv_key: ORIGIN_DRV.to_string(),
            error: "dependency failed".to_string(),
            origin_error: FAILURE_MESSAGE.to_string(),
            build_log: None,
        }
    }

    #[test]
    fn failure_origin_index_exposes_the_causal_derivation() {
        let failures = vec![fixture_failure()];

        let by_origin = failures_by_origin(&failures);
        let failure = by_origin.get(ORIGIN_DRV).copied().unwrap();

        assert_eq!(failure.drv_key, ROOT_DRV);
        assert_eq!(failure.origin_drv_key, ORIGIN_DRV);
        assert_eq!(failure.error, "dependency failed");
        assert_eq!(failure.origin_error, FAILURE_MESSAGE);
    }

    #[test]
    fn failure_origin_index_does_not_mark_an_unrelated_derivation() {
        let failures = vec![fixture_failure()];

        let by_origin = failures_by_origin(&failures);

        assert!(!by_origin.contains_key(ROOT_DRV));
        assert_eq!(by_origin.len(), 1);
    }

    fn fixture_receipt_v2_json() -> serde_json::Value {
        serde_json::json!({
            "schema": super::FOREIGN_REALIZATION_RECEIPT_V2_SCHEMA,
            "status": "complete",
            "strongest_state": "realized",
            "receipt_blake3": "",
            "plan_blake3": "0".repeat(64),
            "import_receipt_blake3": "0".repeat(64),
            "source_bundle_manifest_blake3": "0".repeat(64),
            "build_report_blake3": "0".repeat(64),
            "selected_root_node_ids": ["nix:parent"],
            "selected_root_paths": ["/nix/store/bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb-parent"],
            "execution_profiles": [],
            "cache_policy": {
                "substitution_enabled": true,
                "offline": false,
                "ordered_cache_urls": [],
                "realization_route": "cache-inputs-build-roots-v2"
            },
            "cache_closure": [],
            "sources": [],
            "units": [],
            "roots": [{
                "node_id": "nix:parent",
                "target_derivation": "/nix/store/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa-parent.drv",
                "output_name": "out",
                "path": "/nix/store/bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb-parent",
                "nar_sha256": "0".repeat(64),
                "nar_size": 3,
                "references": [],
                "disposition": "built",
                "signer": {
                    "key_name": "crunch-host-1",
                    "public_key_blake3": "0".repeat(64),
                    "role": "builder"
                },
                "trust_policy_blake3": "0".repeat(64),
                "built": {
                    "mantle_version": "0.0.0",
                    "execution_profile_digest_blake3": "0".repeat(64),
                    "input_closure_blake3": "0".repeat(64)
                },
                "reused": null
            }],
            "failure": null,
            "non_claims": []
        })
    }

    #[test]
    fn receipt_v2_digest_ignores_its_own_field_and_parse_is_closed_world() {
        let value = fixture_receipt_v2_json();
        let mut receipt: super::ForeignRealizationReceiptV2 = serde_json::from_value(value.clone()).unwrap();
        let digest = super::foreign_realization_receipt_v2_digest(&receipt).unwrap();
        assert_eq!(digest.len(), 64);
        assert!(digest.chars().all(|character| character.is_ascii_hexdigit() && !character.is_ascii_uppercase()));

        receipt.receipt_blake3 = digest.clone();
        assert_eq!(super::foreign_realization_receipt_v2_digest(&receipt).unwrap(), digest);
        receipt.receipt_blake3 = "f".repeat(64);
        assert_eq!(super::foreign_realization_receipt_v2_digest(&receipt).unwrap(), digest);

        let mut unknown = value.clone();
        unknown["unexpected"] = serde_json::json!(true);
        assert!(serde_json::from_value::<super::ForeignRealizationReceiptV2>(unknown).is_err());

        let mut missing_roots = value.clone();
        missing_roots.as_object_mut().unwrap().remove("roots");
        assert!(serde_json::from_value::<super::ForeignRealizationReceiptV2>(missing_roots).is_err());

        let mut unknown_root = fixture_receipt_v2_json();
        unknown_root["roots"][0]["unexpected"] = serde_json::json!(true);
        assert!(serde_json::from_value::<super::ForeignRealizationReceiptV2>(unknown_root).is_err());

        // The route-v2 parser is closed-world and rejects inconsistent dispositions.
        let bytes = serde_json::to_vec(&value).unwrap();
        assert!(super::parse_foreign_realization_receipt_v2(&bytes).is_ok());

        let mut both = fixture_receipt_v2_json();
        both["roots"][0]["reused"] = serde_json::json!({
            "original_receipt_blake3": "0".repeat(64),
            "source": "local",
            "builds": 0,
            "downloads": 0
        });
        assert!(super::parse_foreign_realization_receipt_v2(&serde_json::to_vec(&both).unwrap()).is_err());

        let mut neither = fixture_receipt_v2_json();
        neither["roots"][0]["built"] = serde_json::Value::Null;
        assert!(super::parse_foreign_realization_receipt_v2(&serde_json::to_vec(&neither).unwrap()).is_err());

        let mut reused_disposition = fixture_receipt_v2_json();
        reused_disposition["roots"][0]["disposition"] = serde_json::json!("reused");
        assert!(
            super::parse_foreign_realization_receipt_v2(&serde_json::to_vec(&reused_disposition).unwrap()).is_err()
        );

        let mut wrong_schema = fixture_receipt_v2_json();
        wrong_schema["schema"] = serde_json::json!("mantle-foreign-realization-receipt-v1");
        assert!(super::parse_foreign_realization_receipt_v2(&serde_json::to_vec(&wrong_schema).unwrap()).is_err());
    }
}
