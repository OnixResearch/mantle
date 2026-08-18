# Verification

## Baseline

- pueue task `14`: `cargo test -p mantle --bin mantle native_package_target_fragment_matches_supported_path_workspace -- --nocapture`
  - Result: `1 passed; 0 failed` before core changes.

## Focused tests after implementation

- pueue task `26`: `cargo test -p mantle --bin mantle native_manifest_ -- --nocapture`
  - Result: `4 passed; 0 failed`.
  - Covers declared `edition = "2024"`, workspace-inherited `edition.workspace = true`, missing edition default, and host proc-macro derivation args.
- pueue task `27`: `cargo test -p mantle --bin mantle native_unit_graph_ -- --nocapture`
  - Result: `8 passed; 0 failed`.
- pueue task `28`: `cargo test -p mantle --bin mantle host_dependency_topology_ -- --nocapture`
  - Result: `3 passed; 0 failed`.
- pueue task `30`: `cargo test -p mantle --bin mantle native_package_target_fragment_ -- --nocapture`
  - Result: `2 passed; 0 failed`.

## Formatting and Cairn validation

- `cargo fmt --check -p mantle -v`: passed after formatting.
- `nix run path:/home/brittonr/git/cairn#cairn -- validate --root /home/brittonr/git/mantle`: passed before commit `50bae139`.
- `git diff --check`: passed before commit `50bae139`.

## Self-probe oracle checkpoint

- **Question:** Does the committed native edition implementation move topology execution past the Rust 2024 let-chain blocker while preserving a clean probe tree?
- **Inspected evidence:** pueue task `29`, `target/mantle-self-rust-plan-probe-after-50bae139-clean/receipt.json`, `target/mantle-self-rust-plan-probe-after-50bae139-clean/status.txt`, `target/mantle-self-rust-plan-probe-after-50bae139-clean/git-status-short.txt`, and `target/mantle-self-rust-plan-probe-after-50bae139-clean/blocker-summary.txt`.
- **Decision:** The probe is tied to committed code `50bae139` with a clean tree. Native package/target planning and native host-unit graph planning are ready, `nix-compat-derive` now carries `edition=2024`, and the old Rust 2024 let-chain blocker is gone. Topology now stops at the next deterministic runtime environment blocker: build scripts need `RUSTC` in their execution environment.
- **Owner:** coding agent.
- **Next action:** pursue a separate bounded change for native build-script execution environment variables, starting with `$RUSTC`.

Checkpoint excerpt:

```text
probe: target/mantle-self-rust-plan-probe-after-50bae139-clean/receipt.json
head: 50bae139d502f085e3402bcfbf1a7fa46f6d2452
git_status_short_bytes=0

probe_status=0
native_package_target_ready=true
native_host_unit_graph_ready=true
topology_execution=blocked
topology_unit_executions=1
metadata_runs=0

first nix-compat-derive editions:
643:path+file:///home/brittonr/git/mantle/vendor/nix-compat-derive#0.1.0:nix-compat-derive:proc-macro:build edition=2024

blocker classes:
      1 build-script-run-failed

topology blocker:
- build-script-run-failed: Environment variable $RUSTC is not set during execution of build script
```
