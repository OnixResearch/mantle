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
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.37s
     Running unittests src/main.rs (/home/brittonr/.cargo-target/debug/deps/mantle-a789c34159ead64f)

running 1 test
test rust_source_provider::tests::materializer_writes_final_provider_output_from_validated_candidate ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 808 filtered out; finished in 1.34s


$ cargo test -p mantle --bin mantle rust_source_provider::tests::materializer_writes_musl_host_provider_metadata_from_route_plan -- --exact --nocapture
warning: /home/brittonr/git/mantle/Cargo.toml: file `/home/brittonr/git/mantle/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.41s
     Running unittests src/main.rs (/home/brittonr/.cargo-target/debug/deps/mantle-a789c34159ead64f)

running 1 test
test rust_source_provider::tests::materializer_writes_musl_host_provider_metadata_from_route_plan ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 808 filtered out; finished in 1.42s


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
  "input_hash": "11682b4597711533906d9d7d7d1267028154bb42c85dbeb2f15373c7c5d8cb91",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "59d44edda89de5eb153f7c7d5c9c5e39af20b730f7bb7eb7714fe0112d206991",
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
  "input_hash": "fdcb247eec2a2a203698b955971f5dcdcd4f99415bbcc4a9616902e6fbdc2a77",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "bbba183787b1f41969eb6571ce38a44d9773efe5546edab858c0c2b80bbb978f",
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
  "input_hash": "9b23d715ae29191cd2d0c940a0b4e4d34101f88e419e4a4fde3ac842ef5a2368",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "c2a987bec1e632e81a66bc2460a574aa72b926eb91213ece3baec2e08b70f88c",
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}
```

## 2026-06-19 addendum: static musl host links and target-aware mrustc

Commit: `b43fedce keep musl host links static and target-aware`

The 2026-06-19 rerun of the real provider moved the frontier from `__popcountdi2` to an earlier mrustc std/proc-macro helper link: the generated script had switched CC/CXX to source-root musl but left mrustc without `--target x86_64-unknown-linux-musl`, linked host helpers dynamically, and provided an empty `libatomic.a`. This addendum validates the committed fix that injects the mrustc target flag, creates a private `__atomic_compare_exchange_16` shim archive, and makes the private C/C++ wrappers append static musl support libraries.

```text
$ /home/brittonr/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin/rustfmt --edition 2024 src/rust_source_provider.rs
$ /home/brittonr/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin/rustfmt --edition 2024 --check src/rust_source_provider.rs
# pueue task 482: completed successfully

$ /home/brittonr/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin/cargo test -p mantle --bin mantle rust_source_provider::tests::materializer_writes_final_provider_output_from_validated_candidate
running 1 test
test rust_source_provider::tests::materializer_writes_final_provider_output_from_validated_candidate ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 808 filtered out; finished in 1.01s

$ /home/brittonr/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin/cargo test -p mantle --bin mantle rust_source_provider::tests::materializer_writes_musl_host_provider_metadata_from_route_plan
running 1 test
test rust_source_provider::tests::materializer_writes_musl_host_provider_metadata_from_route_plan ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 808 filtered out; finished in 1.04s
# pueue task 483

$ git diff --check
# pueue task 484: completed successfully

$ <source-root musl gcc> static atomic-shim probe
# pueue task 485
# produced: target/musl-static-atomic-shim-probe/probe: ELF 64-bit LSB executable, x86-64, version 1 (SYSV), statically linked, not stripped
# readelf found no interpreter, and the probe executed with status 0.
```

Current Cairn validation/gates were rerun into `target/first-stage-musl-host-llvm-runtime-cairn-2026-06-19.txt`:

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

$ nix run path:/home/brittonr/git/cairn#cairn -- gate proposal first-stage-musl-host-llvm-runtime --root .
{
  "change": "first-stage-musl-host-llvm-runtime",
  "issues": [],
  "layout": "cairn",
  "stage": "proposal",
  "valid": true,
  "verdict": "PASS"
}

$ nix run path:/home/brittonr/git/cairn#cairn -- gate design first-stage-musl-host-llvm-runtime --root .
{
  "change": "first-stage-musl-host-llvm-runtime",
  "issues": [],
  "layout": "cairn",
  "stage": "design",
  "valid": true,
  "verdict": "PASS"
}

$ nix run path:/home/brittonr/git/cairn#cairn -- gate tasks first-stage-musl-host-llvm-runtime --root .
{
  "change": "first-stage-musl-host-llvm-runtime",
  "issues": [],
  "layout": "cairn",
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}
```

## 2026-06-19 addendum: minicargo build-script OUT_DIR normalization

Commit: `96927bf8 share minicargo build outputs with target crates`

Rerun 6 proved the static-shared link guard cleared LLVM `libLTO.so` and advanced into the translated cargo graph, where `libsqlite3-sys` failed because mrustc minicargo ran the build script under `output/cargo-build/host/build_libsqlite3-sys-...` but compiled the target crate with `OUT_DIR=output/cargo-build/build_libsqlite3-sys-...`. This addendum validates the generated-script fix that patches minicargo before it is built so crates with build scripts read the same generated-output directory that their script run populated.

```text
$ /home/brittonr/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin/rustfmt --edition 2024 src/rust_source_provider.rs
$ /home/brittonr/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin/rustfmt --edition 2024 --check src/rust_source_provider.rs

$ CARGO_BUILD_RUSTC_WRAPPER= RUSTC=/home/brittonr/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin/rustc /home/brittonr/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin/cargo test -p mantle --bin mantle rust_source_provider::tests::materializer_writes_final_provider_output_from_validated_candidate
running 1 test
test rust_source_provider::tests::materializer_writes_final_provider_output_from_validated_candidate ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 808 filtered out; finished in 0.90s

$ CARGO_BUILD_RUSTC_WRAPPER= RUSTC=/home/brittonr/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin/rustc /home/brittonr/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin/cargo test -p mantle --bin mantle rust_source_provider::tests::materializer_writes_musl_host_provider_metadata_from_route_plan
running 1 test
test rust_source_provider::tests::materializer_writes_musl_host_provider_metadata_from_route_plan ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 808 filtered out; finished in 0.97s

$ git diff --check
# pueue task 484 completed successfully.
```

## 2026-06-19 addendum: post-OUT_DIR Cairn validation

Transcript saved at `target/first-stage-musl-host-llvm-runtime-cairn-2026-06-19-outdir.txt`.

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
  "input_hash": "c8a5c3c500a2230c782c47fe3a435bec136a5509836925c000fc6ca17c57cece",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "e2ab9446f2b7537774bae8b09f87d3d41bbcf1eeac261e12554ba209ba8537ce",
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
  "input_hash": "a9f0f76722afb0067002fa044efa216b62ee983957a5c9b9536c188263db27b0",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "9ca34c3606b3957e057f5a5f1e2aa7dcbdb48c008bd1ea840034b3f0c2314cf4",
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
  "input_hash": "8db66fa1cc1abff5b9b62f39ffb51ad184983f7289a8f98151aef561de018fbe",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "7763d4a03acb2cc05c76b2f0d0a1ccba951a4d10546338ed766f4b5baa448aeb",
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}
```

## 2026-06-19 addendum: static rustc sysroot wrappers

Commit: `04e93227 keep static rustc wrappers sysroot-explicit`

Rerun 7 cleared the minicargo `OUT_DIR` frontier and reached `run_rustc`, where the mrustc-built static `rustc` panicked during implicit sysroot discovery with `Failed finding sysroot: "dladdr failed"`. This addendum validates the generated-script fix that rewrites the `run_rustc` stage-1, stage-2, and final rustc wrapper rules to pass an explicit `--sysroot`, and preserves that explicit sysroot in Mantle's normalized first-stage provider wrapper.

```text
$ /home/brittonr/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin/rustfmt --edition 2024 src/rust_source_provider.rs
# pueue tasks 505, 509, and 520 completed successfully after incremental edits.

$ cargo test -p mantle --bin mantle first_stage_wrapper_normalization -- --nocapture
running 2 tests
test rust_source_provider::tests::first_stage_wrapper_normalization_rejects_missing_rustc_binary ... ok
test rust_source_provider::tests::first_stage_wrapper_normalization_makes_mrustc_rustc_env_clear_safe ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 807 filtered out; finished in 0.00s
# pueue task 522

$ cargo test -p mantle --bin mantle materializer_writes_musl_host_provider_metadata_from_route_plan -- --nocapture
running 1 test
test rust_source_provider::tests::materializer_writes_musl_host_provider_metadata_from_route_plan ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 808 filtered out; finished in 0.83s
# pueue task 510

$ cargo test -p mantle --bin mantle first_stage_provider_candidate_rejects_tampered_artifact -- --nocapture
running 1 test
test rust_source_provider::tests::first_stage_provider_candidate_rejects_tampered_artifact ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 808 filtered out; finished in 0.80s
# pueue task 512

$ /home/brittonr/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin/rustfmt --edition 2024 --check src/rust_source_provider.rs
$ git diff --check
# pueue task 527 completed successfully.
```

Fresh real-provider rerun `526` is running from commit `04e93227c3a20c83db18c55068386d9f91938e2f` at `target/rust-source-provider-musl-host-route-run-rustc-sysroot-rerun9-2026-06-19`; its result belongs in `provider-rerun-2026-06-18.md` after completion.

## 2026-06-19 addendum: post-sysroot Cairn validation

Transcript saved at `target/first-stage-musl-host-llvm-runtime-cairn-2026-06-19-sysroot-wrapper.txt`.

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

$ nix run path:/home/brittonr/git/cairn#cairn -- gate proposal first-stage-musl-host-llvm-runtime --root .
{
  "change": "first-stage-musl-host-llvm-runtime",
  "issues": [],
  "layout": "cairn",
  "stage": "proposal",
  "valid": true,
  "verdict": "PASS"
}

$ nix run path:/home/brittonr/git/cairn#cairn -- gate design first-stage-musl-host-llvm-runtime --root .
{
  "change": "first-stage-musl-host-llvm-runtime",
  "issues": [],
  "layout": "cairn",
  "stage": "design",
  "valid": true,
  "verdict": "PASS"
}

