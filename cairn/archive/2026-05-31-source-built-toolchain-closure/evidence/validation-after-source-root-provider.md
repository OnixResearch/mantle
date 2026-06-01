# Cairn validation after source-root provider materialization

Task-ID: V5
Covers: rust_package_planning.source_built_toolchain_closure
Date: 2026-05-31
Owner: agent

## Command

```text
cargo fmt --check -p mantle -v
/home/brittonr/.cargo-target/debug/cairn validate --root .
/home/brittonr/.cargo-target/debug/cairn gate tasks source-built-toolchain-closure --root .
```

## Output

```text
cargo fmt --check -p mantle -v
[example (2024)] "/home/brittonr/git/mantle/examples/benchmark_compare.rs"
[example (2024)] "/home/brittonr/git/mantle/examples/benchmark_eval_backends.rs"
[example (2024)] "/home/brittonr/git/mantle/examples/benchmark_eval_smoke.rs"
[example (2024)] "/home/brittonr/git/mantle/examples/benchmark_lazy_eval.rs"
[example (2024)] "/home/brittonr/git/mantle/examples/benchmark_suite.rs"
[lib (2024)] "/home/brittonr/git/mantle/src/lib.rs"
[bin (2024)] "/home/brittonr/git/mantle/src/main.rs"
[test (2024)] "/home/brittonr/git/mantle/tests/attest_cli.rs"
[test (2024)] "/home/brittonr/git/mantle/tests/audit_support.rs"
[test (2024)] "/home/brittonr/git/mantle/tests/benchmark_harness.rs"
[test (2024)] "/home/brittonr/git/mantle/tests/bootstrap_eval.rs"
[test (2024)] "/home/brittonr/git/mantle/tests/bootstrap_parity_cli.rs"
[test (2024)] "/home/brittonr/git/mantle/tests/bootstrap_validate_cli.rs"
[test (2024)] "/home/brittonr/git/mantle/tests/cargo_free_self_build_cli.rs"
[test (2024)] "/home/brittonr/git/mantle/tests/examples_build.rs"
[test (2024)] "/home/brittonr/git/mantle/tests/examples_eval.rs"
[test (2024)] "/home/brittonr/git/mantle/tests/identity_cli.rs"
[test (2024)] "/home/brittonr/git/mantle/tests/integration.rs"
[test (2024)] "/home/brittonr/git/mantle/tests/integration_build.rs"
[test (2024)] "/home/brittonr/git/mantle/tests/operator_diagnostics.rs"
[test (2024)] "/home/brittonr/git/mantle/tests/project_build_smoke.rs"
[test (2024)] "/home/brittonr/git/mantle/tests/project_cli.rs"
[test (2024)] "/home/brittonr/git/mantle/tests/project_refresh_cli.rs"
[test (2024)] "/home/brittonr/git/mantle/tests/release_cli.rs"
[test (2024)] "/home/brittonr/git/mantle/tests/removed_system_cli.rs"
[test (2024)] "/home/brittonr/git/mantle/tests/rust_plan_cli.rs"
[test (2024)] "/home/brittonr/git/mantle/tests/self_hosting.rs"
[test (2024)] "/home/brittonr/git/mantle/tests/smoke.rs"
[test (2024)] "/home/brittonr/git/mantle/tests/stdlib_tests.rs"
[test (2024)] "/home/brittonr/git/mantle/tests/store_gc_cli.rs"
[test (2024)] "/home/brittonr/git/mantle/tests/transcript_cli.rs"
rustfmt --edition 2024 --check ...
{
  "change_issues": [],
  "changes": 1,
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "spec_issues": [],
  "specs_validated": 4,
  "valid": true
}
{
  "change": "source-built-toolchain-closure",
  "input_hash": "16b081955a9338a7f464c1c0d7362d163731261c1861f5cda37260f431f1cec1",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "1d955ac8ca13413fe286459d076c969d137cab09a8938d3dc457c45d5f5a5ac0",
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}
```

## Post-task-check rerun

After checking the validation task, this command was rerun:

```text
/home/brittonr/.cargo-target/debug/cairn validate --root .
/home/brittonr/.cargo-target/debug/cairn gate tasks source-built-toolchain-closure --root .
```

Output:

```json
{
  "change_issues": [],
  "changes": 1,
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "spec_issues": [],
  "specs_validated": 4,
  "valid": true
}
{
  "change": "source-built-toolchain-closure",
  "input_hash": "ba612bbb6ba4ae56813bd313113f8c582ae2ff725a2b1d3cbb42cbb942cc597e",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "0400cff3e3ab4adbe74ee82584fc420f33e3e6c41385e173d961c64775c0c039",
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}
```
