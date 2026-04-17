#[path = "../examples/benchmark_support.rs"]
mod benchmark_support;

use std::path::PathBuf;

use benchmark_support::BENCHMARK_BUNDLE_SCHEMA_V1;
use benchmark_support::BUILD_GRAPH_PHASE_METRIC_NAME;
use benchmark_support::BUILD_GRAPH_WORKLOAD_NAME;
use benchmark_support::CONVERSION_PHASE_METRIC_NAME;
use benchmark_support::CONVERSION_WORKLOAD_NAME;
use benchmark_support::DEFAULT_LOGICAL_STORE_PREFIX;
use benchmark_support::DEFAULT_REPEAT_COUNT;
use benchmark_support::EVAL_PHASE_METRIC_NAME;
use benchmark_support::EVAL_SMOKE_ENTRY_POINT;
use benchmark_support::EVAL_SMOKE_WORKLOAD_NAME;
use benchmark_support::MULTI_PHASE_WORKFLOW_WORKLOAD_NAME;
use benchmark_support::STORE_AWARE_WORKLOAD_NAME;
use benchmark_support::STORE_LOOKUP_PHASE_METRIC_NAME;
use benchmark_support::STORE_PERSISTENCE_PHASE_METRIC_NAME;
use benchmark_support::SUBSTITUTION_PHASE_METRIC_NAME;
use benchmark_support::SUBSTITUTION_WORKLOAD_NAME;
use benchmark_support::SUITE_ENTRY_POINT;
use benchmark_support::TOTAL_PHASE_METRIC_NAME;
use benchmark_support::eval_smoke_request;
use benchmark_support::run_eval_smoke_benchmark;
use benchmark_support::run_suite_benchmark;
use benchmark_support::suite_request;
use benchmark_support::suite_workload_descriptors;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn toml_section(source: &str, section_name: &str) -> String {
    let header = format!("[{section_name}]");
    let subsection_prefix = format!("[{section_name}.");
    let mut in_section = false;
    let mut body_lines = Vec::new();

    for line in source.lines() {
        let trimmed = line.trim();
        let is_header = trimmed.starts_with('[') && trimmed.ends_with(']');
        if is_header {
            if trimmed == header {
                in_section = true;
                continue;
            }
            if in_section {
                if trimmed.starts_with(&subsection_prefix) {
                    continue;
                }
                break;
            }
        }
        if in_section {
            body_lines.push(line);
        }
    }

    body_lines.join("\n")
}

#[test]
fn eval_smoke_benchmark_writes_machine_readable_bundle() {
    let temp_dir = tempfile::tempdir().unwrap();
    let bundle_path = temp_dir.path().join("eval-smoke.json");
    let argv = vec![
        "cargo".to_string(),
        "run".to_string(),
        "--example".to_string(),
        "benchmark_eval_smoke".to_string(),
        "--".to_string(),
        "--bundle-out".to_string(),
        bundle_path.display().to_string(),
        "--repeat-count".to_string(),
        "2".to_string(),
    ];
    let request = eval_smoke_request(bundle_path.clone(), 2, argv);

    let bundle = run_eval_smoke_benchmark(&request).unwrap();

    assert!(bundle_path.exists(), "benchmark bundle must be written");
    assert_eq!(bundle.schema, BENCHMARK_BUNDLE_SCHEMA_V1);
    assert_eq!(bundle.bundle_path, bundle_path.display().to_string());
    assert_eq!(bundle.results.len(), 1);
    assert!(!bundle.commit.is_empty(), "bundle must record commit id");
    assert!(!bundle.toolchain.rustc_version.is_empty(), "bundle must record rustc version");
    assert!(!bundle.toolchain.cargo_version.is_empty(), "bundle must record cargo version");

    let result = &bundle.results[0];
    assert_eq!(result.workload_name, EVAL_SMOKE_WORKLOAD_NAME);
    assert_eq!(result.entry_point, EVAL_SMOKE_ENTRY_POINT);
    assert_eq!(result.repeat_count, 2);
    assert_eq!(result.logical_store_prefix, DEFAULT_LOGICAL_STORE_PREFIX);
    assert_eq!(result.cache_mode, "same-process-repeated-eval");
    assert_eq!(result.root_count, 1);
    assert_eq!(result.phase_metrics.len(), 1);
    assert_eq!(result.phase_metrics[0].name, EVAL_PHASE_METRIC_NAME);
    assert_eq!(result.sample_wall_ns.len(), 2);
    assert!(result.total_wall_ns > 0, "total wall metric must be positive");
    assert!(result.phase_metrics[0].value > 0, "eval phase metric must be positive");

    let bundle_json = std::fs::read_to_string(&bundle_path).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&bundle_json).unwrap();
    assert_eq!(parsed["schema"], BENCHMARK_BUNDLE_SCHEMA_V1);
    assert_eq!(parsed["results"][0]["workload_name"], EVAL_SMOKE_WORKLOAD_NAME);
    assert_eq!(parsed["results"][0]["entry_point"], EVAL_SMOKE_ENTRY_POINT);
    assert_eq!(parsed["results"][0]["logical_store_prefix"], DEFAULT_LOGICAL_STORE_PREFIX);
    assert_eq!(parsed["results"][0]["repeat_count"], 2);
    assert_eq!(parsed["results"][0]["cache_mode"], "same-process-repeated-eval");
    assert_eq!(parsed["results"][0]["phase_metrics"][0]["name"], EVAL_PHASE_METRIC_NAME);
    assert!(
        parsed["results"][0][TOTAL_PHASE_METRIC_NAME].as_u64().unwrap()
            >= parsed["results"][0]["phase_metrics"][0]["value"].as_u64().unwrap()
    );
}

