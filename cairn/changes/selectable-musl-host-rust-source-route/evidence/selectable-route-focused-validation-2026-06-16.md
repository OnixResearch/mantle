# Selectable route focused validation (2026-06-16)

Task-ID: V1
Covers: r[rust_package_planning.source_built_toolchain_closure.selectable_rust_source_route]

## Summary

Focused validation passed for the checked-in musl-host route plan, existing default GNU-host route plan, CLI `--route-plan` parsing, explicit route materialization path, formatting, and whitespace checks.

## rustfmt-check

status: `0`

### stdout

```text
```

### stderr

```text
```

## musl-plan-test

status: `0`

### stdout

```text

running 1 test
test source_toolchain_closure::tests::checked_in_musl_host_rust_source_plan_loads_stagex_mrustc_route ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 806 filtered out; finished in 0.04s

```

### stderr

```text
warning: /home/brittonr/git/mantle/Cargo.toml: file `/home/brittonr/git/mantle/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.18s
     Running unittests src/main.rs (/home/brittonr/.cargo-target/debug/deps/mantle-a789c34159ead64f)
```

## default-plan-test

status: `0`

### stdout

```text

running 1 test
test source_toolchain_closure::tests::checked_in_rust_source_plan_loads_stagex_mrustc_route ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 806 filtered out; finished in 0.04s

```

### stderr

```text
warning: /home/brittonr/git/mantle/Cargo.toml: file `/home/brittonr/git/mantle/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.18s
     Running unittests src/main.rs (/home/brittonr/.cargo-target/debug/deps/mantle-a789c34159ead64f)
```

## cli-route-test

status: `0`

### stdout

```text

running 1 test
test tests::bootstrap_rust_source_provider_action_parses_route_plan ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 806 filtered out; finished in 0.00s

```

### stderr

```text
warning: /home/brittonr/git/mantle/Cargo.toml: file `/home/brittonr/git/mantle/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.17s
     Running unittests src/main.rs (/home/brittonr/.cargo-target/debug/deps/mantle-a789c34159ead64f)
```

## explicit-route-materializer-test

status: `0`

### stdout

```text

running 1 test
test rust_source_provider::tests::materializer_uses_explicit_route_plan_when_default_plan_is_absent ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 806 filtered out; finished in 0.69s

```

### stderr

```text
warning: /home/brittonr/git/mantle/Cargo.toml: file `/home/brittonr/git/mantle/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.19s
     Running unittests src/main.rs (/home/brittonr/.cargo-target/debug/deps/mantle-a789c34159ead64f)
```

## diff-check

status: `0`

### stdout

```text
```

### stderr

```text
```
