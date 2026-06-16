# Source-root musl Rust provider tools focused validation (2026-06-16)

Task-ID: V1
Covers: r[rust_package_planning.source_built_toolchain_closure.source_root_musl_rust_provider_tools]

## Summary

Focused validation passed for source-root musl alias helper coverage, synthetic musl-host provider materialization metadata, default materialization regression coverage, generated x.py script source-root sysroot fallback coverage, checked-in musl route validation, formatting, and whitespace checks.

## rustfmt-check

status: `0`

### stdout

```text
```

### stderr

```text
```

## alias-helper-test

status: `0`

### stdout

```text

running 1 test
test rust_source_provider::tests::musl_target_tool_candidates_include_source_root_aliases_only ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 808 filtered out; finished in 0.00s

```

### stderr

```text
warning: /home/brittonr/git/mantle/Cargo.toml: file `/home/brittonr/git/mantle/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.19s
     Running unittests src/main.rs (/home/brittonr/.cargo-target/debug/deps/mantle-a789c34159ead64f)
```

## musl-host-materializer-test

status: `0`

### stdout

```text

running 1 test
test rust_source_provider::tests::materializer_writes_musl_host_provider_metadata_from_route_plan ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 808 filtered out; finished in 0.70s

```

### stderr

```text
warning: /home/brittonr/git/mantle/Cargo.toml: file `/home/brittonr/git/mantle/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.18s
     Running unittests src/main.rs (/home/brittonr/.cargo-target/debug/deps/mantle-a789c34159ead64f)
```

## default-materializer-test

status: `0`

### stdout

```text

running 1 test
test rust_source_provider::tests::materializer_writes_final_provider_output_from_validated_candidate ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 808 filtered out; finished in 0.62s

```

### stderr

```text
warning: /home/brittonr/git/mantle/Cargo.toml: file `/home/brittonr/git/mantle/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.19s
     Running unittests src/main.rs (/home/brittonr/.cargo-target/debug/deps/mantle-a789c34159ead64f)
```

## generated-xpy-test

status: `0`

### stdout

```text

running 1 test
test rust_source_provider::tests::materializer_generates_xpy_adapters_when_rust_sources_lack_stage_scripts ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 808 filtered out; finished in 1.86s

```

### stderr

```text
warning: /home/brittonr/git/mantle/Cargo.toml: file `/home/brittonr/git/mantle/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.19s
     Running unittests src/main.rs (/home/brittonr/.cargo-target/debug/deps/mantle-a789c34159ead64f)
```

## musl-route-plan-test

status: `0`

### stdout

```text

running 1 test
test source_toolchain_closure::tests::checked_in_musl_host_rust_source_plan_loads_stagex_mrustc_route ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 808 filtered out; finished in 0.04s

```

### stderr

```text
warning: /home/brittonr/git/mantle/Cargo.toml: file `/home/brittonr/git/mantle/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.18s
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
