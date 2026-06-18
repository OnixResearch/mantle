# Focused validation evidence

Task-ID: V2
Covers: r[rust_package_planning.source_built_toolchain_closure.first_stage_musl_stage2_rustc_probe_runtime]
Date: 2026-06-18

## rustfmt

```text
$ /home/brittonr/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin/rustfmt --edition 2024 --check src/rust_source_provider.rs

```

## focused rust_source_provider tests

```text
$ cargo test -p mantle --bin mantle rust_source_provider::tests::materializer_writes_final_provider_output_from_validated_candidate -- --exact
warning: /home/brittonr/git/mantle/Cargo.toml: file `/home/brittonr/git/mantle/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.22s
     Running unittests src/main.rs (/home/brittonr/.cargo-target/debug/deps/mantle-a789c34159ead64f)

running 1 test
test rust_source_provider::tests::materializer_writes_final_provider_output_from_validated_candidate ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 808 filtered out; finished in 1.03s

$ cargo test -p mantle --bin mantle rust_source_provider::tests::musl_target_tool_candidates_include_source_root_aliases_only -- --exact
warning: /home/brittonr/git/mantle/Cargo.toml: file `/home/brittonr/git/mantle/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.27s
     Running unittests src/main.rs (/home/brittonr/.cargo-target/debug/deps/mantle-a789c34159ead64f)

running 1 test
test rust_source_provider::tests::musl_target_tool_candidates_include_source_root_aliases_only ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 808 filtered out; finished in 0.00s

$ cargo test -p mantle --bin mantle rust_source_provider::tests::materializer_writes_musl_host_provider_metadata_from_route_plan -- --exact
warning: /home/brittonr/git/mantle/Cargo.toml: file `/home/brittonr/git/mantle/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.22s
     Running unittests src/main.rs (/home/brittonr/.cargo-target/debug/deps/mantle-a789c34159ead64f)

running 1 test
test rust_source_provider::tests::materializer_writes_musl_host_provider_metadata_from_route_plan ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 808 filtered out; finished in 1.04s


```

## git diff --check

```text
$ git diff --check

```

## cairn validate

```text
$ nix run path:/home/brittonr/git/cairn#cairn -- validate --root .
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

## cairn gate proposal

```text
$ nix run path:/home/brittonr/git/cairn#cairn -- gate proposal first-stage-musl-stage2-rustc-probe-runtime --root .
{
  "change": "first-stage-musl-stage2-rustc-probe-runtime",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "b4e4880b0694ee01864c40392fa1f00d7c956cf7aa0ff623f92d02aaf1bd7bd8",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "816adc3e874d622b16810153d36cc13ab22f6505547ab6a7e0ef23e4ac9f9f18",
  "stage": "proposal",
  "valid": true,
  "verdict": "PASS"
}

```

## cairn gate design

```text
$ nix run path:/home/brittonr/git/cairn#cairn -- gate design first-stage-musl-stage2-rustc-probe-runtime --root .
{
  "change": "first-stage-musl-stage2-rustc-probe-runtime",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "a8a689b18230003e49d6f6cae60ae220e125c31e6b3ba7610dd97e8d506498aa",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "2c3a9831520bea78e8aed808191d0015d74e5dfe1df18e869983cfcfd37a9c83",
  "stage": "design",
  "valid": true,
  "verdict": "PASS"
}

```

## cairn gate tasks

```text
$ nix run path:/home/brittonr/git/cairn#cairn -- gate tasks first-stage-musl-stage2-rustc-probe-runtime --root .
{
  "change": "first-stage-musl-stage2-rustc-probe-runtime",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "cb2419d91025c6a7f58bfa5002bde734031b09e1f0d3e432273201c44c0ac234",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "017dad3f65ea5e95347c7a9024c922a6f918faaf98443bff3bafdf320bbebc9a",
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}

```

## post-task checkbox Cairn validate/gates

## post-task cairn validate

```text
$ nix run path:/home/brittonr/git/cairn#cairn -- validate --root .
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

## post-task cairn gate proposal

```text
$ nix run path:/home/brittonr/git/cairn#cairn -- gate proposal first-stage-musl-stage2-rustc-probe-runtime --root .
{
  "change": "first-stage-musl-stage2-rustc-probe-runtime",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "b6cc5aec36ef19dddef1bb465b5910f40e3a9aca606ab87a856b29da5c59b3a2",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "6ffdff6cfa2ed58b6cf204c1f31afbcffd199fd8bba7a3fc7f226c7f9d7e1a77",
  "stage": "proposal",
  "valid": true,
  "verdict": "PASS"
}

```

## post-task cairn gate design

```text
$ nix run path:/home/brittonr/git/cairn#cairn -- gate design first-stage-musl-stage2-rustc-probe-runtime --root .
{
  "change": "first-stage-musl-stage2-rustc-probe-runtime",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "4d4ab363af75dc67d10d1c619412b179882783c8578753430b34cb451645a6a9",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "6ddddc5c3e2b4e29e71634e25e119c12d36c72e708c8781a9cb095e5dd36884a",
  "stage": "design",
  "valid": true,
  "verdict": "PASS"
}

```

## post-task cairn gate tasks

```text
$ nix run path:/home/brittonr/git/cairn#cairn -- gate tasks first-stage-musl-stage2-rustc-probe-runtime --root .
{
  "change": "first-stage-musl-stage2-rustc-probe-runtime",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "c557e8bc436a0d58765690e52c9b163407c25024c9abd5d4a7b0e061b04fabd9",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "93f393b3fd56d7d7ac980bb6d63f0319d0fea87606032b74f51f371ff1a1c86b",
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}

```