$ nix run path:/home/brittonr/git/cairn#cairn -- gate tasks first-stage-musl-host-llvm-runtime --root .
{
  "change": "first-stage-musl-host-llvm-runtime",
  "issues": [],
  "layout": "cairn",
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}
```

## 2026-06-19 addendum: symlink-shaped static rustc sysroot lookup

Commits: `d9f9a8db make static rustc sysroot lookup symlink-shaped`, `0d4b5ba3 preserve shell argv0 in rustc sysroot wrappers`

Rerun 9 proved that forwarding `--sysroot` is not enough for the mrustc-built static rustc: Rust 1.90 still calls `default_sysroot()`, and its non-dylib path only avoids `dladdr` when `argv[0]` names a symlink under the sysroot `bin/` directory. This addendum validates the generated-script and provider-wrapper fix that creates a sysroot-local symlink to the static rustc binary and execs that symlink, preserving both explicit `--sysroot` and Rust's `env::args().next()` symlink sysroot discovery path. The follow-up `0d4b5ba3` keeps the Makefile recipe from expanding `$0` too early by writing `$$0` into the generated shell script.

```text
$ /home/brittonr/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin/rustfmt --edition 2024 src/rust_source_provider.rs
# pueue tasks 531 and 539 completed successfully.

$ cargo test -p mantle --bin mantle first_stage_wrapper_normalization -- --nocapture
running 2 tests
test rust_source_provider::tests::first_stage_wrapper_normalization_rejects_missing_rustc_binary ... ok
test rust_source_provider::tests::first_stage_wrapper_normalization_makes_mrustc_rustc_env_clear_safe ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 807 filtered out; finished in 0.00s
# pueue tasks 533 and 541

$ cargo test -p mantle --bin mantle materializer_writes_musl_host_provider_metadata_from_route_plan -- --nocapture
running 1 test
test rust_source_provider::tests::materializer_writes_musl_host_provider_metadata_from_route_plan ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 808 filtered out; finished in 0.98s
# pueue tasks 532 and 540

$ cargo test -p mantle --bin mantle first_stage_provider_candidate_rejects_tampered_artifact -- --nocapture
running 1 test
test rust_source_provider::tests::first_stage_provider_candidate_rejects_tampered_artifact ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 808 filtered out; finished in 0.79s
# pueue tasks 532 and 540

$ /home/brittonr/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin/rustfmt --edition 2024 --check src/rust_source_provider.rs
$ git diff --check
# pueue tasks 534 and 540 completed successfully.
```

Rerun `537` was stopped because it launched from the pre-quoting `d9f9a8db` commit and would generate `.bin` instead of `$0.bin`. Fresh real-provider rerun `543` is running from commit `0d4b5ba333c5bb7e84bd23c4b9ea214f4ae005d8` at `target/rust-source-provider-musl-host-route-run-rustc-symlink-rerun11-2026-06-19`; its result belongs in `provider-rerun-2026-06-18.md` after completion.

## 2026-06-19 addendum: post-symlink Cairn validation

Transcript saved at `target/first-stage-musl-host-llvm-runtime-cairn-2026-06-19-symlink-wrapper.txt`.

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

$ nix run path:/home/brittonr/git/cairn#cairn -- gate proposal first-stage-musl-host-llvm-runtime --root .
{
  "change": "first-stage-musl-host-llvm-runtime",
  "issues": [],
  "layout": "cairn",
  "stage": "proposal",
  "valid": true,
  "verdict": "PASS"
}

$ nix run path:/home/brittonr/git/cairn#cairn -- gate design first-stage-musl-host-llvm-runtime --root .
{
  "change": "first-stage-musl-host-llvm-runtime",
  "issues": [],
  "layout": "cairn",
  "stage": "design",
  "valid": true,
  "verdict": "PASS"
}

$ nix run path:/home/brittonr/git/cairn#cairn -- gate tasks first-stage-musl-host-llvm-runtime --root .
{
  "change": "first-stage-musl-host-llvm-runtime",
  "issues": [],
  "layout": "cairn",
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}
```

## 2026-06-19 addendum: explicit sysroot source patch

Commit: `e5a42184 avoid static rustc default sysroot probe`

Rerun 11 showed that generated wrappers passing `--sysroot` still panic because Rust 1.90 eagerly fills `Sysroot.default` with `filesearch::default_sysroot()` even when `Sysroot.explicit` is set. The new generated source patch runs after `RUSTCSRC` extraction and before translated `output/rustc`, replacing the one-line `Sysroot::new` initializer with a `match explicit` that reuses the explicit path as `default` when present.

```text
$ /home/brittonr/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin/rustfmt --edition 2024 src/rust_source_provider.rs
# pueue tasks 546, 548, and 551 completed successfully.

$ cargo test -p mantle --bin mantle materializer_writes_musl_host_provider_metadata_from_route_plan -- --nocapture
running 1 test
test rust_source_provider::tests::materializer_writes_musl_host_provider_metadata_from_route_plan ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 808 filtered out; finished in 0.75s
# pueue task 552

$ cargo test -p mantle --bin mantle first_stage_provider_candidate_rejects_tampered_artifact -- --nocapture
running 1 test
test rust_source_provider::tests::first_stage_provider_candidate_rejects_tampered_artifact ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 808 filtered out; finished in 0.74s
# pueue task 553

$ cargo test -p mantle --bin mantle first_stage_wrapper_normalization -- --nocapture
running 2 tests
test rust_source_provider::tests::first_stage_wrapper_normalization_rejects_missing_rustc_binary ... ok
test rust_source_provider::tests::first_stage_wrapper_normalization_makes_mrustc_rustc_env_clear_safe ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 807 filtered out; finished in 0.00s
# pueue task 553

$ /home/brittonr/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin/rustfmt --edition 2024 --check src/rust_source_provider.rs
$ git diff --check
# pueue task 553 completed successfully.
```

Fresh real-provider rerun `556` from commit `e5a421844ba53ba965b73d9320b57bea4485f804` at `target/rust-source-provider-musl-host-route-explicit-sysroot-rerun12-2026-06-19` is now recorded in `provider-rerun-2026-06-18.md` as the pthread TLS-key frontier.

## 2026-06-19 addendum: post-explicit-sysroot Cairn validation

Transcript saved at `target/first-stage-musl-host-llvm-runtime-cairn-2026-06-19-explicit-sysroot.txt`.

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

$ nix run path:/home/brittonr/git/cairn#cairn -- gate proposal first-stage-musl-host-llvm-runtime --root .
{
  "change": "first-stage-musl-host-llvm-runtime",
  "issues": [],
  "layout": "cairn",
  "stage": "proposal",
  "valid": true,
  "verdict": "PASS"
}

$ nix run path:/home/brittonr/git/cairn#cairn -- gate design first-stage-musl-host-llvm-runtime --root .
{
  "change": "first-stage-musl-host-llvm-runtime",
  "issues": [],
  "layout": "cairn",
  "stage": "design",
  "valid": true,
  "verdict": "PASS"
}

$ nix run path:/home/brittonr/git/cairn#cairn -- gate tasks first-stage-musl-host-llvm-runtime --root .
{
  "change": "first-stage-musl-host-llvm-runtime",
  "issues": [],
  "layout": "cairn",
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}
```

## 2026-06-19 addendum: static musl rustc thread limit

Commit: `2b8d31a1 limit static musl rustc bootstrap threads`

Rerun 12 proved the explicit-sysroot Rust source patch cleared the previous `dladdr` panic and moved into stage-1 `core` compilation, where the static musl `rustc` aborted with `fatal runtime error: out of TLS keys, aborting`. This addendum validates the generated-script fix that patches mrustc minicargo before it is built so rustc invocations include `-Z threads=1` for the static musl compiler host route.

Baseline before the change:

```text
$ cargo test -p mantle --bin mantle first_stage_wrapper_normalization -- --nocapture
# pueue task 716
running 2 tests
test rust_source_provider::tests::first_stage_wrapper_normalization_rejects_missing_rustc_binary ... ok
test rust_source_provider::tests::first_stage_wrapper_normalization_makes_mrustc_rustc_env_clear_safe ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 807 filtered out; finished in 0.01s
```

Focused validation after the change:

```text
$ cargo test -p mantle --bin mantle materializer_writes_musl_host_provider_metadata_from_route_plan -- --nocapture
# pueue task 695
running 1 test
test rust_source_provider::tests::materializer_writes_musl_host_provider_metadata_from_route_plan ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 808 filtered out; finished in 1.06s

$ cargo test -p mantle --bin mantle first_stage_provider_candidate_rejects_tampered_artifact -- --nocapture
# pueue task 697
running 1 test
test rust_source_provider::tests::first_stage_provider_candidate_rejects_tampered_artifact ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 808 filtered out; finished in 0.89s

$ cargo test -p mantle --bin mantle first_stage_wrapper_normalization -- --nocapture
# pueue task 698
running 2 tests
test rust_source_provider::tests::first_stage_wrapper_normalization_makes_mrustc_rustc_env_clear_safe ... ok
test rust_source_provider::tests::first_stage_wrapper_normalization_rejects_missing_rustc_binary ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 807 filtered out; finished in 0.01s

$ rustfmt --edition 2024 --check src/rust_source_provider.rs
$ git diff --check
# pueue task 699 completed successfully.
```

Current Cairn validation/gates were rerun into `target/first-stage-musl-host-llvm-runtime-cairn-2026-06-19-thread-limit.txt`:

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

$ nix run path:/home/brittonr/git/cairn#cairn -- gate proposal first-stage-musl-host-llvm-runtime --root .
{
  "change": "first-stage-musl-host-llvm-runtime",
  "issues": [],
  "layout": "cairn",
  "stage": "proposal",
  "valid": true,
  "verdict": "PASS"
}

$ nix run path:/home/brittonr/git/cairn#cairn -- gate design first-stage-musl-host-llvm-runtime --root .
{
  "change": "first-stage-musl-host-llvm-runtime",
  "issues": [],
  "layout": "cairn",
  "stage": "design",
  "valid": true,
  "verdict": "PASS"
}

$ nix run path:/home/brittonr/git/cairn#cairn -- gate tasks first-stage-musl-host-llvm-runtime --root .
{
  "change": "first-stage-musl-host-llvm-runtime",
  "issues": [],
  "layout": "cairn",
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}
```

Real-provider rerun `715` from commit `2b8d31a169b9e90c8c725d944755317b3238f2ad` at `target/rust-source-provider-musl-host-route-minicargo-threads-rerun13-2026-06-19` is now recorded in `provider-rerun-2026-06-18.md` as proof that `-Z threads=1` was active but insufficient to clear pthread TLS-key exhaustion.

