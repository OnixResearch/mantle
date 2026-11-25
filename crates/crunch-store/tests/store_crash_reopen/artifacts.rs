use std::path::PathBuf;

use serde::Deserialize;
use serde::Serialize;

use super::language::FaultInjectionMode;
use super::language::HistoryProgramV1;
use super::language::StoreOp;
use super::language::StoreProfile;
use super::model::ORACLE_SCHEMA_VERSION;
use super::model::StepClassification;
use super::model::ViolationKind;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RequestedVsAppliedFault {
    pub step_index: usize,
    pub requested: FaultInjectionMode,
    pub applied: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutedScheduleStep {
    pub step_index: usize,
    pub op: StoreOp,
    pub classification: StepClassification,
    pub observed_identities: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CounterexampleArtifact {
    pub schema: String,
    pub crate_version: String,
    pub profile: StoreProfile,
    pub policy_blake3: String,
    pub fault_implementation: String,
    pub oracle_schema: String,
    pub seed: u64,
    pub original_program: HistoryProgramV1,
    pub minimized_program: HistoryProgramV1,
    pub requested_vs_applied_faults: Vec<RequestedVsAppliedFault>,
    pub executed_schedule: Vec<ExecutedScheduleStep>,
    pub violation_kind: ViolationKind,
    pub failure_reason: String,
}

pub fn policy_blake3_for_profile(profile: StoreProfile) -> String {
    let tag = format!("mantle-store-crash-reopen-policy:{}", profile.as_str());
    format!("b3:{}", blake3::hash(tag.as_bytes()).to_hex())
}

pub fn shrink_history_preserving_failure<F>(
    program: &HistoryProgramV1,
    target_kind: ViolationKind,
    mut run_fn: F,
) -> HistoryProgramV1
where
    F: FnMut(&HistoryProgramV1) -> Option<ViolationKind>,
{
    let mut current = program.clone();
    let max_rounds = current.steps.len().saturating_add(1);
    for _round in 0..max_rounds {
        let mut did_shrink = false;
        let len = current.steps.len();
        if len <= 1 {
            break;
        }
        for idx in 0..len {
            let mut candidate = current.clone();
            candidate.steps.remove(idx);
            if run_fn(&candidate) == Some(target_kind) {
                current = candidate;
                did_shrink = true;
                break;
            }
        }
        if !did_shrink {
            break;
        }
    }
    current
}

pub fn persist_counterexample_artifact(artifact: &CounterexampleArtifact) -> Result<PathBuf, String> {
    let encoded = serde_json::to_string_pretty(artifact).map_err(|err| format!("serializing artifact: {err}"))?;
    let base_dir = match std::env::var_os("MANTLE_STORE_HISTORY_ARTIFACT_DIR") {
        Some(dir) if !dir.is_empty() => PathBuf::from(dir),
        _ => std::env::temp_dir().join("mantle-store-history-artifacts"),
    };
    std::fs::create_dir_all(&base_dir).map_err(|err| format!("creating artifact dir: {err}"))?;
    let file_name = format!("counterexample-{}-{}.json", artifact.profile.as_str(), artifact.seed);
    let path = base_dir.join(file_name);
    std::fs::write(&path, encoded).map_err(|err| format!("writing artifact {}: {err}", path.display()))?;
    Ok(path)
}

pub fn build_artifact(
    original_program: HistoryProgramV1,
    minimized_program: HistoryProgramV1,
    requested_vs_applied_faults: Vec<RequestedVsAppliedFault>,
    executed_schedule: Vec<ExecutedScheduleStep>,
    violation_kind: ViolationKind,
    failure_reason: String,
) -> CounterexampleArtifact {
    let profile = original_program.profile;
    let seed = original_program.seed;
    CounterexampleArtifact {
        schema: "mantle-store-counterexample-v1".to_string(),
        crate_version: env!("CARGO_PKG_VERSION").to_string(),
        profile,
        policy_blake3: policy_blake3_for_profile(profile),
        fault_implementation: "store-crash-reopen-adapter-and-sigabrt".to_string(),
        oracle_schema: ORACLE_SCHEMA_VERSION.to_string(),
        seed,
        original_program,
        minimized_program,
        requested_vs_applied_faults,
        executed_schedule,
        violation_kind,
        failure_reason,
    }
}
