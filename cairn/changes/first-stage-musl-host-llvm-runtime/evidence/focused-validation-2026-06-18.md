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

Fresh real-provider rerun `1180` is running from commit `142955c51f294919557362ab82fe95523269f6d9` at `target/rust-source-provider-musl-host-route-prefix2-procmacro-rerun16-2026-06-20`; its result belongs in `provider-rerun-2026-06-18.md` after completion.

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
