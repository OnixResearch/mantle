# Verification

Task-ID: V1
Covers: rust_package_planning.native_dependency_cap_lints

## Question

Does native dependency cap-lints parity move topology past the `derive_builder_core@0.20.2` dependency warning-as-error blocker while keeping local path crates uncapped?

## Inspected evidence

Implementation commit: `4621b5bad1ec9a911ceec85c0eb3f26f805ce6f3` (`match native dependency lint and host metadata parity`).

Focused tests, run with:

```sh
env SNIX_BUILD_SANDBOX_SHELL=/nix/store/8mf4s8c4xjvlkj12p299qylrb30g7zzh-busybox-static-x86_64-unknown-linux-musl-1.37.0/bin/busybox \
    CARGO_TARGET_DIR=/tmp/mantle-cap-lints-target \
    CARGO_INCREMENTAL=0 \
    nix develop -c cargo test -p mantle --bin mantle <test-name> -- --nocapture
```

Passing tests:

- `rust_plan::tests::native_unit_derivation_caps_lints_for_registry_and_git_sources` — `test result: ok. 1 passed; 0 failed`.
- `rust_plan::tests::native_unit_derivation_leaves_path_sources_uncapped` — `test result: ok. 1 passed; 0 failed`.
- `rust_plan::tests::native_host_derivation_caps_lints_for_registry_source` — `test result: ok. 1 passed; 0 failed`.
- `rust_plan::tests::native_host_dependencies_drop_build_script_artifacts_for_proc_macro_units` — `test result: ok. 1 passed; 0 failed`.
- `rust_plan::tests::combined_unit_topology_orders_host_build_script_before_same_package_proc_macro` — `test result: ok. 1 passed; 0 failed`.

Clean self-probe:

- Pueue task: `61` (`clean-cap-lints-probe`).
- Probe receipt: `target/mantle-self-rust-plan-probe-after-4621b5ba-clean/receipt.json`.
- Head: `4621b5bad1ec9a911ceec85c0eb3f26f805ce6f3`.
- Clean tree evidence: `git_status_short_bytes=0`.
- Probe command exit: `probe_status=0`.
- Topology status: `blocked` after `75` unit executions and `30` metadata runs.
- `derive_builder_core@0.20.2`: `status=success`.
- `derive_builder_macro@0.20.2`: `status=success`.
- `rustversion@1.0.22` custom-build: `status=success`.
- `rustversion@1.0.22` proc-macro: `status=success`.
- Remaining blocker: `malformed-build-script-metadata: build-script metadata line 75 is malformed: rustc-link-lib name must be a safe token`.

Cairn validation and readiness checks:

- `/home/brittonr/.cargo-target/debug/cairn validate --root .` — `valid: true`, `changes: 1`, `specs_validated: 2`, no issues.
- `/home/brittonr/.cargo-target/debug/cairn gate tasks rust-topology-cap-lints --root .` — `verdict: PASS`, `valid: true`, no issues.
- `/home/brittonr/.cargo-target/debug/cairn release-readiness --root .` — command ran; global verdict remained `fail` because `tracey_coverage` and `mcp_agent_smoke` failed (`No such file or directory`). `cairn_validate` and `determinism_coverage_audit` passed in that receipt. These failures are repository-wide release-readiness blockers, not regressions in this cap-lints change.
- `/home/brittonr/.cargo-target/debug/cairn archive rust-topology-cap-lints --root . --execute` — archived the completed change. Cairn emitted `cairn/archive/1970-01-01-rust-topology-cap-lints`; this was manually renamed to `cairn/archive/2026-05-28-rust-topology-cap-lints` per repo guidance.
- The archived `ADDED` requirement was manually synced into `cairn/specs/rust-package-planning/spec.md` after archive because this Cairn archive run only moved the change directory.
- Post-archive `/home/brittonr/.cargo-target/debug/cairn validate --root .` — `valid: true`, `changes: 0`, `specs_validated: 1`, no issues.

## Decision

Accepted. Bounded `--cap-lints allow` now applies to non-local registry/git native units, local path units remain uncapped, host and target derivations are covered, and the native self-probe moved past the prior `derive_builder_core` warning-as-error blocker. The remaining self-probe blocker is a distinct build-script metadata parser issue.

## Owner

Mantle agent.

## Next action

Track `rustc-link-lib` metadata parsing separately.
