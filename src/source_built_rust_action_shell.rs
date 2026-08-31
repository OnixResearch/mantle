//! Linux shell for enforcing and recording source-built Rust child actions.
//!
//! The pure action plan owns authority and reconciliation meaning. This module
//! owns file reads, ptrace supervision, output promotion, and evidence writes.

use std::collections::BTreeMap;
use std::fs;
use std::fs::OpenOptions;
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Mutex;

use serde::Serialize;

use crate::RunError;
use crate::protected_exec::OutputPromotionRecord;
use crate::protected_exec::PromotedExecutable;
use crate::protected_exec::ProtectedSeccompAuditEvent;
use crate::protected_exec_ptrace::ProtectedPtraceSupervisor;
use crate::rust_plan::RustChildActionExecutionPhase;
use crate::rust_plan::RustChildActionExecutionPort;
use crate::rust_plan::RustChildActionExecutionScope;
use crate::source_built_rust_action_plan::RustChildAction;
use crate::source_built_rust_action_plan::RustChildActionAuthority;
use crate::source_built_rust_action_plan::RustChildActionPhase;
use crate::source_built_rust_action_plan::RustChildActionPlan;
use crate::source_built_rust_action_plan::RustChildActionPlanInput;
use crate::source_built_rust_action_plan::RustChildActionReconciliation;
use crate::source_built_rust_action_plan::RustChildExecObservation;
use crate::source_built_rust_action_plan::RustChildExecutableAuthority;
use crate::source_built_rust_action_plan::RustFixedExecutableAuthority;
use crate::source_built_rust_action_plan::RustObservedExecutableAuthority;

pub(crate) const RUST_CHILD_ACTION_PLAN_FILE: &str = "plan.json";
pub(crate) const RUST_CHILD_ACTION_AUDIT_FILE: &str = "audit.json";
pub(crate) const RUST_CHILD_ACTION_RECONCILIATION_FILE: &str = "reconciliation.json";
const BLAKE3_HEX_LENGTH: usize = 64;
const EXECUTABLE_MODE_MASK: u32 = 0o111;

#[derive(Debug, Clone)]
struct ProducedExecutableBinding {
    producer_action_id: String,
    output_identity_blake3: String,
    resolved_path: String,
    digest_blake3: String,
}

#[derive(Debug, Default)]
struct RuntimeState {
    active_action_id: Option<String>,
    assigned_event_count: usize,
    observations: Vec<RustChildExecObservation>,
    promotions: Vec<OutputPromotionRecord>,
    produced_by_action: BTreeMap<String, ProducedExecutableBinding>,
}

#[derive(Debug)]
pub(crate) struct SourceBuiltRustActionRuntime {
    plan: RustChildActionPlan,
    evidence_dir: PathBuf,
    supervisor: ProtectedPtraceSupervisor,
    state: Mutex<RuntimeState>,
}

impl SourceBuiltRustActionRuntime {
    pub(crate) fn start(
        authority_path: &Path,
        graph: &crate::rust_plan::UnitDerivationGraphSummary,
        evidence_dir: &Path,
    ) -> Result<Self, RunError> {
        let authority = read_authority(authority_path)?;
        validate_fixed_executable_bytes(&authority.fixed_executables)?;
        let execution_unit_ids = crate::rust_plan::combined_topology_execution_unit_ids(graph).map_err(|blocker| {
            RunError::Build(format!("derive Rust child-action execution scope: {}: {}", blocker.class, blocker.message))
        })?;
        let units = crate::source_built_rust_action_plan::rust_unit_action_inputs_from_graph_selection(
            graph,
            &execution_unit_ids,
        )
        .map_err(action_error)?;
        let plan = crate::source_built_rust_action_plan::plan_rust_child_actions(RustChildActionPlanInput {
            stage_id: authority.stage_id,
            resources: authority.resources,
            fixed_executables: authority.fixed_executables,
            units,
        })
        .map_err(action_error)?;
        fs::create_dir_all(evidence_dir).map_err(|error| {
            RunError::Build(format!("create Rust child-action evidence directory {}: {error}", evidence_dir.display()))
        })?;
        write_new_json(&evidence_dir.join(RUST_CHILD_ACTION_PLAN_FILE), &plan)?;
        let (producer_ids, planned_executables) =
            crate::source_built_rust_action_plan::protected_exec_inputs(&plan).map_err(action_error)?;
        let policy = crate::protected_exec::ProtectedExecPolicy::from_action_plan(&producer_ids, &planned_executables)
            .map_err(|error| RunError::Build(format!("construct Rust child-action exec policy: {error}")))?;
        let supervisor = crate::protected_exec_ptrace::install_exec_supervisor(policy)
            .map_err(|error| RunError::Build(format!("install Rust child-action exec policy: {error}")))?;
        assert!(!plan.actions.is_empty());
        assert_eq!(plan.plan_digest_blake3.len(), BLAKE3_HEX_LENGTH);
        Ok(Self {
            plan,
            evidence_dir: evidence_dir.to_path_buf(),
            supervisor,
            state: Mutex::new(RuntimeState::default()),
        })
    }

