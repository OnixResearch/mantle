//! Cross-boundary parity: the Rust constants must equal the typed Nickel
//! contract export in `packages/spacewasm-reference/stable-report-contract.json`.

use crunch_spacewasm_core::ADMITTED_RUN_STATUSES;
use crunch_spacewasm_core::MAX_CAPTURE_ROLE_BYTES;
use crunch_spacewasm_core::MAX_HARNESS_LINES;
use crunch_spacewasm_core::MAX_RUN_CAPTURES;
use crunch_spacewasm_core::MAX_STABLE_TESTS;
use crunch_spacewasm_core::MAX_SUITE_SECONDS;
use crunch_spacewasm_core::MAX_TEST_NAME_BYTES;
use crunch_spacewasm_core::MAX_TEST_STDOUT_BYTES;
use crunch_spacewasm_core::RUN_RECORD_ENCODING_VERSION;
use crunch_spacewasm_core::RUN_RECORD_SCHEMA;
use crunch_spacewasm_core::STABLE_REPORT_ENCODING_VERSION;
use crunch_spacewasm_core::STABLE_REPORT_SCHEMA;

const EXPORTED_CONTRACT: &str = include_str!("../../../packages/spacewasm-reference/stable-report-contract.json");

fn exported_number(field: &str) -> u64 {
    let parsed: serde_json::Value = serde_json::from_str(EXPORTED_CONTRACT).expect("contract export parses");
    parsed
        .get(field)
        .and_then(serde_json::Value::as_u64)
        .unwrap_or_else(|| panic!("contract export has numeric field {field}"))
}

fn exported_string(field: &str) -> String {
    let parsed: serde_json::Value = serde_json::from_str(EXPORTED_CONTRACT).expect("contract export parses");
    parsed
        .get(field)
        .and_then(serde_json::Value::as_str)
        .map(String::from)
        .unwrap_or_else(|| panic!("contract export has string field {field}"))
}

#[test]
fn rust_bounds_equal_the_typed_nickel_contract() {
    assert_eq!(exported_number("max_tests"), u64::from(MAX_STABLE_TESTS));
    assert_eq!(exported_number("max_harness_lines"), u64::from(MAX_HARNESS_LINES));
    assert_eq!(exported_number("max_test_name_bytes"), MAX_TEST_NAME_BYTES as u64);
    assert_eq!(exported_number("max_test_stdout_bytes"), MAX_TEST_STDOUT_BYTES as u64);
    assert_eq!(exported_number("max_run_captures"), u64::from(MAX_RUN_CAPTURES));
    assert_eq!(exported_number("max_capture_role_bytes"), MAX_CAPTURE_ROLE_BYTES as u64);
    assert_eq!(exported_number("max_suite_seconds"), MAX_SUITE_SECONDS as u64);
    assert_eq!(exported_number("encoding_version"), u64::from(RUN_RECORD_ENCODING_VERSION));
    assert_eq!(u64::from(STABLE_REPORT_ENCODING_VERSION), 1);
    assert_eq!(exported_string("stable_report_schema"), STABLE_REPORT_SCHEMA);
    assert_eq!(exported_string("run_record_schema"), RUN_RECORD_SCHEMA);
}

#[test]
fn admitted_run_statuses_equal_the_typed_nickel_contract() {
    let parsed: serde_json::Value = serde_json::from_str(EXPORTED_CONTRACT).expect("contract export parses");
    let statuses: Vec<String> = parsed["admitted_run_statuses"]
        .as_array()
        .expect("statuses are an array")
        .iter()
        .map(|value| String::from(value.as_str().expect("status is a string")))
        .collect();
    assert_eq!(statuses, ADMITTED_RUN_STATUSES.map(String::from).to_vec());
}