## 2026-06-20 addendum: static musl pthread TLS-key shim

Commit: `1a854f24 keep static rustc off musl pthread TLS keys`

Rerun 13 showed the failing stage-1 `core` compile already carried `-Z force-unstable-if-unmarked -Z threads=1`, so the TLS-key abort was not solely rustc worker parallelism. This addendum validates the narrower runtime-object fix: the existing executable-only source-root musl compatibility object now defines `pthread_key_create`, `pthread_getspecific`, `pthread_setspecific`, and `pthread_key_delete` with a bounded `MANTLE_PTHREAD_TLS_KEY_CAPACITY` pool backed by ELF TLS. The target `cc` wrapper still skips that object for compile-only and `-shared|-dynamiclib` commands.

Baseline before the change:

```text
$ cargo test -p mantle --bin mantle first_stage_wrapper_normalization -- --nocapture
# pueue task 919
running 2 tests
test rust_source_provider::tests::first_stage_wrapper_normalization_rejects_missing_rustc_binary ... ok
test rust_source_provider::tests::first_stage_wrapper_normalization_makes_mrustc_rustc_env_clear_safe ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 807 filtered out; finished in 0.00s
```

Focused validation after the change:

```text
$ cargo test -p mantle --bin mantle materializer_writes_musl_host_provider_metadata_from_route_plan -- --nocapture
# pueue task 971
running 1 test
test rust_source_provider::tests::materializer_writes_musl_host_provider_metadata_from_route_plan ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 808 filtered out; finished in 0.97s

$ cargo test -p mantle --bin mantle first_stage_provider_candidate_rejects_tampered_artifact -- --nocapture
# pueue task 972
running 1 test
test rust_source_provider::tests::first_stage_provider_candidate_rejects_tampered_artifact ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 808 filtered out; finished in 0.78s

$ cargo test -p mantle --bin mantle first_stage_wrapper_normalization -- --nocapture
# pueue task 954
running 2 tests
test rust_source_provider::tests::first_stage_wrapper_normalization_rejects_missing_rustc_binary ... ok
test rust_source_provider::tests::first_stage_wrapper_normalization_makes_mrustc_rustc_env_clear_safe ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 807 filtered out; finished in 0.00s

$ <source-root musl gcc> static pthread TLS-key shim smoke
# pueue task 966
static musl TLS shim smoke passed: /home/brittonr/git/mantle/target/static-musl-pthread-tls-shim-smoke-2026-06-20/tls-smoke

$ rustfmt --edition 2024 --check src/rust_source_provider.rs
$ git diff --check -- src/rust_source_provider.rs
# pueue task 974
rustfmt and diff checks passed
```

Real-provider rerun `983` from commit `1a854f2437d34a32636fe06c1cbacccf9f03e894` at `target/rust-source-provider-musl-host-route-pthread-tls-shim-rerun14-2026-06-19` is now recorded in `provider-rerun-2026-06-18.md` as the strong pthread-symbol collision frontier.

## 2026-06-20 addendum: pthread TLS-key shim linker wrapping

Commit: `1f178373 avoid pthread TLS shim symbol collisions`

Rerun 14 showed that defining strong `pthread_key_*` / `pthread_*specific` symbols in the executable-only compatibility object collides with musl libc archive members when the static link pulls libc's pthread implementation. This addendum validates the narrower interposition fix: the compatibility object now exports `__wrap_pthread_key_create`, `__wrap_pthread_key_delete`, `__wrap_pthread_getspecific`, and `__wrap_pthread_setspecific`, while the target linker wrapper adds matching GNU ld `--wrap` flags only for executable static-support links. Compile-only and `-shared|-dynamiclib` skips remain intact.

Baseline before the change:

```text
$ cargo test -p mantle --bin mantle first_stage_wrapper_normalization -- --nocapture
# pueue task 1018
running 2 tests
test rust_source_provider::tests::first_stage_wrapper_normalization_rejects_missing_rustc_binary ... ok
test rust_source_provider::tests::first_stage_wrapper_normalization_makes_mrustc_rustc_env_clear_safe ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 807 filtered out; finished in 0.00s
```

Focused validation after the change:

```text
$ cargo test -p mantle --bin mantle first_stage_wrapper_normalization -- --nocapture
# pueue task 1027
running 2 tests
test rust_source_provider::tests::first_stage_wrapper_normalization_rejects_missing_rustc_binary ... ok
test rust_source_provider::tests::first_stage_wrapper_normalization_makes_mrustc_rustc_env_clear_safe ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 807 filtered out; finished in 0.01s

$ cargo test -p mantle --bin mantle materializer_writes_musl_host_provider_metadata_from_route_plan -- --nocapture
# pueue task 1023
running 1 test
test rust_source_provider::tests::materializer_writes_musl_host_provider_metadata_from_route_plan ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 808 filtered out; finished in 1.27s

$ cargo test -p mantle --bin mantle first_stage_provider_candidate_rejects_tampered_artifact -- --nocapture
# pueue task 1023
running 1 test
test rust_source_provider::tests::first_stage_provider_candidate_rejects_tampered_artifact ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 808 filtered out; finished in 0.96s

$ <source-root musl gcc> static pthread TLS-key --wrap smoke
# pueue task 1029
wrap smoke ok: /home/brittonr/git/mantle/target/static-musl-pthread-wrap-smoke-2026-06-20/smoke

$ rustfmt --edition 2024 --check src/rust_source_provider.rs
$ git diff --check -- src/rust_source_provider.rs
# pueue task 1030 completed successfully.
```

Real-provider rerun `1048` from commit `1f17837365c10728ca296190de2648cf3916f729` at `target/rust-source-provider-musl-host-route-pthread-wrap-rerun15-2026-06-20` is now recorded in `provider-rerun-2026-06-18.md` as the final rustc host/proc-macro loading frontier.

## 2026-06-20 addendum: final rustc host/proc-macro prefix normalization

Commit: `142955c5 keep final rustc proc macros on prefix-2`

Rerun 15 proved the `--wrap` pthread TLS-key shim cleared the earlier static-musl runtime frontier and advanced into the final Cargo build for rustc, where the stage-2 compiler could not load `tracing_attributes` even though Cargo had built the proc-macro shared object. This addendum validates the generated Makefile patch that rewrites final `CARGO_ENV_RUSTC` so both `PROXY_RUSTC` and `PROXY_MRUSTC` use `$(BINDIR_2)rustc`; final host/proc-macro artifacts are then built for the same prefix-2 compiler that later consumes them.

Focused validation after the change:

```text
$ cargo test -p mantle --bin mantle first_stage_wrapper_normalization -- --nocapture
# pueue task 1165
running 2 tests
test rust_source_provider::tests::first_stage_wrapper_normalization_makes_mrustc_rustc_env_clear_safe ... ok
test rust_source_provider::tests::first_stage_wrapper_normalization_rejects_missing_rustc_binary ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 807 filtered out; finished in 0.00s

$ cargo test -p mantle --bin mantle materializer_writes_musl_host_provider_metadata_from_route_plan -- --nocapture
# pueue task 1175
running 1 test
test rust_source_provider::tests::materializer_writes_musl_host_provider_metadata_from_route_plan ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 808 filtered out; finished in 0.77s

$ cargo test -p mantle --bin mantle first_stage_wrapper_normalization -- --nocapture
# pueue task 1174
running 2 tests
test rust_source_provider::tests::first_stage_wrapper_normalization_rejects_missing_rustc_binary ... ok
test rust_source_provider::tests::first_stage_wrapper_normalization_makes_mrustc_rustc_env_clear_safe ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 807 filtered out; finished in 0.00s

$ cargo test -p mantle --bin mantle first_stage_provider_candidate_rejects_tampered_artifact -- --nocapture
# pueue task 1174
running 1 test
test rust_source_provider::tests::first_stage_provider_candidate_rejects_tampered_artifact ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 808 filtered out; finished in 0.59s

$ /home/brittonr/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin/rustfmt --edition 2024 --check src/rust_source_provider.rs
$ git diff --check -- src/rust_source_provider.rs
# pueue task 1176 completed successfully.
```

Real-provider rerun `1180` from commit `142955c51f294919557362ab82fe95523269f6d9` at `target/rust-source-provider-musl-host-route-prefix2-procmacro-rerun16-2026-06-20` is now recorded in `provider-rerun-2026-06-18.md`; it confirms the prefix-2 `PROXY_MRUSTC` Makefile rewrite landed but was insufficient to clear the final `tracing_attributes` proc-macro loading frontier.

## 2026-06-20 addendum: post-prefix2 Cairn validation

Transcript saved at `target/first-stage-musl-host-llvm-runtime-cairn-2026-06-20-prefix2.txt`.

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
  "input_hash": "e1e7b6446bdbd01d6b171825ef7f69daa7feb86f34807d77973360ffa59f64bf",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "a26676680c3eb5a9aa9fa929059c9067df86e4de827d1d02eef3cdd557e25013",
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
  "input_hash": "0b423ff5e2e1848378f00f1668efcd8a4198b9eb7d0f133e1463fe779c1ca745",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "7a5f3b37948b9ee49e1648c25dfefc18f5ae861c80fb007f8aff4ca3ab4ee0bc",
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
  "input_hash": "17eb8f793d63c6f66199c173ce7b157d75fdedb27a48f986559ffef05c5ae365",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "ee964ecd6fe369f8908e5ba1b59e78c6fa29f200d273b4faa98b8dc4d1619cb0",
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}
```

## 2026-06-20 addendum: post-rerun16 Cairn validation

Transcript saved at `target/first-stage-musl-host-llvm-runtime-cairn-2026-06-20-rerun16.txt`.

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

$ nix run path:/home/brittonr/git/cairn#cairn -- gate proposal first-stage-musl-host-llvm-runtime --root .
{
  "change": "first-stage-musl-host-llvm-runtime",
  "issues": [],
  "layout": "cairn",
  "stage": "proposal",
  "valid": true,
  "verdict": "PASS"
}

$ nix run path:/home/brittonr/git/cairn#cairn -- gate design first-stage-musl-host-llvm-runtime --root .
{
  "change": "first-stage-musl-host-llvm-runtime",
  "issues": [],
  "layout": "cairn",
  "stage": "design",
  "valid": true,
  "verdict": "PASS"
}

$ nix run path:/home/brittonr/git/cairn#cairn -- gate tasks first-stage-musl-host-llvm-runtime --root .
{
  "change": "first-stage-musl-host-llvm-runtime",
  "issues": [],
  "layout": "cairn",
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}
```

