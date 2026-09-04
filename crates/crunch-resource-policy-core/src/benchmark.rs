use alloc::collections::BTreeMap;
use alloc::collections::BTreeSet;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;

use crate::BENCHMARK_DOMAIN;
use crate::MAX_BENCHMARK_WORKLOADS;
use crate::MAX_CHARGE_UNITS;
use crate::MAX_FAULT_CASES;
use crate::MAX_MEASUREMENT_BYTES;
use crate::MAX_MEASUREMENT_MILLISECONDS;
use crate::RESOURCE_BENCHMARK_REPORT_SCHEMA;
use crate::RESOURCE_FAULT_CAMPAIGN_SCHEMA;
use crate::RESOURCE_POLICY_NON_CLAIM;
use crate::ResourceBenchmarkObservation;
use crate::ResourceBenchmarkReport;
use crate::ResourceBenchmarkWorkload;
use crate::ResourceFaultCampaign;
use crate::ResourceFaultKind;
use crate::ResourcePolicyError;
use crate::bounded_count;
use crate::canonical_blake3;
use crate::canonical_strings;
use crate::validate_digest;
use crate::validate_id;

const FAULT_RANK_POSITIVE_OOM: u8 = 0;
const FAULT_RANK_AMBIGUOUS_FAILURE: u8 = 1;
const FAULT_RANK_WORKER_LOSS: u8 = 2;
const FAULT_RANK_DUPLICATE_COMPLETION: u8 = 3;
const FAULT_RANK_ACCOUNTING_INTERRUPTION: u8 = 4;
const FAULT_RANK_CAS_UNAVAILABLE: u8 = 5;

pub fn build_resource_benchmark_report(
    workloads: Vec<ResourceBenchmarkWorkload>,
    observations: Vec<ResourceBenchmarkObservation>,
    baseline_policy_id: String,
    candidate_policy_id: String,
) -> Result<ResourceBenchmarkReport, ResourcePolicyError> {
    validate_id(&baseline_policy_id)?;
    validate_id(&candidate_policy_id)?;
    let workloads = canonical_workloads(workloads)?;
    let observations = canonical_observations(observations, &workloads)?;
    let workload_count = u32::try_from(workloads.len()).map_err(|_| ResourcePolicyError::TooManyBenchmarkWorkloads)?;
    let bounds = workloads
        .iter()
        .map(|workload| (workload.workload_id.as_str(), workload))
        .collect::<BTreeMap<_, _>>();
    let is_correctness_passed = observations.iter().all(|observation| {
        let workload = bounds[observation.workload_id.as_str()];
        observation.correct_output && observation.completion_class == workload.expected_completion_class
    });
    let is_compatibility_passed = observations.iter().all(|observation| {
        let workload = bounds[observation.workload_id.as_str()];
        observation.compatible
            && observation.latency_ms <= workload.latency_ms_max
            && observation.memory_bytes <= workload.memory_bytes_max
            && observation.transfer_bytes <= workload.transfer_bytes_max
            && observation.usage_units <= workload.usage_units_max
    });
    let mut benchmark_evidence = ResourceBenchmarkReport {
        schema: RESOURCE_BENCHMARK_REPORT_SCHEMA.to_string(),
        report_blake3: String::new(),
        baseline_policy_id,
        candidate_policy_id,
        workload_count,
        correctness_passed: is_correctness_passed,
        compatibility_passed: is_compatibility_passed,
        observations,
        non_claim: RESOURCE_POLICY_NON_CLAIM.to_string(),
    };
    benchmark_evidence.report_blake3 = benchmark_report_identity(&benchmark_evidence)?;
    debug_assert_eq!(usize::try_from(benchmark_evidence.workload_count).ok(), Some(workloads.len()));
    debug_assert!(crate::valid_blake3(&benchmark_evidence.report_blake3));
    Ok(benchmark_evidence)
}

