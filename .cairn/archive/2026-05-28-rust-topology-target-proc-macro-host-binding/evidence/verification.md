# Verification

Task-ID: V1
Covers: rust_package_planning.native_target_proc_macro_host_extern_binding

## Question

Does native target proc-macro host extern binding move the topology past the `darling@0.20.11` sysroot `darling_macro` blocker while preserving fail-closed host-artifact checks?

## Inspected evidence

- Implementation: `src/rust_plan.rs`
- Focused test command: `cargo test -p mantle --bin mantle bind_all_host_artifacts`
- Focused test command: `cargo test -p mantle --bin mantle bind_target_unit_artifacts`
- Full focused suite command: `cargo test -p mantle --bin mantle rust_plan::`
- Dirty self-probe artifact: `target/mantle-self-rust-plan-probe-after-target-proc-macro-host-binding-dirty2/receipt.json`
- Dirty self-probe summary: `target/mantle-self-rust-plan-probe-after-target-proc-macro-host-binding-dirty2/blocker-summary.txt`
- Clean self-probe artifact: `target/mantle-self-rust-plan-probe-after-4b957b2c-clean/receipt.json`
- Clean self-probe summary: `target/mantle-self-rust-plan-probe-after-4b957b2c-clean/blocker-summary.txt`

## Decision

Accepted for implementation checkpoint. Mantle now binds consumed proc-macro host artifacts into target `--extern` args, avoids adding `--extern` for custom-build host artifacts, avoids duplicate proc-macro externs when a placeholder already exists, and adds produced proc-macro host artifact directories as dependency search paths for downstream rlib metadata.

Focused tests passed:

```text
running 3 tests
test rust_plan::tests::bind_all_host_artifacts_rewrites_existing_proc_macro_placeholder_without_duplicate ... ok
test rust_plan::tests::bind_all_host_artifacts_does_not_add_extern_for_custom_build_artifact ... ok
test rust_plan::tests::bind_all_host_artifacts_adds_proc_macro_extern_without_dependency_placeholder ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 502 filtered out; finished in 0.00s

running 1 test
test rust_plan::tests::bind_target_unit_artifacts_adds_proc_macro_search_path_for_transitive_metadata ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 505 filtered out; finished in 0.00s
```

The broader rust-plan suite passed:

```text
running 79 tests
...
test result: ok. 79 passed; 0 failed; 0 ignored; 0 measured; 427 filtered out; finished in 0.03s
```

The clean self-probe moved past the prior `darling@0.20.11` sysroot `darling_macro` blocker. `darling@0.20.11` now builds successfully, and the next frontier is Cargo cap-lints parity for registry dependencies (`derive_builder_core` warning promoted to error by crate lints):

```text
probe: target/mantle-self-rust-plan-probe-after-4b957b2c-clean/receipt.json
head: 4b957b2c2492386e57cf8fa32038e2e26d00bfc0
git_status_short_bytes=0

probe_status=0
topology_execution=blocked
topology_unit_executions=53
metadata_runs=21

darling related unit executions:
registry+https://github.com/rust-lang/crates.io-index#darling_core@0.20.11 target=darling_core kind=lib status=success reason=rebuilt-explicit-unit
registry+https://github.com/rust-lang/crates.io-index#darling_macro@0.20.11 target=darling_macro kind=proc-macro status=success reason=rebuilt-explicit-unit
registry+https://github.com/rust-lang/crates.io-index#darling_core@0.23.0 target=darling_core kind=lib status=success reason=rebuilt-explicit-unit
registry+https://github.com/rust-lang/crates.io-index#darling_macro@0.23.0 target=darling_macro kind=proc-macro status=success reason=rebuilt-explicit-unit
registry+https://github.com/rust-lang/crates.io-index#darling@0.20.11 target=darling kind=lib status=success reason=rebuilt-explicit-unit

derive builder related unit executions:
registry+https://github.com/rust-lang/crates.io-index#derive_builder_core@0.20.2 target=derive_builder_core kind=lib status=failed reason=rustc-exit-nonzero

blocker classes:
      1 rustc-failed

topology blocker:
- rustc-failed: error: hiding a lifetime that's elided elsewhere is confusing
  --> /home/brittonr/git/mantle/vendor-deps/derive_builder_core/src/macro_options/darling_opts.rs:28:35
```

## Owner

Mantle agent.

## Next action

Archive this change after validation, then address the next topology frontier: Cargo-compatible `--cap-lints allow` for non-local registry dependencies.