## 2026-06-20 addendum: dynamic mrustc-built rustc for proc macros

Commit: `698bc0eb let mrustc rustc load proc macros dynamically`

Rerun 16 proved prefix-2 `PROXY_MRUSTC` was necessary but insufficient: the final rustc build still failed while consuming `tracing_attributes`. Scratch diagnosis proved the mrustc-built static musl rustc could not `dlopen` proc-macro dylibs. This addendum validates the generated-script fix that links only the mrustc-built rustc executable dynamically, keeps the pthread TLS-key `--wrap` shim, compiles Mantle's compat object as PIC with no-op musl `backtrace*` stubs, recognizes mrustc's `rustc_main` response-file link, and makes prefix-s/prefix-2 wrappers launch through source-root musl `libc.so` with the runtime library path.

Focused validation after the change:

```text
$ /home/brittonr/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin/rustfmt --edition 2024 src/rust_source_provider.rs
$ /home/brittonr/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin/rustfmt --edition 2024 --check src/rust_source_provider.rs

$ cargo test -p mantle --bin mantle first_stage_wrapper_normalization -- --nocapture
# pueue task 31
running 2 tests
test rust_source_provider::tests::first_stage_wrapper_normalization_rejects_missing_rustc_binary ... ok
test rust_source_provider::tests::first_stage_wrapper_normalization_makes_mrustc_rustc_env_clear_safe ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 807 filtered out; finished in 0.01s

$ cargo test -p mantle --bin mantle materializer_writes_musl_host_provider_metadata_from_route_plan -- --nocapture
# pueue task 27
running 1 test
test rust_source_provider::tests::materializer_writes_musl_host_provider_metadata_from_route_plan ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 808 filtered out; finished in 0.75s

$ cargo test -p mantle --bin mantle first_stage_provider_candidate_rejects_tampered_artifact -- --nocapture
# pueue task 27
running 1 test
test rust_source_provider::tests::first_stage_provider_candidate_rejects_tampered_artifact ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 808 filtered out; finished in 0.72s

$ git diff --check -- src/rust_source_provider.rs
# pueue task 27 completed successfully.
```

Scratch proof from rerun 16 is recorded in `provider-rerun-2026-06-18.md` and includes pueue task `24` passing the exact saved `tracing` command with status `0`.

## 2026-06-21 addendum: LLVM vt_gen generated header target

Commit under edit: working tree after `928d35a0 generate LLVM headers during musl bootstrap`

Rerun 21 proved `llvm-headers llvm-config` cleared the missing `Attributes.inc` frontier but still failed in `rustc_llvm` because `MachineValueType.h` includes `llvm/CodeGen/GenVT.inc`. The generated LLVM build tree showed the narrow CMake target is `vt_gen`, and pueue tasks `70` and `108` proved that target generates `GenVT.inc` and clears the exact failed `PassWrapper.cpp` C++ command. This addendum validates the generated-script change that builds `llvm-headers vt_gen llvm-config` before invoking Rust's `rustc_llvm` bridge.

Baseline before the change:

```text
$ cargo test -p mantle --bin mantle materializer_writes_musl_host_provider_metadata_from_route_plan -- --nocapture
# pueue task 87
running 1 test
test rust_source_provider::tests::materializer_writes_musl_host_provider_metadata_from_route_plan ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 808 filtered out; finished in 0.86s
```

Directional scratch proof from rerun 21:

```text
$ /nix/store/...-gnumake-4.4.1/bin/make -C target/rust-source-provider-musl-host-route-llvm-headers-rerun21-2026-06-21/.../rustc-1.90.0-src/build -j 4 vt_gen
# pueue task 70
[100%] Building GenVT.inc...
[100%] Built target vt_gen
GenVT generated: 43121 bytes

$ <exact failed cc-rs C++ command for llvm-wrapper/PassWrapper.cpp>
# pueue task 108
PassWrapper compile cleared after vt_gen
```

Focused validation after the change:

```text
$ /home/brittonr/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin/rustfmt --edition 2024 src/rust_source_provider.rs
# pueue task 91 completed successfully

$ cargo test -p mantle --bin mantle materializer_writes_musl_host_provider_metadata_from_route_plan -- --nocapture
# pueue task 93
running 1 test
test rust_source_provider::tests::materializer_writes_musl_host_provider_metadata_from_route_plan ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 808 filtered out; finished in 1.16s

$ /home/brittonr/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin/rustfmt --edition 2024 --check src/rust_source_provider.rs && git diff --check -- src/rust_source_provider.rs
# pueue task 100 completed successfully
```

Real-provider rerun `17` through `21` frontier movement is recorded in `provider-rerun-2026-06-18.md`; rerun 21 is not a full success claim, only proof that the next committed rerun must include `vt_gen` to materialize `GenVT.inc`.

## 2026-06-20 addendum: post-dynamic-rustc Cairn validation

Transcript saved at `target/first-stage-musl-host-llvm-runtime-cairn-2026-06-20-dynamic-rustc.txt`.

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

$ nix run path:/home/brittonr/git/cairn#cairn -- gate proposal first-stage-musl-host-llvm-runtime --root .
{
  "change": "first-stage-musl-host-llvm-runtime",
  "issues": [],
  "layout": "cairn",
  "stage": "proposal",
  "valid": true,
  "verdict": "PASS"
}

$ nix run path:/home/brittonr/git/cairn#cairn -- gate design first-stage-musl-host-llvm-runtime --root .
{
  "change": "first-stage-musl-host-llvm-runtime",
  "issues": [],
  "layout": "cairn",
  "stage": "design",
  "valid": true,
  "verdict": "PASS"
}

$ nix run path:/home/brittonr/git/cairn#cairn -- gate tasks first-stage-musl-host-llvm-runtime --root .
{
  "change": "first-stage-musl-host-llvm-runtime",
  "issues": [],
  "layout": "cairn",
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}
```

## 2026-06-21 addendum: post-vt-gen Cairn validation

Transcript saved at `target/first-stage-musl-host-llvm-runtime-cairn-2026-06-21-vt-gen.txt`.

```text
$ nix run path:/home/brittonr/git/cairn#cairn -- validate --root .
{
  "change_issues": [],
  "changes": 2,
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "spec_issues": [],
  "specs_validated": 7,
  "valid": true
}

$ nix run path:/home/brittonr/git/cairn#cairn -- gate proposal first-stage-musl-host-llvm-runtime --root .
{
  "change": "first-stage-musl-host-llvm-runtime",
  "issues": [],
  "layout": "cairn",
  "stage": "proposal",
  "valid": true,
  "verdict": "PASS"
}

$ nix run path:/home/brittonr/git/cairn#cairn -- gate design first-stage-musl-host-llvm-runtime --root .
{
  "change": "first-stage-musl-host-llvm-runtime",
  "issues": [],
  "layout": "cairn",
  "stage": "design",
  "valid": true,
  "verdict": "PASS"
}

$ nix run path:/home/brittonr/git/cairn#cairn -- gate tasks first-stage-musl-host-llvm-runtime --root .
{
  "change": "first-stage-musl-host-llvm-runtime",
  "issues": [],
  "layout": "cairn",
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}
```

## 2026-06-21 addendum: LLVM static archive targets for `rustc_llvm`

Commit: `a7d858cb build rustc LLVM static archives during musl bootstrap`

Rerun 22 proved `llvm-headers vt_gen llvm-config` cleared the generated-header frontiers but failed in `rustc_llvm` because `llvm-config --link-static --libs aarch64 arm asmparser bitreader bitwriter coverage instrumentation ipo linker lto x86` rejected missing static LLVM component archives. This addendum validates the generated-script fix that adds the exact CMake static archive targets while keeping optional LLVM shared tools/modules disabled.

```text
$ /home/brittonr/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin/rustfmt --edition 2024 --check src/rust_source_provider.rs
$ cargo test -p mantle --bin mantle rust_source_provider::tests::materializer_writes_musl_host_provider_metadata_from_route_plan -- --exact --nocapture
# pueue task 277
running 1 test
test rust_source_provider::tests::materializer_writes_musl_host_provider_metadata_from_route_plan ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 808 filtered out; finished in 1.29s

$ cargo test -p mantle --bin mantle rust_source_provider::tests::materializer_writes_final_provider_output_from_validated_candidate -- --exact --nocapture
# pueue task 279
running 1 test
test rust_source_provider::tests::materializer_writes_final_provider_output_from_validated_candidate ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 808 filtered out; finished in 1.03s

$ git diff --check
# pueue tasks 284 and 346 completed successfully.
```

Directional scratch proof against rerun 22's configured LLVM build tree:

```text
$ /nix/store/...-gnumake-4.4.1/bin/make -C .../rustc-1.90.0-src/build -j 4 <static archive target list>
# pueue task 257
[100%] Built target LLVMX86TargetMCA

