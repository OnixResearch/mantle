# I8 native dynamic-plan report/provenance plumbing

Task-ID: I8
Covers: build.engine.dynamic.plans.provenance
Date: 2026-05-31T05:56:12Z

## Coverage summary

- Worker results now carry sorted native dynamic-plan report rows.
- Accepted rows include mode label, producer key, declared output name, plan artifact path, raw artifact digest, canonical plan digest, accepted unit IDs, and scheduler action.
- Rejected rows include mode label, producer key, declared output name, optional artifact path/raw digest, rejection reason, empty accepted-unit list, and `rejected` scheduler action.
- Pipeline results thread the rows into JSON build reports as `native_dynamic_plans` for operator/provenance diagnostics.
- Existing `.drv` compatibility discovery remains separate from declared native-plan reporting; native rows carry `mode = "native"`.

## cargo test -p crunch-build worker::tests --lib

Command:

```sh
cargo test -p crunch-build worker::tests --lib
```

Output excerpt:

```text
running 40 tests
test worker::tests::native_dynamic_plan_reports_are_deterministically_sorted ... ok
...
test worker::tests::native_dynamic_plan_rejected_output_schedules_no_units ... ok
...
test worker::tests::native_dynamic_plan_valid_output_schedules_root_unit_in_same_run ... ok
...
test result: ok. 40 passed; 0 failed; 0 ignored; 0 measured; 403 filtered out; finished in 0.13s

exit status: 0
```

## cargo test -p mantle build_report --bin mantle

Command:

```sh
cargo test -p mantle build_report --bin mantle
```

Output excerpt:

```text
running 7 tests
test build_report::tests::build_json_report_includes_artifact_attestation_reference ... ok
...
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 583 filtered out; finished in 0.00s

exit status: 0
```

## cargo test -p crunch-build dynamic_plan --lib

Command:

```sh
cargo test -p crunch-build dynamic_plan --lib
```

Output excerpt:

```text
running 46 tests
...
test worker::tests::native_dynamic_plan_reports_are_deterministically_sorted ... ok
...
test worker::tests::native_dynamic_plan_valid_output_schedules_root_unit_in_same_run ... ok
test worker::tests::native_dynamic_plan_rejected_output_schedules_no_units ... ok
...
test result: ok. 46 passed; 0 failed; 0 ignored; 0 measured; 397 filtered out; finished in 0.02s

exit status: 0
```

## Formatting, diff, and OpenSpec validation

Commands:

```sh
cargo fmt --check -p crunch-build -p crunch-pipeline -p mantle
git diff --check
openspec validate native-dynamic-plans
```

Output:

```text
Change 'native-dynamic-plans' is valid

exit status: 0
```