    pub(crate) fn finish(&self) -> Result<RustChildActionReconciliation, RunError> {
        crate::protected_exec_seccomp::reap_adopted_exec_descendants()
            .map_err(|error| RunError::Build(format!("reap Rust child-action descendants: {error}")))?;
        self.supervisor
            .wait_for_audit_quiescence()
            .map_err(|error| RunError::Build(format!("wait for Rust child-action audit: {error}")))?;
        let raw_events = self.supervisor.audit_events();
        let (observations, promotions, assigned_event_count) = self.finish_state(&raw_events)?;
        let reconciliation =
            crate::source_built_rust_action_plan::rust_child_action_reconciliation(&self.plan, &observations)
                .map_err(action_error)?;
        let audit = crate::source_built_rust_action_plan::rust_child_action_audit(
            &self.plan,
            raw_events,
            observations,
            promotions,
            assigned_event_count,
        )
        .map_err(action_error)?;
        write_new_json(&self.evidence_dir.join(RUST_CHILD_ACTION_AUDIT_FILE), &audit)?;
        write_new_json(&self.evidence_dir.join(RUST_CHILD_ACTION_RECONCILIATION_FILE), &reconciliation)?;
        if !reconciliation.is_complete() {
            return Err(RunError::Build(format!(
                "Rust child-action reconciliation is incomplete: {}",
                reconciliation.blockers.join("; ")
            )));
        }
        assert_eq!(audit.raw_event_count, audit.assigned_event_count);
        assert_eq!(reconciliation.action_plan_digest_blake3, self.plan.plan_digest_blake3);
        Ok(reconciliation)
    }

    fn finish_state(
        &self,
        raw_events: &[ProtectedSeccompAuditEvent],
    ) -> Result<(Vec<RustChildExecObservation>, Vec<OutputPromotionRecord>, usize), RunError> {
        let mut state = self.lock_state()?;
        if let Some(action_id) = &state.active_action_id {
            return Err(RunError::Build(format!("Rust child action remained active at finish: {action_id}")));
        }
        if raw_events.len() > state.assigned_event_count {
            for event in &raw_events[state.assigned_event_count..] {
                state.observations.push(unassigned_observation(event));
            }
            state.assigned_event_count = raw_events.len();
        }
        if state.assigned_event_count != raw_events.len() {
            return Err(RunError::Build(format!(
                "Rust child-action assigned event count {} exceeds raw event count {}",
                state.assigned_event_count,
                raw_events.len()
            )));
        }
        Ok((state.observations.clone(), state.promotions.clone(), state.assigned_event_count))
    }

