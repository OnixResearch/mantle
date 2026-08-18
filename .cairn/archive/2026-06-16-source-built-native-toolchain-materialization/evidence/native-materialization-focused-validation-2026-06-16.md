# Native materialization focused validation (2026-06-16)

Task-ID: V1
Covers: r[rust_package_planning.source_built_toolchain_closure.native_materialization]

## Summary

Focused validation passed for the pure manifest materializer, shell digest helpers, runtime/CRT enforcement tightening, formatting, and whitespace checks.

## rustfmt-check

status: `0`

### stdout

```text
```

### stderr

```text
```

## source-tests

status: `0`

### stdout

```text

running 3 tests
test source_toolchain_closure::tests::native_materialization_rejects_missing_host_runtime_members ... ok
test source_toolchain_closure::tests::native_materialization_rejects_missing_target_helper_members ... ok
test source_toolchain_closure::tests::native_materialization_builds_zero_seed_manifest_from_complete_candidates ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 793 filtered out; finished in 0.00s

```

### stderr

```text
warning: /home/brittonr/git/mantle/Cargo.toml: file `/home/brittonr/git/mantle/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.19s
     Running unittests src/main.rs (/home/brittonr/.cargo-target/debug/deps/mantle-a789c34159ead64f)
```

## shell-tests

status: `0`

### stdout

```text

running 2 tests
test native_toolchain_closure::tests::provider_metadata_path_prefers_native_metadata ... ok
test native_toolchain_closure::tests::directory_digest_changes_when_file_content_changes ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 794 filtered out; finished in 0.01s

```

### stderr

```text
warning: /home/brittonr/git/mantle/Cargo.toml: file `/home/brittonr/git/mantle/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.18s
     Running unittests src/main.rs (/home/brittonr/.cargo-target/debug/deps/mantle-a789c34159ead64f)
```

## enforcement-runtime-test

status: `0`

### stdout

```text

running 1 test
test cargo_free_self_build::tests::receipt_bound_enforcement_rejects_runtime_library_digest_mismatch ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 795 filtered out; finished in 0.00s

```

### stderr

```text
warning: /home/brittonr/git/mantle/Cargo.toml: file `/home/brittonr/git/mantle/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.17s
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
