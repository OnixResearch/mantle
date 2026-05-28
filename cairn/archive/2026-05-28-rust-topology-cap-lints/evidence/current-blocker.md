# Current Blocker

Source: `target/mantle-self-rust-plan-probe-after-4b957b2c-clean/blocker-summary.txt`.

## Summary

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
registry+https://github.com/rust-lang/crates.io-index#darling@0.20.11 target=darling kind=lib status=success reason=rebuilt-explicit-unit

derive builder related unit executions:
registry+https://github.com/rust-lang/crates.io-index#derive_builder_core@0.20.2 target=derive_builder_core kind=lib status=failed reason=rustc-exit-nonzero

blocker classes:
      1 rustc-failed

topology blocker:
- rustc-failed: error: hiding a lifetime that's elided elsewhere is confusing
  --> /home/brittonr/git/mantle/vendor-deps/derive_builder_core/src/macro_options/darling_opts.rs:28:35
```

## Interpretation

The failed unit is registry-backed dependency code. Cargo normally invokes dependency rustc with `--cap-lints allow`, preventing dependency warnings from failing downstream builds. Mantle's direct rustc args do not yet model that cap.
