# Focused validation evidence

Task-ID: V2
Covers: r[rust_package_planning.source_built_toolchain_closure.first_stage_musl_rustc_driver_rlib]
Date: 2026-06-18

## rustfmt

```text
```

## focused rust_source_provider tests

```text
warning: /home/brittonr/git/mantle/Cargo.toml: file `/home/brittonr/git/mantle/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.20s
     Running unittests src/main.rs (/home/brittonr/.cargo-target/debug/deps/mantle-a789c34159ead64f)

running 1 test
test rust_source_provider::tests::materializer_writes_final_provider_output_from_validated_candidate ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 808 filtered out; finished in 1.06s

warning: /home/brittonr/git/mantle/Cargo.toml: file `/home/brittonr/git/mantle/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.21s
     Running unittests src/main.rs (/home/brittonr/.cargo-target/debug/deps/mantle-a789c34159ead64f)

running 1 test
test rust_source_provider::tests::musl_target_tool_candidates_include_source_root_aliases_only ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 808 filtered out; finished in 0.00s

warning: /home/brittonr/git/mantle/Cargo.toml: file `/home/brittonr/git/mantle/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.21s
     Running unittests src/main.rs (/home/brittonr/.cargo-target/debug/deps/mantle-a789c34159ead64f)

running 1 test
test rust_source_provider::tests::materializer_writes_musl_host_provider_metadata_from_route_plan ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 808 filtered out; finished in 0.86s

```

## git diff --check

```text
```

## cairn validate/gates

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
{
  "change": "first-stage-musl-rustc-driver-rlib",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "90972eb3befba57c55c256358f1e692b19938a24e262e4683efbbbb97d1dd87d",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "5c22c617d9ff88556724ac595cb1f0a011865da72ea756d44959e622f8ff9eb5",
  "stage": "proposal",
  "valid": true,
  "verdict": "PASS"
}
{
  "change": "first-stage-musl-rustc-driver-rlib",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "e7df5aeb88477eba43f3e4560c615f476ab77f3a4d24a508ddb6ae6803c6741b",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "e02e01803f43f2255ce56114c33ce15be77a40d0024aa8a77357f1b6a4525914",
  "stage": "design",
  "valid": true,
  "verdict": "PASS"
}
{
  "change": "first-stage-musl-rustc-driver-rlib",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "e88a3c7d4b0eda5e8d2532fa0c035b1aa43f0a9cc06986935f906e757c286b0a",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "f68d16e5298f3f86f6307ae91db27ee4aa67b3ce717e4f87825bb7fe42a2e8ef",
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}
```

## post-task checkbox Cairn validate/gates

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
{
  "change": "first-stage-musl-rustc-driver-rlib",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "6fbefc7500413b8016a3f9a2ddf15df8027693a4478c30e57455d9b54d344b6d",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "ea093113cde0071ec2198a8066de075969e7572031fad203af9674acc35b5dd4",
  "stage": "proposal",
  "valid": true,
  "verdict": "PASS"
}
{
  "change": "first-stage-musl-rustc-driver-rlib",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "a7acdad0e2a6f01a52f5ba2866096483792f8d043cf31c4a29fc9fcb42e747ea",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "06f0d78836673985fd67bbe5659a75822cf1aca8bb33ffea004bc365f298a288",
  "stage": "design",
  "valid": true,
  "verdict": "PASS"
}
{
  "change": "first-stage-musl-rustc-driver-rlib",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "ce6bcb339ed58e3526d959b7cf62d45629bc78b371041c4c54a0cb9cf4749d51",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "ad738e8caf49bd9ece101153740e0d1492288d0266e555052208fc5c13401795",
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}
```
