# Verification

## Baseline

- pueue task `40`: `cargo test -p mantle --bin mantle build_script_child_env_ -- --nocapture`
  - Result: `2 passed; 0 failed` before core changes.
  - Baseline included the reviewed target-derived expectation that this change replaces.

## Focused tests after implementation

- pueue task `42`: `cargo test -p mantle --bin mantle build_script_child_env_ -- --nocapture`
  - Result: `2 passed; 0 failed`.
  - Covers explicit package-name env and fallback behavior without package-name env data.
- pueue task `43`: `cargo test -p mantle --bin mantle native_host_derivation_carries_manifest_package_name_for_build_script_env -- --nocapture`
  - Result: `1 passed; 0 failed`.
  - Covers native host derivation carrying hyphenated manifest package name into build-script env.

## Formatting and Cairn validation

- `cargo fmt --check -p mantle -v`: passed before commit `eb0760a7`.
- `nix run path:/home/brittonr/git/cairn#cairn -- validate --root /home/brittonr/git/mantle`: passed before commit `eb0760a7`.
- `git diff --check`: passed before commit `eb0760a7`.

## Self-probe oracle checkpoint

- **Question:** Does the committed package-name env implementation keep the build-script topology frontier stable while proving custom-build units now carry package-derived `CARGO_PKG_NAME` values?
- **Inspected evidence:** pueue task `44`, `target/mantle-self-rust-plan-probe-after-eb0760a7-clean/receipt.json`, `target/mantle-self-rust-plan-probe-after-eb0760a7-clean/status.txt`, `target/mantle-self-rust-plan-probe-after-eb0760a7-clean/git-status-short.txt`, and `target/mantle-self-rust-plan-probe-after-eb0760a7-clean/blocker-summary.txt`.
- **Decision:** The probe is tied to committed code `eb0760a7` with a clean tree. The previous build-script env frontier remains advanced: three build-script metadata runs still succeed, and custom-build derivations now show package-derived `CARGO_PKG_NAME` values, including hyphenated registry package names such as `async-io`. The remaining blocker is unchanged at the next proc-macro/dependency binding frontier.
- **Owner:** coding agent.
- **Next action:** pursue a separate bounded change for proc-macro host rustc invocation/dependency binding.

Checkpoint excerpt:

```text
probe: target/mantle-self-rust-plan-probe-after-eb0760a7-clean/receipt.json
head: eb0760a7c33c18c56eeb7ad1d4700584b45f2508
git_status_short_bytes=0

probe_status=0
topology_execution=blocked
topology_unit_executions=5
metadata_runs=3

metadata packages:
registry+https://github.com/rust-lang/crates.io-index#anyhow@1.0.102 status=success
registry+https://github.com/rust-lang/crates.io-index#assert_cmd@2.2.0 status=success
registry+https://github.com/rust-lang/crates.io-index#async-io@2.6.0 status=success

first custom-build pkg envs:
registry+https://github.com/rust-lang/crates.io-index#anyhow@1.0.102 CARGO_PKG_NAME=anyhow
registry+https://github.com/rust-lang/crates.io-index#assert_cmd@2.2.0 CARGO_PKG_NAME=assert_cmd
registry+https://github.com/rust-lang/crates.io-index#async-io@2.6.0 CARGO_PKG_NAME=async-io
registry+https://github.com/rust-lang/crates.io-index#atomic-polyfill@1.0.3 CARGO_PKG_NAME=atomic-polyfill
registry+https://github.com/rust-lang/crates.io-index#aws-lc-rs@1.16.2 CARGO_PKG_NAME=aws-lc-rs

blocker classes:
      2 rustc-failed

topology blocker:
- rustc-failed: error[E0432]: unresolved import `proc_macro`
 --> /home/brittonr/git/mantle/vendor-deps/async-stream-impl/src/lib.rs:1:5
```