$ ./bin/llvm-config --link-static --libs aarch64 arm asmparser bitreader bitwriter coverage instrumentation ipo linker lto x86
llvm static archive target proof passed
1217 /tmp/mantle-rerun22-llvm-config-static-libs.txt
```

The direct `build_rustc_llvm_run` binary was not used as proof because replaying it outside minicargo requires the complete generated Cargo environment; the narrow failed `llvm-config` command is the proven frontier here.

Cairn validation transcript saved at `target/first-stage-musl-host-llvm-runtime-cairn-2026-06-21-static-archives.txt`:

```text
$ nix run path:/home/brittonr/git/cairn#cairn -- validate --root .
{
  "change_issues": [],
  "changes": 2,
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "spec_issues": [],
  "specs_validated": 7,
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
  "input_hash": "b4a2256612c5aa6126aa10a6571e06689cac71a08f89fa3294f3d1fcf6c79045",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "818158f4917262f3d08f019c0259b27378d71e0ece5933bcda1bbd28a0f4dc23",
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
  "input_hash": "2ed88c45bc561fa34e75d286e18cb3db04c61b5dfdfca4bcb045d4be57001c8b",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "c9e492472392290029719d886d6202c7d40481fe95b18ddfb418c007b21f74d0",
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
  "input_hash": "d7583dac75797937cd724ba5e318dc0d2faaaff56afb7d3ca75f4a7f40e93f4d",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "d64debb1d9688ab30b01a54803592c4c36ee12c9221f191754063cd1a3126292",
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}
```

## 2026-06-21 addendum: absolute `rcrt1.o` CRT normalization for final static rustc

Commit: `f00b2b6e normalize absolute musl CRT paths`

Rerun 23 proved the static LLVM archive target list cleared the `rustc_llvm` frontier: Cargo completed the `rustc_llvm` build script and produced `librustc_llvm.rlib`. The next failure moved to the final Cargo-built static `rustc`, which copied into `run_rustc/output/prefix/bin/rustc` and then segfaulted on Cargo's `rustc -vV` probe. GDB showed the crash before Rust `main` in musl `_start_c`, dereferencing a null `_DYNAMIC`; the final linker argv contained an absolute `.../target-linker-runtime/rcrt1.o`, which the wrapper did not normalize because it only matched bare `rcrt1.o`.

The fix makes the generated target linker wrapper match both bare and absolute CRT object paths. Non-dynamic `rcrt1.o` now maps to `crt1.o`, while dynamic temporary rustc links still map to `Scrt1.o`; compile-only and `-shared|-dynamiclib` static-support skips remain unchanged.

Focused validation after the change:

```text
$ cargo test -p mantle --bin mantle rust_source_provider::tests::materializer_writes_final_provider_output_from_validated_candidate -- --exact --nocapture
# pueue task 26
test rust_source_provider::tests::materializer_writes_final_provider_output_from_validated_candidate ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 808 filtered out; finished in 0.96s

$ cargo test -p mantle --bin mantle rust_source_provider::tests::materializer_writes_musl_host_provider_metadata_from_route_plan -- --exact --nocapture
# pueue task 26
test rust_source_provider::tests::materializer_writes_musl_host_provider_metadata_from_route_plan ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 808 filtered out; finished in 0.88s

$ /home/brittonr/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin/rustfmt --edition 2024 --check src/rust_source_provider.rs
$ git diff --check -- src/rust_source_provider.rs
# pueue task 31 completed successfully.
```

Directional scratch proof against rerun 23's final `rustc_main` command:

```text
$ <replay line 11168 from rerun23 mrustc-first-stage-build.log with patched scratch wrapper>
# pueue task 30
replay-status=0
.../rustc_main-d16b0f50cc7b51b0: ELF 64-bit LSB executable, x86-64, version 1 (SYSV), statically linked, not stripped
  Type:                              EXEC (Executable file)
  Entry point address:               0x6a7e35
rustc 1.90.0-stable-mrustc
direct-version-exit=0
rustc 1.90.0-stable-mrustc
binary: rustc
commit-hash: unknown
commit-date: unknown
host: x86_64-unknown-linux-musl
release: 1.90.0
LLVM version: 20.1.8
direct-vv-exit=0
```

Cairn validation transcript saved at `target/first-stage-musl-host-llvm-runtime-cairn-2026-06-21-crt-normalization-final.txt`:

```text
$ nix run path:/home/brittonr/git/cairn#cairn -- validate --root .
{
  "change_issues": [],
  "changes": 2,
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "spec_issues": [],
  "specs_validated": 7,
  "valid": true
}
$ nix run path:/home/brittonr/git/cairn#cairn -- gate proposal first-stage-musl-host-llvm-runtime --root .
{
  "change": "first-stage-musl-host-llvm-runtime",
  "issues": [],
  "layout": "cairn",
  "stage": "proposal",
  "valid": true,
  "verdict": "PASS"
}
$ nix run path:/home/brittonr/git/cairn#cairn -- gate design first-stage-musl-host-llvm-runtime --root .
{
  "change": "first-stage-musl-host-llvm-runtime",
  "issues": [],
  "layout": "cairn",
  "stage": "design",
  "valid": true,
  "verdict": "PASS"
}
$ nix run path:/home/brittonr/git/cairn#cairn -- gate tasks first-stage-musl-host-llvm-runtime --root .
{
  "change": "first-stage-musl-host-llvm-runtime",
  "issues": [],
  "layout": "cairn",
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}
```

## 2026-06-22 addendum: `crtbeginS.o` frame-init sanitization

Commit under edit after rerun 27 diagnosis.

Rerun 27 reached `run_rustc` stage-2 `compiler_builtins` and failed before the
build script's `main`: the generated `build_script_build-*` binary was a static
`ET_EXEC` with no dynamic section and exited with signal 139. GDB showed the
crash in `strlen -> get_cie_encoding -> classify_object_over_fdes ->
__register_frame_info -> frame_dummy`. Scratch inspection showed the copied
first-stage target `crtbeginS.o` contributes `frame_dummy` through
`.init_array`/`.fini_array` and an empty `.eh_frame`, causing bad frame
registration at process startup. The source fix makes the generated target
runtime setup require target `objcopy` and strip `.init_array`,
`.rela.init_array`, `.fini_array`, and `.rela.fini_array` from the copied
`crtbeginS.o` before the target linker wrapper maps CRT arguments to that
runtime copy.

Focused validation transcript saved at
`target/rust-source-provider-crtbegin-frame-init-validation-2026-06-22.log`:

```text
$ cargo test -p mantle --bin mantle rust_source_provider::tests::materializer_writes_musl_host_provider_metadata_from_route_plan -- --nocapture
test rust_source_provider::tests::materializer_writes_musl_host_provider_metadata_from_route_plan ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 810 filtered out; finished in 0.72s

$ cargo test -p mantle --bin mantle rust_source_provider::tests::materializer_writes_final_provider_output_from_validated_candidate -- --nocapture
test rust_source_provider::tests::materializer_writes_final_provider_output_from_validated_candidate ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 810 filtered out; finished in 0.64s

$ git diff --check
diff-check: ok

$ nix run path:/home/brittonr/git/cairn#cairn -- validate --root .
{
  "change_issues": [],
  "changes": 2,
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "spec_issues": [],
  "specs_validated": 7,
  "valid": true
}

$ nix run path:/home/brittonr/git/cairn#cairn -- gate proposal first-stage-musl-host-llvm-runtime --root .
{
  "change": "first-stage-musl-host-llvm-runtime",
  "issues": [],
  "layout": "cairn",
  "stage": "proposal",
  "valid": true,
  "verdict": "PASS"
}

$ nix run path:/home/brittonr/git/cairn#cairn -- gate design first-stage-musl-host-llvm-runtime --root .
{
  "change": "first-stage-musl-host-llvm-runtime",
  "issues": [],
  "layout": "cairn",
  "stage": "design",
  "valid": true,
  "verdict": "PASS"
}

$ nix run path:/home/brittonr/git/cairn#cairn -- gate tasks first-stage-musl-host-llvm-runtime --root .
{
  "change": "first-stage-musl-host-llvm-runtime",
  "issues": [],
  "layout": "cairn",
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}
```

Directional scratch proof from rerun 27 is included in the same transcript: the
scratch wrapper was restored to static behavior, only `crtbeginS.o` frame
init/fini hooks were stripped, and rerunning
`make -C run_rustc DYLIB_EXT=rlib output/prefix-2/lib/rustlib/x86_64-unknown-linux-musl/lib/libtest.rlib`
produced `libtest-79eac707c335c2b4.rlib` with no `error:`, `SIGSEGV`, or
`failed` markers in stderr. This scratch continuation is directional only; a
fresh committed full rerun is still required before claiming provider
completion.

## 2026-06-22 addendum: Rust bootstrap target linker wrapper

Commit under edit after rerun 28 diagnosis.

Rerun 28 proved the `crtbeginS.o` frame-init sanitization cleared rerun 27 and
advanced into the Rust 1.91.1 stage1 bootstrap. The new failure was not a
first-stage mrustc frontier: x.py/Cargo used the source-root musl GCC directly
for target build-script links, so bare `rcrt1.o`/`crti.o`/`crtbeginS.o` and
`-lunwind` could not be found. The source fix makes the generated Rust-bootstrap
adapter create a target linker alias/runtime directory, copy/sanitize CRT and
unwind objects, build the local `libatomic.a` shim, and write Rust bootstrap
`[target.<triple>]` `cc`/`cxx`/`ar`/`ranlib`/`linker` entries that point at those
aliases.

Focused validation after the change:

```text
$ cargo test -p mantle --bin mantle rust_source_provider::tests::materializer_writes_musl_host_provider_metadata_from_route_plan -- --nocapture
# pueue task 548
test rust_source_provider::tests::materializer_writes_musl_host_provider_metadata_from_route_plan ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 810 filtered out; finished in 0.98s

$ cargo test -p mantle --bin mantle rust_source_provider::tests::materializer_writes_final_provider_output_from_validated_candidate -- --nocapture
# pueue task 548
test rust_source_provider::tests::materializer_writes_final_provider_output_from_validated_candidate ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 810 filtered out; finished in 0.67s

$ /home/brittonr/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin/rustfmt --edition 2024 --check src/rust_source_provider.rs
$ git diff --check
$ nix run path:/home/brittonr/git/cairn#cairn -- validate --root .
$ nix run path:/home/brittonr/git/cairn#cairn -- gate proposal first-stage-musl-host-llvm-runtime --root .
$ nix run path:/home/brittonr/git/cairn#cairn -- gate design first-stage-musl-host-llvm-runtime --root .
$ nix run path:/home/brittonr/git/cairn#cairn -- gate tasks first-stage-musl-host-llvm-runtime --root .
# pueue task 549 completed successfully; tasks gate verdict: PASS
```

The generated-script assertions now require the Rust-bootstrap wrapper path,
private runtime directory, CRT object mapping, `-static-pie` to `-static`
normalization, `-B`/`-L` runtime flags, local `libatomic.a` shim source, and
`linker = "$MANTLE_TARGET_CC"` after `MANTLE_TARGET_CC` is rewritten to the
wrapper alias. This remains focused validation; rerun 29 is the authoritative
full-provider proof.

## 2026-06-22 addendum: Rust bootstrap shared libgcc fallback

Commit under edit after rerun 29 diagnosis.

Rerun 29 proved the Rust-bootstrap target linker wrapper cleared the missing
bare CRT and `-lunwind` frontier for early build-script links. It advanced to
shared proc-macro links (`clap_derive`, `serde_derive`) where Rust passed
`-shared` and `-lgcc_s`, but the source-root musl GCC closure has no
`libgcc_s.so*`. The source fix keeps `-shared|-dynamiclib` links out of the
executable-only `-static` support path, drops unavailable `-lgcc_s`, and appends
`-Wl,-Bstatic -lgcc -Wl,-Bdynamic` only for shared links.

Focused validation after the change:

```text
$ cargo test -p mantle --bin mantle rust_source_provider::tests::materializer_writes_musl_host_provider_metadata_from_route_plan -- --nocapture
# pueue task 581
test rust_source_provider::tests::materializer_writes_musl_host_provider_metadata_from_route_plan ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 810 filtered out; finished in 1.09s

