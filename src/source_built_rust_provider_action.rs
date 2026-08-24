//! Staged action authority for full-source Rust-provider construction.
//!
//! Each bootstrap stage writes its plan before execution. The Linux shell then
//! scopes seccomp observations to that producer action. Executables created
//! under a declared output root receive a BLAKE3 identity at first exec and are
//! pinned before the kernel continues the launch.

use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::fs;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Component;
use std::path::Path;
use std::path::PathBuf;

use serde::Deserialize;
use serde::Serialize;

use crate::full_source_rust_binding::FullSourceNativeArtifactRole;
use crate::full_source_rust_binding_shell::FullSourceRustExecutionContext;
use crate::protected_exec::OutputPromotionRecord;
use crate::protected_exec::PlannedExecutable;
use crate::protected_exec::PlannedProducedExecutableRoot;
use crate::protected_exec::ProtectedSeccompAuditEvent;
use crate::protected_exec_seccomp::ProtectedSeccompSupervisor;
use crate::rust_source_provider::RustSourceProviderError;
use crate::source_toolchain_closure::RustSourceProviderBootstrapPlan;
use crate::source_toolchain_closure::RustSourceProviderBootstrapStageKind;

pub(crate) const RUST_PROVIDER_ACTION_PLAN_FILE: &str = "rust-provider-action-plan.json";
pub(crate) const RUST_PROVIDER_ACTION_AUDIT_FILE: &str = "rust-provider-action-audit.json";
pub(crate) const RUST_PROVIDER_ACTION_RECONCILIATION_FILE: &str = "rust-provider-action-reconciliation.json";
const AUTHORITY_FILE: &str = "rust-provider-action-authority.json";
const STAGE_EVIDENCE_DIR: &str = "stages";
const AUTHORITY_SCHEMA: &str = "mantle-source-built-rust-provider-action-authority-v1";
const STAGE_PLAN_SCHEMA: &str = "mantle-source-built-rust-provider-stage-action-plan-v1";
const STAGE_RECONCILIATION_SCHEMA: &str = "mantle-source-built-rust-provider-stage-action-reconciliation-v1";
const AGGREGATE_PLAN_SCHEMA: &str = "mantle-source-built-rust-provider-action-plan-v1";
const AGGREGATE_AUDIT_SCHEMA: &str = "mantle-source-built-rust-provider-action-audit-v1";
const AGGREGATE_RECONCILIATION_SCHEMA: &str = "mantle-source-built-rust-provider-action-reconciliation-v1";
const AUTHORITY_DIGEST_CONTEXT: &[u8] = b"mantle-source-built-rust-provider-action-authority-v1\0";
const STAGE_PLAN_DIGEST_CONTEXT: &[u8] = b"mantle-source-built-rust-provider-stage-action-plan-v1\0";
const STAGE_RECONCILIATION_DIGEST_CONTEXT: &[u8] =
    b"mantle-source-built-rust-provider-stage-action-reconciliation-v1\0";
const AGGREGATE_PLAN_DIGEST_CONTEXT: &[u8] = b"mantle-source-built-rust-provider-action-plan-v1\0";
const AGGREGATE_AUDIT_DIGEST_CONTEXT: &[u8] = b"mantle-source-built-rust-provider-action-audit-v1\0";
const AGGREGATE_RECONCILIATION_DIGEST_CONTEXT: &[u8] = b"mantle-source-built-rust-provider-action-reconciliation-v1\0";
const OUTPUT_IDENTITY_CONTEXT: &[u8] = b"mantle-source-built-rust-provider-output-root-v1\0";
const FIXED_AUTHORITY_CONTEXT: &[u8] = b"mantle-source-built-rust-provider-fixed-executable-v1\0";
const BLAKE3_HEX_LENGTH: usize = 64;
const STAGE_COUNT_MAX: usize = 16;
const FIXED_EXECUTABLE_COUNT_MAX: usize = 256;
const OUTPUT_ROOT_COUNT_MAX: usize = 16;
const TEXT_BYTES_MAX: usize = 4_096;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct RustProviderActionLimits {
    pub(crate) parallel_jobs_max: u32,
    pub(crate) open_file_descriptors_max: u32,
    pub(crate) storage_bytes_max: u64,
    pub(crate) exec_events_per_stage_max: u32,
}

#[derive(Debug, Clone)]
pub(crate) struct RustProviderStageActionInput {
    pub(crate) stage_id: String,
    pub(crate) stage_kind: RustSourceProviderBootstrapStageKind,
    pub(crate) predecessor_stage_id: Option<String>,
    pub(crate) plan_path: PathBuf,
    pub(crate) script_path: PathBuf,
    pub(crate) sources_manifest_path: PathBuf,
    pub(crate) bootstrap_metadata_path: Option<PathBuf>,
    pub(crate) output_roots: Vec<PathBuf>,
}

#[derive(Debug, Clone)]
pub(crate) struct RustProviderStageScope {
    stage_id: String,
    plan_digest_blake3: String,
    event_start: usize,
    promotion_start: usize,
    reconciliation_path: PathBuf,
}

