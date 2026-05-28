# Verification

Task-ID: V1
Covers: rust_package_planning.native_build_script_profile_env

## Question

Does native Rust topology now provide deterministic Cargo profile env variables to build-script runtime, and does the self-probe move past the `aws-lc-sys` missing `OPT_LEVEL` blocker?

## Inspected evidence

- Implementation: `src/rust_plan.rs`
- Focused test command: `cargo test -p mantle --bin mantle rust_plan::`
- Dirty self-probe artifact: `target/mantle-self-rust-plan-probe-after-profile-env-dirty/receipt.json`
- Dirty self-probe summary: `target/mantle-self-rust-plan-probe-after-profile-env-dirty/blocker-summary.txt`

## Decision

Accepted for implementation checkpoint. Native build-script runtime env now derives bounded `OPT_LEVEL`, `DEBUG`, and `NUM_JOBS` values from the selected profile and passes them through `build_script_child_env()`. Focused tests cover dev/release profile env values and child env pass-through.

Focused tests passed:

```text
running 73 tests
...
test rust_plan::tests::build_script_profile_env_derives_dev_and_release_defaults ... ok
...
test result: ok. 73 passed; 0 failed; 0 ignored; 0 measured; 427 filtered out; finished in 0.03s
```

The dirty self-probe moved past the prior `aws-lc-sys` missing `OPT_LEVEL` runtime failure:

```text
probe: target/mantle-self-rust-plan-probe-after-profile-env-dirty/receipt.json
head: f4d19ee47de8f7a69b8c42698a0f1d20f4ec4199
git_status_short_bytes=20

probe_status=0
topology_execution=blocked
topology_unit_executions=52
metadata_runs=21

aws-lc related unit executions:
registry+https://github.com/rust-lang/crates.io-index#aws-lc-sys@0.39.1 target=build-script-build kind=custom-build status=success
registry+https://github.com/rust-lang/crates.io-index#aws-lc-rs@1.16.2 target=build-script-build kind=custom-build status=success

blocker classes:
      1 rustc-failed

topology blocker:
- rustc-failed: error[E0658]: use of unstable library feature `rustc_private`: this crate is being loaded from the sysroot, an unstable location; did you mean to load this crate from crates.io via `Cargo.toml` instead?
  --> /home/brittonr/git/mantle/vendor-deps/darling-0.20.11/src/lib.rs:78:1
   |
78 | extern crate darling_macro;
   | ^^^^^^^^^^^^^^^^^^^^^^^^^^^
```

## Owner

Mantle agent.

## Next action

Commit profile env implementation, rerun clean self-probe from committed HEAD, then archive the Cairn change if validation stays clean. Next topology frontier is direct rustc proc-macro dependency binding for `darling` / `darling_macro`.