$ cargo test -p mantle --bin mantle rust_source_provider::tests::materializer_writes_final_provider_output_from_validated_candidate -- --nocapture
# pueue task 581
test rust_source_provider::tests::materializer_writes_final_provider_output_from_validated_candidate ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 810 filtered out; finished in 0.72s
```

The generated-script assertions now require `-lgcc_s` to be dropped,
`shared_link` tracking, the shared-only `-Wl,-Bstatic -lgcc -Wl,-Bdynamic`
fallback, and the existing executable-only static support path. Rerun 30 is the
authoritative full-provider proof.

## 2026-06-22 addendum: Rust bootstrap Cargo static feature config

Commit under edit after rerun 30 Cargo/OpenSSL diagnosis.

Rerun 30 and its scratch continuations reached Rust-bootstrap Cargo tool
build/install work. Cargo's default feature set pulled the curl/OpenSSL path and
failed because the source-root musl provider does not expose an OpenSSL
installation to `openssl-sys`. Rust bootstrap already has a reviewable config
knob for this route: `[build] cargo-native-static = true`. The source fix writes
that knob into the generated x.py config, which makes bootstrap add Cargo's
`all-static` feature instead of relying on ambient OpenSSL. The same edit keeps
stage1 `tools = ["cargo"]` and reserves `tools = ["cargo", "rustdoc"]` for final
build goals containing `rustdoc`.

Local validation also repaired two stale test-fixture issues in the same source
area: escaped the shell grouping braces inside the generated Rust-bootstrap
linker wrapper's Rust `format!` string, and made synthetic x.py/stage scripts
remove the copied read-only musl `libc.so` before writing their fake loader.

Focused validation after the change:

```text
$ /home/brittonr/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin/cargo test -p mantle --bin mantle rust_source_provider::tests::materializer_generates_xpy_adapters_when_rust_sources_lack_stage_scripts --jobs 1 -- --nocapture
# pueue task 186
test rust_source_provider::tests::materializer_generates_xpy_adapters_when_rust_sources_lack_stage_scripts ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 813 filtered out; finished in 2.18s

$ /home/brittonr/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin/cargo test -p mantle --bin mantle rust_source_provider::tests::materializer_generates_xpy_adapters_when_rust_sources_lack_stage_scripts --jobs 1 -- --nocapture
$ /home/brittonr/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin/cargo test -p mantle --bin mantle rust_source_provider::tests::first_stage_proc_macro_rustc_wrapper_skips_duplicate_sysroot --jobs 1 -- --nocapture
# pueue task 199
test rust_source_provider::tests::materializer_generates_xpy_adapters_when_rust_sources_lack_stage_scripts ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 813 filtered out; finished in 2.10s

test rust_source_provider::tests::first_stage_proc_macro_rustc_wrapper_skips_duplicate_sysroot ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 813 filtered out; finished in 0.01s

$ /home/brittonr/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin/cargo test -p mantle --bin mantle rust_source_provider::tests::materializer_writes_musl_host_provider_metadata_from_route_plan --jobs 1 -- --nocapture
$ /home/brittonr/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin/cargo test -p mantle --bin mantle rust_source_provider::tests::materializer_writes_final_provider_output_from_validated_candidate --jobs 1 -- --nocapture
# pueue task 41
test rust_source_provider::tests::materializer_writes_musl_host_provider_metadata_from_route_plan ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 813 filtered out; finished in 0.92s

test rust_source_provider::tests::materializer_writes_final_provider_output_from_validated_candidate ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 813 filtered out; finished in 0.88s

$ /home/brittonr/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin/rustfmt --edition 2024 --check src/rust_source_provider.rs && git diff --check
# pueue task 45 completed successfully.
```

The generated-script assertions now require `cargo-native-static = true`, the
stage1 cargo-only/final rustdoc tool split, duplicate-sysroot preservation in
provider rustc wrappers, and the existing shared/static linker wrapper branches.
A fresh committed full rerun is still required before claiming provider
completion.

Cairn validation transcript saved at
`target/first-stage-musl-host-llvm-runtime-cairn-2026-06-22-cargo-static.txt`:

```text
$ nix run path:/home/brittonr/git/cairn#cairn -- validate --root .
{
  "change_issues": [],
  "changes": 2,
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "spec_issues": [],
  "specs_validated": 7,
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
  "input_hash": "803388ce9574ff5e8af7826bbda61396f07e5bdf7ba3e370287073aff85cbe88",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "e5d8b99d26b51ab1ba6cfb6966fecb444131f289c7d8741bf4f20238e8448eac",
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
  "input_hash": "4f8ed8a193c57329b90adfe810d2658e9765d1ba1511364bf5ae7202e06b4391",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "b9afac7e3499f279b4f3b3ec2a3b21c41cfe2055ed1736f9f529bc4ec6fb1ac0",
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
  "input_hash": "c62c3f4ae6aa017062968569d566491882073a4bc9430b9dada17f67c95dc41e",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "e0f5cf2c6387538e81b43146f76f4c92d40d34616e59693074512295f882f36a",
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}
```

## 2026-06-22 addendum: shared libgcc runtime packaging for dynamic rustc wrappers

Commit under edit: working tree after `076188dd keep Rust bootstrap Cargo off ambient OpenSSL`.

Rerun 31 failed after the mrustc first-stage build completed, while smoke-running
the packaged provider candidate. The candidate wrapper used the dynamic musl
loader but the candidate package omitted `libgcc_s.so.1`, so `rustc_binary`
reported a missing shared library and unresolved `_Unwind_*` / `__popcountdi2`
symbols. The source fix centralizes copying the dynamic libgcc runtime closure:
`libgcc_s.so.1` is required, `libgcc_s.so` is copied when present, first-stage
proc-macro rustc candidate packaging uses that helper, and Rust-bootstrap
dynamic-tool wrapping uses the same helper.

Focused validation after the change:

```text
$ cargo test -p mantle --bin mantle first_stage_proc_macro_rustc_install -- --nocapture
$ cargo test -p mantle --bin mantle rustc_stage_dynamic_tool_wrapping -- --nocapture
# pueue task 34
running 3 tests
...
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 814 filtered out; finished in 0.02s

running 2 tests
test rust_source_provider::tests::rustc_stage_dynamic_tool_wrapping_rejects_missing_shared_libgcc_runtime ... ok
test rust_source_provider::tests::rustc_stage_dynamic_tool_wrapping_copies_shared_libgcc_runtime ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 815 filtered out; finished in 0.02s

$ cargo test -p mantle --bin mantle materializer_generates_xpy_adapters_when_rust_sources_lack_stage_scripts -- --nocapture
$ cargo test -p mantle --bin mantle materializer_writes_musl_host_provider_metadata_from_route_plan -- --nocapture
# pueue task 32
test rust_source_provider::tests::materializer_generates_xpy_adapters_when_rust_sources_lack_stage_scripts ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 816 filtered out; finished in 2.27s

test rust_source_provider::tests::materializer_writes_musl_host_provider_metadata_from_route_plan ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 816 filtered out; finished in 1.04s

$ /home/brittonr/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin/rustfmt --edition 2024 --check src/rust_source_provider.rs
$ git diff --check
# pueue task 36 completed successfully
```

Post-evidence Cairn validation transcript saved at
`target/first-stage-musl-host-llvm-runtime-cairn-2026-06-22-libgcc-runtime.txt`:

```text
$ git diff --check

$ nix run path:/home/brittonr/git/cairn#cairn -- validate --root .
{
  "change_issues": [],
  "changes": 2,
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "spec_issues": [],
  "specs_validated": 7,
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
  "input_hash": "7b3d661b2fd8f58d8fa1f4c1b7855684b81274308c6318edff92973992f85183",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "c6098afdcc3c39ac747f1e087197d67194612667d9d64b2ca495dfd582688b10",
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
  "input_hash": "a1fe4ea205ea6ccf0a5e764073d4e46ddde72372b55a5a2b46efffb39cb35659",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "b42abc063cc1c0ca32e43f5c52e4f84132bd4bc816d05e4d7320ace22082f85a",
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
  "input_hash": "3c38ad795815ee886ea322be686f486b2be3a6bef7606779a01e1a212c39a241",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "e1d844237ff2878e94487753766820f5c10d6aa5c3765d7ea128a7fe762a00b4",
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}
```

## 2026-06-23 addendum: shared libgcc target-sysroot fallback

Rerun 32 proved the previous packaging fix failed closed before smoke because
the generated first-stage target runtime only searched the musl CRT dir for
`libgcc_s.so.1`, while the source-root provider exposes it under the target
sysroot libdir. The source fix adds that target-sysroot libdir fallback to both
first-stage and Rust-bootstrap generated linker runtime setup. The packaging
boundary still requires `libgcc_s.so.1`; the generated scripts only broaden the
source search.

Focused validation:

```text
$ cargo test -p mantle --bin mantle rust_source_provider::tests::materializer_ -- --nocapture
# pueue task 175
running 12 tests
test rust_source_provider::tests::materializer_rejects_missing_route_plan_without_output ... ok
test rust_source_provider::tests::materializer_rejects_existing_output_before_scratch_work ... ok
test rust_source_provider::tests::materializer_rejects_first_stage_source_digest_mismatch_without_output ... ok
test rust_source_provider::tests::materializer_rejects_rust_source_without_stage_script_or_xpy_without_output ... ok
test rust_source_provider::tests::materializer_rejects_rustc_stage1_source_digest_mismatch_without_output ... ok
test rust_source_provider::tests::materializer_rejects_chained_rustc_stage1_source_digest_mismatch_without_output ... ok
test rust_source_provider::tests::materializer_rejects_final_chained_rustc_stage1_source_digest_mismatch_without_output ... ok
test rust_source_provider::tests::materializer_rejects_rustc_final_source_digest_mismatch_without_output ... ok
test rust_source_provider::tests::materializer_uses_explicit_route_plan_when_default_plan_is_absent ... ok
test rust_source_provider::tests::materializer_writes_final_provider_output_from_validated_candidate ... ok
test rust_source_provider::tests::materializer_writes_musl_host_provider_metadata_from_route_plan ... ok
test rust_source_provider::tests::materializer_generates_xpy_adapters_when_rust_sources_lack_stage_scripts ... ok