#[derive(Debug, Clone)]
pub(crate) struct RustProviderActionEvidence {
    pub(crate) plan_path: PathBuf,
    pub(crate) plan_digest_blake3: String,
    pub(crate) audit_path: PathBuf,
    pub(crate) audit_digest_blake3: String,
    pub(crate) reconciliation_path: PathBuf,
    pub(crate) reconciliation_digest_blake3: String,
    pub(crate) planned_action_count: u32,
    pub(crate) matched_action_count: u32,
    pub(crate) observed_event_count: u32,
    pub(crate) matched_event_count: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct FixedExecutableAuthority {
    authority_id: String,
    producer_action_id: String,
    path: String,
    digest_blake3: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct RustProviderActionAuthority {
    schema: String,
    route_plan_digest_blake3: String,
    limits: RustProviderActionLimits,
    producer_action_ids: Vec<String>,
    fixed_executables: Vec<FixedExecutableAuthority>,
    authority_digest_blake3: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct PlannedOutputRoot {
    root: String,
    output_identity_blake3: String,
    promotion_count_max: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct RustProviderStageActionPlan {
    schema: String,
    route_plan_digest_blake3: String,
    authority_digest_blake3: String,
    stage_id: String,
    stage_kind: RustSourceProviderBootstrapStageKind,
    producer_action_ids: Vec<String>,
    input_digests_blake3: BTreeMap<String, String>,
    output_roots: Vec<PlannedOutputRoot>,
    limits: RustProviderActionLimits,
    local_only: bool,
    cache_only_completion_allowed: bool,
    plan_digest_blake3: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct RustProviderStageActionReconciliation {
    schema: String,
    action_plan_digest_blake3: String,
    execution_succeeded: bool,
    observed_event_count: u32,
    matched_event_count: u32,
    denied_event_count: u32,
    promotion_count: u32,
    local_only: bool,
    blockers: Vec<String>,
    reconciliation_digest_blake3: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct RustProviderAggregateActionPlan {
    schema: String,
    route_plan_digest_blake3: String,
    authority_digest_blake3: String,
    adapter_count: u32,
    action_count: u32,
    stage_plan_digests_blake3: Vec<String>,
    local_only: bool,
    cache_only_completion_allowed: bool,
    blockers: Vec<String>,
    plan_digest_blake3: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct RustProviderAggregateAudit {
    schema: String,
    action_plan_digest_blake3: String,
    raw_event_count: u32,
    promotion_count: u32,
    raw_events: Vec<ProtectedSeccompAuditEvent>,
    promotions: Vec<OutputPromotionRecord>,
    audit_digest_blake3: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct RustProviderAggregateReconciliation {
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

#[derive(Debug, Clone)]
struct CompletedStage {
    plan_digest_blake3: String,
    reconciliation: RustProviderStageActionReconciliation,
}

#[derive(Debug)]
pub(crate) struct RustProviderActionRuntime {
    authority: RustProviderActionAuthority,
    evidence_dir: PathBuf,
    expected_stage_ids: BTreeSet<String>,
    completed: BTreeMap<String, CompletedStage>,
    supervisor: ProtectedSeccompSupervisor,
}

impl RustProviderActionRuntime {
    pub(crate) fn start(
        context: &FullSourceRustExecutionContext,
        route: &RustSourceProviderBootstrapPlan,
        route_plan_digest_blake3: &str,
        evidence_dir: &Path,
        limits: RustProviderActionLimits,
    ) -> Result<Self, RustSourceProviderError> {
        validate_limits(&limits)?;
        validate_digest("route plan", route_plan_digest_blake3)?;
        let stage_ids = normalized_stage_ids(route)?;
        let fixed_executables = fixed_executable_authority(context)?;
        let mut producer_action_ids = stage_ids.iter().cloned().collect::<Vec<_>>();
        producer_action_ids.extend(fixed_executables.iter().map(|executable| executable.producer_action_id.clone()));
        producer_action_ids.sort();
        producer_action_ids.dedup();
        let mut authority = RustProviderActionAuthority {
            schema: AUTHORITY_SCHEMA.to_string(),
            route_plan_digest_blake3: route_plan_digest_blake3.to_string(),
            limits,
            producer_action_ids: producer_action_ids.clone(),
            fixed_executables,
            authority_digest_blake3: String::new(),
        };
        authority.authority_digest_blake3 = digest_serialized(AUTHORITY_DIGEST_CONTEXT, &authority)?;
        fs::create_dir_all(evidence_dir).map_err(|error| {
            provider_error(format!("creating Rust provider action evidence {}: {error}", evidence_dir.display()))
        })?;
        write_json_create_new(&evidence_dir.join(AUTHORITY_FILE), &authority)?;
        let planned = authority
            .fixed_executables
            .iter()
            .map(|fixed| PlannedExecutable {
                authorization_id: fixed.authority_id.clone(),
                source_stage_id: fixed.producer_action_id.clone(),
                path: PathBuf::from(&fixed.path),
                digest_hex: fixed.digest_blake3.clone(),
            })
            .collect::<Vec<_>>();
        let policy = crate::protected_exec::ProtectedExecPolicy::from_action_plan(&producer_action_ids, &planned)
            .map_err(|error| provider_error(format!("constructing Rust provider exec policy: {error}")))?;
        let supervisor = crate::protected_exec_seccomp::install_current_thread_exec_supervisor(policy)
            .map_err(|error| provider_error(format!("installing Rust provider exec policy: {error}")))?;
        assert!(!authority.fixed_executables.is_empty());
        assert_eq!(authority.authority_digest_blake3.len(), BLAKE3_HEX_LENGTH);
        Ok(Self {
            authority,
            evidence_dir: evidence_dir.to_path_buf(),
            expected_stage_ids: stage_ids,
            completed: BTreeMap::new(),
            supervisor,
        })
    }

    pub(crate) fn begin_stage(
        &mut self,
        input: RustProviderStageActionInput,
    ) -> Result<RustProviderStageScope, RustSourceProviderError> {
        if !self.expected_stage_ids.contains(&input.stage_id) || self.completed.contains_key(&input.stage_id) {
            return Err(provider_error(format!(
                "Rust provider action stage is unknown or repeated: {}",
                input.stage_id
            )));
        }
        let plan = self.stage_plan(input)?;
        let stage_file = safe_stage_file(&plan.stage_id)?;
        let stage_dir = self.evidence_dir.join(STAGE_EVIDENCE_DIR);
        fs::create_dir_all(&stage_dir).map_err(|error| {
            provider_error(format!("creating Rust provider stage evidence {}: {error}", stage_dir.display()))
        })?;
        let plan_path = stage_dir.join(format!("{stage_file}-plan.json"));
        let reconciliation_path = stage_dir.join(format!("{stage_file}-reconciliation.json"));
        write_json_create_new(&plan_path, &plan)?;
        let roots = plan
            .output_roots
            .iter()
            .map(|root| PlannedProducedExecutableRoot {
                producer_action_id: plan.stage_id.clone(),
                output_identity_blake3: root.output_identity_blake3.clone(),
                root: PathBuf::from(&root.root),
                promotion_count_max: root.promotion_count_max,
            })
            .collect::<Vec<_>>();
        self.supervisor
            .register_planned_produced_roots(&roots)
            .map_err(|error| provider_error(format!("registering Rust provider output roots: {error}")))?;
        self.supervisor
            .begin_producer_action(&plan.stage_id)
            .map_err(|error| provider_error(format!("starting Rust provider action {}: {error}", plan.stage_id)))?;
        let event_start = self.supervisor.audit_events().len();
        let promotion_start = self.supervisor.automatic_promotions().len();
        assert!(plan_path.is_file());
        assert_eq!(plan.plan_digest_blake3.len(), BLAKE3_HEX_LENGTH);
        Ok(RustProviderStageScope {
            stage_id: plan.stage_id,
            plan_digest_blake3: plan.plan_digest_blake3,
            event_start,
            promotion_start,
            reconciliation_path,
        })
    }

    pub(crate) fn end_stage(
        &mut self,
        scope: RustProviderStageScope,
        execution_succeeded: bool,
    ) -> Result<(), RustSourceProviderError> {
        self.supervisor
            .end_producer_action(&scope.stage_id)
            .map_err(|error| provider_error(format!("ending Rust provider action {}: {error}", scope.stage_id)))?;
        let events = self.supervisor.audit_events();
        let promotions = self.supervisor.automatic_promotions();
        if scope.event_start > events.len() || scope.promotion_start > promotions.len() {
            return Err(provider_error(format!("Rust provider action {} observation cursor drifted", scope.stage_id)));
        }
        let stage_events = &events[scope.event_start..];
        let stage_promotions = &promotions[scope.promotion_start..];
        let reconciliation = stage_reconciliation(
            &scope.plan_digest_blake3,
            execution_succeeded,
            stage_events,
            stage_promotions,
            self.authority.limits.exec_events_per_stage_max,
        )?;
        write_json_create_new(&scope.reconciliation_path, &reconciliation)?;
        self.completed.insert(scope.stage_id.clone(), CompletedStage {
            plan_digest_blake3: scope.plan_digest_blake3,
            reconciliation: reconciliation.clone(),
        });
        if !reconciliation.blockers.is_empty() {
            return Err(provider_error(format!(
                "Rust provider action {} reconciliation is incomplete: {}",
                scope.stage_id,
                reconciliation.blockers.join("; ")
            )));
        }
        assert!(self.completed.contains_key(&scope.stage_id));
        assert_eq!(reconciliation.observed_event_count, reconciliation.matched_event_count);
        Ok(())
    }

    pub(crate) fn finish(mut self) -> Result<RustProviderActionEvidence, RustSourceProviderError> {
        crate::protected_exec_seccomp::reap_adopted_exec_descendants()
            .map_err(|error| provider_error(format!("reaping Rust provider descendants: {error}")))?;
        self.supervisor
            .wait_for_audit_quiescence()
            .map_err(|error| provider_error(format!("waiting for Rust provider audit: {error}")))?;
        let missing = self
            .expected_stage_ids
            .difference(&self.completed.keys().cloned().collect())
            .cloned()
            .collect::<Vec<_>>();
        if !missing.is_empty() {
            return Err(provider_error(format!(
                "Rust provider action evidence is missing stages: {}",
                missing.join(",")
            )));
        }
        let (plan, reconciliation) = self.aggregate_documents()?;
        let events = self.supervisor.audit_events();
        let promotions = self.supervisor.automatic_promotions();
        let mut audit = RustProviderAggregateAudit {
            schema: AGGREGATE_AUDIT_SCHEMA.to_string(),
            action_plan_digest_blake3: plan.plan_digest_blake3.clone(),
            raw_event_count: bounded_count("Rust provider audit event", events.len())?,
            promotion_count: bounded_count("Rust provider promotion", promotions.len())?,
            raw_events: events,
            promotions,
            audit_digest_blake3: String::new(),
        };
        audit.audit_digest_blake3 = digest_serialized(AGGREGATE_AUDIT_DIGEST_CONTEXT, &audit)?;
        let plan_path = self.evidence_dir.join(RUST_PROVIDER_ACTION_PLAN_FILE);
        let audit_path = self.evidence_dir.join(RUST_PROVIDER_ACTION_AUDIT_FILE);
        let reconciliation_path = self.evidence_dir.join(RUST_PROVIDER_ACTION_RECONCILIATION_FILE);
        write_json_create_new(&plan_path, &plan)?;
        write_json_create_new(&audit_path, &audit)?;
        write_json_create_new(&reconciliation_path, &reconciliation)?;
        if audit.raw_event_count != reconciliation.observed_event_count {
            return Err(provider_error(format!(
                "Rust provider audit has {} events but stage reconciliation assigned {}",
                audit.raw_event_count, reconciliation.observed_event_count
            )));
        }
        assert_eq!(plan.action_count, reconciliation.matched_action_count);
        assert_eq!(audit.raw_event_count, reconciliation.observed_event_count);
        Ok(RustProviderActionEvidence {
            plan_path,
            plan_digest_blake3: plan.plan_digest_blake3,
            audit_path,
            audit_digest_blake3: audit.audit_digest_blake3,
            reconciliation_path,
            reconciliation_digest_blake3: reconciliation.reconciliation_digest_blake3,
            planned_action_count: reconciliation.planned_action_count,
            matched_action_count: reconciliation.matched_action_count,
            observed_event_count: reconciliation.observed_event_count,
            matched_event_count: reconciliation.matched_event_count,
        })
    }

    fn stage_plan(
        &self,
        input: RustProviderStageActionInput,
    ) -> Result<RustProviderStageActionPlan, RustSourceProviderError> {
        validate_text("stage id", &input.stage_id)?;
        if input.output_roots.is_empty() || input.output_roots.len() > OUTPUT_ROOT_COUNT_MAX {
            return Err(provider_error(format!(
                "Rust provider stage {} output-root count is outside 1..={OUTPUT_ROOT_COUNT_MAX}",
                input.stage_id
            )));
        }
        let mut input_digests_blake3 = BTreeMap::from([
            ("stage-plan".to_string(), file_digest(&input.plan_path)?),
            ("stage-script".to_string(), file_digest(&input.script_path)?),
            ("stage-sources".to_string(), file_digest(&input.sources_manifest_path)?),
        ]);
        if let Some(path) = input.bootstrap_metadata_path {
            input_digests_blake3.insert("bootstrap-metadata".to_string(), file_digest(&path)?);
        }
        let mut output_roots = normalized_output_roots(
            &input.stage_id,
            input.output_roots,
            self.authority.limits.exec_events_per_stage_max,
        )?;
        output_roots.sort_by(|left, right| left.root.cmp(&right.root));
        let producer_action_ids = input.predecessor_stage_id.into_iter().collect::<Vec<_>>();
        let mut plan = RustProviderStageActionPlan {
            schema: STAGE_PLAN_SCHEMA.to_string(),
            route_plan_digest_blake3: self.authority.route_plan_digest_blake3.clone(),
            authority_digest_blake3: self.authority.authority_digest_blake3.clone(),
            stage_id: input.stage_id,
            stage_kind: input.stage_kind,
            producer_action_ids,
            input_digests_blake3,
            output_roots,
            limits: self.authority.limits.clone(),
            local_only: true,
            cache_only_completion_allowed: false,
            plan_digest_blake3: String::new(),
        };
        plan.plan_digest_blake3 = digest_serialized(STAGE_PLAN_DIGEST_CONTEXT, &plan)?;
        assert!(!plan.output_roots.is_empty());
        assert_eq!(plan.plan_digest_blake3.len(), BLAKE3_HEX_LENGTH);
        Ok(plan)
    }

    fn aggregate_documents(
        &mut self,
    ) -> Result<(RustProviderAggregateActionPlan, RustProviderAggregateReconciliation), RustSourceProviderError> {
        let stage_plan_digests_blake3 =
            self.completed.values().map(|stage| stage.plan_digest_blake3.clone()).collect::<Vec<_>>();
        let action_count = bounded_count("Rust provider action", stage_plan_digests_blake3.len())?;
        let mut plan = RustProviderAggregateActionPlan {
            schema: AGGREGATE_PLAN_SCHEMA.to_string(),
            route_plan_digest_blake3: self.authority.route_plan_digest_blake3.clone(),
            authority_digest_blake3: self.authority.authority_digest_blake3.clone(),
            adapter_count: 1,
            action_count,
            stage_plan_digests_blake3,
            local_only: true,
            cache_only_completion_allowed: false,
            blockers: Vec::new(),
            plan_digest_blake3: String::new(),
        };
        plan.plan_digest_blake3 = digest_serialized(AGGREGATE_PLAN_DIGEST_CONTEXT, &plan)?;
        let observed_event_count = self
            .completed
            .values()
            .try_fold(0_u32, |count, stage| count.checked_add(stage.reconciliation.observed_event_count))
            .ok_or_else(|| provider_error("Rust provider observed event count overflow".to_string()))?;
        let matched_event_count = self
            .completed
            .values()
            .try_fold(0_u32, |count, stage| count.checked_add(stage.reconciliation.matched_event_count))
            .ok_or_else(|| provider_error("Rust provider matched event count overflow".to_string()))?;
        if observed_event_count > self.authority.limits.exec_events_per_stage_max {
            return Err(provider_error(format!(
                "Rust provider aggregate exec event count {observed_event_count} exceeds {}",
                self.authority.limits.exec_events_per_stage_max
            )));
        }
        let mut reconciliation = RustProviderAggregateReconciliation {
            schema: AGGREGATE_RECONCILIATION_SCHEMA.to_string(),
            action_plan_digest_blake3: plan.plan_digest_blake3.clone(),
            planned_action_count: action_count,
            matched_action_count: action_count,
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
            digest_serialized(AGGREGATE_RECONCILIATION_DIGEST_CONTEXT, &reconciliation)?;
        assert_eq!(plan.action_count, reconciliation.matched_action_count);
        assert_eq!(observed_event_count, matched_event_count);
        Ok((plan, reconciliation))
    }
}

pub(crate) fn validate_rust_provider_action_evidence(
    evidence_dir: &Path,
) -> Result<RustProviderActionEvidence, RustSourceProviderError> {
    let plan_path = evidence_dir.join(RUST_PROVIDER_ACTION_PLAN_FILE);
    let audit_path = evidence_dir.join(RUST_PROVIDER_ACTION_AUDIT_FILE);
    let reconciliation_path = evidence_dir.join(RUST_PROVIDER_ACTION_RECONCILIATION_FILE);
    let plan: RustProviderAggregateActionPlan = read_json(&plan_path)?;
    let audit: RustProviderAggregateAudit = read_json(&audit_path)?;
    let reconciliation: RustProviderAggregateReconciliation = read_json(&reconciliation_path)?;
    let plan_digest = digest_serialized(AGGREGATE_PLAN_DIGEST_CONTEXT, &RustProviderAggregateActionPlan {
        plan_digest_blake3: String::new(),
        ..plan.clone()
    })?;
    let audit_digest = digest_serialized(AGGREGATE_AUDIT_DIGEST_CONTEXT, &RustProviderAggregateAudit {
        audit_digest_blake3: String::new(),
        ..audit.clone()
    })?;
    let reconciliation_digest =
        digest_serialized(AGGREGATE_RECONCILIATION_DIGEST_CONTEXT, &RustProviderAggregateReconciliation {
            reconciliation_digest_blake3: String::new(),
            ..reconciliation.clone()
        })?;
    let identity_valid = plan.schema == AGGREGATE_PLAN_SCHEMA
        && audit.schema == AGGREGATE_AUDIT_SCHEMA
        && reconciliation.schema == AGGREGATE_RECONCILIATION_SCHEMA
        && plan.plan_digest_blake3 == plan_digest
        && audit.audit_digest_blake3 == audit_digest
        && reconciliation.reconciliation_digest_blake3 == reconciliation_digest
        && audit.action_plan_digest_blake3 == plan.plan_digest_blake3
        && reconciliation.action_plan_digest_blake3 == plan.plan_digest_blake3;
    let counts_valid = plan.action_count > 0
        && plan.action_count == reconciliation.planned_action_count
        && reconciliation.planned_action_count == reconciliation.matched_action_count
        && audit.raw_event_count == reconciliation.observed_event_count
        && reconciliation.observed_event_count == reconciliation.matched_event_count
        && usize::try_from(audit.raw_event_count).ok() == Some(audit.raw_events.len());
    let policy_valid = plan.local_only
        && reconciliation.local_only
        && !plan.cache_only_completion_allowed
        && plan.blockers.is_empty()
        && reconciliation.blockers.is_empty()
        && reconciliation.unknown_event_count == 0
        && reconciliation.missing_action_count == 0
        && reconciliation.authority_violation_count == 0
        && reconciliation.fallback_event_count == 0
        && reconciliation.remote_event_count == 0
        && reconciliation.cache_only_completion_count == 0;
    if !identity_valid || !counts_valid || !policy_valid {
        return Err(provider_error("Rust provider aggregate action evidence is incomplete or mismatched".to_string()));
    }
    validate_stage_evidence(evidence_dir, &plan)?;
    Ok(RustProviderActionEvidence {
        plan_path,
        plan_digest_blake3: plan.plan_digest_blake3,
        audit_path,
        audit_digest_blake3: audit.audit_digest_blake3,
        reconciliation_path,
        reconciliation_digest_blake3: reconciliation.reconciliation_digest_blake3,
        planned_action_count: reconciliation.planned_action_count,
        matched_action_count: reconciliation.matched_action_count,
        observed_event_count: reconciliation.observed_event_count,
        matched_event_count: reconciliation.matched_event_count,
    })
}

fn validate_stage_evidence(
    evidence_dir: &Path,
    aggregate: &RustProviderAggregateActionPlan,
) -> Result<(), RustSourceProviderError> {
    let stage_dir = evidence_dir.join(STAGE_EVIDENCE_DIR);
    let mut plan_digests = Vec::new();
    let entries = fs::read_dir(&stage_dir).map_err(|error| {
        provider_error(format!("reading Rust provider stage evidence {}: {error}", stage_dir.display()))
    })?;
    for entry in entries {
        let path = entry.map_err(|error| provider_error(format!("reading Rust provider stage entry: {error}")))?.path();
        let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
            return Err(provider_error("Rust provider stage evidence name is not UTF-8".to_string()));
        };
        if !name.ends_with("-plan.json") {
            continue;
        }
        let plan: RustProviderStageActionPlan = read_json(&path)?;
        let expected_digest = digest_serialized(STAGE_PLAN_DIGEST_CONTEXT, &RustProviderStageActionPlan {
            plan_digest_blake3: String::new(),
            ..plan.clone()
        })?;
        if plan.schema != STAGE_PLAN_SCHEMA || plan.plan_digest_blake3 != expected_digest {
            return Err(provider_error(format!("Rust provider stage plan is invalid: {}", path.display())));
        }
        let prefix = name.trim_end_matches("-plan.json");
        let reconciliation_path = stage_dir.join(format!("{prefix}-reconciliation.json"));
        let reconciliation: RustProviderStageActionReconciliation = read_json(&reconciliation_path)?;
        let expected_reconciliation =
            digest_serialized(STAGE_RECONCILIATION_DIGEST_CONTEXT, &RustProviderStageActionReconciliation {
                reconciliation_digest_blake3: String::new(),
                ..reconciliation.clone()
            })?;
        if reconciliation.schema != STAGE_RECONCILIATION_SCHEMA
            || reconciliation.action_plan_digest_blake3 != plan.plan_digest_blake3
            || reconciliation.reconciliation_digest_blake3 != expected_reconciliation
            || !reconciliation.blockers.is_empty()
        {
            return Err(provider_error(format!(
                "Rust provider stage reconciliation is invalid: {}",
                reconciliation_path.display()
            )));
        }
        plan_digests.push(plan.plan_digest_blake3);
    }
    plan_digests.sort();
    let mut expected = aggregate.stage_plan_digests_blake3.clone();
    expected.sort();
    if plan_digests != expected || plan_digests.len() != usize::try_from(aggregate.action_count).unwrap_or(usize::MAX) {
        return Err(provider_error(
            "Rust provider stage evidence does not cover the aggregate action plan".to_string(),
        ));
    }
    Ok(())
}

fn read_json<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<T, RustSourceProviderError> {
    let bytes = fs::read(path).map_err(|error| {
        provider_error(format!("reading Rust provider action evidence {}: {error}", path.display()))
    })?;
    serde_json::from_slice(&bytes)
        .map_err(|error| provider_error(format!("parsing Rust provider action evidence {}: {error}", path.display())))
}

fn normalized_stage_ids(route: &RustSourceProviderBootstrapPlan) -> Result<BTreeSet<String>, RustSourceProviderError> {
    if route.stages.is_empty() || route.stages.len() > STAGE_COUNT_MAX {
        return Err(provider_error(format!("Rust provider route stage count is outside 1..={STAGE_COUNT_MAX}")));
    }
    let mut ids = BTreeSet::new();
    for stage in &route.stages {
        validate_text("route stage id", &stage.id)?;
        if !ids.insert(stage.id.clone()) {
            return Err(provider_error(format!("duplicate Rust provider action stage: {}", stage.id)));
        }
    }
    Ok(ids)
}

fn fixed_executable_authority(
    context: &FullSourceRustExecutionContext,
) -> Result<Vec<FixedExecutableAuthority>, RustSourceProviderError> {
    let mut by_path = BTreeMap::<PathBuf, FixedExecutableAuthority>::new();
    for tool in &context.host_tools.manifest.tools {
        let path = canonical_file(Path::new(&tool.path), "full-source Rust host tool")?;
        let observed = file_digest(&path)?;
        if observed != tool.content_digest_blake3 {
            return Err(provider_error(format!("Rust provider host-tool digest mismatch for {}", path.display())));
        }
        insert_fixed(
            &mut by_path,
            measured_fixed(
                &path,
                &format!("host-tool:{}:{}", tool.source_id, tool.construction_receipt_digest_blake3),
            )?,
        )?;
    }
    for (relative, role) in crate::full_source_rust_binding::required_full_source_native_artifacts() {
        if !native_role_is_executable(*role) {
            continue;
        }
        let path = canonical_file(&context.native_provider_dir.join(relative), "full-source native executable")?;
        insert_fixed(
            &mut by_path,
            measured_fixed(&path, &format!("native-provider:{}:{role:?}", context.admission.output_digest_blake3))?,
        )?;
    }
    let fixed = by_path.into_values().collect::<Vec<_>>();
    if fixed.is_empty() || fixed.len() > FIXED_EXECUTABLE_COUNT_MAX {
        return Err(provider_error(format!(
            "Rust provider fixed executable count is outside 1..={FIXED_EXECUTABLE_COUNT_MAX}"
        )));
    }
    assert!(fixed.iter().all(|item| Path::new(&item.path).is_absolute()));
    assert!(fixed.iter().all(|item| item.digest_blake3.len() == BLAKE3_HEX_LENGTH));
    Ok(fixed)
}

fn measured_fixed(path: &Path, producer_action_id: &str) -> Result<FixedExecutableAuthority, RustSourceProviderError> {
    let digest_blake3 = file_digest(path)?;
    let path_text = path_text(path)?;
    let material = serde_json::to_vec(&(&path_text, &digest_blake3, producer_action_id))
        .map_err(|error| provider_error(format!("encoding Rust provider fixed authority: {error}")))?;
    let authority_id = format!("fixed:{}", digest_bytes(FIXED_AUTHORITY_CONTEXT, &material));
    Ok(FixedExecutableAuthority {
        authority_id,
        producer_action_id: producer_action_id.to_string(),
        path: path_text,
        digest_blake3,
    })
}

fn insert_fixed(
    fixed: &mut BTreeMap<PathBuf, FixedExecutableAuthority>,
    authority: FixedExecutableAuthority,
) -> Result<(), RustSourceProviderError> {
    let path = PathBuf::from(&authority.path);
    if let Some(existing) = fixed.get(&path) {
        if existing.digest_blake3 != authority.digest_blake3 {
            return Err(provider_error(format!(
                "Rust provider fixed executable path has conflicting bytes: {}",
                path.display()
            )));
        }
        return Ok(());
    }
    fixed.insert(path, authority);
    Ok(())
}

fn native_role_is_executable(role: FullSourceNativeArtifactRole) -> bool {
    matches!(
        role,
        FullSourceNativeArtifactRole::CCompiler
            | FullSourceNativeArtifactRole::CxxCompiler
            | FullSourceNativeArtifactRole::Preprocessor
            | FullSourceNativeArtifactRole::CompilerInternal
            | FullSourceNativeArtifactRole::Assembler
            | FullSourceNativeArtifactRole::Linker
            | FullSourceNativeArtifactRole::ArchiveTool
            | FullSourceNativeArtifactRole::Ranlib
            | FullSourceNativeArtifactRole::SymbolTool
            | FullSourceNativeArtifactRole::ObjectCopy
            | FullSourceNativeArtifactRole::ObjectDump
            | FullSourceNativeArtifactRole::ObjectFormat
    )
}

fn normalized_output_roots(
    stage_id: &str,
    roots: Vec<PathBuf>,
    promotion_count_max: u32,
) -> Result<Vec<PlannedOutputRoot>, RustSourceProviderError> {
    let mut canonical = Vec::with_capacity(roots.len());
    for root in roots {
        if !root.is_absolute()
            || root.components().any(|component| !matches!(component, Component::RootDir | Component::Normal(_)))
        {
            return Err(provider_error(format!(
                "Rust provider stage {stage_id} output root is not absolute and normalized: {}",
                root.display()
            )));
        }
        if canonical.iter().any(|existing: &PlannedOutputRoot| {
            let existing = Path::new(&existing.root);
            root.starts_with(existing) || existing.starts_with(&root)
        }) {
            return Err(provider_error(format!(
                "Rust provider stage {stage_id} output roots overlap at {}",
                root.display()
            )));
        }
        let root_text = path_text(&root)?;
        let output_identity_blake3 =
            digest_bytes(OUTPUT_IDENTITY_CONTEXT, format!("{stage_id}\0{root_text}").as_bytes());
        canonical.push(PlannedOutputRoot {
            root: root_text,
            output_identity_blake3,
            promotion_count_max,
        });
    }
    Ok(canonical)
}

fn stage_reconciliation(
    plan_digest_blake3: &str,
    execution_succeeded: bool,
    events: &[ProtectedSeccompAuditEvent],
    promotions: &[OutputPromotionRecord],
    event_count_max: u32,
) -> Result<RustProviderStageActionReconciliation, RustSourceProviderError> {
    let observed_event_count = bounded_count("Rust provider stage event", events.len())?;
    let denied_event_count = bounded_count(
        "Rust provider denied event",
        events.iter().filter(|event| event.policy_decision != "allowed").count(),
    )?;
    let promotion_count = bounded_count("Rust provider stage promotion", promotions.len())?;
    let mut blockers = Vec::new();
    if !execution_succeeded {
        blockers.push("stage-execution-failed".to_string());
    }
    if observed_event_count == 0 {
        blockers.push("missing-stage-exec-events".to_string());
    }
    if observed_event_count > event_count_max {
        blockers.push("stage-exec-event-bound-exceeded".to_string());
    }
    if denied_event_count > 0 {
        blockers.push("denied-stage-exec-events".to_string());
    }
    let matched_event_count = observed_event_count.saturating_sub(denied_event_count);
    let mut reconciliation = RustProviderStageActionReconciliation {
        schema: STAGE_RECONCILIATION_SCHEMA.to_string(),
        action_plan_digest_blake3: plan_digest_blake3.to_string(),
        execution_succeeded,
        observed_event_count,
        matched_event_count,
        denied_event_count,
        promotion_count,
        local_only: true,
        blockers,
        reconciliation_digest_blake3: String::new(),
    };
    reconciliation.reconciliation_digest_blake3 =
        digest_serialized(STAGE_RECONCILIATION_DIGEST_CONTEXT, &reconciliation)?;
    Ok(reconciliation)
}

fn canonical_file(path: &Path, label: &str) -> Result<PathBuf, RustSourceProviderError> {
    let path = fs::canonicalize(path)
        .map_err(|error| provider_error(format!("canonicalizing {label} {}: {error}", path.display())))?;
    if !path.is_absolute() || !path.is_file() {
        return Err(provider_error(format!("{label} is not an absolute regular file: {}", path.display())));
    }
    Ok(path)
}

fn safe_stage_file(stage_id: &str) -> Result<String, RustSourceProviderError> {
    validate_text("stage id", stage_id)?;
    Ok(stage_id
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || matches!(character, '-' | '_') {
                character
            } else {
                '_'
            }
        })
        .collect())
}

fn file_digest(path: &Path) -> Result<String, RustSourceProviderError> {
    crate::protected_exec::blake3_file_hex(path)
        .map_err(|error| provider_error(format!("hashing Rust provider action input {}: {error}", path.display())))
}

fn path_text(path: &Path) -> Result<String, RustSourceProviderError> {
    path.to_str()
        .map(str::to_string)
        .ok_or_else(|| provider_error(format!("Rust provider action path is not UTF-8: {}", path.display())))
}

fn validate_limits(limits: &RustProviderActionLimits) -> Result<(), RustSourceProviderError> {
    if limits.parallel_jobs_max == 0
        || limits.open_file_descriptors_max == 0
        || limits.storage_bytes_max == 0
        || limits.exec_events_per_stage_max == 0
    {
        return Err(provider_error("Rust provider action limits must be positive".to_string()));
    }
    Ok(())
}

fn validate_text(label: &str, value: &str) -> Result<(), RustSourceProviderError> {
    if value.trim().is_empty() || value.len() > TEXT_BYTES_MAX || value.contains('\0') {
        return Err(provider_error(format!("invalid Rust provider action {label}")));
    }
    Ok(())
}

fn validate_digest(label: &str, digest: &str) -> Result<(), RustSourceProviderError> {
    if digest.len() != BLAKE3_HEX_LENGTH
        || !digest.bytes().all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
    {
        return Err(provider_error(format!("invalid Rust provider action {label} BLAKE3")));
    }
    Ok(())
}

fn bounded_count(label: &str, count: usize) -> Result<u32, RustSourceProviderError> {
    u32::try_from(count).map_err(|_| provider_error(format!("{label} count exceeds u32")))
}

fn digest_serialized<T: Serialize>(context: &[u8], value: &T) -> Result<String, RustSourceProviderError> {
    let bytes = serde_json::to_vec(value)
        .map_err(|error| provider_error(format!("encoding Rust provider action evidence: {error}")))?;
    Ok(digest_bytes(context, &bytes))
}

fn digest_bytes(context: &[u8], bytes: &[u8]) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(context);
    hasher.update(bytes);
    hasher.finalize().to_hex().to_string()
}

fn write_json_create_new<T: Serialize>(path: &Path, value: &T) -> Result<(), RustSourceProviderError> {
    let bytes = serde_json::to_vec_pretty(value)
        .map_err(|error| provider_error(format!("encoding Rust provider action evidence: {error}")))?;
    let parent = path
        .parent()
        .ok_or_else(|| provider_error(format!("Rust provider action path has no parent: {}", path.display())))?;
    fs::create_dir_all(parent).map_err(|error| {
        provider_error(format!("creating Rust provider action parent {}: {error}", parent.display()))
    })?;
    let mut file =
        OpenOptions::new().write(true).create_new(true).open(path).map_err(|error| {
            provider_error(format!("creating Rust provider action file {}: {error}", path.display()))
        })?;
    file.write_all(&bytes)
        .map_err(|error| provider_error(format!("writing Rust provider action file {}: {error}", path.display())))?;
    file.sync_all()
        .map_err(|error| provider_error(format!("syncing Rust provider action file {}: {error}", path.display())))?;
    assert!(!bytes.is_empty());
    assert!(path.is_file());
    Ok(())
}

fn provider_error(message: String) -> RustSourceProviderError {
    RustSourceProviderError::Build(format!("Rust provider action authority blocked: {message}"))
}

#[cfg(test)]
#[path = "source_built_rust_provider_action_tests.rs"]
mod tests;
