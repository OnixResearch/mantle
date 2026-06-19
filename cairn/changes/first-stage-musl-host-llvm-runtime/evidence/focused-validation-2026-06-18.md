# Focused validation for first-stage musl host LLVM runtime

Task-ID: V1
Covers: rust_package_planning.source_built_toolchain_closure.first_stage_musl_host_llvm_runtime

## Commands and output

```text
$ /home/brittonr/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin/rustfmt --edition 2024 --check src/rust_source_provider.rs

$ cargo test -p mantle --bin mantle rust_source_provider::tests::materializer_writes_final_provider_output_from_validated_candidate -- --exact --nocapture
warning: /home/brittonr/git/mantle/Cargo.toml: file `/home/brittonr/git/mantle/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
    Blocking waiting for file lock on artifact directory
    Finished `test` profile [unoptimized + debuginfo] target(s) in 6.27s
     Running unittests src/main.rs (/home/brittonr/.cargo-target/debug/deps/mantle-a789c34159ead64f)

running 1 test
test rust_source_provider::tests::materializer_writes_final_provider_output_from_validated_candidate ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 808 filtered out; finished in 0.99s


$ cargo test -p mantle --bin mantle rust_source_provider::tests::materializer_writes_musl_host_provider_metadata_from_route_plan -- --exact --nocapture
warning: /home/brittonr/git/mantle/Cargo.toml: file `/home/brittonr/git/mantle/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.25s
     Running unittests src/main.rs (/home/brittonr/.cargo-target/debug/deps/mantle-a789c34159ead64f)

running 1 test
test rust_source_provider::tests::materializer_writes_musl_host_provider_metadata_from_route_plan ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 808 filtered out; finished in 0.99s


$ git diff --check

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

$ nix run path:/home/brittonr/git/cairn#cairn -- gate proposal first-stage-musl-host-llvm-runtime --root .
{
  "change": "first-stage-musl-host-llvm-runtime",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "098e4cd31431599916f9f87435b38c6814a5e242e8378a0a70858ef783d5821f",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "0be81e809b995fc6302bc85beb8d5d2dde3fa4a76552b9b4314670d5885243cd",
  "stage": "proposal",
  "valid": true,
  "verdict": "PASS"
}

$ nix run path:/home/brittonr/git/cairn#cairn -- gate design first-stage-musl-host-llvm-runtime --root .
{
  "change": "first-stage-musl-host-llvm-runtime",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "4db6d4e32453dcf2692c1547c65136b9d647ea5c1b60d1d009546eec594671b4",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "9d903160dce4c63b841de921c70392004908407aa6141de2e5a533192ddebbe8",
  "stage": "design",
  "valid": true,
  "verdict": "PASS"
}

$ nix run path:/home/brittonr/git/cairn#cairn -- gate tasks first-stage-musl-host-llvm-runtime --root .
{
  "change": "first-stage-musl-host-llvm-runtime",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "e40aee6e9ebed8748eccc2a00f82afee917b1039eb2932527d083c47d7d92808",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "7f4aef0cbadddd021c64493628c476c2fe61392ac14087f497701e807b6c68f2",
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}
```

## Post task-checkbox gate rerun

```text
$ nix run path:/home/brittonr/git/cairn#cairn -- gate tasks first-stage-musl-host-llvm-runtime --root .
{
  "change": "first-stage-musl-host-llvm-runtime",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "c23a5f7c9616ae3fe3f2bde19d123c494e4df8a857efe72b71a79c20182c081a",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "963b60979b48fee7669ac537dace0fefd5205b700c2dbf7e23f9b98001297adb",
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}
```
