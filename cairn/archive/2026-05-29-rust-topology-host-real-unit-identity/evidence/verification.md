# Verification Evidence

Task-ID: V1
Covers: rust_package_planning.native_host_real_unit_identity

## Focused tests

- `CARGO_TARGET_DIR=target/host-identity-fix cargo test -p mantle --bin mantle native_host -- --nocapture`
  - Result: 13 passed, 0 failed.
- `CARGO_TARGET_DIR=target/host-identity-fix cargo test -p mantle --bin mantle unit_variant -- --nocapture`
  - Result: 5 passed, 0 failed.
- `CARGO_TARGET_DIR=target/host-identity-fix cargo test -p mantle --bin mantle native_unit_graph -- --nocapture`
  - Result: 10 passed, 0 failed.
- `CARGO_TARGET_DIR=target/host-identity-fix cargo test -p mantle --bin mantle combined_unit_topology -- --nocapture`
  - Result: 7 passed, 0 failed.
- `CARGO_TARGET_DIR=target/host-identity-fix cargo test -p mantle --bin mantle bind_all_host_artifacts_with_index_blocks_ambiguous_package_only_host_candidates -- --nocapture`
  - Result: 1 passed, 0 failed.
- `CARGO_TARGET_DIR=target/host-identity-fix cargo test -p mantle --bin mantle bind_build_script_metadata_with_index_blocks_ambiguous_package_only_metadata_candidates -- --nocapture`
  - Result: 1 passed, 0 failed.

## Cairn validation

- `/nix/store/vdn5zw29nd97ba6dzmfhp5vz6zbm1mkx-cairn-0.1.0/bin/cairn validate --root .`
  - Pre-archive result: `valid: true`, `changes: 8`, `specs_validated: 9`.
- `/nix/store/vdn5zw29nd97ba6dzmfhp5vz6zbm1mkx-cairn-0.1.0/bin/cairn archive rust-topology-host-real-unit-identity --root . --execute`
  - Tool created `cairn/archive/1970-01-01-rust-topology-host-real-unit-identity`; manually renamed to `cairn/archive/2026-05-29-rust-topology-host-real-unit-identity`.
  - Archive only moved the change directory, so the ADDED requirement was manually copied into `cairn/specs/rust-package-planning/spec.md`.
- `/nix/store/vdn5zw29nd97ba6dzmfhp5vz6zbm1mkx-cairn-0.1.0/bin/cairn validate --root .`
  - Post-archive result: `valid: true`, `changes: 7`, `specs_validated: 8`.

## Dirty self-probe

Pueue task: 292 (`host-real-unit-identity-dirty9-self-probe`).

Summary from `target/mantle-self-rust-plan-probe-host-real-unit-identity-dirty9/blocker-summary.txt`:

```text
probe: target/mantle-self-rust-plan-probe-host-real-unit-identity-dirty9/receipt.json
topology_execution_status=success
executions=610
blocker_class=
blocker_message=
```

## Clean self-probe

Pueue task: 294 (`host-real-unit-identity-clean-self-probe`).

Summary from `target/mantle-self-rust-plan-probe-host-real-unit-identity-clean/blocker-summary.txt`:

```text
probe: target/mantle-self-rust-plan-probe-host-real-unit-identity-clean/receipt.json
topology_execution_status=success
executions=610
blocker_class=
blocker_message=
```