#[test]
fn suite_benchmark_writes_full_workload_matrix_bundle() {
    let temp_dir = tempfile::tempdir().unwrap();
    let bundle_path = temp_dir.path().join("suite.json");
    let argv = vec![
        "cargo".to_string(),
        "run".to_string(),
        "--example".to_string(),
        "benchmark_suite".to_string(),
        "--".to_string(),
        "--bundle-out".to_string(),
        bundle_path.display().to_string(),
        "--repeat-count".to_string(),
        "1".to_string(),
    ];
    let request = suite_request(bundle_path.clone(), 1, argv);

    let bundle = run_suite_benchmark(&request).unwrap();

    assert!(bundle_path.exists(), "suite bundle must be written");
    assert_eq!(bundle.results.len(), 6);

    let names: Vec<&str> = bundle.results.iter().map(|result| result.workload_name.as_str()).collect();
    assert!(names.contains(&EVAL_SMOKE_WORKLOAD_NAME));
    assert!(names.contains(&CONVERSION_WORKLOAD_NAME));
    assert!(names.contains(&SUBSTITUTION_WORKLOAD_NAME));
    assert!(names.contains(&BUILD_GRAPH_WORKLOAD_NAME));
    assert!(names.contains(&MULTI_PHASE_WORKFLOW_WORKLOAD_NAME));
    assert!(names.contains(&STORE_AWARE_WORKLOAD_NAME));

    let json = std::fs::read_to_string(&bundle_path).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    let results = parsed["results"].as_array().unwrap();
    assert_eq!(results.len(), 6);
    assert!(results.iter().any(|result| result["entry_point"] == SUITE_ENTRY_POINT));
    assert!(results.iter().any(|result| phase_metric_names(result).contains(&EVAL_PHASE_METRIC_NAME)));
    assert!(results.iter().any(|result| phase_metric_names(result).contains(&CONVERSION_PHASE_METRIC_NAME)));
    assert!(results.iter().any(|result| phase_metric_names(result).contains(&SUBSTITUTION_PHASE_METRIC_NAME)));
    assert!(results.iter().any(|result| phase_metric_names(result).contains(&BUILD_GRAPH_PHASE_METRIC_NAME)));
    assert!(
        results
            .iter()
            .any(|result| phase_metric_names(result).contains(&STORE_PERSISTENCE_PHASE_METRIC_NAME))
    );
    assert!(results.iter().any(|result| phase_metric_names(result).contains(&STORE_LOOKUP_PHASE_METRIC_NAME)));
    assert!(results.iter().all(|result| result["rationale"].as_str().is_some_and(|value| !value.is_empty())));
    assert!(results.iter().all(|result| result[TOTAL_PHASE_METRIC_NAME].as_u64().is_some()));
}

