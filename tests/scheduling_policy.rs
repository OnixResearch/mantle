use std::ffi::OsString;
use std::path::PathBuf;

const VALID_POLICY_EXPRESSION: &str = "let scheduling = import \"scheduling.ncl\" in scheduling.default_policy";
const BOUNDS_EXPRESSION: &str = r#"
let scheduling = import "scheduling.ncl" in {
  known_graph = scheduling.default_known_graph_bounds,
  history = scheduling.default_history_policy,
}
"#;
const INVALID_POLICY_EXPRESSION: &str = r#"
let scheduling = import "scheduling.ncl" in
({ preference_order = ['known-graph, 'unknown-field, 'locality-transfer] }
  | scheduling.SchedulingPolicy)
"#;

fn write_expression(expression: &str) -> (tempfile::TempDir, PathBuf) {
    assert!(!expression.is_empty());
    assert!(expression.contains("scheduling.ncl"));
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("policy.ncl");
    std::fs::write(&path, expression).unwrap();
    (directory, path)
}

fn scheduling_import_paths() -> Vec<OsString> {
    let lib = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("lib");
    assert!(lib.is_dir());
    assert!(lib.join("scheduling.ncl").is_file());
    vec![lib.into_os_string()]
}

#[test]
fn nickel_default_scheduling_policy_matches_rust_runtime_default() {
    let (_directory, path) = write_expression(VALID_POLICY_EXPRESSION);
    let nickel: crunch_build::SchedulingPolicy =
        crunch_eval::evaluate_and_deserialize(&path, &scheduling_import_paths()).unwrap();
    let rust = crunch_build::SchedulingPolicy::default();

    assert_eq!(nickel, rust);
    assert_eq!(nickel.digest_blake3().unwrap(), rust.digest_blake3().unwrap());
}

#[test]
fn nickel_scheduling_bounds_match_rust_kernel_limits() {
    let (_directory, path) = write_expression(BOUNDS_EXPRESSION);
    let value: serde_json::Value = crunch_eval::evaluate_and_deserialize(&path, &scheduling_import_paths()).unwrap();

    assert_eq!(
        value["known_graph"]["max_goals"].as_u64(),
        Some(u64::from(crunch_build::scheduling::MAX_SCHEDULING_GOALS))
    );
    assert_eq!(
        value["known_graph"]["max_edges"].as_u64(),
        Some(u64::from(crunch_build::scheduling::MAX_KNOWN_GRAPH_EDGES))
    );
    assert_eq!(
        value["known_graph"]["max_blocked_root_pressure"].as_u64(),
        Some(u64::from(crunch_build::scheduling::MAX_BLOCKED_ROOT_PRESSURE))
    );
    assert_eq!(
        value["history"]["max_entries"].as_u64(),
        Some(u64::from(crunch_build::scheduling::MAX_HISTORY_ENTRIES))
    );
    assert_eq!(
        value["history"]["max_key_bytes_total"].as_u64(),
        Some(u64::from(crunch_build::scheduling::MAX_HISTORY_KEY_BYTES_TOTAL))
    );
    assert_eq!(
        value["history"]["max_snapshot_bytes"].as_u64(),
        Some(u64::from(crunch_build::scheduling::MAX_HISTORY_SNAPSHOT_BYTES))
    );
}

#[test]
fn nickel_scheduling_contract_rejects_unknown_preference_field() {
    let (_directory, path) = write_expression(INVALID_POLICY_EXPRESSION);
    let result =
        crunch_eval::evaluate_and_deserialize::<crunch_build::SchedulingPolicy>(&path, &scheduling_import_paths());

    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("unknown-field"));
}
