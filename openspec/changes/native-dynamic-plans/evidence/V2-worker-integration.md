# V2 worker integration verification

Task-ID: V2
Covers: build.engine.dynamic.plans.declared.outputs, build.engine.dynamic.plans.scheduler, defaults.dynamic.derivations
Date: 2026-05-31T05:58:58Z

Command:

```sh
cargo test -p crunch-build worker::tests --lib
```

Output excerpt:

```text
running 40 tests
test worker::tests::native_dynamic_plan_reports_are_deterministically_sorted ... ok
test worker::tests::native_dynamic_plan_declared_output_is_accepted ... ok
test worker::tests::native_dynamic_plan_undeclared_output_is_ignored ... ok
test worker::tests::native_dynamic_plan_declared_output_rejections_are_structured ... ok
test worker::tests::native_dynamic_plan_rejected_output_schedules_no_units ... ok
test worker::tests::declared_native_dynamic_output_is_not_compat_drv_discovery ... ok
test worker::tests::native_dynamic_plan_valid_output_schedules_root_unit_in_same_run ... ok
test worker::tests::dynamic_drv_detected_and_built ... ok
test worker::tests::native_dynamic_plan_registers_all_units_but_wants_only_roots ... ok
...
test result: ok. 40 passed; 0 failed; 0 ignored; 0 measured; 403 filtered out; finished in 0.12s

exit status: 0
```
