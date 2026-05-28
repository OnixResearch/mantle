# Verification

Task-ID: V1
Covers: rust_package_planning.native_link_lib_metadata

## Question

Does bounded `rustc-link-lib` metadata parsing move the native rust-plan topology past the previous malformed link-lib parser blocker while preserving fail-closed rejection for unsafe values?

## Inspected evidence

Implementation commit: `1c1952e16ace7283d74e185b95b6ad771d3e99fd` (`parse native link metadata conservatively`).

Baseline before implementation:

```sh
env SNIX_BUILD_SANDBOX_SHELL=/nix/store/8mf4s8c4xjvlkj12p299qylrb30g7zzh-busybox-static-x86_64-unknown-linux-musl-1.37.0/bin/busybox \
    CARGO_TARGET_DIR=/tmp/mantle-link-lib-target \
    CARGO_INCREMENTAL=0 \
    nix develop -c cargo test -p mantle --bin mantle parse_build_script_metadata -- --nocapture
```

Baseline result: `test result: ok. 2 passed; 0 failed`.

Focused post-change tests:

```sh
env SNIX_BUILD_SANDBOX_SHELL=/nix/store/8mf4s8c4xjvlkj12p299qylrb30g7zzh-busybox-static-x86_64-unknown-linux-musl-1.37.0/bin/busybox \
    CARGO_TARGET_DIR=/tmp/mantle-link-lib-target \
    CARGO_INCREMENTAL=0 \
    nix develop -c cargo test -p mantle --bin mantle parse_build_script_metadata -- --nocapture
```

Post-change result: `test result: ok. 4 passed; 0 failed`.

Broader metadata parser/binding focus:

```sh
env SNIX_BUILD_SANDBOX_SHELL=/nix/store/8mf4s8c4xjvlkj12p299qylrb30g7zzh-busybox-static-x86_64-unknown-linux-musl-1.37.0/bin/busybox \
    CARGO_TARGET_DIR=/tmp/mantle-link-lib-target \
    CARGO_INCREMENTAL=0 \
    nix develop -c cargo test -p mantle --bin mantle build_script_metadata -- --nocapture
```

Result: `test result: ok. 5 passed; 0 failed`.

Dirty self-probe:

- Pueue task: `60` (`dirty-link-lib-metadata-probe`).
- Probe receipt: `target/mantle-self-rust-plan-probe-link-lib-dirty/receipt.json`.
- Probe command exit: `probe_status=0`.
- Topology status: `blocked` after `80` unit executions and `33` metadata runs.
- `aws-lc-sys@0.39.1` metadata run: `status=success`, `libs=static=aws_lc_0_39_1_crypto`.
- `aws-lc-rs@1.16.2` metadata run: `status=success`.
- Remaining blocker: `rustc-failed` compiling `jiff-static@0.2.23` because `quote` and `syn` are unresolved.

Clean self-probe:

- Pueue task: `61` (`clean-link-lib-metadata-probe`).
- Probe receipt: `target/mantle-self-rust-plan-probe-after-1c1952e1-clean/receipt.json`.
- Head: `1c1952e16ace7283d74e185b95b6ad771d3e99fd`.
- Clean tree evidence: `git_status_short_bytes=0`.
- Probe command exit: `probe_status=0`.
- Topology status: `blocked` after `80` unit executions and `33` metadata runs.
- `aws-lc-sys@0.39.1` metadata run: `status=success`, `libs=static=aws_lc_0_39_1_crypto`.
- `aws-lc-rs@1.16.2` metadata run: `status=success`.
- Previous malformed `rustc-link-lib` parser blocker is absent.
- Remaining blocker: `rustc-failed` compiling `jiff-static@0.2.23` with unresolved `quote` and `syn` imports.

Cairn validation and readiness checks:

- `/home/brittonr/.cargo-target/debug/cairn validate --root .` — `valid: true`, `changes: 1`, `specs_validated: 2`, no issues.
- `/home/brittonr/.cargo-target/debug/cairn gate tasks rust-topology-link-lib-metadata --root .` — `verdict: PASS`, `valid: true`, no issues.
- `/home/brittonr/.cargo-target/debug/cairn release-readiness --root .` — command ran; global verdict remained `fail` because `tracey_coverage` and `mcp_agent_smoke` failed (`No such file or directory`). `cairn_validate` and `determinism_coverage_audit` passed in that receipt. These failures are repository-wide release-readiness blockers, not regressions in this link-lib metadata change.
- `/home/brittonr/.cargo-target/debug/cairn sync rust-topology-link-lib-metadata --root . --execute` — synced `rust_package_planning.native_link_lib_metadata` into `cairn/specs/rust-package-planning/spec.md`.
- `/home/brittonr/.cargo-target/debug/cairn archive rust-topology-link-lib-metadata --root . --execute` — archived the completed change. Cairn emitted `cairn/archive/1970-01-01-rust-topology-link-lib-metadata`; this was manually renamed to `cairn/archive/2026-05-28-rust-topology-link-lib-metadata` per repo guidance.
- Post-archive `/home/brittonr/.cargo-target/debug/cairn validate --root .` — `valid: true`, `changes: 0`, `specs_validated: 1`, no issues.

## Decision

Accepted. Mantle now accepts safe `+` link names such as `stdc++` plus bounded kind/modifier forms such as `static:+whole-archive,-bundle=crypto_core`, still rejects unsupported or unsafe link metadata, and the clean self-probe moved past the prior `rustc-link-lib name must be a safe token` parser blocker.

## Owner

Mantle agent.

## Next action

Track the `jiff-static` unresolved `quote`/`syn` dependency binding frontier separately.
