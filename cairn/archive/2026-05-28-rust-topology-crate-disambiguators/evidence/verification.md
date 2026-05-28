# Verification

## Focused tests

Task-ID: V1
Covers: rust_package_planning.native_crate_disambiguators

Commands:

```sh
export PATH=/home/brittonr/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin:/nix/store/97vplpbajnr7x03fqh9biz5v6960sv22-clang-wrapper-21.1.8/bin:/nix/store/1sw8whfl5gfblp6r9qdkiw1b4j9fgwar-mold-2.40.4/bin:/nix/store/rvp7qlpf5jqvdckjy1afjb6aha6j8dxg-pkg-config-wrapper-0.29.2/bin:$PATH
export PKG_CONFIG_PATH=/nix/store/1l5jgzy26hkjz1y3apn1051asvn42sfn-openssl-3.6.1-dev/lib/pkgconfig
export SNIX_BUILD_SANDBOX_SHELL=/nix/store/8mf4s8c4xjvlkj12p299qylrb30g7zzh-busybox-static-x86_64-unknown-linux-musl-1.37.0/bin/busybox
export CARGO_TARGET_DIR=/tmp/mantle-crate-disambiguators-tests
cargo test -p mantle --bin mantle rustc_metadata_disambiguator -- --nocapture
cargo test -p mantle --bin mantle native_unit_derivation_emits_stable_metadata_disambiguator -- --nocapture
cargo test -p mantle --bin mantle native_host_unit_derivation_emits_metadata_disambiguator -- --nocapture
```

Result: all three focused filters passed, one test each.

## Dirty topology probe

Task-ID: V2
Covers: rust_package_planning.native_crate_disambiguators

Command: pueue task `140` (`crate-disambiguators-dirty-self-probe`).

Summary from `target/mantle-self-rust-plan-probe-crate-disambiguators-dirty/blocker-summary.txt`:

```text
probe_status=0
topology_execution=blocked
topology_unit_executions=471
metadata_runs=63
object_store_status=success
object_store_reason=rebuilt-explicit-unit
blocker=rustc-failed: error: environment variable `SNIX_BUILD_SANDBOX_SHELL` not defined at compile time
```

This advances past the prior `object_store@0.13.2` / `rand` metadata-loading blocker. The next frontier is compile-time environment propagation for `SNIX_BUILD_SANDBOX_SHELL`.

## Clean topology probe

Task-ID: V2b
Covers: rust_package_planning.native_crate_disambiguators

Command: pueue task `141` (`crate-disambiguators-clean-self-probe`).

Summary from `target/mantle-self-rust-plan-probe-crate-disambiguators-clean/blocker-summary.txt`:

```text
head: ff041f0c54b88b8b0145e11deab9f523c592f6f1
git_status_short_bytes=0
probe_status=0
topology_execution=blocked
topology_unit_executions=471
metadata_runs=63
object_store_status=success
blocker=rustc-failed: error: environment variable `SNIX_BUILD_SANDBOX_SHELL` not defined at compile time
```

This proves the committed tree advances past the `object_store` / `rand` blocker with a clean worktree.

## Cairn validation

Task-ID: V3
Covers: rust_package_planning.native_crate_disambiguators

Command:

```sh
/home/brittonr/.cargo-target/debug/cairn validate --root .
```

Result: `valid: true`, `changes: 1`, `specs_validated: 2`.

## Cairn gates

Task-ID: V4
Covers: rust_package_planning.native_crate_disambiguators

Commands:

```sh
/home/brittonr/.cargo-target/debug/cairn gate proposal rust-topology-crate-disambiguators --root .
/home/brittonr/.cargo-target/debug/cairn gate design rust-topology-crate-disambiguators --root .
/home/brittonr/.cargo-target/debug/cairn gate tasks rust-topology-crate-disambiguators --root .
```

Result: all three gates returned `verdict: PASS`.

## Post-review ambient-independence test

Task-ID: V5
Covers: rust_package_planning.native_crate_disambiguators

Command:

```sh
export PATH=/home/brittonr/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin:/nix/store/97vplpbajnr7x03fqh9biz5v6960sv22-clang-wrapper-21.1.8/bin:/nix/store/1sw8whfl5gfblp6r9qdkiw1b4j9fgwar-mold-2.40.4/bin:/nix/store/rvp7qlpf5jqvdckjy1afjb6aha6j8dxg-pkg-config-wrapper-0.29.2/bin:$PATH
export PKG_CONFIG_PATH=/nix/store/1l5jgzy26hkjz1y3apn1051asvn42sfn-openssl-3.6.1-dev/lib/pkgconfig
export SNIX_BUILD_SANDBOX_SHELL=/nix/store/8mf4s8c4xjvlkj12p299qylrb30g7zzh-busybox-static-x86_64-unknown-linux-musl-1.37.0/bin/busybox
cargo fmt -p mantle -- src/rust_plan.rs
export CARGO_TARGET_DIR=/tmp/mantle-disambiguator-hardening-tests
cargo test -p mantle --bin mantle native_unit_metadata_disambiguator_ignores_ambient_tool_roots -- --nocapture
cargo test -p mantle --bin mantle rustc_metadata_disambiguator -- --nocapture
```

Result: `native_unit_metadata_disambiguator_ignores_ambient_tool_roots` passed and proves the metadata value is unchanged when `RustPlanOptions.root`, `cargo`, and `rustc` point at different temp roots. `rustc_metadata_disambiguator_distinguishes_same_crate_package_versions` still passed.
