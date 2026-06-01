# Toolchain closure CLI validation slice

Task-ID: I2-partial
Covers: rust_package_planning.source_built_toolchain_closure
Date: 2026-05-31
Owner: agent

## Scope

This is partial progress on the unchecked task to accept a receipt-bound toolchain closure and reject host-tool leakage. This slice only accepts `--toolchain-closure <manifest>`, validates the manifest with the pure core, records the validated-but-not-enforced closure in self-build/fixed-point summaries and fixed-point preflight, and keeps `claim=false` / `not-source-built-toolchain-closure` visible.

The broader host `rustc`, Cargo, linker, C compiler, pkg-config, Nix profile tool, PATH helper, and sysroot leakage enforcement remains unchecked.

## Baseline before this slice

Command:

```text
cargo test -p mantle --bin mantle source_toolchain_closure -- --nocapture && \
cargo test -p mantle --bin mantle cargo_free -- --nocapture
```

Output excerpt:

```text
running 10 tests
...
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 590 filtered out; finished in 0.00s

running 14 tests
...
test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 586 filtered out; finished in 0.00s
```

## Post-change validation

Command:

```text
cargo fmt --check -p mantle -v && \
cargo test -p mantle --bin mantle source_toolchain_closure -- --nocapture && \
cargo test -p mantle --bin mantle cargo_free -- --nocapture && \
cargo test -p mantle --test cargo_free_self_build_cli -- --nocapture
```

Environment included:

```text
PATH=/home/brittonr/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin:/home/brittonr/.cargo/bin:/nix/store/97vplpbajnr7x03fqh9biz5v6960sv22-clang-wrapper-21.1.8/bin:/nix/store/27ddg2m33wnp6xrr7il86yjyr41n88ya-mold-2.40.4/bin:/nix/store/rvp7qlpf5jqvdckjy1afjb6aha6j8dxg-pkg-config-wrapper-0.29.2/bin:$PATH
PKG_CONFIG_PATH=/nix/store/1l5jgzy26hkjz1y3apn1051asvn42sfn-openssl-3.6.1-dev/lib/pkgconfig
SNIX_BUILD_SANDBOX_SHELL=/nix/store/8mf4s8c4xjvlkj12p299qylrb30g7zzh-busybox-static-x86_64-unknown-linux-musl-1.37.0/bin/busybox
```

Source-built closure core tests:

```text
running 11 tests
...
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 592 filtered out; finished in 0.00s
```

Cargo-free CLI/unit parsing tests:

```text
running 16 tests
...
test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 587 filtered out; finished in 0.00s
```

Cargo-free self-build CLI tests:

```text
running 11 tests
test cargo_free_fixed_point_rejects_invalid_toolchain_closure_manifest ... ok
test cargo_free_fixed_point_validates_toolchain_closure_manifest_without_claiming_enforcement ... ok
test cargo_free_self_build_rejects_invalid_toolchain_closure_manifest ... ok
test cargo_free_self_build_validates_toolchain_closure_manifest_without_claiming_enforcement ... ok
...
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.17s
```

## Cairn check after evidence update

Command:

```text
/home/brittonr/.cargo-target/debug/cairn validate --root . && \
/home/brittonr/.cargo-target/debug/cairn gate tasks source-built-toolchain-closure --root .
```

Output excerpt:

```json
{
  "change_issues": [],
  "changes": 1,
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "spec_issues": [],
  "specs_validated": 4,
  "valid": true
}
{
  "change": "source-built-toolchain-closure",
  "issues": [],
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}
```

## Decision

This slice proves only manifest acceptance/validation and explicit non-claim preservation. Do not mark I2 complete until host-tool leakage enforcement is implemented and covered by negative tests.