#[test]
fn suite_phase_metrics_cover_phase_two_scope() {
    let temp_dir = tempfile::tempdir().unwrap();
    let bundle_path = temp_dir.path().join("suite.json");
    let argv = vec![
        "cargo".to_string(),
        "run".to_string(),
        "--example".to_string(),
        "benchmark_suite".to_string(),
        "--".to_string(),
        "--bundle-out".to_string(),
        bundle_path.display().to_string(),
        "--repeat-count".to_string(),
        "1".to_string(),
    ];
    let request = suite_request(bundle_path, 1, argv);
    let bundle = run_suite_benchmark(&request).unwrap();

    let metric_names: Vec<&str> = bundle
        .results
        .iter()
        .flat_map(|result| result.phase_metrics.iter().map(|metric| metric.name.as_str()))
        .collect();

    assert!(metric_names.contains(&EVAL_PHASE_METRIC_NAME));
    assert!(metric_names.contains(&CONVERSION_PHASE_METRIC_NAME));
    assert!(metric_names.contains(&SUBSTITUTION_PHASE_METRIC_NAME));
    assert!(metric_names.contains(&BUILD_GRAPH_PHASE_METRIC_NAME));
    assert!(metric_names.contains(&STORE_PERSISTENCE_PHASE_METRIC_NAME));
    assert!(metric_names.contains(&STORE_LOOKUP_PHASE_METRIC_NAME));
}

#[test]
fn suite_has_checked_in_multi_phase_workflow_result() {
    let temp_dir = tempfile::tempdir().unwrap();
    let bundle_path = temp_dir.path().join("suite.json");
    let argv = vec![
        "cargo".to_string(),
        "run".to_string(),
        "--example".to_string(),
        "benchmark_suite".to_string(),
        "--".to_string(),
        "--bundle-out".to_string(),
        bundle_path.display().to_string(),
        "--repeat-count".to_string(),
        "1".to_string(),
    ];
    let request = suite_request(bundle_path, 1, argv);
    let bundle = run_suite_benchmark(&request).unwrap();
    let workflow = bundle
        .results
        .iter()
        .find(|result| result.workload_name == MULTI_PHASE_WORKFLOW_WORKLOAD_NAME)
        .unwrap();
    let metric_names: Vec<&str> = workflow.phase_metrics.iter().map(|metric| metric.name.as_str()).collect();

    assert!(workflow.phase_metrics.len() > 1);
    assert!(metric_names.contains(&EVAL_PHASE_METRIC_NAME));
    assert!(metric_names.contains(&BUILD_GRAPH_PHASE_METRIC_NAME));
}

#[test]
fn store_metrics_only_appear_on_store_aware_workload() {
    let temp_dir = tempfile::tempdir().unwrap();
    let bundle_path = temp_dir.path().join("suite.json");
    let argv = vec![
        "cargo".to_string(),
        "run".to_string(),
        "--example".to_string(),
        "benchmark_suite".to_string(),
        "--".to_string(),
        "--bundle-out".to_string(),
        bundle_path.display().to_string(),
        "--repeat-count".to_string(),
        "1".to_string(),
    ];
    let request = suite_request(bundle_path, 1, argv);
    let bundle = run_suite_benchmark(&request).unwrap();

    for result in &bundle.results {
        let metric_names: Vec<&str> = result.phase_metrics.iter().map(|metric| metric.name.as_str()).collect();
        let has_store_persistence = metric_names.contains(&STORE_PERSISTENCE_PHASE_METRIC_NAME);
        let has_store_lookup = metric_names.contains(&STORE_LOOKUP_PHASE_METRIC_NAME);
        if result.workload_name == STORE_AWARE_WORKLOAD_NAME {
            assert!(has_store_persistence);
            assert!(has_store_lookup);
        } else {
            assert!(!has_store_persistence, "unexpected store persistence metric on {}", result.workload_name);
            assert!(!has_store_lookup, "unexpected store lookup metric on {}", result.workload_name);
        }
    }
}

#[test]
fn suite_workload_descriptors_stay_deterministic() {
    let left = suite_workload_descriptors().unwrap();
    let right = suite_workload_descriptors().unwrap();
    assert_eq!(left, right);
    assert_eq!(left.len(), 6);
}

