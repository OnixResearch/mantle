# Verification: Rust Topology External Linker Mode

Task-ID: rust-topology-external-linker.V1
Covers: r[rust_package_planning.rust_topology_external_linker]
Date: 2026-05-27
Decision owner: coding agent

## Focused tests

Original pueue task `32` covered the initial three runtime-arg tests before
review remediation. The final four-test evidence below is from the direct rerun
after adding `rust_topology_runtime_args_ignore_near_match_linker_mode`.

Command:

```sh
PATH=/run/current-system/sw/bin:/home/brittonr/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin:/nix/store/97vplpbajnr7x03fqh9biz5v6960sv22-clang-wrapper-21.1.8/bin:/nix/store/rvp7qlpf5jqvdckjy1afjb6aha6j8dxg-pkg-config-wrapper-0.29.2/bin:/etc/profiles/per-user/brittonr/bin:/usr/bin:/bin \
PKG_CONFIG_PATH=/nix/store/1l5jgzy26hkjz1y3apn1051asvn42sfn-openssl-3.6.1-dev/lib/pkgconfig \
SNIX_BUILD_SANDBOX_SHELL=/nix/store/8mf4s8c4xjvlkj12p299qylrb30g7zzh-busybox-static-x86_64-unknown-linux-musl-1.37.0/bin/busybox \
RUSTC_WRAPPER= \
CARGO_TARGET_DIR=target/rust-topology-external-linker-check \
cargo test -p mantle --bin mantle rust_topology_runtime_args -- --nocapture
```

Result:

```text
running 4 tests
test rust_plan::tests::rust_topology_runtime_args_ignore_near_match_linker_mode ... ok
test rust_plan::tests::rust_topology_runtime_args_disable_self_contained_linker_by_default ... ok
test rust_plan::tests::rust_topology_runtime_args_preserve_joined_explicit_linker_mode ... ok
test rust_plan::tests::rust_topology_runtime_args_preserve_split_explicit_linker_mode ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 472 filtered out; finished in 0.00s
```

## Self-probe blocker movement

Baseline evidence: `evidence/current-blocker.md`, copied from clean pueue task `31` at `58d9073d`.

```text
blocker classes:
      2 rustc-failed

topology blocker:
- rustc-failed: ...
  = note: /home/brittonr/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/lib/rustlib/x86_64-unknown-linux-gnu/bin/gcc-ld/ld.lld: line 5: /nix/store/3jmhpvfwqag9jcxi21l2lrv79ha7h4p5-rustup-1.29.0/nix-support/ld-wrapper.sh: No such file or directory
```

Post-implementation probe: pueue task `33`, output directory
`target/mantle-self-rust-plan-probe-after-external-linker/`.

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
metadata_runs=1

blocker classes:
      2 rustc-failed

topology blocker:
- rustc-failed: error: let chains are only allowed in Rust 2024 or later
```

The stale rustup `ld-wrapper.sh` blocker is gone. Topology execution now builds
and runs the first custom-build unit (`fuse-backend-rs`) and reaches the next
Rust source/edition blocker in `vendor/nix-compat-derive`.

## Review remediation

Same-family review found `link-self-contained` detection was prefix-based. The
fix now recognizes only exact `link-self-contained` and
`link-self-contained=...` codegen options. The near-match regression
`rust_topology_runtime_args_ignore_near_match_linker_mode` proves
`link-self-containedness=...` does not suppress default
`-C link-self-contained=no` injection.

## Final self-probe oracle checkpoint

- **Question:** Does the current committed external-linker implementation still
  move topology execution past the stale rustup `ld-wrapper.sh` blocker, and was
  the probe run from a clean tree?
- **Inspected evidence:** pueue task `14`,
  `target/mantle-self-rust-plan-probe-after-b3ed3a8e-clean/receipt.json`,
  `target/mantle-self-rust-plan-probe-after-b3ed3a8e-clean/status.txt`,
  `target/mantle-self-rust-plan-probe-after-b3ed3a8e-clean/git-status-short.txt`,
  and the repo-local summary
  `target/mantle-self-rust-plan-probe-after-b3ed3a8e-clean/blocker-summary.txt`.
- **Decision:** The probe is tied to committed code `b3ed3a8e` with a clean
  tree, and it verifies blocker movement from stale rustup `ld-wrapper.sh` to
  the next Rust-edition source issue. This follow-up evidence commit changes
  only review documentation, so the `b3ed3a8e` implementation probe remains the
  relevant execution checkpoint for the code under test.
- **Owner:** coding agent.
- **Next action:** pursue the new deterministic blocker separately by planning
  how native topology should handle Rust 2024-only let-chain syntax in
  `vendor/nix-compat-derive`, or record it as the next known topology frontier
  if out of scope.

Checkpoint excerpt:

```text
probe: target/mantle-self-rust-plan-probe-after-b3ed3a8e-clean/receipt.json
head: b3ed3a8e415581d41b2f4896da7f2d51ed13001a
git_status_short_bytes=0

probe_status=0
topology_execution=blocked
topology_unit_executions=2
metadata_runs=1

blocker classes:
      2 rustc-failed

topology blocker:
- rustc-failed: error: let chains are only allowed in Rust 2024 or later
  --> ./vendor/nix-compat-derive/src/de.rs:39:8
```

## Decision

Implementation satisfies this change: runtime topology args disable rustc
self-contained linker wrappers by default while preserving explicit selected-unit
`link-self-contained` settings.

## Archive note

`cairn archive rust-topology-external-linker --execute` created
`cairn/archive/1970-01-01-rust-topology-external-linker`; this was manually
renamed to `cairn/archive/2026-05-27-rust-topology-external-linker`. `cairn
validate` passed after the rename.