test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 805 filtered out; finished in 3.42s

$ cargo test -p mantle --bin mantle rust_source_provider::tests::first_stage_proc_macro_rustc_install -- --nocapture
# pueue task 160
running 3 tests
...
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 814 filtered out

$ cargo test -p mantle --bin mantle rust_source_provider::tests::rustc_stage_dynamic_tool_wrapping -- --nocapture
# pueue task 160
running 2 tests
test rust_source_provider::tests::rustc_stage_dynamic_tool_wrapping_rejects_missing_shared_libgcc_runtime ... ok
test rust_source_provider::tests::rustc_stage_dynamic_tool_wrapping_copies_shared_libgcc_runtime ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 815 filtered out; finished in 0.02s

$ /home/brittonr/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin/rustfmt --edition 2024 --check src/rust_source_provider.rs && git diff --check
# pueue task 163 completed successfully
```

Post-addendum Cairn validation transcript saved at
`target/first-stage-musl-host-llvm-runtime-cairn-2026-06-23-libgcc-sysroot-fallback.txt`:

```text
$ git diff --check

$ nix run path:/home/brittonr/git/cairn#cairn -- validate --root .
{
  "change_issues": [],
  "changes": 2,
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "spec_issues": [],
  "specs_validated": 7,
  "valid": true
}

$ nix run path:/home/brittonr/git/cairn#cairn -- gate proposal first-stage-musl-host-llvm-runtime --root .
{
  "change": "first-stage-musl-host-llvm-runtime",
  "issues": [],
  "layout": "cairn",
  "stage": "proposal",
  "valid": true,
  "verdict": "PASS"
}

$ nix run path:/home/brittonr/git/cairn#cairn -- gate design first-stage-musl-host-llvm-runtime --root .
{
  "change": "first-stage-musl-host-llvm-runtime",
  "issues": [],
  "layout": "cairn",
  "stage": "design",
  "valid": true,
  "verdict": "PASS"
}

$ nix run path:/home/brittonr/git/cairn#cairn -- gate tasks first-stage-musl-host-llvm-runtime --root .
{
  "change": "first-stage-musl-host-llvm-runtime",
  "issues": [],
  "layout": "cairn",
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}
```

## 2026-06-23 addendum: explicit source-root target toolchain binding

Rerun 33 proved the target-sysroot `libgcc_s.so.1` search was not enough when
generated scripts discovered a Nix musl GCC wrapper before the source-root
compiler. The source fix makes both first-stage and Rust-bootstrap generated
scripts prefer `MANTLE_TARGET_TOOLCHAIN_ROOT`, with `SOURCE_ROOT` as fallback,
before PATH/Nix discovery. Explicit roots now validate the target compiler,
C++ compiler, `ar`, `ranlib`, `objcopy`, `libc.so`, and `libgcc_s.so.1`; invalid
explicit roots fail closed.

Source-root fixture check:

```text
$ SOURCE_ROOT=.pi/source-root-provider-run-20260531T231455Z/store/0jp0idh7pj86vlraa7vcq8igfqp4wrvh-musl-seed-toolchain
$ "$SOURCE_ROOT/bin/x86_64-linux-musl-gcc" -dumpmachine
x86_64-linux-musl
$ "$SOURCE_ROOT/bin/x86_64-linux-musl-gcc" --version | head -1
x86_64-linux-musl-gcc (GCC) 10.5.0
$ test -x "$SOURCE_ROOT/bin/x86_64-linux-musl-gcc"
$ test -x "$SOURCE_ROOT/bin/x86_64-linux-musl-g++"
$ test -x "$SOURCE_ROOT/bin/x86_64-linux-musl-ar"
$ test -x "$SOURCE_ROOT/bin/x86_64-linux-musl-ranlib"
$ test -x "$SOURCE_ROOT/bin/x86_64-linux-musl-objcopy"
$ test -f "$SOURCE_ROOT/x86_64-linux-musl/lib/libc.so"
$ test -f "$SOURCE_ROOT/x86_64-linux-musl/lib/libgcc_s.so.1"
# pueue task 204 completed successfully
```

Focused validation:

```text
$ /home/brittonr/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin/rustfmt --edition 2024 src/rust_source_provider.rs
$ /home/brittonr/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin/rustfmt --edition 2024 --check src/rust_source_provider.rs
# pueue task 186 completed successfully

$ cargo test -p mantle --bin mantle rust_source_provider::tests::materializer_ -- --nocapture
# pueue task 188
running 14 tests
test rust_source_provider::tests::materializer_rejects_existing_output_before_scratch_work ... ok
test rust_source_provider::tests::materializer_rejects_first_stage_source_digest_mismatch_without_output ... ok
test rust_source_provider::tests::materializer_rejects_invalid_explicit_source_root_env_in_subprocess ... ok
test rust_source_provider::tests::materializer_rejects_rustc_stage1_source_digest_mismatch_without_output ... ok
test rust_source_provider::tests::materializer_rejects_rust_source_without_stage_script_or_xpy_without_output ... ok
test rust_source_provider::tests::materializer_rejects_chained_rustc_stage1_source_digest_mismatch_without_output ... ok
test rust_source_provider::tests::materializer_rejects_final_chained_rustc_stage1_source_digest_mismatch_without_output ... ok
test rust_source_provider::tests::materializer_rejects_rustc_final_source_digest_mismatch_without_output ... ok
test rust_source_provider::tests::materializer_uses_explicit_route_plan_when_default_plan_is_absent ... ok
test rust_source_provider::tests::materializer_writes_final_provider_output_from_validated_candidate ... ok
test rust_source_provider::tests::materializer_writes_musl_host_provider_metadata_from_route_plan ... ok
test rust_source_provider::tests::materializer_generates_xpy_adapters_when_rust_sources_lack_stage_scripts ... ok

test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 805 filtered out; finished in 2.97s

$ cargo test -p mantle --bin mantle first_stage_proc_macro_rustc_install -- --nocapture
$ cargo test -p mantle --bin mantle rustc_stage_dynamic_tool_wrapping -- --nocapture
# pueue task 192
running 3 tests
...
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 816 filtered out; finished in 0.03s

running 2 tests
test rust_source_provider::tests::rustc_stage_dynamic_tool_wrapping_rejects_missing_shared_libgcc_runtime ... ok
test rust_source_provider::tests::rustc_stage_dynamic_tool_wrapping_copies_shared_libgcc_runtime ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 817 filtered out; finished in 0.03s

$ git diff --check
# pueue task 197 completed successfully
```

Post-addendum Cairn validation transcript saved at
`target/first-stage-musl-host-llvm-runtime-cairn-2026-06-23-explicit-source-root-binding.txt`:

```text
$ git diff --check

$ nix run path:/home/brittonr/git/cairn#cairn -- validate --root .
{
  "change_issues": [],
  "changes": 2,
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "spec_issues": [],
  "specs_validated": 7,
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
  "input_hash": "05374af44ec76964991c8b752edfc4cf0683f080180decd4aa3e80033b992b41",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "da96dfb161571f6720d535cf1dd75c4a50e1740b2d0a1faeacfcc2f2472c8c61",
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
  "input_hash": "38b39e7019722c32c1d29821edc0537b64a1a0dc7587b4880459284ebafa8a97",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "bb96ac0b25c44d60d48fe93695055679d2296e2093a0da0c7eae0ac079633b61",
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
  "input_hash": "52799f6b932535a7a87756e8311e1fd4a648a1a35dbe70691b574ae744c9ab0e",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "0ea6b75e51399ffd13ee5718df95e5a4a12febad4f8d3c1246eee247363bb60a",
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}
```

## 2026-06-23 addendum: Rust-bootstrap shared proc-macro unwinder link

Rerun 34 reached Rust-bootstrap x.py and failed while stage1 `rustc` loaded a
stage2 proc-macro dylib. A preserved-scratch probe proved the dylib was built by
the compatible stage1 compiler but lacked a load-time definition for
`_Unwind_Resume`. The source fix updates the Rust-bootstrap target linker
wrapper's shared-link branch from static `libgcc` only to static unwinder plus
static `libgcc`: `-Wl,-Bstatic -lunwind -lgcc -Wl,-Bdynamic`.

Focused validation after the change:

```text
$ /home/brittonr/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin/rustfmt --edition 2024 --check src/rust_source_provider.rs
$ cargo test -p mantle --bin mantle rustc_stage_dynamic_tool_wrapping -- --nocapture
# pueue task 563
running 2 tests
test rust_source_provider::tests::rustc_stage_dynamic_tool_wrapping_rejects_missing_shared_libgcc_runtime ... ok
test rust_source_provider::tests::rustc_stage_dynamic_tool_wrapping_copies_shared_libgcc_runtime ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 817 filtered out; finished in 0.04s

$ cargo test -p mantle --bin mantle rust_source_provider::tests::materializer_ -- --nocapture
$ git diff --check
# pueue task 574
running 14 tests
test rust_source_provider::tests::materializer_rejects_existing_output_before_scratch_work ... ok
test rust_source_provider::tests::materializer_rejects_first_stage_source_digest_mismatch_without_output ... ok
test rust_source_provider::tests::materializer_rejects_invalid_explicit_source_root_env_in_subprocess ... ok
test rust_source_provider::tests::materializer_rejects_rustc_stage1_source_digest_mismatch_without_output ... ok
test rust_source_provider::tests::materializer_rejects_rust_source_without_stage_script_or_xpy_without_output ... ok
test rust_source_provider::tests::materializer_rejects_chained_rustc_stage1_source_digest_mismatch_without_output ... ok
test rust_source_provider::tests::materializer_rejects_final_chained_rustc_stage1_source_digest_mismatch_without_output ... ok
test rust_source_provider::tests::materializer_rejects_rustc_final_source_digest_mismatch_without_output ... ok
test rust_source_provider::tests::materializer_uses_explicit_route_plan_when_default_plan_is_absent ... ok
test rust_source_provider::tests::materializer_writes_final_provider_output_from_validated_candidate ... ok
test rust_source_provider::tests::materializer_writes_musl_host_provider_metadata_from_route_plan ... ok
test rust_source_provider::tests::materializer_generates_xpy_adapters_when_rust_sources_lack_stage_scripts ... ok

