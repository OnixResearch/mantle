# First-stage musl libgcc_eh unwind focused validation (2026-06-16)

Task-ID: V2
Covers: r[rust_package_planning.source_built_toolchain_closure.first_stage_musl_libgcc_eh_unwind]

## Summary

Focused validation passed for wrapper script assertions, materializer behavior, formatting, diff checks, and Cairn gates. `CARGO_BUILD_RUSTC_WRAPPER=` overrides the global sccache wrapper during evidence capture.

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

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 808 filtered out; finished in 0.72s

```

### stderr

```text
warning: /home/brittonr/git/mantle/Cargo.toml: file `/home/brittonr/git/mantle/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
   Compiling mantle v0.1.0 (/home/brittonr/git/mantle)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 6.62s
     Running unittests src/main.rs (/home/brittonr/.cargo-target/debug/deps/mantle-a789c34159ead64f)
```

## musl-host-materializer-test

status: `0`

### stdout

```text

running 1 test
test rust_source_provider::tests::materializer_writes_musl_host_provider_metadata_from_route_plan ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 808 filtered out; finished in 0.68s

```

### stderr

```text
warning: /home/brittonr/git/mantle/Cargo.toml: file `/home/brittonr/git/mantle/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.22s
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
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.21s
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

## cairn-gate-proposal

status: `0`

### stdout

```text
{
  "change": "first-stage-musl-libgcc-eh-unwind",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "340c52054f49721d299ad5f1ab08a23c4cdcf3d4c6176f20ce8ad91e9e04e4d4",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "f22e51ad3fc825e56d721b8cd7983736112eff0c880670f2b0fb917e069d56c8",
  "stage": "proposal",
  "valid": true,
  "verdict": "PASS"
}
```

### stderr

```text
```

## cairn-gate-design

status: `0`

### stdout

```text
{
  "change": "first-stage-musl-libgcc-eh-unwind",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "c4c7730ee990c5022db8f699836358d7e35f4342acdfda6ef455529147c86664",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "5242d0d9fcfe7ecb497c916521dabb0754f501bccc057556394e39460d427c57",
  "stage": "design",
  "valid": true,
  "verdict": "PASS"
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
  "change": "first-stage-musl-libgcc-eh-unwind",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "ac9db26bd2e6f266a2824b7a9872c87598138888c6948c134b18cd7a0635f44e",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "96084fc62bbcbee9069d002f2835444d2d3b89d32ff244f7ba33dde81208a368",
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}
```

### stderr

```text
```
