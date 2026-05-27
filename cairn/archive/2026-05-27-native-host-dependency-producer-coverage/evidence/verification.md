# Verification: Native Host Dependency Producer Coverage

Task-ID: native-host-dependency-producer-coverage.V1
Covers: r[rust_package_planning.native_host_dependency_producer_coverage]
Date: 2026-05-27
Decision owner: coding agent

## Focused implementation tests

Command:

```sh
PATH=/run/current-system/sw/bin:/home/brittonr/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin:/nix/store/97vplpbajnr7x03fqh9biz5v6960sv22-clang-wrapper-21.1.8/bin:/nix/store/rvp7qlpf5jqvdckjy1afjb6aha6j8dxg-pkg-config-wrapper-0.29.2/bin:/etc/profiles/per-user/brittonr/bin:/usr/bin:/bin \
PKG_CONFIG_PATH=/nix/store/1l5jgzy26hkjz1y3apn1051asvn42sfn-openssl-3.6.1-dev/lib/pkgconfig \
SNIX_BUILD_SANDBOX_SHELL=/nix/store/8mf4s8c4xjvlkj12p299qylrb30g7zzh-busybox-static-x86_64-unknown-linux-musl-1.37.0/bin/busybox \
RUSTC_WRAPPER= \
CARGO_TARGET_DIR=target/native-host-dependency-producer-check \
cargo test -p mantle --bin mantle host_dependency_topology -- --nocapture
```

Result:

```text
running 3 tests
test rust_plan::tests::host_dependency_topology_blocks_missing_producer ... ok
test rust_plan::tests::host_dependency_topology_accepts_target_lib_producer ... ok
test rust_plan::tests::host_dependency_topology_accepts_proc_macro_host_producer ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 465 filtered out; finished in 0.02s
```

Command:

```sh
PATH=/run/current-system/sw/bin:/home/brittonr/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin:/nix/store/97vplpbajnr7x03fqh9biz5v6960sv22-clang-wrapper-21.1.8/bin:/nix/store/rvp7qlpf5jqvdckjy1afjb6aha6j8dxg-pkg-config-wrapper-0.29.2/bin:/etc/profiles/per-user/brittonr/bin:/usr/bin:/bin \
PKG_CONFIG_PATH=/nix/store/1l5jgzy26hkjz1y3apn1051asvn42sfn-openssl-3.6.1-dev/lib/pkgconfig \
SNIX_BUILD_SANDBOX_SHELL=/nix/store/8mf4s8c4xjvlkj12p299qylrb30g7zzh-busybox-static-x86_64-unknown-linux-musl-1.37.0/bin/busybox \
RUSTC_WRAPPER= \
CARGO_TARGET_DIR=target/native-host-dependency-producer-check \
cargo test -p mantle --bin mantle combined_unit_topology -- --nocapture
```

Result:

```text
running 2 tests
test rust_plan::tests::combined_unit_topology_blocks_missing_target_host_artifact_producer ... ok
test rust_plan::tests::combined_unit_topology_orders_target_host_and_proc_macro_edges ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 466 filtered out; finished in 0.01s
```

## Self-probe blocker movement

Baseline evidence is `evidence/current-blocker.md` and
`target/mantle-self-rust-plan-probe-before-native-host-dependency-producer/blocker-summary.txt`:

```text
blocker classes:
      1 missing-host-dependency-producer

topology blocker:
- missing-host-dependency-producer: no supported target producer lib unit for host dependency package registry+https://github.com/rust-lang/crates.io-index#rustversion@1.0.22
```

Post-implementation probe is pueue task `20`, output directory
`target/mantle-self-rust-plan-probe-after-native-host-dependency-producer-current/`.

```text
probe_status=0
topology_execution=blocked
source_closure=true
native_registry_source_planning=true
native_git_source_planning=true
native_package_target_planning=true
native_unit_graph_planning=true
native_host_unit_graph_planning=true
unit_derivation_graph=true
topology_unit_executions=2
metadata_runs=0

blocker classes:
      2 rustc-failed

topology blocker:
- rustc-failed: error: linking with `/nix/store/97vplpbajnr7x03fqh9biz5v6960sv22-clang-wrapper-21.1.8/bin/cc` failed: exit status: 1
```

The original `missing-host-dependency-producer` blocker for
`rustversion@1.0.22` is gone. The remaining blocker is a topology execution
environment failure from clang wrapper lookup of `bash` inside the hermetic
rustc child environment, matching the existing Mantle rust-plan test gotcha.

## Archive note

`cairn archive native-host-dependency-producer-coverage --execute` created
`cairn/archive/1970-01-01-native-host-dependency-producer-coverage`; this was
manually renamed to
`cairn/archive/2026-05-27-native-host-dependency-producer-coverage` per the repo
Cairn gotcha. `cairn validate` passed after the rename.

## Decision

Implementation satisfies this change: selected host-unit dependency artifacts now
recognize supported proc-macro host producers, target-library host dependency
producers remain supported, and unsupported host dependency producer shapes still
fail closed with `missing-host-dependency-producer`.
