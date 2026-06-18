# Focused validation evidence

Task-ID: V2
Covers: r[rust_package_planning.source_built_toolchain_closure.first_stage_musl_proc_macro_runtime]
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
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.21s
     Running unittests src/main.rs (/home/brittonr/.cargo-target/debug/deps/mantle-a789c34159ead64f)

running 1 test
test rust_source_provider::tests::materializer_writes_final_provider_output_from_validated_candidate ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 808 filtered out; finished in 0.73s

$ cargo test -p mantle --bin mantle rust_source_provider::tests::musl_target_tool_candidates_include_source_root_aliases_only -- --exact
warning: /home/brittonr/git/mantle/Cargo.toml: file `/home/brittonr/git/mantle/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.20s
     Running unittests src/main.rs (/home/brittonr/.cargo-target/debug/deps/mantle-a789c34159ead64f)

running 1 test
test rust_source_provider::tests::musl_target_tool_candidates_include_source_root_aliases_only ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 808 filtered out; finished in 0.00s

$ cargo test -p mantle --bin mantle rust_source_provider::tests::materializer_writes_musl_host_provider_metadata_from_route_plan -- --exact
warning: /home/brittonr/git/mantle/Cargo.toml: file `/home/brittonr/git/mantle/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.21s
     Running unittests src/main.rs (/home/brittonr/.cargo-target/debug/deps/mantle-a789c34159ead64f)

running 1 test
test rust_source_provider::tests::materializer_writes_musl_host_provider_metadata_from_route_plan ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 808 filtered out; finished in 0.73s


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
$ nix run path:/home/brittonr/git/cairn#cairn -- gate proposal first-stage-musl-proc-macro-runtime --root .
{
  "change": "first-stage-musl-proc-macro-runtime",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "d827c1ddd446b6787f74458abe7ec900740ae60b8c33d39f91151eab0c86b378",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "72d31694f19ae37742f985f29ce565ce18ffb150a733c1d99b3db8fdbe2ad377",
  "stage": "proposal",
  "valid": true,
  "verdict": "PASS"
}

```

## cairn gate design

```text
$ nix run path:/home/brittonr/git/cairn#cairn -- gate design first-stage-musl-proc-macro-runtime --root .
{
  "change": "first-stage-musl-proc-macro-runtime",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "745272879f8b684273a18aa1893387daf0c3392c6d99a06113e6990cd9ee52f9",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "cb524b11c1e2a2eb59f88344e673dd3a1577b029b9ac1e48a42af343d79afee1",
  "stage": "design",
  "valid": true,
  "verdict": "PASS"
}

```

## cairn gate tasks

```text
$ nix run path:/home/brittonr/git/cairn#cairn -- gate tasks first-stage-musl-proc-macro-runtime --root .
{
  "change": "first-stage-musl-proc-macro-runtime",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "b307839340b1ca54afea8efc86a38a36a81cd346046241b87845020ade97e03b",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "d9495fd059095cf1355252d586cdbb76e7afd322391bbabeb7c51d4eabfab411",
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
$ nix run path:/home/brittonr/git/cairn#cairn -- gate proposal first-stage-musl-proc-macro-runtime --root .
{
  "change": "first-stage-musl-proc-macro-runtime",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "918b42adaf750669abcc5a047323a55bbbd387d264f578c61691d35bacd71a85",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "b6de3d1e45424d46573313a29d958d90f0682f15be573adcb65175b0d61c5327",
  "stage": "proposal",
  "valid": true,
  "verdict": "PASS"
}

```

## post-task cairn gate design

```text
$ nix run path:/home/brittonr/git/cairn#cairn -- gate design first-stage-musl-proc-macro-runtime --root .
{
  "change": "first-stage-musl-proc-macro-runtime",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "5c63ad36b4fae4495b75eb878072af246395d97210966723016b9de90d617896",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "8af876103403537b9764236714da2800f5eaa38cbb9a0a646275dc400efdc2e3",
  "stage": "design",
  "valid": true,
  "verdict": "PASS"
}

```

## post-task cairn gate tasks

```text
$ nix run path:/home/brittonr/git/cairn#cairn -- gate tasks first-stage-musl-proc-macro-runtime --root .
{
  "change": "first-stage-musl-proc-macro-runtime",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "572cebe0700b7a793ad8a6a38f607ee68d8dda59f6746e0a8507798366591e7f",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "1546007a113fc010cd0109c8d5e68e563f67456ed82fc9717b427ec75d575791",
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}

```

