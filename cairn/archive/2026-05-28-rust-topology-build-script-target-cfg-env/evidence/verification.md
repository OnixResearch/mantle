# Verification

Task-ID: V1
Covers: rust_package_planning.native_build_script_target_cfg_env

## Question

Does native Rust topology now provide deterministic Cargo target cfg env variables to build-script runtime, and does the self-probe move past the `aws-lc-sys` missing `CARGO_CFG_TARGET_ARCH` blocker?

## Inspected evidence

- Implementation: `src/rust_plan.rs`
- Focused test command: `cargo test -p mantle --bin mantle rust_plan::`
- Dirty self-probe artifact: `target/mantle-self-rust-plan-probe-after-target-cfg-dirty/receipt.json`
- Dirty self-probe summary: `target/mantle-self-rust-plan-probe-after-target-cfg-dirty/blocker-summary.txt`
- Clean self-probe artifact: `target/mantle-self-rust-plan-probe-after-c9bf3938-clean/receipt.json`
- Clean self-probe summary: `target/mantle-self-rust-plan-probe-after-c9bf3938-clean/blocker-summary.txt`
- Cairn validation command: `cairn validate --root .`
- Whitespace check: `git diff --check`

## Decision

Accepted. Native build-script runtime env now derives bounded `CARGO_CFG_TARGET_*` values from the selected target triple and passes them through `build_script_child_env()`. Focused tests cover x86_64 Linux values, wasm unknown empty target-env behavior, and child env pass-through.

Focused tests passed:

```text
running 72 tests
...
test rust_plan::tests::build_script_target_cfg_env_derives_x86_64_linux_values ... ok
test rust_plan::tests::build_script_target_cfg_env_uses_empty_env_for_wasm_unknown ... ok
...
test result: ok. 72 passed; 0 failed; 0 ignored; 0 measured; 427 filtered out; finished in 0.03s
```

The committed clean self-probe moved past the prior `aws-lc-sys` missing `CARGO_CFG_TARGET_ARCH` runtime failure:

```text
probe: target/mantle-self-rust-plan-probe-after-c9bf3938-clean/receipt.json
head: c9bf393824242dfb7a05d5f3a0efa9fd29e3132d
git_status_short_bytes=0

probe_status=0
topology_execution=blocked
topology_unit_executions=30
metadata_runs=12

aws-lc related unit executions:
registry+https://github.com/rust-lang/crates.io-index#aws-lc-sys@0.39.1 target=build-script-build kind=custom-build status=success

blocker classes:
      1 build-script-run-failed

topology blocker:
- build-script-run-failed:

error occurred in cc-rs: environment variable OPT_LEVEL not defined
```

Post-archive Cairn validation passed with no active changes:

```json
{
  "change_issues": [],
  "changes": 0,
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "spec_issues": [],
  "specs_validated": 1,
  "valid": true
}
```

`git diff --check` produced no output.

## Owner

Mantle agent.

## Next action

Next topology frontier is native build-script cc-rs profile env parity (`OPT_LEVEL` and likely adjacent Cargo build-script env vars).
