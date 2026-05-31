# I6 native dynamic-plan scanning

Task-ID: I6
Covers: build.engine.dynamic.plans.declared.outputs, build.engine.dynamic.plans.scheduler
Date: 2026-05-31T05:34:19Z

## Pre-change baseline

Ran before I6 edits:

```text
cargo test -p crunch-build worker::tests --lib
running 32 tests
...
test result: ok. 32 passed; 0 failed; 0 ignored; 0 measured; 403 filtered out; finished in 0.12s
```

## I6 coverage summary

- Worker now derives declared native dynamic-plan outputs from `DerivationRegistry` metadata after producer completion.
- Only declared outputs are scanned as native `mantle-plan-v1` artifacts.
- Declared missing, non-regular, over-limit, unreadable, or invalid artifacts produce typed `NativeDynamicPlanRejection` rows tied to producer key and output name.
- Accepted artifacts record producer key, output name, artifact path, raw BLAKE3 artifact digest, canonical plan digest, accepted unit IDs, and the canonical plan value.
- Undeclared plan-looking outputs produce no native scan rows.
- Declared native outputs are skipped by `.drv` compatibility discovery so malformed declared native plans are not scheduled through the compatibility path.

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
running 36 tests
...
test worker::tests::native_dynamic_plan_undeclared_output_is_ignored ... ok
test worker::tests::native_dynamic_plan_declared_output_rejections_are_structured ... ok
test worker::tests::declared_native_dynamic_output_is_not_compat_drv_discovery ... ok
test worker::tests::native_dynamic_plan_declared_output_is_accepted ... ok
...
test result: ok. 36 passed; 0 failed; 0 ignored; 0 measured; 403 filtered out; finished in 0.12s

exit status: 0
```

## cargo test -p crunch-build dynamic_plan --lib

Command:

```sh
cargo test -p crunch-build dynamic_plan --lib
```

Output excerpt:

```text
running 42 tests
...
test worker::tests::native_dynamic_plan_declared_output_rejections_are_structured ... ok
test worker::tests::native_dynamic_plan_declared_output_is_accepted ... ok
test worker::tests::native_dynamic_plan_undeclared_output_is_ignored ... ok

test result: ok. 42 passed; 0 failed; 0 ignored; 0 measured; 397 filtered out; finished in 0.01s

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
