//! Compose validated StageX, native, Rust-provider, and Rust-unit action evidence.
//!
//! This shell does not invent actions after execution. It validates each
//! adapter's pre-execution plan and reconciliation, then writes the bounded
//! root summary consumed by `bootstrap trust-report` and the v2 receipt.

use std::fs;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Component;
use std::path::Path;
use std::path::PathBuf;

use serde::Serialize;

use crate::errors::RunError;
use crate::source_built_fixed_point::SourceBuiltFixedPointPlan;
use crate::source_built_fixed_point_shell::ConstructedProviders;

const ROOT_PLAN_DIGEST_CONTEXT: &[u8] = b"mantle-root-action-trust-plan-v1\0";
const ROOT_RECONCILIATION_DIGEST_CONTEXT: &[u8] = b"mantle-root-action-reconciliation-v1\0";
const BLAKE3_HEX_LENGTH: usize = 64;
const ADAPTER_COUNT: u32 = 5;
const TEXT_BYTES_MAX: usize = 4_096;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RootActionTrustPublication {
    pub(crate) plan_path: PathBuf,
    pub(crate) plan_file_digest_blake3: String,
    pub(crate) reconciliation_path: PathBuf,
    pub(crate) reconciliation_file_digest_blake3: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct RootActionAdapter {
    adapter_id: String,
    plan_path: String,
    plan_file_digest_blake3: String,
    plan_semantic_digest_blake3: String,
    reconciliation_path: String,
    reconciliation_file_digest_blake3: String,
    reconciliation_semantic_digest_blake3: String,
    planned_action_count: u32,
    matched_action_count: u32,
    observed_event_count: u32,
    matched_event_count: u32,
    local_only: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct RootActionTrustPlan {
    schema: String,
    proof_plan_digest_blake3: String,
    adapter_count: u32,
    action_count: u32,
    adapters: Vec<RootActionAdapter>,
    local_only: bool,
    cache_only_completion_allowed: bool,
    blockers: Vec<String>,
    plan_digest_blake3: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct RootActionReconciliation {
    schema: String,
    action_plan_digest_blake3: String,
    planned_action_count: u32,
    matched_action_count: u32,
    observed_event_count: u32,
    matched_event_count: u32,
    unknown_event_count: u32,
    missing_action_count: u32,
    authority_violation_count: u32,
    fallback_event_count: u32,
    remote_event_count: u32,
    cache_only_completion_count: u32,
    local_only: bool,
    blockers: Vec<String>,
    reconciliation_digest_blake3: String,
}

pub(crate) fn write_root_action_trust(
    proof_root: &Path,
    fixed_point_dir: &Path,
    proof_plan: &SourceBuiltFixedPointPlan,
    providers: &ConstructedProviders,
) -> Result<RootActionTrustPublication, RunError> {
    validate_digest("proof plan", &proof_plan.plan_digest_blake3)?;
    let adapters = vec![
        stagex_adapter(proof_root, providers)?,
        native_adapter(proof_root, providers)?,
        rust_provider_adapter(proof_root, providers)?,
        rust_unit_adapter(proof_root, fixed_point_dir, "stage1")?,
        rust_unit_adapter(proof_root, fixed_point_dir, "stage2")?,
    ];
    let plan = build_root_plan(&proof_plan.plan_digest_blake3, adapters)?;
    let plan_path = proof_root.join(crate::source_built_trust_report::ROOT_ACTION_TRUST_PLAN_FILE);
    write_json_create_new(&plan_path, &plan)?;
    let plan_file_digest_blake3 = file_digest(&plan_path)?;
    let reconciliation = build_root_reconciliation(&plan_file_digest_blake3, &plan)?;
    let reconciliation_path = proof_root.join(crate::source_built_trust_report::ROOT_ACTION_RECONCILIATION_FILE);
    write_json_create_new(&reconciliation_path, &reconciliation)?;
    let reconciliation_file_digest_blake3 = file_digest(&reconciliation_path)?;
    assert_eq!(reconciliation.planned_action_count, reconciliation.matched_action_count);
    assert_eq!(reconciliation.observed_event_count, reconciliation.matched_event_count);
    Ok(RootActionTrustPublication {
        plan_path,
        plan_file_digest_blake3,
        reconciliation_path,
        reconciliation_file_digest_blake3,
    })
}

fn build_root_plan(
    proof_plan_digest_blake3: &str,
    mut adapters: Vec<RootActionAdapter>,
) -> Result<RootActionTrustPlan, RunError> {
    validate_digest("proof plan", proof_plan_digest_blake3)?;
    adapters.sort_by(|left, right| left.adapter_id.cmp(&right.adapter_id));
    if adapters.len() != usize::try_from(ADAPTER_COUNT).unwrap_or(usize::MAX) {
        return Err(root_error(format!("root action adapter count {} does not match {ADAPTER_COUNT}", adapters.len())));
    }
    if adapters.iter().any(|adapter| {
        adapter.planned_action_count == 0
            || adapter.planned_action_count != adapter.matched_action_count
            || adapter.observed_event_count == 0
            || adapter.observed_event_count != adapter.matched_event_count
            || !adapter.local_only
    }) {
        return Err(root_error("root action adapters contain incomplete action or event coverage".to_string()));
    }
    let action_count = sum_adapter_count(&adapters, |adapter| adapter.planned_action_count, "planned actions")?;
    let mut plan = RootActionTrustPlan {
        schema: crate::source_built_trust_report::ROOT_ACTION_TRUST_PLAN_SCHEMA.to_string(),
        proof_plan_digest_blake3: proof_plan_digest_blake3.to_string(),
        adapter_count: ADAPTER_COUNT,
        action_count,
        adapters,
        local_only: true,
        cache_only_completion_allowed: false,
        blockers: Vec::new(),
        plan_digest_blake3: String::new(),
    };
    plan.plan_digest_blake3 = digest_serialized(ROOT_PLAN_DIGEST_CONTEXT, &plan)?;
    Ok(plan)
}

fn build_root_reconciliation(
    plan_file_digest_blake3: &str,
    plan: &RootActionTrustPlan,
) -> Result<RootActionReconciliation, RunError> {
    validate_digest("root plan file", plan_file_digest_blake3)?;
    let matched_action_count =
        sum_adapter_count(&plan.adapters, |adapter| adapter.matched_action_count, "matched actions")?;
    let observed_event_count =
        sum_adapter_count(&plan.adapters, |adapter| adapter.observed_event_count, "observed events")?;
    let matched_event_count =
        sum_adapter_count(&plan.adapters, |adapter| adapter.matched_event_count, "matched events")?;
    let mut reconciliation = RootActionReconciliation {
        schema: crate::source_built_trust_report::ROOT_ACTION_RECONCILIATION_SCHEMA.to_string(),
        action_plan_digest_blake3: plan_file_digest_blake3.to_string(),
        planned_action_count: plan.action_count,
        matched_action_count,
        observed_event_count,
        matched_event_count,
        unknown_event_count: 0,
        missing_action_count: 0,
        authority_violation_count: 0,
        fallback_event_count: 0,
        remote_event_count: 0,
        cache_only_completion_count: 0,
        local_only: true,
        blockers: Vec::new(),
        reconciliation_digest_blake3: String::new(),
    };
    reconciliation.reconciliation_digest_blake3 =
        digest_serialized(ROOT_RECONCILIATION_DIGEST_CONTEXT, &reconciliation)?;
    if reconciliation.planned_action_count != reconciliation.matched_action_count
        || reconciliation.observed_event_count != reconciliation.matched_event_count
    {
        return Err(root_error("root action reconciliation counts differ".to_string()));
    }
    Ok(reconciliation)
}

fn stagex_adapter(proof_root: &Path, providers: &ConstructedProviders) -> Result<RootActionAdapter, RunError> {
    let plan_path = providers.stagex_transition_execution_dir.join(crate::stagex_transition::PLAN_FILE_NAME);
    let audit_path = providers.stagex_transition_execution_dir.join(crate::stagex_transition::AUDIT_FILE_NAME);
    let report_path = providers.stagex_transition_execution_dir.join(crate::stagex_transition::REPORT_FILE_NAME);
    let plan: crunch_bootstrap_core::StagexMaterializationPlan = read_json(&plan_path, "StageX plan")?;
    let validation = crunch_bootstrap_core::validate_stagex_plan(&plan);
    if !validation.is_valid() {
        return Err(root_error("StageX action plan is invalid".to_string()));
    }
    let report: serde_json::Value = read_json(&report_path, "StageX report")?;
    let semantic_plan_digest = crunch_bootstrap_core::stagex_plan_digest_blake3(&plan)
        .map_err(|error| root_error(format!("digesting StageX action plan: {error}")))?;
    if report.pointer("/status").and_then(serde_json::Value::as_str) != Some("complete")
        || report.pointer("/plan_digest_blake3").and_then(serde_json::Value::as_str)
            != Some(semantic_plan_digest.as_str())
    {
        return Err(root_error("StageX report does not bind its complete action plan".to_string()));
    }
    let events: Vec<crate::protected_exec::ProtectedSeccompAuditEvent> = read_json(&audit_path, "StageX audit")?;
    if events.is_empty() || events.iter().any(|event| event.policy_decision != "allowed") {
        return Err(root_error("StageX audit contains missing or denied execution events".to_string()));
    }
    let action_count =
        u32::try_from(plan.stages.len()).map_err(|_| root_error("StageX action count exceeds u32".to_string()))?;
    let event_count =
        u32::try_from(events.len()).map_err(|_| root_error("StageX event count exceeds u32".to_string()))?;
    adapter(
        proof_root,
        "stagex",
        &plan_path,
        &semantic_plan_digest,
        &audit_path,
        &file_digest(&audit_path)?,
        action_count,
        action_count,
        event_count,
        event_count,
    )
}

fn native_adapter(proof_root: &Path, providers: &ConstructedProviders) -> Result<RootActionAdapter, RunError> {
    let evidence = providers
        .native_action_trust
        .as_ref()
        .ok_or_else(|| root_error("native provider action trust is absent".to_string()))?;
    crate::source_built_derivation_action_plan::validate_eager_derivation_action_plan(&evidence.plan)
        .map_err(|error| root_error(format!("invalid native action plan: {error}")))?;
    crate::source_built_derivation_action_plan::require_complete_eager_derivation_reconciliation(
        &evidence.reconciliation,
    )
    .map_err(|error| root_error(format!("invalid native action reconciliation: {error}")))?;
    adapter(
        proof_root,
        "native-provider",
        &evidence.plan_path,
        &evidence.plan.plan_digest_blake3,
        &evidence.reconciliation_path,
        &evidence.reconciliation.reconciliation_digest_blake3,
        evidence.plan.action_count,
        evidence.reconciliation.matched_action_count,
        evidence.reconciliation.observed_event_count,
        evidence.reconciliation.matched_event_count,
    )
}

fn rust_provider_adapter(proof_root: &Path, providers: &ConstructedProviders) -> Result<RootActionAdapter, RunError> {
    let evidence = providers
        .rust_provider
        .action_trust
        .as_ref()
        .ok_or_else(|| root_error("Rust provider action trust is absent".to_string()))?;
    let evidence_dir = evidence
        .plan_path
        .parent()
        .ok_or_else(|| root_error("Rust provider action plan has no parent".to_string()))?;
    let validated = crate::source_built_rust_provider_action::validate_rust_provider_action_evidence(evidence_dir)
        .map_err(|error| root_error(format!("invalid Rust provider action trust: {error}")))?;
    validate_digest("Rust provider audit", &validated.audit_digest_blake3)?;
    adapter(
        proof_root,
        "rust-provider",
        &validated.plan_path,
        &validated.plan_digest_blake3,
        &validated.reconciliation_path,
        &validated.reconciliation_digest_blake3,
        validated.planned_action_count,
        validated.matched_action_count,
        validated.observed_event_count,
        validated.matched_event_count,
    )
}

fn rust_unit_adapter(proof_root: &Path, fixed_point_dir: &Path, stage: &str) -> Result<RootActionAdapter, RunError> {
    let meta: serde_json::Value = read_json(&fixed_point_dir.join("meta.json"), "fixed-point summary")?;
    let plan_path = fixed_point_artifact_path(fixed_point_dir, &meta, stage, "rust_child_action_plan")?;
    let audit_path = fixed_point_artifact_path(fixed_point_dir, &meta, stage, "rust_child_action_audit")?;
    let reconciliation_path =
        fixed_point_artifact_path(fixed_point_dir, &meta, stage, "rust_child_action_reconciliation")?;
    let plan: crate::source_built_rust_action_plan::RustChildActionPlan = read_json(&plan_path, "Rust unit plan")?;
    let audit: crate::source_built_rust_action_plan::RustChildActionAudit = read_json(&audit_path, "Rust unit audit")?;
    let reconciliation: crate::source_built_rust_action_plan::RustChildActionReconciliation =
        read_json(&reconciliation_path, "Rust unit reconciliation")?;
    crate::source_built_rust_action_plan::validate_rust_child_action_plan(&plan)
        .map_err(|error| root_error(format!("invalid {stage} Rust unit plan: {error}")))?;
    crate::source_built_rust_action_plan::validate_rust_child_action_audit(&plan, &audit)
        .map_err(|error| root_error(format!("invalid {stage} Rust unit audit: {error}")))?;
    crate::source_built_rust_action_plan::validate_rust_child_action_reconciliation(&plan, &reconciliation)
        .map_err(|error| root_error(format!("invalid {stage} Rust unit reconciliation: {error}")))?;
    if !reconciliation.is_complete() {
        return Err(root_error(format!("{stage} Rust unit reconciliation is incomplete")));
    }
    adapter(
        proof_root,
        &format!("rust-units-{stage}"),
        &plan_path,
        &plan.plan_digest_blake3,
        &reconciliation_path,
        &reconciliation.reconciliation_digest_blake3,
        plan.action_count,
        reconciliation.matched_action_count,
        reconciliation.observed_event_count,
        reconciliation.matched_event_count,
    )
}

#[allow(clippy::too_many_arguments)]
fn adapter(
    proof_root: &Path,
    adapter_id: &str,
    plan_path: &Path,
    plan_semantic_digest_blake3: &str,
    reconciliation_path: &Path,
    reconciliation_semantic_digest_blake3: &str,
    planned_action_count: u32,
    matched_action_count: u32,
    observed_event_count: u32,
    matched_event_count: u32,
) -> Result<RootActionAdapter, RunError> {
    validate_text("adapter id", adapter_id)?;
    validate_digest("adapter plan", plan_semantic_digest_blake3)?;
    validate_digest("adapter reconciliation", reconciliation_semantic_digest_blake3)?;
    Ok(RootActionAdapter {
        adapter_id: adapter_id.to_string(),
        plan_path: proof_relative_path(proof_root, plan_path)?,
        plan_file_digest_blake3: file_digest(plan_path)?,
        plan_semantic_digest_blake3: plan_semantic_digest_blake3.to_string(),
        reconciliation_path: proof_relative_path(proof_root, reconciliation_path)?,
        reconciliation_file_digest_blake3: file_digest(reconciliation_path)?,
        reconciliation_semantic_digest_blake3: reconciliation_semantic_digest_blake3.to_string(),
        planned_action_count,
        matched_action_count,
        observed_event_count,
        matched_event_count,
        local_only: true,
    })
}

fn fixed_point_artifact_path(
    fixed_point_dir: &Path,
    meta: &serde_json::Value,
    stage: &str,
    field: &str,
) -> Result<PathBuf, RunError> {
    let pointer = format!("/{stage}/{field}");
    let relative = meta
        .pointer(&pointer)
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| root_error(format!("fixed-point summary is missing {pointer}")))?;
    let relative = Path::new(relative);
    if relative.is_absolute() || relative.components().any(|component| !matches!(component, Component::Normal(_))) {
        return Err(root_error(format!("fixed-point action path is unsafe: {relative:?}")));
    }
    let path = fixed_point_dir.join(relative);
    if !path.is_file() {
        return Err(root_error(format!("fixed-point action artifact is missing: {}", path.display())));
    }
    Ok(path)
}

fn proof_relative_path(proof_root: &Path, path: &Path) -> Result<String, RunError> {
    let relative = path
        .strip_prefix(proof_root)
        .map_err(|_| root_error(format!("action evidence is outside proof root: {}", path.display())))?;
    if relative.as_os_str().is_empty()
        || relative.components().any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(root_error(format!("action evidence path is unsafe: {}", path.display())));
    }
    relative
        .to_str()
        .map(str::to_string)
        .ok_or_else(|| root_error(format!("action evidence path is not UTF-8: {}", path.display())))
}

fn sum_adapter_count(
    adapters: &[RootActionAdapter],
    select: impl Fn(&RootActionAdapter) -> u32,
    label: &str,
) -> Result<u32, RunError> {
    adapters.iter().try_fold(0_u32, |count, adapter| {
        count
            .checked_add(select(adapter))
            .ok_or_else(|| root_error(format!("root action {label} overflow")))
    })
}

fn read_json<T: serde::de::DeserializeOwned>(path: &Path, label: &str) -> Result<T, RunError> {
    let bytes = fs::read(path).map_err(|error| root_error(format!("reading {label} {}: {error}", path.display())))?;
    serde_json::from_slice(&bytes).map_err(|error| root_error(format!("parsing {label} {}: {error}", path.display())))
}

fn file_digest(path: &Path) -> Result<String, RunError> {
    crate::protected_exec::blake3_file_hex(path)
        .map_err(|error| root_error(format!("hashing root action evidence {}: {error}", path.display())))
}

fn write_json_create_new<T: Serialize>(path: &Path, value: &T) -> Result<(), RunError> {
    let bytes = serde_json::to_vec_pretty(value)
        .map_err(|error| root_error(format!("serializing root action evidence: {error}")))?;
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|error| root_error(format!("creating root action evidence {}: {error}", path.display())))?;
    file.write_all(&bytes)
        .map_err(|error| root_error(format!("writing root action evidence {}: {error}", path.display())))?;
    file.sync_all()
        .map_err(|error| root_error(format!("syncing root action evidence {}: {error}", path.display())))?;
    assert!(!bytes.is_empty());
    assert!(path.is_file());
    Ok(())
}

