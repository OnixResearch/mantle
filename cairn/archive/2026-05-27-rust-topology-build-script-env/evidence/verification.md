# Verification

## Baseline

- pueue task `32`: `cargo test -p mantle --bin mantle unit_derivation_graph_represents_build_script_host_units -- --nocapture`
  - Result: `1 passed; 0 failed` before core changes.

## Focused tests after implementation

- pueue task `38`: `cargo test -p mantle --bin mantle build_script_child_env_ -- --nocapture`
  - Result: `2 passed; 0 failed`.
  - Covers positive deterministic env/cwd planning and negative missing-source behavior.
- pueue task `34`: `cargo test -p mantle --bin mantle unit_derivation_graph_represents_build_script_host_units -- --nocapture`
  - Result: `1 passed; 0 failed` after implementation.

## Formatting and Cairn validation

- `cargo fmt --check -p mantle -v`: passed before commit `514fff33`.
- `nix run path:/home/brittonr/git/cairn#cairn -- validate --root /home/brittonr/git/mantle`: passed before commit `514fff33`.
- `git diff --check`: passed before commit `514fff33`.

## Self-probe oracle checkpoint

- **Question:** Does the committed build-script env implementation move topology execution past the missing `$RUSTC` build-script blocker while preserving a clean probe tree?
- **Inspected evidence:** pueue task `39`, `target/mantle-self-rust-plan-probe-after-514fff33-clean/receipt.json`, `target/mantle-self-rust-plan-probe-after-514fff33-clean/status.txt`, `target/mantle-self-rust-plan-probe-after-514fff33-clean/git-status-short.txt`, and `target/mantle-self-rust-plan-probe-after-514fff33-clean/blocker-summary.txt`.
- **Decision:** The probe is tied to committed code `514fff33` with a clean tree. The previous missing `$RUSTC` build-script blocker is gone: three build-script metadata runs now succeed (`anyhow`, `assert_cmd`, and `async-io`). Topology now stops at the next deterministic proc-macro/dependency binding blocker for `async-stream-impl` (`proc_macro` / `proc_macro2` unresolved imports).
- **Owner:** coding agent.
- **Next action:** pursue a separate bounded change for proc-macro host rustc invocation/dependency binding.

Checkpoint excerpt:

```text
probe: target/mantle-self-rust-plan-probe-after-514fff33-clean/receipt.json
head: 514fff33b11ae16eab664d7ab47eefe039e790b7
git_status_short_bytes=0

probe_status=0
topology_execution=blocked
topology_unit_executions=5
metadata_runs=3

metadata packages:
registry+https://github.com/rust-lang/crates.io-index#anyhow@1.0.102 status=success
registry+https://github.com/rust-lang/crates.io-index#assert_cmd@2.2.0 status=success
registry+https://github.com/rust-lang/crates.io-index#async-io@2.6.0 status=success

blocker classes:
      2 rustc-failed

topology blocker:
- rustc-failed: error[E0432]: unresolved import `proc_macro`
 --> /home/brittonr/git/mantle/vendor-deps/async-stream-impl/src/lib.rs:1:5
```
