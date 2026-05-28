# Verification: Rust Topology Tool PATH Environment

Task-ID: rust-topology-tool-path-env.V1
Covers: r[rust_package_planning.rust_topology_tool_path_env]
Date: 2026-05-27
Decision owner: coding agent

## Focused tests

Command: pueue task `30`

```sh
PATH=/run/current-system/sw/bin:/home/brittonr/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin:/nix/store/97vplpbajnr7x03fqh9biz5v6960sv22-clang-wrapper-21.1.8/bin:/nix/store/rvp7qlpf5jqvdckjy1afjb6aha6j8dxg-pkg-config-wrapper-0.29.2/bin:/etc/profiles/per-user/brittonr/bin:/usr/bin:/bin \
PKG_CONFIG_PATH=/nix/store/1l5jgzy26hkjz1y3apn1051asvn42sfn-openssl-3.6.1-dev/lib/pkgconfig \
SNIX_BUILD_SANDBOX_SHELL=/nix/store/8mf4s8c4xjvlkj12p299qylrb30g7zzh-busybox-static-x86_64-unknown-linux-musl-1.37.0/bin/busybox \
RUSTC_WRAPPER= \
CARGO_TARGET_DIR=target/rust-topology-tool-path-env-check \
cargo test -p mantle --bin mantle rust_topology_child_env -- --nocapture
```

Result:

```text
running 3 tests
test rust_plan::tests::rust_topology_child_env_omits_empty_inherited_path ... ok
test rust_plan::tests::rust_topology_child_env_preserves_non_empty_inherited_path ... ok
test rust_plan::tests::rust_topology_child_env_prefers_explicit_derivation_path ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 469 filtered out; finished in 0.00s
```

## Self-probe blocker movement

Baseline evidence: `evidence/current-blocker.md`, copied from clean pueue task `28` at `c3ccf629`.

```text
blocker classes:
      2 rustc-failed

topology blocker:
- rustc-failed: ...
  = note: env: 'bash': No such file or directory
          clang: error: unable to execute command: No such file or directory
```

Post-implementation probe: pueue task `29`, output directory
`target/mantle-self-rust-plan-probe-after-tool-path-env/`.

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
topology_unit_executions=1
metadata_runs=0

blocker classes:
      2 rustc-failed

topology blocker:
- rustc-failed: error: linking with `/nix/store/97vplpbajnr7x03fqh9biz5v6960sv22-clang-wrapper-21.1.8/bin/cc` failed: exit status: 1
  = note: /home/brittonr/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/lib/rustlib/x86_64-unknown-linux-gnu/bin/gcc-ld/ld.lld: line 5: /nix/store/3jmhpvfwqag9jcxi21l2lrv79ha7h4p5-rustup-1.29.0/nix-support/ld-wrapper.sh: No such file or directory
```

The original `env: 'bash': No such file or directory` blocker is gone. The
next blocker is a broken rustup linker wrapper path inside the selected toolchain,
not missing child `PATH`.

## Decision

Implementation satisfies this change: Rust topology child environments preserve
non-empty caller `PATH`, omit empty `PATH`, and keep explicit derivation
environment overrides deterministic.

## Archive note

`cairn archive rust-topology-tool-path-env --execute` created
`cairn/archive/1970-01-01-rust-topology-tool-path-env`; this was manually
renamed to `cairn/archive/2026-05-27-rust-topology-tool-path-env`. `cairn
validate` passed after the rename.