    fn lock_state(&self) -> Result<std::sync::MutexGuard<'_, RuntimeState>, RunError> {
        self.state
            .lock()
            .map_err(|_| RunError::Build("Rust child-action runtime state lock was poisoned".to_string()))
    }

    fn action(&self, unit_id: &str, phase: RustChildActionExecutionPhase) -> Result<&RustChildAction, RunError> {
        let phase = match phase {
            RustChildActionExecutionPhase::CompileUnit => RustChildActionPhase::CompileUnit,
            RustChildActionExecutionPhase::RunBuildScript => RustChildActionPhase::RunBuildScript,
        };
        crate::source_built_rust_action_plan::rust_child_action_for_unit(&self.plan, unit_id, phase)
            .map_err(action_error)
    }

    fn observe_action_events(
        &self,
        action: &RustChildAction,
        events: &[ProtectedSeccompAuditEvent],
        state: &RuntimeState,
    ) -> Vec<RustChildExecObservation> {
        events.iter().map(|event| self.observation(action, event, state)).collect()
    }

    fn observation(
        &self,
        action: &RustChildAction,
        event: &ProtectedSeccompAuditEvent,
        state: &RuntimeState,
    ) -> RustChildExecObservation {
        if let Some(binding) = state.produced_by_action.get(&action.action_id)
            && binding.resolved_path == path_string(&event.resolved_host_path)
            && binding.digest_blake3 == event.digest_hex
        {
            return RustChildExecObservation {
                action_id: action.action_id.clone(),
                executable: RustObservedExecutableAuthority::Produced {
                    producer_action_id: binding.producer_action_id.clone(),
                    output_identity_blake3: binding.output_identity_blake3.clone(),
                },
                resolved_path: binding.resolved_path.clone(),
                digest_blake3: event.digest_hex.clone(),
                policy_decision: event.policy_decision.clone(),
            };
        }
        let authority_id = matching_fixed_authority(&self.plan.fixed_executables, event)
            .map_or_else(|| unbound_authority_id(event), |authority| authority.authority_id.clone());
        RustChildExecObservation {
            action_id: action.action_id.clone(),
            executable: RustObservedExecutableAuthority::Fixed { authority_id },
            resolved_path: path_string(&event.resolved_host_path),
            digest_blake3: normalized_event_digest(event),
            policy_decision: event.policy_decision.clone(),
        }
    }
}

impl RustChildActionExecutionPort for SourceBuiltRustActionRuntime {
    fn run_output(&self, command: &mut std::process::Command) -> Result<std::process::Output, RunError> {
        self.supervisor
            .output(command)
            .map_err(|error| RunError::Build(format!("run ptrace-supervised Rust child action: {error}")))
    }

    fn begin_action(
        &self,
        unit_id: &str,
        phase: RustChildActionExecutionPhase,
    ) -> Result<RustChildActionExecutionScope, RunError> {
        let action = self.action(unit_id, phase)?;
        let current_event_count = self.supervisor.audit_events().len();
        let mut state = self.lock_state()?;
        if let Some(active) = &state.active_action_id {
            return Err(RunError::Build(format!("Rust child action {active} overlaps {}", action.action_id)));
        }
        if current_event_count != state.assigned_event_count {
            return Err(RunError::Build(format!(
                "Rust child-action audit has {} unassigned events before {}",
                current_event_count.saturating_sub(state.assigned_event_count),
                action.action_id
            )));
        }
        self.supervisor
            .begin_producer_action(&action.action_id)
            .map_err(|error| RunError::Build(format!("begin Rust producer action {}: {error}", action.action_id)))?;
        state.active_action_id = Some(action.action_id.clone());
        assert_eq!(current_event_count, state.assigned_event_count);
        assert!(state.active_action_id.is_some());
        Ok(RustChildActionExecutionScope {
            action_id: action.action_id.clone(),
            audit_event_start: current_event_count,
        })
    }

