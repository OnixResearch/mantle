//! Lazy root evaluation benchmark.
//!
//! Measures lazy root discovery, selected-root forcing, and eager all-roots
//! baseline on the wide_package_set fixture.

#[path = "benchmark_support.rs"]
mod benchmark_support;

use std::ffi::OsString;
use std::path::Path;
use std::path::PathBuf;
use std::time::Instant;

use benchmark_support::BenchmarkMetric;
use benchmark_support::BenchmarkResult;
use benchmark_support::Error;
use crunch_eval::session::EvaluationSession;
use crunch_glue::CrunchDerivation;

pub const LAZY_DISCOVERY_WORKLOAD: &str = "lazy-root-discovery-wide-package-set";
pub const LAZY_SELECTED_ROOT_WORKLOAD: &str = "lazy-selected-root-wide-package-set";
pub const EAGER_ALL_ROOTS_WORKLOAD: &str = "eager-all-roots-wide-package-set";
pub const PARALLEL_ALL_ROOTS_WORKLOAD: &str = "parallel-all-roots-wide-package-set";

const WIDE_FIXTURE_PATH: &str = "tests/fixtures/wide_package_set.ncl";
const SELECTED_ROOT_LABEL: &str = "alpha";
const DEFAULT_REPEAT_COUNT: u32 = 10;
const DEFAULT_PARALLEL_ALL_ROOTS_CONCURRENCY: u32 = 4;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn fixture_path() -> PathBuf {
    repo_root().join(WIDE_FIXTURE_PATH)
}

fn stdlib_import_path() -> OsString {
    crunch_eval::stdlib::stdlib_import_path().expect("stdlib import path").into_os_string()
}

fn import_paths() -> Vec<OsString> {
    vec![stdlib_import_path()]
}

struct LazyMetrics {
    root_discovery_wall_ns: u64,
    selected_root_force_wall_ns: u64,
    selected_root_total_wall_ns: u64,
    explicit_top_level_root_force_count: u32,
    explicit_nonselected_root_force_count: u32,
}

fn measure_lazy_discovery(path: &Path, import_paths: &[OsString]) -> Result<(u64, u32), Error> {
    let start = Instant::now();
    let session = EvaluationSession::open_file(path, import_paths)?;
    let elapsed_ns = start.elapsed().as_nanos() as u64;
    let root_count = session.root_labels().len() as u32;
    Ok((elapsed_ns, root_count))
}

fn measure_lazy_selected_root(path: &Path, import_paths: &[OsString], label: &str) -> Result<LazyMetrics, Error> {
    let total_start = Instant::now();

    let discovery_start = Instant::now();
    let mut session = EvaluationSession::open_file(path, import_paths)?;
    let discovery_ns = discovery_start.elapsed().as_nanos() as u64;

    let force_start = Instant::now();
    let _drv: CrunchDerivation = session.force_root(label)?;
    let force_ns = force_start.elapsed().as_nanos() as u64;

    let total_ns = total_start.elapsed().as_nanos() as u64;

    let force_count = session.explicit_force_count();
    // When forcing one root, the session deep-exports all roots (Nickel limitation).
    // The explicit force count is 1 — we only asked for one root.
    // Nonselected = total forced - 1 (the selected one).
    let nonselected = force_count.saturating_sub(1);

    Ok(LazyMetrics {
        root_discovery_wall_ns: discovery_ns,
        selected_root_force_wall_ns: force_ns,
        selected_root_total_wall_ns: total_ns,
        explicit_top_level_root_force_count: force_count,
        explicit_nonselected_root_force_count: nonselected,
    })
}

fn measure_eager_all_roots(path: &Path, import_paths: &[OsString]) -> Result<(u64, u32), Error> {
    let start = Instant::now();
    let mut session = EvaluationSession::open_file(path, import_paths)?;
    let roots = session.force_all_roots::<CrunchDerivation>()?;
    let elapsed_ns = start.elapsed().as_nanos() as u64;
    Ok((elapsed_ns, roots.len() as u32))
}

fn measure_parallel_all_roots(path: &Path, import_paths: &[OsString], concurrency: u32) -> Result<(u64, u32), Error> {
    let start = Instant::now();
    let session = EvaluationSession::open_file(path, import_paths)?;
    let roots = session.force_all_roots_bounded::<CrunchDerivation>(concurrency)?;
    let elapsed_ns = start.elapsed().as_nanos() as u64;
    Ok((elapsed_ns, roots.len() as u32))
}