test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 805 filtered out; finished in 3.09s

$ cargo test -p mantle --bin mantle first_stage_proc_macro_rustc_install -- --nocapture
# pueue task 575
running 3 tests
test rust_source_provider::tests::first_stage_proc_macro_rustc_install_rejects_missing_runtime_loader ... ok
test rust_source_provider::tests::first_stage_proc_macro_rustc_install_rejects_missing_shared_libgcc_runtime ... ok
test rust_source_provider::tests::first_stage_proc_macro_rustc_install_writes_relocatable_loader_wrapper ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 816 filtered out; finished in 0.02s
```

Post-addendum Cairn validation transcript saved at
`target/first-stage-musl-host-llvm-runtime-cairn-2026-06-23-procmacro-unwind.txt`:

```text
$ /home/brittonr/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin/rustfmt --edition 2024 --check src/rust_source_provider.rs

$ git diff --check

$ nix run path:/home/brittonr/git/cairn#cairn -- validate --root .
{
  "change_issues": [],
  "changes": 2,
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "spec_issues": [],
  "specs_validated": 7,
  "valid": true
}

$ nix run path:/home/brittonr/git/cairn#cairn -- gate proposal first-stage-musl-host-llvm-runtime --root .
{
  "change": "first-stage-musl-host-llvm-runtime",
  "issues": [],
  "layout": "cairn",
  "stage": "proposal",
  "valid": true,
  "verdict": "PASS"
}

$ nix run path:/home/brittonr/git/cairn#cairn -- gate design first-stage-musl-host-llvm-runtime --root .
{
  "change": "first-stage-musl-host-llvm-runtime",
  "issues": [],
  "layout": "cairn",
  "stage": "design",
  "valid": true,
  "verdict": "PASS"
}

$ nix run path:/home/brittonr/git/cairn#cairn -- gate tasks first-stage-musl-host-llvm-runtime --root .
{
  "change": "first-stage-musl-host-llvm-runtime",
  "issues": [],
  "layout": "cairn",
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}
```

## 2026-06-23 addendum: final x.py rustdoc install-goal split

Rerun 35 proved the Rust-bootstrap shared proc-macro unwinder fix by advancing
past the rerun-34 `_Unwind_Resume` failure and into the Rust 1.94 final x.py
stage. The final stage then failed before compiling final Rust sources because
Rust 1.94 has no `install rustdoc` path. The source fix keeps final rustdoc in
the provider by separating x.py CLI goals from bootstrap tool selection: final
x.py now runs `install rustc cargo library/std`, while the generated config still
sets bootstrap `tools = ["cargo", "rustdoc"]` so the Rustc dist component copies
`bin/rustdoc`. Stage1 remains cargo-only.

Baseline before the change:

```text
$ cargo test -p mantle --bin mantle rust_source_provider::tests::materializer_
# pueue task 79
running 14 tests
...
test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 805 filtered out; finished in 3.87s
```

Focused validation after the change:

```text
$ cargo test -p mantle --bin mantle rust_source_provider::tests::materializer_
# pueue task 97
running 14 tests
...
test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 805 filtered out; finished in 4.19s

$ cargo test -p mantle --bin mantle rustc_stage_dynamic_tool_wrapping
# pueue task 114
running 2 tests
test rust_source_provider::tests::rustc_stage_dynamic_tool_wrapping_rejects_missing_shared_libgcc_runtime ... ok
test rust_source_provider::tests::rustc_stage_dynamic_tool_wrapping_copies_shared_libgcc_runtime ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 817 filtered out; finished in 0.05s

$ cargo test -p mantle --bin mantle first_stage_proc_macro_rustc_install
# pueue task 110
running 3 tests
test rust_source_provider::tests::first_stage_proc_macro_rustc_install_rejects_missing_runtime_loader ... ok
test rust_source_provider::tests::first_stage_proc_macro_rustc_install_rejects_missing_shared_libgcc_runtime ... ok
test rust_source_provider::tests::first_stage_proc_macro_rustc_install_writes_relocatable_loader_wrapper ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 816 filtered out; finished in 0.03s

$ /home/brittonr/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin/rustfmt --edition 2024 --check src/rust_source_provider.rs
$ git diff --check
# pueue task 116
rustfmt-check-and-diff-check: ok
```

Post-addendum Cairn validation transcript saved at
`target/first-stage-musl-host-llvm-runtime-cairn-2026-06-23-final-rustdoc-goal.txt`:

```text
$ git diff --check

$ /home/brittonr/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin/rustfmt --edition 2024 --check src/rust_source_provider.rs

$ nix run path:/home/brittonr/git/cairn#cairn -- validate --root .
{
  "change_issues": [],
  "changes": 2,
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "spec_issues": [],
  "specs_validated": 7,
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
  "input_hash": "e3cafffedf5eadce1a8791a220387a2bac0b929e0ed0da6ebee0936655f73bc3",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "4d5eee759965371aafa29dec70fd5deaf811bf8e86135e43ddf3e824e0812db3",
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
  "input_hash": "c137a9dcda225930dbd041a5acb1c1ecc1129173fbee7fb24fa11b25a5ae4f8d",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "888c009af4cb68509f43c1a5de229594e56f0efc2956465d3533ff8ce52c7b4b",
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
  "input_hash": "7d757b8f2731f9cc8614a1b858784d128ff40097742cf03ccfaf62328f06901d",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "b478ad0142d0e9879fd38ac1b17d36c86d1e038ddf0163a2da7f23c60e5f25ae",
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}
```

## 2026-06-24 addendum: rustdoc rustc-private tool rlib lookup

Rerun 36 cleared the invalid final `install rustdoc` x.py goal and reached
`rustdoc_tool_binary`, then failed because Rust bootstrap's rustc-private tool
build saw only sysroot `.rmeta` metadata for compiler/private crates while the
required `.rlib` artifacts already existed under the stage2 `Mode::Rustc`
`release/deps` output. The source fix patches generated musl-host x.py adapters
to add that `deps` directory to `RUSTC_ADDITIONAL_SYSROOT_PATHS` for
`Mode::ToolRustcPrivate` builds.

Baseline before the change was attempted with the normal focused test command,
but the host clang wrapper rejected the repo's configured `-fuse-ld=mold` before
any source assertion ran. No source baseline claim is made from that blocked
run.

Focused validation after the change:

```text
$ /home/brittonr/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin/rustfmt --edition 2024 src/rust_source_provider.rs
# pueue task 17 completed successfully.

$ cargo test -p mantle --bin mantle rust_source_provider::tests::materializer_writes_musl_host_provider_metadata_from_route_plan -- --exact
# pueue task 34, with a local clang wrapper that strips the stale -fuse-ld=mold host flag
running 1 test
test rust_source_provider::tests::materializer_writes_musl_host_provider_metadata_from_route_plan ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 818 filtered out; finished in 1.44s

$ cargo test -p mantle --bin mantle rust_source_provider::tests::materializer_generates_xpy_adapters_when_rust_sources_lack_stage_scripts -- --exact
# pueue task 43, same local clang wrapper
running 1 test
test rust_source_provider::tests::materializer_generates_xpy_adapters_when_rust_sources_lack_stage_scripts ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 818 filtered out; finished in 2.64s

$ cargo test -p mantle --bin mantle rust_source_provider::tests::materializer_rejects_rust_source_without_stage_script_or_xpy_without_output -- --exact
# pueue task 161, same local clang wrapper
running 1 test
test rust_source_provider::tests::materializer_rejects_rust_source_without_stage_script_or_xpy_without_output ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 818 filtered out; finished in 0.79s

$ /home/brittonr/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin/rustfmt --edition 2024 --check src/rust_source_provider.rs
$ git diff --check
# pueue task 169 completed successfully.
```

Directional scratch continuation from rerun 36:

```text
$ sh ./run-rustc-final.sh > rustc-final-tool-sysroot-continuation.log 2>&1
# pueue task 53 in target/.../rustc-final after manually applying the generated-source patch
Building stage2 rustdoc_tool_binary (stage1 -> stage2, x86_64-unknown-linux-musl)
...
Build completed successfully in 0:14:09
rustc final products ready

$ target/.../rustc-final-output/bin/rustdoc --version
# pueue task 141
rustdoc 1.94.0 (4a4ef493e 2026-03-02) (built from a source tarball)
```

This scratch proof is not authoritative provider completion evidence. A fresh
full rerun from the committed source is still required before claiming the final
provider or final rustdoc route complete.

Post-addendum Cairn validation transcript saved at
`target/first-stage-musl-host-llvm-runtime-cairn-2026-06-24-rustdoc-rlib.txt`:

```text
$ git diff --check

$ /home/brittonr/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin/rustfmt --edition 2024 --check src/rust_source_provider.rs

$ nix run path:/home/brittonr/git/cairn#cairn -- validate --root .
{
  "change_issues": [],
  "changes": 2,
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "spec_issues": [],
  "specs_validated": 7,
  "valid": true
}

$ nix run path:/home/brittonr/git/cairn#cairn -- gate proposal first-stage-musl-host-llvm-runtime --root .
{
  "change": "first-stage-musl-host-llvm-runtime",
  "issues": [],
  "layout": "cairn",
  "stage": "proposal",
  "valid": true,
  "verdict": "PASS"
}

$ nix run path:/home/brittonr/git/cairn#cairn -- gate design first-stage-musl-host-llvm-runtime --root .
{
  "change": "first-stage-musl-host-llvm-runtime",
  "issues": [],
  "layout": "cairn",
  "stage": "design",
  "valid": true,
  "verdict": "PASS"
}

$ nix run path:/home/brittonr/git/cairn#cairn -- gate tasks first-stage-musl-host-llvm-runtime --root .
{
  "change": "first-stage-musl-host-llvm-runtime",
  "issues": [],
  "layout": "cairn",
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}
```