    fn end_action(&self, scope: RustChildActionExecutionScope) -> Result<(), RunError> {
        self.supervisor
            .wait_for_audit_quiescence()
            .map_err(|error| RunError::Build(format!("wait for Rust child-action audit: {error}")))?;
        let raw_events = self.supervisor.audit_events();
        if scope.audit_event_start > raw_events.len() {
            return Err(RunError::Build(format!(
                "Rust child action {} audit start exceeds current event count",
                scope.action_id
            )));
        }
        let action = self
            .plan
            .actions
            .iter()
            .find(|action| action.action_id == scope.action_id)
            .ok_or_else(|| RunError::Build(format!("Rust child action disappeared: {}", scope.action_id)))?;
        let mut state = self.lock_state()?;
        if state.active_action_id.as_deref() != Some(scope.action_id.as_str()) {
            return Err(RunError::Build(format!("Rust child action scope drifted: {}", scope.action_id)));
        }
        let observations = self.observe_action_events(action, &raw_events[scope.audit_event_start..], &state);
        self.supervisor
            .end_producer_action(&scope.action_id)
            .map_err(|error| RunError::Build(format!("end Rust producer action {}: {error}", scope.action_id)))?;
        state.observations.extend(observations);
        state.assigned_event_count = raw_events.len();
        state.active_action_id = None;
        assert_eq!(state.assigned_event_count, raw_events.len());
        assert!(state.active_action_id.is_none());
        Ok(())
    }

    fn promote_build_script(&self, unit_id: &str, executable: &Path) -> Result<(), RunError> {
        let action = self.action(unit_id, RustChildActionExecutionPhase::RunBuildScript)?;
        let RustChildExecutableAuthority::Produced {
            producer_action_id,
            output_identity_blake3,
        } = &action.executable
        else {
            return Err(RunError::Build(format!(
                "Rust build-script action {} lacks produced executable authority",
                action.action_id
            )));
        };
        let resolved_path = canonical_executable(executable)?;
        let digest_blake3 = crate::protected_exec::blake3_file_hex(&resolved_path)
            .map_err(|error| RunError::Build(format!("hash Rust build script {}: {error}", resolved_path.display())))?;
        let promotion = self
            .supervisor
            .promote_verified_output(producer_action_id, &[format!("output-identity:{output_identity_blake3}")], &[
                PromotedExecutable {
                    path: resolved_path.clone(),
                    digest_hex: digest_blake3.clone(),
                },
            ])
            .map_err(|error| {
                RunError::Build(format!("promote Rust build script {}: {error}", resolved_path.display()))
            })?;
        let binding = ProducedExecutableBinding {
            producer_action_id: producer_action_id.clone(),
            output_identity_blake3: output_identity_blake3.clone(),
            resolved_path: path_string(&resolved_path),
            digest_blake3,
        };
        let mut state = self.lock_state()?;
        if state.produced_by_action.insert(action.action_id.clone(), binding).is_some() {
            return Err(RunError::Build(format!("Rust build script was promoted twice: {}", action.action_id)));
        }
        state.promotions.push(promotion);
        assert!(state.produced_by_action.contains_key(&action.action_id));
        assert!(!state.promotions.is_empty());
        Ok(())
    }
}

fn read_authority(path: &Path) -> Result<RustChildActionAuthority, RunError> {
    let bytes = fs::read(path)
        .map_err(|error| RunError::Build(format!("read Rust child-action authority {}: {error}", path.display())))?;
    let authority = serde_json::from_slice::<RustChildActionAuthority>(&bytes)
        .map_err(|error| RunError::Build(format!("parse Rust child-action authority {}: {error}", path.display())))?;
    crate::source_built_rust_action_plan::validate_rust_child_action_authority(&authority).map_err(action_error)?;
    assert!(!bytes.is_empty());
    assert!(!authority.fixed_executables.is_empty());
    Ok(authority)
}

fn validate_fixed_executable_bytes(executables: &[RustFixedExecutableAuthority]) -> Result<(), RunError> {
    for executable in executables {
        let path = Path::new(&executable.path);
        let resolved = canonical_executable(path)?;
        if resolved != path {
            return Err(RunError::Build(format!(
                "Rust child-action fixed executable path is not canonical: {}",
                path.display()
            )));
        }
        let observed = crate::protected_exec::blake3_file_hex(path)
            .map_err(|error| RunError::Build(format!("hash Rust fixed executable {}: {error}", path.display())))?;
        if observed != executable.digest_blake3 {
            return Err(RunError::Build(format!(
                "Rust child-action fixed executable digest mismatch for {}: expected {}, got {observed}",
                executable.authority_id, executable.digest_blake3
            )));
        }
    }
    assert!(!executables.is_empty());
    assert!(executables.iter().all(|executable| Path::new(&executable.path).is_absolute()));
    Ok(())
}