pub fn validate_resource_benchmark_report(
    report: ResourceBenchmarkReport,
) -> Result<ResourceBenchmarkReport, ResourcePolicyError> {
    if report.schema != RESOURCE_BENCHMARK_REPORT_SCHEMA || report.non_claim != RESOURCE_POLICY_NON_CLAIM {
        return Err(ResourcePolicyError::InvalidSchema);
    }
    validate_digest(&report.report_blake3)?;
    validate_id(&report.baseline_policy_id)?;
    validate_id(&report.candidate_policy_id)?;
    bounded_count(report.observations.len(), MAX_BENCHMARK_WORKLOADS, ResourcePolicyError::TooManyBenchmarkWorkloads)?;
    if usize::try_from(report.workload_count).ok() != Some(report.observations.len())
        || benchmark_report_identity(&report)? != report.report_blake3
    {
        return Err(ResourcePolicyError::BenchmarkRejected);
    }
    debug_assert_eq!(usize::try_from(report.workload_count).ok(), Some(report.observations.len()));
    debug_assert!(crate::valid_blake3(&report.report_blake3));
    Ok(report)
}

pub fn validate_resource_fault_campaign(
    mut campaign: ResourceFaultCampaign,
) -> Result<ResourceFaultCampaign, ResourcePolicyError> {
    if campaign.schema != RESOURCE_FAULT_CAMPAIGN_SCHEMA || campaign.non_claim != RESOURCE_POLICY_NON_CLAIM {
        return Err(ResourcePolicyError::InvalidSchema);
    }
    validate_id(&campaign.chaoscontrol_revision)?;
    validate_id(&campaign.chaoscontrol_contract_path)?;
    validate_digest(&campaign.chaoscontrol_contract_blake3)?;
    bounded_count(campaign.cases.len(), MAX_FAULT_CASES, ResourcePolicyError::TooManyFaultCases)?;
    campaign.cases.sort_by(|left, right| left.case_id.cmp(&right.case_id));
    let mut ids = BTreeSet::new();
    let mut kinds = BTreeSet::new();
    for case in &campaign.cases {
        validate_id(&case.case_id)?;
        validate_id(&case.expected_reason_code)?;
        if !ids.insert(case.case_id.as_str()) || !kinds.insert(fault_rank(case.kind)) {
            return Err(ResourcePolicyError::DuplicateIdentity);
        }
    }
    let required = [
        ResourceFaultKind::PositiveOom,
        ResourceFaultKind::AmbiguousFailure,
        ResourceFaultKind::WorkerLoss,
        ResourceFaultKind::DuplicateCompletion,
        ResourceFaultKind::AccountingInterruption,
        ResourceFaultKind::CasUnavailable,
    ];
    if required.iter().any(|kind| !campaign.cases.iter().any(|case| case.kind == *kind)) {
        return Err(ResourcePolicyError::BenchmarkRejected);
    }
    debug_assert!(campaign.cases.len() >= required.len());
    debug_assert_eq!(ids.len(), campaign.cases.len());
    Ok(campaign)
}

fn canonical_workloads(
    mut workloads: Vec<ResourceBenchmarkWorkload>,
) -> Result<Vec<ResourceBenchmarkWorkload>, ResourcePolicyError> {
    bounded_count(workloads.len(), MAX_BENCHMARK_WORKLOADS, ResourcePolicyError::TooManyBenchmarkWorkloads)?;
    if workloads.is_empty() {
        return Err(ResourcePolicyError::BenchmarkRejected);
    }
    for workload in &mut workloads {
        validate_workload(workload)?;
        workload.platform.required_features =
            canonical_strings(workload.platform.required_features.clone(), crate::MAX_FEATURES)?;
    }
    workloads.sort_by(|left, right| left.workload_id.cmp(&right.workload_id));
    if workloads.windows(crate::PAIR_WINDOW_SIZE).any(|pair| pair[0].workload_id == pair[1].workload_id) {
        return Err(ResourcePolicyError::DuplicateIdentity);
    }
    debug_assert!(!workloads.is_empty());
    debug_assert!(workloads.windows(crate::PAIR_WINDOW_SIZE).all(|pair| pair[0].workload_id < pair[1].workload_id));
    Ok(workloads)
}

