# Typed contract for the SpaceWasm stable test-fact evidence (version 1).
#
# This is the typed source of truth for the stable-report and run-record
# schemas and their named bounds. The checked-in JSON export
# (`stable-report-contract.json`) is deterministic; the Rust core proves its
# constants equal these values.
let Bound = std.contract.from_predicate (fun value => std.is_number value && value >= 0) in

{
  schema = "mantle-spacewasm-stable-report-contract-v1",
  stable_report_schema = "mantle-spacewasm-stable-report-v1",
  run_record_schema = "mantle-spacewasm-run-record-v1",
  encoding_version = 1,
  max_tests = 4096,
  max_harness_lines = 8192,
  max_test_name_bytes = 1024,
  max_test_stdout_bytes = 65536,
  max_suite_seconds = 86400,
  max_run_captures = 8,
  max_capture_role_bytes = 128,
  admitted_run_statuses = ["passed", "failed", "signal", "timeout", "cancelled", "unavailable"],
} | {
  schema | String,
  stable_report_schema | String,
  run_record_schema | String,
  encoding_version | Bound,
  max_tests | Bound,
  max_harness_lines | Bound,
  max_test_name_bytes | Bound,
  max_test_stdout_bytes | Bound,
  max_suite_seconds | Bound,
  max_run_captures | Bound,
  max_capture_role_bytes | Bound,
  admitted_run_statuses | Array String,
}
