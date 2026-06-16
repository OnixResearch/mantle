# First-stage musl static-pie normalization focused validation (2026-06-16)

Task-ID: V2
Covers: r[rust_package_planning.source_built_toolchain_closure.first_stage_musl_static_pie_normalization]

## Summary

Focused validation passed for wrapper script assertions, materializer behavior, formatting, diff checks, and Cairn gates. `CARGO_BUILD_RUSTC_WRAPPER=` is set to override the global sccache wrapper during evidence capture.

## rustfmt-check

status: `0`

### stdout

```text
```

### stderr

```text
```

## default-materializer-test

status: `0`

### stdout

```text

running 1 test
test rust_source_provider::tests::materializer_writes_final_provider_output_from_validated_candidate ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 808 filtered out; finished in 0.71s

```

### stderr

```text
warning: /home/brittonr/git/mantle/Cargo.toml: file `/home/brittonr/git/mantle/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
   Compiling mantle v0.1.0 (/home/brittonr/git/mantle)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 5.31s
     Running unittests src/main.rs (/home/brittonr/.cargo-target/debug/deps/mantle-a789c34159ead64f)
```

## musl-host-materializer-test

status: `0`

### stdout

```text

running 1 test
test rust_source_provider::tests::materializer_writes_musl_host_provider_metadata_from_route_plan ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 808 filtered out; finished in 0.52s

```

### stderr

```text
warning: /home/brittonr/git/mantle/Cargo.toml: file `/home/brittonr/git/mantle/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.21s
     Running unittests src/main.rs (/home/brittonr/.cargo-target/debug/deps/mantle-a789c34159ead64f)
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

## diff-check

status: `0`

### stdout

```text
```

### stderr

```text
```

## cairn-validate

status: `0`

### stdout

```text
{
  "change_issues": [],
  "changes": 1,
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "spec_issues": [],
  "specs_validated": 6,
  "valid": true
}
```

### stderr

```text
```

## cairn-gate-tasks

status: `0`

### stdout

```text
{
  "change": "first-stage-musl-static-pie-normalization",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "e6b2977ae71c4c7cd9b5b5fe291525afec6e5f5c9e96f6ac8f68ffaa450ce22d",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "c3594658125dea8b21e678aad60099fee4d0fcc4e27dc3399f8f7e9bcc5c3c89",
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}
```

### stderr

```text
```