fn canonical_executable(path: &Path) -> Result<PathBuf, RunError> {
    let resolved = fs::canonicalize(path)
        .map_err(|error| RunError::Build(format!("resolve Rust child executable {}: {error}", path.display())))?;
    let metadata = fs::metadata(&resolved)
        .map_err(|error| RunError::Build(format!("inspect Rust child executable {}: {error}", resolved.display())))?;
    if !metadata.is_file() || metadata.permissions().mode() & EXECUTABLE_MODE_MASK == 0 {
        return Err(RunError::Build(format!(
            "Rust child executable is not an executable file: {}",
            resolved.display()
        )));
    }
    assert!(resolved.is_absolute());
    assert!(metadata.is_file());
    Ok(resolved)
}

fn matching_fixed_authority<'a>(
    fixed: &'a [RustFixedExecutableAuthority],
    event: &ProtectedSeccompAuditEvent,
) -> Option<&'a RustFixedExecutableAuthority> {
    let path = path_string(&event.resolved_host_path);
    fixed.iter().find(|authority| authority.path == path && authority.digest_blake3 == event.digest_hex)
}

fn unassigned_observation(event: &ProtectedSeccompAuditEvent) -> RustChildExecObservation {
    RustChildExecObservation {
        action_id: format!("unassigned:{}", event_id(event)),
        executable: RustObservedExecutableAuthority::Fixed {
            authority_id: unbound_authority_id(event),
        },
        resolved_path: path_string(&event.resolved_host_path),
        digest_blake3: normalized_event_digest(event),
        policy_decision: event.policy_decision.clone(),
    }
}

fn unbound_authority_id(event: &ProtectedSeccompAuditEvent) -> String {
    format!("unbound:{}", event_id(event))
}

fn event_id(event: &ProtectedSeccompAuditEvent) -> String {
    let bytes = serde_json::to_vec(event).expect("protected seccomp audit events serialize");
    blake3::hash(&bytes).to_hex().to_string()
}

fn normalized_event_digest(event: &ProtectedSeccompAuditEvent) -> String {
    if event.digest_hex.len() == BLAKE3_HEX_LENGTH {
        return event.digest_hex.clone();
    }
    event_id(event)
}

fn write_new_json<T: Serialize>(path: &Path, value: &T) -> Result<(), RunError> {
    let bytes = serde_json::to_vec_pretty(value)
        .map_err(|error| RunError::Build(format!("encode Rust child-action evidence {}: {error}", path.display())))?;
    let parent = path
        .parent()
        .ok_or_else(|| RunError::Build(format!("Rust child-action evidence path has no parent: {}", path.display())))?;
    fs::create_dir_all(parent).map_err(|error| {
        RunError::Build(format!("create Rust child-action evidence parent {}: {error}", parent.display()))
    })?;
    let mut file =
        OpenOptions::new().write(true).create_new(true).open(path).map_err(|error| {
            RunError::Build(format!("create Rust child-action evidence {}: {error}", path.display()))
        })?;
    file.write_all(&bytes)
        .map_err(|error| RunError::Build(format!("write Rust child-action evidence {}: {error}", path.display())))?;
    file.sync_all()
        .map_err(|error| RunError::Build(format!("sync Rust child-action evidence {}: {error}", path.display())))?;
    assert!(!bytes.is_empty());
    assert!(path.is_file());
    Ok(())
}

fn path_string(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

fn action_error(error: crate::source_built_rust_action_plan::RustChildActionPlanError) -> RunError {
    RunError::Build(format!("Rust child-action authority blocked: {error}"))
}

#[cfg(test)]
#[path = "source_built_rust_action_shell_tests.rs"]
mod tests;