fn digest_serialized<T: Serialize>(context: &[u8], value: &T) -> Result<String, RunError> {
    let bytes = serde_json::to_vec(value)
        .map_err(|error| root_error(format!("serializing root action digest material: {error}")))?;
    let mut hasher = blake3::Hasher::new();
    hasher.update(context);
    hasher.update(&bytes);
    Ok(hasher.finalize().to_hex().to_string())
}

fn validate_digest(label: &str, digest: &str) -> Result<(), RunError> {
    if digest.len() != BLAKE3_HEX_LENGTH
        || !digest.bytes().all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
    {
        return Err(root_error(format!("invalid {label} BLAKE3")));
    }
    Ok(())
}

fn validate_text(label: &str, value: &str) -> Result<(), RunError> {
    if value.trim().is_empty() || value.len() > TEXT_BYTES_MAX || value.contains('\0') {
        return Err(root_error(format!("invalid root action {label}")));
    }
    Ok(())
}

fn root_error(message: String) -> RunError {
    RunError::Build(format!("root action trust blocked: {message}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    const DIGEST: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    const TEST_ACTIONS_PER_ADAPTER: u32 = 2;
    const TEST_EVENTS_PER_ADAPTER: u32 = 3;
    const TEST_PARTIAL_EVENT_COUNT: u32 = 2;

    #[test]
    fn root_documents_sum_complete_adapters_and_reject_partial_coverage() {
        let adapters = (0..ADAPTER_COUNT)
            .map(|index| adapter_fixture(index, TEST_ACTIONS_PER_ADAPTER, TEST_EVENTS_PER_ADAPTER))
            .collect::<Vec<_>>();
        let plan = build_root_plan(DIGEST, adapters.clone()).unwrap();
        let reconciliation = build_root_reconciliation(DIGEST, &plan).unwrap();
        let mut partial = adapters;
        partial[0].matched_event_count = TEST_PARTIAL_EVENT_COUNT;
        let error = build_root_plan(DIGEST, partial).unwrap_err();

        assert_eq!(plan.adapter_count, ADAPTER_COUNT);
        assert_eq!(plan.action_count, ADAPTER_COUNT * TEST_ACTIONS_PER_ADAPTER);
        assert_eq!(reconciliation.planned_action_count, reconciliation.matched_action_count);
        assert_eq!(reconciliation.observed_event_count, ADAPTER_COUNT * TEST_EVENTS_PER_ADAPTER);
        assert!(error.to_string().contains("incomplete action or event coverage"));
    }

    #[test]
    fn fixed_point_action_path_rejects_traversal() {
        let dir = tempfile::tempdir().unwrap();
        let meta = serde_json::json!({
            "stage1": {"rust_child_action_plan": "../escape.json"}
        });

        let error = fixed_point_artifact_path(dir.path(), &meta, "stage1", "rust_child_action_plan").unwrap_err();

        assert!(error.to_string().contains("unsafe"));
        assert!(!dir.path().join("escape.json").exists());
    }

    fn adapter_fixture(index: u32, actions: u32, events: u32) -> RootActionAdapter {
        RootActionAdapter {
            adapter_id: format!("adapter-{index}"),
            plan_path: format!("adapter-{index}/plan.json"),
            plan_file_digest_blake3: DIGEST.to_string(),
            plan_semantic_digest_blake3: DIGEST.to_string(),
            reconciliation_path: format!("adapter-{index}/reconciliation.json"),
            reconciliation_file_digest_blake3: DIGEST.to_string(),
            reconciliation_semantic_digest_blake3: DIGEST.to_string(),
            planned_action_count: actions,
            matched_action_count: actions,
            observed_event_count: events,
            matched_event_count: events,
            local_only: true,
        }
    }
}
