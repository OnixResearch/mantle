# I7 native dynamic-plan scheduling

Task-ID: I7
Covers: build.engine.dynamic.plans.scheduler
Date: 2026-05-31T05:44:19Z

## Coverage summary

- Accepted native plan artifacts are converted into `nix_compat::Derivation` values and inserted into `DerivationRegistry`.
- Unit-output dependencies are registered in topological order and wired as derivation input edges.
- `Worker::want()` is called only for plan roots; normal lazy dependency wiring builds the root dependency closure.
- Non-root units that are not reachable from a root are registered in the build registry source (`DerivationRegistry`) but do not become worker goals and are not built.
- Rejected native plan artifacts do not register or schedule units.
- Declared native outputs remain excluded from `.drv` compatibility discovery.

## cargo fmt --check and diff check

Command:

```sh
cargo fmt --check -p crunch-build
git diff --check
```

Output:

```text

exit status: 0
```

## cargo test -p crunch-build worker::tests --lib

Command:

```sh
cargo test -p crunch-build worker::tests --lib
```

Output excerpt:

```text
running 39 tests
...
test worker::tests::native_dynamic_plan_rejected_output_schedules_no_units ... ok
test worker::tests::native_dynamic_plan_valid_output_schedules_root_unit_in_same_run ... ok
test worker::tests::native_dynamic_plan_registers_all_units_but_wants_only_roots ... ok
...
test result: ok. 39 passed; 0 failed; 0 ignored; 0 measured; 403 filtered out; finished in 0.13s

exit status: 0
```

## cargo test -p crunch-build dynamic_plan --lib

Command:

```sh
cargo test -p crunch-build dynamic_plan --lib
```

Output excerpt:

```text
running 45 tests
...
test worker::tests::native_dynamic_plan_rejected_output_schedules_no_units ... ok
test worker::tests::native_dynamic_plan_valid_output_schedules_root_unit_in_same_run ... ok
test worker::tests::native_dynamic_plan_registers_all_units_but_wants_only_roots ... ok

test result: ok. 45 passed; 0 failed; 0 ignored; 0 measured; 397 filtered out; finished in 0.02s

exit status: 0
```

## openspec validate native-dynamic-plans

Command:

```sh
openspec validate native-dynamic-plans
```

Output:

```text
Change 'native-dynamic-plans' is valid

exit status: 0
```
