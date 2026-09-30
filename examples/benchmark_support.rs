#![allow(dead_code)]

use std::ffi::OsString;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::time::Instant;

use crunch_delta::bench_suite as delta_bench_suite;
use crunch_delta::plan_transfer;
use crunch_glue::ConversionCache;
use crunch_glue::CrunchDerivation;
use nix_compat::store_path::StorePath;
use serde::Deserialize;
use serde::Serialize;
use snix_castore::Node;
use tokio::io::AsyncWriteExt;

pub const BENCHMARK_BUNDLE_SCHEMA_V1: &str = "crunch-benchmark-bundle-v1";
pub const EVAL_SMOKE_ENTRY_POINT: &str = "cargo run --example benchmark_eval_smoke --";
pub const SUITE_ENTRY_POINT: &str = "cargo run --example benchmark_suite --";
pub const COMPARE_ENTRY_POINT: &str = "cargo run --example benchmark_compare --";
pub const DEFAULT_REPEAT_COUNT: u32 = 5;
pub const DEFAULT_LOGICAL_STORE_PREFIX: &str = "/crunch/store";
pub const EVAL_SMOKE_WORKLOAD_NAME: &str = "eval-fetch-git";
pub const CONVERSION_WORKLOAD_NAME: &str = "convert-multi-output";
pub const SUBSTITUTION_WORKLOAD_NAME: &str = "substitution-plan-delta-suite";
pub const BUILD_GRAPH_WORKLOAD_NAME: &str = "build-graph-package-set";
pub const MULTI_PHASE_WORKFLOW_WORKLOAD_NAME: &str = "workflow-package-set-eval-build-graph";
pub const STORE_AWARE_WORKLOAD_NAME: &str = "store-persist-lookup-blob";
pub const EVAL_SMOKE_CACHE_MODE: &str = "same-process-repeated-eval";
pub const EVAL_PHASE_METRIC_NAME: &str = "evaluation_wall_ns";
pub const CONVERSION_PHASE_METRIC_NAME: &str = "conversion_wall_ns";
pub const SUBSTITUTION_PHASE_METRIC_NAME: &str = "substitution_planning_wall_ns";
pub const BUILD_GRAPH_PHASE_METRIC_NAME: &str = "build_graph_wall_ns";
pub const STORE_PERSISTENCE_PHASE_METRIC_NAME: &str = "store_persistence_wall_ns";
pub const STORE_LOOKUP_PHASE_METRIC_NAME: &str = "store_lookup_wall_ns";
pub const TOTAL_PHASE_METRIC_NAME: &str = "total_wall_ns";
pub const LAZY_DISCOVERY_WORKLOAD_NAME: &str = "lazy-root-discovery-wide-package-set";
pub const LAZY_SELECTED_ROOT_WORKLOAD_NAME: &str = "lazy-selected-root-wide-package-set";
pub const EAGER_ALL_ROOTS_WORKLOAD_NAME: &str = "eager-all-roots-wide-package-set";
pub const PARALLEL_ALL_ROOTS_WORKLOAD_NAME: &str = "parallel-all-roots-wide-package-set";
pub const LAZY_DISCOVERY_METRIC_NAME: &str = "root_discovery_wall_ns";
pub const LAZY_SELECTED_ROOT_TOTAL_METRIC_NAME: &str = "selected_root_total_wall_ns";
pub const LAZY_SELECTED_ROOT_FORCE_METRIC_NAME: &str = "selected_root_force_wall_ns";
pub const LAZY_FORCE_COUNT_METRIC_NAME: &str = "explicit_top_level_root_force_count";
pub const LAZY_NONSELECTED_FORCE_COUNT_METRIC_NAME: &str = "explicit_nonselected_root_force_count";
pub const LAZY_ALL_ROOTS_METRIC_NAME: &str = "all_roots_total_wall_ns";
pub const PARALLEL_ALL_ROOTS_METRIC_NAME: &str = "parallel_all_roots_total_wall_ns";
pub const PARALLEL_ROOT_CONCURRENCY_METRIC_NAME: &str = "parallel_root_eval_concurrency";
const WIDE_FIXTURE_PATH: &str = "tests/fixtures/wide_package_set.ncl";
const LAZY_SELECTED_ROOT_LABEL: &str = "alpha";
const DEFAULT_PARALLEL_ALL_ROOTS_CONCURRENCY: u32 = 4;

#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
    Eval(crunch_eval::Error),
    Convert(crunch_glue::Error),
    Delta(crunch_delta::PlanError),
    Store(crunch_store::Error),
    Json(serde_json::Error),
    Command { tool: String, detail: String },
    InvalidArgument(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Io(err) => write!(f, "I/O error: {err}"),
            Error::Eval(err) => write!(f, "evaluation error: {err}"),
            Error::Convert(err) => write!(f, "conversion error: {err}"),
            Error::Delta(err) => write!(f, "delta planning error: {err}"),
            Error::Store(err) => write!(f, "store error: {err}"),
            Error::Json(err) => write!(f, "JSON error: {err}"),
            Error::Command { tool, detail } => write!(f, "command `{tool}` failed: {detail}"),
            Error::InvalidArgument(detail) => write!(f, "invalid argument: {detail}"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Io(err) => Some(err),
            Error::Eval(err) => Some(err),
            Error::Convert(err) => Some(err),
            Error::Delta(err) => Some(err),
            Error::Store(err) => Some(err),
            Error::Json(err) => Some(err),
            Error::Command { .. } | Error::InvalidArgument(_) => None,
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err)
    }
}

impl From<crunch_eval::Error> for Error {
    fn from(err: crunch_eval::Error) -> Self {
        Self::Eval(err)
    }
}

impl From<crunch_glue::Error> for Error {
    fn from(err: crunch_glue::Error) -> Self {
        Self::Convert(err)
    }
}

impl From<crunch_delta::PlanError> for Error {
    fn from(err: crunch_delta::PlanError) -> Self {
        Self::Delta(err)
    }
}

impl From<crunch_store::Error> for Error {
    fn from(err: crunch_store::Error) -> Self {
        Self::Store(err)
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Self::Json(err)
    }
}

