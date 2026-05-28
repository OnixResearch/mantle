# Verification

## Focused tests

Task-ID: V1
Covers: rust_package_planning.native_compile_env_allowlist

Commands:

```sh
export PATH=/home/brittonr/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin:/nix/store/97vplpbajnr7x03fqh9biz5v6960sv22-clang-wrapper-21.1.8/bin:/nix/store/1sw8whfl5gfblp6r9qdkiw1b4j9fgwar-mold-2.40.4/bin:/nix/store/rvp7qlpf5jqvdckjy1afjb6aha6j8dxg-pkg-config-wrapper-0.29.2/bin:$PATH
export PKG_CONFIG_PATH=/nix/store/1l5jgzy26hkjz1y3apn1051asvn42sfn-openssl-3.6.1-dev/lib/pkgconfig
export SNIX_BUILD_SANDBOX_SHELL=/nix/store/8mf4s8c4xjvlkj12p299qylrb30g7zzh-busybox-static-x86_64-unknown-linux-musl-1.37.0/bin/busybox
export CARGO_TARGET_DIR=/tmp/mantle-compile-env-tests
cargo test -p mantle --bin mantle rust_topology_child_env -- --nocapture
cargo test -p mantle --bin mantle allowed_rust_topology_compile_env -- --nocapture
```

Result: `rust_topology_child_env` filter passed 5 tests; `allowed_rust_topology_compile_env` filter passed 1 test.

## Dirty topology probe

Task-ID: V2
Covers: rust_package_planning.native_compile_env_allowlist

Command: pueue task `142` (`compile-env-dirty-self-probe`).

Summary from `target/mantle-self-rust-plan-probe-compile-env-dirty/blocker-summary.txt`:

```text
probe_status=3
receipt missing topology_execution
error: unit 0:path+file:///home/brittonr/git/mantle#0.1.0:mantle:lib:build reached execution before dependency package path+file:///home/brittonr/git/mantle#0.1.0 was produced
```

The prior `SNIX_BUILD_SANDBOX_SHELL` compile-time env blocker is gone. The next frontier is same-package self dependency handling for the root `mantle` lib unit.

## Clean topology probe

Task-ID: V2b
Covers: rust_package_planning.native_compile_env_allowlist

Command: pueue task `144` (`compile-env-clean-self-probe`).

Summary from `target/mantle-self-rust-plan-probe-compile-env-clean/blocker-summary.txt`:

```text
head: 3eab6c439121f3142f74b2fb921a13e18c3ad29e
git_status_short_bytes=0
probe_status=3
receipt missing topology_execution
error: unit 0:path+file:///home/brittonr/git/mantle#0.1.0:mantle:lib:build reached execution before dependency package path+file:///home/brittonr/git/mantle#0.1.0 was produced
```

This proves the committed tree advances past the `SNIX_BUILD_SANDBOX_SHELL` blocker with a clean worktree.

## Cairn validation

Task-ID: V3
Covers: rust_package_planning.native_compile_env_allowlist

Command:

```sh
/home/brittonr/.cargo-target/debug/cairn validate --root .
```

Result: `valid: true`, `changes: 1`, `specs_validated: 2`.

## Cairn gates

Task-ID: V4
Covers: rust_package_planning.native_compile_env_allowlist

Commands:

```sh
/home/brittonr/.cargo-target/debug/cairn gate proposal rust-topology-compile-env-allowlist --root .
/home/brittonr/.cargo-target/debug/cairn gate design rust-topology-compile-env-allowlist --root .
/home/brittonr/.cargo-target/debug/cairn gate tasks rust-topology-compile-env-allowlist --root .
```

Result: all three gates returned `verdict: PASS`.