fn median(samples: &[u64]) -> u64 {
    assert!(!samples.is_empty(), "need at least one sample");
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    sorted[sorted.len() / 2]
}

fn run_lazy_benchmarks(repeat_count: u32) -> Result<Vec<BenchmarkResult>, Error> {
    let path = fixture_path();
    let imports = import_paths();
    let mut results = Vec::with_capacity(4);

    // 1. Lazy root discovery
    {
        let mut samples = Vec::with_capacity(repeat_count as usize);
        let mut root_count = 0u32;
        for _ in 0..repeat_count {
            let (ns, count) = measure_lazy_discovery(&path, &imports)?;
            samples.push(ns);
            root_count = count;
        }
        let med = median(&samples);
        results.push(BenchmarkResult {
            workload_name: LAZY_DISCOVERY_WORKLOAD.to_string(),
            workload_kind: "lazy-eval".to_string(),
            workload_path: WIDE_FIXTURE_PATH.to_string(),
            rationale: "Measures lazy root label discovery without deep-forcing any root values".to_string(),
            operation: "open_file + root_labels".to_string(),
            entry_point: "benchmark_lazy_eval".to_string(),
            command_argv: std::env::args().collect(),
            cache_mode: "cold-per-sample".to_string(),
            repeat_count,
            logical_store_prefix: "/nix/store".to_string(),
            hermeticity_mode: None,
            root_count,
            total_wall_ns: med,
            sample_wall_ns: samples,
            phase_metrics: vec![BenchmarkMetric {
                name: "root_discovery_wall_ns".to_string(),
                unit: "ns".to_string(),
                value: med,
            }],
        });
    }

    // 2. Lazy selected-root forcing
    {
        let mut samples_total = Vec::with_capacity(repeat_count as usize);
        let mut samples_discovery = Vec::with_capacity(repeat_count as usize);
        let mut samples_force = Vec::with_capacity(repeat_count as usize);
        let mut last_metrics: Option<LazyMetrics> = None;
        for _ in 0..repeat_count {
            let m = measure_lazy_selected_root(&path, &imports, SELECTED_ROOT_LABEL)?;
            samples_total.push(m.selected_root_total_wall_ns);
            samples_discovery.push(m.root_discovery_wall_ns);
            samples_force.push(m.selected_root_force_wall_ns);
            last_metrics = Some(m);
        }
        let lm = last_metrics.expect("at least one sample");
        let med_total = median(&samples_total);
        results.push(BenchmarkResult {
            workload_name: LAZY_SELECTED_ROOT_WORKLOAD.to_string(),
            workload_kind: "lazy-eval".to_string(),
            workload_path: WIDE_FIXTURE_PATH.to_string(),
            rationale: "Measures end-to-end latency to obtain one selected root from a wide package set".to_string(),
            operation: format!("open_file + force_root(\"{}\")", SELECTED_ROOT_LABEL),
            entry_point: "benchmark_lazy_eval".to_string(),
            command_argv: std::env::args().collect(),
            cache_mode: "cold-per-sample".to_string(),
            repeat_count,
            logical_store_prefix: "/nix/store".to_string(),
            hermeticity_mode: None,
            root_count: 1,
            total_wall_ns: med_total,
            sample_wall_ns: samples_total,
            phase_metrics: vec![
                BenchmarkMetric {
                    name: "selected_root_total_wall_ns".to_string(),
                    unit: "ns".to_string(),
                    value: med_total,
                },
                BenchmarkMetric {
                    name: "root_discovery_wall_ns".to_string(),
                    unit: "ns".to_string(),
                    value: median(&samples_discovery),
                },
                BenchmarkMetric {
                    name: "selected_root_force_wall_ns".to_string(),
                    unit: "ns".to_string(),
                    value: median(&samples_force),
                },
                BenchmarkMetric {
                    name: "explicit_top_level_root_force_count".to_string(),
                    unit: "count".to_string(),
                    value: lm.explicit_top_level_root_force_count as u64,
                },
                BenchmarkMetric {
                    name: "explicit_nonselected_root_force_count".to_string(),
                    unit: "count".to_string(),
                    value: lm.explicit_nonselected_root_force_count as u64,
                },
            ],
        });
    }

    // 3. Eager all-roots guardrail
    {
        let mut samples = Vec::with_capacity(repeat_count as usize);
        let mut root_count = 0u32;
        for _ in 0..repeat_count {
            let (ns, count) = measure_eager_all_roots(&path, &imports)?;
            samples.push(ns);
            root_count = count;
        }
        let med = median(&samples);
        results.push(BenchmarkResult {
            workload_name: EAGER_ALL_ROOTS_WORKLOAD.to_string(),
            workload_kind: "lazy-eval".to_string(),
            workload_path: WIDE_FIXTURE_PATH.to_string(),
            rationale: "Guardrail: measures all-roots eager path to detect regressions from lazy changes".to_string(),
            operation: "open_file + force_all_roots".to_string(),
            entry_point: "benchmark_lazy_eval".to_string(),
            command_argv: std::env::args().collect(),
            cache_mode: "cold-per-sample".to_string(),
            repeat_count,
            logical_store_prefix: "/nix/store".to_string(),
            hermeticity_mode: None,
            root_count,
            total_wall_ns: med,
            sample_wall_ns: samples,
            phase_metrics: vec![BenchmarkMetric {
                name: "all_roots_total_wall_ns".to_string(),
                unit: "ns".to_string(),
                value: med,
            }],
        });
    }

    // 4. Parallel all-roots throughput path
    {
        let mut samples = Vec::with_capacity(repeat_count as usize);
        let mut root_count = 0u32;
        for _ in 0..repeat_count {
            let (ns, count) = measure_parallel_all_roots(&path, &imports, DEFAULT_PARALLEL_ALL_ROOTS_CONCURRENCY)?;
            samples.push(ns);
            root_count = count;
        }
        let med = median(&samples);
        results.push(BenchmarkResult {
            workload_name: PARALLEL_ALL_ROOTS_WORKLOAD.to_string(),
            workload_kind: "lazy-eval".to_string(),
            workload_path: WIDE_FIXTURE_PATH.to_string(),
            rationale: "Measures bounded parallel all-roots forcing throughput on the same wide package-set fixture"
                .to_string(),
            operation: format!(
                "open_file + force_all_roots_bounded(concurrency={DEFAULT_PARALLEL_ALL_ROOTS_CONCURRENCY})"
            ),
            entry_point: "benchmark_lazy_eval".to_string(),
            command_argv: std::env::args().collect(),
            cache_mode: "cold-per-sample isolated benchmark run".to_string(),
            repeat_count,
            logical_store_prefix: "/nix/store".to_string(),
            hermeticity_mode: None,
            root_count,
            total_wall_ns: med,
            sample_wall_ns: samples,
            phase_metrics: vec![
                BenchmarkMetric {
                    name: "parallel_all_roots_total_wall_ns".to_string(),
                    unit: "ns".to_string(),
                    value: med,
                },
                BenchmarkMetric {
                    name: "parallel_root_eval_concurrency".to_string(),
                    unit: "count".to_string(),
                    value: DEFAULT_PARALLEL_ALL_ROOTS_CONCURRENCY as u64,
                },
            ],
        });
    }

    Ok(results)
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let repeat_count = args
        .iter()
        .position(|a| a == "--repeat-count")
        .and_then(|i| args.get(i + 1))
        .and_then(|v| v.parse().ok())
        .unwrap_or(DEFAULT_REPEAT_COUNT);

    let bundle_out = args.iter().position(|a| a == "--bundle-out").and_then(|i| args.get(i + 1).cloned());

    let results = run_lazy_benchmarks(repeat_count).unwrap_or_else(|e| {
        eprintln!("benchmark failed: {e}");
        std::process::exit(1);
    });

    if let Some(out_path) = bundle_out {
        let json = serde_json::to_string_pretty(&results).unwrap();
        std::fs::write(&out_path, &json).unwrap();
        eprintln!("wrote benchmark results to {out_path}");
    } else {
        // Print human-readable summary to stdout
        for result in &results {
            println!("workload: {}", result.workload_name);
            println!("  total_wall_ns: {} (median of {} samples)", result.total_wall_ns, result.repeat_count);
            for metric in &result.phase_metrics {
                println!("  {}: {} {}", metric.name, metric.value, metric.unit);
            }
            println!();
        }
    }
}