#[derive(Debug, Clone)]
pub struct BenchmarkRequest {
    pub repeat_count: u32,
    pub bundle_out_path: PathBuf,
    pub command_argv: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkBundle {
    pub schema: String,
    pub generated_unix_s: u64,
    #[serde(default)]
    pub resource_cohort: EvaluationResourceCohort,
    pub repo_root: String,
    pub bundle_path: String,
    pub commit: String,
    pub git_dirty: bool,
    pub toolchain: ToolchainContext,
    pub host: HostContext,
    pub results: Vec<BenchmarkResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolchainContext {
    pub rustc_version: String,
    pub cargo_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostContext {
    pub os: String,
    pub arch: String,
    pub hostname: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvaluationResourceCohort {
    pub host_class: String,
    pub target: String,
    pub evaluator_id: String,
    pub evaluator_version: String,
    pub toolchain_id: String,
    pub policy_ref: String,
    pub fixture_set_id: String,
    pub repeat_count: u32,
    pub warm_state: String,
    pub cpu_time_support: MeasurementSupport,
    pub peak_rss_support: MeasurementSupport,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MeasurementSupport {
    pub status: String,
    pub mechanism: String,
    pub reason: Option<String>,
}

impl Default for MeasurementSupport {
    fn default() -> Self {
        Self {
            status: "unavailable".to_string(),
            mechanism: "legacy-bundle-no-mechanism".to_string(),
            reason: Some("legacy-bundle-has-no-resource-support-facts".to_string()),
        }
    }
}

impl Default for EvaluationResourceCohort {
    fn default() -> Self {
        Self {
            host_class: "legacy-unknown".to_string(),
            target: "legacy-unknown".to_string(),
            evaluator_id: "legacy-unknown".to_string(),
            evaluator_version: "legacy-unknown".to_string(),
            toolchain_id: "legacy-unknown".to_string(),
            policy_ref: "legacy-unbound".to_string(),
            fixture_set_id: "legacy-unbound".to_string(),
            repeat_count: 0,
            warm_state: "legacy-unknown".to_string(),
            cpu_time_support: MeasurementSupport::default(),
            peak_rss_support: MeasurementSupport::default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BenchmarkResourceMetric {
    pub name: String,
    pub unit: String,
    pub role: String,
    pub status: String,
    pub value: Option<u64>,
    pub mechanism: String,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkResult {
    pub workload_name: String,
    pub workload_kind: String,
    pub workload_path: String,
    pub rationale: String,
    pub operation: String,
    pub entry_point: String,
    pub command_argv: Vec<String>,
    pub cache_mode: String,
    pub repeat_count: u32,
    pub logical_store_prefix: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hermeticity_mode: Option<String>,
    pub root_count: u32,
    pub total_wall_ns: u64,
    pub sample_wall_ns: Vec<u64>,
    pub phase_metrics: Vec<BenchmarkMetric>,
    #[serde(default)]
    pub resource_metrics: Vec<BenchmarkResourceMetric>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkMetric {
    pub name: String,
    pub unit: String,
    pub value: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkloadDescriptor {
    pub workload_name: String,
    pub workload_kind: String,
    pub workload_path: String,
    pub rationale: String,
    pub operation: String,
    pub entry_point: String,
    pub cache_mode: String,
    pub logical_store_prefix: String,
}

#[derive(Clone)]
struct BenchmarkContext {
    repo_root: PathBuf,
    toolchain: ToolchainContext,
    host: HostContext,
    commit: String,
    git_dirty: bool,
}

#[derive(Clone)]
struct EvalWorkload {
    descriptor: WorkloadDescriptor,
    workload_path: PathBuf,
    import_paths: Vec<OsString>,
}

#[derive(Clone)]
struct ConversionWorkload {
    descriptor: WorkloadDescriptor,
    roots: Vec<(String, CrunchDerivation)>,
    metric_name: &'static str,
}

#[derive(Clone)]
struct SubstitutionWorkload {
    descriptor: WorkloadDescriptor,
    case_count: u32,
}

#[derive(Clone)]
struct MultiPhaseWorkflowWorkload {
    descriptor: WorkloadDescriptor,
    workload_path: PathBuf,
    import_paths: Vec<OsString>,
    conversion_store_prefix: String,
}

#[derive(Clone)]
struct StoreAwareWorkload {
    descriptor: WorkloadDescriptor,
}

pub fn default_eval_smoke_bundle_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target").join("benchmarks").join("eval-smoke.json")
}

pub fn default_suite_bundle_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target").join("benchmarks").join("suite.json")
}

pub fn eval_smoke_request(bundle_out_path: PathBuf, repeat_count: u32, command_argv: Vec<String>) -> BenchmarkRequest {
    BenchmarkRequest {
        repeat_count,
        bundle_out_path,
        command_argv,
    }
}

pub fn suite_request(bundle_out_path: PathBuf, repeat_count: u32, command_argv: Vec<String>) -> BenchmarkRequest {
    BenchmarkRequest {
        repeat_count,
        bundle_out_path,
        command_argv,
    }
}

pub fn run_eval_smoke_benchmark(request: &BenchmarkRequest) -> Result<BenchmarkBundle, Error> {
    validate_request(request)?;
    let context = benchmark_context()?;
    let workload = eval_smoke_workload(&context.repo_root)?;
    let result = benchmark_eval_workload(&workload, request.repeat_count, &request.command_argv)?;
    write_bundle(build_bundle(&context, &request.bundle_out_path, vec![result])?)
}

pub fn run_suite_benchmark(request: &BenchmarkRequest) -> Result<BenchmarkBundle, Error> {
    validate_request(request)?;
    let context = benchmark_context()?;
    let workloads = suite_workloads(&context.repo_root)?;
    let mut results = Vec::with_capacity(workloads.len());

    for workload in workloads {
        let result = match workload {
            SuiteWorkload::Evaluation(workload) => {
                benchmark_eval_workload(&workload, request.repeat_count, &request.command_argv)?
            }
            SuiteWorkload::Conversion(workload) => {
                benchmark_conversion_workload(&workload, request.repeat_count, &request.command_argv)?
            }
            SuiteWorkload::Substitution(workload) => {
                benchmark_substitution_workload(&workload, request.repeat_count, &request.command_argv)?
            }
            SuiteWorkload::MultiPhaseWorkflow(workload) => {
                benchmark_multi_phase_workflow_workload(&workload, request.repeat_count, &request.command_argv)?
            }
            SuiteWorkload::StoreAware(workload) => {
                benchmark_store_aware_workload(&workload, request.repeat_count, &request.command_argv)?
            }
            SuiteWorkload::LazyEval(workload) => {
                benchmark_lazy_eval_workload(&workload, request.repeat_count, &request.command_argv)?
            }
        };
        results.push(result);
    }

    write_bundle(build_bundle(&context, &request.bundle_out_path, results)?)
}

pub fn suite_workload_descriptors() -> Result<Vec<WorkloadDescriptor>, Error> {
    let repo_root = repo_root_path()?;
    let workloads = suite_workloads(&repo_root)?;
    Ok(workloads.into_iter().map(|workload| workload.descriptor()).collect())
}

pub fn render_bundle_json(bundle: &BenchmarkBundle) -> Result<String, Error> {
    if bundle.schema != BENCHMARK_BUNDLE_SCHEMA_V1 {
        return Err(Error::InvalidArgument(format!("unexpected benchmark bundle schema: {}", bundle.schema)));
    }
    if bundle.results.is_empty() {
        return Err(Error::InvalidArgument("bundle must contain at least one result".to_string()));
    }
    Ok(serde_json::to_string_pretty(bundle)?)
}

#[derive(Debug, Clone, PartialEq)]
pub struct CompareThresholds {
    pub absolute_threshold_ns: u64,
    pub percent_threshold: f64,
    pub named_absolute_thresholds: std::collections::BTreeMap<String, u64>,
    pub named_percent_thresholds: std::collections::BTreeMap<String, f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkComparisonReport {
    pub schema: String,
    pub baseline_path: String,
    pub fresh_path: String,
    pub thresholds: BenchmarkComparisonThresholds,
    pub resource_cohort_compatible: bool,
    pub resource_cohort_mismatches: Vec<String>,
    pub matched_workloads: Vec<WorkloadComparison>,
    pub missing_from_fresh: Vec<String>,
    pub missing_from_baseline: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub largest_regression: Option<MetricComparison>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub largest_win: Option<MetricComparison>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkComparisonThresholds {
    pub absolute_threshold_ns: u64,
    pub percent_threshold: f64,
    pub named_absolute_thresholds: std::collections::BTreeMap<String, u64>,
    pub named_percent_thresholds: std::collections::BTreeMap<String, f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkloadComparison {
    pub workload_name: String,
    pub matched_metrics: Vec<MetricComparison>,
    pub missing_from_fresh: Vec<String>,
    pub missing_from_baseline: Vec<String>,
    pub unavailable_in_both: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricComparison {
    pub workload_name: String,
    pub metric_name: String,
    pub baseline_value: u64,
    pub fresh_value: u64,
    pub delta_ns: i128,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delta_percent: Option<f64>,
    pub direction: String,
    pub threshold_applied: bool,
    pub absolute_threshold: u64,
    pub percent_threshold: f64,
    pub exceeds_absolute_threshold: bool,
    pub exceeds_percent_threshold: bool,
}

pub fn default_compare_thresholds() -> CompareThresholds {
    CompareThresholds {
        absolute_threshold_ns: 0,
        percent_threshold: 0.0,
        named_absolute_thresholds: std::collections::BTreeMap::new(),
        named_percent_thresholds: std::collections::BTreeMap::new(),
    }
}

pub fn compare_benchmark_files(
    baseline_path: &Path,
    fresh_path: &Path,
    thresholds: &CompareThresholds,
) -> Result<BenchmarkComparisonReport, Error> {
    let baseline = read_bundle_json(baseline_path)?;
    let fresh = read_bundle_json(fresh_path)?;
    compare_bundles(
        &baseline,
        &fresh,
        &baseline_path.display().to_string(),
        &fresh_path.display().to_string(),
        thresholds,
    )
}

pub fn compare_bundles(
    baseline: &BenchmarkBundle,
    fresh: &BenchmarkBundle,
    baseline_path: &str,
    fresh_path: &str,
    thresholds: &CompareThresholds,
) -> Result<BenchmarkComparisonReport, Error> {
    validate_compare_thresholds(thresholds)?;
    validate_bundle_for_compare(baseline)?;
    validate_bundle_for_compare(fresh)?;

    let resource_cohort_mismatches = resource_cohort_mismatches(&baseline.resource_cohort, &fresh.resource_cohort);
    let resource_cohort_compatible = resource_cohort_mismatches.is_empty();
    let baseline_index = index_results_by_name(&baseline.results)?;
    let fresh_index = index_results_by_name(&fresh.results)?;
    let mut workload_names: Vec<String> = baseline_index.keys().chain(fresh_index.keys()).cloned().collect();
    workload_names.sort();
    workload_names.dedup();

    let mut matched_workloads = Vec::new();
    let mut missing_from_fresh = Vec::new();
    let mut missing_from_baseline = Vec::new();

    for workload_name in workload_names {
        match (baseline_index.get(&workload_name), fresh_index.get(&workload_name)) {
            (Some(baseline_result), Some(fresh_result)) => {
                matched_workloads.push(compare_workload(
                    baseline_result,
                    fresh_result,
                    thresholds,
                    resource_cohort_compatible,
                )?);
            }
            (Some(_), None) => missing_from_fresh.push(workload_name),
            (None, Some(_)) => missing_from_baseline.push(workload_name),
            (None, None) => {}
        }
    }

    let all_metric_deltas: Vec<MetricComparison> =
        matched_workloads.iter().flat_map(|workload| workload.matched_metrics.clone()).collect();
    let largest_regression = all_metric_deltas
        .iter()
        .filter(|metric| metric.delta_ns > 0)
        .max_by_key(|metric| metric.delta_ns)
        .cloned();
    let largest_win = all_metric_deltas
        .iter()
        .filter(|metric| metric.delta_ns < 0)
        .min_by_key(|metric| metric.delta_ns)
        .cloned();

    Ok(BenchmarkComparisonReport {
        schema: "crunch-benchmark-compare-v1".to_string(),
        baseline_path: baseline_path.to_string(),
        fresh_path: fresh_path.to_string(),
        thresholds: BenchmarkComparisonThresholds {
            absolute_threshold_ns: thresholds.absolute_threshold_ns,
            percent_threshold: thresholds.percent_threshold,
            named_absolute_thresholds: thresholds.named_absolute_thresholds.clone(),
            named_percent_thresholds: thresholds.named_percent_thresholds.clone(),
        },
        resource_cohort_compatible,
        resource_cohort_mismatches,
        matched_workloads,
        missing_from_fresh,
        missing_from_baseline,
        largest_regression,
        largest_win,
    })
}

pub fn render_comparison_json(report: &BenchmarkComparisonReport) -> Result<String, Error> {
    Ok(serde_json::to_string_pretty(report)?)
}

pub fn render_comparison_human(report: &BenchmarkComparisonReport) -> String {
    let mut out = String::new();
    out.push_str(&format!("baseline={}\n", report.baseline_path));
    out.push_str(&format!("fresh={}\n", report.fresh_path));
    out.push_str(&format!(
        "thresholds: absolute_ns={} percent={}\n",
        report.thresholds.absolute_threshold_ns, report.thresholds.percent_threshold
    ));
    out.push_str(&format!(
        "resource_cohort_compatible={} mismatches={:?}\n",
        report.resource_cohort_compatible, report.resource_cohort_mismatches
    ));

    if let Some(metric) = &report.largest_regression {
        out.push_str(&format!(
            "largest regression: {}.{} delta_ns={} delta_percent={}\n",
            metric.workload_name,
            metric.metric_name,
            metric.delta_ns,
            render_optional_percent(metric.delta_percent),
        ));
    }
    if let Some(metric) = &report.largest_win {
        out.push_str(&format!(
            "largest win: {}.{} delta_ns={} delta_percent={}\n",
            metric.workload_name,
            metric.metric_name,
            metric.delta_ns,
            render_optional_percent(metric.delta_percent),
        ));
    }

    for workload in &report.matched_workloads {
        out.push_str(&format!("workload={}\n", workload.workload_name));
        for metric in &workload.matched_metrics {
            out.push_str(&format!(
                "  metric={} baseline={} fresh={} delta_ns={} delta_percent={} direction={} highlighted={}\n",
                metric.metric_name,
                metric.baseline_value,
                metric.fresh_value,
                metric.delta_ns,
                render_optional_percent(metric.delta_percent),
                metric.direction,
                metric.exceeds_absolute_threshold || metric.exceeds_percent_threshold,
            ));
        }
        if !workload.missing_from_fresh.is_empty() {
            out.push_str(&format!("  missing_from_fresh={:?}\n", workload.missing_from_fresh));
        }
        if !workload.missing_from_baseline.is_empty() {
            out.push_str(&format!("  missing_from_baseline={:?}\n", workload.missing_from_baseline));
        }
        if !workload.unavailable_in_both.is_empty() {
            out.push_str(&format!("  unavailable_in_both={:?}\n", workload.unavailable_in_both));
        }
    }

    if !report.missing_from_fresh.is_empty() {
        out.push_str(&format!("missing_from_fresh={:?}\n", report.missing_from_fresh));
    }
    if !report.missing_from_baseline.is_empty() {
        out.push_str(&format!("missing_from_baseline={:?}\n", report.missing_from_baseline));
    }

    out
}

fn resource_cohort_mismatches(baseline: &EvaluationResourceCohort, fresh: &EvaluationResourceCohort) -> Vec<String> {
    let fields = [
        (baseline.host_class == fresh.host_class, "host-class"),
        (baseline.target == fresh.target, "target"),
        (baseline.evaluator_id == fresh.evaluator_id, "evaluator-id"),
        (baseline.evaluator_version == fresh.evaluator_version, "evaluator-version"),
        (baseline.toolchain_id == fresh.toolchain_id, "toolchain-id"),
        (baseline.policy_ref == fresh.policy_ref, "policy-ref"),
        (baseline.fixture_set_id == fresh.fixture_set_id, "fixture-set-id"),
        (baseline.repeat_count == fresh.repeat_count, "repeat-count"),
        (baseline.warm_state == fresh.warm_state, "warm-state"),
        (baseline.cpu_time_support == fresh.cpu_time_support, "cpu-time-support"),
        (baseline.peak_rss_support == fresh.peak_rss_support, "peak-rss-support"),
    ];
    fields.into_iter().filter(|(matches, _)| !matches).map(|(_, name)| name.to_string()).collect()
}

fn render_optional_percent(percent: Option<f64>) -> String {
    match percent {
        Some(value) => format!("{value:.2}"),
        None => "n/a".to_string(),
    }
}

fn validate_compare_thresholds(thresholds: &CompareThresholds) -> Result<(), Error> {
    if !thresholds.percent_threshold.is_finite() {
        return Err(Error::InvalidArgument("percent threshold must be finite".to_string()));
    }
    if thresholds.percent_threshold < 0.0 {
        return Err(Error::InvalidArgument("percent threshold must be >= 0".to_string()));
    }
    for (name, value) in &thresholds.named_percent_thresholds {
        if !value.is_finite() || *value < 0.0 {
            return Err(Error::InvalidArgument(format!("named percent threshold must be finite and >= 0: {name}")));
        }
    }
    Ok(())
}

fn validate_bundle_for_compare(bundle: &BenchmarkBundle) -> Result<(), Error> {
    if bundle.schema != BENCHMARK_BUNDLE_SCHEMA_V1 {
        return Err(Error::InvalidArgument(format!("unexpected benchmark bundle schema: {}", bundle.schema)));
    }
    if bundle.results.is_empty() {
        return Err(Error::InvalidArgument("benchmark bundle must contain results".to_string()));
    }
    Ok(())
}

fn read_bundle_json(path: &Path) -> Result<BenchmarkBundle, Error> {
    let content = std::fs::read_to_string(path)?;
    let bundle: BenchmarkBundle = serde_json::from_str(&content)?;
    validate_bundle_for_compare(&bundle)?;
    Ok(bundle)
}

fn index_results_by_name(
    results: &[BenchmarkResult],
) -> Result<std::collections::BTreeMap<String, &BenchmarkResult>, Error> {
    let mut index = std::collections::BTreeMap::new();
    for result in results {
        if index.insert(result.workload_name.clone(), result).is_some() {
            return Err(Error::InvalidArgument(format!("duplicate workload in bundle: {}", result.workload_name)));
        }
    }
    Ok(index)
}

fn compare_workload(
    baseline: &BenchmarkResult,
    fresh: &BenchmarkResult,
    thresholds: &CompareThresholds,
    resource_cohort_compatible: bool,
) -> Result<WorkloadComparison, Error> {
    let baseline_metrics = index_metrics(baseline)?;
    let fresh_metrics = index_metrics(fresh)?;
    let mut metric_names: Vec<String> = baseline_metrics.keys().chain(fresh_metrics.keys()).cloned().collect();
    metric_names.sort();
    metric_names.dedup();

    let mut matched_metrics = Vec::new();
    let mut missing_from_fresh = Vec::new();
    let mut missing_from_baseline = Vec::new();
    let mut unavailable_in_both = Vec::new();

    for metric_name in metric_names {
        match (baseline_metrics.get(&metric_name), fresh_metrics.get(&metric_name)) {
            (Some(Some(baseline_value)), Some(Some(fresh_value))) => {
                matched_metrics.push(compare_metric(
                    &baseline.workload_name,
                    &metric_name,
                    *baseline_value,
                    *fresh_value,
                    thresholds,
                    resource_cohort_compatible,
                )?);
            }
            (Some(None), Some(None)) => unavailable_in_both.push(metric_name),
            (Some(Some(_)), Some(None) | None) => missing_from_fresh.push(metric_name),
            (Some(None) | None, Some(Some(_))) => missing_from_baseline.push(metric_name),
            (Some(None), None) => missing_from_fresh.push(metric_name),
            (None, Some(None)) => missing_from_baseline.push(metric_name),
            (None, None) => {}
        }
    }

    Ok(WorkloadComparison {
        workload_name: baseline.workload_name.clone(),
        matched_metrics,
        missing_from_fresh,
        missing_from_baseline,
        unavailable_in_both,
    })
}

fn index_metrics(result: &BenchmarkResult) -> Result<std::collections::BTreeMap<String, Option<u64>>, Error> {
    let mut metrics = std::collections::BTreeMap::new();
    metrics.insert(TOTAL_PHASE_METRIC_NAME.to_string(), Some(result.total_wall_ns));
    for metric in &result.phase_metrics {
        if metrics.insert(metric.name.clone(), Some(metric.value)).is_some() {
            return Err(Error::InvalidArgument(format!(
                "duplicate metric in workload {}: {}",
                result.workload_name, metric.name
            )));
        }
    }
    for metric in &result.resource_metrics {
        if metrics.insert(metric.name.clone(), metric.value).is_some() {
            return Err(Error::InvalidArgument(format!(
                "duplicate resource metric in workload {}: {}",
                result.workload_name, metric.name
            )));
        }
    }
    Ok(metrics)
}

fn compare_metric(
    workload_name: &str,
    metric_name: &str,
    baseline_value: u64,
    fresh_value: u64,
    thresholds: &CompareThresholds,
    resource_cohort_compatible: bool,
) -> Result<MetricComparison, Error> {
    let delta_ns = i128::from(fresh_value) - i128::from(baseline_value);
    let delta_percent = if baseline_value == 0 {
        None
    } else {
        Some(((fresh_value as f64 - baseline_value as f64) / baseline_value as f64) * 100.0)
    };
    let absolute_threshold = thresholds
        .named_absolute_thresholds
        .get(metric_name)
        .copied()
        .unwrap_or(thresholds.absolute_threshold_ns);
    let percent_threshold = thresholds
        .named_percent_thresholds
        .get(metric_name)
        .copied()
        .unwrap_or(thresholds.percent_threshold);
    let threshold_applied = resource_cohort_compatible;
    let exceeds_absolute_threshold = threshold_applied && delta_ns.unsigned_abs() >= u128::from(absolute_threshold);
    let exceeds_percent_threshold =
        threshold_applied && delta_percent.is_some_and(|percent| percent.abs() >= percent_threshold);
    let direction = if delta_ns > 0 {
        "regression"
    } else if delta_ns < 0 {
        "win"
    } else {
        "flat"
    };

    Ok(MetricComparison {
        workload_name: workload_name.to_string(),
        metric_name: metric_name.to_string(),
        baseline_value,
        fresh_value,
        delta_ns,
        delta_percent,
        direction: direction.to_string(),
        threshold_applied,
        absolute_threshold,
        percent_threshold,
        exceeds_absolute_threshold,
        exceeds_percent_threshold,
    })
}

#[derive(Clone)]
struct LazyEvalWorkload {
    descriptor: WorkloadDescriptor,
    workload_path: PathBuf,
    import_paths: Vec<OsString>,
    selected_root_label: String,
}

enum SuiteWorkload {
    Evaluation(EvalWorkload),
    Conversion(ConversionWorkload),
    Substitution(SubstitutionWorkload),
    MultiPhaseWorkflow(MultiPhaseWorkflowWorkload),
    StoreAware(StoreAwareWorkload),
    LazyEval(LazyEvalWorkload),
}

impl SuiteWorkload {
    fn descriptor(self) -> WorkloadDescriptor {
        match self {
            SuiteWorkload::Evaluation(workload) => workload.descriptor,
            SuiteWorkload::Conversion(workload) => workload.descriptor,
            SuiteWorkload::Substitution(workload) => workload.descriptor,
            SuiteWorkload::MultiPhaseWorkflow(workload) => workload.descriptor,
            SuiteWorkload::StoreAware(workload) => workload.descriptor,
            SuiteWorkload::LazyEval(workload) => workload.descriptor,
        }
    }
}

fn validate_request(request: &BenchmarkRequest) -> Result<(), Error> {
    if request.repeat_count == 0 {
        return Err(Error::InvalidArgument("repeat_count must be > 0".to_string()));
    }
    if request.command_argv.is_empty() {
        return Err(Error::InvalidArgument("command_argv must not be empty".to_string()));
    }
    if request.bundle_out_path.as_os_str().is_empty() {
        return Err(Error::InvalidArgument("bundle_out_path must not be empty".to_string()));
    }
    Ok(())
}

fn benchmark_context() -> Result<BenchmarkContext, Error> {
    let repo_root = repo_root_path()?;
    let toolchain = collect_toolchain_context()?;
    let host = collect_host_context();
    let commit = git_commit(&repo_root)?;
    let git_dirty = git_is_dirty(&repo_root)?;
    Ok(BenchmarkContext {
        repo_root,
        toolchain,
        host,
        commit,
        git_dirty,
    })
}

fn build_bundle(
    context: &BenchmarkContext,
    bundle_out_path: &Path,
    results: Vec<BenchmarkResult>,
) -> Result<BenchmarkBundle, Error> {
    if results.is_empty() {
        return Err(Error::InvalidArgument("benchmark bundle must contain at least one result".to_string()));
    }
    let resource_cohort = build_resource_cohort(context, &results)?;
    Ok(BenchmarkBundle {
        schema: BENCHMARK_BUNDLE_SCHEMA_V1.to_string(),
        generated_unix_s: unix_timestamp_now()?,
        resource_cohort,
        repo_root: context.repo_root.display().to_string(),
        bundle_path: bundle_out_path.display().to_string(),
        commit: context.commit.clone(),
        git_dirty: context.git_dirty,
        toolchain: context.toolchain.clone(),
        host: context.host.clone(),
        results,
    })
}

fn build_resource_cohort(
    context: &BenchmarkContext,
    results: &[BenchmarkResult],
) -> Result<EvaluationResourceCohort, Error> {
    let repeat_count = results
        .first()
        .map(|result| result.repeat_count)
        .ok_or_else(|| Error::InvalidArgument("resource cohort requires results".to_string()))?;
    if results.iter().any(|result| result.repeat_count != repeat_count) {
        return Err(Error::InvalidArgument("resource cohort requires one repeat policy".to_string()));
    }
    let host_class = format!("{}-{}", context.host.os, context.host.arch);
    let toolchain_id = digest_fields(&[
        context.toolchain.rustc_version.as_str(),
        context.toolchain.cargo_version.as_str(),
    ])?;
    let mut fixture_fields: Vec<&str> = results
        .iter()
        .flat_map(|result| [result.workload_name.as_str(), result.workload_path.as_str()])
        .collect();
    fixture_fields.sort_unstable();
    let fixture_set_id = digest_fields(&fixture_fields)?;
    let unavailable = MeasurementSupport {
        status: "unavailable".to_string(),
        mechanism: "unavailable".to_string(),
        reason: Some("in-process-benchmark-has-no-operation-scoped-resource-observation".to_string()),
    };
    Ok(EvaluationResourceCohort {
        host_class: host_class.clone(),
        target: host_class,
        evaluator_id: crunch_eval::EVALUATOR_ID.to_string(),
        evaluator_version: crunch_eval::EVALUATOR_VERSION.to_string(),
        toolchain_id,
        policy_ref: "evaluation-policy:in-process-observe-only-v1".to_string(),
        fixture_set_id,
        repeat_count,
        warm_state: "declared-per-workload".to_string(),
        cpu_time_support: unavailable.clone(),
        peak_rss_support: unavailable,
    })
}

fn digest_fields(fields: &[&str]) -> Result<String, Error> {
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"mantle:benchmark-resource-cohort:v1\0");
    for field in fields {
        let bytes = field.as_bytes();
        let byte_count = u64::try_from(bytes.len())
            .map_err(|_| Error::InvalidArgument("resource cohort field is too large".to_string()))?;
        hasher.update(&byte_count.to_le_bytes());
        hasher.update(bytes);
    }
    Ok(format!("blake3:{}", hasher.finalize().to_hex()))
}

fn write_bundle(bundle: BenchmarkBundle) -> Result<BenchmarkBundle, Error> {
    let bundle_path = PathBuf::from(&bundle.bundle_path);
    let parent_dir = bundle_parent_dir(&bundle_path);
    std::fs::create_dir_all(parent_dir)?;
    let bundle_json = render_bundle_json(&bundle)?;
    std::fs::write(&bundle_path, bundle_json)?;
    Ok(bundle)
}

fn bundle_parent_dir(bundle_out_path: &Path) -> &Path {
    match bundle_out_path.parent() {
        Some(parent) if !parent.as_os_str().is_empty() => parent,
        _ => Path::new("."),
    }
}

fn repo_root_path() -> Result<PathBuf, Error> {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    if !repo_root.is_absolute() {
        return Err(Error::InvalidArgument("repo root must be absolute".to_string()));
    }
    if !repo_root.join("Cargo.toml").exists() {
        return Err(Error::InvalidArgument("repo root must contain Cargo.toml".to_string()));
    }
    Ok(repo_root)
}

fn suite_workloads(repo_root: &Path) -> Result<Vec<SuiteWorkload>, Error> {
    let common_import_paths = workload_import_paths(repo_root)?;
    let eval_workload = EvalWorkload {
        descriptor: workload_descriptor(
            EVAL_SMOKE_WORKLOAD_NAME,
            "evaluation",
            repo_root.join("examples").join("fetch-git.ncl"),
            "Nickel evaluation of a checked-in fetcher derivation exercises derivation extraction without introducing build or network execution.",
            "crunch_eval::evaluate_and_extract_named_roots::<CrunchDerivation>",
            SUITE_ENTRY_POINT,
            EVAL_SMOKE_CACHE_MODE,
            DEFAULT_LOGICAL_STORE_PREFIX,
        ),
        workload_path: repo_root.join("examples").join("fetch-git.ncl"),
        import_paths: common_import_paths.clone(),
    };
    let conversion_roots = evaluate_roots(&repo_root.join("examples").join("multi-output.ncl"), &common_import_paths)?;
    let build_graph_path = repo_root.join("examples").join("package-set.ncl");
    let build_graph_roots = evaluate_roots(&build_graph_path, &common_import_paths)?;
    let substitution_case_count = usize_to_u32(delta_bench_suite().cases.len())?;

    Ok(vec![
        SuiteWorkload::Evaluation(eval_workload.clone()),
        SuiteWorkload::Conversion(ConversionWorkload {
            descriptor: workload_descriptor(
                CONVERSION_WORKLOAD_NAME,
                "conversion",
                repo_root.join("examples").join("multi-output.ncl"),
                "Multi-output conversion exercises store-path construction, output naming, and glue-layer derivation lowering without requiring a sandbox build.",
                "crunch_glue::convert over a checked-in multi-output derivation",
                SUITE_ENTRY_POINT,
                "pre-evaluated-roots repeated conversion",
                "/nix/store",
            ),
            roots: conversion_roots,
            metric_name: CONVERSION_PHASE_METRIC_NAME,
        }),
        SuiteWorkload::Substitution(SubstitutionWorkload {
            descriptor: WorkloadDescriptor {
                workload_name: SUBSTITUTION_WORKLOAD_NAME.to_string(),
                workload_kind: "substitution".to_string(),
                workload_path: "crates/crunch-delta/src/fixtures.rs::bench_suite()".to_string(),
                rationale: "The fixed crunch-delta bench suite is a checked-in substitution-planning workload that measures reuse planning with no network or store mutation.".to_string(),
                operation: "crunch_delta::plan_transfer over crunch_delta::bench_suite()".to_string(),
                entry_point: SUITE_ENTRY_POINT.to_string(),
                cache_mode: "checked-in delta fixture suite".to_string(),
                logical_store_prefix: DEFAULT_LOGICAL_STORE_PREFIX.to_string(),
            },
            case_count: substitution_case_count,
        }),
        SuiteWorkload::Conversion(ConversionWorkload {
            descriptor: workload_descriptor(
                BUILD_GRAPH_WORKLOAD_NAME,
                "build-graph",
                build_graph_path.clone(),
                "The package-set example contains multiple named roots and shared builder inputs, so repeated conversion measures build-graph preparation on a checked-in derivation graph.",
                "crunch_glue::convert over pre-evaluated package-set roots",
                SUITE_ENTRY_POINT,
                "pre-evaluated-roots repeated graph conversion",
                "/nix/store",
            ),
            roots: build_graph_roots,
            metric_name: BUILD_GRAPH_PHASE_METRIC_NAME,
        }),
        SuiteWorkload::MultiPhaseWorkflow(MultiPhaseWorkflowWorkload {
            descriptor: workload_descriptor(
                MULTI_PHASE_WORKFLOW_WORKLOAD_NAME,
                "workflow",
                build_graph_path.clone(),
                "A checked-in package-set workflow that evaluates then lowers the same roots in one run, so maintainers can separate evaluation cost from build-graph preparation cost.",
                "crunch_eval::evaluate_and_extract_named_roots::<CrunchDerivation> + crunch_glue::convert over examples/package-set.ncl",
                SUITE_ENTRY_POINT,
                "same-process repeated eval+graph conversion",
                "/nix/store",
            ),
            workload_path: build_graph_path,
            import_paths: common_import_paths.clone(),
            conversion_store_prefix: "/nix/store".to_string(),
        }),
        SuiteWorkload::StoreAware(StoreAwareWorkload {
            descriptor: WorkloadDescriptor {
                workload_name: STORE_AWARE_WORKLOAD_NAME.to_string(),
                workload_kind: "store".to_string(),
                workload_path: "examples/benchmark_support.rs::benchmark_store_aware_workload".to_string(),
                rationale: "A fresh temp store per sample persists one signed blob output, reopens the store, and times the follow-up cached-node lookup without inventing store phases for unrelated workloads.".to_string(),
                operation: "crunch_store::StoreHandle::persist_and_export_signed_output + StoreHandle::cached_node_for_path over deterministic local bytes".to_string(),
                entry_point: SUITE_ENTRY_POINT.to_string(),
                cache_mode: "fresh temp store per sample".to_string(),
                logical_store_prefix: DEFAULT_LOGICAL_STORE_PREFIX.to_string(),
            },
        }),
        SuiteWorkload::LazyEval(LazyEvalWorkload {
            descriptor: workload_descriptor(
                LAZY_DISCOVERY_WORKLOAD_NAME,
                "lazy-eval",
                repo_root.join(WIDE_FIXTURE_PATH),
                "Measures lazy root label discovery on a wide package-set fixture without deep-forcing any root values.",
                "EvaluationSession::open_file + root_labels",
                SUITE_ENTRY_POINT,
                "cold-per-sample",
                "/nix/store",
            ),
            workload_path: repo_root.join(WIDE_FIXTURE_PATH),
            import_paths: common_import_paths.clone(),
            selected_root_label: String::new(),
        }),
        SuiteWorkload::LazyEval(LazyEvalWorkload {
            descriptor: workload_descriptor(
                LAZY_SELECTED_ROOT_WORKLOAD_NAME,
                "lazy-eval",
                repo_root.join(WIDE_FIXTURE_PATH),
                "Measures end-to-end latency to obtain one selected root from a wide package set through the lazy session API.",
                &format!("EvaluationSession::open_file + force_root(\"{}\")", LAZY_SELECTED_ROOT_LABEL),
                SUITE_ENTRY_POINT,
                "cold-per-sample",
                "/nix/store",
            ),
            workload_path: repo_root.join(WIDE_FIXTURE_PATH),
            import_paths: common_import_paths.clone(),
            selected_root_label: LAZY_SELECTED_ROOT_LABEL.to_string(),
        }),
        SuiteWorkload::LazyEval(LazyEvalWorkload {
            descriptor: workload_descriptor(
                EAGER_ALL_ROOTS_WORKLOAD_NAME,
                "lazy-eval",
                repo_root.join(WIDE_FIXTURE_PATH),
                "Guardrail: measures all-roots eager path on the same fixture to detect regressions from lazy changes.",
                "EvaluationSession::open_file + force_all_roots",
                SUITE_ENTRY_POINT,
                "cold-per-sample",
                "/nix/store",
            ),
            workload_path: repo_root.join(WIDE_FIXTURE_PATH),
            import_paths: common_import_paths.clone(),
            selected_root_label: String::new(),
        }),
        SuiteWorkload::LazyEval(LazyEvalWorkload {
            descriptor: workload_descriptor(
                PARALLEL_ALL_ROOTS_WORKLOAD_NAME,
                "lazy-eval",
                repo_root.join(WIDE_FIXTURE_PATH),
                "Measures bounded parallel all-roots forcing on the same wide package-set fixture and records the concurrency used for the run.",
                &format!(
                    "EvaluationSession::open_file + force_all_roots_bounded(concurrency={DEFAULT_PARALLEL_ALL_ROOTS_CONCURRENCY})"
                ),
                SUITE_ENTRY_POINT,
                "cold-per-sample isolated benchmark run",
                "/nix/store",
            ),
            workload_path: repo_root.join(WIDE_FIXTURE_PATH),
            import_paths: common_import_paths,
            selected_root_label: String::new(),
        }),
    ])
}

fn workload_descriptor(
    workload_name: &str,
    workload_kind: &str,
    workload_path: PathBuf,
    rationale: &str,
    operation: &str,
    entry_point: &str,
    cache_mode: &str,
    logical_store_prefix: &str,
) -> WorkloadDescriptor {
    WorkloadDescriptor {
        workload_name: workload_name.to_string(),
        workload_kind: workload_kind.to_string(),
        workload_path: workload_path.display().to_string(),
        rationale: rationale.to_string(),
        operation: operation.to_string(),
        entry_point: entry_point.to_string(),
        cache_mode: cache_mode.to_string(),
        logical_store_prefix: logical_store_prefix.to_string(),
    }
}

fn workload_import_paths(repo_root: &Path) -> Result<Vec<OsString>, Error> {
    let stdlib_path = crunch_eval::stdlib::stdlib_import_path().map_err(|err| Error::Command {
        tool: "stdlib_import_path".to_string(),
        detail: err.to_string(),
    })?;
    if !stdlib_path.exists() {
        return Err(Error::InvalidArgument(format!("stdlib path must exist: {}", stdlib_path.display())));
    }
    Ok(vec![stdlib_path.into_os_string(), repo_root.as_os_str().to_os_string()])
}

fn evaluate_roots(file: &Path, import_paths: &[OsString]) -> Result<Vec<(String, CrunchDerivation)>, Error> {
    let roots = crunch_eval::evaluate_and_extract_named_roots::<CrunchDerivation>(file, import_paths)?;
    if roots.is_empty() {
        return Err(Error::InvalidArgument(format!("benchmark workload produced no roots: {}", file.display())));
    }
    Ok(roots)
}

fn eval_smoke_workload(repo_root: &Path) -> Result<EvalWorkload, Error> {
    let import_paths = workload_import_paths(repo_root)?;
    let workload_path = repo_root.join("examples").join("fetch-git.ncl");
    if !workload_path.exists() {
        return Err(Error::InvalidArgument(format!("eval smoke workload must exist: {}", workload_path.display())));
    }
    Ok(EvalWorkload {
        descriptor: workload_descriptor(
            EVAL_SMOKE_WORKLOAD_NAME,
            "evaluation",
            workload_path.clone(),
            "Nickel evaluation of a checked-in fetcher derivation exercises derivation extraction without introducing build or network execution.",
            "crunch_eval::evaluate_and_extract_named_roots::<CrunchDerivation>",
            EVAL_SMOKE_ENTRY_POINT,
            EVAL_SMOKE_CACHE_MODE,
            DEFAULT_LOGICAL_STORE_PREFIX,
        ),
        workload_path,
        import_paths,
    })
}

fn benchmark_eval_workload(
    workload: &EvalWorkload,
    repeat_count: u32,
    command_argv: &[String],
) -> Result<BenchmarkResult, Error> {
    let timed = time_repeated_operation(repeat_count, || {
        let roots = crunch_eval::evaluate_and_extract_named_roots::<CrunchDerivation>(
            &workload.workload_path,
            &workload.import_paths,
        )?;
        usize_to_u32(roots.len())
    })?;
    let phase_metrics = vec![named_metric(
        EVAL_PHASE_METRIC_NAME,
        sum_samples_ns(&timed.sample_wall_ns)?,
    )];
    build_result(
        &workload.descriptor,
        command_argv,
        repeat_count,
        timed.root_count,
        timed.total_wall_ns,
        timed.sample_wall_ns,
        phase_metrics,
    )
}

fn benchmark_conversion_workload(
    workload: &ConversionWorkload,
    repeat_count: u32,
    command_argv: &[String],
) -> Result<BenchmarkResult, Error> {
    let roots = &workload.roots;
    let root_count = usize_to_u32(roots.len())?;
    let store_prefix = workload.descriptor.logical_store_prefix.clone();
    let timed = time_repeated_operation(repeat_count, || {
        let mut cache = ConversionCache::new(&store_prefix);
        for (_, drv) in roots {
            let _ = crunch_glue::convert(drv, &mut cache)?;
        }
        Ok(root_count)
    })?;
    let phase_metrics = vec![named_metric(
        workload.metric_name,
        sum_samples_ns(&timed.sample_wall_ns)?,
    )];
    build_result(
        &workload.descriptor,
        command_argv,
        repeat_count,
        timed.root_count,
        timed.total_wall_ns,
        timed.sample_wall_ns,
        phase_metrics,
    )
}

fn benchmark_substitution_workload(
    workload: &SubstitutionWorkload,
    repeat_count: u32,
    command_argv: &[String],
) -> Result<BenchmarkResult, Error> {
    let suite = delta_bench_suite();
    let timed = time_repeated_operation(repeat_count, move || {
        for case in &suite.cases {
            let plan = plan_transfer(&case.sender, &case.receiver)?;
            if plan.transferred_bytes != case.expected_coarse_bytes {
                return Err(Error::InvalidArgument(format!(
                    "delta bench case {} regressed: expected {}, got {}",
                    case.name, case.expected_coarse_bytes, plan.transferred_bytes
                )));
            }
        }
        usize_to_u32(suite.cases.len())
    })?;
    let phase_metrics = vec![named_metric(
        SUBSTITUTION_PHASE_METRIC_NAME,
        sum_samples_ns(&timed.sample_wall_ns)?,
    )];
    build_result(
        &workload.descriptor,
        command_argv,
        repeat_count,
        timed.root_count.max(workload.case_count),
        timed.total_wall_ns,
        timed.sample_wall_ns,
        phase_metrics,
    )
}

fn benchmark_multi_phase_workflow_workload(
    workload: &MultiPhaseWorkflowWorkload,
    repeat_count: u32,
    command_argv: &[String],
) -> Result<BenchmarkResult, Error> {
    let timed = time_repeated_operation_with_phase_metrics(repeat_count, || {
        let evaluation_started_at = Instant::now();
        let roots = evaluate_roots(&workload.workload_path, &workload.import_paths)?;
        let evaluation_wall_ns = duration_to_ns_u64(evaluation_started_at.elapsed())?;
        let root_count = usize_to_u32(roots.len())?;

        let conversion_started_at = Instant::now();
        let mut cache = ConversionCache::new(&workload.conversion_store_prefix);
        for (_, drv) in &roots {
            let _ = crunch_glue::convert(drv, &mut cache)?;
        }
        let build_graph_wall_ns = duration_to_ns_u64(conversion_started_at.elapsed())?;

        Ok(PhasedSample {
            root_count,
            phase_metrics: vec![
                named_metric(EVAL_PHASE_METRIC_NAME, evaluation_wall_ns),
                named_metric(BUILD_GRAPH_PHASE_METRIC_NAME, build_graph_wall_ns),
            ],
        })
    })?;

    build_result(
        &workload.descriptor,
        command_argv,
        repeat_count,
        timed.root_count,
        timed.total_wall_ns,
        timed.sample_wall_ns,
        timed.phase_metrics,
    )
}

fn benchmark_store_aware_workload(
    workload: &StoreAwareWorkload,
    repeat_count: u32,
    command_argv: &[String],
) -> Result<BenchmarkResult, Error> {
    let runtime = build_benchmark_runtime()?;
    let timed = time_repeated_operation_with_phase_metrics(repeat_count, || {
        run_store_aware_sample(&runtime, &workload.descriptor.logical_store_prefix)
    })?;

    build_result(
        &workload.descriptor,
        command_argv,
        repeat_count,
        timed.root_count,
        timed.total_wall_ns,
        timed.sample_wall_ns,
        timed.phase_metrics,
    )
}

fn benchmark_lazy_eval_workload(
    workload: &LazyEvalWorkload,
    repeat_count: u32,
    command_argv: &[String],
) -> Result<BenchmarkResult, Error> {
    use crunch_eval::session::EvaluationSession;

    let name = workload.descriptor.workload_name.as_str();
    match name {
        LAZY_DISCOVERY_WORKLOAD_NAME => {
            let timed =
                time_repeated_operation_with_phase_metrics(repeat_count, || {
                    let session = EvaluationSession::open_file(&workload.workload_path, &workload.import_paths)
                        .map_err(|e| Error::Command {
                            tool: "EvaluationSession".into(),
                            detail: e.to_string(),
                        })?;
                    let root_count = usize_to_u32(session.root_labels().len())?;
                    Ok(PhasedSample {
                        root_count,
                        phase_metrics: vec![named_metric(LAZY_DISCOVERY_METRIC_NAME, 0)],
                    })
                })?;
            build_result(
                &workload.descriptor,
                command_argv,
                repeat_count,
                timed.root_count,
                timed.total_wall_ns,
                timed.sample_wall_ns,
                vec![named_metric(LAZY_DISCOVERY_METRIC_NAME, timed.total_wall_ns)],
            )
        }
        LAZY_SELECTED_ROOT_WORKLOAD_NAME => {
            let timed = time_repeated_operation_with_phase_metrics(repeat_count, || {
                let total_start = Instant::now();
                let discovery_start = Instant::now();
                let mut session = EvaluationSession::open_file(&workload.workload_path, &workload.import_paths)
                    .map_err(|e| Error::Command {
                        tool: "EvaluationSession".into(),
                        detail: e.to_string(),
                    })?;
                let discovery_ns = duration_to_ns_u64(discovery_start.elapsed())?;
                let force_start = Instant::now();
                let _drv: CrunchDerivation =
                    session.force_root(&workload.selected_root_label).map_err(|e| Error::Command {
                        tool: "force_root".into(),
                        detail: e.to_string(),
                    })?;
                let force_ns = duration_to_ns_u64(force_start.elapsed())?;
                let total_ns = duration_to_ns_u64(total_start.elapsed())?;
                let force_count = session.explicit_force_count();
                let nonselected = force_count.saturating_sub(1);
                Ok(PhasedSample {
                    root_count: 1,
                    phase_metrics: vec![
                        named_metric(LAZY_SELECTED_ROOT_TOTAL_METRIC_NAME, total_ns),
                        named_metric(LAZY_DISCOVERY_METRIC_NAME, discovery_ns),
                        named_metric(LAZY_SELECTED_ROOT_FORCE_METRIC_NAME, force_ns),
                        named_metric(LAZY_FORCE_COUNT_METRIC_NAME, force_count as u64),
                        named_metric(LAZY_NONSELECTED_FORCE_COUNT_METRIC_NAME, nonselected as u64),
                    ],
                })
            })?;
            build_result(
                &workload.descriptor,
                command_argv,
                repeat_count,
                timed.root_count,
                timed.total_wall_ns,
                timed.sample_wall_ns,
                timed.phase_metrics,
            )
        }
        EAGER_ALL_ROOTS_WORKLOAD_NAME => {
            let timed = time_repeated_operation_with_phase_metrics(repeat_count, || {
                let mut session = EvaluationSession::open_file(&workload.workload_path, &workload.import_paths)
                    .map_err(|e| Error::Command {
                        tool: "EvaluationSession".into(),
                        detail: e.to_string(),
                    })?;
                let roots = session.force_all_roots::<CrunchDerivation>().map_err(|e| Error::Command {
                    tool: "force_all_roots".into(),
                    detail: e.to_string(),
                })?;
                let root_count = usize_to_u32(roots.len())?;
                Ok(PhasedSample {
                    root_count,
                    phase_metrics: vec![named_metric(LAZY_ALL_ROOTS_METRIC_NAME, 0)],
                })
            })?;
            build_result(
                &workload.descriptor,
                command_argv,
                repeat_count,
                timed.root_count,
                timed.total_wall_ns,
                timed.sample_wall_ns,
                vec![named_metric(LAZY_ALL_ROOTS_METRIC_NAME, timed.total_wall_ns)],
            )
        }
        PARALLEL_ALL_ROOTS_WORKLOAD_NAME => {
            let timed = time_repeated_operation_with_phase_metrics(repeat_count, || {
                let mut session = EvaluationSession::open_file(&workload.workload_path, &workload.import_paths)
                    .map_err(|e| Error::Command {
                        tool: "EvaluationSession".into(),
                        detail: e.to_string(),
                    })?;
                let roots = session
                    .force_all_roots_bounded::<CrunchDerivation>(DEFAULT_PARALLEL_ALL_ROOTS_CONCURRENCY)
                    .map_err(|e| Error::Command {
                        tool: "force_all_roots_bounded".into(),
                        detail: e.to_string(),
                    })?;
                let root_count = usize_to_u32(roots.len())?;
                Ok(PhasedSample {
                    root_count,
                    phase_metrics: vec![
                        named_metric(PARALLEL_ALL_ROOTS_METRIC_NAME, 0),
                        named_metric(
                            PARALLEL_ROOT_CONCURRENCY_METRIC_NAME,
                            DEFAULT_PARALLEL_ALL_ROOTS_CONCURRENCY as u64,
                        ),
                    ],
                })
            })?;
            build_result(
                &workload.descriptor,
                command_argv,
                repeat_count,
                timed.root_count,
                timed.total_wall_ns,
                timed.sample_wall_ns,
                vec![
                    named_metric(PARALLEL_ALL_ROOTS_METRIC_NAME, timed.total_wall_ns),
                    named_metric(PARALLEL_ROOT_CONCURRENCY_METRIC_NAME, DEFAULT_PARALLEL_ALL_ROOTS_CONCURRENCY as u64),
                ],
            )
        }
        _ => Err(Error::InvalidArgument(format!("unknown lazy-eval workload: {name}"))),
    }
}

struct TimedSamples {
    root_count: u32,
    total_wall_ns: u64,
    sample_wall_ns: Vec<u64>,
}

struct PhasedSample {
    root_count: u32,
    phase_metrics: Vec<BenchmarkMetric>,
}

struct TimedSamplesWithPhaseMetrics {
    root_count: u32,
    total_wall_ns: u64,
    sample_wall_ns: Vec<u64>,
    phase_metrics: Vec<BenchmarkMetric>,
}

fn time_repeated_operation<F>(repeat_count: u32, mut operation: F) -> Result<TimedSamples, Error>
where F: FnMut() -> Result<u32, Error> {
    if repeat_count == 0 {
        return Err(Error::InvalidArgument("repeat_count must be positive".to_string()));
    }

    let mut sample_wall_ns = Vec::with_capacity(repeat_count as usize);
    let mut root_count: Option<u32> = None;
    let started_at = Instant::now();

    for _ in 0..repeat_count {
        let sample_started_at = Instant::now();
        let sample_root_count = operation()?;
        let sample_elapsed_ns = duration_to_ns_u64(sample_started_at.elapsed())?;
        sample_wall_ns.push(sample_elapsed_ns);
        record_root_count(&mut root_count, sample_root_count)?;
    }

    Ok(TimedSamples {
        root_count: root_count.unwrap_or(0),
        total_wall_ns: duration_to_ns_u64(started_at.elapsed())?,
        sample_wall_ns,
    })
}

fn time_repeated_operation_with_phase_metrics<F>(
    repeat_count: u32,
    mut operation: F,
) -> Result<TimedSamplesWithPhaseMetrics, Error>
where
    F: FnMut() -> Result<PhasedSample, Error>,
{
    if repeat_count == 0 {
        return Err(Error::InvalidArgument("repeat_count must be positive".to_string()));
    }

    let mut sample_wall_ns = Vec::with_capacity(repeat_count as usize);
    let mut root_count: Option<u32> = None;
    let mut metric_names: Option<Vec<String>> = None;
    let mut metric_totals = std::collections::BTreeMap::new();
    let started_at = Instant::now();

    for _ in 0..repeat_count {
        let sample_started_at = Instant::now();
        let sample = operation()?;
        let sample_elapsed_ns = duration_to_ns_u64(sample_started_at.elapsed())?;
        sample_wall_ns.push(sample_elapsed_ns);
        record_root_count(&mut root_count, sample.root_count)?;
        record_phase_metric_names(&mut metric_names, &sample.phase_metrics)?;
        accumulate_phase_metric_totals(&mut metric_totals, &sample.phase_metrics)?;
    }

    let phase_metrics = build_phase_metrics_from_totals(metric_names, &metric_totals)?;
    Ok(TimedSamplesWithPhaseMetrics {
        root_count: root_count.unwrap_or(0),
        total_wall_ns: duration_to_ns_u64(started_at.elapsed())?,
        sample_wall_ns,
        phase_metrics,
    })
}

fn build_result(
    descriptor: &WorkloadDescriptor,
    command_argv: &[String],
    repeat_count: u32,
    root_count: u32,
    total_wall_ns: u64,
    sample_wall_ns: Vec<u64>,
    phase_metrics: Vec<BenchmarkMetric>,
) -> Result<BenchmarkResult, Error> {
    if command_argv.is_empty() {
        return Err(Error::InvalidArgument("command argv must not be empty".to_string()));
    }
    if sample_wall_ns.is_empty() {
        return Err(Error::InvalidArgument("sample wall times must not be empty".to_string()));
    }
    Ok(BenchmarkResult {
        workload_name: descriptor.workload_name.clone(),
        workload_kind: descriptor.workload_kind.clone(),
        workload_path: descriptor.workload_path.clone(),
        rationale: descriptor.rationale.clone(),
        operation: descriptor.operation.clone(),
        entry_point: descriptor.entry_point.clone(),
        command_argv: command_argv.to_vec(),
        cache_mode: descriptor.cache_mode.clone(),
        repeat_count,
        logical_store_prefix: descriptor.logical_store_prefix.clone(),
        hermeticity_mode: None,
        root_count,
        total_wall_ns,
        sample_wall_ns,
        phase_metrics,
        resource_metrics: unavailable_in_process_resource_metrics(),
    })
}

pub fn unavailable_in_process_resource_metrics() -> Vec<BenchmarkResourceMetric> {
    vec![
        BenchmarkResourceMetric {
            name: "cpu_time_ms".to_string(),
            unit: "milliseconds".to_string(),
            role: "observation".to_string(),
            status: "unavailable".to_string(),
            value: None,
            mechanism: "unavailable".to_string(),
            reason: Some("in-process-benchmark-has-no-operation-scoped-cpu-observation".to_string()),
        },
        BenchmarkResourceMetric {
            name: "peak_rss_bytes".to_string(),
            unit: "bytes".to_string(),
            role: "observation".to_string(),
            status: "unavailable".to_string(),
            value: None,
            mechanism: "unavailable".to_string(),
            reason: Some("in-process-benchmark-has-no-operation-scoped-peak-rss".to_string()),
        },
    ]
}

fn named_metric(name: &str, value: u64) -> BenchmarkMetric {
    BenchmarkMetric {
        name: name.to_string(),
        unit: "ns".to_string(),
        value,
    }
}

fn sum_samples_ns(sample_wall_ns: &[u64]) -> Result<u64, Error> {
    if sample_wall_ns.is_empty() {
        return Err(Error::InvalidArgument("sample wall times must not be empty".to_string()));
    }
    let mut total_ns = 0u64;
    for sample_ns in sample_wall_ns {
        total_ns = total_ns
            .checked_add(*sample_ns)
            .ok_or_else(|| Error::InvalidArgument("sample wall time sum overflowed u64".to_string()))?;
    }
    Ok(total_ns)
}

fn record_root_count(root_count: &mut Option<u32>, sample_root_count: u32) -> Result<(), Error> {
    match root_count {
        Some(existing) if *existing != sample_root_count => Err(Error::InvalidArgument(format!(
            "workload root count changed across samples: expected {existing}, got {sample_root_count}"
        ))),
        Some(_) => Ok(()),
        None => {
            *root_count = Some(sample_root_count);
            Ok(())
        }
    }
}

fn record_phase_metric_names(
    metric_names: &mut Option<Vec<String>>,
    sample_metrics: &[BenchmarkMetric],
) -> Result<(), Error> {
    let sample_names = phase_metric_names(sample_metrics)?;
    match metric_names {
        Some(existing) if *existing != sample_names => Err(Error::InvalidArgument(format!(
            "workload phase metrics changed across samples: expected {:?}, got {:?}",
            existing, sample_names
        ))),
        Some(_) => Ok(()),
        None => {
            *metric_names = Some(sample_names);
            Ok(())
        }
    }
}

fn phase_metric_names(sample_metrics: &[BenchmarkMetric]) -> Result<Vec<String>, Error> {
    if sample_metrics.is_empty() {
        return Err(Error::InvalidArgument("phase metric sample must not be empty".to_string()));
    }

    let mut names = Vec::with_capacity(sample_metrics.len());
    for metric in sample_metrics {
        if metric.name.is_empty() {
            return Err(Error::InvalidArgument("phase metric name must not be empty".to_string()));
        }
        if names.iter().any(|name| name == &metric.name) {
            return Err(Error::InvalidArgument(format!("duplicate phase metric in sample: {}", metric.name)));
        }
        names.push(metric.name.clone());
    }
    Ok(names)
}

fn accumulate_phase_metric_totals(
    metric_totals: &mut std::collections::BTreeMap<String, u64>,
    sample_metrics: &[BenchmarkMetric],
) -> Result<(), Error> {
    let _ = phase_metric_names(sample_metrics)?;
    for metric in sample_metrics {
        let total = metric_totals.entry(metric.name.clone()).or_insert(0);
        *total = total
            .checked_add(metric.value)
            .ok_or_else(|| Error::InvalidArgument(format!("phase metric sum overflowed u64: {}", metric.name)))?;
    }
    Ok(())
}

fn build_phase_metrics_from_totals(
    metric_names: Option<Vec<String>>,
    metric_totals: &std::collections::BTreeMap<String, u64>,
) -> Result<Vec<BenchmarkMetric>, Error> {
    let metric_names =
        metric_names.ok_or_else(|| Error::InvalidArgument("phase metric samples must not be empty".to_string()))?;
    let mut phase_metrics = Vec::with_capacity(metric_names.len());
    for metric_name in metric_names {
        let metric_value = metric_totals
            .get(&metric_name)
            .copied()
            .ok_or_else(|| Error::InvalidArgument(format!("missing phase metric total: {metric_name}")))?;
        phase_metrics.push(named_metric(&metric_name, metric_value));
    }
    Ok(phase_metrics)
}

fn build_benchmark_runtime() -> Result<tokio::runtime::Runtime, Error> {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|err| Error::InvalidArgument(format!("building benchmark runtime: {err}")))
}

fn run_store_aware_sample(runtime: &tokio::runtime::Runtime, store_prefix: &str) -> Result<PhasedSample, Error> {
    let state_dir = tempfile::tempdir()?;
    let output_dir = tempfile::tempdir()?;
    let output_path = benchmark_store_path()?;
    let payload_bytes = benchmark_store_payload();

    runtime.block_on(async {
        let mut store = open_benchmark_store(state_dir.path(), output_dir.path(), store_prefix).await?;
        let node = store_local_blob(&store, payload_bytes).await?;
        let path_info = benchmark_signed_pathinfo(output_path.clone(), node.clone())?;

        let persistence_started_at = Instant::now();
        let persisted_path_info = store
            .persist_and_export_signed_output(crunch_store::PersistOutputRequest {
                output_name: "out",
                output_path: &output_path,
                path_info,
                final_node: node,
                provenance: None,
                is_root: true,
                root_source: None,
            })
            .await?;
        assert_eq!(persisted_path_info.store_path, output_path, "persisted path must stay stable");
        let store_persistence_wall_ns = duration_to_ns_u64(persistence_started_at.elapsed())?;
        drop(store);

        let mut reopened = open_benchmark_store(state_dir.path(), output_dir.path(), store_prefix).await?;
        let lookup_started_at = Instant::now();
        let cached_node = reopened.cached_node_for_path(&output_path).await?;
        let store_lookup_wall_ns = duration_to_ns_u64(lookup_started_at.elapsed())?;
        if cached_node.is_none() {
            return Err(Error::InvalidArgument(format!("store-aware benchmark missed cached node: {output_path}")));
        }

        Ok(PhasedSample {
            root_count: 1,
            phase_metrics: vec![
                named_metric(STORE_PERSISTENCE_PHASE_METRIC_NAME, store_persistence_wall_ns),
                named_metric(STORE_LOOKUP_PHASE_METRIC_NAME, store_lookup_wall_ns),
            ],
        })
    })
}

async fn open_benchmark_store(
    state_dir: &Path,
    output_dir: &Path,
    store_prefix: &str,
) -> Result<crunch_store::StoreHandle, Error> {
    crunch_store::StoreHandle::open(crunch_store::StoreConfig {
        backend: crunch_store::StoreBackend::Snix,
        state_dir: state_dir.to_path_buf(),
        output_dir: output_dir.to_path_buf(),
        remote_cache_urls: Vec::new(),
        base_state_dirs: Vec::new(),
        fallback_mode: crunch_store::StoreFallbackMode::Practical,
        store_dir: store_prefix.to_string(),
    })
    .await
    .map_err(Error::from)
}

async fn store_local_blob(store: &crunch_store::StoreHandle, contents: &[u8]) -> Result<Node, Error> {
    let mut writer = store.blob_service().open_write().await;
    writer.write_all(contents).await?;
    let digest = writer.close().await.map_err(Error::Io)?;
    Ok(Node::File {
        digest,
        size: u64::try_from(contents.len())
            .map_err(|_| Error::InvalidArgument(format!("payload length does not fit in u64: {}", contents.len())))?,
        executable: false,
    })
}

fn benchmark_store_payload() -> &'static [u8] {
    b"store-aware-benchmark-payload-v1\n"
}

fn benchmark_store_path() -> Result<StorePath<String>, Error> {
    StorePath::from_name_and_digest_fixed("benchmark-store-output", [7u8; 20])
        .map_err(|err| Error::InvalidArgument(format!("building benchmark store path: {err}")))
}

fn benchmark_signed_pathinfo(
    store_path: StorePath<String>,
    node: Node,
) -> Result<snix_store::path_info::PathInfo, Error> {
    let signature = nix_compat::narinfo::Signature::<&str>::parse(
        "cache.nixos.org-1:TsTTb3WGTZKphvYdBHXwo6weVILmTytUjLB+vcX89fOjjRicCHmKA4RCPMVLkj6TMJ4GMX3HPVWRdD1hkeKZBQ==",
    )
    .map_err(|err| Error::InvalidArgument(format!("parsing benchmark signature: {err}")))?
    .to_owned();

    Ok(snix_store::path_info::PathInfo {
        store_path,
        node,
        references: Vec::new(),
        nar_size: u64::try_from(benchmark_store_payload().len()).map_err(|_| {
            Error::InvalidArgument(format!("payload length does not fit in u64: {}", benchmark_store_payload().len()))
        })?,
        nar_sha256: [3u8; 32],
        signatures: vec![signature],
        deriver: None,
        ca: None,
    })
}

fn collect_toolchain_context() -> Result<ToolchainContext, Error> {
    let rustc_version = run_command_and_capture("rustc", &["--version", "--verbose"])?;
    let cargo_tool = std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
    let cargo_version = run_command_and_capture(&cargo_tool, &["--version"])?;
    Ok(ToolchainContext {
        rustc_version,
        cargo_version,
    })
}

fn collect_host_context() -> HostContext {
    HostContext {
        os: std::env::consts::OS.to_string(),
        arch: std::env::consts::ARCH.to_string(),
        hostname: host_name(),
    }
}

fn git_commit(repo_root: &Path) -> Result<String, Error> {
    if !repo_root.join(".git").exists() {
        return Ok(std::env::var("MANTLE_BENCHMARK_SOURCE_COMMIT").unwrap_or_else(|_| "nix-source".to_string()));
    }
    run_command_and_capture("git", &[
        "-C",
        &repo_root.display().to_string(),
        "rev-parse",
        "--short=12",
        "HEAD",
    ])
}

fn git_is_dirty(repo_root: &Path) -> Result<bool, Error> {
    if !repo_root.join(".git").exists() {
        return Ok(false);
    }
    let status = run_command_and_capture_allowing_empty("git", &[
        "-C",
        &repo_root.display().to_string(),
        "status",
        "--short",
        "--untracked-files=no",
    ])?;
    Ok(!status.is_empty())
}

fn host_name() -> Option<String> {
    if let Ok(hostname) = std::env::var("HOSTNAME") {
        let trimmed = hostname.trim();
        if !trimmed.is_empty() {
            return Some(trimmed.to_string());
        }
    }

    std::fs::read_to_string("/proc/sys/kernel/hostname")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn unix_timestamp_now() -> Result<u64, Error> {
    let duration = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|err| Error::InvalidArgument(format!("system clock before unix epoch: {err}")))?;
    Ok(duration.as_secs())
}

fn run_command_and_capture(tool: &str, args: &[&str]) -> Result<String, Error> {
    let stdout = run_command_and_capture_allowing_empty(tool, args)?;
    if stdout.is_empty() {
        return Err(Error::Command {
            tool: tool.to_string(),
            detail: format!("command returned empty stdout for args {args:?}"),
        });
    }
    Ok(stdout)
}

fn run_command_and_capture_allowing_empty(tool: &str, args: &[&str]) -> Result<String, Error> {
    if tool.is_empty() {
        return Err(Error::InvalidArgument("tool must not be empty".to_string()));
    }
    let output = Command::new(tool).args(args).output().map_err(|err| Error::Command {
        tool: tool.to_string(),
        detail: err.to_string(),
    })?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        return Err(Error::Command {
            tool: tool.to_string(),
            detail: format!("status={} stdout={stdout:?} stderr={stderr:?}", output.status),
        });
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn duration_to_ns_u64(duration: std::time::Duration) -> Result<u64, Error> {
    let nanos = duration.as_nanos();
    u64::try_from(nanos).map_err(|_| Error::InvalidArgument(format!("duration overflowed u64 nanoseconds: {nanos}")))
}

fn usize_to_u32(value: usize) -> Result<u32, Error> {
    u32::try_from(value).map_err(|_| Error::InvalidArgument(format!("value does not fit in u32: {value}")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_benchmark_paths_stay_under_target() {
        let smoke_path = default_eval_smoke_bundle_path();
        let suite_path = default_suite_bundle_path();
        assert!(smoke_path.ends_with(Path::new("target/benchmarks/eval-smoke.json")));
        assert!(suite_path.ends_with(Path::new("target/benchmarks/suite.json")));
    }

    #[test]
    fn suite_workload_descriptors_cover_required_kinds() {
        let descriptors = suite_workload_descriptors().unwrap();
        let names: Vec<String> = descriptors.iter().map(|d| d.workload_name.clone()).collect();
        let kinds: Vec<String> = descriptors.iter().map(|d| d.workload_kind.clone()).collect();
        assert_eq!(descriptors.len(), 10);
        assert!(names.contains(&EVAL_SMOKE_WORKLOAD_NAME.to_string()));
        assert!(names.contains(&CONVERSION_WORKLOAD_NAME.to_string()));
        assert!(names.contains(&SUBSTITUTION_WORKLOAD_NAME.to_string()));
        assert!(names.contains(&BUILD_GRAPH_WORKLOAD_NAME.to_string()));
        assert!(names.contains(&MULTI_PHASE_WORKFLOW_WORKLOAD_NAME.to_string()));
        assert!(names.contains(&STORE_AWARE_WORKLOAD_NAME.to_string()));
        assert!(names.contains(&LAZY_DISCOVERY_WORKLOAD_NAME.to_string()));
        assert!(names.contains(&LAZY_SELECTED_ROOT_WORKLOAD_NAME.to_string()));
        assert!(names.contains(&EAGER_ALL_ROOTS_WORKLOAD_NAME.to_string()));
        assert!(names.contains(&PARALLEL_ALL_ROOTS_WORKLOAD_NAME.to_string()));
        assert!(kinds.contains(&"evaluation".to_string()));
        assert!(kinds.contains(&"conversion".to_string()));
        assert!(kinds.contains(&"substitution".to_string()));
        assert!(kinds.contains(&"build-graph".to_string()));
        assert!(kinds.contains(&"workflow".to_string()));
        assert!(kinds.contains(&"store".to_string()));
        assert!(kinds.contains(&"lazy-eval".to_string()));
    }

    #[test]
    fn suite_workload_setup_is_deterministic() {
        let left = suite_workload_descriptors().unwrap();
        let right = suite_workload_descriptors().unwrap();
        assert_eq!(left, right);
    }

    #[test]
    fn sum_samples_ns_adds_all_samples() {
        let total_ns = sum_samples_ns(&[11, 13, 17]).unwrap();
        assert_eq!(total_ns, 41);
    }

    #[test]
    fn record_root_count_rejects_drift() {
        let mut root_count = None;
        record_root_count(&mut root_count, 1).unwrap();
        let err = record_root_count(&mut root_count, 2).unwrap_err();
        assert!(format!("{err}").contains("root count changed"));
    }

    #[test]
    fn render_bundle_json_keeps_schema() {
        let bundle =
            fixture_bundle("abc123", vec![fixture_result(EVAL_SMOKE_WORKLOAD_NAME, EVAL_PHASE_METRIC_NAME, 7, 7)]);
        let json = render_bundle_json(&bundle).unwrap();
        assert!(json.contains(BENCHMARK_BUNDLE_SCHEMA_V1));
        assert!(json.contains(EVAL_SMOKE_WORKLOAD_NAME));
    }

    #[test]
    fn compare_bundles_matches_workloads_and_metrics() {
        let baseline = fixture_bundle("baseline", vec![
            fixture_result(EVAL_SMOKE_WORKLOAD_NAME, EVAL_PHASE_METRIC_NAME, 100, 90),
            fixture_result(CONVERSION_WORKLOAD_NAME, CONVERSION_PHASE_METRIC_NAME, 60, 50),
        ]);
        let fresh = fixture_bundle("fresh", vec![
            fixture_result(EVAL_SMOKE_WORKLOAD_NAME, EVAL_PHASE_METRIC_NAME, 120, 100),
            fixture_result(CONVERSION_WORKLOAD_NAME, CONVERSION_PHASE_METRIC_NAME, 55, 45),
            fixture_result(SUBSTITUTION_WORKLOAD_NAME, SUBSTITUTION_PHASE_METRIC_NAME, 20, 18),
        ]);

        let report =
            compare_bundles(&baseline, &fresh, "baseline.json", "fresh.json", &default_compare_thresholds()).unwrap();

        assert_eq!(report.matched_workloads.len(), 2);
        assert_eq!(report.missing_from_baseline, vec![SUBSTITUTION_WORKLOAD_NAME.to_string()]);
        assert!(report.missing_from_fresh.is_empty());
        assert_eq!(report.matched_workloads[0].workload_name, CONVERSION_WORKLOAD_NAME);
        assert!(
            report.matched_workloads[0]
                .matched_metrics
                .iter()
                .any(|metric| metric.metric_name == CONVERSION_PHASE_METRIC_NAME)
        );
        assert!(
            report.matched_workloads[1]
                .matched_metrics
                .iter()
                .any(|metric| metric.metric_name == TOTAL_PHASE_METRIC_NAME)
        );
    }

    #[test]
    fn compare_bundles_highlight_largest_regressions_and_wins() {
        let baseline = fixture_bundle("baseline", vec![
            fixture_result(EVAL_SMOKE_WORKLOAD_NAME, EVAL_PHASE_METRIC_NAME, 100, 90),
            fixture_result(CONVERSION_WORKLOAD_NAME, CONVERSION_PHASE_METRIC_NAME, 60, 50),
        ]);
        let fresh = fixture_bundle("fresh", vec![
            fixture_result(EVAL_SMOKE_WORKLOAD_NAME, EVAL_PHASE_METRIC_NAME, 135, 120),
            fixture_result(CONVERSION_WORKLOAD_NAME, CONVERSION_PHASE_METRIC_NAME, 45, 35),
        ]);
        let thresholds = CompareThresholds {
            absolute_threshold_ns: 10,
            percent_threshold: 20.0,
            named_absolute_thresholds: std::collections::BTreeMap::new(),
            named_percent_thresholds: std::collections::BTreeMap::new(),
        };

        let report = compare_bundles(&baseline, &fresh, "baseline.json", "fresh.json", &thresholds).unwrap();
        let human = render_comparison_human(&report);

        assert_eq!(report.largest_regression.as_ref().unwrap().workload_name, EVAL_SMOKE_WORKLOAD_NAME);
        assert_eq!(report.largest_win.as_ref().unwrap().workload_name, CONVERSION_WORKLOAD_NAME);
        assert!(report.largest_regression.as_ref().unwrap().exceeds_absolute_threshold);
        assert!(report.largest_win.as_ref().unwrap().exceeds_percent_threshold);
        assert!(human.contains("largest regression"));
        assert!(human.contains("largest win"));
        assert!(human.contains(EVAL_SMOKE_WORKLOAD_NAME));
        assert!(human.contains(CONVERSION_WORKLOAD_NAME));
    }

    #[test]
    fn compare_benchmark_files_reads_fixed_fixtures() {
        let temp_dir = tempfile::tempdir().unwrap();
        let baseline_path = temp_dir.path().join("baseline.json");
        let fresh_path = temp_dir.path().join("fresh.json");
        let baseline = fixture_bundle("baseline", vec![fixture_result(
            EVAL_SMOKE_WORKLOAD_NAME,
            EVAL_PHASE_METRIC_NAME,
            100,
            90,
        )]);
        let fresh = fixture_bundle("fresh", vec![fixture_result(
            EVAL_SMOKE_WORKLOAD_NAME,
            EVAL_PHASE_METRIC_NAME,
            120,
            95,
        )]);
        std::fs::write(&baseline_path, render_bundle_json(&baseline).unwrap()).unwrap();
        std::fs::write(&fresh_path, render_bundle_json(&fresh).unwrap()).unwrap();

        let report = compare_benchmark_files(&baseline_path, &fresh_path, &default_compare_thresholds()).unwrap();

        assert_eq!(report.baseline_path, baseline_path.display().to_string());
        assert_eq!(report.fresh_path, fresh_path.display().to_string());
        assert_eq!(report.matched_workloads.len(), 1);
        assert_eq!(report.matched_workloads[0].matched_metrics.len(), 2);
    }

    #[test]
    fn build_result_allows_workloads_without_phase_metrics() {
        let descriptor = fixture_descriptor("opaque-workload");
        let result = build_result(&descriptor, &["bench".to_string()], 1, 1, 25, vec![25], Vec::new()).unwrap();

        assert!(result.phase_metrics.is_empty());
        assert_eq!(result.total_wall_ns, 25);
        let bundle = fixture_bundle("opaque", vec![result]);
        let json = render_bundle_json(&bundle).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["results"][0]["phase_metrics"], serde_json::json!([]));
    }

    #[test]
    fn compare_bundles_do_not_fabricate_missing_phase_metrics() {
        let baseline = fixture_bundle("baseline", vec![fixture_result_without_phase_metric("opaque-workload", 40)]);
        let fresh = fixture_bundle("fresh", vec![fixture_result_without_phase_metric("opaque-workload", 45)]);

        let report =
            compare_bundles(&baseline, &fresh, "baseline.json", "fresh.json", &default_compare_thresholds()).unwrap();
        let workload = &report.matched_workloads[0];

        assert_eq!(workload.matched_metrics.len(), 1);
        assert_eq!(workload.matched_metrics[0].metric_name, TOTAL_PHASE_METRIC_NAME);
        assert!(workload.missing_from_baseline.is_empty());
        assert!(workload.missing_from_fresh.is_empty());
    }

    #[test]
    fn compare_bundles_report_sparse_metric_omissions_per_workload() {
        let baseline = fixture_bundle("baseline", vec![fixture_multi_metric_result(
            MULTI_PHASE_WORKFLOW_WORKLOAD_NAME,
            vec![(EVAL_PHASE_METRIC_NAME, 100), (BUILD_GRAPH_PHASE_METRIC_NAME, 60)],
            170,
        )]);
        let fresh = fixture_bundle("fresh", vec![fixture_multi_metric_result(
            MULTI_PHASE_WORKFLOW_WORKLOAD_NAME,
            vec![
                (EVAL_PHASE_METRIC_NAME, 110),
                (BUILD_GRAPH_PHASE_METRIC_NAME, 55),
                (STORE_LOOKUP_PHASE_METRIC_NAME, 9),
            ],
            180,
        )]);

        let report =
            compare_bundles(&baseline, &fresh, "baseline.json", "fresh.json", &default_compare_thresholds()).unwrap();
        let workload = &report.matched_workloads[0];

        assert_eq!(workload.workload_name, MULTI_PHASE_WORKFLOW_WORKLOAD_NAME);
        assert_eq!(workload.matched_metrics.len(), 3);
        assert_eq!(workload.missing_from_baseline, vec![STORE_LOOKUP_PHASE_METRIC_NAME.to_string()]);
        assert!(workload.missing_from_fresh.is_empty());
    }

    #[test]
    fn compare_rejects_threshold_application_across_mismatched_resource_cohorts() {
        let baseline = fixture_bundle("baseline", vec![fixture_result(
            EVAL_SMOKE_WORKLOAD_NAME,
            EVAL_PHASE_METRIC_NAME,
            100,
            90,
        )]);
        let mut fresh = fixture_bundle("fresh", vec![fixture_result(
            EVAL_SMOKE_WORKLOAD_NAME,
            EVAL_PHASE_METRIC_NAME,
            200,
            180,
        )]);
        fresh.resource_cohort.host_class = "different-host".to_string();
        let report =
            compare_bundles(&baseline, &fresh, "baseline.json", "fresh.json", &default_compare_thresholds()).unwrap();
        assert!(!report.resource_cohort_compatible);
        assert_eq!(report.resource_cohort_mismatches, vec!["host-class".to_string()]);
        assert!(report.matched_workloads[0].matched_metrics.iter().all(|metric| !metric.threshold_applied));
        assert!(report.matched_workloads[0].matched_metrics.iter().all(|metric| !metric.exceeds_absolute_threshold));
    }

    #[test]
    fn compatible_cohorts_apply_named_memory_thresholds_and_report_missing_values() {
        let mut baseline_result = fixture_result(EVAL_SMOKE_WORKLOAD_NAME, EVAL_PHASE_METRIC_NAME, 100, 90);
        baseline_result.resource_metrics = vec![fixture_peak_rss_metric(Some(100))];
        let mut fresh_result = fixture_result(EVAL_SMOKE_WORKLOAD_NAME, EVAL_PHASE_METRIC_NAME, 110, 95);
        fresh_result.resource_metrics = vec![fixture_peak_rss_metric(Some(150))];
        let baseline = fixture_bundle("baseline", vec![baseline_result]);
        let fresh = fixture_bundle("fresh", vec![fresh_result]);
        let mut thresholds = default_compare_thresholds();
        thresholds.named_absolute_thresholds.insert("peak_rss_bytes".to_string(), 40);
        thresholds.named_percent_thresholds.insert("peak_rss_bytes".to_string(), 20.0);
        let report = compare_bundles(&baseline, &fresh, "baseline.json", "fresh.json", &thresholds).unwrap();
        assert!(report.resource_cohort_compatible);
        let memory = report.matched_workloads[0]
            .matched_metrics
            .iter()
            .find(|metric| metric.metric_name == "peak_rss_bytes")
            .unwrap();
        assert!(memory.threshold_applied);
        assert_eq!(memory.absolute_threshold, 40);
        assert!(memory.exceeds_absolute_threshold);
        assert!(memory.exceeds_percent_threshold);

        let mut missing_fresh = fresh.clone();
        missing_fresh.results[0].resource_metrics = vec![fixture_peak_rss_metric(None)];
        let missing = compare_bundles(&baseline, &missing_fresh, "baseline.json", "fresh.json", &thresholds).unwrap();
        assert_eq!(missing.matched_workloads[0].missing_from_fresh, vec!["peak_rss_bytes".to_string()]);
    }

    fn fixture_peak_rss_metric(value: Option<u64>) -> BenchmarkResourceMetric {
        BenchmarkResourceMetric {
            name: "peak_rss_bytes".to_string(),
            unit: "bytes".to_string(),
            role: "observation".to_string(),
            status: if value.is_some() { "observed" } else { "unavailable" }.to_string(),
            value,
            mechanism: "fixture-rusage".to_string(),
            reason: value.is_none().then(|| "fixture-missing".to_string()),
        }
    }

    fn fixture_bundle(commit: &str, results: Vec<BenchmarkResult>) -> BenchmarkBundle {
        BenchmarkBundle {
            schema: BENCHMARK_BUNDLE_SCHEMA_V1.to_string(),
            generated_unix_s: 1,
            resource_cohort: fixture_resource_cohort(),
            repo_root: "/tmp/repo".to_string(),
            bundle_path: "/tmp/repo/target/benchmarks/suite.json".to_string(),
            commit: commit.to_string(),
            git_dirty: false,
            toolchain: ToolchainContext {
                rustc_version: "rustc 1".to_string(),
                cargo_version: "cargo 1".to_string(),
            },
            host: HostContext {
                os: "linux".to_string(),
                arch: "x86_64".to_string(),
                hostname: Some("host".to_string()),
            },
            results,
        }
    }

    fn fixture_result(
        workload_name: &str,
        phase_metric_name: &str,
        total_wall_ns: u64,
        phase_wall_ns: u64,
    ) -> BenchmarkResult {
        BenchmarkResult {
            workload_name: workload_name.to_string(),
            workload_kind: "fixture".to_string(),
            workload_path: "fixture.ncl".to_string(),
            rationale: "fixture".to_string(),
            operation: "fixture".to_string(),
            entry_point: COMPARE_ENTRY_POINT.to_string(),
            command_argv: vec!["compare".to_string()],
            cache_mode: "fixture".to_string(),
            repeat_count: 1,
            logical_store_prefix: DEFAULT_LOGICAL_STORE_PREFIX.to_string(),
            hermeticity_mode: None,
            root_count: 1,
            total_wall_ns,
            sample_wall_ns: vec![phase_wall_ns],
            phase_metrics: vec![named_metric(phase_metric_name, phase_wall_ns)],
            resource_metrics: unavailable_in_process_resource_metrics(),
        }
    }

    fn fixture_result_without_phase_metric(workload_name: &str, total_wall_ns: u64) -> BenchmarkResult {
        BenchmarkResult {
            workload_name: workload_name.to_string(),
            workload_kind: "fixture".to_string(),
            workload_path: "fixture.ncl".to_string(),
            rationale: "fixture".to_string(),
            operation: "fixture".to_string(),
            entry_point: COMPARE_ENTRY_POINT.to_string(),
            command_argv: vec!["compare".to_string()],
            cache_mode: "fixture".to_string(),
            repeat_count: 1,
            logical_store_prefix: DEFAULT_LOGICAL_STORE_PREFIX.to_string(),
            hermeticity_mode: None,
            root_count: 1,
            total_wall_ns,
            sample_wall_ns: vec![total_wall_ns],
            phase_metrics: Vec::new(),
            resource_metrics: unavailable_in_process_resource_metrics(),
        }
    }

    fn fixture_multi_metric_result(
        workload_name: &str,
        phase_metrics: Vec<(&str, u64)>,
        total_wall_ns: u64,
    ) -> BenchmarkResult {
        BenchmarkResult {
            workload_name: workload_name.to_string(),
            workload_kind: "fixture".to_string(),
            workload_path: "fixture.ncl".to_string(),
            rationale: "fixture".to_string(),
            operation: "fixture".to_string(),
            entry_point: COMPARE_ENTRY_POINT.to_string(),
            command_argv: vec!["compare".to_string()],
            cache_mode: "fixture".to_string(),
            repeat_count: 1,
            logical_store_prefix: DEFAULT_LOGICAL_STORE_PREFIX.to_string(),
            hermeticity_mode: None,
            root_count: 1,
            total_wall_ns,
            sample_wall_ns: vec![total_wall_ns],
            phase_metrics: phase_metrics.into_iter().map(|(name, value)| named_metric(name, value)).collect(),
            resource_metrics: unavailable_in_process_resource_metrics(),
        }
    }

    fn fixture_resource_cohort() -> EvaluationResourceCohort {
        let unavailable = MeasurementSupport {
            status: "unavailable".to_string(),
            mechanism: "unavailable".to_string(),
            reason: Some("fixture-unavailable".to_string()),
        };
        EvaluationResourceCohort {
            host_class: "linux-x86_64".to_string(),
            target: "linux-x86_64".to_string(),
            evaluator_id: "nickel-lang".to_string(),
            evaluator_version: "2.0.0".to_string(),
            toolchain_id: "fixture-toolchain".to_string(),
            policy_ref: "fixture-policy".to_string(),
            fixture_set_id: "fixture-set".to_string(),
            repeat_count: 1,
            warm_state: "fixture".to_string(),
            cpu_time_support: unavailable.clone(),
            peak_rss_support: unavailable,
        }
    }

    fn fixture_descriptor(workload_name: &str) -> WorkloadDescriptor {
        WorkloadDescriptor {
            workload_name: workload_name.to_string(),
            workload_kind: "fixture".to_string(),
            workload_path: "fixture.ncl".to_string(),
            rationale: "fixture".to_string(),
            operation: "fixture".to_string(),
            entry_point: COMPARE_ENTRY_POINT.to_string(),
            cache_mode: "fixture".to_string(),
            logical_store_prefix: DEFAULT_LOGICAL_STORE_PREFIX.to_string(),
        }
    }
}
