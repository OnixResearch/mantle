# Focused validation evidence

Task-ID: V2
Covers: r[rust_package_planning.source_built_toolchain_closure.first_stage_musl_stage2_prefix_runtime]
Date: 2026-06-18

## rustfmt

```text
$ /home/brittonr/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin/rustfmt --edition 2024 --check src/rust_source_provider.rs

```

## focused rust_source_provider test

```text
$ /home/brittonr/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin/cargo test -p mantle --bin mantle rust_source_provider::tests::materializer_writes_final_provider_output_from_validated_candidate -- --exact --nocapture
warning: /home/brittonr/git/mantle/Cargo.toml: file `/home/brittonr/git/mantle/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.30s
     Running unittests src/main.rs (/home/brittonr/.cargo-target/debug/deps/mantle-a789c34159ead64f)

running 1 test
test rust_source_provider::tests::materializer_writes_final_provider_output_from_validated_candidate ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 808 filtered out; finished in 0.68s


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
$ nix run path:/home/brittonr/git/cairn#cairn -- gate proposal first-stage-musl-stage2-prefix-runtime --root .
{
  "change": "first-stage-musl-stage2-prefix-runtime",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "3966e8021f8962a32808343264d54648a19b6b1293214ec0f26d0ccb1d21755e",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "ee955d9077b7c285035e35bf49baba657d24deeb66dc8ef822871acf52277d37",
  "stage": "proposal",
  "valid": true,
  "verdict": "PASS"
}

```

## cairn gate design

```text
$ nix run path:/home/brittonr/git/cairn#cairn -- gate design first-stage-musl-stage2-prefix-runtime --root .
{
  "change": "first-stage-musl-stage2-prefix-runtime",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "49005868fbf75e3cc5050e1a8e7c0e2e0951b018f79d50807de5708916c19d9d",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "6761c27cfffa3963f476bf9d187455d16d2221b4d6830f6bf218617548e4bb8d",
  "stage": "design",
  "valid": true,
  "verdict": "PASS"
}

```

## cairn gate tasks

```text
$ nix run path:/home/brittonr/git/cairn#cairn -- gate tasks first-stage-musl-stage2-prefix-runtime --root .
{
  "change": "first-stage-musl-stage2-prefix-runtime",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "3fb9445060f489f05bd772c4b2e9daadc375dc36f2ba6ea1e581cb0cf0a01826",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "acd98ce58c5d74c21e0ae8f9fb99a3b74b53788f78b9e965576e28fe08ee4d93",
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
$ nix run path:/home/brittonr/git/cairn#cairn -- gate proposal first-stage-musl-stage2-prefix-runtime --root .
{
  "change": "first-stage-musl-stage2-prefix-runtime",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "955600ee93ee1d3a1e0fb8af23631d727f37f5a3fdfeaf4cdae7a2de32e65444",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "4bf545f77b09c32c910d790e8c5f4445147f91d3ac3757f84ed512fb482843f0",
  "stage": "proposal",
  "valid": true,
  "verdict": "PASS"
}

```

## post-task cairn gate design

```text
$ nix run path:/home/brittonr/git/cairn#cairn -- gate design first-stage-musl-stage2-prefix-runtime --root .
{
  "change": "first-stage-musl-stage2-prefix-runtime",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "c7dc19a7156bc2914a316758e307281606349bbfa7c7e0172e32050f3b852981",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "6374550ded4742497759722e2e4523f56cafd57573ebddaf766f413d5ee7d6af",
  "stage": "design",
  "valid": true,
  "verdict": "PASS"
}

```

## post-task cairn gate tasks

```text
$ nix run path:/home/brittonr/git/cairn#cairn -- gate tasks first-stage-musl-stage2-prefix-runtime --root .
{
  "change": "first-stage-musl-stage2-prefix-runtime",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "b3e0398b2b801be70ad818034b7bd625b1d4be8abfc5883e3a7541b0362edd37",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "5eee027e6369cf07ae1e967b3e46a48d8c454bb73dbb38bee25bfa77c99d18c0",
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}

```
