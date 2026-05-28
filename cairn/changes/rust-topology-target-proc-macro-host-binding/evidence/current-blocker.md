# Current Blocker

Source: `target/mantle-self-rust-plan-probe-after-689b44c4-clean/blocker-summary.txt` and receipt inspection of `target/mantle-self-rust-plan-probe-after-689b44c4-clean/receipt.json`.

## Summary

```text
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

## Receipt detail

`darling@0.20.11` already has a planned consumed proc-macro host artifact:

```text
registry+https://github.com/rust-lang/crates.io-index#darling@0.20.11 target=darling kind=lib execution=target
hosts=registry+https://github.com/rust-lang/crates.io-index#darling_macro@0.20.11:darling_macro=host-artifact:539:proc-macro:darling_macro
```

But its dependency artifacts and rustc args include only `darling_core`; no `--extern darling_macro=...` is present before rustc execution. Rustc therefore resolves `extern crate darling_macro` from the sysroot and fails closed.
