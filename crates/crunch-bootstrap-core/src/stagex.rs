use alloc::collections::BTreeMap;
use alloc::collections::BTreeSet;
use alloc::format;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;

use serde::Deserialize;
use serde::Serialize;

use crate::Blake3Hex;

pub const STAGEX_PLAN_SCHEMA_V1: &str = "mantle-stagex-materialization-plan-v1";
pub const STAGEX_RECEIPT_SCHEMA_V1: &str = "mantle-stagex-lineage-provider-receipt-v1";
pub const STAGEX_STAGE_STATUS_COMPLETE: &str = "complete";
pub const STAGEX_RECEIPT_STATUS_COMPLETE: &str = "complete";
pub const STAGEX_SEED_SOURCE_STAGE_ID: &str = "seed";

const STAGEX_STAGE_COUNT_HARD_MAX: u32 = 512;
const STAGEX_EXECUTABLE_COUNT_HARD_MAX: u32 = 4_096;
const STAGEX_STAGE_EDGE_COUNT_HARD_MAX: u32 = 8_192;
const STAGEX_PARALLEL_JOB_COUNT_HARD_MAX: u32 = 64;
const STAGEX_COLLECTION_ITEM_COUNT_HARD_MAX: u32 = 16_384;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StagexMaterializationPlan {
    pub schema_version: String,
    pub lineage_manifest_digest_blake3: String,
    pub source_state_digest_blake3: String,
    pub protected_transition_stage_id: String,
    pub environment_assumption_ids: Vec<String>,
    pub stage_count_max: u32,
    pub stages: Vec<StagexStagePlan>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StagexStagePlan {
    pub id: String,
    pub immediate_predecessor_stage_ids: Vec<String>,
    pub source_artifact_ids: Vec<String>,
    pub input_artifact_ids: Vec<String>,
    pub output_artifact_ids: Vec<String>,
    pub executable_authorizations: Vec<StagexExecutableAuthorization>,
    pub timeout_ms_max: u64,
    pub output_bytes_max: u64,
    pub parallel_job_count_max: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StagexExecutableAuthorization {
    pub id: String,
    pub absolute_path: String,
    pub role: String,
    pub source_stage_id: String,
    pub digest_blake3: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StagexLineageReceipt {
    pub schema_version: String,
    pub lineage_receipt_status: String,
    pub plan_digest_blake3: String,
    pub lineage_manifest_digest_blake3: String,
    pub stage_graph_digest_blake3: String,
    pub source_state_digest_blake3: String,
    pub normalized_provider_digest_blake3: String,
    pub output_digest_blake3: String,
    pub protected_exec_audit_digest_blake3: String,
    pub final_bundle_digest_blake3: String,
    pub fallback_events: Vec<String>,
    pub stage_reports: Vec<StagexStageReport>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StagexStageReport {
    pub stage_id: String,
    pub stage_plan_digest_blake3: String,
    pub predecessor_reports: Vec<StagexPredecessorReport>,
    pub executable_event_ids: Vec<String>,
    pub output_observations: Vec<StagexOutputObservation>,
    pub status: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StagexPredecessorReport {
    pub stage_id: String,
    pub report_digest_blake3: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StagexOutputObservation {
    pub artifact_id: String,
    pub digest_blake3: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StagexValidationResult {
    pub errors: Vec<StagexValidationError>,
}

impl StagexValidationResult {
    pub fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StagexValidationError {
    UnsupportedSchema {
        expected: String,
        actual: String,
    },
    EmptyField(String),
    InvalidBlake3 {
        field: String,
        value: String,
    },
    CollectionLimit {
        field: String,
        actual: u32,
        limit: u32,
    },
    InvalidLimit(String),
    DuplicateId {
        field: String,
        id: String,
    },
    MissingStage(String),
    MissingPredecessor {
        stage_id: String,
        predecessor_id: String,
    },
    CyclicStageGraph,
    UnreachableProtectedStage(String),
    InvalidExecutablePath {
        authorization_id: String,
        path: String,
    },
    InvalidExecutableSource {
        authorization_id: String,
        source_stage_id: String,
    },
    MissingProtectedExecutable(String),
    UndeclaredArtifact {
        stage_id: String,
        artifact_id: String,
    },
    ReceiptStatus(String),
    FallbackEvent(String),
    StageSetMismatch,
    StagePlanDigestMismatch(String),
    PredecessorReportMismatch(String),
    ExecutableEventMismatch(String),
    OutputObservationMismatch(String),
}

impl core::fmt::Display for StagexValidationError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::UnsupportedSchema { expected, actual } => {
                write!(f, "unsupported StageX schema '{actual}'; expected '{expected}'")
            }
            Self::EmptyField(field) => write!(f, "required StageX field is empty: {field}"),
            Self::InvalidBlake3 { field, value } => write!(f, "invalid StageX BLAKE3 for {field}: {value}"),
            Self::CollectionLimit { field, actual, limit } => {
                write!(f, "StageX collection {field} has {actual} entries; limit is {limit}")
            }
            Self::InvalidLimit(field) => write!(f, "StageX limit is zero or exceeds the hard bound: {field}"),
            Self::DuplicateId { field, id } => write!(f, "duplicate StageX {field} id: {id}"),
            Self::MissingStage(stage_id) => write!(f, "missing StageX stage: {stage_id}"),
            Self::MissingPredecessor {
                stage_id,
                predecessor_id,
            } => write!(f, "StageX stage {stage_id} names missing predecessor {predecessor_id}"),
            Self::CyclicStageGraph => write!(f, "StageX stage graph is cyclic"),
            Self::UnreachableProtectedStage(stage_id) => {
                write!(f, "StageX protected stage is not reachable from a root: {stage_id}")
            }
            Self::InvalidExecutablePath { authorization_id, path } => {
                write!(f, "StageX executable authorization {authorization_id} has invalid absolute path {path}")
            }
            Self::InvalidExecutableSource {
                authorization_id,
                source_stage_id,
            } => write!(
                f,
                "StageX executable authorization {authorization_id} has invalid source stage {source_stage_id}"
            ),
            Self::MissingProtectedExecutable(stage_id) => {
                write!(f, "protected StageX stage has no executable authorization: {stage_id}")
            }
            Self::UndeclaredArtifact { stage_id, artifact_id } => {
                write!(f, "StageX stage {stage_id} consumes undeclared artifact {artifact_id}")
            }
            Self::ReceiptStatus(status) => write!(f, "StageX receipt status is not complete: {status}"),
            Self::FallbackEvent(event) => write!(f, "StageX receipt contains fallback event: {event}"),
            Self::StageSetMismatch => write!(f, "StageX receipt stage set does not match the plan"),
            Self::StagePlanDigestMismatch(stage_id) => {
                write!(f, "StageX report plan digest mismatch for stage {stage_id}")
            }
            Self::PredecessorReportMismatch(stage_id) => {
                write!(f, "StageX predecessor report mismatch for stage {stage_id}")
            }
            Self::ExecutableEventMismatch(stage_id) => {
                write!(f, "StageX executable event mismatch for stage {stage_id}")
            }
            Self::OutputObservationMismatch(stage_id) => {
                write!(f, "StageX output observation mismatch for stage {stage_id}")
            }
        }
    }
}

pub fn stagex_plan_digest_blake3(plan: &StagexMaterializationPlan) -> Result<String, StagexValidationError> {
    let bytes =
        serde_json::to_vec(plan).map_err(|_| StagexValidationError::EmptyField("plan serialization".to_string()))?;
    assert!(!bytes.is_empty(), "serialized StageX plan must not be empty");
    Ok(blake3::hash(&bytes).to_hex().to_string())
}

pub fn stagex_stage_plan_digest_blake3(stage: &StagexStagePlan) -> Result<String, StagexValidationError> {
    let bytes = serde_json::to_vec(stage)
        .map_err(|_| StagexValidationError::EmptyField(format!("stage serialization: {}", stage.id)))?;
    assert!(!bytes.is_empty(), "serialized StageX stage must not be empty");
    Ok(blake3::hash(&bytes).to_hex().to_string())
}

pub fn stagex_stage_report_digest_blake3(report: &StagexStageReport) -> Result<String, StagexValidationError> {
    let bytes = serde_json::to_vec(report)
        .map_err(|_| StagexValidationError::EmptyField(format!("stage report serialization: {}", report.stage_id)))?;
    assert!(!bytes.is_empty(), "serialized StageX stage report must not be empty");
    Ok(blake3::hash(&bytes).to_hex().to_string())
}

pub fn validate_stagex_plan(plan: &StagexMaterializationPlan) -> StagexValidationResult {
    let mut errors = Vec::new();
    validate_plan_header(plan, &mut errors);
    validate_plan_stage_limits(plan, &mut errors);
    let stage_ids = validate_stage_ids(plan, &mut errors);
    let output_producers = validate_stage_artifacts(plan, &mut errors);
    validate_stage_predecessors(plan, &stage_ids, &mut errors);
    validate_stage_graph(plan, &stage_ids, &mut errors);
    validate_stage_executables(plan, &stage_ids, &mut errors);
    validate_stage_inputs(plan, &output_producers, &mut errors);
    StagexValidationResult { errors }
}

pub fn validate_stagex_receipt(
    plan: &StagexMaterializationPlan,
    receipt: &StagexLineageReceipt,
) -> StagexValidationResult {
    let mut errors = validate_stagex_plan(plan).errors;
    validate_receipt_header(plan, receipt, &mut errors);
    validate_receipt_fallbacks(receipt, &mut errors);
    let reports = validate_receipt_stage_set(plan, receipt, &mut errors);
    validate_receipt_stage_reports(plan, &reports, &mut errors);
    StagexValidationResult { errors }
}

fn validate_plan_header(plan: &StagexMaterializationPlan, errors: &mut Vec<StagexValidationError>) {
    validate_schema(&plan.schema_version, STAGEX_PLAN_SCHEMA_V1, errors);
    validate_blake3("lineage_manifest_digest_blake3", &plan.lineage_manifest_digest_blake3, errors);
    validate_blake3("source_state_digest_blake3", &plan.source_state_digest_blake3, errors);
    require_non_empty("protected_transition_stage_id", &plan.protected_transition_stage_id, errors);
    validate_unique_non_empty("environment_assumption_ids", &plan.environment_assumption_ids, errors);
    if plan.environment_assumption_ids.is_empty() {
        errors.push(StagexValidationError::EmptyField("environment_assumption_ids".to_string()));
    }
    if plan.stage_count_max == 0 || plan.stage_count_max > STAGEX_STAGE_COUNT_HARD_MAX {
        errors.push(StagexValidationError::InvalidLimit("stage_count_max".to_string()));
    }
    bounded_len("stages", plan.stages.len(), plan.stage_count_max, errors);
    bounded_len(
        "environment_assumption_ids",
        plan.environment_assumption_ids.len(),
        STAGEX_COLLECTION_ITEM_COUNT_HARD_MAX,
        errors,
    );
}

fn validate_plan_stage_limits(plan: &StagexMaterializationPlan, errors: &mut Vec<StagexValidationError>) {
    for stage in &plan.stages {
        require_non_empty("stages[].id", &stage.id, errors);
        if stage.timeout_ms_max == 0 {
            errors.push(StagexValidationError::InvalidLimit(format!("stage {} timeout_ms_max", stage.id)));
        }
        if stage.output_bytes_max == 0 {
            errors.push(StagexValidationError::InvalidLimit(format!("stage {} output_bytes_max", stage.id)));
        }
        if stage.parallel_job_count_max == 0 || stage.parallel_job_count_max > STAGEX_PARALLEL_JOB_COUNT_HARD_MAX {
            errors.push(StagexValidationError::InvalidLimit(format!("stage {} parallel_job_count_max", stage.id)));
        }
        bounded_len(
            &format!("stage {} executable_authorizations", stage.id),
            stage.executable_authorizations.len(),
            STAGEX_EXECUTABLE_COUNT_HARD_MAX,
            errors,
        );
        bounded_stage_collections(stage, errors);
    }
}

fn bounded_stage_collections(stage: &StagexStagePlan, errors: &mut Vec<StagexValidationError>) {
    for (field, len) in [
        ("immediate_predecessor_stage_ids", stage.immediate_predecessor_stage_ids.len()),
        ("source_artifact_ids", stage.source_artifact_ids.len()),
        ("input_artifact_ids", stage.input_artifact_ids.len()),
        ("output_artifact_ids", stage.output_artifact_ids.len()),
    ] {
        bounded_len(&format!("stage {} {field}", stage.id), len, STAGEX_COLLECTION_ITEM_COUNT_HARD_MAX, errors);
    }
    if stage.output_artifact_ids.is_empty() {
        errors.push(StagexValidationError::EmptyField(format!("stage {} output_artifact_ids", stage.id)));
    }
}

fn validate_stage_ids(plan: &StagexMaterializationPlan, errors: &mut Vec<StagexValidationError>) -> BTreeSet<String> {
    let mut stage_ids = BTreeSet::new();
    for stage in &plan.stages {
        if !stage_ids.insert(stage.id.clone()) {
            errors.push(StagexValidationError::DuplicateId {
                field: "stage".to_string(),
                id: stage.id.clone(),
            });
        }
    }
    if !stage_ids.contains(&plan.protected_transition_stage_id) {
        errors.push(StagexValidationError::MissingStage(plan.protected_transition_stage_id.clone()));
    }
    stage_ids
}

fn validate_stage_artifacts(
    plan: &StagexMaterializationPlan,
    errors: &mut Vec<StagexValidationError>,
) -> BTreeMap<String, String> {
    let mut producers = BTreeMap::new();
    for stage in &plan.stages {
        validate_unique_non_empty(
            &format!("stage {} source_artifact_ids", stage.id),
            &stage.source_artifact_ids,
            errors,
        );
        validate_unique_non_empty(
            &format!("stage {} output_artifact_ids", stage.id),
            &stage.output_artifact_ids,
            errors,
        );
        for artifact_id in &stage.output_artifact_ids {
            if producers.insert(artifact_id.clone(), stage.id.clone()).is_some() {
                errors.push(StagexValidationError::DuplicateId {
                    field: "output artifact".to_string(),
                    id: artifact_id.clone(),
                });
            }
        }
    }
    producers
}

fn validate_stage_predecessors(
    plan: &StagexMaterializationPlan,
    stage_ids: &BTreeSet<String>,
    errors: &mut Vec<StagexValidationError>,
) {
    let mut edge_count: u32 = 0;
    for stage in &plan.stages {
        validate_unique_non_empty(
            &format!("stage {} immediate_predecessor_stage_ids", stage.id),
            &stage.immediate_predecessor_stage_ids,
            errors,
        );
        for predecessor in &stage.immediate_predecessor_stage_ids {
            edge_count = edge_count.saturating_add(1);
            if predecessor == &stage.id || !stage_ids.contains(predecessor) {
                errors.push(StagexValidationError::MissingPredecessor {
                    stage_id: stage.id.clone(),
                    predecessor_id: predecessor.clone(),
                });
            }
        }
    }
    if edge_count > STAGEX_STAGE_EDGE_COUNT_HARD_MAX {
        errors.push(StagexValidationError::CollectionLimit {
            field: "stage edges".to_string(),
            actual: edge_count,
            limit: STAGEX_STAGE_EDGE_COUNT_HARD_MAX,
        });
    }
}

fn validate_stage_graph(
    plan: &StagexMaterializationPlan,
    stage_ids: &BTreeSet<String>,
    errors: &mut Vec<StagexValidationError>,
) {
    let mut reached = BTreeSet::new();
    let mut remaining = stage_ids.clone();
    let mut progress = true;
    while progress && !remaining.is_empty() {
        progress = false;
        for stage in &plan.stages {
            if !remaining.contains(&stage.id) {
                continue;
            }
            if stage.immediate_predecessor_stage_ids.iter().all(|id| reached.contains(id)) {
                reached.insert(stage.id.clone());
                remaining.remove(&stage.id);
                progress = true;
            }
        }
    }
    if !remaining.is_empty() {
        errors.push(StagexValidationError::CyclicStageGraph);
    }
    if !reached.contains(&plan.protected_transition_stage_id) {
        errors.push(StagexValidationError::UnreachableProtectedStage(plan.protected_transition_stage_id.clone()));
    }
}

fn validate_stage_executables(
    plan: &StagexMaterializationPlan,
    stage_ids: &BTreeSet<String>,
    errors: &mut Vec<StagexValidationError>,
) {
    let protected_stage_ids = protected_stage_ids(plan);
    let ancestors_by_stage = stage_ancestors(plan);
    let mut authorization_ids = BTreeSet::new();
    for stage in &plan.stages {
        if protected_stage_ids.contains(&stage.id) && stage.executable_authorizations.is_empty() {
            errors.push(StagexValidationError::MissingProtectedExecutable(stage.id.clone()));
        }
        let ancestors = ancestors_by_stage.get(&stage.id).cloned().unwrap_or_default();
        for authorization in &stage.executable_authorizations {
            validate_authorization(stage, authorization, stage_ids, &ancestors, &mut authorization_ids, errors);
        }
    }
}

fn validate_authorization(
    stage: &StagexStagePlan,
    authorization: &StagexExecutableAuthorization,
    stage_ids: &BTreeSet<String>,
    ancestor_stage_ids: &BTreeSet<String>,
    authorization_ids: &mut BTreeSet<String>,
    errors: &mut Vec<StagexValidationError>,
) {
    require_non_empty("executable_authorizations[].id", &authorization.id, errors);
    require_non_empty("executable_authorizations[].role", &authorization.role, errors);
    if !authorization_ids.insert(authorization.id.clone()) {
        errors.push(StagexValidationError::DuplicateId {
            field: "executable authorization".to_string(),
            id: authorization.id.clone(),
        });
    }
    if !is_safe_absolute_path(&authorization.absolute_path) {
        errors.push(StagexValidationError::InvalidExecutablePath {
            authorization_id: authorization.id.clone(),
            path: authorization.absolute_path.clone(),
        });
    }
    validate_blake3(
        &format!("stage {} authorization {} digest_blake3", stage.id, authorization.id),
        &authorization.digest_blake3,
        errors,
    );
    let source_is_seed = authorization.source_stage_id == STAGEX_SEED_SOURCE_STAGE_ID;
    let source_is_known = stage_ids.contains(&authorization.source_stage_id);
    let source_precedes_stage = ancestor_stage_ids.contains(&authorization.source_stage_id);
    if !source_is_seed && (!source_is_known || !source_precedes_stage) {
        errors.push(StagexValidationError::InvalidExecutableSource {
            authorization_id: authorization.id.clone(),
            source_stage_id: authorization.source_stage_id.clone(),
        });
    }
}

fn stage_ancestors(plan: &StagexMaterializationPlan) -> BTreeMap<String, BTreeSet<String>> {
    let mut ancestors: BTreeMap<String, BTreeSet<String>> =
        plan.stages.iter().map(|stage| (stage.id.clone(), BTreeSet::new())).collect();
    let mut progress = true;
    while progress {
        progress = false;
        for stage in &plan.stages {
            let mut expanded = ancestors.get(&stage.id).cloned().unwrap_or_default();
            for predecessor in &stage.immediate_predecessor_stage_ids {
                expanded.insert(predecessor.clone());
                if let Some(transitive) = ancestors.get(predecessor) {
                    expanded.extend(transitive.iter().cloned());
                }
            }
            let current = ancestors.entry(stage.id.clone()).or_default();
            if *current != expanded {
                *current = expanded;
                progress = true;
            }
        }
    }
    ancestors
}

fn validate_stage_inputs(
    plan: &StagexMaterializationPlan,
    output_producers: &BTreeMap<String, String>,
    errors: &mut Vec<StagexValidationError>,
) {
    for stage in &plan.stages {
        validate_unique_non_empty(&format!("stage {} input_artifact_ids", stage.id), &stage.input_artifact_ids, errors);
        for artifact_id in &stage.input_artifact_ids {
            let Some(producer) = output_producers.get(artifact_id) else {
                errors.push(StagexValidationError::UndeclaredArtifact {
                    stage_id: stage.id.clone(),
                    artifact_id: artifact_id.clone(),
                });
                continue;
            };
            if !stage.immediate_predecessor_stage_ids.contains(producer) {
                errors.push(StagexValidationError::UndeclaredArtifact {
                    stage_id: stage.id.clone(),
                    artifact_id: artifact_id.clone(),
                });
            }
        }
    }
}

fn protected_stage_ids(plan: &StagexMaterializationPlan) -> BTreeSet<String> {
    let mut protected = BTreeSet::new();
    protected.insert(plan.protected_transition_stage_id.clone());
    let mut progress = true;
    while progress {
        progress = false;
        for stage in &plan.stages {
            if protected.contains(&stage.id) {
                continue;
            }
            if stage.immediate_predecessor_stage_ids.iter().any(|id| protected.contains(id)) {
                protected.insert(stage.id.clone());
                progress = true;
            }
        }
    }
    protected
}

fn validate_receipt_header(
    plan: &StagexMaterializationPlan,
    receipt: &StagexLineageReceipt,
    errors: &mut Vec<StagexValidationError>,
) {
    validate_schema(&receipt.schema_version, STAGEX_RECEIPT_SCHEMA_V1, errors);
    if receipt.lineage_receipt_status != STAGEX_RECEIPT_STATUS_COMPLETE {
        errors.push(StagexValidationError::ReceiptStatus(receipt.lineage_receipt_status.clone()));
    }
    let expected_plan_digest = stagex_plan_digest_blake3(plan).unwrap_or_default();
    if receipt.plan_digest_blake3 != expected_plan_digest {
        errors.push(StagexValidationError::InvalidBlake3 {
            field: "plan_digest_blake3".to_string(),
            value: receipt.plan_digest_blake3.clone(),
        });
    }
    if receipt.lineage_manifest_digest_blake3 != plan.lineage_manifest_digest_blake3 {
        errors.push(StagexValidationError::InvalidBlake3 {
            field: "lineage_manifest_digest_blake3".to_string(),
            value: receipt.lineage_manifest_digest_blake3.clone(),
        });
    }
    if receipt.source_state_digest_blake3 != plan.source_state_digest_blake3 {
        errors.push(StagexValidationError::InvalidBlake3 {
            field: "source_state_digest_blake3".to_string(),
            value: receipt.source_state_digest_blake3.clone(),
        });
    }
    for (field, value) in receipt_digest_fields(receipt) {
        validate_blake3(field, value, errors);
    }
}

fn receipt_digest_fields(receipt: &StagexLineageReceipt) -> [(&str, &str); 8] {
    [
        ("plan_digest_blake3", &receipt.plan_digest_blake3),
        ("lineage_manifest_digest_blake3", &receipt.lineage_manifest_digest_blake3),
        ("stage_graph_digest_blake3", &receipt.stage_graph_digest_blake3),
        ("source_state_digest_blake3", &receipt.source_state_digest_blake3),
        ("normalized_provider_digest_blake3", &receipt.normalized_provider_digest_blake3),
        ("output_digest_blake3", &receipt.output_digest_blake3),
        ("protected_exec_audit_digest_blake3", &receipt.protected_exec_audit_digest_blake3),
        ("final_bundle_digest_blake3", &receipt.final_bundle_digest_blake3),
    ]
}

fn validate_receipt_fallbacks(receipt: &StagexLineageReceipt, errors: &mut Vec<StagexValidationError>) {
    for event in &receipt.fallback_events {
        errors.push(StagexValidationError::FallbackEvent(event.clone()));
    }
}

fn validate_receipt_stage_set<'a>(
    plan: &StagexMaterializationPlan,
    receipt: &'a StagexLineageReceipt,
    errors: &mut Vec<StagexValidationError>,
) -> BTreeMap<String, &'a StagexStageReport> {
    let planned_ids: BTreeSet<&str> = plan.stages.iter().map(|stage| stage.id.as_str()).collect();
    let mut reports = BTreeMap::new();
    for report in &receipt.stage_reports {
        if reports.insert(report.stage_id.clone(), report).is_some() {
            errors.push(StagexValidationError::DuplicateId {
                field: "stage report".to_string(),
                id: report.stage_id.clone(),
            });
        }
    }
    let reported_ids: BTreeSet<&str> = reports.keys().map(String::as_str).collect();
    if planned_ids != reported_ids {
        errors.push(StagexValidationError::StageSetMismatch);
    }
    reports
}

fn validate_receipt_stage_reports(
    plan: &StagexMaterializationPlan,
    reports: &BTreeMap<String, &StagexStageReport>,
    errors: &mut Vec<StagexValidationError>,
) {
    for stage in &plan.stages {
        let Some(report) = reports.get(&stage.id) else {
            continue;
        };
        if report.status != STAGEX_STAGE_STATUS_COMPLETE {
            errors.push(StagexValidationError::ReceiptStatus(format!("stage {}: {}", stage.id, report.status)));
        }
        validate_report_plan_digest(stage, report, errors);
        validate_report_predecessors(stage, reports, report, errors);
        validate_report_events(stage, report, errors);
        validate_report_outputs(stage, report, errors);
    }
}

fn validate_report_plan_digest(
    stage: &StagexStagePlan,
    report: &StagexStageReport,
    errors: &mut Vec<StagexValidationError>,
) {
    let expected = stagex_stage_plan_digest_blake3(stage).unwrap_or_default();
    if report.stage_plan_digest_blake3 != expected {
        errors.push(StagexValidationError::StagePlanDigestMismatch(stage.id.clone()));
    }
}

fn validate_report_predecessors(
    stage: &StagexStagePlan,
    reports: &BTreeMap<String, &StagexStageReport>,
    report: &StagexStageReport,
    errors: &mut Vec<StagexValidationError>,
) {
    let expected_ids: BTreeSet<&str> = stage.immediate_predecessor_stage_ids.iter().map(String::as_str).collect();
    let observed_ids: BTreeSet<&str> = report.predecessor_reports.iter().map(|item| item.stage_id.as_str()).collect();
    if expected_ids != observed_ids {
        errors.push(StagexValidationError::PredecessorReportMismatch(stage.id.clone()));
        return;
    }
    for predecessor in &report.predecessor_reports {
        let Some(predecessor_report) = reports.get(&predecessor.stage_id) else {
            errors.push(StagexValidationError::PredecessorReportMismatch(stage.id.clone()));
            continue;
        };
        let expected_digest = stagex_stage_report_digest_blake3(predecessor_report).unwrap_or_default();
        if predecessor.report_digest_blake3 != expected_digest {
            errors.push(StagexValidationError::PredecessorReportMismatch(stage.id.clone()));
        }
    }
}

fn validate_report_events(
    stage: &StagexStagePlan,
    report: &StagexStageReport,
    errors: &mut Vec<StagexValidationError>,
) {
    let expected: BTreeSet<&str> = stage.executable_authorizations.iter().map(|item| item.id.as_str()).collect();
    let observed: BTreeSet<&str> = report.executable_event_ids.iter().map(String::as_str).collect();
    if expected != observed {
        errors.push(StagexValidationError::ExecutableEventMismatch(stage.id.clone()));
    }
}

fn validate_report_outputs(
    stage: &StagexStagePlan,
    report: &StagexStageReport,
    errors: &mut Vec<StagexValidationError>,
) {
    let expected: BTreeSet<&str> = stage.output_artifact_ids.iter().map(String::as_str).collect();
    let observed: BTreeSet<&str> = report.output_observations.iter().map(|item| item.artifact_id.as_str()).collect();
    if expected != observed {
        errors.push(StagexValidationError::OutputObservationMismatch(stage.id.clone()));
    }
    for output in &report.output_observations {
        validate_blake3(&format!("stage {} output {}", stage.id, output.artifact_id), &output.digest_blake3, errors);
    }
}

fn validate_schema(actual: &str, expected: &str, errors: &mut Vec<StagexValidationError>) {
    if actual != expected {
        errors.push(StagexValidationError::UnsupportedSchema {
            expected: expected.to_string(),
            actual: actual.to_string(),
        });
    }
}

fn validate_blake3(field: &str, value: &str, errors: &mut Vec<StagexValidationError>) {
    let digest = Blake3Hex::new(value.to_string());
    if !digest.is_valid_format() {
        errors.push(StagexValidationError::InvalidBlake3 {
            field: field.to_string(),
            value: value.to_string(),
        });
    }
}

fn validate_unique_non_empty(field: &str, values: &[String], errors: &mut Vec<StagexValidationError>) {
    let mut seen = BTreeSet::new();
    for value in values {
        require_non_empty(field, value, errors);
        if !seen.insert(value.as_str()) {
            errors.push(StagexValidationError::DuplicateId {
                field: field.to_string(),
                id: value.clone(),
            });
        }
    }
}

fn require_non_empty(field: &str, value: &str, errors: &mut Vec<StagexValidationError>) {
    if value.trim().is_empty() {
        errors.push(StagexValidationError::EmptyField(field.to_string()));
    }
}

fn bounded_len(field: &str, len: usize, limit: u32, errors: &mut Vec<StagexValidationError>) {
    let actual = u32::try_from(len).unwrap_or(u32::MAX);
    if actual > limit {
        errors.push(StagexValidationError::CollectionLimit {
            field: field.to_string(),
            actual,
            limit,
        });
    }
}

fn is_safe_absolute_path(path: &str) -> bool {
    path.starts_with('/') && !path.contains("/../") && !path.ends_with("/..") && !path.contains('\0')
}

#[cfg(test)]
mod tests {
    use alloc::vec;

    use super::*;
    use crate::BLAKE3_HEX_LENGTH;

    const DIGEST_A: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    const DIGEST_B: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
    const DIGEST_C: &str = "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc";
    const STAGE_COUNT_MAX: u32 = 8;
    const STAGE_TIMEOUT_MS_MAX: u64 = 30_000;
    const STAGE_OUTPUT_BYTES_MAX: u64 = 1_048_576;
    const STAGE_PARALLEL_JOB_COUNT_MAX: u32 = 1;

    fn authorization(id: &str, path: &str, source_stage_id: &str, digest: &str) -> StagexExecutableAuthorization {
        StagexExecutableAuthorization {
            id: id.to_string(),
            absolute_path: path.to_string(),
            role: "assembler".to_string(),
            source_stage_id: source_stage_id.to_string(),
            digest_blake3: digest.to_string(),
        }
    }

    fn stage(
        id: &str,
        predecessors: Vec<&str>,
        inputs: Vec<&str>,
        outputs: Vec<&str>,
        authorizations: Vec<StagexExecutableAuthorization>,
    ) -> StagexStagePlan {
        StagexStagePlan {
            id: id.to_string(),
            immediate_predecessor_stage_ids: predecessors.into_iter().map(str::to_string).collect(),
            source_artifact_ids: vec![format!("source:{id}")],
            input_artifact_ids: inputs.into_iter().map(str::to_string).collect(),
            output_artifact_ids: outputs.into_iter().map(str::to_string).collect(),
            executable_authorizations: authorizations,
            timeout_ms_max: STAGE_TIMEOUT_MS_MAX,
            output_bytes_max: STAGE_OUTPUT_BYTES_MAX,
            parallel_job_count_max: STAGE_PARALLEL_JOB_COUNT_MAX,
        }
    }

    fn valid_plan() -> StagexMaterializationPlan {
        StagexMaterializationPlan {
            schema_version: STAGEX_PLAN_SCHEMA_V1.to_string(),
            lineage_manifest_digest_blake3: DIGEST_A.to_string(),
            source_state_digest_blake3: DIGEST_B.to_string(),
            protected_transition_stage_id: "hex0-reproduction".to_string(),
            environment_assumption_ids: vec!["linux-kernel".to_string(), "mantle-orchestrator".to_string()],
            stage_count_max: STAGE_COUNT_MAX,
            stages: vec![
                stage("hex0-reproduction", vec![], vec![], vec!["hex0-reproduced"], vec![authorization(
                    "exec:hex0-seed",
                    "/stage/hex0-seed",
                    STAGEX_SEED_SOURCE_STAGE_ID,
                    DIGEST_A,
                )]),
                stage("kaem-transition", vec!["hex0-reproduction"], vec!["hex0-reproduced"], vec!["kaem-0"], vec![
                    authorization("exec:hex0", "/stage/hex0", "hex0-reproduction", DIGEST_A),
                ]),
            ],
        }
    }

    fn report_for(stage: &StagexStagePlan, prior: &[StagexStageReport]) -> StagexStageReport {
        let prior_by_id: BTreeMap<&str, &StagexStageReport> =
            prior.iter().map(|report| (report.stage_id.as_str(), report)).collect();
        StagexStageReport {
            stage_id: stage.id.clone(),
            stage_plan_digest_blake3: stagex_stage_plan_digest_blake3(stage).unwrap(),
            predecessor_reports: stage
                .immediate_predecessor_stage_ids
                .iter()
                .map(|stage_id| StagexPredecessorReport {
                    stage_id: stage_id.clone(),
                    report_digest_blake3: stagex_stage_report_digest_blake3(prior_by_id[stage_id.as_str()]).unwrap(),
                })
                .collect(),
            executable_event_ids: stage.executable_authorizations.iter().map(|item| item.id.clone()).collect(),
            output_observations: stage
                .output_artifact_ids
                .iter()
                .map(|artifact_id| StagexOutputObservation {
                    artifact_id: artifact_id.clone(),
                    digest_blake3: DIGEST_C.to_string(),
                })
                .collect(),
            status: STAGEX_STAGE_STATUS_COMPLETE.to_string(),
        }
    }

    fn valid_receipt(plan: &StagexMaterializationPlan) -> StagexLineageReceipt {
        let mut reports = Vec::new();
        for stage in &plan.stages {
            reports.push(report_for(stage, &reports));
        }
        StagexLineageReceipt {
            schema_version: STAGEX_RECEIPT_SCHEMA_V1.to_string(),
            lineage_receipt_status: STAGEX_RECEIPT_STATUS_COMPLETE.to_string(),
            plan_digest_blake3: stagex_plan_digest_blake3(plan).unwrap(),
            lineage_manifest_digest_blake3: plan.lineage_manifest_digest_blake3.clone(),
            stage_graph_digest_blake3: DIGEST_C.to_string(),
            source_state_digest_blake3: plan.source_state_digest_blake3.clone(),
            normalized_provider_digest_blake3: DIGEST_A.to_string(),
            output_digest_blake3: DIGEST_B.to_string(),
            protected_exec_audit_digest_blake3: DIGEST_C.to_string(),
            final_bundle_digest_blake3: DIGEST_A.to_string(),
            fallback_events: Vec::new(),
            stage_reports: reports,
        }
    }

    #[test]
    fn valid_plan_and_complete_receipt_pass() {
        let plan = valid_plan();
        let receipt = valid_receipt(&plan);
        assert!(validate_stagex_plan(&plan).is_valid());
        assert!(validate_stagex_receipt(&plan, &receipt).is_valid());
    }

    #[test]
    fn cyclic_or_missing_predecessor_fails() {
        let mut plan = valid_plan();
        plan.stages[0].immediate_predecessor_stage_ids = vec!["kaem-transition".to_string()];
        let result = validate_stagex_plan(&plan);
        assert!(!result.is_valid());
        assert!(result.errors.contains(&StagexValidationError::CyclicStageGraph));
    }

    #[test]
    fn relative_or_substituted_executable_fails() {
        let mut plan = valid_plan();
        plan.stages[0].executable_authorizations[0].absolute_path = "./hex0-seed".to_string();
        plan.stages[1].executable_authorizations[0].source_stage_id = "host-tools".to_string();
        let result = validate_stagex_plan(&plan);
        assert!(!result.is_valid());
        assert!(
            result
                .errors
                .iter()
                .any(|error| matches!(error, StagexValidationError::InvalidExecutablePath { .. }))
        );
        assert!(
            result
                .errors
                .iter()
                .any(|error| matches!(error, StagexValidationError::InvalidExecutableSource { .. }))
        );
    }

    #[test]
    fn future_or_same_stage_executable_source_fails() {
        let mut plan = valid_plan();
        plan.stages[0].executable_authorizations[0].source_stage_id = "kaem-transition".to_string();
        plan.stages[1].executable_authorizations[0].source_stage_id = "kaem-transition".to_string();
        let result = validate_stagex_plan(&plan);
        let invalid_source_count = result
            .errors
            .iter()
            .filter(|error| matches!(error, StagexValidationError::InvalidExecutableSource { .. }))
            .count();
        assert!(!result.is_valid());
        assert_eq!(invalid_source_count, 2);
    }

    #[test]
    fn zero_limits_and_missing_protected_exec_fail() {
        let mut plan = valid_plan();
        plan.stages[0].timeout_ms_max = 0;
        plan.stages[0].executable_authorizations.clear();
        let result = validate_stagex_plan(&plan);
        assert!(!result.is_valid());
        assert!(result.errors.iter().any(|error| matches!(error, StagexValidationError::InvalidLimit(_))));
        assert!(
            result
                .errors
                .contains(&StagexValidationError::MissingProtectedExecutable("hex0-reproduction".to_string()))
        );
    }

    #[test]
    fn fallback_or_scaffold_receipt_fails() {
        let plan = valid_plan();
        let mut receipt = valid_receipt(&plan);
        receipt.lineage_receipt_status = "scaffold-only".to_string();
        receipt.fallback_events.push("host-shell:/bin/sh".to_string());
        let result = validate_stagex_receipt(&plan, &receipt);
        assert!(!result.is_valid());
        assert!(result.errors.iter().any(|error| matches!(error, StagexValidationError::ReceiptStatus(_))));
        assert!(result.errors.iter().any(|error| matches!(error, StagexValidationError::FallbackEvent(_))));
    }

    #[test]
    fn stale_stage_digest_or_missing_observation_fails() {
        let plan = valid_plan();
        let mut receipt = valid_receipt(&plan);
        receipt.stage_reports[0].stage_plan_digest_blake3 = DIGEST_B.to_string();
        receipt.stage_reports[1].output_observations.clear();
        let result = validate_stagex_receipt(&plan, &receipt);
        assert!(!result.is_valid());
        assert!(result.errors.iter().any(|error| matches!(error, StagexValidationError::StagePlanDigestMismatch(_))));
        assert!(
            result
                .errors
                .iter()
                .any(|error| matches!(error, StagexValidationError::OutputObservationMismatch(_)))
        );
    }

    #[test]
    fn malformed_digest_and_undeclared_input_fail() {
        let mut plan = valid_plan();
        plan.lineage_manifest_digest_blake3 = "not-a-digest".to_string();
        plan.stages[1].input_artifact_ids = vec!["host-tool-output".to_string()];
        let result = validate_stagex_plan(&plan);
        assert!(!result.is_valid());
        assert!(result.errors.iter().any(|error| matches!(error, StagexValidationError::InvalidBlake3 { .. })));
        assert!(result.errors.iter().any(|error| matches!(error, StagexValidationError::UndeclaredArtifact { .. })));
    }

    #[test]
    fn stage_digest_is_deterministic_and_sensitive() {
        let plan = valid_plan();
        let first = stagex_stage_plan_digest_blake3(&plan.stages[0]).unwrap();
        let second = stagex_stage_plan_digest_blake3(&plan.stages[0]).unwrap();
        let mut changed = plan.stages[0].clone();
        changed.output_bytes_max = changed.output_bytes_max.saturating_add(1);
        let changed_digest = stagex_stage_plan_digest_blake3(&changed).unwrap();
        assert_eq!(first, second);
        assert_ne!(first, changed_digest);
        assert_eq!(first.len(), BLAKE3_HEX_LENGTH);
    }
}