fn validate_workload(workload: &ResourceBenchmarkWorkload) -> Result<(), ResourcePolicyError> {
    validate_id(&workload.workload_id)?;
    validate_id(&workload.source.repository)?;
    validate_id(&workload.source.revision)?;
    validate_digest(&workload.source.source_blake3)?;
    validate_id(&workload.source.license)?;
    validate_digest(&workload.source.license_blake3)?;
    validate_id(&workload.source.adaptation)?;
    validate_digest(&workload.fixed_input_blake3)?;
    validate_id(&workload.platform.architecture)?;
    validate_id(&workload.platform.platform)?;
    validate_id(&workload.platform.trust_tier)?;
    validate_id(&workload.platform.isolation)?;
    validate_id(&workload.expected_completion_class)?;
    if workload.latency_ms_max == 0 || workload.latency_ms_max > MAX_MEASUREMENT_MILLISECONDS {
        return Err(ResourcePolicyError::InvalidBounds);
    }
    if workload.memory_bytes_max == 0 || workload.memory_bytes_max > MAX_MEASUREMENT_BYTES {
        return Err(ResourcePolicyError::InvalidBounds);
    }
    if workload.transfer_bytes_max > MAX_MEASUREMENT_BYTES {
        return Err(ResourcePolicyError::InvalidBounds);
    }
    if workload.usage_units_max == 0 || workload.usage_units_max > MAX_CHARGE_UNITS {
        return Err(ResourcePolicyError::InvalidBounds);
    }
    debug_assert!(workload.latency_ms_max <= MAX_MEASUREMENT_MILLISECONDS);
    debug_assert!(workload.memory_bytes_max <= MAX_MEASUREMENT_BYTES);
    Ok(())
}

fn canonical_observations(
    mut observations: Vec<ResourceBenchmarkObservation>,
    workloads: &[ResourceBenchmarkWorkload],
) -> Result<Vec<ResourceBenchmarkObservation>, ResourcePolicyError> {
    if observations.len() != workloads.len() {
        return Err(ResourcePolicyError::BenchmarkRejected);
    }
    for observation in &observations {
        validate_id(&observation.workload_id)?;
        validate_id(&observation.static_class_id)?;
        validate_id(&observation.policy_class_id)?;
        validate_id(&observation.completion_class)?;
        if observation.throughput_units == 0 {
            return Err(ResourcePolicyError::InvalidBounds);
        }
        if observation.latency_ms > MAX_MEASUREMENT_MILLISECONDS {
            return Err(ResourcePolicyError::InvalidBounds);
        }
        if observation.memory_bytes > MAX_MEASUREMENT_BYTES
            || observation.transfer_bytes > MAX_MEASUREMENT_BYTES
            || observation.usage_units > MAX_CHARGE_UNITS
        {
            return Err(ResourcePolicyError::InvalidBounds);
        }
    }
    observations.sort_by(|left, right| left.workload_id.cmp(&right.workload_id));
    if observations.windows(crate::PAIR_WINDOW_SIZE).any(|pair| pair[0].workload_id == pair[1].workload_id)
        || observations
            .iter()
            .zip(workloads)
            .any(|(observation, workload)| observation.workload_id != workload.workload_id)
    {
        return Err(ResourcePolicyError::BenchmarkRejected);
    }
    debug_assert_eq!(observations.len(), workloads.len());
    debug_assert!(observations.windows(crate::PAIR_WINDOW_SIZE).all(|pair| pair[0].workload_id < pair[1].workload_id));
    Ok(observations)
}

fn fault_rank(kind: ResourceFaultKind) -> u8 {
    match kind {
        ResourceFaultKind::PositiveOom => FAULT_RANK_POSITIVE_OOM,
        ResourceFaultKind::AmbiguousFailure => FAULT_RANK_AMBIGUOUS_FAILURE,
        ResourceFaultKind::WorkerLoss => FAULT_RANK_WORKER_LOSS,
        ResourceFaultKind::DuplicateCompletion => FAULT_RANK_DUPLICATE_COMPLETION,
        ResourceFaultKind::AccountingInterruption => FAULT_RANK_ACCOUNTING_INTERRUPTION,
        ResourceFaultKind::CasUnavailable => FAULT_RANK_CAS_UNAVAILABLE,
    }
}

fn benchmark_report_identity(report: &ResourceBenchmarkReport) -> Result<String, ResourcePolicyError> {
    let mut material = report.clone();
    material.report_blake3.clear();
    let identity = canonical_blake3(BENCHMARK_DOMAIN, &material)?;
    debug_assert!(crate::valid_blake3(&identity));
    debug_assert!(material.report_blake3.is_empty());
    Ok(identity)
}
