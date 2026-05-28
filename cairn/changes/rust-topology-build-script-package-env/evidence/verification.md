# Verification

Task-ID: V1
Covers: rust_package_planning.native_build_script_package_metadata_env

## Question

Does native Rust topology now provide Cargo package metadata environment variables for custom-build compilation and runtime, and does the self-probe move past the `aws-lc-sys` missing `CARGO_PKG_VERSION` blocker?

## Inspected evidence

- Implementation: `src/rust_plan.rs`
- Focused test command: `cargo test -p mantle --bin mantle rust_plan::`
- Self-probe artifact: `target/mantle-self-rust-plan-probe-after-package-env-dirty/receipt.json`
- Self-probe summary: `target/mantle-self-rust-plan-probe-after-package-env-dirty/blocker-summary.txt`
- Cairn validation command: `cairn validate --root .`
- Whitespace check: `git diff --check`

## Decision

Accepted. Native package facts now carry bounded `CARGO_PKG_*` values, native rustc derivations receive those values, and build-script child execution passes them through. Focused tests cover version components, empty optional metadata defaults, compile-time derivation env, and runtime child env.

The dirty self-probe moved past the prior `aws-lc-sys` compile failure:

```text
probe: target/mantle-self-rust-plan-probe-after-package-env-dirty/receipt.json
head: bdeb06a5472b38ede18fe00b953715cbbcd8f5f6
git_status_short_bytes=20

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
thread 'main' (949298) panicked at /home/brittonr/git/mantle/vendor-deps/aws-lc-sys/builder/main.rs:160:39:
missing env var "CARGO_CFG_TARGET_ARCH"
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

Focused tests passed:

```text
running 69 tests
...
test result: ok. 69 passed; 0 failed; 0 ignored; 0 measured; 427 filtered out; finished in 0.03s
```

Cairn validation passed before final task closure:

```json
{
  "change_issues": [],
  "changes": 1,
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "spec_issues": [],
  "specs_validated": 2,
  "valid": true
}
```

`git diff --check` produced no output.

## Owner

Mantle agent.

## Next action

Commit package-env implementation, rerun clean self-probe from committed HEAD, then archive the Cairn change if review stays clean. Next topology frontier is native build-script `CARGO_CFG_*` target cfg runtime env parity.