#[test]
fn toml_section_keeps_dotted_subsections() {
    let source = r#"
[dependencies]
foo = "1"
[dependencies.bar]
path = "vendor/bar"
[dev-dependencies]
baz = "2"
"#;

    let dependencies = toml_section(source, "dependencies");
    let dev_dependencies = toml_section(source, "dev-dependencies");

    assert!(dependencies.contains("foo = \"1\""));
    assert!(dependencies.contains("path = \"vendor/bar\""));
    assert!(!dependencies.contains("baz = \"2\""));
    assert!(dev_dependencies.contains("baz = \"2\""));
}

#[test]
fn benchmark_runtime_boundary_stays_out_of_library_path() {
    let cargo_toml = std::fs::read_to_string(repo_root().join("Cargo.toml")).unwrap();
    let lib_rs = std::fs::read_to_string(repo_root().join("src").join("lib.rs")).unwrap();
    let main_rs = std::fs::read_to_string(repo_root().join("src").join("main.rs")).unwrap();
    let dependency_section = toml_section(&cargo_toml, "dependencies");
    let dev_dependency_section = toml_section(&cargo_toml, "dev-dependencies");

    assert!(!lib_rs.contains("pub mod benchmark"));
    assert!(!main_rs.contains("mod benchmark"));
    assert!(dev_dependency_section.contains("crunch-delta = { path = \"crates/crunch-delta\" }"));
    assert!(!dependency_section.contains("crunch-delta = { path = \"crates/crunch-delta\" }"));
    assert!(repo_root().join("examples").join("benchmark_support.rs").exists());
}

#[test]
fn benchmark_docs_cover_all_workloads_and_entry_points() {
    let docs = std::fs::read_to_string(repo_root().join("docs").join("benchmark-suite.md")).unwrap();
    assert!(docs.contains(EVAL_SMOKE_WORKLOAD_NAME));
    assert!(docs.contains(CONVERSION_WORKLOAD_NAME));
    assert!(docs.contains(SUBSTITUTION_WORKLOAD_NAME));
    assert!(docs.contains(BUILD_GRAPH_WORKLOAD_NAME));
    assert!(docs.contains("benchmark_eval_smoke"));
    assert!(docs.contains("benchmark_suite"));
    assert!(docs.contains("benchmark_compare"));
    assert!(docs.contains("evaluation_wall_ns"));
    assert!(docs.contains("conversion_wall_ns"));
    assert!(docs.contains("substitution_planning_wall_ns"));
    assert!(docs.contains("build_graph_wall_ns"));
    assert!(docs.contains("store_persistence_wall_ns"));
    assert!(docs.contains("store_lookup_wall_ns"));
    assert!(docs.contains("phase_metrics` empty"));
    assert!(docs.contains("Ordinary development"));
    assert!(docs.contains("Optimization-specific experiments"));
    assert!(docs.contains("/nix/store"));
    assert!(docs.contains("/crunch/store"));
    assert!(docs.contains("workflow-package-set-eval-build-graph"));
    assert!(docs.contains("store-persist-lookup-blob"));
}

fn phase_metric_names(result: &serde_json::Value) -> Vec<&str> {
    result["phase_metrics"]
        .as_array()
        .unwrap()
        .iter()
        .map(|metric| metric["name"].as_str().unwrap())
        .collect()
}

#[test]
fn benchmark_entry_points_are_checked_in_examples() {
    let smoke_entry = repo_root().join("examples").join("benchmark_eval_smoke.rs");
    let suite_entry = repo_root().join("examples").join("benchmark_suite.rs");
    let compare_entry = repo_root().join("examples").join("benchmark_compare.rs");
    let smoke_source = std::fs::read_to_string(smoke_entry).unwrap();
    let suite_source = std::fs::read_to_string(suite_entry).unwrap();
    let compare_source = std::fs::read_to_string(compare_entry).unwrap();

    assert_eq!(DEFAULT_REPEAT_COUNT, 5);
    assert!(smoke_source.contains("run_eval_smoke_benchmark"));
    assert!(suite_source.contains("run_suite_benchmark"));
    assert!(compare_source.contains("compare_benchmark_files"));
    assert!(compare_source.contains("COMPARE_ENTRY_POINT"));
    assert!(smoke_source.contains("METRIC"));
    assert!(suite_source.contains("METRIC"));
}
