# V3 report/provenance verification

Task-ID: V3
Covers: build.engine.dynamic.plans.provenance
Date: 2026-05-31T05:58:58Z

## build report/provenance command

```sh
cargo test -p mantle build_report --bin mantle
```

Output excerpt:

```text
running 7 tests
test build_report::tests::build_json_report_includes_artifact_attestation_reference ... ok
test build_report::tests::render_build_json_report_serializes_full_substitution_fields_stably ... ok
...
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 583 filtered out; finished in 0.00s

exit status: 0
```

## worker-side report row coverage

Command:

```sh
cargo test -p crunch-build worker::tests --lib
```

Output excerpt:

```text
running 40 tests
test worker::tests::native_dynamic_plan_reports_are_deterministically_sorted ... ok
test worker::tests::native_dynamic_plan_rejected_output_schedules_no_units ... ok
test worker::tests::native_dynamic_plan_valid_output_schedules_root_unit_in_same_run ... ok
...
test result: ok. 40 passed; 0 failed; 0 ignored; 0 measured; 403 filtered out; finished in 0.12s

exit status: 0
```
